//! Per-request dialect and HTTP routing metadata for both MCP generations.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hyper::{HeaderMap, StatusCode};
use serde_json::{Value, json};

pub(super) const VERSION: &str = "2026-07-28";
pub(super) const PROTOCOL: &str = "io.modelcontextprotocol/protocolVersion";
pub(super) const CAPABILITIES: &str = "io.modelcontextprotocol/clientCapabilities";
const CLIENT: &str = "io.modelcontextprotocol/clientInfo";
pub(crate) const RETRY_KEY: &str = "io.clipmill/requestId";

pub(crate) fn valid_retry_key(key: &str) -> bool {
    !key.trim().is_empty() && key.len() <= 255 && key.bytes().all(|byte| (32..=126).contains(&byte))
}

const SERVER: &str = "io.modelcontextprotocol/serverInfo";
pub(super) const TASKS: &str = "io.modelcontextprotocol/tasks";

pub(super) fn versions() -> Vec<&'static str> {
    std::iter::once(VERSION)
        .chain(super::PROTOCOL_VERSIONS)
        .collect()
}

pub(super) fn server_info() -> Value {
    json!({ "name": "clipmill", "title": "ClipMill", "version": env!("CARGO_PKG_VERSION") })
}

pub(super) fn discovery() -> Value {
    json!({
        "supportedVersions": versions(),
        "capabilities": { "tools": {}, "resources": {}, "extensions": { (TASKS): {} } },
        "instructions": super::INSTRUCTIONS,
    })
}

pub(super) fn is_modern(body: &Value) -> bool {
    let meta = &body["params"]["_meta"];
    body["method"] == "server/discover"
        || [PROTOCOL, CAPABILITIES, CLIENT]
            .iter()
            .any(|key| meta.get(key).is_some())
}

pub(super) fn check_request(body: &Value) -> Result<(), Value> {
    let id = &body["id"];
    let meta = &body["params"]["_meta"];
    let Some(version) = meta[PROTOCOL].as_str() else {
        return Err(super::error(
            id,
            -32602,
            "Missing the per-request protocol version",
        ));
    };
    if version != VERSION {
        let mut response = super::error(id, -32022, "Unsupported protocol version");
        response["error"]["data"] = json!({ "supported": versions(), "requested": version });
        return Err(response);
    }
    let capabilities = &meta[CAPABILITIES];
    if !capabilities.is_object()
        || capabilities
            .get("extensions")
            .is_some_and(|v| !v.is_object())
        || capabilities["extensions"]
            .get(TASKS)
            .is_some_and(|v| !v.is_object())
        || meta
            .get(CLIENT)
            .is_some_and(|info| !info["name"].is_string() || !info["version"].is_string())
    {
        return Err(super::error(
            id,
            -32602,
            "Invalid per-request client metadata",
        ));
    }
    Ok(())
}

pub(super) fn finish(mut result: Value) -> Value {
    if result.get("resultType").is_none() {
        result["resultType"] = json!("complete");
    }
    result["_meta"][SERVER] = server_info();
    result
}

pub(super) fn offers_tasks(params: &Value) -> bool {
    params["_meta"][CAPABILITIES]["extensions"][TASKS].is_object()
}

pub(super) fn missing_tasks(id: &Value) -> Value {
    let mut response = super::error(id, -32021, "This request requires the Tasks extension");
    response["error"]["data"] =
        json!({ "requiredCapabilities": { "extensions": { (TASKS): {} } } });
    response
}

/// Decode the base64 sentinel required for non-ASCII and whitespace-padded names.
fn mirrored_name(value: &str) -> Option<String> {
    if let Some(encoded) = value
        .strip_prefix("=?base64?")
        .and_then(|v| v.strip_suffix("?="))
    {
        String::from_utf8(STANDARD.decode(encoded).ok()?).ok()
    } else {
        Some(value.to_owned())
    }
}

fn single_header<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    let mut values = headers.get_all(key).iter();
    let value = values.next()?.to_str().ok()?;
    values.next().is_none().then_some(value)
}

pub(crate) fn http_is_modern(headers: &HeaderMap, body: &Value) -> bool {
    let header = single_header(headers, "mcp-protocol-version");
    is_modern(body)
        || headers.contains_key("mcp-method")
        || header.is_some_and(|v| !super::PROTOCOL_VERSIONS.contains(&v))
}

pub(crate) fn check_http(headers: &HeaderMap, body: &Value) -> Option<Value> {
    let protocol = single_header(headers, "mcp-protocol-version");
    let method = single_header(headers, "mcp-method");
    let expected_method = body["method"].as_str();
    let expected_name = match expected_method {
        Some("tools/call" | "prompts/get") => Some(&body["params"]["name"]),
        Some("resources/read") => Some(&body["params"]["uri"]),
        Some("tasks/get" | "tasks/cancel" | "tasks/update") => Some(&body["params"]["taskId"]),
        _ => None,
    };
    let matches_name = expected_name.is_none_or(|name| {
        single_header(headers, "mcp-name")
            .and_then(mirrored_name)
            .as_deref()
            .zip(name.as_str())
            .is_some_and(|(header, param)| header == param)
    });
    if protocol.is_none()
        || protocol != body["params"]["_meta"][PROTOCOL].as_str()
        || method.is_none()
        || method != expected_method
        || !matches_name
    {
        return Some(super::error(
            &body["id"],
            -32020,
            "MCP routing headers must match the request body",
        ));
    }
    None
}

pub(crate) fn http_status(response: &Value) -> StatusCode {
    match response["error"]["code"].as_i64() {
        Some(-32601) => StatusCode::NOT_FOUND,
        Some(-32603) => StatusCode::INTERNAL_SERVER_ERROR,
        Some(AUTH_ERROR) => StatusCode::UNAUTHORIZED,
        Some(SCOPE_ERROR) => StatusCode::FORBIDDEN,
        Some(_) => StatusCode::BAD_REQUEST,
        None => StatusCode::OK,
    }
}

// Server-defined errors for a connection whose credentials or grants changed.
pub(super) const AUTH_ERROR: i64 = -32040;
pub(super) const SCOPE_ERROR: i64 = -32041;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use hyper::header::HeaderValue;

    #[test]
    fn http_task_routing_checks_version_method_name_and_duplicates() {
        let body = json!({"id":1,"method":"tasks/get","params":{"taskId":"job_£","_meta":{(PROTOCOL):VERSION}}});
        let mut headers = HeaderMap::new();
        headers.insert("mcp-protocol-version", HeaderValue::from_static(VERSION));
        headers.insert("mcp-method", HeaderValue::from_static("tasks/get"));
        assert_eq!(
            check_http(&headers, &body).unwrap()["error"]["code"],
            -32020
        );
        headers.insert("mcp-name", HeaderValue::from_static("=?base64?am9iX8Kj?="));
        assert!(check_http(&headers, &body).is_none());
        headers.insert("mcp-method", HeaderValue::from_static("tasks/cancel"));
        assert!(check_http(&headers, &body).is_some());
        headers.insert("mcp-method", HeaderValue::from_static("tasks/get"));
        headers.append("mcp-name", HeaderValue::from_static("different"));
        assert!(check_http(&headers, &body).is_some());
    }

    #[test]
    fn legacy_headers_require_no_modern_session() {
        let mut headers = HeaderMap::new();
        let legacy = json!({"method":"tools/list","id":3});
        assert!(!http_is_modern(&headers, &legacy));
        headers.insert(
            "mcp-protocol-version",
            HeaderValue::from_static("2025-11-25"),
        );
        assert!(!http_is_modern(&headers, &legacy));
        headers.insert(
            "mcp-protocol-version",
            HeaderValue::from_static("2099-01-01"),
        );
        assert!(http_is_modern(&headers, &legacy));
    }
}
