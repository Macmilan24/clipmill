//! The builtin that turns aligned words into captions.
//!
//! Fifth of the model-free builtins, and the same shape as the four before it:
//! it reads published documents and writes another. Everything it decides lives
//! in `clipmill-captions`, which does no I/O — this module reads the inputs,
//! keys the result, and publishes it.
//!
//! Two of its three inputs are optional, and that is a decision rather than
//! leniency. Without the evidence index there are no sentence boundaries to
//! prefer breaking at and no salient terms emphasis may come from; without shot
//! detection there is nothing known about where the picture changes. A caption
//! set built without either is a weaker one, so both are recorded in the key
//! and in the document — the alternative is two different readings of a
//! recording sharing one content address.

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_captions::{DeriveRequest, Inputs, Span, Weights};
use clipmill_contracts::proto::ipc::v1::CaptionsStagePayloadV1;
use clipmill_contracts::schemas::{
    evidence_shots::EvidenceShots, index_transcript::IndexTranscript,
    speech_transcript::SpeechTranscript,
};
use clipmill_core::{ArtifactId, Sha256Digest};
use prost::Message;
use serde_json::{Map, json};

use crate::{
    artifacts::ArtifactHandle,
    inputs::{self, Wanted},
    jobs::{CAPTIONS_STAGE_KEY_VERSION, LeasedTask, TaskExecutionError},
    media::{self, ProgressSlot},
};

/// The task kind this module executes.
pub(crate) const KIND_CAPTIONS: &str = "derive-captions";
pub(crate) const IMPLEMENTATION: &str = "clipmill-captions-dp@1.0.0";
const OUTPUT_FILE: &str = "captions.json";

/// Read the words, segment them twice, and publish the result.
pub(crate) async fn execute_captions_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 3);
    let payload = CaptionsStagePayloadV1::decode(task.payload.as_slice())
        .map_err(|_| TaskExecutionError::deterministic("task payload is not a captions payload"))?;
    if payload.key_version != CAPTIONS_STAGE_KEY_VERSION || payload.stage != KIND_CAPTIONS {
        return Err(TaskExecutionError::deterministic(
            "task payload does not describe caption derivation",
        ));
    }

    let resolved = inputs::resolve(
        artifacts,
        task,
        &[
            Wanted::required("speech.transcript.v1"),
            Wanted::optional("index.transcript.v1"),
            Wanted::optional("evidence.shots.v1"),
        ],
    )
    .await?;
    let transcript: SpeechTranscript = resolved.read("speech.transcript.v1", "transcript.json")?;
    let index: Option<IndexTranscript> =
        resolved.read_optional("index.transcript.v1", "index.json")?;
    let shots: Option<EvidenceShots> = resolved.read_optional("evidence.shots.v1", "shots.json")?;
    progress.set("stages", 1, 3);

    let transcript_address = resolved.address("speech.transcript.v1")?;
    let index_address = resolved.optional_address("index.transcript.v1");
    let shots_address = resolved.optional_address("evidence.shots.v1");

    let mut request = DeriveRequest::new(IMPLEMENTATION);
    request.span = span_of(&payload);
    let document = clipmill_captions::derive(
        &transcript,
        index.as_ref(),
        shots.as_ref(),
        Inputs {
            transcript_artifact_id: &transcript_address,
            index_artifact_id: index_address.as_deref(),
            shots_artifact_id: shots_address.as_deref(),
        },
        &request,
    )
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    progress.set("stages", 2, 3);

    let fingerprint: Sha256Digest = transcript
        .source_fingerprint
        .strip_prefix("sha256:")
        .unwrap_or_default()
        .parse()
        .map_err(|_| TaskExecutionError::deterministic("the transcript carries no fingerprint"))?;
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "captions.cues.v1".to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: KIND_CAPTIONS.to_owned(),
            implementation: IMPLEMENTATION.to_owned(),
            model_digest: None,
        },
        inputs: resolved.addresses(),
        policy: NetworkPolicy::LocalLock,
        config: config_of(&payload, &request.weights),
        semantic_version: "clipmill.captions.cues.v1".to_owned(),
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

/// The window the payload asked for, or the transcript's own coverage.
fn span_of(payload: &CaptionsStagePayloadV1) -> Option<Span> {
    if payload.span_end_ticks == 0 || payload.span_end_ticks <= payload.span_start_ticks {
        return None;
    }
    Some(Span {
        start_ticks: i64::try_from(payload.span_start_ticks).unwrap_or(0),
        end_ticks: i64::try_from(payload.span_end_ticks).unwrap_or(i64::MAX),
    })
}

/// Everything the segmentation was told, by name.
///
/// The weights are here because re-tuning any one of them is a different
/// reading of the same words, and the window is here because captioning a clip
/// is not a slice of captioning the recording — a cue may not cross the edge of
/// its window, so the edges change where the breaks fall.
fn config_of(
    payload: &CaptionsStagePayloadV1,
    weights: &Weights,
) -> Map<String, serde_json::Value> {
    let mut config = Map::new();
    config.insert("algorithm".to_owned(), json!("clipmill.captions.cues.v1"));
    config.insert(
        "span_start_ticks".to_owned(),
        json!(payload.span_start_ticks),
    );
    config.insert("span_end_ticks".to_owned(), json!(payload.span_end_ticks));
    config.insert("reading_rate".to_owned(), json!(weights.reading_rate));
    config.insert("line_balance".to_owned(), json!(weights.line_balance));
    config.insert("orphan".to_owned(), json!(weights.orphan));
    config.insert("break_quality".to_owned(), json!(weights.break_quality));
    config.insert("short_cue".to_owned(), json!(weights.short_cue));
    config.insert(
        "filler_lexicon".to_owned(),
        json!(clipmill_captions::FILLER_LEXICON),
    );
    config
}
