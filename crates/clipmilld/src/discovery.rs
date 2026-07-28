//! The builtin that searches a recording for clips worth considering.
//!
//! Third of the model-free builtins, and the same shape as the two before it:
//! it loads nothing, reads published documents, and writes one more. Everything
//! it decides lives in `clipmill-discovery`, which does no I/O — this module
//! reads the inputs, keys the result, and publishes it.
//!
//! Its three documents arrive on the lease — declared by the plan when this runs
//! standalone, delivered by a dependency inside the analyze DAG — and each is
//! matched against the artifact kind its own manifest declares. A plan pointing
//! at a transcript where an index belongs is refused rather than searched, and
//! the key covers the addresses whichever route found them.

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
    inputs::{self, Wanted},
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
    let resolved = inputs::resolve(
        artifacts,
        task,
        &[
            Wanted::required("index.transcript.v1"),
            Wanted::required("speech.transcript.v1"),
            Wanted::optional("media.loudness_envelope.v1"),
        ],
    )
    .await?;
    let index: IndexTranscript = resolved.read("index.transcript.v1", "index.json")?;
    let transcript: SpeechTranscript = resolved.read("speech.transcript.v1", "transcript.json")?;
    let loudness: Option<MediaLoudnessEnvelope> =
        resolved.read_optional("media.loudness_envelope.v1", "loudness.json")?;
    let index_id = resolved.address("index.transcript.v1")?;
    let transcript_id = resolved.address("speech.transcript.v1")?;
    let loudness_id = resolved.optional_address("media.loudness_envelope.v1");
    progress.set("stages", 1, 3);

    let parameters = parameters_of(&payload);
    let document = clipmill_discovery::discover(
        &index,
        &transcript,
        loudness.as_ref(),
        Inputs {
            index: &index_id,
            transcript: &transcript_id,
            loudness: loudness_id.as_deref(),
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
        inputs: resolved.addresses(),
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
#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_contracts::proto::ipc::v1::{ClipDurationV1, DiscoverStagePayloadV1};

    use super::{KIND_DISCOVER, parameters_of};
    use crate::jobs::DISCOVER_STAGE_KEY_VERSION;

    fn payload() -> DiscoverStagePayloadV1 {
        DiscoverStagePayloadV1 {
            key_version: DISCOVER_STAGE_KEY_VERSION.to_owned(),
            stage: KIND_DISCOVER.to_owned(),
            duration: None,
            exploration_floor: 0,
        }
    }

    /// Zero means "no opinion" on the wire, so a caller who does not care about
    /// clip length does not have to know what the daemon would pick.
    #[test]
    fn an_unset_length_takes_the_daemon_default() {
        assert_eq!(
            parameters_of(&payload()),
            clipmill_discovery::Parameters::DEFAULT
        );
        let zeroed = DiscoverStagePayloadV1 {
            duration: Some(ClipDurationV1 {
                min_ticks: 0,
                max_ticks: 0,
            }),
            ..payload()
        };
        assert_eq!(
            parameters_of(&zeroed),
            clipmill_discovery::Parameters::DEFAULT
        );
    }

    #[test]
    fn a_stated_length_is_honoured_exactly() {
        let asked = DiscoverStagePayloadV1 {
            duration: Some(ClipDurationV1 {
                min_ticks: 30 * 90_000,
                max_ticks: 60 * 90_000,
            }),
            exploration_floor: 5,
            ..payload()
        };
        let parameters = parameters_of(&asked);
        assert_eq!(parameters.min_ticks, 30 * 90_000);
        assert_eq!(parameters.max_ticks, 60 * 90_000);
        assert_eq!(parameters.exploration_floor, 5);
    }

    /// One half stated and the other left alone is a real request: a caller who
    /// wants clips no shorter than thirty seconds should not have to name a
    /// ceiling as well.
    #[test]
    fn half_a_length_leaves_the_other_half_at_the_default() {
        let asked = DiscoverStagePayloadV1 {
            duration: Some(ClipDurationV1 {
                min_ticks: 30 * 90_000,
                max_ticks: 0,
            }),
            ..payload()
        };
        let parameters = parameters_of(&asked);
        assert_eq!(parameters.min_ticks, 30 * 90_000);
        assert_eq!(
            parameters.max_ticks,
            clipmill_discovery::Parameters::DEFAULT.max_ticks
        );
    }
}
