//! The recording-scoped editorial operations exposed to the Studio.
use std::sync::Arc;

use clipmill_contracts::proto::ipc::v1::{EngineCallRequest, request, response};
use serde_json::{Value, json};
use tauri::State;

use crate::DaemonSupervisor;

async fn editorial_call(
    supervisor: &DaemonSupervisor,
    operation: &str,
    args: Value,
) -> Result<Value, String> {
    let body = request::Body::EngineCall(EngineCallRequest {
        token: String::new(),
        call_json: json!({ "op": operation, "args": args }).to_string(),
    });
    let response = supervisor
        .client()
        .call(body)
        .await
        .map_err(|error| error.to_string())?;
    let response::Body::EngineCall(reply) = response else {
        return Err("The engine returned an unexpected editorial response.".to_owned());
    };
    let answer: Value =
        serde_json::from_str(&reply.result_json).map_err(|error| error.to_string())?;
    if answer["ok"] == true {
        return answer
            .get("result")
            .cloned()
            .ok_or_else(|| "The engine returned no result.".to_owned());
    }
    Err(answer["error"]["message"]
        .as_str()
        .unwrap_or("The engine could not finish the request.")
        .to_owned())
}

#[tauri::command]
pub async fn find_moments(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    source_id: String,
    job_id: String,
    query: String,
) -> Result<Value, String> {
    editorial_call(
        &supervisor,
        "find_moments",
        json!({
            "project_id": project_id, "source_id": source_id, "job_id": job_id, "query": query
        }),
    )
    .await
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all(deserialize = "camelCase"), deny_unknown_fields)]
pub struct BriefLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    min_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<u64>,
}

#[tauri::command]
pub async fn reprompt(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    source_id: String,
    job_id: String,
    prompt: String,
    limits: Option<BriefLimits>,
) -> Result<String, String> {
    let mut args = json!(limits.unwrap_or_default());
    args["project_id"] = json!(project_id);
    args["source_id"] = json!(source_id);
    args["job_id"] = json!(job_id);
    args["prompt"] = json!(prompt);
    let job = editorial_call(&supervisor, "reprompt", args).await?;
    job["job_id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "The engine returned no job for this brief.".to_owned())
}

#[derive(serde::Serialize)]
pub struct PlanDecision {
    decision: String,
    reason: String,
}

#[tauri::command]
pub async fn plan_decision(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    source_id: String,
    job_id: String,
    clip_id: String,
    decision: Option<String>,
    reason: Option<String>,
) -> Result<PlanDecision, String> {
    let mut args = json!({
        "project_id": project_id, "source_id": source_id, "job_id": job_id, "clip_id": clip_id
    });
    let op = if let Some(decision) = decision {
        args["decision"] = json!(decision);
        if let Some(reason) = reason {
            args["reason"] = json!(reason);
        }
        "set_decision"
    } else {
        "describe_clip"
    };
    let result = editorial_call(&supervisor, op, args).await?;
    Ok(PlanDecision {
        decision: result["decision"]
            .as_str()
            .unwrap_or("undecided")
            .to_owned(),
        reason: result["reason"].as_str().unwrap_or_default().to_owned(),
    })
}
