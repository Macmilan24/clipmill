//! The stage registry: every task kind the daemon will run, declared with the
//! artifact it produces.
//!
//! Phase 0 keyed every worker artifact through one hardcoded recipe, which
//! meant a new stage could be planned, leased, and published without anyone
//! deciding what its cache key should be — and two stages could collide on one
//! address by accident. A kind now exists only if it is registered here, and
//! registration states the artifact kind, the semantic version that
//! invalidates cached outputs when the stage's meaning changes, whether a
//! model's identity belongs in the key, and which pinned binaries the stage
//! may be handed.
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

use crate::{implementations, jobs::LeasedTask, models::ModelRegistry};

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
    /// The capability this stage runs a model for, when it runs one.
    ///
    /// Not a model name. Which model serves a capability is a per-device
    /// decision (D19), taken when a job is planned and recorded on the task as
    /// its implementation; naming one model here would be the static default
    /// that decision replaces. The chosen model's digest still joins the
    /// artifact key, so re-pinning a model invalidates exactly the stages that
    /// used it — not the whole cache, and not nothing.
    pub capability: Option<&'static str>,
    /// Pinned executables from `bom.toml` this stage may be handed on its
    /// lease, by bill-of-materials name.
    ///
    /// A model is not the only versioned input a stage can have. A decoder is
    /// one too — two FFmpeg builds hand a detector different pixels — and a
    /// worker that resolved one from the PATH would publish observations that
    /// two machines could disagree about while sharing a content address. So
    /// the daemon names the binary, and the stage payload carries its build
    /// identity into the key. Empty for every stage that needs nothing but its
    /// inputs, which is most of them.
    pub tools: &'static [&'static str],
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
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "device-profile",
        output_kind: "evidence.device_profile.v1",
        semantic_version: "clipmill.device_profile.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    // The W11 ingest fan-out (book ch. 12).
    Recipe {
        kind: "ingest-proxy",
        output_kind: "media.proxy.v1",
        semantic_version: "clipmill.media.proxy.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "ingest-audio-16k",
        output_kind: "media.audio_16k.v1",
        semantic_version: "clipmill.media.audio.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "ingest-audio-48k",
        output_kind: "media.audio_48k.v1",
        semantic_version: "clipmill.media.audio.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "ingest-loudness",
        output_kind: "media.loudness_envelope.v1",
        semantic_version: "clipmill.media.loudness_envelope.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "ingest-reference-index",
        output_kind: "media.reference_index.v1",
        semantic_version: "clipmill.media.reference_index.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "ingest-filmstrip",
        output_kind: "media.filmstrip.v1",
        semantic_version: "clipmill.media.filmstrip.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "ingest-audio-peaks",
        output_kind: "media.audio_peaks.v1",
        semantic_version: "clipmill.media.audio_peaks.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "ingest-frames",
        output_kind: "media.frames.v1",
        semantic_version: "clipmill.media.frames.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "ingest-manifest",
        output_kind: "media.ingest_manifest.v1",
        semantic_version: "clipmill.media.ingest_manifest.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    // The W13 render compiler (book ch. 17).
    Recipe {
        kind: "render-clip",
        output_kind: "render.clip.v1",
        semantic_version: "clipmill.render.clip.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    // The W15 speech chain (book ch. 13). Three leased stages that each run a
    // model, and one builtin that runs none.
    //
    // The capabilities below are what put a model's digest into an artifact
    // key, by way of whichever implementation the device chose. Re-pinning
    // whisper invalidates every transcript recognized by whisper and nothing
    // else; re-pinning the aligner invalidates every alignment and leaves the
    // text alone. That is the whole reason the chain is three stages.
    Recipe {
        kind: "speech-vad",
        output_kind: "speech.vad.v1",
        semantic_version: "clipmill.speech.vad.v1",
        executor: Executor::Worker,
        capability: Some("vad"),
        tools: &[],
    },
    Recipe {
        kind: "speech-asr",
        output_kind: "speech.asr.v1",
        semantic_version: "clipmill.speech.asr.v1",
        executor: Executor::Worker,
        capability: Some("asr"),
        tools: &[],
    },
    Recipe {
        kind: "speech-align",
        output_kind: "speech.alignment.v1",
        semantic_version: "clipmill.speech.alignment.v1",
        executor: Executor::Worker,
        capability: Some("forced-align"),
        tools: &[],
    },
    Recipe {
        kind: "speech-transcript",
        output_kind: "speech.transcript.v1",
        semantic_version: "clipmill.speech.transcript.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    // The W17 evidence index (book ch. 14). Builtin and modelless, like the
    // speech assembly: it reads two published documents and writes a third.
    Recipe {
        kind: "index-transcript",
        output_kind: "index.transcript.v1",
        semantic_version: "clipmill.index.transcript.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    // The W18 proposer mesh (book ch. 15). Builtin and modelless: it reads
    // three published documents and writes a fourth.
    Recipe {
        kind: "discover-candidates",
        output_kind: "discovery.candidates.v1",
        semantic_version: "clipmill.discovery.candidates.v1",
        executor: Executor::Builtin,
        capability: None,
        tools: &[],
    },
    // The W16 shot detector (book ch. 13). Leased, but modelless: it is
    // arithmetic over decoded pixels, so there is no capability to bind and no
    // model digest to key against. What makes its output reproducible is the
    // decoder, and that identity travels in the payload — which the key
    // already covers.
    Recipe {
        kind: "detect-shots",
        output_kind: "evidence.shots.v1",
        semantic_version: "clipmill.evidence.shots.v1",
        executor: Executor::Worker,
        capability: None,
        tools: &["ffmpeg"],
    },
    // The reference worker family. It carries no model, which is precisely
    // why it is the right shape to prove the leased path with.
    Recipe {
        kind: "demo-seed",
        output_kind: "evidence.demo.seed.v1",
        semantic_version: "clipmill.demo.v1",
        executor: Executor::Worker,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "demo-left",
        output_kind: "evidence.demo.left.v1",
        semantic_version: "clipmill.demo.v1",
        executor: Executor::Worker,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "demo-right",
        output_kind: "evidence.demo.right.v1",
        semantic_version: "clipmill.demo.v1",
        executor: Executor::Worker,
        capability: None,
        tools: &[],
    },
    Recipe {
        kind: "demo-join",
        output_kind: "evidence.demo.final.v1",
        semantic_version: "clipmill.demo.v1",
        executor: Executor::Worker,
        capability: None,
        tools: &[],
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
    let model_digest = match recipe.capability {
        None => None,
        Some(capability) => {
            let name = model_for(&task.kind, capability, &task.implementation)?;
            Some(
                models
                    .get(name)
                    .ok_or(RegistryError::MissingModel {
                        stage: recipe.kind,
                        model: name,
                    })?
                    .digest(),
            )
        }
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

/// The model a planned task will run, resolved from the implementation the
/// plan committed to.
///
/// This is the single road from a task to its model identity. The daemon calls
/// it twice — once to key the artifact, once to hand the worker its weights —
/// and both answers come from the one string the plan wrote down, so the key
/// cannot describe one model while the worker loads another. Neither the
/// device profile nor the model registry is consulted here: re-measuring a
/// device changes what the next plan chooses and never what an existing task
/// means.
pub(crate) fn model_for(
    stage: &str,
    capability: &'static str,
    implementation: &str,
) -> Result<&'static str, RegistryError> {
    let chosen = implementations::lookup(implementation).ok_or_else(|| {
        RegistryError::UnknownImplementation {
            stage: stage.to_owned(),
            implementation: implementation.to_owned(),
        }
    })?;
    if chosen.capability != capability || chosen.stage != stage {
        return Err(RegistryError::ImplementationMismatch {
            stage: stage.to_owned(),
            implementation: implementation.to_owned(),
            capability,
        });
    }
    Ok(chosen.model)
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
    #[error(
        "stage {stage} was planned with implementation {implementation}, which nothing registers"
    )]
    UnknownImplementation {
        stage: String,
        implementation: String,
    },
    #[error("implementation {implementation} does not serve {capability} for stage {stage}")]
    ImplementationMismatch {
        stage: String,
        implementation: String,
        capability: &'static str,
    },
    #[error(transparent)]
    Recipe(#[from] RecipeError),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{
        Executor, REGISTRY, RegistryError, lookup, model_for, plan_declaration_is_registered,
    };

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

    /// Every capability a stage runs must have somewhere to be served from,
    /// or the stage could be planned and then never keyed.
    #[test]
    fn every_stage_capability_has_at_least_one_implementation() {
        for recipe in REGISTRY {
            let Some(capability) = recipe.capability else {
                continue;
            };
            assert!(
                !capability.is_empty(),
                "{} names no capability",
                recipe.kind
            );
            assert!(
                crate::implementations::candidates_for_stage(recipe.kind)
                    .any(|implementation| implementation.capability == capability),
                "{} runs {capability}, which nothing implements",
                recipe.kind
            );
        }
    }

    /// A tool is only reachable by a stage that registered it, and only a
    /// leased stage can be handed one at all — a builtin runs the sidecars
    /// through the daemon's own supervised runner, which is a different and
    /// narrower path than a lease.
    #[test]
    fn only_leased_stages_declare_tools_and_only_known_ones() {
        let known = ["ffmpeg"];
        let mut declaring = Vec::new();
        for recipe in REGISTRY {
            if recipe.tools.is_empty() {
                continue;
            }
            declaring.push(recipe.kind);
            assert_eq!(
                recipe.executor,
                Executor::Worker,
                "{} is builtin and does not need a lease-delivered binary",
                recipe.kind
            );
            for tool in recipe.tools {
                assert!(
                    known.contains(tool),
                    "{} declares {tool}, which the daemon cannot resolve",
                    recipe.kind
                );
            }
        }
        // Stated as a list rather than a count: a stage acquiring the right to
        // spawn a pinned binary should show up in a diff of this test.
        assert_eq!(declaring, ["detect-shots"]);
    }

    /// Shot detection is leased but modelless. It is the case that would break
    /// if `capability` were ever read as "does this stage need a worker".
    #[test]
    fn shot_detection_is_leased_without_a_model_and_keyed_without_one() {
        let recipe = lookup("detect-shots").expect("registered");
        assert_eq!(recipe.executor, Executor::Worker);
        assert!(
            recipe.capability.is_none(),
            "shot detection runs arithmetic over pixels, not a model"
        );
        assert_eq!(recipe.tools, ["ffmpeg"]);
        assert!(plan_declaration_is_registered(
            "detect-shots",
            "evidence.shots.v1"
        ));
    }

    /// The three model-running speech stages are leased; the assembly that
    /// fuses them runs in the daemon, because it loads nothing.
    #[test]
    fn the_speech_chain_leases_exactly_the_stages_that_run_a_model() {
        for kind in ["speech-vad", "speech-asr", "speech-align"] {
            let recipe = lookup(kind).expect("registered");
            assert_eq!(recipe.executor, Executor::Worker, "{kind}");
            assert!(recipe.capability.is_some(), "{kind} runs a model");
        }
        let assembly = lookup("speech-transcript").expect("registered");
        assert_eq!(assembly.executor, Executor::Builtin);
        assert!(assembly.capability.is_none(), "assembly runs no model");
    }

    /// The key and the weights must come from the same decision. Whatever the
    /// device chose, the model behind a task is whatever its implementation
    /// says — never a per-stage default the plan did not agree to.
    #[test]
    fn a_task_resolves_to_the_model_its_implementation_names() {
        assert_eq!(
            model_for("speech-asr", "asr", "clipmill-worker-asr@0.1.0").expect("registered"),
            "whisper-base"
        );
        assert_eq!(
            model_for("speech-asr", "asr", "clipmill-worker-speech-mlx@0.1.0/asr")
                .expect("registered"),
            "qwen3-asr-mlx"
        );
    }

    /// An implementation from another stage cannot be smuggled into a task
    /// row: it would key the artifact against a model that stage never runs.
    #[test]
    fn an_implementation_from_another_stage_is_refused() {
        assert!(matches!(
            model_for("speech-asr", "asr", "clipmill-worker-align@0.1.0"),
            Err(RegistryError::ImplementationMismatch { .. })
        ));
        assert!(matches!(
            model_for("speech-asr", "asr", "clipmill-worker-asr@9.9.9"),
            Err(RegistryError::UnknownImplementation { .. })
        ));
    }
}
