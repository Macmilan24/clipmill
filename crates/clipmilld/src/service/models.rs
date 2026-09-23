//! The model library's requests: listing, acquiring, removing and choosing.
//!
//! Every mutation answers with the library as it then stands, so a screen
//! never has to guess what its click did; downloads and checks carry on in
//! the background and the next listing says how far they got.

use std::sync::Arc;

use clipmill_contracts::proto::ipc::v1::{
    AddCustomModelRequest, CancelModelDownloadRequest, DownloadModelsRequest, ErrorCode,
    ForgetModelRequest, InspectHubModelRequest, RemoveModelRequest, SetModelChoiceRequest,
    VerifyModelRequest, response,
};

use super::{Reply, Service, error_reply, response_reply};
use crate::{
    implementations::{self, Implementation},
    library::{self, ModelLibrary, Refusal},
    selection::{Binding, Bindings},
};

impl Service {
    /// Give the service its model library. Separate from construction so the
    /// tests that build a service without a workspace need not invent one.
    #[must_use]
    pub(crate) fn with_library(mut self, library: Arc<ModelLibrary>) -> Self {
        self.library = Some(library);
        self
    }

    pub(crate) async fn stop_model_library(&self) {
        if let Some(library) = &self.library {
            library.stop().await;
        }
    }

    /// What the last verified device profile bound, before the library.
    fn profile_bindings(&self) -> Bindings {
        self.scheduler
            .as_ref()
            .map(crate::jobs::SchedulerHandle::bindings)
            .unwrap_or_default()
    }

    /// The bindings a plan is built against, and readiness is judged by.
    ///
    /// Resolved once per plan, and written into its tasks: a choice made while
    /// an analysis runs changes the next analysis, never the one in flight.
    pub(crate) fn planning_bindings(&self) -> Bindings {
        let raw = self.profile_bindings();
        match &self.library {
            Some(library) => library.effective_bindings(&raw),
            None => with_default_editorial(raw),
        }
    }

    /// The implementation the next editorial task of `stage` would run.
    pub(crate) fn editorial_implementation(&self, stage: &str) -> Option<&'static Implementation> {
        self.planning_bindings()
            .for_stage(stage)
            .and_then(|binding| implementations::lookup(&binding.implementation))
            .filter(|implementation| implementation.stage == stage)
            .or_else(|| implementations::candidates_for_stage(stage).next())
    }

    /// The memory an analysis may use, as admission counts it.
    fn memory_budget(&self) -> u64 {
        self.scheduler
            .as_ref()
            .map_or(0, |scheduler| scheduler.machine_capacity().ram_bytes)
    }

    fn library_reply(&self, request_id: String, library: &ModelLibrary) -> Reply {
        response_reply(
            request_id,
            response::Body::ModelLibrary(
                library.list(&self.profile_bindings(), self.memory_budget()),
            ),
        )
    }

    fn library_or_refuse(&self, request_id: &str) -> Result<Arc<ModelLibrary>, Reply> {
        self.library.clone().ok_or_else(|| {
            error_reply(
                request_id.to_owned(),
                ErrorCode::Unavailable,
                "this daemon has no model library",
            )
        })
    }

    pub(super) fn list_models(&self, request_id: String) -> Reply {
        match self.library_or_refuse(&request_id) {
            Ok(library) => self.library_reply(request_id, &library),
            Err(reply) => reply,
        }
    }

    pub(super) fn download_models(
        &self,
        request_id: String,
        asked: &DownloadModelsRequest,
    ) -> Reply {
        let library = match self.library_or_refuse(&request_id) {
            Ok(library) => library,
            Err(reply) => return reply,
        };
        if asked.names.is_empty() || asked.names.len() > 64 {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "name between one and 64 models to download",
            );
        }
        match library.download(&asked.names) {
            Ok(()) => self.library_reply(request_id, &library),
            Err(refusal) => refusal_reply(request_id, &refusal),
        }
    }

    pub(super) fn cancel_model_download(
        &self,
        request_id: String,
        asked: &CancelModelDownloadRequest,
    ) -> Reply {
        let library = match self.library_or_refuse(&request_id) {
            Ok(library) => library,
            Err(reply) => return reply,
        };
        library.cancel(&asked.name);
        self.library_reply(request_id, &library)
    }

    pub(super) async fn remove_model(
        &self,
        request_id: String,
        asked: &RemoveModelRequest,
    ) -> Reply {
        let library = match self.library_or_refuse(&request_id) {
            Ok(library) => library,
            Err(reply) => return reply,
        };
        match library.remove(&asked.name).await {
            Ok(_) => self.library_reply(request_id, &library),
            Err(refusal) => refusal_reply(request_id, &refusal),
        }
    }

    pub(super) fn verify_model(&self, request_id: String, asked: &VerifyModelRequest) -> Reply {
        let library = match self.library_or_refuse(&request_id) {
            Ok(library) => library,
            Err(reply) => return reply,
        };
        match library.verify(&asked.name) {
            Ok(()) => self.library_reply(request_id, &library),
            Err(refusal) => refusal_reply(request_id, &refusal),
        }
    }

    pub(super) fn set_model_choice(
        &self,
        request_id: String,
        asked: &SetModelChoiceRequest,
    ) -> Reply {
        let library = match self.library_or_refuse(&request_id) {
            Ok(library) => library,
            Err(reply) => return reply,
        };
        match library.set_choice(&asked.capability, &asked.model) {
            Ok(()) => self.library_reply(request_id, &library),
            Err(refusal) => refusal_reply(request_id, &refusal),
        }
    }

    pub(super) async fn inspect_hub_model(
        &self,
        request_id: String,
        asked: &InspectHubModelRequest,
    ) -> Reply {
        let library = match self.library_or_refuse(&request_id) {
            Ok(library) => library,
            Err(reply) => return reply,
        };
        let answer = library
            .inspect(&asked.repo, &asked.revision, &asked.capability)
            .await;
        response_reply(request_id, response::Body::InspectHubModel(answer))
    }

    pub(super) async fn add_custom_model(
        &self,
        request_id: String,
        asked: &AddCustomModelRequest,
    ) -> Reply {
        let library = match self.library_or_refuse(&request_id) {
            Ok(library) => library,
            Err(reply) => return reply,
        };
        let request = library::AddRequest {
            repo: asked.repo.trim(),
            commit: asked.commit.trim(),
            capability: &asked.capability,
            weights_file: &asked.weights_file,
            name: asked.name.trim(),
            title: &asked.title,
        };
        match library.add_custom(request).await {
            Ok(()) => self.library_reply(request_id, &library),
            Err(refusal) => refusal_reply(request_id, &refusal),
        }
    }

    pub(super) async fn forget_model(
        &self,
        request_id: String,
        asked: &ForgetModelRequest,
    ) -> Reply {
        let library = match self.library_or_refuse(&request_id) {
            Ok(library) => library,
            Err(reply) => return reply,
        };
        match library.forget(&asked.name).await {
            Ok(_) => self.library_reply(request_id, &library),
            Err(refusal) => refusal_reply(request_id, &refusal),
        }
    }
}

/// A service with no library still plans editorial stages against the
/// bundled model, exactly as it did before the library existed.
fn with_default_editorial(mut bindings: Bindings) -> Bindings {
    if let Some(job) = library::job("editorial") {
        for stage in job.stages {
            if bindings.for_stage(stage).is_none()
                && let Some(implementation) = implementations::candidates_for_stage(stage)
                    .find(|implementation| !implementation.opt_in)
            {
                bindings.set(Binding::decided(implementation, "default"));
            }
        }
    }
    bindings
}

fn refusal_reply(request_id: String, refusal: &Refusal) -> Reply {
    let code = match refusal {
        Refusal::NotFound(_) => ErrorCode::NotFound,
        Refusal::Invalid(_) => ErrorCode::InvalidArgument,
        Refusal::Conflict(_) => ErrorCode::Conflict,
        Refusal::Unavailable(_) => ErrorCode::Unavailable,
    };
    error_reply(request_id, code, refusal.message())
}
