//! The builtin that turns a transcript into an index.
//!
//! Like the speech assembly beside it, this runs in the daemon rather than in
//! a worker because it loads no model: it is arithmetic over two published
//! JSON documents, and the two-lifecycle rule puts model-free derivation where
//! the artifacts already are. Everything it decides lives in
//! `clipmill-evidence`, which does no I/O — this module is the part that reads
//! the inputs, keys the result, and publishes it.
//!
//! The two documents it reads are named in the task payload rather than
//! reached through dependencies, because both were published by earlier jobs
//! and a task's inputs are the outputs of the tasks it depends on. Each is
//! then checked against the artifact kind its own manifest declares, so a
//! payload that named shot cuts where a transcript belongs is refused rather
//! than parsed into nonsense.

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_contracts::proto::ipc::v1::IndexStagePayloadV1;
use clipmill_contracts::schemas::{
    evidence_shots::EvidenceShots, speech_transcript::SpeechTranscript,
};
use clipmill_core::{ArtifactId, Sha256Digest};
use clipmill_evidence::{Inputs, Parameters};
use prost::Message;
use serde_json::{Map, json};

use crate::{
    artifacts::ArtifactHandle,
    jobs::{INDEX_STAGE_KEY_VERSION, LeasedTask, TaskExecutionError},
    media::{self, ProgressSlot},
};

/// The task kind this module executes.
pub(crate) const KIND_INDEX: &str = "index-transcript";
pub(crate) const IMPLEMENTATION: &str = "clipmill-evidence-index@1.0.0";
const OUTPUT_FILE: &str = "index.json";

/// Read the transcript, and the shot cuts if the plan supplied any, and
/// publish the index over them.
#[allow(
    clippy::too_many_lines,
    reason = "read, derive, key, publish — the order is the point"
)]
pub(crate) async fn execute_index_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 3);
    let payload = IndexStagePayloadV1::decode(task.payload.as_slice()).map_err(|_| {
        TaskExecutionError::deterministic("task payload is not an evidence index payload")
    })?;
    if payload.key_version != INDEX_STAGE_KEY_VERSION || payload.stage != KIND_INDEX {
        return Err(TaskExecutionError::deterministic(
            "task payload does not describe the evidence index",
        ));
    }
    if payload.transcript_artifact_id.is_empty() {
        return Err(TaskExecutionError::deterministic(
            "the evidence index has no transcript to read",
        ));
    }

    let transcript: SpeechTranscript = read_input(
        artifacts,
        &payload.transcript_artifact_id,
        "speech.transcript.v1",
        "transcript.json",
    )
    .await?;
    let shots: Option<EvidenceShots> = if payload.shots_artifact_id.is_empty() {
        None
    } else {
        Some(
            read_input(
                artifacts,
                &payload.shots_artifact_id,
                "evidence.shots.v1",
                "shots.json",
            )
            .await?,
        )
    };
    let transcript_id = payload.transcript_artifact_id.clone();
    progress.set("stages", 1, 3);

    let parameters = Parameters::DEFAULT;
    let document = clipmill_evidence::index(
        &transcript,
        shots.as_ref(),
        Inputs {
            transcript: &transcript_id,
            shots: Some(payload.shots_artifact_id.as_str()).filter(|id| !id.is_empty()),
        },
        parameters,
        IMPLEMENTATION,
    )
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    progress.set("stages", 2, 3);

    let fingerprint: Sha256Digest = transcript
        .source_fingerprint
        .strip_prefix("sha256:")
        .unwrap_or_default()
        .parse()
        .map_err(|_| TaskExecutionError::deterministic("the transcript carries no fingerprint"))?;
    let mut config = Map::new();
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.index.transcript.v1"),
    );
    // The parameters reach the key by name, so a re-tune is a different
    // observation rather than a correction of this one — and so re-tuning one
    // of them does not invalidate an index that never depended on it.
    config.insert(
        "utterance_gap_ticks".to_owned(),
        json!(parameters.utterance_gap_ticks),
    );
    config.insert(
        "block_sentences".to_owned(),
        json!(parameters.block_sentences),
    );
    config.insert(
        "boundary_cutoff_milli".to_owned(),
        json!(parameters.boundary_cutoff_milli),
    );
    config.insert("stopwords".to_owned(), json!(clipmill_evidence::STOPWORDS));
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "index.transcript.v1".to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: KIND_INDEX.to_owned(),
            implementation: IMPLEMENTATION.to_owned(),
            // No model ran. Naming one would put a digest in the key that had
            // nothing to do with what this stage computed.
            model_digest: None,
        },
        // The addresses the stage actually read, so the key covers them even
        // though no dependency delivered them.
        inputs: inputs_for(&payload)?,
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.index.transcript.v1".to_owned(),
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
    let value = serde_json::to_value(&document)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let result = async {
        media::write_canonical_json(&staging, &path, &value)?;
        media::commit_staging(artifacts, staging_id.clone(), vec![path]).await
    }
    .await;
    if result.is_err() {
        media::abandon_staging(artifacts, staging_id).await;
    }
    progress.set("stages", 3, 3);
    result
}

/// Open one named artifact and read one document out of it.
///
/// The kind is checked against what the artifact's own manifest declares. A
/// payload that named shot cuts where a transcript belongs would otherwise be
/// a parse error at best and a plausible-looking index at worst.
async fn read_input<T: serde::de::DeserializeOwned>(
    artifacts: &ArtifactHandle,
    artifact_id: &str,
    expected_kind: &str,
    file: &str,
) -> Result<T, TaskExecutionError> {
    let parsed: ArtifactId = artifact_id
        .parse()
        .map_err(|_| TaskExecutionError::deterministic("input artifact id is not an address"))?;
    let lease = artifacts
        .open(parsed)
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    if lease.kind() != expected_kind {
        return Err(TaskExecutionError::deterministic(format!(
            "the evidence index was pointed at a {}, not a {expected_kind}",
            lease.kind()
        )));
    }
    media::read_artifact_document(&lease, file)
}

/// The addresses this stage read, in a stable order, for the artifact key.
fn inputs_for(payload: &IndexStagePayloadV1) -> Result<Vec<ArtifactId>, TaskExecutionError> {
    let mut inputs = Vec::new();
    for address in [
        payload.transcript_artifact_id.as_str(),
        payload.shots_artifact_id.as_str(),
    ] {
        if address.is_empty() {
            continue;
        }
        inputs.push(address.parse::<ArtifactId>().map_err(|_| {
            TaskExecutionError::deterministic("input artifact id is not an address")
        })?);
    }
    Ok(inputs)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_contracts::proto::ipc::v1::IndexStagePayloadV1;
    use prost::Message;

    use super::{INDEX_STAGE_KEY_VERSION, KIND_INDEX, inputs_for};

    const TRANSCRIPT: &str =
        "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
    const SHOTS: &str = "sha256:9c0f000000000000000000000000000000000000000000000000000000000031";

    fn payload() -> IndexStagePayloadV1 {
        IndexStagePayloadV1 {
            key_version: INDEX_STAGE_KEY_VERSION.to_owned(),
            stage: KIND_INDEX.to_owned(),
            transcript_artifact_id: TRANSCRIPT.to_owned(),
            shots_artifact_id: String::new(),
        }
    }

    /// The key must cover both documents the stage read. A key that named only
    /// the transcript would serve one recording's index for another's shots.
    #[test]
    fn the_key_covers_every_document_that_was_read() {
        assert_eq!(inputs_for(&payload()).expect("addresses").len(), 1);
        let with_shots = IndexStagePayloadV1 {
            shots_artifact_id: SHOTS.to_owned(),
            ..payload()
        };
        let inputs = inputs_for(&with_shots).expect("addresses");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].to_string(), TRANSCRIPT);
        assert_eq!(inputs[1].to_string(), SHOTS);
    }

    #[test]
    fn an_address_that_is_not_an_address_is_refused() {
        let malformed = IndexStagePayloadV1 {
            transcript_artifact_id: "/var/folders/transcript.json".to_owned(),
            ..payload()
        };
        assert!(inputs_for(&malformed).is_err());
    }

    /// A payload from another stage decodes into this message shape without
    /// complaint — protobuf field numbers do not carry meaning — so the stage
    /// name is checked rather than assumed.
    #[test]
    fn a_payload_from_another_stage_is_named_as_such() {
        let borrowed = IndexStagePayloadV1 {
            stage: "detect-shots".to_owned(),
            ..payload()
        };
        assert_ne!(borrowed.stage, KIND_INDEX);
        let decoded = IndexStagePayloadV1::decode(borrowed.encode_to_vec().as_slice())
            .expect("it decodes, which is the point");
        assert_ne!(decoded.stage, KIND_INDEX);
    }
}
