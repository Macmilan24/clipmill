//! The builtin that decides which clips are worth showing.
//!
//! Fourth of the model-free builtins and the last stage of the analysis: it
//! reads three published documents and writes the one a results board renders.
//! Everything it decides lives in `clipmill-discovery::ranking`, which does no
//! I/O — this module reads the inputs, keys the result, and publishes it.
//!
//! Its three documents arrive on the lease — declared by the plan when this runs
//! standalone, delivered by a dependency inside the analyze DAG. The artifact
//! key is computed from the addresses either way, so the same three documents
//! ranked twice through different routes hit the same cache entry.

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_contracts::proto::ipc::v1::RankStagePayloadV1;
use clipmill_contracts::schemas::{
    discovery_candidates::DiscoveryCandidates, index_transcript::IndexTranscript,
    speech_transcript::SpeechTranscript,
};
use clipmill_core::{ArtifactId, Sha256Digest};
use clipmill_discovery::{RankingInputs, Request};
use prost::Message;
use serde_json::{Map, json};

use crate::{
    artifacts::ArtifactHandle,
    inputs::{self, Wanted},
    jobs::{LeasedTask, RANK_STAGE_KEY_VERSION, TaskExecutionError},
    media::{self, ProgressSlot},
};

/// The task kind this module executes.
pub(crate) const KIND_RANK: &str = "rank-candidates";
pub(crate) const IMPLEMENTATION: &str = "clipmill-ranking-baseline@1.0.0";
const OUTPUT_FILE: &str = "ranking.json";

/// Score the cohort, cut every clip, select the set, and publish it.
#[allow(
    clippy::too_many_lines,
    reason = "read, rank, key, publish — the order is the point"
)]
pub(crate) async fn execute_rank_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 3);
    let payload = RankStagePayloadV1::decode(task.payload.as_slice())
        .map_err(|_| TaskExecutionError::deterministic("task payload is not a ranking payload"))?;
    if payload.key_version != RANK_STAGE_KEY_VERSION || payload.stage != KIND_RANK {
        return Err(TaskExecutionError::deterministic(
            "task payload does not describe ranking",
        ));
    }

    let resolved = inputs::resolve(
        artifacts,
        task,
        &[
            Wanted::required("discovery.candidates.v1"),
            Wanted::required("index.transcript.v1"),
            Wanted::required("speech.transcript.v1"),
        ],
    )
    .await?;
    let candidates: DiscoveryCandidates =
        resolved.read("discovery.candidates.v1", "candidates.json")?;
    let index: IndexTranscript = resolved.read("index.transcript.v1", "index.json")?;
    let transcript: SpeechTranscript = resolved.read("speech.transcript.v1", "transcript.json")?;
    progress.set("stages", 1, 3);

    let request = request_of(&payload);
    let document = clipmill_discovery::rank(
        &candidates,
        &index,
        &transcript,
        RankingInputs {
            candidates: &resolved.address("discovery.candidates.v1")?,
            index: &resolved.address("index.transcript.v1")?,
            transcript: &resolved.address("speech.transcript.v1")?,
        },
        request,
        IMPLEMENTATION,
    )
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    // Checked before publication rather than trusted: a set whose selected ids
    // are not in its own cohort would be given a content address by a store
    // that has no way to notice.
    if !clipmill_discovery::ranking::is_well_formed(&document) {
        return Err(TaskExecutionError::deterministic(
            "the ranked set is not internally consistent",
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
    config.insert("algorithm".to_owned(), json!("clipmill.ranking.set.v1"));
    // Everything the ranking was told, by name. Asking for a different number
    // of clips, or a different diversity trade-off, is a different answer
    // rather than a filter over this one — and re-tuning any of the three
    // rubrics invalidates exactly the sets that rubric produced.
    config.insert("count".to_owned(), json!(request.count));
    config.insert("diversity_milli".to_owned(), json!(payload.diversity_milli));
    config.insert(
        "scorer".to_owned(),
        json!(clipmill_discovery::SCORER_RUBRIC),
    );
    config.insert(
        "boundary".to_owned(),
        json!(document.rubric.boundary.as_str()),
    );
    config.insert(
        "selector".to_owned(),
        json!(clipmill_discovery::ranking::SELECTOR_RUBRIC),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "ranking.set.v1".to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: KIND_RANK.to_owned(),
            implementation: IMPLEMENTATION.to_owned(),
            model_digest: None,
        },
        inputs: resolved.addresses(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.ranking.set.v1".to_owned(),
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
/// nothing. Diversity travels as thousandths so the value that reaches an
/// artifact key is an integer.
pub(crate) fn request_of(payload: &RankStagePayloadV1) -> Request {
    let default = Request::DEFAULT;
    Request {
        count: if payload.count > 0 {
            payload.count
        } else {
            default.count
        },
        diversity: if payload.diversity_milli > 0 {
            #[allow(
                clippy::cast_precision_loss,
                reason = "a value under a thousand, exact in f64"
            )]
            let value = payload.diversity_milli as f64 / 1000.0;
            value.min(1.0)
        } else {
            default.diversity
        },
    }
}

#[cfg(test)]
mod tests;
