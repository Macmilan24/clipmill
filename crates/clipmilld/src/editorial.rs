//! The builtin that cuts a transcript into the windows an editorial model reads.
//!
//! The first of the editorial stages (plan, Milestone 2), and the one that
//! runs no model: like the evidence index beside it, this is arithmetic over
//! published documents, so the two-lifecycle rule puts it in the daemon where
//! the artifacts already are. Everything it decides lives in
//! `clipmill-editorial`, which does no I/O — this module reads the inputs,
//! keys the result, and publishes it.
//!
//! Both documents arrive on the lease, delivered by the index and transcript
//! tasks inside the analyze DAG, and each is matched against the artifact
//! kind its manifest declares. The transcript is read for one thing only: its
//! address, which the index names as the document it was built over. An index
//! and a transcript that do not belong to each other are refused rather than
//! cut into windows that cite words the transcript never held.

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_contracts::proto::ipc::v1::EditorialStagePayloadV1;
use clipmill_contracts::schemas::index_transcript::IndexTranscript;
use clipmill_core::{ArtifactId, Sha256Digest};
use clipmill_editorial::{Budget, Inputs};
use prost::Message;
use serde_json::{Map, json};

use crate::{
    artifacts::ArtifactHandle,
    inputs::{self, Wanted},
    jobs::{EDITORIAL_STAGE_KEY_VERSION, LeasedTask, TaskExecutionError},
    media::{self, ProgressSlot},
};

/// The task kind this module executes.
pub(crate) const KIND_WINDOWS: &str = clipmill_editorial::STAGE;
pub(crate) const IMPLEMENTATION: &str = "clipmill-editorial-windows@1.0.0";
const OUTPUT_FILE: &str = "windows.json";

/// Read the index and the transcript it was built over, and publish the
/// windows cut from them.
pub(crate) async fn execute_windows_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 3);
    let payload = EditorialStagePayloadV1::decode(task.payload.as_slice()).map_err(|_| {
        TaskExecutionError::deterministic("task payload is not an editorial stage payload")
    })?;
    if payload.key_version != EDITORIAL_STAGE_KEY_VERSION || payload.stage != KIND_WINDOWS {
        return Err(TaskExecutionError::deterministic(
            "task payload does not describe the editorial windows",
        ));
    }
    let resolved = inputs::resolve(
        artifacts,
        task,
        &[
            Wanted::required("index.transcript.v1"),
            Wanted::required("speech.transcript.v1"),
        ],
    )
    .await?;
    let index: IndexTranscript = resolved.read("index.transcript.v1", "index.json")?;
    let index_id = resolved.address("index.transcript.v1")?;
    let transcript_id = resolved.address("speech.transcript.v1")?;
    if index.inputs.transcript_artifact_id.as_str() != transcript_id {
        return Err(TaskExecutionError::deterministic(
            "the index was not built over the transcript this stage was given",
        ));
    }
    progress.set("stages", 1, 3);

    let budget = Budget::DEFAULT;
    let document = clipmill_editorial::windows(
        &index,
        Inputs {
            index: &index_id,
            transcript: &transcript_id,
        },
        budget,
        IMPLEMENTATION,
    )
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    progress.set("stages", 2, 3);

    let fingerprint: Sha256Digest = index
        .source_fingerprint
        .strip_prefix("sha256:")
        .unwrap_or_default()
        .parse()
        .map_err(|_| TaskExecutionError::deterministic("the index carries no fingerprint"))?;
    let mut config = Map::new();
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.editorial.windows.v1"),
    );
    // The budget reaches the key by name, so a re-cut under another budget is
    // a different reading of the transcript rather than a correction of this
    // one — and a model's proposals, keyed on the windows they read, are not
    // mistaken for proposals over windows it never saw.
    config.insert("target_words".to_owned(), json!(budget.target_words));
    config.insert("overlap_words".to_owned(), json!(budget.overlap_words));
    config.insert(
        "context_sentences".to_owned(),
        json!(budget.context_sentences),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "editorial.windows.v1".to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: KIND_WINDOWS.to_owned(),
            implementation: IMPLEMENTATION.to_owned(),
            // No model ran. The model that reads these windows names itself
            // in its own artifact.
            model_digest: None,
        },
        inputs: resolved.addresses(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.editorial.windows.v1".to_owned(),
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

    use clipmill_contracts::proto::ipc::v1::EditorialStagePayloadV1;
    use prost::Message;

    use super::{EDITORIAL_STAGE_KEY_VERSION, KIND_WINDOWS};

    fn payload() -> EditorialStagePayloadV1 {
        EditorialStagePayloadV1 {
            key_version: EDITORIAL_STAGE_KEY_VERSION.to_owned(),
            stage: KIND_WINDOWS.to_owned(),
        }
    }

    /// The stage name the recipe registry knows is the one the crate that
    /// does the cutting writes into its documents, so a windows document's
    /// producer and the task that published it never disagree.
    #[test]
    fn the_stage_is_named_once() {
        assert_eq!(KIND_WINDOWS, "editorial-windows");
    }

    /// A payload from another stage decodes into this message shape without
    /// complaint — protobuf field numbers do not carry meaning — so the stage
    /// name is checked rather than assumed.
    #[test]
    fn a_payload_from_another_stage_is_named_as_such() {
        let borrowed = EditorialStagePayloadV1 {
            stage: "index-transcript".to_owned(),
            ..payload()
        };
        let decoded = EditorialStagePayloadV1::decode(borrowed.encode_to_vec().as_slice())
            .expect("it decodes, which is the point");
        assert_ne!(decoded.stage, KIND_WINDOWS);
    }

    /// Nothing about what the stage reads is in here; the recipe covers the
    /// addresses the lease delivered, and the budget is the recipe's too.
    #[test]
    fn the_payload_names_no_input_and_no_budget() {
        let encoded = String::from_utf8_lossy(&payload().encode_to_vec()).into_owned();
        assert!(!encoded.contains("sha256:"));
        assert!(!encoded.contains("600"));
    }
}
