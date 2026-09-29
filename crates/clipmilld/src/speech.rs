//! Assemble VAD, recognition, and alignment into one transcript observation.
//!
//! Unaligned words retain their text with interpolated timing and invalid-span
//! markers, so downstream boundaries cannot mistake interpolation for measured
//! word timing. This model-free derivation runs in the daemon over the three
//! published documents.

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
        let mut collapsed = Vec::new();
        for word in measured {
            // A timestamp pair on the same aligner tick is not a measured
            // interval. Keep the spoken text and mark the timing as inferred;
            // the run it collapsed with is spread out below.
            let is_collapsed = word.end_ticks <= word.start_ticks;
            collapsed.push(is_collapsed);
            ordered.push((
                word.start_ticks,
                if is_collapsed {
                    word.start_ticks.saturating_add(1)
                } else {
                    word.end_ticks
                },
                (*word.text).clone(),
                if is_collapsed {
                    transcript::WordTiming::Interpolated
                } else {
                    transcript::WordTiming::Aligned
                },
                if is_collapsed {
                    0.0
                } else {
                    word.confidence.p50
                },
                if is_collapsed {
                    0.0
                } else {
                    word.confidence.p10
                },
            ));
        }
        spread_collapsed_runs(&mut ordered, &collapsed, &activity.silences);
        ordered.extend(spread);
        ordered.sort_by_key(|entry| (entry.0, entry.1));

        let (ordered, loops) = drop_repeated_loops(&ordered);
        invalid.extend(loops);
        let ordered = without_annotations(ordered, |entry| entry.2.as_str());
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
            text: strip_annotations(segment.text.as_str())
                .parse()
                .unwrap_or_else(|_| segment.text.clone()),
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

/// The longest a collapsed word may be stretched to, when nothing measured
/// bounds it: about one syllable-heavy word at conversational speed.
const COLLAPSED_WORD_MAX_TICKS: u64 = 36_000;
/// The fewest times one word may repeat back to back before the run is read
/// as a recognizer loop rather than speech.
const LOOP_MIN_REPEATS: usize = 5;
/// How many of a looped word are kept: a stutter is real speech, twenty of
/// them in a row is the decoder repeating itself.
const LOOP_KEPT: usize = 2;

/// Give each run of words the aligner collapsed onto one tick its own time.
///
/// The Qwen3 aligner reports on an 80 ms grid, and a fast run of short words
/// can land on a single tick with no length at all. Left there, every word of
/// the run starts together and a karaoke highlight jumps over all of them at
/// once. The run is spread from where it was placed toward the next measured
/// word, in proportion to each word's syllables, never into a silence the
/// voice detector found and never past a plausible speaking pace. The words
/// stay labelled interpolated: this is a better guess, not a measurement.
fn spread_collapsed_runs(
    ordered: &mut [SpreadWord],
    collapsed: &[bool],
    silences: &[clipmill_contracts::schemas::speech_vad::Interval],
) {
    let mut index = 0;
    while index < collapsed.len() {
        if !collapsed[index] {
            index += 1;
            continue;
        }
        let mut end = index + 1;
        while end < collapsed.len() && collapsed[end] && ordered[end].0 == ordered[index].0 {
            end += 1;
        }
        let run_start = ordered[index].0;
        let previous_end = index
            .checked_sub(1)
            .map_or(run_start, |before| ordered[before].1.max(run_start));
        let count = u64::try_from(end - index).unwrap_or(1).max(1);
        let mut limit = run_start.saturating_add(COLLAPSED_WORD_MAX_TICKS.saturating_mul(count));
        if let Some(next) = ordered.get(end) {
            limit = limit.min(next.0);
        }
        if let Some(silence) = silences
            .iter()
            .find(|gap| gap.start_ticks > previous_end && gap.start_ticks < limit)
        {
            limit = silence.start_ticks;
        }
        let span = limit.saturating_sub(previous_end);
        if span >= count {
            let weights = ordered[index..end]
                .iter()
                .map(|word| syllables(&word.2))
                .collect::<Vec<_>>();
            let total = weights.iter().sum::<u64>().max(1);
            let mut cursor = previous_end;
            let mut spent = 0;
            for (word, weight) in ordered[index..end].iter_mut().zip(&weights) {
                spent += weight;
                let next = previous_end + span * spent / total;
                word.0 = cursor;
                word.1 = next.max(cursor + 1);
                cursor = word.1;
            }
        }
        index = end;
    }
}

/// A rough syllable count: groups of vowels, at least one per word.
fn syllables(text: &str) -> u64 {
    let mut groups = 0_u64;
    let mut in_vowel = false;
    for character in text.chars().flat_map(char::to_lowercase) {
        let vowel = matches!(character, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
        if vowel && !in_vowel {
            groups += 1;
        }
        in_vowel = vowel;
    }
    groups.max(1)
}

/// Remove a word the recognizer repeated far past anything a speaker says.
///
/// Whisper's small models fall into loops on noise and music: "it" twenty
/// times, "I" thirteen. Captions and the editorial model would read those as
/// speech. A run of one normalized word repeated [`LOOP_MIN_REPEATS`] or more
/// times keeps its first [`LOOP_KEPT`] and the rest is declared invalid, so
/// the gap stays visible to anyone reading the transcript.
fn drop_repeated_loops(
    ordered: &[SpreadWord],
) -> (Vec<SpreadWord>, Vec<transcript::InvalidRegion>) {
    let key = |text: &str| {
        text.chars()
            .filter(|character| character.is_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    let mut kept = Vec::with_capacity(ordered.len());
    let mut invalid = Vec::new();
    let mut index = 0;
    while index < ordered.len() {
        let word = key(&ordered[index].2);
        let mut end = index + 1;
        while end < ordered.len() && !word.is_empty() && key(&ordered[end].2) == word {
            end += 1;
        }
        if end - index >= LOOP_MIN_REPEATS {
            kept.extend(ordered[index..index + LOOP_KEPT].iter().cloned());
            invalid.push(transcript::InvalidRegion {
                start_ticks: ordered[index + LOOP_KEPT].0,
                end_ticks: ordered[end - 1].1,
                reason: transcript::InvalidRegionReason::DecodeFailed,
                detail: Some(
                    "the recognizer repeated one word in a loop; the repeats were removed"
                        .to_owned()
                        .try_into()
                        .unwrap_or_else(|_| unreachable!("a non-empty literal")),
                ),
            });
        } else {
            kept.extend(ordered[index..end].iter().cloned());
        }
        index = end;
    }
    (kept, invalid)
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
    // Each sits between the words its neighbours occupy — and words dropped
    // side by side share that gap in order, rather than each taking all of
    // it. Two words given one interval read as two words at once, which no
    // caption can carry and the document's own rules refuse.
    let placed_at = aligned_positions(segment, &missing, measured);
    let mut missing = missing
        .iter()
        .filter_map(|span| span.word_index.map(|position| (position, *span)))
        .collect::<Vec<_>>();
    missing.sort_by_key(|(position, _)| *position);
    let mut spread = Vec::new();
    let mut run_start = 0;
    while run_start < missing.len() {
        let (first_position, _) = missing[run_start];
        let before = placed_at
            .iter()
            .filter(|(at, _)| *at < first_position)
            .map(|(_, word)| word.end_ticks)
            .max()
            .unwrap_or(segment.hint_start_ticks);
        let after = placed_at
            .iter()
            .filter(|(at, _)| *at > first_position)
            .map(|(_, word)| word.start_ticks)
            .min()
            .unwrap_or(segment.hint_end_ticks);
        // The run: every missing word up to the next placed one.
        let mut run_end = run_start + 1;
        while run_end < missing.len()
            && placed_at
                .iter()
                .all(|(at, _)| *at < first_position || *at > missing[run_end].0)
        {
            run_end += 1;
        }
        let run = &missing[run_start..run_end];
        let count = u64::try_from(run.len()).unwrap_or(1).max(1);
        // Each word gets an equal share of the gap and never less than one
        // tick, so the run stays in order even when its neighbours meet and
        // the invalid region is left to say what the timing is worth.
        let each = (after.saturating_sub(before) / count).max(1);
        for (offset, (_, span)) in (0..).zip(run) {
            let start = before.saturating_add(offset * each);
            spread.push((
                start,
                start.saturating_add(each),
                span.text.clone(),
                transcript::WordTiming::Interpolated,
                0.0,
                0.0,
            ));
        }
        run_start = run_end;
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

/// Recognition writes silence and sound markers as words: whisper.cpp's
/// `[BLANK_AUDIO]` and `[music]`, or `(speaking in foreign language)`, which
/// arrives as four. Nobody said them, so a transcript leaves them out: every
/// word from an opening bracket to the one that closes it, within a segment.
fn without_annotations<T>(words: Vec<T>, text: impl Fn(&T) -> &str) -> Vec<T> {
    let mut kept = Vec::with_capacity(words.len());
    let mut closing = None;
    for word in words {
        let text = text(&word).trim();
        if let Some(close) = closing {
            if text.contains(close) {
                closing = None;
            }
            continue;
        }
        let close = match text.chars().next() {
            Some('[') => ']',
            Some('(') => ')',
            _ => {
                kept.push(word);
                continue;
            }
        };
        if !text.contains(close) {
            closing = Some(close);
        }
    }
    kept
}

/// A segment's text without the annotations recognition wrote into it. Only
/// a bracket that closes is an annotation; one that never does is kept.
fn strip_annotations(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find(['[', '(']) {
        let close = if rest[open..].starts_with('[') {
            ']'
        } else {
            ')'
        };
        let Some(length) = rest[open..].find(close) else {
            break;
        };
        kept.push_str(&rest[..open]);
        kept.push(' ');
        rest = &rest[open + length + close.len_utf8()..];
    }
    kept.push_str(rest);
    kept.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The task kind this module executes.
pub(crate) const KIND_TRANSCRIPT: &str = "speech-transcript";
/// 1.2.0 leaves out the annotations recognition writes as words.
pub(crate) const IMPLEMENTATION: &str = "clipmill-transcript-assembly@1.2.0";
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
