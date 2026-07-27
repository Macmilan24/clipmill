//! Reading structure out of a transcript.
//!
//! Discovery has to propose spans of a recording worth clipping. Without an
//! index it would propose them over a flat list of words, which means every
//! proposer re-deriving "where does a sentence end" slightly differently and
//! none of them able to say why a boundary was chosen. This crate derives that
//! structure once, states how each part of it was decided, and links every
//! unit back to the words it came from (book ch. 14, Rule 14.1).
//!
//! Two levels, and the line between them is what the stage is willing to
//! claim. **L1** is what the recording states about itself: utterances where
//! voice activity heard a pause, sentences where the recognizer punctuated,
//! and the edges a clip may start or end on. **L2** is topics, by lexical
//! cohesion over the words rather than by any model that reads them — a real
//! approximation, published under a name that says so.
//!
//! There is no L3 and no open-loop detection. Both would require understanding
//! the words, and a stage that claimed it here would be handing the next stage
//! a promise nobody kept.
//!
//! Nothing in this crate does any I/O. It takes two parsed documents and
//! returns a third, which is what lets the whole level be tested against
//! transcripts written by hand — including the ones no real recording
//! produces.

pub mod confidence;
mod stopwords;
mod topics;
mod units;

use clipmill_contracts::schemas::evidence_shots::EvidenceShots;
use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::speech_transcript::SpeechTranscript;

pub use stopwords::IDENTIFIER as STOPWORDS;

/// Who produced an index, recorded in the document.
pub const STAGE: &str = "index-transcript";

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("the transcript was never analyzed, so there is no structure to read out of it")]
    NotAnalyzed,
    #[error("the transcript and the shot detection describe different sources")]
    MismatchedSources,
    #[error("{field} is not a well-formed content address")]
    MalformedAddress { field: &'static str },
}

/// The decision parameters, all of which reach the artifact key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Parameters {
    /// A pause at least this long ends an utterance.
    pub utterance_gap_ticks: u64,
    /// Sentences compared on each side of a gap when looking for a topic
    /// boundary.
    pub block_sentences: u64,
    /// Standard deviations below the mean depth at which the topic boundary
    /// threshold sits, scaled by a thousand so the value that reaches a
    /// protobuf payload and an artifact key is an integer.
    pub boundary_cutoff_milli: u64,
}

impl Parameters {
    /// Chosen for conversational speech. A third of a second of quiet ends an
    /// utterance, which is voice activity's own default; two sentences of
    /// context on each side of a gap is enough to see a subject change without
    /// smoothing away a short one; and half a standard deviation below the
    /// mean is Hearst's own cutoff, kept because nothing measured here
    /// justifies moving it yet.
    pub const DEFAULT: Self = Self {
        utterance_gap_ticks: 27_000,
        block_sentences: 2,
        boundary_cutoff_milli: 500,
    };

    fn cutoff(self) -> f64 {
        #[allow(clippy::cast_precision_loss, reason = "a small integer, exact in f64")]
        let value = self.boundary_cutoff_milli as f64 / 1000.0;
        value
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The artifacts an index was built from.
#[derive(Clone, Copy, Debug)]
pub struct Inputs<'a> {
    pub transcript: &'a str,
    /// Absent for a source with no video, which is a different document rather
    /// than the same one with a shorter edge list.
    pub shots: Option<&'a str>,
}

/// Derive the index. Every unit published here resolves to words the
/// transcript measured; nothing is invented that a consumer could not have
/// derived itself from the same two documents.
#[allow(
    clippy::too_many_lines,
    reason = "one pass over the words, then one over the sentences; splitting it would hide the order"
)]
pub fn index(
    transcript: &SpeechTranscript,
    shots: Option<&EvidenceShots>,
    inputs: Inputs<'_>,
    parameters: Parameters,
    implementation: &str,
) -> Result<index::IndexTranscript, IndexError> {
    if !transcript.coverage.analyzed {
        return Err(IndexError::NotAnalyzed);
    }
    if let Some(shots) = shots
        && *shots.source_fingerprint != *transcript.source_fingerprint
    {
        return Err(IndexError::MismatchedSources);
    }

    let coverage = (
        transcript.coverage.start_ticks,
        transcript.coverage.end_ticks,
    );
    let words = &transcript.words;
    let runs = units::utterance_runs(words, &transcript.silences, parameters.utterance_gap_ticks);

    let mut utterances = Vec::new();
    let mut sentences = Vec::new();
    let mut sentence_tokens = Vec::new();
    for (position, run) in runs.iter().enumerate() {
        let start = words[run.first].start_ticks;
        let end = words[run.last()].end_ticks;
        let before = runs
            .get(position.wrapping_sub(1))
            .map_or(coverage.0, |previous| words[previous.last()].end_ticks);
        let after = runs
            .get(position + 1)
            .map_or(coverage.1, |next| words[next.first].start_ticks);
        utterances.push(index::Utterance {
            index: as_u64(position),
            start_ticks: start,
            end_ticks: end,
            first_word_index: as_u64(run.first),
            word_count: at_least_one(run.count),
            text: string(&units::text_of(words, *run), "utterance text")?,
            pause_before_ticks: start.saturating_sub(before),
            pause_after_ticks: after.saturating_sub(end),
            words_per_minute: units::words_per_minute(run.count, start, end),
            confidence: units::confidence_of(words, *run),
        });

        for (sentence, terminator) in units::sentence_runs(words, &transcript.segments, *run) {
            let start = words[sentence.first].start_ticks;
            let end = words[sentence.last()].end_ticks;
            let text = units::text_of(words, sentence);
            sentence_tokens.push(topics::tokenize(&text));
            sentences.push(index::Sentence {
                index: as_u64(sentences.len()),
                utterance_index: as_u64(position),
                start_ticks: start,
                end_ticks: end,
                first_word_index: as_u64(sentence.first),
                word_count: at_least_one(sentence.count),
                text: string(&text, "sentence text")?,
                terminator,
                words_per_minute: units::words_per_minute(sentence.count, start, end),
                confidence: units::confidence_of(words, sentence),
            });
        }
    }

    // The recording ran out rather than the speaker stopping. Only the last
    // sentence can be in that position, and only when nothing punctuated it
    // and no real pause followed.
    if let Some(last) = sentences.last_mut()
        && matches!(last.terminator, index::SentenceTerminator::UtteranceEnd)
        && coverage.1.saturating_sub(last.end_ticks) < parameters.utterance_gap_ticks
    {
        last.terminator = index::SentenceTerminator::CoverageEnd;
    }

    let cuts = shots
        .map(|shots| shots.cuts.iter().map(|cut| cut.t_ticks).collect::<Vec<_>>())
        .unwrap_or_default();
    let edges = units::edges(&transcript.silences, &cuts, coverage);

    let topics = topics::segment(
        &sentence_tokens,
        usize::try_from(parameters.block_sentences).unwrap_or(1),
        parameters.cutoff(),
    )
    .into_iter()
    .enumerate()
    .map(|(position, topic)| {
        let first = &sentences[topic.first_sentence];
        let last = &sentences[topic.first_sentence + topic.sentence_count - 1];
        Ok(index::Topic {
            index: as_u64(position),
            start_ticks: first.start_ticks,
            end_ticks: last.end_ticks,
            first_sentence_index: as_u64(topic.first_sentence),
            sentence_count: at_least_one(topic.sentence_count),
            opening_depth: topic.opening_depth,
            keywords: topic
                .keywords
                .into_iter()
                .map(|(term, count)| {
                    Ok(index::Keyword {
                        term: string(&term, "keyword")?,
                        count: std::num::NonZeroU64::new(count).unwrap_or(MIN),
                    })
                })
                .collect::<Result<Vec<_>, IndexError>>()?,
        })
    })
    .collect::<Result<Vec<_>, IndexError>>()?;

    Ok(index::IndexTranscript {
        schema_version: serde_json::json!("clipmill.index.transcript.v1"),
        source_fingerprint: parse(transcript.source_fingerprint.as_str(), "source_fingerprint")?,
        inputs: index::IndexTranscriptInputs {
            transcript_artifact_id: parse(inputs.transcript, "transcript_artifact_id")?,
            shots_artifact_id: inputs
                .shots
                .map(|id| parse(id, "shots_artifact_id"))
                .transpose()?,
        },
        producer: index::Producer {
            stage: parse(STAGE, "producer stage")?,
            implementation: parse(implementation, "producer implementation")?,
        },
        language: parse(transcript.language.as_str(), "language")?,
        segmentation: index::IndexTranscriptSegmentation {
            utterance_gap_ticks: parameters.utterance_gap_ticks,
            block_sentences: std::num::NonZeroU64::new(parameters.block_sentences).unwrap_or(MIN),
            boundary_cutoff: parameters.cutoff(),
            stopwords: parse(STOPWORDS, "stopwords")?,
        },
        coverage: index::Coverage {
            start_ticks: coverage.0,
            end_ticks: coverage.1,
            analyzed: true,
        },
        utterances,
        sentences,
        edges,
        topics,
        // Carried rather than recomputed. The index inherits exactly the
        // uncertainty the transcript declared, and adds none of its own.
        invalid_regions: transcript
            .invalid_regions
            .iter()
            .map(|region| index::InvalidRegion {
                start_ticks: region.start_ticks,
                end_ticks: region.end_ticks,
                reason: reason(region.reason),
                detail: region
                    .detail
                    .as_ref()
                    .and_then(|text| text.as_str().parse().ok()),
            })
            .collect(),
    })
}

fn reason(
    from: clipmill_contracts::schemas::speech_transcript::InvalidRegionReason,
) -> index::InvalidRegionReason {
    use clipmill_contracts::schemas::speech_transcript::InvalidRegionReason as From;
    match from {
        From::NotAnalyzed => index::InvalidRegionReason::NotAnalyzed,
        From::NoAudio => index::InvalidRegionReason::NoAudio,
        From::DecodeFailed => index::InvalidRegionReason::DecodeFailed,
        From::AlignmentUnavailable => index::InvalidRegionReason::AlignmentUnavailable,
        From::TimingInterpolated => index::InvalidRegionReason::TimingInterpolated,
    }
}

fn as_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// The smallest count the contract admits.
///
/// The schema says a unit holds at least one member, and typify turns that
/// into a type that cannot hold zero — which is the right shape, since a
/// sentence with no words is not a sentence. Every call below is over a run
/// built to be non-empty, so the fallback is unreachable rather than a
/// silently clamped value; it exists because the alternative is an `unwrap`
/// this workspace refuses.
const MIN: std::num::NonZeroU64 = std::num::NonZeroU64::MIN;

fn at_least_one(value: usize) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(as_u64(value)).unwrap_or(MIN)
}

fn parse<T: std::str::FromStr>(value: &str, field: &'static str) -> Result<T, IndexError> {
    value
        .parse()
        .map_err(|_| IndexError::MalformedAddress { field })
}

fn string<T: std::str::FromStr>(value: &str, field: &'static str) -> Result<T, IndexError> {
    value
        .parse()
        .map_err(|_| IndexError::MalformedAddress { field })
}

#[cfg(test)]
mod tests;
