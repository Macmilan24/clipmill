//! Build and publish a recording's Timeline Index (decision R74).
//!
//! A model-free derivation in the daemon, like the transcript index it reads:
//! the transcript's words, the index's sentences, utterances and topics, and,
//! when the plan ran them, speaker turns and shots, written into one SQLite
//! file by `clipmill-timeline` and published as `timeline.index.v1`.

use std::collections::BTreeMap;

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_contracts::proto::ipc::v1::IndexStagePayloadV1;
use clipmill_contracts::schemas::{
    evidence_shots::EvidenceShots, index_transcript::IndexTranscript,
    speech_speakers::SpeechSpeakers, speech_transcript::SpeechTranscript,
};
use clipmill_core::{ArtifactId, Sha256Digest};
use clipmill_timeline::{Recording, Sentence, Shot, Topic, Turn, Utterance, Word};
use prost::Message;
use serde_json::{Map, json};

use crate::{
    artifacts::ArtifactHandle,
    inputs::{self, Wanted},
    jobs::{LeasedTask, TaskExecutionError},
    media::{self, ProgressSlot},
};

/// The task kind this module executes.
pub(crate) const KIND_TIMELINE: &str = "index-timeline";
pub(crate) const IMPLEMENTATION: &str = "clipmill-timeline@1.0.0";
/// Key version the stage payload carries.
pub(crate) const TIMELINE_STAGE_KEY_VERSION: &str = "clipmill.timeline-stage.v1";
const OUTPUT_FILE: &str = "timeline.sqlite";

/// Read the recording's evidence and publish its index.
#[allow(
    clippy::too_many_lines,
    reason = "read, check, key, build, publish — the order is the point, as in evidence.rs"
)]
pub(crate) async fn execute_timeline_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 3);
    let payload = IndexStagePayloadV1::decode(task.payload.as_slice()).map_err(|_| {
        TaskExecutionError::deterministic("task payload is not a timeline index payload")
    })?;
    if payload.key_version != TIMELINE_STAGE_KEY_VERSION || payload.stage != KIND_TIMELINE {
        return Err(TaskExecutionError::deterministic(
            "task payload does not describe the timeline index",
        ));
    }
    let resolved = inputs::resolve(
        artifacts,
        task,
        &[
            Wanted::required("speech.transcript.v1"),
            Wanted::required("index.transcript.v1"),
            Wanted::optional("speech.speakers.v1"),
            Wanted::optional("evidence.shots.v1"),
        ],
    )
    .await?;
    let transcript: SpeechTranscript = resolved.read("speech.transcript.v1", "transcript.json")?;
    let index: IndexTranscript = resolved.read("index.transcript.v1", "index.json")?;
    let speakers: Option<SpeechSpeakers> =
        resolved.read_optional("speech.speakers.v1", "speakers.json")?;
    let shots: Option<EvidenceShots> = resolved.read_optional("evidence.shots.v1", "shots.json")?;
    if transcript.source_fingerprint.as_str() != index.source_fingerprint.as_str()
        || speakers.as_ref().is_some_and(|speakers| {
            speakers.source_fingerprint.as_str() != index.source_fingerprint.as_str()
        })
        || shots.as_ref().is_some_and(|shots| {
            shots.source_fingerprint.as_str() != index.source_fingerprint.as_str()
        })
    {
        return Err(TaskExecutionError::deterministic(
            "the timeline's inputs describe different recordings",
        ));
    }
    progress.set("stages", 1, 3);

    let recording = recording(&transcript, &index, speakers.as_ref(), shots.as_ref());
    let fingerprint: Sha256Digest = index
        .source_fingerprint
        .strip_prefix("sha256:")
        .unwrap_or_default()
        .parse()
        .map_err(|_| TaskExecutionError::deterministic("the index carries no fingerprint"))?;
    let mut config = Map::new();
    config.insert(
        "schema".to_owned(),
        json!(clipmill_timeline::SCHEMA_VERSION),
    );
    config.insert(
        "pause_event_ticks".to_owned(),
        json!(clipmill_timeline::PAUSE_EVENT_TICKS),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "timeline.index.v1".to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: KIND_TIMELINE.to_owned(),
            implementation: IMPLEMENTATION.to_owned(),
            model_digest: None,
        },
        inputs: resolved.addresses(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: clipmill_timeline::SCHEMA_VERSION.to_owned(),
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
    let result = async {
        // The store creates the file with its private permissions; SQLite
        // then fills it in place, writing nothing beside it.
        drop(
            staging
                .create_file(&path)
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?,
        );
        let disk = staging.path().join(path.as_path());
        tokio::task::spawn_blocking(move || clipmill_timeline::build(&disk, &recording))
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?
            .map_err(|error| match error {
                clipmill_timeline::TimelineError::Invalid(detail) => {
                    TaskExecutionError::deterministic(detail)
                }
                other => TaskExecutionError::transient(other.to_string()),
            })?;
        progress.set("stages", 2, 3);
        media::commit_staging(artifacts, staging_id.clone(), vec![path]).await
    }
    .await;
    if result.is_err() {
        media::abandon_staging(artifacts, staging_id).await;
    }
    progress.set("stages", 3, 3);
    result
}

/// The evidence, in the shape the index is built from.
fn recording(
    transcript: &SpeechTranscript,
    index: &IndexTranscript,
    speakers: Option<&SpeechSpeakers>,
    shots: Option<&EvidenceShots>,
) -> Recording {
    // Speaker ids become the labels people see ("S1", "S2"...), numbered in
    // order of first speech so the same recording always names them alike.
    let mut labels: BTreeMap<&str, String> = BTreeMap::new();
    let turns = speakers.map_or_else(Vec::new, |speakers| {
        let mut turns: Vec<_> = speakers.turns.iter().collect();
        turns.sort_by_key(|turn| (turn.start_ticks, turn.end_ticks.get()));
        turns
            .into_iter()
            .map(|turn| {
                let next = labels.len() + 1;
                let speaker = labels
                    .entry(turn.speaker_id.as_str())
                    .or_insert_with(|| format!("S{next}"))
                    .clone();
                Turn {
                    speaker,
                    start_ticks: turn.start_ticks,
                    end_ticks: turn.end_ticks.get(),
                }
            })
            .collect()
    });
    Recording {
        source_fingerprint: index.source_fingerprint.to_string(),
        language: index.language.to_string(),
        words: transcript
            .words
            .iter()
            .map(|word| Word {
                index: word.index,
                text: word.text.to_string(),
                start_ticks: word.start_ticks,
                end_ticks: word.end_ticks,
            })
            .collect(),
        sentences: index
            .sentences
            .iter()
            .map(|sentence| Sentence {
                index: sentence.index,
                text: sentence.text.to_string(),
                start_ticks: sentence.start_ticks,
                end_ticks: sentence.end_ticks,
                first_word_index: sentence.first_word_index,
                word_count: sentence.word_count.get(),
                utterance_index: sentence.utterance_index,
            })
            .collect(),
        utterances: index
            .utterances
            .iter()
            .map(|utterance| Utterance {
                index: utterance.index,
                start_ticks: utterance.start_ticks,
                end_ticks: utterance.end_ticks,
                pause_before_ticks: utterance.pause_before_ticks,
                pause_after_ticks: utterance.pause_after_ticks,
            })
            .collect(),
        topics: index
            .topics
            .iter()
            .map(|topic| Topic {
                index: topic.index,
                start_ticks: topic.start_ticks,
                end_ticks: topic.end_ticks,
                first_sentence_index: topic.first_sentence_index,
                sentence_count: topic.sentence_count.get(),
                keywords: topic
                    .keywords
                    .iter()
                    .map(|keyword| keyword.term.to_string())
                    .collect(),
            })
            .collect(),
        turns,
        shots: shots.map_or_else(Vec::new, |shots| {
            shots
                .shots
                .iter()
                .map(|shot| Shot {
                    start_ticks: shot.start_ticks,
                    end_ticks: shot.end_ticks,
                })
                .collect()
        }),
        events: Vec::new(),
    }
}
