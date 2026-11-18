//! Editing and progress resources shared by the CLI and gateway.

use clipmill_contracts::proto::ipc::v1::{
    ApplyEditCommandRequest, EditDoc, GetEditDocRequest, GetSourceRequest, ListEditHistoryRequest,
    TaskState, request, response,
};
use serde_json::{Value, json};

use super::Service;
use super::api_v2::{ApiError, ApiResult, arg_str};
use crate::jobs::EventFilter;

fn document_json(value: &str) -> ApiResult {
    serde_json::from_str(value).map_err(|_| ApiError::new("internal", "invalid stored document"))
}

fn edit(doc: &EditDoc) -> ApiResult {
    Ok(json!({
        "doc_id": doc.doc_id, "project_id": doc.project_id, "revision": doc.revision,
        "source_id": doc.source_id, "candidate_id": doc.candidate_id, "job_id": doc.job_id,
        "document": document_json(&doc.document_json)?
    }))
}

fn revision(args: &Value) -> Result<u64, ApiError> {
    args["expected_revision"].as_u64().ok_or_else(|| {
        ApiError::invalid("expected_revision must name the edit revision you viewed")
    })
}

impl Service {
    pub(super) async fn get_api_source(&self, args: &Value) -> ApiResult {
        let body = self
            .v1(
                request::Body::GetSource(GetSourceRequest {
                    source_id: arg_str(args, "source_id")?.to_owned(),
                }),
                None,
            )
            .await?;
        let response::Body::GetSource(found) = body else {
            return Err(ApiError::new("internal", "unexpected recording response"));
        };
        let source = found
            .source
            .ok_or_else(|| ApiError::not_found("recording not found"))?;
        if args
            .get("project_id")
            .is_some_and(|p| p.as_str() != Some(&source.project_id))
        {
            return Err(ApiError::not_found(
                "recording not found in the requested project",
            ));
        }
        Ok(json!({
            "source_id":source.source_id,"project_id":source.project_id,"path":source.absolute_path,
            "bytes":source.byte_size,"missing":source.missing,"source_map":document_json(&found.source_map_json)?
        }))
    }

    pub(super) async fn timeline(&self, args: &Value) -> ApiResult {
        let run = self.run(args).await?;
        let layers = [
            "timeline.index.v1",
            "index.transcript.v1",
            "mind.script.v1",
            "mind.memory.v1",
            "text.embeddings.v1",
        ];
        let artifacts: Vec<_> = layers
            .iter()
            .filter_map(|kind| {
                run.artifact(kind)
                    .map(|address| json!({"kind":kind,"artifact_id":address}))
            })
            .collect();
        Ok(json!({"job_id":run.job.job_id,"source_id":run.source_id,"artifacts":artifacts}))
    }

    pub(super) async fn job_events(&self, args: &Value) -> ApiResult {
        let job_id = arg_str(args, "job_id")?;
        self.job(job_id).await?;
        let after = args
            .get("after")
            .map_or(Some(0), Value::as_u64)
            .ok_or_else(|| ApiError::invalid("after must be a nonnegative event cursor"))?;
        let limit = args
            .get("limit")
            .map_or(Some(100), Value::as_u64)
            .filter(|limit| (1..=500).contains(limit))
            .ok_or_else(|| ApiError::invalid("limit must be from 1 to 500"))?;
        let events = self
            .database
            .list_events_page(
                after,
                EventFilter {
                    job_id: Some(job_id.to_owned()),
                    project_id: None,
                },
                i64::try_from(limit).unwrap_or(500),
            )
            .await
            .map_err(|e| ApiError::new("unavailable", e.to_string()))?;
        let cursor = events.last().map_or(after, |e| e.event_id);
        let entries: Vec<_>=events.into_iter().map(|e|json!({
            "event_id":e.event_id,"task_id":e.task_id,"attempt":e.attempt,
            "state":TaskState::try_from(e.state).unwrap_or_default().as_str_name().trim_start_matches("TASK_STATE_").to_lowercase(),
            "at_unix_millis":e.at_unix_millis,"wait_reason":e.wait_reason,
            "progress":{"unit":e.progress_unit,"done":e.progress_done,"total":e.progress_total}
        })).collect();
        Ok(json!({"job_id":job_id,"events":entries,"cursor":cursor}))
    }

    pub(super) async fn get_api_edit(&self, args: &Value) -> ApiResult {
        let body = self
            .v1(
                request::Body::GetEditDoc(GetEditDocRequest {
                    doc_id: arg_str(args, "doc_id")?.to_owned(),
                }),
                None,
            )
            .await?;
        let response::Body::GetEditDoc(found) = body else {
            return Err(ApiError::new("internal", "unexpected edit response"));
        };
        edit(
            found
                .doc
                .as_ref()
                .ok_or_else(|| ApiError::not_found("edit not found"))?,
        )
    }

    async fn edit_history(
        &self,
        doc_id: &str,
    ) -> Result<Vec<clipmill_contracts::proto::ipc::v1::EditHistoryEntryV1>, ApiError> {
        let body = self
            .v1(
                request::Body::ListEditHistory(ListEditHistoryRequest {
                    doc_id: doc_id.to_owned(),
                }),
                None,
            )
            .await?;
        let response::Body::ListEditHistory(history) = body else {
            return Err(ApiError::new("internal", "unexpected edit history"));
        };
        Ok(history.entries)
    }

    pub(super) async fn api_edit_log(&self, args: &Value) -> ApiResult {
        let history = self.edit_history(arg_str(args, "doc_id")?).await?;
        let entries=history.iter().map(|e|Ok(json!({
            "revision":e.revision,"at_unix_millis":e.applied_unix_millis,
            "command":document_json(&e.command_json)?,"inverse":document_json(&e.inverse_json)?
        }))).collect::<Result<Vec<_>,ApiError>>()?;
        Ok(json!({"entries":entries}))
    }

    pub(super) async fn change_edit(
        &self,
        args: &Value,
        id: Option<String>,
        undo: bool,
    ) -> ApiResult {
        let doc_id = arg_str(args, "doc_id")?;
        let expected_revision = revision(args)?;
        let command_json = if undo {
            // A retry names the same saved inverse, even after that undo committed.
            self.edit_history(doc_id)
                .await?
                .into_iter()
                .find(|e| e.revision == expected_revision)
                .ok_or_else(|| ApiError::invalid("no command produced the requested revision"))?
                .inverse_json
        } else {
            let commands = args["commands"]
                .as_array()
                .filter(|c| !c.is_empty() && c.len() <= 100)
                .ok_or_else(|| {
                    ApiError::invalid("commands must contain from 1 to 100 edit commands")
                })?;
            json!({"op":"batch","commands":commands}).to_string()
        };
        let body = self
            .v1(
                request::Body::ApplyEditCommand(ApplyEditCommandRequest {
                    doc_id: doc_id.to_owned(),
                    expected_revision,
                    command_json,
                }),
                id,
            )
            .await?;
        let response::Body::ApplyEditCommand(applied) = body else {
            return Err(ApiError::new("internal", "unexpected edit result"));
        };
        let doc = applied
            .doc
            .ok_or_else(|| ApiError::new("internal", "missing edited document"))?;
        Ok(
            json!({"doc_id":doc.doc_id,"revision":doc.revision,"inverse":document_json(&applied.inverse_command_json)?}),
        )
    }
}
