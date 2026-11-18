//! `clipmill serve`: the engine API and MCP over HTTP, on this computer only.
//!
//! For scripts, automation tools and agents that speak HTTP rather than stdio.
//! It listens on 127.0.0.1 and nowhere else, and it is a doorway a web page in
//! the person's browser must not walk through, so:
//!
//! - every call carries a connected-agent token (`Authorization: Bearer cmt_…`);
//!   there is no anonymous access over HTTP, whatever the socket allows;
//! - `Host` must name this loopback address and port, which defeats DNS
//!   rebinding;
//! - a request with an `Origin` is refused unless that origin was allowed when
//!   the gateway started, and no CORS headers are ever sent;
//! - bodies must be JSON and at most 256 KiB, and at most 16 requests run at
//!   once.
//!
//! `POST /v2/<op>` takes the operation's arguments as the body and answers as
//! the engine does. `POST /mcp` takes one MCP JSON-RPC message.
//! `GET /v2/openapi.json` describes every operation.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use http_body_util::{BodyExt, Full, Limited};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

use crate::client::Engine;
use crate::tools::TOOLS;

pub const DEFAULT_PORT: u16 = 7428;
const MAX_BODY: usize = 256 * 1024;
const IN_FLIGHT: usize = 16;

/// How the gateway was started.
#[derive(Clone, Debug)]
pub struct Settings {
    pub socket: PathBuf,
    pub port: u16,
    /// Browser origins allowed to call, exactly as sent; empty by default.
    pub allowed_origins: Vec<String>,
}

struct Shared {
    settings: Settings,
    hosts: [String; 3],
    permits: Semaphore,
}

type Answer = Response<Full<Bytes>>;

fn respond(status: StatusCode, body: &Value) -> Answer {
    let mut response = Response::new(Full::new(Bytes::from(body.to_string())));
    *response.status_mut() = status;
    response.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    response.headers_mut().insert(
        hyper::header::CACHE_CONTROL,
        hyper::header::HeaderValue::from_static("no-store"),
    );
    response
}

fn refuse(status: StatusCode, code: &str, message: &str) -> Answer {
    respond(
        status,
        &json!({ "ok": false, "error": { "code": code, "message": message, "retryable": false } }),
    )
}

fn status_for(code: &str) -> StatusCode {
    match code {
        "invalid_argument" => StatusCode::BAD_REQUEST,
        "unauthenticated" => StatusCode::UNAUTHORIZED,
        "out_of_scope" | "policy_denied" => StatusCode::FORBIDDEN,
        "not_found" => StatusCode::NOT_FOUND,
        "conflict" => StatusCode::CONFLICT,
        "unavailable" | "busy" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn header(request: &Request<Incoming>, name: hyper::header::HeaderName) -> Option<&str> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

fn retry_key(headers: &hyper::HeaderMap) -> Result<Option<String>, &'static str> {
    let mut values = headers.get_all("idempotency-key").iter();
    let Some(header) = values.next() else {
        return Ok(None);
    };
    let bad_key = "Send one Idempotency-Key with 1 to 255 printable ASCII bytes";
    if values.next().is_some() {
        return Err(bad_key);
    }
    let raw = header.to_str().map_err(|_| bad_key)?;
    let key = if raw.starts_with('"') {
        serde_json::from_str::<String>(raw).map_err(|_| bad_key)?
    } else {
        raw.to_owned()
    };
    if !crate::mcp::wire::valid_retry_key(&key) {
        return Err(bad_key);
    }
    Ok(Some(key))
}

/// The `OpenAPI` description of every engine operation the tools expose.
pub fn openapi(port: u16) -> Value {
    let mut paths = serde_json::Map::new();
    for tool in TOOLS.iter().filter(|tool| !tool.op.is_empty()) {
        paths.insert(
            format!("/v2/{}", tool.op),
            json!({
                "post": {
                    "operationId": tool.op,
                    "parameters": [{ "in": "header", "name": "Idempotency-Key", "required": false,
                        "schema": { "type": "string", "minLength": 1, "maxLength": 255 },
                        "description": "Reuse this key to retry the same write with the same arguments." }],
                    "summary": tool.title,
                    "description": tool.description,
                    "requestBody": { "required": true, "content": { "application/json": { "schema": (tool.input)() } } },
                    "responses": {
                        "200": { "description": "`{\"ok\": true, \"result\": …}`" },
                        "4XX": { "description": "`{\"ok\": false, \"error\": {\"code\", \"message\", \"retryable\"}}`" },
                    },
                    "security": [{ "token": [] }],
                }
            }),
        );
    }
    json!({
        "openapi": "3.1.0",
        "info": { "title": "ClipMill engine API", "version": env!("CARGO_PKG_VERSION"), "description": "See docs/engine-api.md." },
        "servers": [{ "url": format!("http://127.0.0.1:{port}") }],
        "components": { "securitySchemes": { "token": { "type": "http", "scheme": "bearer" } } },
        "paths": paths,
    })
}

async fn body_json(request: Request<Incoming>) -> Result<Value, Answer> {
    let bytes = Limited::new(request.into_body(), MAX_BODY)
        .collect()
        .await
        .map_err(|_| {
            refuse(
                StatusCode::PAYLOAD_TOO_LARGE,
                "invalid_argument",
                "the body is larger than 256 KiB",
            )
        })?
        .to_bytes();
    if bytes.is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_slice(&bytes).map_err(|_| {
        refuse(
            StatusCode::BAD_REQUEST,
            "invalid_argument",
            "the body is not JSON",
        )
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "the checks run in order, each a refusal; splitting them hides the order"
)]
async fn answer(shared: Arc<Shared>, request: Request<Incoming>) -> Answer {
    // DNS rebinding: a page that resolved its own name to 127.0.0.1 still
    // sends its own name here.
    let host = header(&request, hyper::header::HOST).unwrap_or_default();
    if !shared.hosts.iter().any(|allowed| allowed == host) {
        return refuse(
            StatusCode::MISDIRECTED_REQUEST,
            "invalid_argument",
            "this gateway answers only to its loopback address",
        );
    }
    // A web page: refused unless its origin was allowed by name.
    if let Some(origin) = header(&request, hyper::header::ORIGIN)
        && !shared
            .settings
            .allowed_origins
            .iter()
            .any(|allowed| allowed == origin)
    {
        return refuse(
            StatusCode::FORBIDDEN,
            "policy_denied",
            "requests from web pages are not allowed",
        );
    }
    let path = request.uri().path().to_owned();
    if request.method() == Method::GET && path == "/v2/openapi.json" {
        return respond(StatusCode::OK, &openapi(shared.settings.port));
    }
    if request.method() != Method::POST {
        return refuse(
            StatusCode::METHOD_NOT_ALLOWED,
            "invalid_argument",
            "use POST",
        );
    }
    let json_body = header(&request, hyper::header::CONTENT_TYPE).is_some_and(|value| {
        value
            .split(';')
            .next()
            .is_some_and(|kind| kind.trim() == "application/json")
    });
    if !json_body {
        return refuse(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "invalid_argument",
            "send application/json",
        );
    }
    let token = header(&request, hyper::header::AUTHORIZATION)
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_owned);
    let Some(token) = token else {
        return refuse(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "send a connected-agent token as Authorization: Bearer; create one in ClipMill's Settings, Connected agents",
        );
    };
    let Ok(_permit) = shared.permits.try_acquire() else {
        return refuse(
            StatusCode::SERVICE_UNAVAILABLE,
            "busy",
            "too many requests at once; try again",
        );
    };
    let engine = Engine::new(shared.settings.socket.clone(), token);
    let headers = request.headers().clone();
    let mut body = match body_json(request).await {
        Ok(body) => body,
        Err(refusal) => return refusal,
    };
    let key = match retry_key(&headers) {
        Ok(key) => key,
        Err(reason) => return refuse(StatusCode::BAD_REQUEST, "invalid_argument", reason),
    };
    if path == "/mcp" {
        return mcp_answer(&engine, &headers, &mut body, key.as_deref()).await;
    }
    let Some(op) = path
        .strip_prefix("/v2/")
        .filter(|op| !op.is_empty() && !op.contains('/'))
    else {
        return refuse(
            StatusCode::NOT_FOUND,
            "not_found",
            "the paths are /v2/<operation> and /mcp",
        );
    };
    match engine.call_keyed(op, body, key.as_deref()).await {
        Ok(Ok(result)) => respond(StatusCode::OK, &json!({ "ok": true, "result": result })),
        Ok(Err(error)) => respond(
            status_for(error["code"].as_str().unwrap_or("internal")),
            &json!({ "ok": false, "error": error }),
        ),
        Err(link) => refuse(
            StatusCode::SERVICE_UNAVAILABLE,
            "unavailable",
            &link.to_string(),
        ),
    }
}

async fn mcp_answer(
    engine: &Engine,
    headers: &hyper::HeaderMap,
    body: &mut Value,
    retry_key: Option<&str>,
) -> Answer {
    use crate::mcp::wire;
    match engine.call("whoami", json!({})).await {
        Ok(Ok(_)) => {}
        Ok(Err(problem)) => {
            let code = problem["code"].as_str().unwrap_or("internal");
            return refuse(
                status_for(code),
                code,
                problem["message"]
                    .as_str()
                    .unwrap_or("Authentication failed"),
            );
        }
        Err(link) => {
            return refuse(
                StatusCode::SERVICE_UNAVAILABLE,
                "unavailable",
                &link.to_string(),
            );
        }
    }
    let modern = wire::http_is_modern(headers, body);
    if modern && let Some(refusal) = wire::check_http(headers, body) {
        return respond(StatusCode::BAD_REQUEST, &refusal);
    }
    if let Some(key) = retry_key
        && body["method"] == "tools/call"
    {
        if !body["params"].is_object()
            || body["params"]
                .get("_meta")
                .is_some_and(|meta| !meta.is_object())
            || body["params"]["_meta"]
                .get(wire::RETRY_KEY)
                .is_some_and(|value| value != key)
        {
            let error = crate::mcp::error(
                &body["id"],
                -32602,
                "Invalid tool metadata or conflicting request keys",
            );
            return respond(StatusCode::BAD_REQUEST, &error);
        }
        body["params"]["_meta"][wire::RETRY_KEY] = json!(key);
    }
    match crate::mcp::handle(engine, body).await {
        Some(reply) => respond(
            if modern {
                wire::http_status(&reply)
            } else {
                StatusCode::OK
            },
            &reply,
        ),
        None => Response::builder()
            .status(StatusCode::ACCEPTED)
            .body(Full::new(Bytes::new()))
            .unwrap_or_else(|_| refuse(StatusCode::INTERNAL_SERVER_ERROR, "internal", "")),
    }
}

/// Listen on 127.0.0.1 until the process is stopped.
pub async fn serve(settings: Settings) -> std::io::Result<()> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, settings.port));
    let listener = TcpListener::bind(address).await?;
    let port = listener.local_addr()?.port();
    let shared = Arc::new(Shared {
        hosts: [
            format!("127.0.0.1:{port}"),
            format!("localhost:{port}"),
            format!("[::1]:{port}"),
        ],
        settings: Settings { port, ..settings },
        permits: Semaphore::new(IN_FLIGHT),
    });
    eprintln!(
        "clipmill: the engine API is at http://127.0.0.1:{port}/v2/ and MCP at http://127.0.0.1:{port}/mcp"
    );
    loop {
        let (stream, _) = listener.accept().await?;
        let shared = shared.clone();
        tokio::spawn(async move {
            let service = service_fn(move |request| {
                let shared = shared.clone();
                async move { Ok::<_, std::convert::Infallible>(answer(shared, request).await) }
            });
            if let Err(error) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .await
            {
                eprintln!("clipmill: a gateway connection ended: {error}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operation_is_described() {
        let spec = openapi(DEFAULT_PORT);
        let paths = spec["paths"].as_object().map_or(0, serde_json::Map::len);
        assert_eq!(
            paths,
            TOOLS.iter().filter(|tool| !tool.op.is_empty()).count()
        );
        assert_eq!(
            spec["paths"]["/v2/start_analysis"]["post"]["operationId"],
            "start_analysis"
        );
    }

    #[test]
    fn retry_keys_accept_plain_or_quoted_values_and_reject_ambiguous_headers() {
        let mut headers = hyper::HeaderMap::new();
        assert_eq!(retry_key(&headers), Ok(None));
        for input in ["export-7", "\"export-7\""] {
            headers.insert(
                "idempotency-key",
                hyper::header::HeaderValue::from_static(input),
            );
            assert_eq!(retry_key(&headers), Ok(Some("export-7".to_owned())));
        }
        headers.append(
            "idempotency-key",
            hyper::header::HeaderValue::from_static("another"),
        );
        assert!(retry_key(&headers).is_err());
        headers.insert(
            "idempotency-key",
            hyper::header::HeaderValue::from_static("\"\""),
        );
        assert!(retry_key(&headers).is_err());
    }

    #[test]
    fn errors_map_to_http_statuses() {
        assert_eq!(status_for("out_of_scope"), StatusCode::FORBIDDEN);
        assert_eq!(status_for("unauthenticated"), StatusCode::UNAUTHORIZED);
        assert_eq!(
            status_for("anything else"),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
