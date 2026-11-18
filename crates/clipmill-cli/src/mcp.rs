//! `clipmill mcp`: ClipMill as an MCP server over stdio (R81).
//!
//! An agent host (Claude Desktop, an IDE, any MCP client) starts this process
//! and talks JSON-RPC over its stdin and stdout, one message per line. Each tool
//! is one engine API call, so an agent works the same engine the person does,
//! under the token the person gave it, and every call is in the audit log.
//! Nothing is written to stdout except protocol messages.

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

fn error(id: &Value, code: i64, message: &str) -> Value {
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
    match engine.call(op, args).await {
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

/// The answer to one message, or nothing for a notification.
pub async fn handle(engine: &Engine, message: &Value) -> Option<Value> {
    let id = message.get("id").cloned();
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        // A response to something this server never asks, or garbage.
        return id.map(|id| error(&id, -32600, "Invalid Request"));
    };
    let id = id?;
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "initialize" => reply(
            &id,
            &json!({
                "protocolVersion": negotiate(params["protocolVersion"].as_str()),
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "clipmill", "title": "ClipMill", "version": env!("CARGO_PKG_VERSION") },
                "instructions": INSTRUCTIONS,
            }),
        ),
        "ping" => reply(&id, &json!({})),
        "tools/list" => reply(
            &id,
            &json!({ "tools": TOOLS.iter().map(listing).collect::<Vec<_>>() }),
        ),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let Some(tool) = TOOLS.iter().find(|tool| tool.name == name) else {
                return Some(error(&id, -32602, &format!("Unknown tool: {name}")));
            };
            let result = if tool.op.is_empty() {
                wait_for_job(engine, &arguments).await
            } else {
                engine_result(engine, tool.op, arguments).await
            };
            reply(&id, &tool_result(result))
        }
        _ => error(&id, -32601, &format!("Method not found: {method}")),
    })
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
