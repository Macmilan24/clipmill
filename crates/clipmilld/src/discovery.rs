//! The builtin that searches a recording for clips worth considering.
//!
//! Third of the model-free builtins, and the same shape as the two before it:
//! it loads nothing, reads published documents, and writes one more. Everything
//! it decides lives in `clipmill-discovery`, which does no I/O — this module
//! reads the inputs, keys the result, and publishes it.
//!
//! Three documents arrive as content addresses in the payload rather than
//! through dependencies, because all three were published by earlier jobs and a
//! task's inputs are the outputs of the tasks it depends on. Each is checked
//! against the artifact kind its own manifest declares, so a payload pointing
//! at a transcript where an index belongs is refused rather than searched.

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_contracts::proto::ipc::v1::DiscoverStagePayloadV1;
use clipmill_contracts::schemas::{
    index_transcript::IndexTranscript, media_loudness_envelope::MediaLoudnessEnvelope,
    speech_transcript::SpeechTranscript,
};
use clipmill_core::{ArtifactId, Sha256Digest};
use clipmill_discovery::{Inputs, Parameters};
use prost::Message;
use serde_json::{Map, json};

use crate::{
    artifacts::ArtifactHandle,
    jobs::{DISCOVER_STAGE_KEY_VERSION, LeasedTask, TaskExecutionError},
    media::{self, ProgressSlot},
};

/// The task kind this module executes.
pub(crate) const KIND_DISCOVER: &str = "discover-candidates";
pub(crate) const IMPLEMENTATION: &str = "clipmill-discovery-mesh@1.0.0";
const OUTPUT_FILE: &str = "candidates.json";

/// Read the index, the transcript, and the loudness envelope if there is one,
/// then publish the candidate set over them.
#[allow(
    clippy::too_many_lines,
    reason = "read, search, key, publish — the order is the point"
)]
pub(crate) async fn execute_discover_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 3);
    let payload = DiscoverStagePayloadV1::decode(task.payload.as_slice()).map_err(|_| {
        TaskExecutionError::deterministic("task payload is not a discovery payload")
    })?;
    if payload.key_version != DISCOVER_STAGE_KEY_VERSION || payload.stage != KIND_DISCOVER {
        return Err(TaskExecutionError::deterministic(
            "task payload does not describe discovery",
        ));
    }
    if payload.index_artifact_id.is_empty() || payload.transcript_artifact_id.is_empty() {
        return Err(TaskExecutionError::deterministic(
            "discovery has no index or no transcript to read",
        ));
    }

    let index: IndexTranscript = read_input(
        artifacts,
        &payload.index_artifact_id,
        "index.transcript.v1",
        "index.json",
    )
    .await?;
    let transcript: SpeechTranscript = read_input(
        artifacts,
        &payload.transcript_artifact_id,
        "speech.transcript.v1",
        "transcript.json",
    )
    .await?;
    let loudness: Option<MediaLoudnessEnvelope> = if payload.loudness_artifact_id.is_empty() {
        None
    } else {
        Some(
            read_input(
                artifacts,
                &payload.loudness_artifact_id,
                "media.loudness_envelope.v1",
                "loudness.json",
            )
            .await?,
        )
    };
    progress.set("stages", 1, 3);

    let parameters = parameters_of(&payload);
    let document = clipmill_discovery::discover(
        &index,
        &transcript,
        loudness.as_ref(),
        Inputs {
            index: &payload.index_artifact_id,
            transcript: &payload.transcript_artifact_id,
            loudness: Some(payload.loudness_artifact_id.as_str()).filter(|id| !id.is_empty()),
        },
        parameters,
        IMPLEMENTATION,
    )
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    // Checked here rather than trusted, because a candidate set that lost a
    // candidate between search and publication would be a set whose clusters
    // point at nothing — and the store would happily give it an address.
    if !clipmill_discovery::is_well_formed(&document) {
        return Err(TaskExecutionError::deterministic(
            "the candidate set is not internally consistent",
        ));
    }
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
        json!("clipmill.discovery.candidates.v1"),
    );
    // Everything the search was told, by name. Asking for a different clip
    // length is a different search rather than a filter over this one, and the
    // proposer version invalidates exactly the sets an older mesh nominated.
    config.insert("min_ticks".to_owned(), json!(parameters.min_ticks));
    config.insert("max_ticks".to_owned(), json!(parameters.max_ticks));
    config.insert(
        "exploration_floor".to_owned(),
        json!(parameters.exploration_floor),
    );
    config.insert(
        "proposers".to_owned(),
        json!(clipmill_discovery::PROPOSER_VERSION),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "discovery.candidates.v1".to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: KIND_DISCOVER.to_owned(),
            implementation: IMPLEMENTATION.to_owned(),
            model_digest: None,
        },
        inputs: inputs_for(&payload)?,
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.discovery.candidates.v1".to_owned(),
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

/// What the payload asked for, with the daemon's defaults where it said
/// nothing. Zero means "no opinion" on the wire, so a caller with none does not
/// have to know what the defaults are.
pub(crate) fn parameters_of(payload: &DiscoverStagePayloadV1) -> Parameters {
    let default = Parameters::DEFAULT;
    let duration = payload.duration.as_ref();
    Parameters {
        min_ticks: duration
            .map(|range| range.min_ticks)
            .filter(|ticks| *ticks > 0)
            .unwrap_or(default.min_ticks),
        max_ticks: duration
            .map(|range| range.max_ticks)
            .filter(|ticks| *ticks > 0)
            .unwrap_or(default.max_ticks),
        exploration_floor: if payload.exploration_floor > 0 {
            payload.exploration_floor
        } else {
            default.exploration_floor
        },
    }
}

/// Open one named artifact and read one document out of it, checking the kind
/// its manifest declares rather than trusting the payload's word for it.
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
            "discovery was pointed at a {}, not a {expected_kind}",
            lease.kind()
        )));
    }
    media::read_artifact_document(&lease, file)
}

/// The addresses this stage read, in a stable order, for the artifact key.
pub(crate) fn inputs_for(
    payload: &DiscoverStagePayloadV1,
) -> Result<Vec<ArtifactId>, TaskExecutionError> {
    let mut inputs = Vec::new();
    for address in [
        payload.index_artifact_id.as_str(),
        payload.transcript_artifact_id.as_str(),
        payload.loudness_artifact_id.as_str(),
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
mod tests;
