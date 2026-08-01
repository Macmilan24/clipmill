//! Turning aligned words into the published caption document.
//!
//! Everything decided here is decided once. The tokens are built from the
//! transcript and tagged against the lexicon and the topic index; the two
//! intents are then two runs of the same segmenter over the same tokens with
//! different numbers. That ordering is the design rule: there is no path
//! through this module that produces words for one intent which the other
//! cannot see.
//!
//! What the segmenter is told about the recording comes from documents that may
//! not exist. Without the evidence index there are no sentence boundaries to
//! break at and no salient terms to emphasise, and without shot detection there
//! are no cuts to avoid. Both are optional and both are recorded as present or
//! absent, because a cue set built without them is a weaker one and a reader
//! should be able to tell which they are holding.

use std::num::NonZeroU64;

use clipmill_contracts::schemas::{
    captions_cues::{
        CaptionCues, CaptionCuesDirection, CaptionCuesInputs, CaptionCuesIntents,
        CaptionCuesSegmentation, CaptionCuesSegmentationEmphasisSource,
        CaptionCuesSegmentationWeights, Coverage, Cue as DocumentCue, CueRegion, Intent,
        Line as DocumentLine, Producer, Profile as DocumentProfile, Token as DocumentToken,
    },
    evidence_shots::EvidenceShots,
    index_transcript::IndexTranscript,
    speech_transcript::SpeechTranscript,
};
use serde_json::json;
use thiserror::Error;

use crate::lexicon::{self, Break, FILLER_LEXICON};
use crate::profile::{self, Direction, Profile};
use crate::segment::{self, Span, Token, Weights};

/// A term must appear at least this many times in its topic before it is
/// treated as salient enough to emphasise. Emphasis comes from evidence, and a
/// word said once is not yet evidence of anything.
const SALIENT_OCCURRENCES: u64 = 2;

/// What was read, by address. Recorded in the document so any cue can be walked
/// back to the observations behind it.
#[derive(Clone, Copy, Debug)]
pub struct Inputs<'a> {
    pub transcript_artifact_id: &'a str,
    pub index_artifact_id: Option<&'a str>,
    pub shots_artifact_id: Option<&'a str>,
}

/// What the caller wants segmented, and how.
#[derive(Clone, Debug)]
pub struct DeriveRequest {
    /// The window to caption. `None` is the transcript's whole coverage.
    pub span: Option<Span>,
    pub weights: Weights,
    /// Where cues sit. One stable anchor in this phase, by decision rather than
    /// by omission — a caption that changes lane every cue reads as broken even
    /// when every move was locally right.
    pub region: CueRegion,
    pub implementation: String,
}

impl DeriveRequest {
    /// The defaults every caller uses unless it has a reason not to.
    pub fn new(implementation: impl Into<String>) -> Self {
        Self {
            span: None,
            weights: Weights::default(),
            region: CueRegion::LowerSafe,
            implementation: implementation.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum DeriveError {
    #[error("the transcript states no words to caption")]
    NoWords,
    #[error("the span has no extent to caption inside")]
    EmptySpan,
    #[error("the transcript and the index describe different recordings")]
    Mismatched,
    #[error("cue segmentation refused: {0}")]
    Segmentation(#[from] segment::SegmentError),
    #[error("a value did not fit the published contract: {0}")]
    Contract(String),
}

/// Derive the caption document.
pub fn derive(
    transcript: &SpeechTranscript,
    index: Option<&IndexTranscript>,
    shots: Option<&EvidenceShots>,
    inputs: Inputs<'_>,
    request: &DeriveRequest,
) -> Result<CaptionCues, DeriveError> {
    let fingerprint = transcript.source_fingerprint.as_str();
    if index.is_some_and(|other| other.source_fingerprint.as_str() != fingerprint)
        || shots.is_some_and(|other| other.source_fingerprint.as_str() != fingerprint)
    {
        return Err(DeriveError::Mismatched);
    }

    let language = transcript.language.as_str().to_owned();
    let (profiles, direction, _known) = profile::for_language(&language);
    let span = span_of(transcript, request)?;
    let tokens = tokens_of(transcript, index, span);
    if tokens.is_empty() {
        return Err(DeriveError::NoWords);
    }
    let shot_cuts = cuts_of(shots, span);

    let accessibility = intent_of(&tokens, &shot_cuts, span, profiles.accessibility, request)?;
    let burn_in = intent_of(&tokens, &shot_cuts, span, profiles.burn_in, request)?;

    let document_tokens = tokens
        .iter()
        .enumerate()
        .map(|(index, token)| document_token(index, token))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CaptionCues {
        schema_version: json!("clipmill.captions.cues.v1"),
        source_fingerprint: parse(transcript.source_fingerprint.as_str())?,
        inputs: CaptionCuesInputs {
            transcript_artifact_id: parse(inputs.transcript_artifact_id)?,
            index_artifact_id: inputs.index_artifact_id.map(parse).transpose()?,
            shots_artifact_id: inputs.shots_artifact_id.map(parse).transpose()?,
        },
        producer: Producer {
            stage: "captions.derive"
                .parse()
                .map_err(|_| DeriveError::Contract("stage".to_owned()))?,
            implementation: request
                .implementation
                .parse()
                .map_err(|_| DeriveError::Contract("implementation".to_owned()))?,
        },
        language: language
            .parse()
            .map_err(|_| DeriveError::Contract("language".to_owned()))?,
        direction: match direction {
            Direction::Ltr => CaptionCuesDirection::Ltr,
            Direction::Rtl => CaptionCuesDirection::Rtl,
        },
        segmentation: CaptionCuesSegmentation {
            span_start_ticks: as_u64(span.start_ticks),
            span_end_ticks: nonzero(span.end_ticks)?,
            weights: CaptionCuesSegmentationWeights {
                reading_rate: request.weights.reading_rate,
                line_balance: request.weights.line_balance,
                orphan: request.weights.orphan,
                break_quality: request.weights.break_quality,
                short_cue: request.weights.short_cue,
            },
            filler_lexicon: FILLER_LEXICON
                .parse()
                .map_err(|_| DeriveError::Contract("filler_lexicon".to_owned()))?,
            emphasis_source: if index.is_some() {
                CaptionCuesSegmentationEmphasisSource::IndexKeywords
            } else {
                CaptionCuesSegmentationEmphasisSource::None
            },
            had_index: index.is_some(),
            had_shots: shots.is_some(),
        },
        coverage: Coverage {
            start_ticks: as_u64(span.start_ticks),
            end_ticks: as_u64(span.end_ticks),
            analyzed: true,
        },
        tokens: document_tokens,
        intents: CaptionCuesIntents {
            accessibility,
            burn_in,
        },
        // Nothing has been corrected at derive time. The overlay exists so a
        // later re-transcription can propose without erasing, and an empty one
        // is the honest state of a document nobody has edited.
        corrections: Vec::new(),
        invalid_regions: Vec::new(),
    })
}

/// The window to caption: what the caller asked for, clamped to what the
/// transcript actually covers.
fn span_of(transcript: &SpeechTranscript, request: &DeriveRequest) -> Result<Span, DeriveError> {
    let covered = Span {
        start_ticks: as_i64(transcript.coverage.start_ticks),
        end_ticks: as_i64(transcript.coverage.end_ticks),
    };
    let wanted = request.span.unwrap_or(covered);
    let span = Span {
        start_ticks: wanted.start_ticks.max(covered.start_ticks),
        end_ticks: wanted.end_ticks.min(covered.end_ticks),
    };
    if span.end_ticks <= span.start_ticks {
        return Err(DeriveError::EmptySpan);
    }
    Ok(span)
}

/// Every word inside the span, tagged.
fn tokens_of(
    transcript: &SpeechTranscript,
    index: Option<&IndexTranscript>,
    span: Span,
) -> Vec<Built> {
    let salient = salient_terms(index);
    let sentence_ends = sentence_ends(index);

    transcript
        .words
        .iter()
        .filter(|word| {
            as_i64(word.start_ticks) >= span.start_ticks && as_i64(word.end_ticks) <= span.end_ticks
        })
        .map(|word| {
            let text = word.text.to_string();
            let normalized = lexicon::normalize(&text);
            let filler = lexicon::is_filler(&normalized);
            // A sentence the index found ending here outranks the punctuation,
            // because the index saw the pause as well as the full stop.
            let punctuation = lexicon::break_after(&text);
            let break_after = if sentence_ends.contains(&word.index) {
                Break::Sentence
            } else {
                punctuation
            };
            Built {
                token: Token {
                    orphans: lexicon::orphans_if_last(&normalized),
                    // A filler may never carry emphasis: emphasising "um" is
                    // the clearest possible signal that nothing understood the
                    // sentence.
                    emphasis: !filler && !normalized.is_empty() && salient.contains(&normalized),
                    filler,
                    text,
                    normalized,
                    start_ticks: as_i64(word.start_ticks),
                    end_ticks: as_i64(word.end_ticks),
                    break_after,
                },
                word_index: word.index,
                confidence: word.confidence.p50,
            }
        })
        .collect()
}

/// One token, plus what it needs to point back at the transcript.
struct Built {
    token: Token,
    word_index: u64,
    confidence: f64,
}

/// The terms a topic appeared to be about, which is the only place emphasis is
/// allowed to come from.
fn salient_terms(index: Option<&IndexTranscript>) -> Vec<String> {
    let mut terms: Vec<String> = index
        .into_iter()
        .flat_map(|document| document.topics.iter())
        .flat_map(|topic| topic.keywords.iter())
        .filter(|keyword| keyword.count.get() >= SALIENT_OCCURRENCES)
        .map(|keyword| lexicon::normalize(keyword.term.as_str()))
        .filter(|term| !term.is_empty())
        .collect();
    terms.sort_unstable();
    terms.dedup();
    terms
}

/// The word index each sentence ends on.
fn sentence_ends(index: Option<&IndexTranscript>) -> Vec<u64> {
    let mut ends: Vec<u64> = index
        .into_iter()
        .flat_map(|document| document.sentences.iter())
        .map(|sentence| sentence.first_word_index + sentence.word_count.get() - 1)
        .collect();
    ends.sort_unstable();
    ends.dedup();
    ends
}

/// The cuts inside the span, in order.
fn cuts_of(shots: Option<&EvidenceShots>, span: Span) -> Vec<i64> {
    let mut shot_cuts: Vec<i64> = shots
        .into_iter()
        .flat_map(|document| document.cuts.iter())
        .map(|cut| as_i64(cut.t_ticks))
        .filter(|at| *at > span.start_ticks && *at < span.end_ticks)
        .collect();
    shot_cuts.sort_unstable();
    shot_cuts.dedup();
    shot_cuts
}

/// One grouping of the tokens, segmented and written out.
fn intent_of(
    tokens: &[Built],
    shot_cuts: &[i64],
    span: Span,
    profile: Profile,
    request: &DeriveRequest,
) -> Result<Intent, DeriveError> {
    let words: Vec<Token> = tokens.iter().map(|built| built.token.clone()).collect();
    let cues = segment::segment(&words, shot_cuts, span, profile, request.weights)?;
    let written = cues
        .iter()
        .enumerate()
        .map(|(ordinal, cue)| document_cue(ordinal, cue, request.region))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Intent {
        profile: document_profile(profile)?,
        cues: written,
    })
}

fn document_cue(
    ordinal: usize,
    cue: &segment::Cue,
    region: CueRegion,
) -> Result<DocumentCue, DeriveError> {
    Ok(DocumentCue {
        // Ordinal rather than random: a cue's identity is its position in this
        // document, and two runs over the same words must agree about it.
        cue_id: format!("cue_{}", ordinal + 1)
            .parse()
            .map_err(|_| DeriveError::Contract("cue_id".to_owned()))?,
        start_ticks: as_u64(cue.start_ticks),
        end_ticks: nonzero(cue.end_ticks)?,
        first_token: as_u64_from_usize(cue.first_token),
        token_count: nonzero_usize(cue.token_count)?,
        region,
        lines: cue
            .lines
            .iter()
            .map(|line| {
                Ok(DocumentLine {
                    first_token: as_u64_from_usize(line.first_token),
                    token_count: nonzero_usize(line.token_count)?,
                    characters: nonzero_usize(line.characters)?,
                })
            })
            .collect::<Result<Vec<_>, DeriveError>>()?,
        characters: nonzero_usize(cue.characters)?,
        reading_rate_cps: cue.reading_rate_cps,
    })
}

fn document_profile(profile: Profile) -> Result<DocumentProfile, DeriveError> {
    Ok(DocumentProfile {
        max_line_characters: nonzero_usize(profile.max_line_characters)?,
        max_lines: nonzero_usize(profile.max_lines)?,
        reading_rate_cps: profile.reading_rate_cps,
        min_duration_ticks: nonzero(profile.min_duration_ticks)?,
        max_duration_ticks: nonzero(profile.max_duration_ticks)?,
        min_gap_ticks: as_u64(profile.min_gap_ticks),
    })
}

fn document_token(index: usize, built: &Built) -> Result<DocumentToken, DeriveError> {
    Ok(DocumentToken {
        index: as_u64_from_usize(index),
        word_index: built.word_index,
        text: built
            .token
            .text
            .parse()
            .map_err(|_| DeriveError::Contract("token text".to_owned()))?,
        normalized: built.token.normalized.clone(),
        start_ticks: as_u64(built.token.start_ticks),
        end_ticks: nonzero(built.token.end_ticks)?,
        confidence: built.confidence.clamp(0.0, 1.0),
        speaker: None,
        filler: built.token.filler,
        emphasis: built.token.emphasis,
    })
}

fn parse(value: &str) -> Result<clipmill_contracts::schemas::captions_cues::Sha256, DeriveError> {
    value
        .parse()
        .map_err(|_| DeriveError::Contract(format!("{value} is not an artifact address")))
}

fn as_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn as_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

fn as_u64_from_usize(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn nonzero(value: i64) -> Result<NonZeroU64, DeriveError> {
    NonZeroU64::new(as_u64(value))
        .ok_or_else(|| DeriveError::Contract("a value the contract requires to be positive".into()))
}

fn nonzero_usize(value: usize) -> Result<NonZeroU64, DeriveError> {
    NonZeroU64::new(as_u64_from_usize(value))
        .ok_or_else(|| DeriveError::Contract("a value the contract requires to be positive".into()))
}
