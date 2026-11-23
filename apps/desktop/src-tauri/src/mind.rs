//! The recording-scoped editorial operations exposed to the Studio.
use std::{collections::BTreeMap, sync::Arc};

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

/// The id the engine keeps each of a run's planned clips under, by plan id.
///
/// A planned clip's decisions and edit are stored under an id derived from
/// the plan itself, so Results asks the engine rather than deriving it a
/// second time.
#[tauri::command]
pub async fn plan_clip_ids(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    source_id: String,
    job_id: String,
) -> Result<BTreeMap<String, String>, String> {
    let listed = editorial_call(
        &supervisor,
        "list_clips",
        json!({ "project_id": project_id, "source_id": source_id, "job_id": job_id }),
    )
    .await?;
    Ok(plan_ids(&listed))
}

fn plan_ids(listed: &Value) -> BTreeMap<String, String> {
    listed["clips"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|clip| clip["kind"] == "plan")
        .filter_map(|clip| {
            Some((
                clip["clip_id"].as_str()?.to_owned(),
                clip["candidate_id"].as_str()?.to_owned(),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::plan_ids;

    #[test]
    fn planned_clips_are_named_by_their_stored_ids() {
        let listed = json!({
            "job_id": "job_1",
            "clips": [
                { "clip_id": "plan_a", "kind": "plan", "candidate_id": "cand_0123456789abcdef" },
                { "clip_id": "cand_moment", "kind": "moment" },
                { "clip_id": "plan_b", "kind": "plan" },
            ],
        });
        let ids = plan_ids(&listed);
        assert_eq!(ids.len(), 1);
        assert_eq!(ids["plan_a"], "cand_0123456789abcdef");
    }
}
