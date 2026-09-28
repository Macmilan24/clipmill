//! The processing engine's requests, and the request that stops the daemon.
//!
//! Like the model library, every engine request answers with the engine as it
//! then stands. A daemon without a managed engine (a development checkout)
//! answers `managed: false` and refuses to install anything, because its
//! workers are started by hand.

use std::sync::Arc;

use clipmill_contracts::proto::ipc::v1::{
    EngineResponse, ErrorCode, GetDeviceProfileRequest, InstallEngineRequest, ShutdownResponse,
    response,
};
use prost::Message as _;
use sha2::{Digest as _, Sha256};

use super::{Reply, Service, error_reply, response_reply};
use crate::engine::{Engine, Refusal};

/// The most parts one request may name.
const MAX_PARTS: usize = 32;

impl Service {
    /// Give the service the engine a packaged app installs and runs.
    #[must_use]
    pub(crate) fn with_engine(mut self, engine: Arc<Engine>) -> Self {
        self.engine = Some(engine);
        self
    }

    /// Run `YouTube` imports through `helper` instead of the checkout's script:
    /// a packaged app's engine launcher for its `YouTube` import component.
    #[must_use]
    pub(crate) fn with_youtube_helper(mut self, helper: std::path::PathBuf) -> Self {
        if let (Some(decoder), Some(storage)) = (&self.decoder, &self.storage) {
            self.youtube = Some(Arc::new(super::youtube::YoutubeRuntime::new(
                crate::youtube_transport::YoutubeDownloader::new(helper, decoder.clone()),
                storage.data.join("imports"),
            )));
        }
        self
    }

    /// What a shutdown request wakes; the daemon's serve loop waits on it.
    pub(crate) fn shutdown_requested(&self) -> Arc<tokio::sync::Notify> {
        Arc::clone(&self.shutdown)
    }

    pub(crate) fn engine(&self) -> Option<&Arc<Engine>> {
        self.engine.as_ref()
    }

    fn engine_reply(&self, request_id: String) -> Reply {
        let listing = self
            .engine
            .as_ref()
            .map_or_else(EngineResponse::default, |engine| engine.list());
        response_reply(request_id, response::Body::Engine(listing))
    }

    pub(super) fn get_engine(&self, request_id: String) -> Reply {
        self.engine_reply(request_id)
    }

    pub(super) fn install_engine(&self, request_id: String, asked: &InstallEngineRequest) -> Reply {
        let Some(engine) = &self.engine else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "this ClipMill runs from source: install workers with just setup and start them with just workers",
            );
        };
        if asked.parts.len() > MAX_PARTS {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "name at most 32 engine parts",
            );
        }
        match engine.install(&asked.parts) {
            Ok(()) => self.engine_reply(request_id),
            Err(refusal) => {
                let code = match refusal {
                    Refusal::NotFound(_) => ErrorCode::NotFound,
                    Refusal::Unavailable(_) => ErrorCode::Unavailable,
                };
                error_reply(request_id, code, refusal.message())
            }
        }
    }

    pub(super) fn cancel_engine_install(&self, request_id: String) -> Reply {
        if let Some(engine) = &self.engine {
            engine.cancel();
        }
        self.engine_reply(request_id)
    }

    /// Answer, then let the serve loop stop the daemon the way a signal does.
    pub(super) fn shutdown(&self, request_id: String) -> Reply {
        tracing::info!("stopping because the app asked");
        self.shutdown.notify_one();
        response_reply(request_id, response::Body::Shutdown(ShutdownResponse {}))
    }

    /// On a Mac, prove the planned editorial model runs here once its
    /// component and weights are both installed, then measure the device again
    /// so the scheduler admits the Metal workers.
    ///
    /// A development checkout does this with `tools/editorial-runtime-check.py`
    /// from `just workers`. Here it is tried once per machine, model and
    /// install; a failure stays on the editorial component until one of them
    /// changes. A receipt that already matches is only re-applied, when the
    /// scheduler has not yet seen it.
    pub(crate) async fn prove_editorial_runtime(&self) {
        if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            return;
        }
        let (Some(engine), Some(storage), Some(profiler)) =
            (&self.engine, &self.storage, &self.device_profiler)
        else {
            return;
        };
        let Some((install, _)) = engine.editorial_install() else {
            return;
        };
        let Some(implementation) = self.editorial_implementation("editorial-propose") else {
            return;
        };
        if !self
            .model_files_present(implementation.model, &storage.weights)
            .0
        {
            return;
        }
        let (Some(manifest), Some(binding)) = (
            self.models.get(implementation.model),
            self.models.binding(implementation.model, &storage.weights),
        ) else {
            return;
        };
        let Ok(fingerprint) = profiler.hardware_fingerprint().await else {
            return;
        };
        let digest = format!("sha256:{}", manifest.digest());
        let receipt = storage.state.join("editorial-runtime.json");
        let key = format!("{fingerprint}/{digest}/{install}");
        let admitted = self.scheduler.as_ref().is_some_and(|scheduler| {
            scheduler.machine_capacity().accelerator_mask
                & crate::jobs::accelerator_bit("metal").unwrap_or(0)
                != 0
        });
        if receipt_matches(&receipt, &fingerprint, &digest) {
            if !admitted && !engine.proof_tried(&key) {
                engine.mark_proof_tried(&key);
                self.remeasure_device().await;
            }
            return;
        }
        if engine.proof_tried(&key) {
            return;
        }
        tracing::info!(
            model = implementation.model,
            "proving the editorial model runs here"
        );
        match engine
            .prove_editorial(&key, &binding.encode_to_vec(), &receipt, &fingerprint)
            .await
        {
            Ok(()) => {
                tracing::info!(
                    model = implementation.model,
                    "the editorial model ran; measuring again"
                );
                self.remeasure_device().await;
            }
            Err(message) => {
                tracing::warn!(
                    model = implementation.model,
                    message,
                    "the editorial model check failed"
                );
            }
        }
    }

    /// Measure the device again, as Models' Rescan does, so a new receipt
    /// reaches the scheduler.
    async fn remeasure_device(&self) {
        let request_id = format!("engine-remeasure-{}", ulid::Ulid::new());
        let request = GetDeviceProfileRequest { remeasure: true };
        let mut hasher = Sha256::new();
        hasher.update(request_id.as_bytes());
        hasher.update(request.encode_to_vec());
        let hash: [u8; 32] = hasher.finalize().into();
        let reply = self.get_device_profile(request_id, hash, &request).await;
        if reply.outcome != super::Outcome::Success {
            tracing::warn!("measuring the device after the editorial check did not complete");
        }
    }
}

/// Whether `receipt` proves the model with `digest` ran on this machine, as
/// the device measurement will judge it (selection.rs).
fn receipt_matches(receipt: &std::path::Path, fingerprint: &str, digest: &str) -> bool {
    let Ok(bytes) = std::fs::read(receipt) else {
        return false;
    };
    let Ok(proof) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    proof["schema_version"] == "clipmill.editorial.runtime.v1"
        && proof["runtime"] == "mlx-vlm@0.7.1/clipmill-json-v2"
        && proof["hardware_fingerprint"] == fingerprint
        && proof["model_digest"] == digest
        && proof["validated"] == true
}
