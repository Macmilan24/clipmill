//! `clipmill mcp`: ClipMill as an MCP server over stdio (R81).
//!
//! An MCP client starts this process
//! and talks JSON-RPC over its stdin and stdout, one message per line. Each tool
//! is one engine API call, so an agent works the same engine the person does,
//! under the token the person gave it, and every call is in the audit log.
//! Nothing is written to stdout except protocol messages.

mod tasks;
pub(crate) mod wire;

use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::client::Engine;
use crate::tools::{TOOLS, listing};

/// Protocol versions this server speaks, newest first. A client asking for
/// one of these gets it; any other gets the newest, and decides for itself.
pub const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "ClipMill turns long recordings into short clips, on this computer. \
Typical flow: clipmill_create_project, clipmill_add_source with an absolute file path, clipmill_analyze, \
clipmill_wait_for_job until it succeeds, clipmill_list_clips, clipmill_describe_clip to read a clip's words, \
then clipmill_create_clip and clipmill_export_clip. Clips of kind \"plan\" were built by ClipMill's editorial \
mind from parts of the recording; explain their form to the person. Ask the person before exporting, and \
pass their own statement of their rights to the recording.";

const WAIT_LIMIT_SECONDS: f64 = 120.0;
const POLL: Duration = Duration::from_millis(750);

fn negotiate(requested: Option<&str>) -> &'static str {
    PROTOCOL_VERSIONS
        .iter()
        .copied()
        .find(|version| Some(*version) == requested)
        .unwrap_or(PROTOCOL_VERSIONS[0])
}

pub(crate) fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn reply(id: &Value, result: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

/// A tool result: the engine's answer as structured content and as text.
fn tool_result(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => {
            let structured = if value.is_object() {
                value
            } else {
                json!({ "result": value })
            };
            json!({
                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&structured).unwrap_or_default() }],
                "structuredContent": structured,
                "isError": false,
            })
        }
        Err(message) => json!({
            "content": [{ "type": "text", "text": message }],
            "isError": true,
        }),
    }
}

async fn engine_result(engine: &Engine, op: &str, args: Value) -> Result<Value, String> {
    keyed_engine_result(engine, op, args, None).await
}

async fn keyed_engine_result(
    engine: &Engine,
    op: &str,
    args: Value,
    key: Option<&str>,
) -> Result<Value, String> {
    match engine.call_keyed(op, args, key).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(refusal)) => Err(format!(
            "{} ({})",
            refusal["message"]
                .as_str()
                .unwrap_or("ClipMill refused the call"),
            refusal["code"].as_str().unwrap_or("error")
        )),
        Err(link) => Err(link.to_string()),
    }
}

async fn wait_for_job(engine: &Engine, args: &Value) -> Result<Value, String> {
    let job_id = args["job_id"].as_str().unwrap_or_default().to_owned();
    let seconds = args["seconds"]
        .as_f64()
        .unwrap_or(60.0)
        .clamp(0.0, WAIT_LIMIT_SECONDS);
    let deadline = tokio::time::Instant::now() + Duration::from_secs_f64(seconds);
    loop {
        let job = engine_result(engine, "get_job", json!({ "job_id": job_id })).await?;
        let state = job["state"].as_str().unwrap_or_default();
        if matches!(state, "succeeded" | "failed" | "cancelled")
            || tokio::time::Instant::now() >= deadline
        {
            return Ok(job);
        }
        tokio::time::sleep(POLL).await;
    }
}

async fn call_tool(engine: &Engine, params: &Value, modern: bool) -> Result<Value, String> {
    let name = params["name"].as_str().unwrap_or_default();
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let tool = TOOLS
        .iter()
        .find(|tool| tool.name == name)
        .ok_or_else(|| format!("Unknown tool: {name}"))?;
    if !arguments.is_object() {
        return Err("Tool arguments must be an object".to_owned());
    }
    let key = match params["_meta"].get(wire::RETRY_KEY) {
        None => None,
        Some(value) => Some(
            value
                .as_str()
                .filter(|key| wire::valid_retry_key(key))
                .ok_or_else(|| {
                    "The request key must be 1 to 255 printable ASCII bytes".to_owned()
                })?,
        ),
    };
    let result = if tool.op.is_empty() {
        wait_for_job(engine, &arguments).await
    } else {
        keyed_engine_result(engine, tool.op, arguments, key).await
    };
    let ordinary = tool_result(result);
    if modern && wire::offers_tasks(params) && tasks::creates_job(tool.op) {
        Ok(tasks::after_submit(engine, ordinary).await)
    } else {
        Ok(ordinary)
    }
}

/// The answer to one message, or nothing for a notification.
pub async fn handle(engine: &Engine, message: &Value) -> Option<Value> {
    let id = message.get("id")?;
    if message["jsonrpc"] != "2.0" || !(id.is_string() || id.is_i64() || id.is_u64()) {
        return Some(error(&Value::Null, -32600, "Invalid Request"));
    }
    let Some(method) = message["method"].as_str() else {
        return Some(error(id, -32600, "Invalid Request"));
    };
    let params = &message["params"];
    let modern = wire::is_modern(message);
    if modern && let Err(invalid) = wire::check_request(message) {
        return Some(invalid);
    }
    let result = match method {
        "server/discover" => wire::discovery(),
        "initialize" if !modern => json!({
            "protocolVersion": negotiate(params["protocolVersion"].as_str()),
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": wire::server_info(),
            "instructions": INSTRUCTIONS,
        }),
        "ping" => json!({}),
        "tools/list" => {
            let mut page = json!({ "tools": TOOLS.iter().map(listing).collect::<Vec<_>>() });
            if modern {
                page["ttlMs"] = json!(60_000);
                page["cacheScope"] = json!("private");
            }
            page
        }
        "tools/call" => match call_tool(engine, params, modern).await {
            Ok(result) => result,
            Err(message) => return Some(error(id, -32602, &message)),
        },
        "tasks/get" | "tasks/update" | "tasks/cancel" if modern => {
            return Some(tasks::handle(engine, id, method, params).await);
        }
        _ => return Some(error(id, -32601, &format!("Method not found: {method}"))),
    };
    Some(reply(
        id,
        &if modern { wire::finish(result) } else { result },
    ))
}

/// Serve until the client closes stdin.
pub async fn serve(engine: Engine) -> std::io::Result<()> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let answer = match serde_json::from_str::<Value>(&line) {
            Ok(message) => handle(&engine, &message).await,
            Err(_) => Some(error(&Value::Null, -32700, "Parse error")),
        };
        if let Some(answer) = answer {
            let mut text = answer.to_string();
            text.push('\n');
            stdout.write_all(text.as_bytes()).await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn engine() -> Engine {
        // Nothing listens here: the tests below never reach the engine.
        Engine::new(
            std::path::PathBuf::from("/nonexistent/clipmilld.sock"),
            String::new(),
        )
    }

    #[tokio::test]
    async fn initialize_negotiates_a_version_and_offers_tools() {
        let answer = handle(
            &engine(),
            &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}}),
        )
        .await
        .unwrap();
        assert_eq!(answer["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(answer["result"]["serverInfo"]["name"], "clipmill");
        let newer = handle(
            &engine(),
            &json!({"jsonrpc": "2.0", "id": 2, "method": "initialize", "params": {"protocolVersion": "2099-01-01"}}),
        )
        .await
        .unwrap();
        assert_eq!(newer["result"]["protocolVersion"], PROTOCOL_VERSIONS[0]);
    }

    fn modern(method: &str, caps: &Value) -> Value {
        json!({"jsonrpc":"2.0","id":7,"method":method,"params":{"_meta":{
            (wire::PROTOCOL):wire::VERSION,(wire::CAPABILITIES):caps
        }}})
    }

    #[tokio::test]
    async fn discovery_and_capabilities_are_stateless() {
        let engine = engine();
        let caps = json!({"extensions":{(wire::TASKS):{}}});
        let result = handle(&engine, &modern("server/discover", &caps))
            .await
            .unwrap();
        assert_eq!(result["result"]["resultType"], "complete");
        assert_eq!(result["result"]["supportedVersions"][0], wire::VERSION);
        assert_eq!(
            result["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            "clipmill"
        );
        assert!(result["result"]["capabilities"]["extensions"][wire::TASKS].is_object());
        for method in ["tasks/get", "tasks/update", "tasks/cancel"] {
            let denied = handle(&engine, &modern(method, &json!({}))).await.unwrap();
            assert_eq!(denied["error"]["code"], -32021);
            assert_eq!(denied["error"]["data"]["requiredCapabilities"], caps);
        }
        let page = handle(&engine, &modern("tools/list", &json!({})))
            .await
            .unwrap();
        assert_eq!(page["result"]["resultType"], "complete");
        assert!(page["result"]["tools"].is_array());
    }

    #[tokio::test]
    async fn missing_metadata_and_unknown_modern_versions_are_protocol_errors() {
        let mut call = modern("server/discover", &json!({}));
        call["params"]["_meta"][wire::PROTOCOL] = json!("2099-01-01");
        let unavailable = handle(&engine(), &call).await.unwrap();
        assert_eq!(unavailable["error"]["code"], -32022);
        assert_eq!(unavailable["error"]["data"]["requested"], "2099-01-01");
        assert_eq!(unavailable["error"]["data"]["supported"][0], wire::VERSION);
        call["params"]["_meta"] = json!({});
        let invalid = handle(&engine(), &call).await.unwrap();
        assert_eq!(invalid["error"]["code"], -32602);
    }

    #[tokio::test]
    async fn every_tool_is_listed_with_an_object_schema() {
        let answer = handle(
            &engine(),
            &json!({"jsonrpc": "2.0", "id": 3, "method": "tools/list"}),
        )
        .await
        .unwrap();
        let tools = answer["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), TOOLS.len());
        for tool in tools {
            assert_eq!(tool["inputSchema"]["type"], "object", "{}", tool["name"]);
            assert!(tool["name"].as_str().unwrap().starts_with("clipmill_"));
        }
    }

    #[tokio::test]
    async fn notifications_get_no_answer_and_unknowns_get_errors() {
        assert!(
            handle(
                &engine(),
                &json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
            )
            .await
            .is_none()
        );
        let unknown = handle(
            &engine(),
            &json!({"jsonrpc": "2.0", "id": 4, "method": "resources/list"}),
        )
        .await
        .unwrap();
        assert_eq!(unknown["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn a_tool_call_without_clipmill_running_says_so() {
        let answer = handle(
            &engine(),
            &json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "clipmill_status", "arguments": {}}}),
        )
        .await
        .unwrap();
        assert_eq!(answer["result"]["isError"], true);
        assert!(
            answer["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("not running")
        );
    }
}
