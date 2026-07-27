//! Turning a speech benchmark into a binding (D19).
//!
//! The daemon cannot run the benchmark itself. Every candidate implementation
//! lives in a Python worker family with its own environment, and the only
//! place a model's real cost can be observed is inside the environment that
//! loads it — a number measured anywhere else would be a guess wearing a
//! measurement's clothes. So `tools/bench/speech-benchmark.py` runs each
//! installed implementation over the pinned fixture and writes what it saw
//! into the daemon's private state directory, and this module decides whether
//! to believe it.
//!
//! Belief is not a matter of trust. The measurement is accepted only when it
//! names *this* hardware fingerprint and the model digests the registry pins
//! right now, so a benchmark survives neither a hardware change nor a
//! re-pinned weight. When it is missing or stale the profile says so and the
//! binding falls back to the portable implementation — stated as a fallback,
//! never presented as a choice somebody measured.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    implementations::{self, Implementation},
    models::ModelRegistry,
};

/// What the benchmark tool writes. Path-free by construction: it names models
/// and implementations, never the directories they were loaded from.
#[derive(Clone, Debug, Deserialize)]
struct BenchmarkDocument {
    schema_version: String,
    hardware_fingerprint: String,
    #[serde(default)]
    measurements: Vec<BenchmarkMeasurement>,
}

#[derive(Clone, Debug, Deserialize)]
struct BenchmarkMeasurement {
    implementation: String,
    model_digest: String,
    runnable: bool,
    #[serde(default)]
    real_time_factor: Option<f64>,
    #[serde(default)]
    peak_resident_bytes: Option<u64>,
    #[serde(default)]
    unavailable_reason: Option<String>,
}

const BENCHMARK_SCHEMA: &str = "clipmill.speech_benchmark.v1";

/// How a capability's implementation was decided.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelectedBy {
    /// A benchmark bound to this device ranked the runnable candidates.
    Measured,
    /// Only one implementation is registered, so nothing was ranked.
    SoleCandidate,
    /// Candidates exist, but no measurement covers this device.
    UnmeasuredFallback,
}

impl SelectedBy {
    fn as_str(self) -> &'static str {
        match self {
            Self::Measured => "measured",
            Self::SoleCandidate => "sole_candidate",
            Self::UnmeasuredFallback => "unmeasured_fallback",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "measured" => Some(Self::Measured),
            "sole_candidate" => Some(Self::SoleCandidate),
            "unmeasured_fallback" => Some(Self::UnmeasuredFallback),
            _ => None,
        }
    }
}

/// One capability's answer: which implementation, and on what grounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Binding {
    pub capability: String,
    pub stage: String,
    pub implementation: String,
    pub model: String,
    pub backend: String,
    pub selected_by: String,
}

impl Binding {
    pub(crate) fn was_measured(&self) -> bool {
        SelectedBy::parse(&self.selected_by) == Some(SelectedBy::Measured)
    }
}

/// Every stage's chosen implementation, keyed by stage kind.
///
/// A stage missing from the map has no registered candidate, which is a
/// programming error rather than a runtime condition — the plan factories only
/// ask about stages the registry declares.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Bindings {
    by_stage: BTreeMap<String, Binding>,
}

impl Bindings {
    pub(crate) fn for_stage(&self, stage: &str) -> Option<&Binding> {
        self.by_stage.get(stage)
    }

    /// The binding every stage falls back to before any device profile has
    /// been measured — the portable candidate, named as such.
    pub(crate) fn portable() -> Self {
        let mut by_stage = BTreeMap::new();
        for capability in implementations::candidates_for_capability_names() {
            let candidates = implementations::candidates_for_capability(capability).count();
            let Some(implementation) = implementations::candidates_for_capability(capability)
                .find(|candidate| candidate.portable)
            else {
                continue;
            };
            let reason = if candidates == 1 {
                SelectedBy::SoleCandidate
            } else {
                SelectedBy::UnmeasuredFallback
            };
            by_stage.insert(
                implementation.stage.to_owned(),
                binding_of(implementation, reason),
            );
        }
        Self { by_stage }
    }

    pub(crate) fn from_profile(value: &Value) -> Self {
        let mut by_stage = BTreeMap::new();
        let entries = value
            .get("selection")
            .and_then(|selection| selection.get("bindings"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten();
        for entry in entries {
            let Some(binding) = parse_binding(entry) else {
                continue;
            };
            by_stage.insert(binding.stage.clone(), binding);
        }
        Self { by_stage }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.by_stage.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Binding> {
        self.by_stage.values()
    }
}

fn parse_binding(entry: &Value) -> Option<Binding> {
    let field = |name: &str| {
        entry
            .get(name)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    let selected_by = field("selected_by")?;
    SelectedBy::parse(&selected_by)?;
    Some(Binding {
        capability: field("capability")?,
        stage: field("stage")?,
        implementation: field("implementation")?,
        model: field("model")?,
        backend: field("backend")?,
        selected_by,
    })
}

fn binding_of(implementation: &Implementation, reason: SelectedBy) -> Binding {
    Binding {
        capability: implementation.capability.to_owned(),
        stage: implementation.stage.to_owned(),
        implementation: implementation.name.to_owned(),
        model: implementation.model.to_owned(),
        backend: implementation.backend.to_owned(),
        selected_by: reason.as_str().to_owned(),
    }
}

/// What a device profile learned about its implementations.
#[derive(Clone, Debug)]
pub(crate) struct Selection {
    /// The `selection` block, ready to be signed with the rest of the profile.
    pub value: Value,
    /// Accelerator classes something was measured actually running on.
    ///
    /// This is the only evidence the daemon accepts that an accelerator is
    /// usable. It cannot load MLX to look, and "macOS on ARM has Metal" is a
    /// static platform default — the thing D19 exists to replace. A model that
    /// ran on this machine and reported how fast it was is a stronger claim
    /// than any probe, so the scheduler admits an accelerated worker exactly
    /// when a benchmark has demonstrated the accelerator, and not before.
    pub proven_accelerators: BTreeSet<&'static str>,
}

/// The `selection` block of a device profile: the bindings, and the evidence.
#[allow(
    clippy::too_many_lines,
    reason = "one pass over the candidates, keeping the evidence beside the decision it produced"
)]
pub(crate) fn measure(
    benchmark_path: &Path,
    hardware_fingerprint: &str,
    models: &ModelRegistry,
) -> Selection {
    let measurements = read_benchmark(benchmark_path, hardware_fingerprint);
    let mut bindings = Vec::new();
    let mut candidates = Vec::new();
    let mut proven_accelerators = BTreeSet::new();
    for capability in implementations::candidates_for_capability_names() {
        let registered = implementations::candidates_for_capability(capability).collect::<Vec<_>>();
        let mut runnable: Vec<(&Implementation, f64, u64)> = Vec::new();
        for implementation in &registered {
            // A candidate whose model the registry no longer pins cannot be
            // planned at all, so it is reported unavailable rather than
            // silently dropped: a candidate list that omits its own gaps is
            // not evidence.
            let Some(digest) = models
                .get(implementation.model)
                .map(|manifest| manifest.digest().to_string())
            else {
                candidates.push(unavailable_candidate(
                    implementation,
                    &format!("{} is not pinned by this registry", implementation.model),
                    None,
                ));
                continue;
            };
            let digest = format!("sha256:{digest}");
            match measurements.as_ref().and_then(|measured| {
                measured
                    .iter()
                    .find(|entry| entry.implementation == implementation.name)
            }) {
                None => candidates.push(unavailable_candidate(
                    implementation,
                    "no benchmark on this device covers this implementation",
                    Some(&digest),
                )),
                Some(entry) if entry.model_digest != digest => {
                    candidates.push(unavailable_candidate(
                        implementation,
                        "the benchmark measured a different revision of this model",
                        Some(&digest),
                    ));
                }
                Some(entry) => match runnable_measurement(entry) {
                    Some((factor, peak)) => {
                        runnable.push((implementation, factor, peak));
                        if !implementation.accelerator_class.is_empty() {
                            proven_accelerators.insert(implementation.accelerator_class);
                        }
                        candidates.push(json!({
                            "backend": implementation.backend,
                            "capability": implementation.capability,
                            "implementation": implementation.name,
                            "model": implementation.model,
                            "model_digest": digest,
                            "peak_resident_bytes": peak,
                            "real_time_factor": factor,
                            "runnable": true,
                        }));
                    }
                    None => candidates.push(unavailable_candidate(
                        implementation,
                        entry
                            .unavailable_reason
                            .as_deref()
                            .unwrap_or("the benchmark reported no usable measurement"),
                        Some(&digest),
                    )),
                },
            }
        }
        // Fastest first; a tie on speed goes to the one that fits in less
        // memory, and a tie on both goes to the earlier name so the answer
        // does not depend on iteration order.
        runnable.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then(left.2.cmp(&right.2))
                .then(left.0.name.cmp(right.0.name))
        });
        let portable = registered
            .iter()
            .copied()
            .find(|implementation| implementation.portable);
        let chosen = match (runnable.first(), portable) {
            (Some((implementation, _, _)), _) if registered.len() > 1 => {
                Some((*implementation, SelectedBy::Measured))
            }
            (Some((implementation, _, _)), _) => Some((*implementation, SelectedBy::SoleCandidate)),
            (None, Some(implementation)) if registered.len() > 1 => {
                Some((implementation, SelectedBy::UnmeasuredFallback))
            }
            (None, Some(implementation)) => Some((implementation, SelectedBy::SoleCandidate)),
            (None, None) => None,
        };
        if let Some((implementation, reason)) = chosen {
            let mut entry = json!({
                "backend": implementation.backend,
                "capability": implementation.capability,
                "implementation": implementation.name,
                "model": implementation.model,
                "selected_by": reason.as_str(),
                "stage": implementation.stage,
            });
            if let (SelectedBy::Measured, Some((_, factor, _))) = (reason, runnable.first()) {
                let detail = format!(
                    "fastest of {} runnable candidates at {factor:.2}x real time",
                    runnable.len()
                );
                entry["detail"] = Value::String(detail);
            }
            bindings.push(entry);
        }
    }
    Selection {
        value: json!({ "bindings": bindings, "candidates": candidates }),
        proven_accelerators,
    }
}

fn runnable_measurement(entry: &BenchmarkMeasurement) -> Option<(f64, u64)> {
    if !entry.runnable {
        return None;
    }
    let factor = entry
        .real_time_factor
        .filter(|value| value.is_finite() && *value > 0.0)?;
    let peak = entry.peak_resident_bytes.filter(|value| *value > 0)?;
    Some((factor, peak))
}

fn unavailable_candidate(
    implementation: &Implementation,
    reason: &str,
    digest: Option<&str>,
) -> Value {
    json!({
        "backend": implementation.backend,
        "capability": implementation.capability,
        "implementation": implementation.name,
        "model": implementation.model,
        // A candidate whose model is unpinned still needs a digest-shaped
        // field, and an all-zero digest is the honest value: nothing is
        // pinned, so nothing was measured against anything.
        "model_digest": digest.unwrap_or("sha256:0000000000000000000000000000000000000000000000000000000000000000"),
        "runnable": false,
        "unavailable_reason": reason.chars().take(512).collect::<String>(),
    })
}

/// The benchmark, if there is one bound to this device.
///
/// Every rejection is silent by design — a missing or stale benchmark is a
/// normal state of a machine nobody has benchmarked yet, not a fault. What it
/// causes is visible in the profile: every candidate reports that no
/// measurement covers it, and the binding says `unmeasured_fallback`.
fn read_benchmark(path: &Path, hardware_fingerprint: &str) -> Option<Vec<BenchmarkMeasurement>> {
    let text = fs::read_to_string(path).ok()?;
    let document: BenchmarkDocument = serde_json::from_str(&text).ok()?;
    if document.schema_version != BENCHMARK_SCHEMA
        || document.hardware_fingerprint != hardware_fingerprint
    {
        return None;
    }
    Some(document.measurements)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::path::Path;

    use serde_json::{Value, json};
    use tempfile::TempDir;

    use super::{Bindings, Selection, measure};

    /// Every test here is about the block the profile publishes; the
    /// accelerator half is exercised where it is consumed.
    fn measure_value(
        benchmark_path: &Path,
        hardware_fingerprint: &str,
        models: &ModelRegistry,
    ) -> Value {
        let Selection { value, .. } = measure(benchmark_path, hardware_fingerprint, models);
        value
    }
    use crate::models::ModelRegistry;

    const FINGERPRINT: &str =
        "sha256:aa11bb22cc33dd44ee55ff66aa11bb22cc33dd44ee55ff66aa11bb22cc33dd44";

    fn registry() -> ModelRegistry {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry");
        ModelRegistry::load(&path).expect("the published registry loads")
    }

    fn digest_of(models: &ModelRegistry, name: &str) -> String {
        format!("sha256:{}", models.get(name).expect("pinned").digest())
    }

    fn benchmark(temp: &TempDir, document: &Value) -> std::path::PathBuf {
        let path = temp.path().join("speech-benchmark.json");
        std::fs::write(&path, serde_json::to_vec(document).expect("json")).expect("write");
        path
    }

    fn binding_for<'a>(selection: &'a Value, capability: &str) -> &'a Value {
        selection["bindings"]
            .as_array()
            .expect("bindings")
            .iter()
            .find(|entry| entry["capability"] == capability)
            .expect("a binding for the capability")
    }

    fn candidate_for<'a>(selection: &'a Value, implementation: &str) -> &'a Value {
        selection["candidates"]
            .as_array()
            .expect("candidates")
            .iter()
            .find(|entry| entry["implementation"] == implementation)
            .expect("a candidate for the implementation")
    }

    /// The state of a machine nobody has benchmarked. Every capability still
    /// resolves, and every one of them says out loud that nothing was
    /// measured.
    #[test]
    fn a_device_with_no_benchmark_falls_back_and_says_so() {
        let models = registry();
        let selection = measure_value(
            Path::new("/nonexistent/benchmark.json"),
            FINGERPRINT,
            &models,
        );

        assert_eq!(binding_for(&selection, "asr")["model"], "whisper-base");
        assert_eq!(
            binding_for(&selection, "asr")["selected_by"],
            "unmeasured_fallback"
        );
        // Voice activity has one implementation, so no measurement could have
        // changed the answer and calling it a fallback would overstate it.
        assert_eq!(
            binding_for(&selection, "vad")["selected_by"],
            "sole_candidate"
        );
        let candidate = candidate_for(&selection, "clipmill-worker-speech-mlx@0.1.0/asr");
        assert_eq!(candidate["runnable"], false);
        assert!(
            candidate["unavailable_reason"]
                .as_str()
                .expect("a reason")
                .contains("no benchmark")
        );
    }

    /// The measurement is what changes the answer — and only for the
    /// capabilities it actually covers.
    #[test]
    fn a_faster_candidate_wins_the_capability_it_was_measured_for() {
        let models = registry();
        let temp = TempDir::new().expect("temp");
        let path = benchmark(
            &temp,
            &json!({
                "schema_version": "clipmill.speech_benchmark.v1",
                "hardware_fingerprint": FINGERPRINT,
                "measurements": [
                    {
                        "implementation": "clipmill-worker-asr@0.1.0",
                        "model_digest": digest_of(&models, "whisper-base"),
                        "runnable": true,
                        "real_time_factor": 3.5,
                        "peak_resident_bytes": 700_000_000_u64,
                    },
                    {
                        "implementation": "clipmill-worker-speech-mlx@0.1.0/asr",
                        "model_digest": digest_of(&models, "qwen3-asr-mlx"),
                        "runnable": true,
                        "real_time_factor": 19.0,
                        "peak_resident_bytes": 3_400_000_000_u64,
                    },
                ],
            }),
        );

        let selection = measure_value(&path, FINGERPRINT, &models);
        let asr = binding_for(&selection, "asr");
        assert_eq!(asr["model"], "qwen3-asr-mlx");
        assert_eq!(asr["selected_by"], "measured");
        assert!(
            asr["detail"].as_str().expect("a detail").contains("19.00x"),
            "the binding states the measurement that decided it"
        );
        // Alignment was not measured, so it must not have moved.
        assert_eq!(
            binding_for(&selection, "forced-align")["selected_by"],
            "unmeasured_fallback"
        );
        assert_eq!(
            binding_for(&selection, "forced-align")["model"],
            "wav2vec2-ctc-en"
        );
    }

    /// A benchmark from another machine is not evidence about this one.
    #[test]
    fn a_benchmark_taken_on_other_hardware_is_ignored_entirely() {
        let models = registry();
        let temp = TempDir::new().expect("temp");
        let path = benchmark(
            &temp,
            &json!({
                "schema_version": "clipmill.speech_benchmark.v1",
                "hardware_fingerprint": "sha256:".to_owned() + &"9".repeat(64),
                "measurements": [{
                    "implementation": "clipmill-worker-speech-mlx@0.1.0/asr",
                    "model_digest": digest_of(&models, "qwen3-asr-mlx"),
                    "runnable": true,
                    "real_time_factor": 40.0,
                    "peak_resident_bytes": 3_400_000_000_u64,
                }],
            }),
        );

        let selection = measure_value(&path, FINGERPRINT, &models);
        assert_eq!(
            binding_for(&selection, "asr")["selected_by"],
            "unmeasured_fallback"
        );
    }

    /// Re-pinning a model invalidates the measurement that was taken against
    /// the old one, and nothing else.
    #[test]
    fn a_measurement_of_a_repinned_model_is_stale_not_merely_old() {
        let models = registry();
        let temp = TempDir::new().expect("temp");
        let path = benchmark(
            &temp,
            &json!({
                "schema_version": "clipmill.speech_benchmark.v1",
                "hardware_fingerprint": FINGERPRINT,
                "measurements": [
                    {
                        "implementation": "clipmill-worker-speech-mlx@0.1.0/asr",
                        "model_digest": "sha256:".to_owned() + &"1".repeat(64),
                        "runnable": true,
                        "real_time_factor": 40.0,
                        "peak_resident_bytes": 3_400_000_000_u64,
                    },
                    {
                        "implementation": "clipmill-worker-align@0.1.0",
                        "model_digest": digest_of(&models, "wav2vec2-ctc-en"),
                        "runnable": true,
                        "real_time_factor": 8.0,
                        "peak_resident_bytes": 900_000_000_u64,
                    },
                ],
            }),
        );

        let selection = measure_value(&path, FINGERPRINT, &models);
        assert_eq!(
            binding_for(&selection, "asr")["selected_by"],
            "unmeasured_fallback",
            "the only ASR measurement was against a model nobody pins now"
        );
        assert!(
            candidate_for(&selection, "clipmill-worker-speech-mlx@0.1.0/asr")["unavailable_reason"]
                .as_str()
                .expect("a reason")
                .contains("different revision")
        );
        // The other capability's measurement is untouched by the first's
        // staleness, which is the whole point of keying per model.
        assert_eq!(
            binding_for(&selection, "forced-align")["selected_by"],
            "measured"
        );
    }

    /// A device where the accelerated candidate exists but cannot run is a
    /// measured answer, not an absence of one.
    #[test]
    fn a_candidate_the_benchmark_could_not_run_is_reported_with_its_reason() {
        let models = registry();
        let temp = TempDir::new().expect("temp");
        let path = benchmark(
            &temp,
            &json!({
                "schema_version": "clipmill.speech_benchmark.v1",
                "hardware_fingerprint": FINGERPRINT,
                "measurements": [
                    {
                        "implementation": "clipmill-worker-speech-mlx@0.1.0/asr",
                        "model_digest": digest_of(&models, "qwen3-asr-mlx"),
                        "runnable": false,
                        "unavailable_reason": "mlx is not installed on this platform",
                    },
                    {
                        "implementation": "clipmill-worker-asr@0.1.0",
                        "model_digest": digest_of(&models, "whisper-base"),
                        "runnable": true,
                        "real_time_factor": 3.5,
                        "peak_resident_bytes": 700_000_000_u64,
                    },
                ],
            }),
        );

        let selection = measure_value(&path, FINGERPRINT, &models);
        let asr = binding_for(&selection, "asr");
        assert_eq!(asr["model"], "whisper-base");
        assert_eq!(
            asr["selected_by"], "measured",
            "the benchmark ran and established which candidate works here"
        );
        assert_eq!(
            candidate_for(&selection, "clipmill-worker-speech-mlx@0.1.0/asr")["unavailable_reason"],
            "mlx is not installed on this platform"
        );
    }

    /// A measurement claiming to be runnable without numbers is not a
    /// measurement.
    #[test]
    fn a_runnable_claim_with_no_numbers_behind_it_is_refused() {
        let models = registry();
        let temp = TempDir::new().expect("temp");
        let path = benchmark(
            &temp,
            &json!({
                "schema_version": "clipmill.speech_benchmark.v1",
                "hardware_fingerprint": FINGERPRINT,
                "measurements": [{
                    "implementation": "clipmill-worker-speech-mlx@0.1.0/asr",
                    "model_digest": digest_of(&models, "qwen3-asr-mlx"),
                    "runnable": true,
                }],
            }),
        );

        let selection = measure_value(&path, FINGERPRINT, &models);
        assert_eq!(
            binding_for(&selection, "asr")["selected_by"],
            "unmeasured_fallback"
        );
        assert_eq!(
            candidate_for(&selection, "clipmill-worker-speech-mlx@0.1.0/asr")["runnable"],
            false
        );
    }

    /// The bindings a profile publishes are the bindings a plan reads back.
    #[test]
    fn bindings_survive_the_round_trip_through_a_profile() {
        let models = registry();
        let selection = measure_value(Path::new("/nonexistent"), FINGERPRINT, &models);
        let profile = json!({ "selection": selection });
        let bindings = Bindings::from_profile(&profile);

        assert_eq!(
            bindings
                .for_stage("speech-asr")
                .expect("a binding")
                .implementation,
            "clipmill-worker-asr@0.1.0"
        );
        assert!(bindings.for_stage("not-a-stage").is_none());
        assert_eq!(bindings, Bindings::portable());
    }

    /// A profile from before selection existed leaves the daemon with nothing
    /// to read, and the caller has to notice rather than get a default that
    /// looks measured.
    #[test]
    fn a_profile_without_a_selection_block_yields_no_bindings() {
        assert!(Bindings::from_profile(&json!({})).is_empty());
        assert!(!Bindings::portable().is_empty());
        assert!(
            !Bindings::portable()
                .iter()
                .any(super::Binding::was_measured)
        );
    }
}
