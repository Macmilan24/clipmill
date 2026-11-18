//! Durable, caller-bound task handles for the job-producing tools.

use chrono::{DateTime, SecondsFormat};
use serde_json::{Value, json};

use super::{Engine, error, tool_result, wire};

pub(super) fn creates_job(op: &str) -> bool {
    matches!(
        op,
        "start_analysis" | "export_clip" | "propose_clips" | "reprompt" | "render_preview"
    )
}

fn timestamp(job: &Value, key: &str) -> Option<String> {
    Some(
        DateTime::from_timestamp_millis(job[key].as_i64()?)?
            .to_rfc3339_opts(SecondsFormat::Millis, true),
    )
}

fn snapshot(job: &Value, creation: bool) -> Option<Value> {
    let state = job["state"].as_str()?;
    let status = match state {
        "succeeded" | "failed" => "completed",
        "cancelled" => "cancelled",
        "queued" | "running" | "cancelling" => "working",
        _ => return None,
    };
    let mut task = json!({
        "resultType": if creation { "task" } else { "complete" },
        "taskId": job["job_id"].as_str()?, "status": status,
        "statusMessage": state, "createdAt": timestamp(job, "created_unix_millis")?,
        "lastUpdatedAt": timestamp(job, "updated_unix_millis")?, "ttlMs": null,
        "pollIntervalMs": 750,
    });
    if !creation && status == "completed" {
        let mut final_result = tool_result(Ok(job.clone()));
        final_result["isError"] = json!(state == "failed");
        task["result"] = wire::finish(final_result);
    }
    Some(task)
}

/// The submit response may contain just a job ID. Return a handle only after
/// the owner can read its durable state, including its original timestamps.
pub(super) async fn after_submit(engine: &Engine, ordinary: Value) -> Value {
    if let Some(id) = ordinary["structuredContent"]["job_id"].as_str()
        && let Ok(Ok(job)) = engine.call("get_owned_job", json!({ "job_id": id })).await
        && let Some(task) = snapshot(&job, true)
    {
        return task;
    }
    ordinary
}

fn rejected(id: &Value, refusal: &Value) -> Value {
    let code = match refusal["code"].as_str() {
        Some("not_found" | "invalid_argument" | "conflict") => -32602,
        Some("unauthenticated") => wire::AUTH_ERROR,
        Some("out_of_scope" | "policy_denied") => wire::SCOPE_ERROR,
        _ => -32603,
    };
    error(
        id,
        code,
        refusal["message"].as_str().unwrap_or("Task request failed"),
    )
}

/// Share the engine's actor check for reads and input acknowledgements. These
/// jobs never request mid-flight input. The Tasks extension recommends ignoring
/// unknown and already-answered input keys, so valid updates are a no-op.
pub(super) async fn handle(engine: &Engine, id: &Value, method: &str, params: &Value) -> Value {
    if !wire::offers_tasks(params) {
        return wire::missing_tasks(id);
    }
    let Some(task_id) = params["taskId"].as_str().filter(|id| !id.is_empty()) else {
        return error(id, -32602, "A taskId is required");
    };
    if method == "tasks/update" && !params["inputResponses"].is_object() {
        return error(id, -32602, "An inputResponses object is required");
    }
    let op = if method == "tasks/cancel" {
        "cancel_owned_job"
    } else {
        "get_owned_job"
    };
    match engine.call(op, json!({ "job_id": task_id })).await {
        Ok(Ok(job)) if method == "tasks/get" => match snapshot(&job, false) {
            Some(result) => super::reply(id, &wire::finish(result)),
            None => error(id, -32603, "The engine returned an invalid task state"),
        },
        Ok(Ok(_)) => super::reply(id, &wire::finish(json!({}))),
        Ok(Err(refusal)) => rejected(id, &refusal),
        Err(link) => error(id, -32603, &link.to_string()),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn job(state: &str) -> Value {
        json!({
            "job_id": "job_1", "state": state,
            "created_unix_millis": 0, "updated_unix_millis": 90_123,
            "progress": { "done": 2, "total": 3 }, "failure": "decoder lost its source",
        })
    }

    #[test]
    fn creation_and_polling_use_durable_job_times() {
        let created = snapshot(&job("queued"), true).unwrap();
        let poll = snapshot(&job("running"), false).unwrap();
        assert_eq!(created["createdAt"], "1970-01-01T00:00:00.000Z");
        assert_eq!(created["lastUpdatedAt"], "1970-01-01T00:01:30.123Z");
        assert_eq!(created["taskId"], poll["taskId"]);
        assert_eq!(created["resultType"], "task");
        assert_eq!(poll["resultType"], "complete");
        assert!(poll.get("result").is_none());
        assert!(snapshot(&json!({"job_id": "incomplete"}), false).is_none());
    }

    #[test]
    fn completed_tasks_embed_tool_results_and_cooperative_cancellation_is_visible() {
        for (state, is_error) in [("succeeded", false), ("failed", true)] {
            let completed = snapshot(&job(state), false).unwrap();
            assert_eq!(completed["status"], "completed");
            assert_eq!(completed["result"]["resultType"], "complete");
            assert_eq!(completed["result"]["isError"], is_error);
            assert_eq!(completed["result"]["structuredContent"], job(state));
        }
        let cancelling = snapshot(&job("cancelling"), false).unwrap();
        assert_eq!(cancelling["status"], "working");
        let cancelled = snapshot(&job("cancelled"), false).unwrap();
        assert_eq!(cancelled["status"], "cancelled");
        assert!(cancelled.get("result").is_none());
    }
}
