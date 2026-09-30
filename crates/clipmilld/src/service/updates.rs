//! The app's "a new version is out" question.

use clipmill_contracts::proto::ipc::v1::{CheckForUpdateResponse, ErrorCode, response};

use super::{Reply, Service, error_reply, response_reply};
use crate::updates::{ReleaseCheck, Version};

impl Service {
    /// Which release is newest, and whether it is newer than this one.
    pub(super) async fn check_for_update(&self, request_id: String) -> Reply {
        let Some(current) = Version::current() else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "This build has no release version to compare.",
            );
        };
        // Started, so counted, whatever GitHub answers.
        self.policy.note_task_start("update-check");
        let latest = match ReleaseCheck::new() {
            Ok(check) => check.latest().await,
            Err(error) => Err(error),
        };
        match latest {
            Ok(latest) => response_reply(
                request_id,
                response::Body::CheckForUpdate(CheckForUpdateResponse {
                    current_version: current.to_string(),
                    latest_version: latest.to_string(),
                    newer: latest > current,
                }),
            ),
            Err(error) => error_reply(request_id, ErrorCode::Unavailable, error.to_string()),
        }
    }
}
