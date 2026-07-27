//! Assembling the speech chain's three artifacts into one observation.
//!
//! Voice activity, recognition, and forced alignment are separate stages with
//! separate failure states, which is the point (book ch. 13): a bad alignment
//! degrades word timing without costing anyone the text. But every consumer
//! downstream — the evidence index, discovery, ranking, captions, the editor —
//! wants one document, and would otherwise each re-derive the fusion, each
//! slightly differently.
//!
//! Fusing them is mostly bookkeeping with one real decision in it: what to do
//! with a word the aligner would not place. Dropping it loses what was said.
//! Publishing it with invented timing is worse, because nothing downstream
//! could tell that timing apart from a measurement. So it is carried, its
//! interval is spread across the space its neighbours left, it is labelled
//! `interpolated`, and its span is declared invalid — which is what makes the
//! boundary optimizer refuse to cut inside it.
//!
//! This runs in the daemon rather than in a worker because it loads no model.
//! It is arithmetic over three JSON documents, and the two-lifecycle rule puts
//! model-free derivation where the artifacts already are.

use std::collections::BTreeMap;

use clipmill_artifacts::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};
use clipmill_core::{ArtifactId, Sha256Digest};
use serde_json::{Map, json};

use crate::{
    artifacts::ArtifactHandle,
    jobs::{LeasedTask, TaskExecutionError},
    media::{self, ProgressSlot},
};

use clipmill_evidence::confidence::distribution;

use clipmill_contracts::schemas::{
    speech_alignment::SpeechAlignment, speech_asr::SpeechAsr, speech_transcript as transcript,
    speech_vad::SpeechVad,
};

/// What the recognizer's contract fixes, and what a Rust reader has to check
/// itself: typify carries a JSON Schema `const` without validating it.
const TIMING_AUTHORITY: &str = "forced_alignment";

#[derive(Debug, thiserror::Error)]
pub(crate) enum AssemblyError {
    #[error("the recognition artifact claims {claimed} owns word timing, not forced alignment")]
    TimingAuthority { claimed: String },
    #[error("{stage} was never analyzed for this audio, so there is nothing to assemble")]
    NotAnalyzed { stage: &'static str },
    #[error("the three inputs describe different sources")]
    MismatchedSources,
    #[error("alignment placed a word in segment {segment}, which recognition never produced")]
    UnknownSegment { segment: u64 },
    #[error("{field} is not a well-formed content address")]
    MalformedAddress { field: &'static str },
}

/// One assembled transcript, ready to serialize.
pub(crate) struct Assembled {
    pub document: transcript::SpeechTranscript,
}

/// Fuse the three artifacts. Everything published here is traceable to one of
/// them; nothing is inferred that a consumer could not have inferred itself.
#[allow(
    clippy::too_many_lines,
    reason = "one pass over the segments; splitting it would hide the order"
)]
pub(crate) fn assemble(
    activity: &SpeechVad,
    recognized: &SpeechAsr,
    alignment: &SpeechAlignment,
    inputs: Inputs<'_>,
    assembler: &str,
) -> Result<Assembled, AssemblyError> {
    // The one check the generated type does not make. A recognizer that
    // declared its own token positions authoritative is the arrangement the
    // whole three-stage split exists to prevent, and it must not be possible
    // to assemble one into a transcript.
    let claimed = recognized
        .timing_authority
        .as_str()
        .unwrap_or_default()
        .to_owned();
    if claimed != TIMING_AUTHORITY {
        return Err(AssemblyError::TimingAuthority { claimed });
    }
    if !activity.coverage.analyzed {
        return Err(AssemblyError::NotAnalyzed {
            stage: "voice activity",
        });
    }
    if !recognized.coverage.analyzed {
        return Err(AssemblyError::NotAnalyzed {
            stage: "recognition",
        });
    }
    if !alignment.coverage.analyzed {
        return Err(AssemblyError::NotAnalyzed { stage: "alignment" });
    }
    if *activity.source_fingerprint != *recognized.source_fingerprint
        || *activity.source_fingerprint != *alignment.source_fingerprint
    {
        return Err(AssemblyError::MismatchedSources);
    }

    let mut placed: BTreeMap<u64, Vec<&clipmill_contracts::schemas::speech_alignment::Word>> =
        BTreeMap::new();
    for word in &alignment.words {
        placed.entry(word.segment_index).or_default().push(word);
    }
    let known = recognized
        .segments
        .iter()
        .map(|segment| segment.index)
        .collect::<std::collections::BTreeSet<_>>();
    if let Some(segment) = placed.keys().find(|index| !known.contains(index)) {
        return Err(AssemblyError::UnknownSegment { segment: *segment });
    }

    let mut words = Vec::new();
    let mut segments = Vec::new();
    let mut invalid = Vec::new();
    for segment in &recognized.segments {
        let first_word_index = u64::try_from(words.len()).unwrap_or(u64::MAX);
        let measured = placed.remove(&segment.index).unwrap_or_default();
        let spread = spread_words(segment, alignment, &measured);
        let mut ordered = Vec::new();
        for word in measured {
            ordered.push((
                word.start_ticks,
                word.end_ticks,
                (*word.text).clone(),
                transcript::WordTiming::Aligned,
                word.confidence.p50,
                word.confidence.p10,
            ));
        }
        ordered.extend(spread);
        ordered.sort_by_key(|entry| (entry.0, entry.1));

        for (start, end, text, timing, p50, p10) in ordered {
            if matches!(timing, transcript::WordTiming::Interpolated) {
                invalid.push(transcript::InvalidRegion {
                    start_ticks: start,
                    end_ticks: end,
                    reason: transcript::InvalidRegionReason::TimingInterpolated,
                    detail: Some(
                        "word timing was spread; the aligner placed nothing here"
                            .to_owned()
                            .try_into()
                            .unwrap_or_else(|_| unreachable!("a non-empty literal")),
                    ),
                });
            }
            words.push(transcript::Word {
                index: u64::try_from(words.len()).unwrap_or(u64::MAX),
                segment_index: segment.index,
                text: text.try_into().unwrap_or_else(|_| {
                    unreachable!("empty words are filtered before they reach here")
                }),
                start_ticks: start,
                end_ticks: end,
                confidence: transcript::Confidence { p50, p10 },
                timing,
            });
        }

        let count = u64::try_from(words.len()).unwrap_or(u64::MAX) - first_word_index;
        if count == 0 {
            continue;
        }
        let members = &words[usize::try_from(first_word_index).unwrap_or(0)..];
        segments.push(transcript::Segment {
            index: segment.index,
            start_ticks: members[0].start_ticks,
            end_ticks: members[members.len() - 1].end_ticks,
            text: segment.text.clone(),
            first_word_index,
            word_count: count,
            confidence: transcript::Confidence {
                p50: segment.confidence.p50,
                p10: segment.confidence.p10,
            },
        });
    }

    // Regions the upstream stages already declared invalid travel through
    // unchanged. A consumer reading only the transcript still has to be able
    // to see every span the chain does not vouch for.
    invalid.extend(
        recognized
            .invalid_regions
            .iter()
            .map(|region| transcript::InvalidRegion {
                start_ticks: region.start_ticks,
                end_ticks: region.end_ticks,
                reason: transcript::InvalidRegionReason::DecodeFailed,
                detail: region.detail.as_ref().and_then(|text| text.parse().ok()),
            }),
    );
    invalid.extend(
        alignment
            .invalid_regions
            .iter()
            .map(|region| transcript::InvalidRegion {
                start_ticks: region.start_ticks,
                end_ticks: region.end_ticks,
                reason: transcript::InvalidRegionReason::AlignmentUnavailable,
                detail: region.detail.as_ref().and_then(|text| text.parse().ok()),
            }),
    );
    invalid.sort_by_key(|region| (region.start_ticks, region.end_ticks));

    // Over the segments, not the words. A word's confidence in this document
    // is its *timing* confidence, which is what the aligner measured; the
    // question the document-level number answers — is this text safe to quote
    // — is one only recognition can answer.
    let p50 = distribution(
        &segments
            .iter()
            .map(|segment| segment.confidence.p50)
            .collect::<Vec<_>>(),
    )
    .0;
    let p10 = distribution(
        &segments
            .iter()
            .map(|segment| segment.confidence.p10)
            .collect::<Vec<_>>(),
    )
    .1;
    // The speech these words were placed within, per utterance — the same
    // reading the aligner publishes. Summing the words instead would omit the
    // gaps between them and understate how much of the recording has measured
    // timing.
    let mut aligned_ticks = 0;
    for segment in &segments {
        let first = usize::try_from(segment.first_word_index).unwrap_or(0);
        let count = usize::try_from(segment.word_count).unwrap_or(0);
        let members = &words[first..first + count];
        let measured = members
            .iter()
            .filter(|word| matches!(word.timing, transcript::WordTiming::Aligned))
            .collect::<Vec<_>>();
        if let (Some(first), Some(last)) = (measured.first(), measured.last()) {
            aligned_ticks += last.end_ticks.saturating_sub(first.start_ticks);
        }
    }

    let document =
        transcript::SpeechTranscript {
            schema_version: serde_json::json!("clipmill.speech.transcript.v1"),
            source_fingerprint: activity.source_fingerprint.as_str().parse().map_err(|_| {
                AssemblyError::MalformedAddress {
                    field: "source_fingerprint",
                }
            })?,
            inputs: transcript::SpeechTranscriptInputs {
                vad_artifact_id: inputs.vad.parse().map_err(|_| {
                    AssemblyError::MalformedAddress {
                        field: "vad_artifact_id",
                    }
                })?,
                asr_artifact_id: inputs.asr.parse().map_err(|_| {
                    AssemblyError::MalformedAddress {
                        field: "asr_artifact_id",
                    }
                })?,
                alignment_artifact_id: inputs.alignment.parse().map_err(|_| {
                    AssemblyError::MalformedAddress {
                        field: "alignment_artifact_id",
                    }
                })?,
                audio_artifact_id: activity.audio_artifact_id.as_str().parse().ok(),
            },
            producers: producers(activity, recognized, alignment, assembler),
            language: recognized
                .language
                .as_str()
                .parse()
                .map_err(|_| AssemblyError::MalformedAddress { field: "language" })?,
            language_confidence: recognized.language_confidence,
            confidence: transcript::Confidence { p50, p10 },
            coverage: transcript::Coverage {
                start_ticks: activity.coverage.start_ticks,
                end_ticks: activity.coverage.end_ticks,
                analyzed: true,
                speech_ticks: activity.speech_ticks,
                aligned_ticks,
                sampling_plan: "speech-chain-v1".parse().ok(),
            },
            words,
            segments,
            silences: activity
                .silences
                .iter()
                .map(|gap| transcript::Interval {
                    start_ticks: gap.start_ticks,
                    end_ticks: gap.end_ticks,
                })
                .collect(),
            invalid_regions: invalid,
        };
    Ok(Assembled { document })
}

/// The artifacts this transcript was fused from.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Inputs<'a> {
    pub vad: &'a str,
    pub asr: &'a str,
    pub alignment: &'a str,
}

type SpreadWord = (u64, u64, String, transcript::WordTiming, f64, f64);

/// Timing for the words the aligner would not place.
///
/// Each is put back where it belongs in the utterance and given the space its
/// neighbours left. When nothing in the utterance was placed, the words share
/// the decode window evenly. Either way the result is labelled interpolated
/// and its span is declared invalid, because a spread interval is a guess and
/// a consumer that cannot tell it from a measurement will cut inside a word.
fn spread_words(
    segment: &clipmill_contracts::schemas::speech_asr::AsrSegment,
    alignment: &SpeechAlignment,
    measured: &[&clipmill_contracts::schemas::speech_alignment::Word],
) -> Vec<SpreadWord> {
    let missing = alignment
        .unaligned
        .iter()
        .filter(|span| span.segment_index == segment.index)
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Vec::new();
    }

    // A whole utterance nobody placed: share its decode window evenly. The
    // recognizer's hint is not word timing, but it is the smallest span
    // certainly known to contain these words.
    if measured.is_empty() {
        let tokens = segment.text.split_whitespace().collect::<Vec<_>>();
        if tokens.is_empty() {
            return Vec::new();
        }
        let span = segment
            .hint_end_ticks
            .saturating_sub(segment.hint_start_ticks);
        let each = span / u64::try_from(tokens.len()).unwrap_or(1).max(1);
        return tokens
            .iter()
            .enumerate()
            .map(|(position, token)| {
                let offset = u64::try_from(position).unwrap_or(0) * each;
                (
                    segment.hint_start_ticks + offset,
                    segment.hint_start_ticks + offset + each,
                    (*token).to_owned(),
                    transcript::WordTiming::Interpolated,
                    0.0,
                    0.0,
                )
            })
            .collect();
    }

    // Individual words, dropped out of an utterance that otherwise aligned.
    // Each sits between the words its neighbours occupy.
    let placed_at = aligned_positions(segment, &missing, measured);
    let mut spread = Vec::new();
    for span in missing {
        let Some(position) = span.word_index else {
            continue;
        };
        let before = placed_at
            .iter()
            .filter(|(at, _)| *at < position)
            .map(|(_, word)| word.end_ticks)
            .max()
            .unwrap_or(segment.hint_start_ticks);
        let after = placed_at
            .iter()
            .filter(|(at, _)| *at > position)
            .map(|(_, word)| word.start_ticks)
            .min()
            .unwrap_or(segment.hint_end_ticks);
        let (start, end) = if after > before {
            (before, after)
        } else {
            // Neighbours meet: give the word a single frame of the timebase so
            // it still has an interval, and let the invalid region say what it
            // is worth.
            (before, before.saturating_add(1))
        };
        spread.push((
            start,
            end,
            span.text.clone(),
            transcript::WordTiming::Interpolated,
            0.0,
            0.0,
        ));
    }
    spread
}

/// Where each aligned word sits in its utterance's text.
///
/// Reconstructed by elimination rather than by matching text. Matching looked
/// simpler and was wrong twice over: the recognizer writes "tick." where the
/// aligner scored "tick", so punctuation made a word unfindable, and a word
/// nobody could find then sorted as though it came first. The aligner emits
/// words in order and names the positions it skipped, so the positions it
/// kept are exactly the rest — no comparison of strings required.
fn aligned_positions<'a>(
    segment: &clipmill_contracts::schemas::speech_asr::AsrSegment,
    missing: &[&clipmill_contracts::schemas::speech_alignment::UnalignedSpan],
    measured: &[&'a clipmill_contracts::schemas::speech_alignment::Word],
) -> Vec<(u64, &'a clipmill_contracts::schemas::speech_alignment::Word)> {
    let skipped = missing
        .iter()
        .filter_map(|span| span.word_index)
        .collect::<std::collections::BTreeSet<_>>();
    let tokens = u64::try_from(segment.text.split_whitespace().count()).unwrap_or(u64::MAX);
    (0..tokens)
        .filter(|position| !skipped.contains(position))
        .zip(measured.iter().copied())
        .collect()
}

fn producers(
    activity: &SpeechVad,
    recognized: &SpeechAsr,
    alignment: &SpeechAlignment,
    assembler: &str,
) -> Vec<transcript::Producer> {
    let mut producers = vec![
        producer(
            &activity.producer.stage,
            &activity.producer.implementation,
            activity
                .producer
                .model_digest
                .as_deref()
                .map(ToOwned::to_owned),
        ),
        producer(
            &recognized.producer.stage,
            &recognized.producer.implementation,
            recognized
                .producer
                .model_digest
                .as_deref()
                .map(ToOwned::to_owned),
        ),
        producer(
            &alignment.producer.stage,
            &alignment.producer.implementation,
            alignment
                .producer
                .model_digest
                .as_deref()
                .map(ToOwned::to_owned),
        ),
    ];
    // The assembly itself, which runs no model. Naming it keeps the producer
    // list a complete account of who touched the document rather than a list
    // of the interesting parts.
    producers.push(producer("speech-transcript", assembler, None));
    producers
}

fn producer(
    stage: &str,
    implementation: &str,
    model_digest: Option<String>,
) -> transcript::Producer {
    transcript::Producer {
        stage: stage
            .parse()
            .unwrap_or_else(|_| unreachable!("a stage name is never empty")),
        implementation: implementation
            .parse()
            .unwrap_or_else(|_| unreachable!("an implementation name is never empty")),
        model_digest: model_digest.and_then(|digest| digest.parse().ok()),
        calibration: None,
    }
}

/// The task kind this module executes.
pub(crate) const KIND_TRANSCRIPT: &str = "speech-transcript";
pub(crate) const IMPLEMENTATION: &str = "clipmill-transcript-assembly@1.0.0";
const OUTPUT_FILE: &str = "transcript.json";

/// Read the three published artifacts and publish the transcript that fuses
/// them.
///
/// Inputs are matched by the artifact kind their manifest declares, not by
/// the order the plan happened to list them. Positional matching works right
/// up until someone reorders a dependency list, and then it produces a
/// transcript that reads voice activity as alignment.
pub(crate) async fn execute_transcript_task(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    progress.set("stages", 0, 4);
    let mut activity: Option<(String, SpeechVad)> = None;
    let mut recognized: Option<(String, SpeechAsr)> = None;
    let mut alignment: Option<(String, SpeechAlignment)> = None;
    for artifact_id in &task.input_artifact_ids {
        let lease = artifacts
            .open(*artifact_id)
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        let id = artifact_id.to_string();
        match lease.kind() {
            "speech.vad.v1" => {
                activity = Some((id, media::read_artifact_document(&lease, "vad.json")?));
            }
            "speech.asr.v1" => {
                recognized = Some((id, media::read_artifact_document(&lease, "asr.json")?));
            }
            "speech.alignment.v1" => {
                alignment = Some((id, media::read_artifact_document(&lease, "alignment.json")?));
            }
            other => {
                return Err(TaskExecutionError::deterministic(format!(
                    "assembly was given a {other}, which is not part of the speech chain"
                )));
            }
        }
    }
    let (vad_id, activity) = activity.ok_or_else(|| {
        TaskExecutionError::deterministic("assembly has no voice activity to read")
    })?;
    let (asr_id, recognized) = recognized
        .ok_or_else(|| TaskExecutionError::deterministic("assembly has no recognition to read"))?;
    let (alignment_id, alignment) = alignment
        .ok_or_else(|| TaskExecutionError::deterministic("assembly has no alignment to read"))?;
    progress.set("stages", 3, 4);

    let assembled = assemble(
        &activity,
        &recognized,
        &alignment,
        Inputs {
            vad: &vad_id,
            asr: &asr_id,
            alignment: &alignment_id,
        },
        IMPLEMENTATION,
    )
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;

    let fingerprint: Sha256Digest = activity
        .source_fingerprint
        .strip_prefix("sha256:")
        .unwrap_or_default()
        .parse()
        .map_err(|_| TaskExecutionError::deterministic("the inputs carry no source fingerprint"))?;
    let mut config = Map::new();
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.speech.transcript.v1"),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "speech.transcript.v1".to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: KIND_TRANSCRIPT.to_owned(),
            implementation: IMPLEMENTATION.to_owned(),
            // Assembly runs no model. Naming one here would put a digest in
            // the key that had nothing to do with what this stage computed.
            model_digest: None,
        },
        inputs: task.input_artifact_ids.clone(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.speech.transcript.v1".to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;

    let staging = match media::prepare_or_hit(artifacts, recipe).await? {
        media::Prepared::Hit(artifact_id) => {
            progress.set("stages", 4, 4);
            return Ok(artifact_id);
        }
        media::Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let path = media::artifact_path(OUTPUT_FILE)?;
    let document = serde_json::to_value(&assembled.document)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let result = async {
        media::write_canonical_json(&staging, &path, &document)?;
        media::commit_staging(artifacts, staging_id.clone(), vec![path]).await
    }
    .await;
    if result.is_err() {
        media::abandon_staging(artifacts, staging_id).await;
    }
    progress.set("stages", 4, 4);
    result
}

#[cfg(test)]
mod tests;
