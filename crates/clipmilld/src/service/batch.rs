//! Durable export selection, using the existing immutable export pipeline.
use super::{
    Outcome, Reply, Service, error_reply, export_submission_reply, response_reply,
    store_error_reply, unix_millis,
};
use crate::db::BatchCommand;
use clipmill_contracts::proto::ipc::v1::{
    CancelJobRequest, ErrorCode, ExportBatchItemV1, ExportBatchResponse, ExportBatchV1,
    ExportClipRequest, ExportClipResponse, ExportRequestV1, JobState, ListExportBatchesResponse,
    PlanExportRequest, Request, Response, SubmitExportBatchRequest, UpdateExportBatchItemRequest,
    request, response,
};
use prost::Message;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

impl Service {
    #[allow(
        clippy::too_many_lines,
        reason = "bounded collection validation precedes a single durable admission"
    )]
    pub(super) async fn submit_export_batch(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        asked: SubmitExportBatchRequest,
    ) -> Reply {
        match self
            .database
            .replay_request(request_id.clone(), request_hash)
            .await
        {
            Ok(Some(bytes)) => {
                return Reply {
                    bytes,
                    outcome: Outcome::Success,
                };
            }
            Ok(None) => {}
            Err(error) => return store_error_reply(request_id, &error),
        }
        if asked.requests.is_empty() || asked.requests.len() > 50 {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "Select between one and fifty edits",
            );
        }
        let mut docs = BTreeSet::new();
        let mut ordinals = BTreeSet::new();
        for request in &asked.requests {
            if request.expected_revision.is_none()
                || request.source_attestation.is_empty()
                || !docs.insert(&request.doc_id)
                || request.index == 0
                || !ordinals.insert(request.index)
                || !request.naming_pattern.contains("{index}")
            {
                return error_reply(
                    request_id,
                    ErrorCode::InvalidArgument,
                    "Review each distinct edit revision and source rights; batch filenames need {index}",
                );
            }
        }
        // Each export revalidates before dispatch. This additional admission
        // check prevents a collection whose individual estimates fit, but
        // whose combined size already exceeds the destination's free space.
        let mut volumes: BTreeMap<PathBuf, (u64, u64)> = BTreeMap::new();
        let mut filenames = BTreeSet::new();
        for asked_item in &asked.requests {
            let planned = self
                .plan_export(
                    request_id.clone(),
                    &PlanExportRequest {
                        request: Some(asked_item.clone()),
                    },
                )
                .await;
            if let Some(response::Body::PlanExport(plan)) =
                Response::decode(planned.bytes.as_slice())
                    .ok()
                    .and_then(|reply| reply.body)
                && let Ok(folder) = crate::export::resolve_destination(&asked_item.destination_dir)
            {
                for name in &plan.file_names {
                    if !filenames.insert(folder.join(name)) {
                        return error_reply(
                            request_id,
                            ErrorCode::InvalidArgument,
                            "Two selected edits would write the same filename. Use a distinct index in each name.",
                        );
                    }
                }
                if plan.available_known {
                    let tally = volumes.entry(folder).or_insert((0, plan.available_bytes));
                    tally.0 = tally.0.saturating_add(plan.estimated_bytes);
                    tally.1 = tally.1.min(plan.available_bytes);
                }
            }
        }
        if volumes
            .values()
            .any(|(required, available)| required > available)
        {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "The selected collection needs more space than the destination has available. Free space or choose fewer clips.",
            );
        }
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        let batch = ExportBatchV1 {
            batch_id: format!("batch_{}", ulid::Ulid::new()),
            created_unix_millis: now,
            items: asked
                .requests
                .into_iter()
                .map(|request| ExportBatchItemV1 {
                    project_id: String::new(),
                    index: request.index,
                    request: Some(request),
                    state: "pending".to_owned(),
                    attempt: 0,
                    queued: None,
                    error: String::new(),
                })
                .collect(),
        };
        match self
            .database
            .export_batches(BatchCommand::Create {
                request_id: request_id.clone(),
                request_hash,
                batch,
            })
            .await
        {
            Ok(mut batches) => {
                let Some(batch) = batches.pop() else {
                    return error_reply(request_id, ErrorCode::Internal, "No saved export batch");
                };
                self.dispatch_batch(batch.batch_id.clone());
                batch_reply(request_id, batch)
            }
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    pub(super) async fn list_export_batches(&self, request_id: String) -> Reply {
        match self.database.export_batches(BatchCommand::List).await {
            Ok(batches) => response_reply(
                request_id,
                response::Body::ListExportBatches(ListExportBatchesResponse { batches }),
            ),
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    pub(crate) async fn resume_export_batches(&self) {
        if let Ok(batches) = self.database.export_batches(BatchCommand::List).await {
            for batch in batches
                .into_iter()
                .filter(|batch| batch.items.iter().any(|item| item.state == "pending"))
            {
                self.dispatch_batch(batch.batch_id);
            }
        }
    }

    fn dispatch_batch(&self, id: String) {
        let service = self.clone();
        tokio::spawn(async move {
            service.admit_batch(&id).await;
        });
    }

    async fn admit_batch(&self, id: &str) {
        // Serializes admission and explicit retry/cancel in this daemon. SQLite
        // still conditions links on attempt+state, and the daemon lock forbids
        // a second process writing beside this one.
        let _guard = self.batch_admission.lock().await;
        let Ok(batches) = self.database.export_batches(BatchCommand::List).await else {
            return;
        };
        let Some(batch) = batches.into_iter().find(|batch| batch.batch_id == id) else {
            return;
        };
        for item in batch
            .items
            .into_iter()
            .filter(|item| item.state == "pending")
        {
            let Some(request) = &item.request else {
                continue;
            };
            let request_id = format!("{id}/{}/{}", item.index, item.attempt);
            let envelope = Request {
                request_id: request_id.clone(),
                body: Some(request::Body::ExportClip(ExportClipRequest {
                    request: Some(request.clone()),
                })),
            };
            let hash = Sha256::digest(envelope.encode_to_vec()).into();
            let reply = if item.attempt > 0 && item.queued.is_some() {
                self.retry_frozen_export(request_id, hash, request, item.queued.as_ref())
                    .await
            } else {
                self.export_clip(
                    request_id,
                    hash,
                    &ExportClipRequest {
                        request: Some(request.clone()),
                    },
                )
                .await
            };
            let (queued, error) = match Response::decode(reply.bytes.as_slice())
                .ok()
                .and_then(|response| response.body)
            {
                Some(response::Body::ExportClip(export)) => (Some(export), String::new()),
                Some(response::Body::Error(error)) => (None, error.message),
                _ => (
                    None,
                    "The daemon did not acknowledge this export; retry the item.".to_owned(),
                ),
            };
            let _ = self
                .database
                .export_batches(BatchCommand::Link {
                    batch_id: id.to_owned(),
                    index: item.index,
                    attempt: item.attempt,
                    queued,
                    error,
                })
                .await;
        }
    }

    async fn retry_frozen_export(
        &self,
        request_id: String,
        hash: [u8; 32],
        request: &ExportRequestV1,
        previous: Option<&ExportClipResponse>,
    ) -> Reply {
        match self.database.replay_request(request_id.clone(), hash).await {
            Ok(Some(bytes)) => return export_submission_reply(request_id, &bytes),
            Ok(None) => {}
            Err(error) => return store_error_reply(request_id, &error),
        }
        let Some(previous) = previous else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "No approved snapshot to retry",
            );
        };
        let record = match self.database.get_edit_doc(request.doc_id.clone()).await {
            Ok(record) => record,
            Err(error) => return store_error_reply(request_id, &error),
        };
        let (Ok(project), Ok(ir)) = (record.project_id.parse(), previous.ir_artifact_id.parse())
        else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "Saved export identity is invalid",
            );
        };
        self.submit_export(
            request_id,
            hash,
            &project,
            request.clone(),
            ir,
            previous.revision,
        )
        .await
    }

    #[allow(
        clippy::too_many_lines,
        reason = "request replay, job cancellation and item transition are one ordered admission operation"
    )]
    pub(super) async fn update_export_batch_item(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        update: UpdateExportBatchItemRequest,
    ) -> Reply {
        let guard = self.batch_admission.lock().await;
        match self
            .database
            .replay_request(request_id.clone(), request_hash)
            .await
        {
            Ok(Some(bytes)) => {
                return Reply {
                    bytes,
                    outcome: Outcome::Success,
                };
            }
            Ok(None) => {}
            Err(error) => return store_error_reply(request_id, &error),
        }
        if !matches!(update.action.as_str(), "retry" | "cancel") {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "Choose retry or cancel",
            );
        }
        let batches = match self.database.export_batches(BatchCommand::List).await {
            Ok(value) => value,
            Err(error) => return store_error_reply(request_id, &error),
        };
        let Some(item) = batches
            .iter()
            .find(|batch| batch.batch_id == update.batch_id)
            .and_then(|batch| batch.items.iter().find(|item| item.index == update.index))
        else {
            return error_reply(request_id, ErrorCode::NotFound, "No such batch item");
        };
        if let Some(queued) = &item.queued {
            let job = match self.database.get_job(queued.job_id.clone()).await {
                Ok(job) => job,
                Err(error) => return store_error_reply(request_id, &error),
            };
            if update.action == "retry"
                && !matches!(
                    JobState::try_from(job.state),
                    Ok(JobState::Failed | JobState::Cancelled)
                )
            {
                return error_reply(
                    request_id,
                    ErrorCode::Conflict,
                    "Only a failed or cancelled export can be retried",
                );
            }
            if update.action == "cancel" {
                if matches!(JobState::try_from(job.state), Ok(JobState::Succeeded)) {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "This export is already delivered",
                    );
                }
                let cancel = CancelJobRequest {
                    job_id: queued.job_id.clone(),
                };
                let cancel_id =
                    format!("{}/{}/{}/cancel", update.batch_id, item.index, item.attempt);
                let envelope = Request {
                    request_id: cancel_id.clone(),
                    body: Some(request::Body::CancelJob(cancel)),
                };
                let hash = Sha256::digest(envelope.encode_to_vec()).into();
                let result = self.cancel_job(cancel_id, hash, &queued.job_id).await;
                let state = match Response::decode(result.bytes.as_slice())
                    .ok()
                    .and_then(|reply| reply.body)
                {
                    Some(response::Body::CancelJob(reply)) => {
                        reply.job.and_then(|job| JobState::try_from(job.state).ok())
                    }
                    Some(response::Body::Error(error)) => {
                        return error_reply(request_id, ErrorCode::Conflict, error.message);
                    }
                    _ => None,
                };
                if state == Some(JobState::Succeeded) {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "This export is already delivered",
                    );
                }
                if !matches!(state, Some(JobState::Cancelled | JobState::Failed)) {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "The export could not be cancelled; refresh its status",
                    );
                }
            }
        }
        let result = self
            .database
            .export_batches(BatchCommand::Update {
                request_id: request_id.clone(),
                request_hash,
                completed_unix_millis: unix_millis().unwrap_or(0),
                batch_id: update.batch_id.clone(),
                index: update.index,
                action: update.action.clone(),
            })
            .await;
        drop(guard);
        match result {
            Ok(mut batches) => {
                if update.action == "retry" {
                    self.dispatch_batch(update.batch_id);
                }
                match batches.pop() {
                    Some(batch) => batch_reply(request_id, batch),
                    None => error_reply(request_id, ErrorCode::Internal, "No saved export batch"),
                }
            }
            Err(error) => store_error_reply(request_id, &error),
        }
    }
}

fn batch_reply(request_id: String, batch: ExportBatchV1) -> Reply {
    response_reply(
        request_id,
        response::Body::ExportBatch(ExportBatchResponse { batch: Some(batch) }),
    )
}
