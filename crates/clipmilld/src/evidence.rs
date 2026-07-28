//! The builtin that turns a transcript into an index.
//!
//! Like the speech assembly beside it, this runs in the daemon rather than in
//! a worker because it loads no model: it is arithmetic over two published
//! JSON documents, and the two-lifecycle rule puts model-free derivation where
//! the artifacts already are. Everything it decides lives in
//! `clipmill-evidence`, which does no I/O — this module is the part that reads
//! the inputs, keys the result, and publishes it.
//!
//! Its two documents arrive on the lease — declared by the plan when this runs
//! standalone, delivered by a dependency inside the analyze DAG — and each is
//! matched against the artifact kind its own manifest declares. A plan that
//! named shot cuts where a transcript belongs is refused rather than parsed into
//! nonsense.

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
    inputs::{self, Wanted},
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
    let resolved = inputs::resolve(
        artifacts,
        task,
        &[
            Wanted::required("speech.transcript.v1"),
            Wanted::optional("evidence.shots.v1"),
        ],
    )
    .await?;
    let transcript: SpeechTranscript = resolved.read("speech.transcript.v1", "transcript.json")?;
    let shots: Option<EvidenceShots> = resolved.read_optional("evidence.shots.v1", "shots.json")?;
    let transcript_id = resolved.address("speech.transcript.v1")?;
    let shots_id = resolved.optional_address("evidence.shots.v1");
    progress.set("stages", 1, 3);

    let parameters = Parameters::DEFAULT;
    let document = clipmill_evidence::index(
        &transcript,
        shots.as_ref(),
        Inputs {
            transcript: &transcript_id,
            shots: shots_id.as_deref(),
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
        inputs: resolved.addresses(),
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

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_contracts::proto::ipc::v1::IndexStagePayloadV1;
    use prost::Message;

    use super::{INDEX_STAGE_KEY_VERSION, KIND_INDEX};

    fn payload() -> IndexStagePayloadV1 {
        IndexStagePayloadV1 {
            key_version: INDEX_STAGE_KEY_VERSION.to_owned(),
            stage: KIND_INDEX.to_owned(),
        }
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

    /// Nothing about what the stage reads is in here. Two indexes over different
    /// transcripts are still different artifacts — the recipe covers the
    /// addresses the lease delivered — but they encode the same payload, which is
    /// exactly what lets one route's index be the other route's cache hit.
    #[test]
    fn the_payload_names_no_input() {
        let encoded = String::from_utf8_lossy(&payload().encode_to_vec()).into_owned();
        assert!(!encoded.contains("sha256:"));
    }
}
