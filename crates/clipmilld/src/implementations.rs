//! What can serve a capability here, and which one this device chose.
//!
//! A stage used to name one model. That is a static per-platform default in
//! disguise: it says "whisper on every machine" in a place nobody reads, and
//! the machine with an accelerator sitting idle has no way to say so. D19 is
//! the decision that selection comes from measurement instead, and this module
//! is where the candidates it chooses between are declared.
//!
//! An *implementation* is the triple that actually decides an answer: a worker
//! family, the model it loads, and the backend it loads it on. Two
//! implementations of one capability produce different bytes from the same
//! audio, so they are different producers of different observations — which is
//! why the chosen one reaches the artifact key rather than being an invisible
//! runtime detail. A transcript recognized by Qwen3 on Metal and one
//! recognized by whisper.cpp on a CPU are not interchangeable and must not
//! share a content address.
//!
//! Selection is resolved once, when a job is planned, and written into the
//! task row. Re-measuring the device later changes what the *next* plan
//! chooses and leaves every published artifact exactly where it is.

use std::collections::BTreeSet;

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
    /// The runtime a worker declares to serve this. Only a worker registered
    /// on the same backend can be handed the task.
    pub backend: &'static str,
    /// The accelerator class the scheduler requires of whoever leases it.
    /// Empty means any machine will do, which is what makes a candidate the
    /// portable one.
    pub accelerator_class: &'static str,
    /// Whether this implementation runs on every supported platform. Exactly
    /// one candidate per capability is portable: it is what an unmeasured
    /// device falls back to, and what the offline exit gate rests on.
    pub portable: bool,
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
        backend: "onnx-cpu",
        accelerator_class: "",
        portable: true,
    },
    Implementation {
        name: "clipmill-worker-asr@0.1.0",
        capability: "asr",
        stage: "speech-asr",
        model: "whisper-base",
        backend: "cpu",
        accelerator_class: "",
        portable: true,
    },
    Implementation {
        name: "clipmill-worker-speech-mlx@0.1.0/asr",
        capability: "asr",
        stage: "speech-asr",
        model: "qwen3-asr-mlx",
        backend: "mlx",
        accelerator_class: "metal",
        portable: false,
    },
    Implementation {
        name: "clipmill-worker-align@0.1.0",
        capability: "forced-align",
        stage: "speech-align",
        model: "wav2vec2-ctc-en",
        backend: "onnx-cpu",
        accelerator_class: "",
        portable: true,
    },
    Implementation {
        name: "clipmill-worker-speech-mlx@0.1.0/align",
        capability: "forced-align",
        stage: "speech-align",
        model: "qwen3-aligner-mlx",
        backend: "mlx",
        accelerator_class: "metal",
        portable: false,
    },
    // The face detector. One candidate and no accelerated sibling: YuNet is a
    // 230 kB CPU graph whose whole appeal is having no runtime tail, and an
    // accelerated variant would be a second implementation to keep honest for
    // no measurable gain.
    Implementation {
        name: "clipmill-worker-faces@0.1.0",
        capability: "detect-faces",
        stage: "detect-faces",
        model: "yunet-face",
        backend: "onnx-cpu",
        accelerator_class: "",
        portable: true,
    },
];

/// Every implementation registered for a stage.
pub(crate) fn candidates_for_stage(stage: &str) -> impl Iterator<Item = &'static Implementation> {
    IMPLEMENTATIONS
        .iter()
        .filter(move |implementation| implementation.stage == stage)
}

/// Every implementation registered for a capability.
pub(crate) fn candidates_for_capability(
    capability: &str,
) -> impl Iterator<Item = &'static Implementation> {
    IMPLEMENTATIONS
        .iter()
        .filter(move |implementation| implementation.capability == capability)
}

/// The implementation a task was planned with.
///
/// Every consumer of a task's model identity goes through here, so the key the
/// daemon computed and the weights it hands the worker cannot disagree: both
/// are derived from the one string the plan committed to.
pub(crate) fn lookup(name: &str) -> Option<&'static Implementation> {
    IMPLEMENTATIONS
        .iter()
        .find(|implementation| implementation.name == name)
}

/// The candidate that runs anywhere.
pub(crate) fn portable_for_stage(stage: &str) -> Option<&'static Implementation> {
    candidates_for_stage(stage).find(|implementation| implementation.portable)
}

/// Every capability something here can serve, in a stable order.
///
/// Sorted rather than declaration-ordered, because it drives the order of the
/// bindings in a signed device profile and a canonical document must not
/// depend on where a line happens to sit in this file.
pub(crate) fn candidates_for_capability_names() -> BTreeSet<&'static str> {
    IMPLEMENTATIONS
        .iter()
        .map(|implementation| implementation.capability)
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic)]

    use std::collections::BTreeSet;

    use super::{
        IMPLEMENTATIONS, candidates_for_capability, candidates_for_capability_names,
        candidates_for_stage, lookup, portable_for_stage,
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
            if implementation.portable {
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
        assert_eq!(candidates_for_stage("speech-asr").count(), 2);
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
}
