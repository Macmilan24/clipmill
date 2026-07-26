//! The stage registry: every task kind the daemon will run, declared with the
//! artifact it produces.
//!
//! Phase 0 keyed every worker artifact through one hardcoded recipe, which
//! meant a new stage could be planned, leased, and published without anyone
//! deciding what its cache key should be — and two stages could collide on one
//! address by accident. A kind now exists only if it is registered here, and
//! registration states the artifact kind, the semantic version that
//! invalidates cached outputs when the stage's meaning changes, and whether a
//! model's identity belongs in the key.
//!
//! Plan validation consults this table, so an unregistered kind is refused at
//! submit rather than discovered by a worker that cannot serve it.

use clipmill_artifacts::{
    ArtifactRecipe, NetworkPolicy, Producer, RecipeError, RecipeSpec, Timebase,
};
use clipmill_core::Sha256Digest;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{jobs::LeasedTask, models::ModelRegistry};

/// Who runs a stage. The distinction is not cosmetic: a builtin stage builds
/// its own recipe from facts only it has (a probe's stream layout, a render's
/// filter graph), while a leased stage is keyed from its payload here, because
/// the daemon must know the key before it hands work to anyone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Executor {
    /// The daemon executes it in process.
    Builtin,
    /// A registered worker leases it.
    Worker,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Recipe {
    /// The task kind a plan factory emits.
    pub kind: &'static str,
    /// The artifact kind it publishes.
    pub output_kind: &'static str,
    /// Bumped when the stage's meaning changes. Two runs of different
    /// semantic versions must not share a cache entry even if every other
    /// input matches.
    pub semantic_version: &'static str,
    /// Who runs it. Read by the scheduler's capability list and by the tests
    /// that keep the builtin and leased sets from drifting apart.
    #[allow(
        dead_code,
        reason = "declared per stage; consumed by the admission path"
    )]
    pub executor: Executor,
    /// Registry name of the model this stage runs, when it runs one. The
    /// model's digest joins the artifact key, so re-pinning a model
    /// invalidates exactly the stages that used it — not the whole cache, and
    /// not nothing.
    pub model: Option<&'static str>,
}

/// Every kind the daemon will plan or lease.
///
/// Builtin media stages appear here even though they build their own recipes,
/// because this is also the list plan validation checks against: a kind that
/// is not here cannot be submitted.
const REGISTRY: &[Recipe] = &[
    // Daemon-owned source and device stages.
    Recipe {
        kind: "probe-source",
        output_kind: "evidence.source_map.v1",
        semantic_version: "clipmill.source_map.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "device-profile",
        output_kind: "evidence.device_profile.v1",
        semantic_version: "clipmill.device_profile.v1",
        executor: Executor::Builtin,
        model: None,
    },
    // The W11 ingest fan-out (book ch. 12).
    Recipe {
        kind: "ingest-proxy",
        output_kind: "media.proxy.v1",
        semantic_version: "clipmill.media.proxy.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "ingest-audio-16k",
        output_kind: "media.audio_16k.v1",
        semantic_version: "clipmill.media.audio.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "ingest-audio-48k",
        output_kind: "media.audio_48k.v1",
        semantic_version: "clipmill.media.audio.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "ingest-loudness",
        output_kind: "media.loudness_envelope.v1",
        semantic_version: "clipmill.media.loudness_envelope.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "ingest-reference-index",
        output_kind: "media.reference_index.v1",
        semantic_version: "clipmill.media.reference_index.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "ingest-filmstrip",
        output_kind: "media.filmstrip.v1",
        semantic_version: "clipmill.media.filmstrip.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "ingest-audio-peaks",
        output_kind: "media.audio_peaks.v1",
        semantic_version: "clipmill.media.audio_peaks.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "ingest-frames",
        output_kind: "media.frames.v1",
        semantic_version: "clipmill.media.frames.v1",
        executor: Executor::Builtin,
        model: None,
    },
    Recipe {
        kind: "ingest-manifest",
        output_kind: "media.ingest_manifest.v1",
        semantic_version: "clipmill.media.ingest_manifest.v1",
        executor: Executor::Builtin,
        model: None,
    },
    // The W13 render compiler (book ch. 17).
    Recipe {
        kind: "render-clip",
        output_kind: "render.clip.v1",
        semantic_version: "clipmill.render.clip.v1",
        executor: Executor::Builtin,
        model: None,
    },
    // The reference worker family. It carries no model, which is precisely
    // why it is the right shape to prove the leased path with.
    Recipe {
        kind: "demo-seed",
        output_kind: "evidence.demo.seed.v1",
        semantic_version: "clipmill.demo.v1",
        executor: Executor::Worker,
        model: None,
    },
    Recipe {
        kind: "demo-left",
        output_kind: "evidence.demo.left.v1",
        semantic_version: "clipmill.demo.v1",
        executor: Executor::Worker,
        model: None,
    },
    Recipe {
        kind: "demo-right",
        output_kind: "evidence.demo.right.v1",
        semantic_version: "clipmill.demo.v1",
        executor: Executor::Worker,
        model: None,
    },
    Recipe {
        kind: "demo-join",
        output_kind: "evidence.demo.final.v1",
        semantic_version: "clipmill.demo.v1",
        executor: Executor::Worker,
        model: None,
    },
];

pub(crate) fn lookup(kind: &str) -> Option<&'static Recipe> {
    REGISTRY.iter().find(|recipe| recipe.kind == kind)
}

/// Whether a plan may declare this kind producing this artifact.
///
/// Both halves matter. An unknown kind is a stage nobody registered; a known
/// kind with the wrong output kind is a plan that would publish under an
/// address the registry says belongs to something else.
pub(crate) fn plan_declaration_is_registered(kind: &str, output_kind: &str) -> bool {
    lookup(kind).is_some_and(|recipe| recipe.output_kind == output_kind)
}

/// The artifact key for a leased task.
///
/// The daemon computes this before handing work out, so a cache hit can be
/// served without a worker ever running — and so two workers cannot disagree
/// about what a result should be called.
pub(crate) fn worker_recipe(
    task: &LeasedTask,
    models: &ModelRegistry,
) -> Result<ArtifactRecipe, RegistryError> {
    let recipe =
        lookup(&task.kind).ok_or_else(|| RegistryError::UnknownStage(task.kind.clone()))?;
    // A stage that runs a model cannot be keyed without it. Falling back to a
    // key with no model identity would let two different models' outputs
    // share one content address, which is the one failure a content-addressed
    // store cannot detect afterwards.
    let model_digest = match recipe.model {
        None => None,
        Some(name) => Some(
            models
                .get(name)
                .ok_or(RegistryError::MissingModel {
                    stage: recipe.kind,
                    model: name,
                })?
                .digest(),
        ),
    };
    let mut source_hasher = Sha256::new();
    source_hasher.update(b"clipmill.worker.payload.v1\0");
    source_hasher.update(task.kind.as_bytes());
    source_hasher.update(b"\0");
    source_hasher.update(&task.payload);
    let source_fingerprint = Sha256Digest::from_bytes(source_hasher.finalize().into());
    let mut config = Map::new();
    config.insert("task_kind".to_owned(), Value::String(task.kind.clone()));
    ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: recipe.output_kind.to_owned(),
        source_fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest,
        },
        inputs: task.input_artifact_ids.clone(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: recipe.semantic_version.to_owned(),
    })
    .map_err(RegistryError::Recipe)
}

/// The registry is the authority on which stages exist, so "no such stage" is
/// its own answer rather than a malformed recipe.
#[derive(Debug, Error)]
pub(crate) enum RegistryError {
    #[error("no registered stage named {0}")]
    UnknownStage(String),
    #[error("stage {stage} runs model {model}, which the registry does not pin")]
    MissingModel {
        stage: &'static str,
        model: &'static str,
    },
    #[error(transparent)]
    Recipe(#[from] RecipeError),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{Executor, REGISTRY, lookup, plan_declaration_is_registered};

    #[test]
    fn every_kind_is_registered_exactly_once() {
        let mut kinds = REGISTRY
            .iter()
            .map(|recipe| recipe.kind)
            .collect::<Vec<_>>();
        let total = kinds.len();
        kinds.sort_unstable();
        kinds.dedup();
        assert_eq!(kinds.len(), total, "a task kind is registered twice");
    }

    /// Two stages sharing an artifact kind would collide on one content
    /// address whenever their other inputs happened to match.
    #[test]
    fn no_two_stages_publish_the_same_artifact_kind() {
        let mut outputs = REGISTRY
            .iter()
            .map(|recipe| recipe.output_kind)
            .collect::<Vec<_>>();
        let total = outputs.len();
        outputs.sort_unstable();
        outputs.dedup();
        assert_eq!(outputs.len(), total, "two stages publish the same kind");
    }

    #[test]
    fn a_plan_may_only_declare_registered_pairings() {
        assert!(plan_declaration_is_registered(
            "render-clip",
            "render.clip.v1"
        ));
        assert!(!plan_declaration_is_registered(
            "render-clip",
            "media.proxy.v1"
        ));
        assert!(!plan_declaration_is_registered(
            "not-a-stage",
            "anything.v1"
        ));
    }

    #[test]
    fn the_builtin_stages_are_the_ones_the_daemon_runs_itself() {
        for kind in [
            "probe-source",
            "device-profile",
            "ingest-proxy",
            "ingest-manifest",
            "render-clip",
        ] {
            let recipe = lookup(kind).expect("registered");
            assert_eq!(recipe.executor, Executor::Builtin, "{kind}");
        }
        for kind in ["demo-seed", "demo-left", "demo-right", "demo-join"] {
            let recipe = lookup(kind).expect("registered");
            assert_eq!(recipe.executor, Executor::Worker, "{kind}");
        }
    }

    /// A stage that runs a model must name it, so the model's digest can join
    /// the artifact key. Nothing in Phase 1's current set runs one; the speech
    /// stack is where this earns its keep.
    #[test]
    fn a_registered_model_name_is_never_empty() {
        for recipe in REGISTRY {
            assert!(
                recipe.model.is_none_or(|name| !name.is_empty()),
                "{} names an empty model",
                recipe.kind
            );
        }
    }
}
