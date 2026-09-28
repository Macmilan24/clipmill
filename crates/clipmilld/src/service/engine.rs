//! The processing engine's requests, and the request that stops the daemon.
//!
//! Like the model library, every engine request answers with the engine as it
//! then stands. A daemon without a managed engine (a development checkout)
//! answers `managed: false` and refuses to install anything, because its
//! workers are started by hand.

use std::sync::Arc;

use clipmill_contracts::proto::ipc::v1::{
    EngineResponse, ErrorCode, InstallEngineRequest, ShutdownResponse, response,
};

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
}
