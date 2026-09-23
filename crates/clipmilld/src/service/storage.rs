//! Storage: what this installation uses on disk, and freeing what nobody uses.
//!
//! The report is categories rather than one number, because each answers a
//! different action. Clean-up acts only on what nothing references or runs:
//! generated files no root reaches, scratch that has gone quiet, interrupted
//! downloads, and old database backups. Model weights are removed through the
//! model library, which knows which analyses still need them.
//!
//! Generated files are collected by the same loop as the scheduled pass (see
//! `collector`), and a request waits for the whole pass: it re-verifies every
//! file a project reaches before removing anything, which on a large library
//! takes minutes, not a request's usual moment.

use std::time::{Duration, SystemTime};

use clipmill_contracts::proto::ipc::v1::{
    CleanStorageRequest, CleanStorageResponse, ErrorCode, GetStorageStatsResponse,
    StorageCategoryV1, response,
};

use super::{Reply, Service, error_reply, response_reply};
use crate::{
    collector::{CleanUpError, Collector},
    storage::{self, Category, Report},
};

/// The youngest an unreferenced object may be and still be collected by a
/// clean-up somebody asked for. A few minutes cover the moment between an
/// object being published and its root being written, when nothing references
/// it yet; the scheduled collection keeps its much longer retention period.
pub(crate) const CLEAN_GRACE: Duration = Duration::from_mins(15);

impl Service {
    pub(crate) fn with_collector(mut self, collector: Collector) -> Self {
        self.collector = Some(collector);
        self
    }

    /// What this installation is using on disk, by category.
    ///
    /// The store answers for artifacts from manifests it already holds. The
    /// rest are directory walks, so the whole measurement goes to a blocking
    /// thread — a screen asking how much disk it is using must never be the
    /// thing that stalls the event loop.
    pub(super) async fn get_storage_stats(&self, request_id: String) -> Reply {
        match self.measure_storage().await {
            Ok(stats) => response_reply(request_id, response::Body::GetStorageStats(stats)),
            Err((code, message)) => error_reply(request_id, code, message),
        }
    }

    pub(super) async fn clean_storage(
        &self,
        request_id: String,
        asked: &CleanStorageRequest,
    ) -> Reply {
        let freed = match asked.action.as_str() {
            "unused_files" => self.clean_unused_files().await,
            "temporary" => self.clean_temporary().await,
            "backups" => self.clean_backups().await,
            _ => Err((
                ErrorCode::InvalidArgument,
                "choose unused_files, temporary or backups".to_owned(),
            )),
        };
        let freed = match freed {
            Ok(freed) => freed,
            Err((code, message)) => return error_reply(request_id, code, message),
        };
        tracing::info!(
            action = asked.action,
            freed_bytes = freed.bytes,
            removed = freed.items,
            "storage cleaned"
        );
        let storage = self.measure_storage().await.ok();
        response_reply(
            request_id,
            response::Body::CleanStorage(CleanStorageResponse {
                freed_bytes: freed.bytes,
                removed_items: freed.items,
                storage,
            }),
        )
    }

    async fn measure_storage(&self) -> Result<GetStorageStatsResponse, (ErrorCode, String)> {
        let (Some(artifacts), Some(dirs)) = (self.artifacts.as_ref(), self.storage.clone()) else {
            return Err((
                ErrorCode::Unavailable,
                "this daemon measures no storage".to_owned(),
            ));
        };
        let usage = artifacts.usage().await.map_err(|_| {
            (
                ErrorCode::Unavailable,
                "the artifact store is not answering".to_owned(),
            )
        })?;
        // An estimate, so a failure to read the roots is shown as unknown
        // rather than failing the whole report.
        let reclaimable = match self.database.list_artifact_roots().await {
            Ok(roots) => artifacts
                .reclaimable(roots, SystemTime::now(), CLEAN_GRACE)
                .await
                .ok(),
            Err(_) => None,
        };
        let report = tokio::task::spawn_blocking(move || dirs.measure(usage))
            .await
            .map_err(|_| {
                (
                    ErrorCode::Internal,
                    "the storage measurement did not finish".to_owned(),
                )
            })?;
        Ok(storage_response(
            &report,
            self.retention_grace,
            reclaimable.map(|usage| Category {
                bytes: usage.bytes,
                items: usage.objects,
            }),
        ))
    }

    /// Collect every generated file no root reaches, without waiting out the
    /// retention period. The store re-verifies every reachable manifest and
    /// keeps whatever a reader holds, exactly as the scheduled pass does.
    ///
    /// The answer is the finished pass. One that yields to other work carries
    /// on where it stopped, so a busy engine makes this slower, never a
    /// clean-up that reports nothing freed while it left everything in place.
    async fn clean_unused_files(&self) -> Result<Category, (ErrorCode, String)> {
        let collector = self.collector.as_ref().ok_or((
            ErrorCode::Unavailable,
            "this daemon collects no artifacts".to_owned(),
        ))?;
        let freed = collector
            .clean_up(CLEAN_GRACE)
            .await
            .map_err(|error| match error {
                CleanUpError::Roots => (
                    ErrorCode::Unavailable,
                    "ClipMill could not read what your projects use, so nothing was removed."
                        .to_owned(),
                ),
                CleanUpError::Refused(detail) => (
                    ErrorCode::Conflict,
                    format!("The clean-up stopped to keep your projects safe: {detail}"),
                ),
                CleanUpError::Stopped => (
                    ErrorCode::Unavailable,
                    "ClipMill stopped before the clean-up finished.".to_owned(),
                ),
            })?;
        Ok(Category {
            bytes: freed.bytes,
            items: freed.objects,
        })
    }

    async fn clean_temporary(&self) -> Result<Category, (ErrorCode, String)> {
        let dirs = self.storage.clone().ok_or((
            ErrorCode::Unavailable,
            "this daemon measures no storage".to_owned(),
        ))?;
        let downloading = self
            .library
            .as_ref()
            .map(|library| library.busy_models())
            .unwrap_or_default();
        tokio::task::spawn_blocking(move || {
            dirs.clean_temporary(SystemTime::now(), storage::SCRATCH_QUIET, &downloading)
        })
        .await
        .map_err(|_| {
            (
                ErrorCode::Internal,
                "the clean-up did not finish".to_owned(),
            )
        })
    }

    async fn clean_backups(&self) -> Result<Category, (ErrorCode, String)> {
        let dirs = self.storage.clone().ok_or((
            ErrorCode::Unavailable,
            "this daemon measures no storage".to_owned(),
        ))?;
        tokio::task::spawn_blocking(move || dirs.clean_backups(1))
            .await
            .map_err(|_| {
                (
                    ErrorCode::Internal,
                    "the clean-up did not finish".to_owned(),
                )
            })
    }
}

fn storage_response(
    report: &Report,
    retention: Duration,
    reclaimable: Option<Category>,
) -> GetStorageStatsResponse {
    let category = |key: &str, measured: Category, path: &std::path::Path| StorageCategoryV1 {
        key: key.to_owned(),
        bytes: measured.bytes,
        items: measured.items,
        path: path.to_string_lossy().into_owned(),
    };
    GetStorageStatsResponse {
        categories: vec![
            // The original four keep their places; callers read them as a list.
            category(
                storage::ARTIFACTS,
                report.artifacts,
                &report.paths.artifacts,
            ),
            category(storage::MODELS, report.models, &report.paths.models),
            category(storage::STATE, report.state, &report.paths.state),
            category(storage::IMPORTS, report.imports, &report.paths.imports),
            category(storage::BACKUPS, report.backups, &report.paths.backups),
            category(
                storage::TEMPORARY,
                report.temporary,
                &report.paths.temporary,
            ),
        ],
        available_bytes: report.available_bytes.unwrap_or(0),
        available_known: report.available_bytes.is_some(),
        retention_grace_seconds: retention.as_secs(),
        reclaimable_bytes: reclaimable.map_or(0, |category| category.bytes),
        reclaimable_items: reclaimable.map_or(0, |category| category.items),
        reclaimable_known: reclaimable.is_some(),
    }
}
