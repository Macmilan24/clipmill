//! The fan-in that closes an analysis.
//!
//! A job roots exactly one artifact, and garbage collection walks recipe inputs
//! from the roots. An analysis produces ten observations, so the last task names
//! all of them: the manifest's recipe lists every stage, and reachability of the
//! whole analysis follows from that one root. Delete the manifest and the
//! analysis becomes collectable; keep it and nothing under it can be swept.
//!
//! It is also the document a shell reads to find out what a project has. One
//! read instead of nine, and the nine addresses in it are what the reader opens
//! next.
//!
//! Two things it refuses to guess. Coverage is the narrowest of what the stages
//! that measured the recording actually examined, not the source's duration —
//! a consumer reading a candidate outside that range is reading a claim nobody
//! made. And a stage the plan never ran is listed as skipped with the property of
//! the source that skipped it, because "this recording has no shot cuts" and
//! "nobody looked for shot cuts" are different facts and the difference is
//! invisible from an empty list.

use std::collections::BTreeMap;

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_contracts::proto::ipc::v1::AnalysisStagePayloadV1;
use clipmill_core::{ArtifactId, Sha256Digest};
use prost::Message;
use serde_json::{Map, Value, json};

use crate::{
    artifacts::ArtifactHandle,
    jobs::{ANALYSIS_STAGE_KEY_VERSION, LeasedTask, TaskExecutionError},
    media::{self, ProgressSlot},
};

/// The task kind this module executes.
pub(crate) const KIND_MANIFEST: &str = "analysis-manifest";
pub(crate) const IMPLEMENTATION: &str = "clipmill-analysis-manifest@1.0.0";
const OUTPUT_FILE: &str = "analysis.json";

/// Stage kinds the manifest may name, in the order an analysis produces them.
///
/// A list rather than "whatever turned up", because the document's own schema
/// enumerates them: a plan that grew a stage nobody added here would publish an
/// artifact that fails its own contract, and finding that out at write time with
/// the kind named beats finding it out in a consumer.
const STAGE_ORDER: [&str; 10] = [
    "evidence.source_map.v1",
    "media.ingest_manifest.v1",
    "speech.vad.v1",
    "speech.asr.v1",
    "speech.alignment.v1",
    "speech.transcript.v1",
    "evidence.shots.v1",
    "index.transcript.v1",
    "discovery.candidates.v1",
    "ranking.set.v1",
];

/// Where each stage that states a coverage span keeps it.
///
/// Only the two that measured the recording. The index, the candidate set, and
/// the ranked set each carry a coverage block too, but every one of them is
/// derived from these — their spans cannot be wider, and intersecting them again
/// would be counting the same measurement twice.
const MEASURED_COVERAGE: [(&str, &str); 2] = [
    ("speech.transcript.v1", "transcript.json"),
    ("evidence.shots.v1", "shots.json"),
];

/// Read every stage this analysis produced and publish the manifest over them.
#[allow(
    clippy::too_many_lines,
    reason = "gather, intersect coverage, key, publish — the order is the point"
)]
pub(crate) async fn execute_manifest_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 3);
    let payload = AnalysisStagePayloadV1::decode(task.payload.as_slice()).map_err(|_| {
        TaskExecutionError::deterministic("task payload is not an analysis manifest payload")
    })?;
    if payload.key_version != ANALYSIS_STAGE_KEY_VERSION || payload.stage != KIND_MANIFEST {
        return Err(TaskExecutionError::deterministic(
            "task payload does not describe the analysis manifest",
        ));
    }
    if task.input_artifact_ids.is_empty() {
        return Err(TaskExecutionError::deterministic(
            "an analysis manifest with no stages describes no analysis",
        ));
    }

    // Every dependency's output, indexed by the kind it declares. Matched by
    // kind rather than by position: the plan's dependency order is an ordering
    // constraint, and reading the third input as the transcript because the
    // transcript is usually third is how a manifest ends up naming a candidate
    // set as an index.
    let mut by_kind: BTreeMap<String, ArtifactId> = BTreeMap::new();
    for artifact_id in &task.input_artifact_ids {
        let lease = artifacts
            .open(*artifact_id)
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        let kind = lease.kind().to_owned();
        if !STAGE_ORDER.contains(&kind.as_str()) {
            return Err(TaskExecutionError::deterministic(format!(
                "{kind} is not a stage an analysis manifest names"
            )));
        }
        if by_kind.insert(kind.clone(), *artifact_id).is_some() {
            return Err(TaskExecutionError::deterministic(format!(
                "this analysis produced two {kind} artifacts and names no choice"
            )));
        }
    }
    progress.set("stages", 1, 3);

    let mut coverage: Option<(u64, u64, bool)> = None;
    for (kind, file) in MEASURED_COVERAGE {
        let Some(artifact_id) = by_kind.get(kind) else {
            continue;
        };
        let lease = artifacts
            .open(*artifact_id)
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        let document: Value = media::read_artifact_document(&lease, file)?;
        let span = read_coverage(&document).ok_or_else(|| {
            TaskExecutionError::deterministic(format!("{kind} states no coverage"))
        })?;
        coverage = Some(match coverage {
            None => span,
            // The narrowest of them, and analyzed only if every one of them
            // was: a span two stages disagree about is a span only one of them
            // can speak for.
            Some(current) => (
                current.0.max(span.0),
                current.1.min(span.1),
                current.2 && span.2,
            ),
        });
    }
    let (start_ticks, end_ticks, analyzed) = coverage.ok_or_else(|| {
        TaskExecutionError::deterministic("no stage in this analysis measured the recording")
    })?;
    // An intersection can be empty — two stages that examined disjoint spans
    // have no span between them. Reported as an examined range of zero length
    // rather than as an inverted one nobody could read.
    let end_ticks = end_ticks.max(start_ticks);

    let stages = STAGE_ORDER
        .iter()
        .filter_map(|kind| {
            by_kind
                .get(*kind)
                .map(|artifact_id| json!({ "kind": kind, "artifact_id": artifact_id.to_string() }))
        })
        .collect::<Vec<_>>();
    let skipped = payload
        .skipped
        .iter()
        .map(|stage| json!({ "kind": stage.kind, "reason": stage.reason }))
        .collect::<Vec<_>>();
    progress.set("stages", 2, 3);

    let fingerprint: Sha256Digest = payload
        .source_fingerprint
        .strip_prefix("sha256:")
        .unwrap_or_default()
        .parse()
        .map_err(|_| {
            TaskExecutionError::deterministic("the analysis names no source fingerprint")
        })?;
    let mut config = Map::new();
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.analysis.manifest.v1"),
    );
    // The skip list reaches the key, because an analysis that skipped shot
    // detection is a different analysis from one that ran it — even when every
    // artifact it does name is identical.
    config.insert("skipped".to_owned(), json!(skipped));
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "analysis.manifest.v1".to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: KIND_MANIFEST.to_owned(),
            implementation: IMPLEMENTATION.to_owned(),
            model_digest: None,
        },
        // Every stage, in the document's own order rather than the dependency
        // order that delivered them, so an analysis whose plan listed its stages
        // differently is still the same analysis.
        inputs: STAGE_ORDER
            .iter()
            .filter_map(|kind| by_kind.get(*kind).copied())
            .collect(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.analysis.manifest.v1".to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;

    let staging = match media::prepare_or_hit(artifacts, recipe).await? {
        media::Prepared::Hit(artifact_id) => {
            progress.set("stages", 3, 3);
            return Ok(artifact_id);
        }
        media::Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let path = media::artifact_path(OUTPUT_FILE)?;
    let mut document = Map::new();
    document.insert(
        "schema_version".to_owned(),
        json!("clipmill.analysis.manifest.v1"),
    );
    document.insert(
        "source_fingerprint".to_owned(),
        json!(payload.source_fingerprint),
    );
    document.insert("stages".to_owned(), json!(stages));
    document.insert(
        "coverage".to_owned(),
        json!({
            "start_ticks": start_ticks,
            "end_ticks": end_ticks,
            "analyzed": analyzed,
        }),
    );
    // Absent rather than empty when nothing was skipped: an empty list reads
    // like a list somebody is still filling in.
    if !skipped.is_empty() {
        document.insert("skipped".to_owned(), json!(skipped));
    }
    let result = async {
        media::write_canonical_json(&staging, &path, &Value::Object(document))?;
        media::commit_staging(artifacts, staging_id.clone(), vec![path]).await
    }
    .await;
    if result.is_err() {
        media::abandon_staging(artifacts, staging_id).await;
    }
    progress.set("stages", 3, 3);
    result
}

/// The coverage block of any document that states one.
///
/// Read structurally rather than through a generated type because the two
/// documents this reads are different schemas that happen to agree about this
/// one object, and naming both types here would mean deserializing two whole
/// observations to look at six fields.
fn read_coverage(document: &Value) -> Option<(u64, u64, bool)> {
    let coverage = document.get("coverage")?;
    Some((
        coverage.get("start_ticks")?.as_u64()?,
        coverage.get("end_ticks")?.as_u64()?,
        coverage.get("analyzed")?.as_bool()?,
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use serde_json::json;

    use super::{MEASURED_COVERAGE, STAGE_ORDER, read_coverage};

    /// The document's schema enumerates the kinds it accepts, so this list and
    /// that enum have to agree — and this is the copy the daemon writes from.
    #[test]
    fn every_stage_the_schema_names_is_a_stage_this_module_names() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../contracts/schemas/clipmill.analysis.manifest.v1.json"
        ))
        .expect("the schema parses");
        let named = schema["$defs"]["stage"]["properties"]["kind"]["enum"]
            .as_array()
            .expect("the schema enumerates stage kinds")
            .iter()
            .map(|value| value.as_str().expect("a string").to_owned())
            .collect::<Vec<_>>();
        assert_eq!(named, STAGE_ORDER.to_vec());
    }

    /// Both stages this intersects coverage from must be stages it can name.
    #[test]
    fn coverage_is_only_read_from_stages_the_manifest_names() {
        for (kind, _) in MEASURED_COVERAGE {
            assert!(STAGE_ORDER.contains(&kind), "{kind} is not a named stage");
        }
    }

    #[test]
    fn a_document_with_no_coverage_reports_none_rather_than_zero() {
        assert_eq!(read_coverage(&json!({})), None);
        assert_eq!(
            read_coverage(&json!({"coverage": {"start_ticks": 0, "end_ticks": 90_000}})),
            None,
            "a coverage block that does not say whether it was analyzed is not one"
        );
        assert_eq!(
            read_coverage(
                &json!({"coverage": {"start_ticks": 90, "end_ticks": 90_000, "analyzed": true}})
            ),
            Some((90, 90_000, true))
        );
    }
}
