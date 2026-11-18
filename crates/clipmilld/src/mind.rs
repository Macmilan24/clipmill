//! The editorial mind's model-free stages (architecture §4.3, §4.6).
//!
//! Two builtins frame the model's work. `mind-script` turns the Timeline Index
//! into the annotated script, cut into the windows a small model reads one at
//! a time, so the memory and markup passes read a published artifact rather
//! than each rebuilding it. `mind-cut` reads what those passes wrote and runs
//! the cutter, publishing plans of parts that already pass the plan checks
//! against the recording's words and the length asked for.

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_contracts::proto::ipc::v1::EditorialStagePayloadV1;
use clipmill_contracts::schemas::{
    mind_memory::MindMemory, mind_script::MindScript, mind_unit_scores::MindUnitScores,
};
use clipmill_core::{ArtifactId, Sha256Digest};
use clipmill_mind::cutter::{self, Material, Scout};
use clipmill_mind::script::{self, Chapter, Cue, Level, Sentence, Source};
use clipmill_timeline::TimelineIndex;
use prost::Message;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::{
    artifacts::ArtifactHandle,
    inputs::{self, Wanted},
    jobs::{LeasedTask, TaskExecutionError},
    media::{self, ProgressSlot},
};

pub(crate) const KIND_SCRIPT: &str = "mind-script";
pub(crate) const KIND_CUT: &str = "mind-cut";
pub(crate) const SCRIPT_IMPLEMENTATION: &str = "clipmill-mind-script@1.0.0";
pub(crate) const CUT_IMPLEMENTATION: &str = "clipmill-mind-cut@1.2.0";
/// Key version every mind stage payload carries.
pub(crate) const MIND_STAGE_KEY_VERSION: &str = "clipmill.mind-stage.v1";
/// About how much script one call reads: room for a 4B model's prompt, the
/// window and its answer in a modest context.
pub(crate) const WINDOW_TOKENS: u64 = 3_000;
/// Topic keywords shown on a chapter line.
const CHAPTER_KEYWORDS: usize = 3;
/// The length asked for when a request names none, as the editorial stages
/// default it.
const DEFAULT_LENGTH: (u64, u64) = (20 * 90_000, 90 * 90_000);

fn decode(task: &LeasedTask, stage: &str) -> Result<EditorialStagePayloadV1, TaskExecutionError> {
    let payload = EditorialStagePayloadV1::decode(task.payload.as_slice())
        .map_err(|_| TaskExecutionError::deterministic("task payload is not a mind payload"))?;
    if payload.key_version != MIND_STAGE_KEY_VERSION || payload.stage != stage {
        return Err(TaskExecutionError::deterministic(format!(
            "task payload does not describe {stage}"
        )));
    }
    Ok(payload)
}

fn fingerprint(value: &str) -> Result<Sha256Digest, TaskExecutionError> {
    value
        .strip_prefix("sha256:")
        .unwrap_or_default()
        .parse()
        .map_err(|_| TaskExecutionError::deterministic("an input carries no source fingerprint"))
}

/// Key, stage and publish one JSON document, or serve it from the cache.
async fn publish(
    artifacts: &ArtifactHandle,
    spec: RecipeSpec,
    file: &str,
    build: impl FnOnce() -> Result<Value, TaskExecutionError>,
) -> Result<ArtifactId, TaskExecutionError> {
    let recipe = ArtifactRecipe::try_from_spec(spec)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let staging = match media::prepare_or_hit(artifacts, recipe).await? {
        media::Prepared::Hit(artifact_id) => return Ok(artifact_id),
        media::Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let path = media::artifact_path(file)?;
    let result = async {
        let value = build()?;
        media::write_canonical_json(&staging, &path, &value)?;
        media::commit_staging(artifacts, staging_id.clone(), vec![path]).await
    }
    .await;
    if result.is_err() {
        media::abandon_staging(artifacts, staging_id).await;
    }
    result
}

fn spec(
    kind: &str,
    stage: &str,
    implementation: &str,
    source: Sha256Digest,
    inputs: Vec<ArtifactId>,
    config: Map<String, Value>,
) -> RecipeSpec {
    RecipeSpec {
        kind: kind.to_owned(),
        source_fingerprint: source,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: stage.to_owned(),
            implementation: implementation.to_owned(),
            model_digest: None,
        },
        inputs,
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: format!("clipmill.{kind}"),
    }
}

/// What the mind's stages read from a published index.
struct Indexed {
    source: Source,
    /// Every word as `(start, end)`: the edges a cut may use.
    words: Vec<(u64, u64)>,
    source_fingerprint: String,
}

/// The recording as the script module reads it, from the published index.
fn read_index(index: &TimelineIndex) -> Result<Indexed, String> {
    let error = |error: clipmill_timeline::TimelineError| error.to_string();
    let source_fingerprint = index
        .meta("source_fingerprint")
        .map_err(error)?
        .ok_or("the index names no recording")?;
    let duration_ticks = index.end_ticks().map_err(error)?;
    let sentences = index
        .sentences_between(0, u64::MAX)
        .map_err(error)?
        .into_iter()
        .map(|row| Sentence {
            index: row.index,
            start_ticks: row.start_ticks,
            end_ticks: row.end_ticks,
            speaker: row.speaker,
            text: row.text,
        })
        .collect();
    // Pauses are read from the gaps between sentences; everything else heard
    // or seen is a cue.
    let cues = index
        .events_between(0, u64::MAX, &[])
        .map_err(error)?
        .into_iter()
        .filter(|event| event.kind != "pause")
        .map(|event| Cue {
            kind: event.kind,
            start_ticks: event.start_ticks,
            end_ticks: event.end_ticks,
            label: event.label,
        })
        .collect();
    let shot_starts = index
        .shots_between(0, u64::MAX)
        .map_err(error)?
        .into_iter()
        .map(|shot| shot.start_ticks)
        .filter(|start| *start > 0)
        .collect();
    let chapters = index
        .topics()
        .map_err(error)?
        .into_iter()
        .map(|(start_ticks, mut keywords)| {
            keywords.truncate(CHAPTER_KEYWORDS);
            Chapter {
                start_ticks,
                keywords,
            }
        })
        .collect();
    let words = index.word_times().map_err(error)?;
    Ok(Indexed {
        source: Source {
            duration_ticks,
            sentences,
            cues,
            cuts: shot_starts,
            chapters,
        },
        words,
        source_fingerprint,
    })
}

async fn open_index(resolved: &inputs::Resolved) -> Result<Indexed, TaskExecutionError> {
    let path = resolved.verified_file("timeline.index.v1", "timeline.sqlite")?;
    tokio::task::spawn_blocking(move || {
        let index = TimelineIndex::open(&path).map_err(|error| error.to_string())?;
        read_index(&index)
    })
    .await
    .map_err(|error| TaskExecutionError::transient(error.to_string()))?
    .map_err(TaskExecutionError::deterministic)
}

/// The annotated script document: windows of the compact script and the
/// sentences behind them.
pub(crate) fn script_document(source: &Source, source_fingerprint: &str) -> Value {
    let level = Level::Compact;
    let whole = script::render(source, level, None);
    let letters = script::speaker_letters(source);
    let units: Vec<Value> = source
        .sentences
        .iter()
        .map(|sentence| {
            let mut unit = json!({
                "index": sentence.index,
                "start_ticks": sentence.start_ticks,
                "end_ticks": sentence.end_ticks,
                "text": sentence.text,
            });
            if let Some(letter) = sentence.speaker.as_deref().and_then(|s| letters.get(s)) {
                unit["speaker"] = json!(letter);
            }
            unit
        })
        .collect();
    let windows: Vec<Value> = script::windows(source, level, WINDOW_TOKENS)
        .into_iter()
        .map(|window| {
            json!({
                "text": window.script.text,
                "units": window.units,
                "tokens": window.script.tokens,
            })
        })
        .collect();
    json!({
        "schema_version": "clipmill.mind.script.v1",
        "source_fingerprint": source_fingerprint,
        "level": level.as_str(),
        "digest": format!("sha256:{}", hex::encode(Sha256::digest(whole.text.as_bytes()))),
        "duration_ticks": source.duration_ticks,
        "units": units,
        "windows": windows,
    })
}

/// Write the annotated script a model reads, from the Timeline Index.
pub(crate) async fn execute_script_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 2);
    decode(task, KIND_SCRIPT)?;
    let resolved =
        inputs::resolve(artifacts, task, &[Wanted::required("timeline.index.v1")]).await?;
    let Indexed {
        source,
        source_fingerprint,
        ..
    } = open_index(&resolved).await?;
    progress.set("stages", 1, 2);
    let mut config = Map::new();
    config.insert("level".to_owned(), json!(Level::Compact.as_str()));
    config.insert("window_tokens".to_owned(), json!(WINDOW_TOKENS));
    config.insert("chapter_keywords".to_owned(), json!(CHAPTER_KEYWORDS));
    let published = publish(
        artifacts,
        spec(
            "mind.script.v1",
            KIND_SCRIPT,
            SCRIPT_IMPLEMENTATION,
            fingerprint(&source_fingerprint)?,
            resolved.addresses(),
            config,
        ),
        "script.json",
        || Ok(script_document(&source, &source_fingerprint)),
    )
    .await;
    progress.set("stages", 2, 2);
    published
}

const fn scout_name(scout: Scout) -> &'static str {
    match scout {
        Scout::Event => "event",
        Scout::Link => "link",
        Scout::ColdOpen => "cold_open",
        Scout::Theme => "theme",
        Scout::Span => "span",
        Scout::Brief => "brief",
        Scout::Reaction => "reaction",
    }
}

/// The script's sentences back as a source the cutter reads, with the
/// script's letters for voices.
fn script_source(script: &MindScript) -> Source {
    Source {
        duration_ticks: script.duration_ticks,
        sentences: script
            .units
            .iter()
            .map(|unit| Sentence {
                index: unit.index,
                start_ticks: unit.start_ticks,
                end_ticks: unit.end_ticks,
                speaker: unit
                    .speaker
                    .as_ref()
                    .map(|letter| letter.as_str().to_owned()),
                text: unit.text.clone(),
            })
            .collect(),
        ..Source::default()
    }
}

/// Plans of parts from the memory and the markup.
#[allow(
    clippy::too_many_lines,
    reason = "read, check, key, cut, publish — in that order, as the other builtins"
)]
pub(crate) async fn execute_cut_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 3);
    let payload = decode(task, KIND_CUT)?;
    let resolved = inputs::resolve(
        artifacts,
        task,
        &[
            Wanted::required("timeline.index.v1"),
            Wanted::required("mind.script.v1"),
            Wanted::optional("mind.memory.v1"),
            Wanted::optional("mind.unit_scores.v1"),
        ],
    )
    .await?;
    let script: MindScript = resolved.read("mind.script.v1", "script.json")?;
    let memory: Option<MindMemory> = resolved.read_optional("mind.memory.v1", "memory.json")?;
    let scores: Option<MindUnitScores> =
        resolved.read_optional("mind.unit_scores.v1", "unit_scores.json")?;
    let Indexed {
        words,
        source_fingerprint,
        source: indexed_source,
    } = open_index(&resolved).await?;
    if script.source_fingerprint.as_str() != source_fingerprint
        || memory
            .as_ref()
            .is_some_and(|m| m.source_fingerprint.as_str() != source_fingerprint)
        || scores
            .as_ref()
            .is_some_and(|s| s.source_fingerprint.as_str() != source_fingerprint)
    {
        return Err(TaskExecutionError::deterministic(
            "the cutter's inputs describe different recordings",
        ));
    }
    progress.set("stages", 1, 3);

    let length = payload
        .duration
        .filter(|range| range.min_ticks > 0 && range.max_ticks >= range.min_ticks)
        .map_or(DEFAULT_LENGTH, |range| (range.min_ticks, range.max_ticks));
    let mut source = script_source(&script);
    source.cues = indexed_source.cues;
    if let Some(memory) = &memory {
        let problems = clipmill_mind::memory::check(memory, &source);
        if !problems.is_empty() {
            tracing::info!(
                problems = problems.len(),
                "the recording memory has items the cutter will not find a place for"
            );
        }
    }
    let mut inputs_document = json!({
        "timeline_artifact_id": resolved.address("timeline.index.v1")?,
        "script_artifact_id": resolved.address("mind.script.v1")?,
    });
    if let Some(address) = resolved.optional_address("mind.memory.v1") {
        inputs_document["memory_artifact_id"] = json!(address);
    }
    if let Some(address) = resolved.optional_address("mind.unit_scores.v1") {
        inputs_document["scores_artifact_id"] = json!(address);
    }
    let mut config = Map::new();
    config.insert("min_ticks".to_owned(), json!(length.0));
    config.insert("max_ticks".to_owned(), json!(length.1));
    config.insert("algorithm".to_owned(), json!("clipmill.mind.cutter.v3"));
    config.insert(
        "editorial_prompt".to_owned(),
        json!(payload.editorial_prompt),
    );
    progress.set("stages", 2, 3);
    let published = publish(
        artifacts,
        spec(
            "mind.candidates.v1",
            KIND_CUT,
            CUT_IMPLEMENTATION,
            fingerprint(&source_fingerprint)?,
            resolved.addresses(),
            config,
        ),
        "candidates.json",
        || {
            let found = cutter::candidates_for_brief(
                &Material {
                    source_fingerprint: &source_fingerprint,
                    source: &source,
                    words: &words,
                    scores: scores.as_ref(),
                    memory: memory.as_ref(),
                    length,
                },
                &payload.editorial_prompt,
            );
            let candidates = found
                .into_iter()
                .map(|candidate| {
                    Ok(json!({
                        "scout": scout_name(candidate.scout),
                        "score": candidate.score,
                        "alternatives": candidate.alternatives,
                        "plan": serde_json::to_value(&candidate.plan).map_err(|error| {
                            TaskExecutionError::deterministic(error.to_string())
                        })?,
                    }))
                })
                .collect::<Result<Vec<Value>, TaskExecutionError>>()?;
            Ok(json!({
                "schema_version": "clipmill.mind.candidates.v1",
                "source_fingerprint": source_fingerprint,
                "length": { "min_ticks": length.0, "max_ticks": length.1 },
                "inputs": inputs_document,
                "candidates": candidates,
            }))
        },
    )
    .await;
    progress.set("stages", 3, 3);
    published
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use clipmill_contracts::schemas::mind_script::MindScript;
    use clipmill_mind::script::{Sentence, Source};

    use super::script_document;

    #[test]
    fn the_script_document_meets_its_contract_and_owns_every_sentence_once() {
        let source = Source {
            duration_ticks: 600 * 90_000,
            sentences: (0..80)
                .map(|i| Sentence {
                    index: i,
                    start_ticks: i * 7 * 90_000,
                    end_ticks: i * 7 * 90_000 + 6 * 90_000,
                    speaker: Some(if i % 3 == 0 { "spk_1" } else { "spk_0" }.to_owned()),
                    text: format!("Sentence {i} says something worth hearing about the deal."),
                })
                .collect(),
            ..Source::default()
        };
        let value = script_document(&source, &format!("sha256:{}", "a".repeat(64)));
        let script: MindScript = serde_json::from_value(value).expect("meets its contract");
        assert_eq!(script.units.len(), 80);
        assert_eq!(script.units[0].speaker.as_ref().unwrap().as_str(), "A");
        let owned: Vec<u64> = script
            .windows
            .iter()
            .flat_map(|w| w.units.clone())
            .collect();
        assert_eq!(owned, (0..80).collect::<Vec<_>>());
    }
}
