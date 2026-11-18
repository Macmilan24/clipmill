//! Ask for rights on the original tools/call before submitting the export job.
use serde_json::{Value, json};

use super::{Engine, engine_result, tool_result, wire};

pub(super) fn form(params: &Value) -> bool {
    let capability = &params["_meta"][wire::CAPABILITIES]["elicitation"];
    capability["form"].is_object()
        || capability
            .as_object()
            .is_some_and(serde_json::Map::is_empty)
}

pub(super) enum Confirmation {
    Reply(Value),
    Accepted { rights: String, retry_key: String },
}

pub(super) async fn confirm(
    engine: &Engine,
    arguments: &Value,
    params: &Value,
) -> Result<Confirmation, String> {
    let mut args = json!({"arguments":arguments});
    if let Some(state) = params.get("requestState") {
        args["state"] = state.clone();
    } else if params.get("inputResponses").is_some() {
        return Err("An export response needs its original requestState".to_owned());
    }
    let check = match engine_result(engine, "mcp_export_consent", args).await {
        Ok(check) => check,
        Err(problem) if params.get("requestState").is_some() => return Err(problem),
        Err(problem) => return Ok(Confirmation::Reply(tool_result(Err(problem)))),
    };
    let state = check["state"]
        .as_str()
        .ok_or("The engine did not issue an export confirmation")?;
    if params.get("requestState").is_none() {
        let destination = arguments["destination_dir"].as_str().unwrap_or_default();
        return Ok(Confirmation::Reply(json!({
            "resultType":"input_required","requestState":state,
            "inputRequests":{"rights":{"method":"elicitation/create","params":{
                "mode":"form","message":format!("Export this clip to {destination}? Confirm that you own this recording or have permission to use it."),
                "requestedSchema":{"type":"object","properties":{
                    "rights":{"type":"string","title":"Your rights to the recording","minLength":1,"maxLength":2048}
                },"required":["rights"]}
            }}}
        })));
    }
    let reply = &params["inputResponses"]["rights"];
    match reply["action"].as_str() {
        Some("accept") => {
            let rights = reply["content"]["rights"]
                .as_str()
                .map(str::trim)
                .filter(|text| !text.is_empty() && text.len() <= 2048)
                .ok_or("Enter your rights statement to accept the export")?;
            Ok(Confirmation::Accepted {
                rights: rights.to_owned(),
                retry_key: format!("export-{state}"),
            })
        }
        Some("decline" | "cancel") => Ok(Confirmation::Reply(tool_result(Err(
            "Export cancelled by the user.".to_owned(),
        )))),
        _ => Err(
            "An export confirmation must contain an accept, decline or cancel response".to_owned(),
        ),
    }
}
