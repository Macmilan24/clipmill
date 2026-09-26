//! Register capability implementations and resolve planned producer identities.
//!
//! Worker family, model, and backend identify an implementation and enter the
//! artifact key. Selection is fixed in the task row when a job is planned;
//! subsequent device measurements affect only future plans.

use std::{collections::BTreeSet, sync::RwLock};

/// One way to serve one capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Implementation {
    /// Recorded on the task and carried into `producer.implementation`. This
    /// is the value that makes two implementations' outputs different
    /// artifacts rather than one artifact with two histories.
    pub name: &'static str,
    /// The capability it serves, as the model registry spells it.
    pub capability: &'static str,
    /// The stage kind whose tasks it runs.
    pub stage: &'static str,
    /// Registry name of the model it loads. Its digest joins the artifact key.
    pub model: &'static str,
    /// The runtime a worker declares to serve this.
    pub backend: &'static str,
    /// The worker family that runs it, as that worker registers itself. A
    /// task planned with this implementation is leased only to a worker of
    /// this family: two families can serve one stage — whisper.cpp and MLX
    /// both transcribe — and each loads only its own models.
    pub worker: &'static str,
    /// The accelerator class the scheduler requires of whoever leases it.
    /// Empty means any machine will do, which is what makes a candidate the
    /// portable one.
    pub accelerator_class: &'static str,
    /// Whether this implementation runs on every supported platform. Exactly
    /// one candidate per capability is portable: it is what an unmeasured
    /// device falls back to, and what the offline exit gate rests on.
    pub portable: bool,
    /// Never chosen by measurement; planned only when a person picks its
    /// model. A benchmark ranks speed, and these candidates are here for
    /// something else — accuracy, or a model the person pinned themselves —
    /// so ranking them by speed would choose against the reason they exist.
    /// They stay out of the signed device profile for the same reason.
    pub opt_in: bool,
    /// How accurate it is among its capability's candidates, from published
    /// error rates rather than anything measured here: 1 is the floor. Among
    /// the candidates that keep up on this machine the most accurate is
    /// chosen; speed decides only between equals, and when none keeps up.
    pub accuracy: u8,
}

/// Every implementation the daemon knows how to plan.
///
/// The order within a capability is not a preference. Selection reads the
/// device profile, and where it has no measurement it takes the portable
/// candidate by that flag rather than by position — a list whose order quietly
/// meant "best first" would be the static default this table exists to
/// replace.
const IMPLEMENTATIONS: &[Implementation] = &[
    Implementation {
        name: "clipmill-worker-vad@0.1.0",
        capability: "vad",
        stage: "speech-vad",
        model: "silero-vad",
        worker: "speech-vad",
        backend: "onnx-cpu",
        accelerator_class: "",
        portable: true,
        opt_in: false,
        accuracy: 1,
    },
    Implementation {
        name: "clipmill-worker-asr@0.1.0",
        capability: "asr",
        stage: "speech-asr",
        model: "whisper-base",
        worker: "speech-asr",
        backend: "cpu",
        accelerator_class: "",
        portable: true,
        opt_in: false,
        accuracy: 1,
    },
    // The accurate end of the whisper.cpp family, on the same worker as the
    // base model. Slower on every machine, so a benchmark that ranks speed
    // would never pick it; it runs when a person chooses accuracy.
    Implementation {
        name: "clipmill-worker-asr@0.1.0/whisper-large-v3-turbo",
        capability: "asr",
        stage: "speech-asr",
        model: "whisper-large-v3-turbo",
        worker: "speech-asr",
        backend: "cpu",
        accelerator_class: "",
        portable: false,
        opt_in: true,
        accuracy: 3,
    },
    Implementation {
        name: "clipmill-worker-speech-mlx@0.1.0/asr",
        capability: "asr",
        stage: "speech-asr",
        model: "qwen3-asr-mlx",
        worker: "speech-mlx",
        backend: "mlx",
        accelerator_class: "metal",
        portable: false,
        opt_in: false,
        accuracy: 3,
    },
    Implementation {
        name: "clipmill-worker-align@0.1.0",
        capability: "forced-align",
        stage: "speech-align",
        model: "wav2vec2-ctc-en",
        worker: "speech-align",
        backend: "onnx-cpu",
        accelerator_class: "",
        portable: true,
        opt_in: false,
        accuracy: 1,
    },
    Implementation {
        name: "clipmill-worker-speech-mlx@0.1.0/align",
        capability: "forced-align",
        stage: "speech-align",
        model: "qwen3-aligner-mlx",
        worker: "speech-mlx",
        backend: "mlx",
        accelerator_class: "metal",
        portable: false,
        opt_in: false,
        accuracy: 1,
    },
    Implementation {
        name: "clipmill-worker-editorial@0.2.0/propose",
        capability: "editorial",
        stage: "editorial-propose",
        model: "qwen3-5-editorial-mlx",
        worker: "editorial",
        backend: "mlx",
        accelerator_class: "metal",
        portable: false,
        opt_in: false,
        accuracy: 1,
    },
    Implementation {
        name: "clipmill-worker-editorial@0.2.0/review",
        capability: "editorial",
        stage: "editorial-review",
        model: "qwen3-5-editorial-mlx",
        worker: "editorial",
        backend: "mlx",
        accelerator_class: "metal",
        portable: false,
        opt_in: false,
        accuracy: 1,
    },
    Implementation {
        name: "clipmill-worker-editorial@0.2.0/look",
        capability: "editorial",
        stage: "editorial-look",
        model: "qwen3-5-editorial-mlx",
        worker: "editorial",
        backend: "mlx",
        accelerator_class: "metal",
        portable: false,
        opt_in: false,
        accuracy: 1,
    },
    Implementation {
        name: "clipmill-worker-editorial@0.2.0/metadata",
        capability: "editorial",
        stage: "youtube-metadata",
        model: "qwen3-5-editorial-mlx",
        worker: "editorial",
        backend: "mlx",
        accelerator_class: "metal",
        portable: false,
        opt_in: false,
        accuracy: 1,
    },
    // The face detector. One candidate and no accelerated sibling: YuNet is a
    // 230 kB CPU graph whose whole appeal is having no runtime tail, and an
    // accelerated variant would be a second implementation to keep honest for
    // no measurable gain.
    Implementation {
        name: crate::jobs::FACES_IMPLEMENTATION,
        capability: "detect-faces",
        stage: "detect-faces",
        model: "yunet-face",
        worker: "detect-faces",
        backend: "onnx-cpu",
        accelerator_class: "",
        portable: true,
        opt_in: false,
        accuracy: 1,
    },
];

/// Implementations for models a person pinned, added while the daemon runs.
///
/// Held as `&'static` like the table above, so every consumer keeps one type
/// and a task row can name a person's model exactly as it names a bundled one.
/// Each entry is allocated once per model name for the life of the process —
/// forgetting a model and pinning it again reuses what is here — so the memory
/// is bounded by how many distinct models were ever pinned in this session.
/// An entry whose model is no longer registered is inert: nothing can be keyed
/// against a model the registry does not pin.
static CUSTOM: RwLock<Vec<&'static Implementation>> = RwLock::new(Vec::new());

/// The worker a person's own model can run on, per capability.
///
/// Only families whose worker loads whatever the lease binds: whisper.cpp
/// reads the one GGML file a manifest pins, and the editorial worker loads an
/// MLX model directory. The other capabilities read fixed graph formats with
/// fixed label sets, where a different file is a different program.
pub(crate) fn custom_runtime(capability: &str) -> Option<CustomRuntime> {
    match capability {
        "asr" => Some(CustomRuntime {
            family: "whisper.cpp",
            runtime: "whisper.cpp",
            backend: "cpu",
            quantization: "ggml",
            worker: "speech-asr",
        }),
        "editorial" => Some(CustomRuntime {
            family: "mlx-vlm",
            runtime: "mlx",
            backend: "mlx",
            quantization: "mlx",
            worker: "editorial",
        }),
        _ => None,
    }
}

/// How a person's model for one capability is loaded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CustomRuntime {
    pub family: &'static str,
    pub runtime: &'static str,
    pub backend: &'static str,
    pub quantization: &'static str,
    /// The worker family that loads it.
    pub worker: &'static str,
}

/// Make a person's model plannable: one implementation per stage its worker
/// runs, each opt-in, so measurement never picks it and only a choice does.
pub(crate) fn register_custom(model: &str, capability: &str) -> Result<(), &'static str> {
    let runtime = custom_runtime(capability)
        .ok_or("ClipMill has no worker that runs your own model for this job")?;
    let mut custom = CUSTOM
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if custom
        .iter()
        .any(|implementation| implementation.model == model)
    {
        return Ok(());
    }
    let model: &'static str = Box::leak(model.to_owned().into_boxed_str());
    let stages: &[(&str, &'static str)] = match capability {
        "asr" => &[("clipmill-worker-asr@0.1.0", "speech-asr")],
        _ => &[
            (
                "clipmill-worker-editorial@0.2.0/propose",
                "editorial-propose",
            ),
            ("clipmill-worker-editorial@0.2.0/review", "editorial-review"),
            ("clipmill-worker-editorial@0.2.0/look", "editorial-look"),
            (
                "clipmill-worker-editorial@0.2.0/metadata",
                "youtube-metadata",
            ),
        ],
    };
    for (prefix, stage) in stages {
        let name: &'static str = Box::leak(format!("{prefix}/custom/{model}").into_boxed_str());
        custom.push(Box::leak(Box::new(Implementation {
            name,
            capability: if capability == "asr" {
                "asr"
            } else {
                "editorial"
            },
            stage,
            model,
            backend: runtime.backend,
            worker: runtime.worker,
            accelerator_class: if runtime.backend == "mlx" {
                "metal"
            } else {
                ""
            },
            portable: false,
            opt_in: true,
            accuracy: 1,
        })));
    }
    Ok(())
}

/// The bundled table, then every implementation added for a pinned model.
fn all() -> Vec<&'static Implementation> {
    let custom = CUSTOM
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    IMPLEMENTATIONS
        .iter()
        .chain(custom.iter().copied())
        .collect()
}

/// Every implementation registered for a stage.
pub(crate) fn candidates_for_stage(stage: &str) -> impl Iterator<Item = &'static Implementation> {
    all()
        .into_iter()
        .filter(move |implementation| implementation.stage == stage)
}

/// Every implementation registered for a capability.
pub(crate) fn candidates_for_capability(
    capability: &str,
) -> impl Iterator<Item = &'static Implementation> {
    all()
        .into_iter()
        .filter(move |implementation| implementation.capability == capability)
}

/// The candidates a benchmark may rank: everything but the opt-in ones.
pub(crate) fn measured_candidates_for_capability(
    capability: &str,
) -> impl Iterator<Item = &'static Implementation> {
    candidates_for_capability(capability).filter(|implementation| !implementation.opt_in)
}

/// The implementation a task was planned with.
///
/// Every consumer of a task's model identity goes through here, so the key the
/// daemon computed and the weights it hands the worker cannot disagree: both
/// are derived from the one string the plan committed to.
pub(crate) fn lookup(name: &str) -> Option<&'static Implementation> {
    all()
        .into_iter()
        .find(|implementation| implementation.name == name)
}

/// The implementation of a stage that loads a given model, if any does.
pub(crate) fn for_stage_and_model(stage: &str, model: &str) -> Option<&'static Implementation> {
    candidates_for_stage(stage).find(|implementation| implementation.model == model)
}

/// The candidate that runs anywhere.
pub(crate) fn portable_for_stage(stage: &str) -> Option<&'static Implementation> {
    candidates_for_stage(stage).find(|implementation| implementation.portable)
}

/// An implementation that loads a model. Every implementation of one model
/// runs in one worker family, whichever stage it serves.
pub(crate) fn for_model(model: &str) -> Option<&'static Implementation> {
    all()
        .into_iter()
        .find(|implementation| implementation.model == model)
}

/// Whether a worker of `family` runs the implementation a task was planned
/// with. A task whose implementation is not in this table — a stage that
/// loads no model — is for any worker that declares its stage.
pub(crate) fn runs(family: &str, implementation: &str) -> bool {
    lookup(implementation).is_none_or(|known| known.worker == family)
}

/// The implementations of these stages another family runs: the tasks a
/// worker of `family` declaring them must never be handed.
pub(crate) fn foreign_to(family: &str, stages: &[String]) -> Vec<&'static str> {
    all()
        .into_iter()
        .filter(|implementation| {
            implementation.worker != family
                && stages.iter().any(|stage| stage == implementation.stage)
        })
        .map(|implementation| implementation.name)
        .collect()
}

/// What a person calls a worker family, for sentences that say which one to
/// start.
pub(crate) fn worker_title(family: &str) -> &'static str {
    match family {
        "speech-vad" => "speech-detection worker",
        "speech-asr" => "whisper.cpp worker",
        "speech-align" => "word-timing worker",
        "speech-mlx" => "MLX speech worker",
        "editorial" => "editorial worker",
        "detect-faces" => "face-tracking worker",
        "detect-shots" => "shot-detection worker",
        _ => "worker",
    }
}

/// Every capability something here can serve, in a stable order.
///
/// Sorted rather than declaration-ordered, because it drives the order of the
/// bindings in a signed device profile and a canonical document must not
/// depend on where a line happens to sit in this file.
pub(crate) fn candidates_for_capability_names() -> BTreeSet<&'static str> {
    IMPLEMENTATIONS
        .iter()
        .filter(|implementation| implementation.capability != "editorial")
        .map(|implementation| implementation.capability)
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic)]

    use std::collections::BTreeSet;

    use super::{
        IMPLEMENTATIONS, candidates_for_capability, candidates_for_capability_names,
        candidates_for_stage, custom_runtime, for_model, for_stage_and_model, foreign_to, lookup,
        measured_candidates_for_capability, portable_for_stage, register_custom, runs,
        worker_title,
    };

    #[test]
    fn an_implementation_name_identifies_exactly_one_implementation() {
        let names = IMPLEMENTATIONS
            .iter()
            .map(|implementation| implementation.name)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names.len(),
            IMPLEMENTATIONS.len(),
            "two implementations share a name, so a task row cannot say which model it meant"
        );
    }

    /// The whole phase's "runs offline on a machine with no accelerator" claim
    /// reduces to this: every capability keeps one candidate that needs
    /// nothing special, and it is the one an unmeasured device takes.
    #[test]
    fn every_capability_has_exactly_one_portable_candidate() {
        for capability in IMPLEMENTATIONS
            .iter()
            .filter(|implementation| implementation.capability != "editorial")
            .map(|implementation| implementation.capability)
            .collect::<BTreeSet<_>>()
        {
            let portable = candidates_for_capability(capability)
                .filter(|implementation| implementation.portable)
                .collect::<Vec<_>>();
            assert_eq!(
                portable.len(),
                1,
                "{capability} has {} portable candidates",
                portable.len()
            );
            assert!(
                portable[0].accelerator_class.is_empty(),
                "{capability}'s portable candidate demands an accelerator"
            );
        }
    }

    /// A candidate that needs an accelerator must say which class, or the
    /// scheduler would hand its task to a machine that cannot run it.
    #[test]
    fn an_accelerated_candidate_names_the_class_it_needs() {
        for implementation in IMPLEMENTATIONS {
            // An opt-in candidate is chosen by a person, never by measurement,
            // and one of them runs on the CPU on purpose.
            if implementation.portable || implementation.opt_in {
                continue;
            }
            assert!(
                !implementation.accelerator_class.is_empty(),
                "{} is not portable and names no accelerator class",
                implementation.name
            );
        }
    }

    /// Candidates for one capability must agree on which stage runs them, or
    /// selection would bind a capability to an implementation the plan never
    /// asks for.
    #[test]
    fn a_capability_is_served_by_one_stage() {
        for implementation in IMPLEMENTATIONS {
            if implementation.capability == "editorial" {
                continue;
            } // one chosen model serves three operations
            let stages = candidates_for_capability(implementation.capability)
                .map(|candidate| candidate.stage)
                .collect::<BTreeSet<_>>();
            assert_eq!(
                stages.len(),
                1,
                "{} is served by {stages:?}",
                implementation.capability
            );
        }
    }

    /// Every model named here must be one the published registry pins, since
    /// a chosen implementation whose model nobody pinned cannot be keyed.
    #[test]
    fn every_candidate_model_is_pinned_on_a_matching_backend() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry");
        let models = crate::models::ModelRegistry::load(&path).expect("the registry loads");
        for implementation in IMPLEMENTATIONS {
            let manifest = models.get(implementation.model).unwrap_or_else(|| {
                panic!(
                    "{} names {}, which nothing pins",
                    implementation.name, implementation.model
                )
            });
            assert_eq!(
                manifest.capability, implementation.capability,
                "{} loads a model pinned for another capability",
                implementation.name
            );
            assert_eq!(
                manifest.backend, implementation.backend,
                "{} declares a backend the manifest disagrees with",
                implementation.name
            );
        }
    }

    /// A worker package is one process with one registered family, so every
    /// implementation it provides must name that family — and a person must be
    /// told its name when it is the one to start.
    #[test]
    fn every_implementation_of_one_worker_package_names_one_family() {
        let mut families = std::collections::BTreeMap::new();
        for implementation in IMPLEMENTATIONS {
            let package = implementation
                .name
                .split_once('@')
                .map_or(implementation.name, |(package, _)| package);
            let family = families.entry(package).or_insert(implementation.worker);
            assert_eq!(
                *family, implementation.worker,
                "{} names a second family for {package}",
                implementation.name
            );
            assert_ne!(
                worker_title(implementation.worker),
                "worker",
                "{} has no name to give a person",
                implementation.worker
            );
        }
    }

    /// The whisper.cpp and MLX speech workers both declare transcription and
    /// word timing; each is handed only the tasks planned for its own models.
    #[test]
    fn a_task_goes_only_to_the_family_its_implementation_names() {
        let stages = ["speech-asr".to_owned(), "speech-align".to_owned()];
        let foreign = foreign_to("speech-mlx", &stages);
        assert!(foreign.contains(&"clipmill-worker-asr@0.1.0"));
        assert!(foreign.contains(&"clipmill-worker-align@0.1.0"));
        assert!(!foreign.contains(&"clipmill-worker-speech-mlx@0.1.0/align"));
        let foreign = foreign_to("speech-align", &["speech-align".to_owned()]);
        assert_eq!(foreign, ["clipmill-worker-speech-mlx@0.1.0/align"]);
        assert!(runs("speech-mlx", "clipmill-worker-speech-mlx@0.1.0/align"));
        assert!(!runs(
            "speech-align",
            "clipmill-worker-speech-mlx@0.1.0/align"
        ));
        assert!(
            runs("detect-shots", "clipmill-worker-shots@0.1.0"),
            "a stage that loads no model is for whoever declares it"
        );
        assert_eq!(
            for_model("qwen3-aligner-mlx").map(|found| found.worker),
            Some("speech-mlx")
        );
        assert_eq!(
            for_model("qwen3-5-editorial-mlx").map(|found| found.worker),
            Some("editorial")
        );
    }

    #[test]
    fn the_capability_list_is_every_capability_exactly_once_in_a_stable_order() {
        assert_eq!(
            candidates_for_capability_names(),
            ["asr", "detect-faces", "forced-align", "vad"]
                .into_iter()
                .collect(),
        );
        assert!(
            candidates_for_capability_names().into_iter().eq([
                "asr",
                "detect-faces",
                "forced-align",
                "vad"
            ]),
            "a signed profile's binding order must not follow this file's line order"
        );
    }

    #[test]
    fn a_stage_resolves_to_its_candidates_and_its_fallback() {
        // Counted against the bundled table: other tests in this process may
        // register a person's model for the same stage.
        assert_eq!(
            IMPLEMENTATIONS
                .iter()
                .filter(|implementation| implementation.stage == "speech-asr")
                .count(),
            3
        );
        assert_eq!(
            IMPLEMENTATIONS
                .iter()
                .filter(
                    |implementation| implementation.stage == "speech-asr" && !implementation.opt_in
                )
                .count(),
            2,
            "a benchmark ranks the base model against the accelerated one"
        );
        assert_eq!(
            portable_for_stage("speech-asr").expect("a fallback").model,
            "whisper-base"
        );
        assert_eq!(candidates_for_stage("not-a-stage").count(), 0);
        assert!(portable_for_stage("not-a-stage").is_none());
        assert!(lookup("nobody-registered-this").is_none());
        assert_eq!(
            lookup("clipmill-worker-align@0.1.0")
                .expect("registered")
                .model,
            "wav2vec2-ctc-en"
        );
    }

    /// The larger whisper model is on the table so a person can choose it,
    /// and off the benchmark so speed never chooses against accuracy.
    #[test]
    fn opt_in_candidates_are_never_ranked() {
        let large = lookup("clipmill-worker-asr@0.1.0/whisper-large-v3-turbo")
            .expect("the accurate whisper model is plannable");
        assert!(large.opt_in);
        assert!(!large.portable);
        assert!(
            measured_candidates_for_capability("asr")
                .all(|implementation| implementation.model != "whisper-large-v3-turbo")
        );
        assert_eq!(
            for_stage_and_model("speech-asr", "whisper-large-v3-turbo").map(|found| found.name),
            Some(large.name)
        );
    }

    /// A person's model becomes plannable under its own implementation names,
    /// once, and only for the families whose worker can load it.
    #[test]
    fn a_pinned_model_is_registered_for_every_stage_its_worker_runs() {
        register_custom("test-whisper-pinned", "asr").expect("whisper.cpp takes any GGML model");
        register_custom("test-whisper-pinned", "asr").expect("registering twice is harmless");
        let transcription = candidates_for_stage("speech-asr")
            .filter(|implementation| implementation.model == "test-whisper-pinned")
            .collect::<Vec<_>>();
        assert_eq!(
            transcription.len(),
            1,
            "one entry, however often registered"
        );
        assert_eq!(
            transcription[0].name,
            "clipmill-worker-asr@0.1.0/custom/test-whisper-pinned"
        );
        assert!(transcription[0].opt_in);
        assert_eq!(transcription[0].worker, "speech-asr");
        assert!(transcription[0].accelerator_class.is_empty());
        assert!(lookup(transcription[0].name).is_some());

        register_custom("test-editorial-pinned", "editorial").expect("MLX loads a model directory");
        for stage in [
            "editorial-propose",
            "editorial-review",
            "editorial-look",
            "youtube-metadata",
        ] {
            let found = for_stage_and_model(stage, "test-editorial-pinned")
                .unwrap_or_else(|| panic!("{stage} cannot run the pinned model"));
            assert_eq!(found.worker, "editorial");
            assert_eq!(found.accelerator_class, "metal");
            assert_eq!(found.backend, "mlx");
            assert!(found.opt_in);
        }

        assert!(register_custom("test-vad-pinned", "vad").is_err());
        assert!(custom_runtime("detect-faces").is_none());
    }
}
