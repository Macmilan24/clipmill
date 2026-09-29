//! The model library: what can run here, what is installed, and getting it.
//!
//! A fresh installation has pinned manifests and no weights. This is where a
//! person sees every model a ClipMill worker can run, downloads the ones they
//! want, removes the ones they do not, and says which one does each job.
//!
//! Acquisition is an explicit network operation the person starts, never a
//! side effect of editing or analysing: the Local Lock counts every download
//! and every repository inspection. Every file is checked against its pinned
//! SHA-256 before it may exist under its final name, and workers hash it again
//! before loading. Nothing here is fitted to one machine: every model is
//! listed on every device, and one that needs more memory than this device has
//! is flagged, not hidden.
//!
//! The library also decides which model an analysis plans for each job. The
//! signed device profile still says what a benchmark measured (D19); a
//! person's explicit choice comes first, and a default whose weights are not
//! installed yields to a candidate whose weights are — so removing a model
//! never silently strands an analysis on a file that is not there.

mod custom;
#[cfg(test)]
mod tests;

pub(crate) use custom::AddRequest;

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use clipmill_contracts::proto::ipc::v1::{
    InspectHubModelResponse, ListModelsResponse, ModelDownloadV1, ModelJobV1, ModelV1,
};
use clipmill_hub::{Hub, PinnedFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{Notify, watch};

use crate::{
    db::DbHandle,
    implementations::{self, Implementation},
    models::{HUGGING_FACE, ModelManifest, ModelOrigin, ModelRegistry},
    policy::LocalLockPolicy,
    selection::{Binding, Bindings},
};

const CHOICES_SCHEMA: &str = "clipmill.model_choices.v1";
/// Where downloads in flight live, beside the models they will become. Same
/// volume, so installing a verified file is a rename rather than a copy.
const STAGING: &str = ".partial";
/// Progress is published at most this often in bytes, so a fast link does not
/// take the library's lock for every network chunk.
const PROGRESS_STEP: u64 = 1024 * 1024;
const HASH_BUFFER: usize = 1024 * 1024;

/// A job an analysis runs, in the order the library presents them.
#[derive(Debug)]
pub(crate) struct Job {
    pub capability: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    /// The stages it serves. The first is the one its models are listed by.
    pub stages: &'static [&'static str],
}

pub(crate) const JOBS: &[Job] = &[
    Job {
        capability: "editorial",
        title: "Editorial AI",
        summary: "Finds complete moments, reviews them in context, checks visual references and drafts titles.",
        stages: &[
            "editorial-propose",
            "editorial-review",
            "editorial-look",
            "youtube-metadata",
        ],
    },
    Job {
        capability: "asr",
        title: "Transcription",
        summary: "Turns speech into words.",
        stages: &["speech-asr"],
    },
    Job {
        capability: "forced-align",
        title: "Word timing",
        summary: "Times each word against the audio, so captions land exactly.",
        stages: &["speech-align"],
    },
    Job {
        capability: "vad",
        title: "Speech detection",
        summary: "Finds where people speak, so nothing else is transcribed.",
        stages: &["speech-vad"],
    },
    Job {
        capability: "speaker-embed",
        title: "Telling voices apart",
        summary: "Hears who speaks when, so transcripts name each voice.",
        stages: &["speech-speakers"],
    },
    Job {
        capability: "detect-faces",
        title: "Face tracking",
        summary: "Finds faces, so vertical crops follow the speaker.",
        stages: &["detect-faces"],
    },
];

pub(crate) fn job(capability: &str) -> Option<&'static Job> {
    JOBS.iter().find(|job| job.capability == capability)
}

/// Where a model's weights stand on disk, from the pinned sizes alone. The
/// digest is checked by a download, by Verify, and by the worker at load.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Install {
    Installed,
    Partial,
    Missing,
}

impl Install {
    fn as_str(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Partial => "partial",
            Self::Missing => "missing",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Queued,
    Downloading,
    Verifying,
    Failed,
    Cancelled,
}

impl Phase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Downloading => "downloading",
            Self::Verifying => "verifying",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn busy(self) -> bool {
        matches!(self, Self::Queued | Self::Downloading | Self::Verifying)
    }
}

#[derive(Clone, Debug)]
struct Progress {
    phase: Phase,
    received: u64,
    total: u64,
    current: String,
    error: String,
    updated: u64,
}

impl Progress {
    fn new(phase: Phase, total: u64) -> Self {
        Self {
            phase,
            received: 0,
            total,
            current: String::new(),
            error: String::new(),
            updated: unix_millis(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Work {
    Download(String),
    Verify(String),
}

impl Work {
    fn model(&self) -> &str {
        match self {
            Self::Download(name) | Self::Verify(name) => name,
        }
    }
}

#[derive(Debug, Default)]
struct State {
    /// Capability → model name: the person's explicit choices.
    choices: BTreeMap<String, String>,
    /// Model → the latest attempt, while busy and after a failure or cancel.
    progress: BTreeMap<String, Progress>,
    queue: VecDeque<Work>,
    active: Option<(String, watch::Sender<bool>)>,
}

/// Why the library refused. The message is one sentence a screen can show.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Refusal {
    NotFound(String),
    Invalid(String),
    Conflict(String),
    Unavailable(String),
}

impl Refusal {
    pub(crate) fn message(&self) -> &str {
        match self {
            Self::NotFound(message)
            | Self::Invalid(message)
            | Self::Conflict(message)
            | Self::Unavailable(message) => message,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ChoicesDocument {
    schema_version: String,
    #[serde(default)]
    choices: BTreeMap<String, String>,
}

/// Where the library keeps what it owns.
#[derive(Clone, Debug)]
pub(crate) struct LibraryPaths {
    /// Installed weights, one directory per model name.
    pub weights: PathBuf,
    /// Manifests the person pinned, one `<name>.json` each.
    pub custom: PathBuf,
    /// The person's per-job choices.
    pub choices: PathBuf,
}

#[derive(Debug)]
pub(crate) struct ModelLibrary {
    registry: Arc<ModelRegistry>,
    paths: LibraryPaths,
    database: Option<DbHandle>,
    policy: Arc<LocalLockPolicy>,
    hub: Option<Hub>,
    /// Physical memory, measured once. Fixed hardware; not a live reading.
    total_memory: u64,
    state: Mutex<State>,
    wake: Notify,
    stopping: AtomicBool,
    runner: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl ModelLibrary {
    /// Build the library and start its download runner.
    ///
    /// Every model the person pinned is made plannable here, before any
    /// request can name one. Must be called inside a Tokio runtime.
    pub(crate) fn start(
        registry: Arc<ModelRegistry>,
        paths: LibraryPaths,
        database: Option<DbHandle>,
        policy: Arc<LocalLockPolicy>,
        total_memory: u64,
    ) -> Arc<Self> {
        let hub = match Hub::new() {
            Ok(hub) => Some(hub),
            Err(error) => {
                tracing::warn!(%error, "model downloads are unavailable");
                None
            }
        };
        Self::start_with_hub(registry, paths, database, policy, total_memory, hub)
    }

    /// The same, with the transport supplied: a loopback fixture in tests,
    /// none at all where downloads must be refused.
    pub(crate) fn start_with_hub(
        registry: Arc<ModelRegistry>,
        paths: LibraryPaths,
        database: Option<DbHandle>,
        policy: Arc<LocalLockPolicy>,
        total_memory: u64,
        hub: Option<Hub>,
    ) -> Arc<Self> {
        for manifest in registry.manifests() {
            if manifest.origin == ModelOrigin::Custom
                && let Err(detail) =
                    implementations::register_custom(&manifest.name, &manifest.capability)
            {
                tracing::warn!(
                    model = manifest.name,
                    detail,
                    "a pinned model cannot be planned"
                );
                registry.unregister(&manifest.name);
            }
        }
        let choices = read_choices(&paths.choices);
        let library = Arc::new(Self {
            registry,
            paths,
            database,
            policy,
            hub,
            total_memory,
            state: Mutex::new(State {
                choices,
                ..State::default()
            }),
            wake: Notify::new(),
            stopping: AtomicBool::new(false),
            runner: Mutex::new(None),
        });
        let runner = tokio::spawn(Arc::clone(&library).run());
        *library
            .runner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(runner);
        library
    }

    /// Cancel whatever is running and wait, briefly, for it to stop. What a
    /// cancelled download received stays on disk for the next attempt.
    pub(crate) async fn stop(&self) {
        self.stopping.store(true, Ordering::SeqCst);
        {
            let mut state = self.lock();
            state.queue.clear();
            if let Some((_, cancel)) = &state.active {
                let _ = cancel.send(true);
            }
        }
        self.wake.notify_one();
        let runner = self
            .runner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(runner) = runner
            && tokio::time::timeout(Duration::from_secs(10), runner)
                .await
                .is_err()
        {
            tracing::warn!("the model download runner did not stop in time");
        }
    }

    /// The whole library as it stands now.
    ///
    /// `raw` is what the signed device profile bound, and `budget` the memory
    /// an analysis may use as the scheduler counts it; both change with the
    /// device, which is why they are arguments rather than state.
    pub(crate) fn list(&self, raw: &Bindings, budget: u64) -> ListModelsResponse {
        let manifests = self.registry.manifests();
        let effective = self.effective_bindings(raw);
        let (choices, progress) = {
            let state = self.lock();
            (state.choices.clone(), state.progress.clone())
        };
        let in_use = JOBS
            .iter()
            .filter_map(|job| effective.for_stage(job.stages[0]))
            .map(|binding| binding.model.clone())
            .collect::<BTreeSet<_>>();
        let mut recommended_missing = Vec::new();
        let mut recommended_missing_bytes = 0_u64;
        let mut models = Vec::with_capacity(manifests.len());
        let better = JOBS
            .iter()
            .filter_map(|job| {
                self.more_accurate_here(job, &manifests, &effective, budget)
                    .map(|model| (job.capability, model))
            })
            .collect::<BTreeMap<_, _>>();
        for manifest in &manifests {
            let (install, installed_bytes) = self.install_state(manifest);
            let supported = supported(manifest);
            if !self.offered(manifest) {
                continue;
            }
            if manifest.catalog.recommended
                && manifest.origin == ModelOrigin::Bundled
                && supported.is_ok()
                && install != Install::Installed
            {
                recommended_missing.push(manifest.name.clone());
                recommended_missing_bytes = recommended_missing_bytes
                    .saturating_add(manifest.download_bytes().saturating_sub(installed_bytes));
            }
            let mut row = self.model_row(
                manifest,
                (install, installed_bytes),
                supported,
                budget,
                progress.get(&manifest.name),
                in_use.contains(&manifest.name),
            );
            row.recommended |= better.values().any(|model| *model == manifest.name);
            models.push(row);
        }
        let jobs = JOBS
            .iter()
            .map(|job| {
                let binding = effective.for_stage(job.stages[0]);
                let mut listed = manifests
                    .iter()
                    .filter(|manifest| manifest.capability == job.capability)
                    .filter(|manifest| self.offered(manifest))
                    .filter(|manifest| {
                        implementations::for_stage_and_model(job.stages[0], &manifest.name)
                            .is_some()
                    })
                    .collect::<Vec<_>>();
                // Bundled before pinned, recommended first, then the smaller.
                listed.sort_by_key(|manifest| {
                    (
                        manifest.origin == ModelOrigin::Custom,
                        !manifest.catalog.recommended,
                        manifest.download_bytes(),
                    )
                });
                ModelJobV1 {
                    capability: job.capability.to_owned(),
                    title: job.title.to_owned(),
                    summary: job.summary.to_owned(),
                    model: binding
                        .map(|binding| binding.model.clone())
                        .unwrap_or_default(),
                    selected_by: binding
                        .map_or("unavailable", |binding| {
                            display_reason(&binding.selected_by)
                        })
                        .to_owned(),
                    choice: choices.get(job.capability).cloned().unwrap_or_default(),
                    models: listed
                        .into_iter()
                        .map(|manifest| manifest.name.clone())
                        .collect(),
                    more_accurate: better.get(job.capability).cloned().unwrap_or_default(),
                }
            })
            .collect();
        let available = available_space(&self.paths.weights);
        ListModelsResponse {
            models,
            jobs,
            install_path: self.paths.weights.to_string_lossy().into_owned(),
            available_bytes: available.unwrap_or(0),
            available_known: available.is_some(),
            memory_total_bytes: self.total_memory,
            memory_budget_bytes: budget,
            recommended_missing_bytes,
            recommended_missing,
        }
    }

    /// Whether the library offers a model on this computer: one it can run,
    /// or one some of which is on disk, where the list is how it gets
    /// removed. A Windows PC is not offered the Mac's MLX models at all.
    fn offered(&self, manifest: &ModelManifest) -> bool {
        supported(manifest).is_ok() || self.install_state(manifest).0 != Install::Missing
    }

    /// The most accurate model for a job that this computer can run and hold
    /// in memory, when it is more accurate than the one analyses plan: shown
    /// as recommended here with a note, so the choice is never capped at the
    /// floor every machine can run — and never made for the person either.
    /// Ties go to one measurement may choose, then to the smaller download.
    fn more_accurate_here(
        &self,
        job: &Job,
        manifests: &[Arc<ModelManifest>],
        effective: &Bindings,
        budget: u64,
    ) -> Option<String> {
        let stage = job.stages[0];
        let accuracy = |model: &str| {
            implementations::for_stage_and_model(stage, model)
                .map_or(0, |implementation| implementation.accuracy)
        };
        let planned = effective
            .for_stage(stage)
            .map_or(0, |binding| accuracy(&binding.model));
        manifests
            .iter()
            .filter(|manifest| {
                manifest.capability == job.capability
                    && manifest.origin == ModelOrigin::Bundled
                    && supported(manifest).is_ok()
                    && memory_fit(manifest.memory.resident_bytes(), budget, self.total_memory)
                        == "fits"
            })
            .filter_map(|manifest| {
                let implementation = implementations::for_stage_and_model(stage, &manifest.name)?;
                (implementation.accuracy > planned).then_some((implementation, manifest))
            })
            .max_by(|(left, left_manifest), (right, right_manifest)| {
                left.accuracy
                    .cmp(&right.accuracy)
                    .then(right.opt_in.cmp(&left.opt_in))
                    .then(
                        right_manifest
                            .download_bytes()
                            .cmp(&left_manifest.download_bytes()),
                    )
            })
            .map(|(_, manifest)| manifest.name.clone())
    }

    fn model_row(
        &self,
        manifest: &ModelManifest,
        (install, installed_bytes): (Install, u64),
        supported: Result<(), &'static str>,
        budget: u64,
        progress: Option<&Progress>,
        in_use: bool,
    ) -> ModelV1 {
        let memory = manifest.memory.resident_bytes();
        let worker = implementations::for_model(&manifest.name).map(|found| found.worker);
        ModelV1 {
            name: manifest.name.clone(),
            title: manifest.title().to_owned(),
            summary: manifest.catalog.summary.clone(),
            capability: manifest.capability.clone(),
            runtime: manifest.runtime.clone(),
            backend: manifest.backend.clone(),
            quantization: manifest.quantization.clone(),
            license_spdx: manifest.license.spdx.clone(),
            source_repo: manifest.source.repo.clone(),
            source_revision: manifest.source.revision.clone(),
            download_bytes: manifest.download_bytes(),
            memory_bytes: memory,
            recommended: manifest.catalog.recommended,
            supported: supported.is_ok(),
            unsupported_reason: supported.err().unwrap_or_default().to_owned(),
            memory_fit: memory_fit(memory, budget, self.total_memory).to_owned(),
            custom: manifest.origin == ModelOrigin::Custom,
            install_state: install.as_str().to_owned(),
            installed_bytes,
            download: progress.map(|entry| ModelDownloadV1 {
                state: entry.phase.as_str().to_owned(),
                received_bytes: entry.received,
                total_bytes: entry.total,
                current_file: entry.current.clone(),
                error: entry.error.clone(),
                updated_unix_millis: entry.updated,
            }),
            in_use,
            worker: worker.unwrap_or_default().to_owned(),
            worker_title: worker
                .map(implementations::worker_title)
                .unwrap_or_default()
                .to_owned(),
            // Whoever holds the worker roster says; the library cannot see it.
            worker_connected: false,
        }
    }

    /// The bindings a new analysis plans against.
    ///
    /// Per job: the person's choice, when its weights are installed; else what
    /// the device profile bound (or the bundled model, for a job with no
    /// benchmark), when installed; else any installed candidate, the portable
    /// one first; else the default anyway, which readiness then reports as not
    /// installed rather than planning something the person never downloaded.
    pub(crate) fn effective_bindings(&self, raw: &Bindings) -> Bindings {
        let choices = self.lock().choices.clone();
        let mut effective = raw.clone();
        for job in JOBS {
            let Some((implementation, reason)) = self.decide(job, raw, &choices) else {
                continue;
            };
            for stage in job.stages {
                let Some(serving) =
                    implementations::for_stage_and_model(stage, implementation.model)
                else {
                    continue;
                };
                effective.set(match &reason {
                    Reason::Raw(binding)
                        if binding.stage == *stage && binding.implementation == serving.name =>
                    {
                        binding.clone()
                    }
                    Reason::Raw(binding) => Binding::decided(serving, &binding.selected_by),
                    Reason::Decided(label) => Binding::decided(serving, label),
                });
            }
        }
        effective
    }

    fn decide(
        &self,
        job: &Job,
        raw: &Bindings,
        choices: &BTreeMap<String, String>,
    ) -> Option<(&'static Implementation, Reason)> {
        let primary = job.stages[0];
        let registered = implementations::candidates_for_stage(primary).collect::<Vec<_>>();
        // What this device could run: pinned here, on a runtime it can load.
        // The default below comes from the full list, so a job whose model is
        // not even registered is still reported — as missing — not dropped.
        let candidates = registered
            .iter()
            .copied()
            .filter(|implementation| {
                self.registry
                    .get(implementation.model)
                    .is_some_and(|manifest| supported(&manifest).is_ok())
            })
            .collect::<Vec<_>>();
        let installed = |model: &str| {
            self.registry
                .get(model)
                .is_some_and(|manifest| self.install_state(&manifest).0 == Install::Installed)
        };
        if let Some(chosen) = choices.get(job.capability)
            && let Some(implementation) = candidates
                .iter()
                .copied()
                .find(|implementation| implementation.model == chosen.as_str())
            && installed(chosen)
        {
            return Some((implementation, Reason::Decided("chosen")));
        }
        let default = raw
            .for_stage(primary)
            .and_then(|binding| {
                implementations::lookup(&binding.implementation)
                    .map(|implementation| (implementation, Reason::Raw(binding.clone())))
            })
            .or_else(|| {
                // One this platform runs, where there is one: on Windows the
                // default editorial model is the GGUF build, not the Mac's.
                candidates
                    .iter()
                    .chain(registered.iter())
                    .copied()
                    .find(|implementation| !implementation.opt_in)
                    .map(|implementation| (implementation, Reason::Decided("default")))
            });
        if let Some((implementation, _)) = &default
            && installed(implementation.model)
        {
            return default;
        }
        let mut fallbacks = candidates
            .iter()
            .copied()
            .filter(|implementation| installed(implementation.model))
            .collect::<Vec<_>>();
        fallbacks.sort_by_key(|implementation| (!implementation.portable, implementation.opt_in));
        if let Some(implementation) = fallbacks.first() {
            return Some((implementation, Reason::Decided("installed_fallback")));
        }
        default
    }

    /// Queue downloads. A model already queued or running is left alone; one
    /// that failed or was cancelled starts again from what it kept.
    pub(crate) fn download(&self, names: &[String]) -> Result<(), Refusal> {
        if self.hub.is_none() {
            return Err(Refusal::Unavailable(
                "Downloads are unavailable: ClipMill could not start its secure connection."
                    .to_owned(),
            ));
        }
        let mut manifests = Vec::with_capacity(names.len());
        for name in names {
            let manifest = self.registry.get(name).ok_or_else(|| {
                Refusal::NotFound(format!("No model named {name} is registered."))
            })?;
            downloadable(&manifest)?;
            manifests.push(manifest);
        }
        {
            let mut state = self.lock();
            for manifest in manifests {
                if state
                    .progress
                    .get(&manifest.name)
                    .is_some_and(|progress| progress.phase.busy())
                {
                    continue;
                }
                state.progress.insert(
                    manifest.name.clone(),
                    Progress::new(Phase::Queued, manifest.download_bytes()),
                );
                state.queue.push_back(Work::Download(manifest.name.clone()));
            }
        }
        self.wake.notify_one();
        Ok(())
    }

    /// Stop a download or a verification. Queued work is dropped; running
    /// work is told to stop and keeps what it received.
    pub(crate) fn cancel(&self, name: &str) {
        let mut state = self.lock();
        let before = state.queue.len();
        state.queue.retain(|work| work.model() != name);
        let dequeued = state.queue.len() != before;
        let running = state
            .active
            .as_ref()
            .filter(|(active, _)| active == name)
            .map(|(_, cancel)| cancel.clone());
        if let Some(cancel) = running {
            let _ = cancel.send(true);
        } else if dequeued && let Some(progress) = state.progress.get_mut(name) {
            progress.phase = Phase::Cancelled;
            progress.updated = unix_millis();
        }
    }

    /// Queue a verification: every installed file hashed against its pin.
    pub(crate) fn verify(&self, name: &str) -> Result<(), Refusal> {
        let manifest = self
            .registry
            .get(name)
            .ok_or_else(|| Refusal::NotFound(format!("No model named {name} is registered.")))?;
        {
            let mut state = self.lock();
            if state
                .progress
                .get(name)
                .is_some_and(|progress| progress.phase.busy())
            {
                return Err(Refusal::Conflict(
                    "This model is already downloading or being checked.".to_owned(),
                ));
            }
            state.progress.insert(
                name.to_owned(),
                Progress::new(Phase::Queued, self.install_state(&manifest).1),
            );
            state.queue.push_back(Work::Verify(name.to_owned()));
        }
        self.wake.notify_one();
        Ok(())
    }

    /// Delete a model's installed weights and whatever a download kept.
    /// Returns the bytes freed.
    pub(crate) async fn remove(&self, name: &str) -> Result<u64, Refusal> {
        let manifest = self
            .registry
            .get(name)
            .ok_or_else(|| Refusal::NotFound(format!("No model named {name} is registered.")))?;
        self.refuse_if_busy(&manifest).await?;
        let root = self.paths.weights.join(&manifest.name);
        let staging = self.staging(&manifest.name);
        let freed = tokio::task::spawn_blocking(move || {
            let freed = tree_bytes(&root) + tree_bytes(&staging);
            remove_tree(&root)?;
            remove_tree(&staging)?;
            Ok::<u64, std::io::Error>(freed)
        })
        .await
        .map_err(|error| Refusal::Unavailable(error.to_string()))?
        .map_err(|error| {
            Refusal::Unavailable(format!("The model's files could not be removed: {error}"))
        })?;
        let mut state = self.lock();
        state.progress.remove(name);
        if state.choices.values().any(|chosen| chosen == name) {
            state.choices.retain(|_, chosen| chosen != name);
            self.save_choices(&state.choices);
        }
        drop(state);
        tracing::info!(model = name, freed, "model weights removed");
        Ok(freed)
    }

    /// Remove a model the person pinned: weights, manifest and all.
    pub(crate) async fn forget(&self, name: &str) -> Result<u64, Refusal> {
        let manifest = self
            .registry
            .get(name)
            .ok_or_else(|| Refusal::NotFound(format!("No model named {name} is registered.")))?;
        if manifest.origin != ModelOrigin::Custom {
            return Err(Refusal::Invalid(
                "Models that ship with ClipMill can be removed but not forgotten.".to_owned(),
            ));
        }
        let freed = self.remove(name).await?;
        let path = self.paths.custom.join(format!("{name}.json"));
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(Refusal::Unavailable(format!(
                    "The model's manifest could not be removed: {error}"
                )));
            }
        }
        self.registry.unregister(name);
        Ok(freed)
    }

    /// Say which model does a job. Empty returns it to automatic selection.
    pub(crate) fn set_choice(&self, capability: &str, model: &str) -> Result<(), Refusal> {
        let job = job(capability)
            .ok_or_else(|| Refusal::Invalid(format!("{capability} is not a job ClipMill runs.")))?;
        if !model.is_empty() {
            let manifest = self.registry.get(model).ok_or_else(|| {
                Refusal::NotFound(format!("No model named {model} is registered."))
            })?;
            if implementations::for_stage_and_model(job.stages[0], model).is_none() {
                return Err(Refusal::Invalid(format!(
                    "{} cannot do {}.",
                    manifest.title(),
                    job.title.to_lowercase()
                )));
            }
            if let Err(reason) = supported(&manifest) {
                return Err(Refusal::Invalid(format!("{} {reason}", manifest.title())));
            }
            if self.install_state(&manifest).0 != Install::Installed {
                return Err(Refusal::Conflict(format!(
                    "Download {} before choosing it.",
                    manifest.title()
                )));
            }
        }
        let mut state = self.lock();
        if model.is_empty() {
            state.choices.remove(capability);
        } else {
            state
                .choices
                .insert(capability.to_owned(), model.to_owned());
        }
        self.save_choices(&state.choices);
        Ok(())
    }

    /// Read a repository and say what pinning it would pin. A network
    /// operation, so the Local Lock counts it.
    pub(crate) async fn inspect(
        &self,
        repo: &str,
        revision: &str,
        capability: &str,
    ) -> InspectHubModelResponse {
        let Some(hub) = &self.hub else {
            return custom::refused(
                repo,
                capability,
                "Downloads are unavailable on this system.",
            );
        };
        // Refusals that need no network are made before the Lock is told
        // anything happened: nothing was sent for them.
        if implementations::custom_runtime(capability).is_none() {
            return custom::refused(
                repo,
                capability,
                "Only transcription and editorial models can be added from Hugging Face.",
            );
        }
        if let Err(error) = clipmill_hub::check_repository(repo.trim()) {
            return custom::refused(repo, capability, &error.to_string());
        }
        self.policy.note_task_start("model-inspect");
        custom::inspect(hub, &self.registry, repo, revision, capability).await
    }

    /// Pin a person's model, register it, and queue its download.
    pub(crate) async fn add_custom(&self, request: AddRequest<'_>) -> Result<(), Refusal> {
        let hub = self.hub.as_ref().ok_or_else(|| {
            Refusal::Unavailable("Downloads are unavailable on this system.".to_owned())
        })?;
        if self.registry.get(request.name).is_some() {
            return Err(Refusal::Conflict(format!(
                "A model named {} is already registered. Choose another name.",
                request.name
            )));
        }
        self.policy.note_task_start("model-inspect");
        let pinned = custom::pin(hub, &request).await.map_err(Refusal::Invalid)?;
        implementations::register_custom(&pinned.manifest.name, &pinned.manifest.capability)
            .map_err(|detail| Refusal::Invalid(detail.to_owned()))?;
        let bytes = serde_json::to_vec_pretty(&pinned.manifest)
            .map_err(|error| Refusal::Unavailable(error.to_string()))?;
        fs::create_dir_all(&self.paths.custom)
            .and_then(|()| {
                write_private(
                    &self
                        .paths
                        .custom
                        .join(format!("{}.json", pinned.manifest.name)),
                    &bytes,
                )
            })
            .map_err(|error| {
                Refusal::Unavailable(format!("The model could not be saved: {error}"))
            })?;
        let manifest = self
            .registry
            .register(pinned.manifest)
            .map_err(|error| Refusal::Invalid(error.to_string()))?;
        // The small files were fetched and hashed to pin them; installing
        // them now saves fetching the same bytes twice.
        let root = self.paths.weights.join(&manifest.name);
        for (path, contents) in pinned.fetched {
            let target = root.join(&path);
            if let Err(error) = target
                .parent()
                .map_or(Ok(()), fs::create_dir_all)
                .and_then(|()| write_private(&target, &contents))
            {
                tracing::warn!(model = manifest.name, path, %error, "a pinned file was not kept");
            }
        }
        self.download(std::slice::from_ref(&manifest.name))
    }

    /// Whether every pinned file is present at its pinned size.
    pub(crate) fn install_state(&self, manifest: &ModelManifest) -> (Install, u64) {
        let root = self.paths.weights.join(&manifest.name);
        let mut present = 0_u64;
        let mut complete = true;
        for file in &manifest.files {
            match fs::metadata(root.join(&file.path)) {
                Ok(metadata) if metadata.is_file() && metadata.len() == file.bytes => {
                    present = present.saturating_add(file.bytes);
                }
                _ => complete = false,
            }
        }
        if complete {
            (Install::Installed, present)
        } else if present > 0 || self.staging(&manifest.name).exists() {
            (Install::Partial, present)
        } else {
            (Install::Missing, 0)
        }
    }

    /// Models with a download or check queued or running: their staged bytes
    /// are in use, and a clean-up must leave them alone.
    pub(crate) fn busy_models(&self) -> Vec<String> {
        self.lock()
            .progress
            .iter()
            .filter(|(_, progress)| progress.phase.busy())
            .map(|(name, _)| name.clone())
            .collect()
    }

    async fn refuse_if_busy(&self, manifest: &ModelManifest) -> Result<(), Refusal> {
        if self
            .lock()
            .progress
            .get(&manifest.name)
            .is_some_and(|progress| progress.phase.busy())
        {
            return Err(Refusal::Conflict(
                "Cancel the download or check first, then remove the model.".to_owned(),
            ));
        }
        if let Some(database) = &self.database {
            let unfinished = database
                .unfinished_implementations()
                .await
                .map_err(|error| {
                    Refusal::Unavailable(format!(
                        "ClipMill could not check running analyses: {error}"
                    ))
                })?;
            let used = unfinished.iter().any(|implementation| {
                implementations::lookup(implementation)
                    .is_some_and(|found| found.model == manifest.name)
            });
            if used {
                return Err(Refusal::Conflict(format!(
                    "An analysis that has not finished uses {}. Let it finish or cancel it, then remove the model.",
                    manifest.title()
                )));
            }
        }
        Ok(())
    }

    async fn run(self: Arc<Self>) {
        loop {
            let notified = self.wake.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.stopping.load(Ordering::SeqCst) {
                return;
            }
            if let Some((work, mut cancel)) = self.next_work() {
                let name = work.model().to_owned();
                let outcome = match &work {
                    Work::Download(_) => self.execute_download(&name, &mut cancel).await,
                    Work::Verify(_) => self.execute_verify(&name, &mut cancel).await,
                };
                self.finish(&name, outcome);
                continue;
            }
            notified.await;
        }
    }

    fn next_work(&self) -> Option<(Work, watch::Receiver<bool>)> {
        let mut state = self.lock();
        let work = state.queue.pop_front()?;
        let (sender, receiver) = watch::channel(false);
        state.active = Some((work.model().to_owned(), sender));
        if let Some(progress) = state.progress.get_mut(work.model()) {
            progress.phase = match work {
                Work::Download(_) => Phase::Downloading,
                Work::Verify(_) => Phase::Verifying,
            };
            progress.updated = unix_millis();
        }
        Some((work, receiver))
    }

    fn finish(&self, name: &str, outcome: Outcome) {
        let mut state = self.lock();
        state.active = None;
        match outcome {
            Outcome::Done => {
                state.progress.remove(name);
            }
            Outcome::Cancelled => {
                if let Some(progress) = state.progress.get_mut(name) {
                    progress.phase = Phase::Cancelled;
                    progress.updated = unix_millis();
                }
            }
            Outcome::Failed(message) => {
                tracing::warn!(model = name, error = message, "model library work failed");
                if let Some(progress) = state.progress.get_mut(name) {
                    progress.phase = Phase::Failed;
                    progress.error = message;
                    progress.updated = unix_millis();
                }
            }
        }
    }

    fn report(&self, name: &str, phase: Phase, received: u64, current: &str) {
        let mut state = self.lock();
        if let Some(progress) = state.progress.get_mut(name) {
            progress.phase = phase;
            progress.received = received;
            if progress.current != current {
                current.clone_into(&mut progress.current);
            }
            progress.updated = unix_millis();
        }
    }

    async fn execute_download(&self, name: &str, cancel: &mut watch::Receiver<bool>) -> Outcome {
        let Some(manifest) = self.registry.get(name) else {
            return Outcome::Failed("The model is no longer registered.".to_owned());
        };
        if let Err(refusal) = downloadable(&manifest) {
            return Outcome::Failed(refusal.message().to_owned());
        }
        let Some(hub) = &self.hub else {
            return Outcome::Failed("Downloads are unavailable on this system.".to_owned());
        };
        // A network operation the person started: the Lock says so.
        self.policy.note_task_start("model-download");
        let root = self.paths.weights.join(&manifest.name);
        let staging = self.staging(&manifest.name);
        let mut done = 0_u64;
        for file in &manifest.files {
            if *cancel.borrow() {
                return Outcome::Cancelled;
            }
            let target = root.join(&file.path);
            // Already here at its pinned size: prove the digest rather than
            // trust the size, and keep it when it matches.
            if fs::metadata(&target).is_ok_and(|meta| meta.is_file() && meta.len() == file.bytes) {
                self.report(name, Phase::Verifying, done, &file.path);
                match hash_file(&target).await {
                    Ok(digest) if digest == file.sha256 => {
                        done = done.saturating_add(file.bytes);
                        self.report(name, Phase::Downloading, done, &file.path);
                        continue;
                    }
                    Ok(_) => {
                        let _ = fs::remove_file(&target);
                    }
                    Err(error) => return Outcome::Failed(format!("{}: {error}", file.path)),
                }
            }
            let partial = staging.join(format!("{}.part", file.path));
            if let Some(parent) = partial.parent()
                && let Err(error) = fs::create_dir_all(parent)
            {
                return Outcome::Failed(format!("The download could not be staged: {error}"));
            }
            self.report(name, Phase::Downloading, done, &file.path);
            let mut last = 0_u64;
            let base = done;
            let mut progress = |received: u64| {
                if received >= last.saturating_add(PROGRESS_STEP) || received == file.bytes {
                    last = received;
                    self.report(
                        name,
                        Phase::Downloading,
                        base.saturating_add(received),
                        &file.path,
                    );
                }
            };
            let fetched = hub
                .fetch(
                    PinnedFile {
                        repo: &manifest.source.repo,
                        revision: &manifest.source.revision,
                        path: &file.path,
                        sha256: &file.sha256,
                        bytes: file.bytes,
                    },
                    &partial,
                    &mut progress,
                    cancel,
                )
                .await;
            match fetched {
                Ok(()) => {}
                Err(clipmill_hub::Error::Cancelled) => return Outcome::Cancelled,
                Err(error) => return Outcome::Failed(error.to_string()),
            }
            if let Err(error) = install(&partial, &target) {
                return Outcome::Failed(format!("{} could not be installed: {error}", file.path));
            }
            done = done.saturating_add(file.bytes);
            self.report(name, Phase::Downloading, done, &file.path);
        }
        let _ = remove_tree(&staging);
        tracing::info!(model = name, bytes = done, "model installed and verified");
        Outcome::Done
    }

    async fn execute_verify(&self, name: &str, cancel: &mut watch::Receiver<bool>) -> Outcome {
        let Some(manifest) = self.registry.get(name) else {
            return Outcome::Failed("The model is no longer registered.".to_owned());
        };
        let root = self.paths.weights.join(&manifest.name);
        let mut checked = 0_u64;
        let mut damaged = Vec::new();
        let mut missing = 0_usize;
        for file in &manifest.files {
            if *cancel.borrow() {
                return Outcome::Cancelled;
            }
            let target = root.join(&file.path);
            self.report(name, Phase::Verifying, checked, &file.path);
            if !fs::metadata(&target).is_ok_and(|meta| meta.is_file()) {
                missing += 1;
                continue;
            }
            match hash_file(&target).await {
                Ok(digest)
                    if digest == file.sha256
                        && fs::metadata(&target).is_ok_and(|meta| meta.len() == file.bytes) =>
                {
                    checked = checked.saturating_add(file.bytes);
                }
                Ok(_) => {
                    // Removed, so the next download fetches exactly this file
                    // and a worker is never handed bytes that fail its check.
                    let _ = fs::remove_file(&target);
                    damaged.push(file.path.clone());
                }
                Err(error) => return Outcome::Failed(format!("{}: {error}", file.path)),
            }
        }
        if damaged.is_empty() && missing == 0 {
            Outcome::Done
        } else if damaged.is_empty() {
            Outcome::Failed(format!(
                "{missing} pinned file(s) are not installed. Download the model to complete it."
            ))
        } else {
            Outcome::Failed(format!(
                "{} file(s) no longer matched their pins and were removed: {}. Download the model to repair it.",
                damaged.len(),
                damaged.join(", ")
            ))
        }
    }

    fn staging(&self, name: &str) -> PathBuf {
        self.paths.weights.join(STAGING).join(name)
    }

    fn save_choices(&self, choices: &BTreeMap<String, String>) {
        let document = ChoicesDocument {
            schema_version: CHOICES_SCHEMA.to_owned(),
            choices: choices.clone(),
        };
        let written = serde_json::to_vec_pretty(&document)
            .map_err(std::io::Error::other)
            .and_then(|bytes| write_private(&self.paths.choices, &bytes));
        if let Err(error) = written {
            tracing::warn!(%error, "model choices were not saved");
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[derive(Clone, Debug)]
enum Reason {
    /// What the signed profile bound, kept as it was.
    Raw(Binding),
    /// What the library decided: chosen, default or `installed_fallback`.
    Decided(&'static str),
}

#[derive(Debug)]
enum Outcome {
    Done,
    Cancelled,
    Failed(String),
}

/// The word a screen shows for why a model does a job.
fn display_reason(selected_by: &str) -> &'static str {
    match selected_by {
        "chosen" => "chosen",
        "measured" => "measured",
        "unmeasured_fallback" => "portable",
        "installed_fallback" => "installed_fallback",
        _ => "default",
    }
}

/// Whether this platform can load a model's runtime at all.
pub(crate) fn supported(manifest: &ModelManifest) -> Result<(), &'static str> {
    if manifest.runtime == "mlx" && !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return Err("Runs only on Macs with Apple silicon.");
    }
    // The pinned llama.cpp server ships in the Windows and Linux builds only.
    if manifest.runtime == "llama.cpp"
        && !cfg!(all(
            any(target_os = "windows", target_os = "linux"),
            target_arch = "x86_64"
        ))
    {
        return Err("Runs on Windows and Linux PCs; a Mac runs the MLX build.");
    }
    Ok(())
}

/// How the memory a model needs compares with this device. A warning scale,
/// never a gate: the catalog is not built for one machine.
pub(crate) fn memory_fit(required: u64, budget: u64, total: u64) -> &'static str {
    if budget > 0 && required <= budget {
        "fits"
    } else if total > 0 && required <= total {
        "tight"
    } else if total > 0 || budget > 0 {
        "too_large"
    } else {
        "unknown"
    }
}

/// Whether the library may fetch this model at all.
fn downloadable(manifest: &ModelManifest) -> Result<(), Refusal> {
    if manifest.source.provider.trim_end_matches('/') != HUGGING_FACE {
        return Err(Refusal::Invalid(format!(
            "{}'s weights come from {}, which ClipMill does not download from. Place them in the models folder by hand.",
            manifest.title(),
            manifest.source.provider
        )));
    }
    // The bundled registry is held to this by `check-models.py`; a manifest
    // pinned in the app by the pinning step. Checked again here because a
    // file on disk can change after either.
    if manifest.license.class != "permissive" {
        return Err(Refusal::Invalid(format!(
            "{} is not under a licence that permits publishing what it helps make.",
            manifest.title()
        )));
    }
    Ok(())
}

/// Give a verified download its final name. The staging directory sits on
/// the same volume, so this is a rename, and the parent is synced after it.
fn install(partial: &Path, target: &Path) -> std::io::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(partial, target)?;
    if let Some(parent) = target.parent() {
        crate::platform::sync_dir(parent)?;
    }
    Ok(())
}

async fn hash_file(path: &Path) -> std::io::Result<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut file = fs::File::open(&path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; HASH_BUFFER];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(clipmill_hub::hex(&hasher.finalize()))
    })
    .await
    .map_err(std::io::Error::other)?
}

/// Write a private file so a crash leaves the old bytes or the new, never half.
pub(crate) fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt as _;

    let pending = path.with_extension("pending");
    let mut options = fs::OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let written = options.open(&pending).and_then(|mut file| {
        file.write_all(bytes)?;
        file.sync_all()
    });
    match written.and_then(|()| fs::rename(&pending, path)) {
        Ok(()) => {
            if let Some(parent) = path.parent()
                && let Ok(directory) = fs::File::open(parent)
            {
                let _ = directory.sync_all();
            }
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&pending);
            Err(error)
        }
    }
}

fn read_choices(path: &Path) -> BTreeMap<String, String> {
    let Ok(bytes) = fs::read(path) else {
        return BTreeMap::new();
    };
    match serde_json::from_slice::<ChoicesDocument>(&bytes) {
        Ok(document) if document.schema_version == CHOICES_SCHEMA => document
            .choices
            .into_iter()
            .filter(|(capability, _)| job(capability).is_some())
            .collect(),
        _ => {
            tracing::warn!("model choices are unreadable; every job is automatic");
            BTreeMap::new()
        }
    }
}

/// Remove a directory tree, or the link standing where one was expected.
/// A symbolic link is removed as a link: its target is somebody else's.
fn remove_tree(path: &Path) -> std::io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => {
            fs::remove_file(path)
        }
        Ok(_) => fs::remove_dir_all(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Bytes under a directory, links counted as themselves and never followed.
pub(crate) fn tree_bytes(root: &Path) -> u64 {
    let mut total = 0_u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                pending.push(entry.path());
            } else if let Ok(metadata) = entry.metadata() {
                total = total.saturating_add(metadata.len());
            }
        }
    }
    total
}

/// Free space where weights would land, or on the nearest folder above it
/// that exists: a fresh install has not created the models folder yet.
fn available_space(path: &Path) -> Option<u64> {
    let mut candidate = path;
    loop {
        if candidate.exists() {
            return fs2::available_space(candidate).ok();
        }
        candidate = candidate.parent()?;
    }
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
