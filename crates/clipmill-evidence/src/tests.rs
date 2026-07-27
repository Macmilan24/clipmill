//! The index, against transcripts written by hand.
//!
//! Real recordings do not contain a speaker who never pauses, a recognizer
//! that punctuates nothing, or a segment whose text and word count disagree.
//! Those are the cases that decide whether a sentence boundary is an
//! observation or a guess, so they are written out here rather than waited for.
#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp
)]

use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::speech_transcript as transcript;

use super::{Inputs, Parameters, index as build};

const FINGERPRINT: &str = "sha256:31ab000000000000000000000000000000000000000000000000000000000007";
const TRANSCRIPT_ID: &str =
    "sha256:7a11000000000000000000000000000000000000000000000000000000000042";

fn word(at: u64, segment: u64, text: &str, start: u64, end: u64) -> transcript::Word {
    transcript::Word {
        index: at,
        segment_index: segment,
        text: text.parse().expect("a word is never empty"),
        start_ticks: start,
        end_ticks: end,
        confidence: transcript::Confidence { p50: 0.9, p10: 0.8 },
        timing: transcript::WordTiming::Aligned,
    }
}

fn segment(
    at: u64,
    first: u64,
    count: u64,
    text: &str,
    start: u64,
    end: u64,
) -> transcript::Segment {
    transcript::Segment {
        index: at,
        first_word_index: first,
        word_count: count,
        text: text.to_owned(),
        start_ticks: start,
        end_ticks: end,
        confidence: transcript::Confidence { p50: 0.9, p10: 0.8 },
    }
}

/// A transcript with two utterances a full second apart, the first punctuated
/// and the second not.
fn two_utterances() -> transcript::SpeechTranscript {
    build_transcript(
        vec![
            word(0, 0, "the", 0, 9_000),
            word(1, 0, "renderer", 9_000, 18_000),
            word(2, 0, "draws", 18_000, 27_000),
            word(3, 1, "frames", 117_000, 126_000),
            word(4, 1, "arrive", 126_000, 135_000),
        ],
        vec![
            segment(0, 0, 3, "The renderer draws.", 0, 27_000),
            segment(1, 3, 2, "Frames arrive", 117_000, 135_000),
        ],
        vec![transcript::Interval {
            start_ticks: 27_000,
            end_ticks: 117_000,
        }],
        (0, 180_000),
    )
}

fn build_transcript(
    words: Vec<transcript::Word>,
    segments: Vec<transcript::Segment>,
    silences: Vec<transcript::Interval>,
    coverage: (u64, u64),
) -> transcript::SpeechTranscript {
    transcript::SpeechTranscript {
        schema_version: serde_json::json!("clipmill.speech.transcript.v1"),
        source_fingerprint: FINGERPRINT.parse().expect("a fixture digest"),
        inputs: transcript::SpeechTranscriptInputs {
            vad_artifact_id: TRANSCRIPT_ID.parse().expect("a fixture digest"),
            asr_artifact_id: TRANSCRIPT_ID.parse().expect("a fixture digest"),
            alignment_artifact_id: TRANSCRIPT_ID.parse().expect("a fixture digest"),
            audio_artifact_id: None,
        },
        producers: Vec::new(),
        language: "en".parse().expect("a language tag"),
        language_confidence: Some(0.99),
        confidence: transcript::Confidence { p50: 0.9, p10: 0.8 },
        coverage: transcript::Coverage {
            start_ticks: coverage.0,
            end_ticks: coverage.1,
            analyzed: true,
            speech_ticks: 0,
            aligned_ticks: 0,
            sampling_plan: None,
        },
        words,
        segments,
        silences,
        invalid_regions: Vec::new(),
    }
}

fn indexed(document: &transcript::SpeechTranscript) -> index::IndexTranscript {
    build(
        document,
        None,
        Inputs {
            transcript: TRANSCRIPT_ID,
            shots: None,
        },
        Parameters::DEFAULT,
        "clipmill-evidence-test@0.0.1",
    )
    .expect("the index builds")
}

#[test]
fn a_pause_ends_an_utterance_and_a_full_stop_ends_a_sentence() {
    let result = indexed(&two_utterances());
    assert_eq!(result.utterances.len(), 2);
    assert_eq!(result.utterances[0].word_count.get(), 3);
    assert_eq!(result.utterances[1].first_word_index, 3);
    assert_eq!(result.sentences.len(), 2);
    assert!(matches!(
        result.sentences[0].terminator,
        index::SentenceTerminator::Punctuation
    ));
}

/// The punctuation is in the recognizer's segment text, not in the word list —
/// the aligner scored `draws`, not `draws.`. A stage that only looked at the
/// word would find no sentence boundaries at all in any real transcript.
#[test]
fn punctuation_is_recovered_from_the_segment_the_aligner_stripped_it_from() {
    let document = two_utterances();
    assert!(!document.words[2].text.as_str().ends_with('.'));
    let result = indexed(&document);
    assert!(matches!(
        result.sentences[0].terminator,
        index::SentenceTerminator::Punctuation
    ));
}

/// A segment whose text and word count disagree describes something other than
/// the words it claims to. Lining them up by position would attach the full
/// stop to whichever word happened to sit at that offset.
#[test]
fn a_segment_that_does_not_match_its_words_yields_no_punctuation() {
    let mut document = two_utterances();
    document.segments[0].text = "The renderer draws a picture.".to_owned();
    let result = indexed(&document);
    assert!(matches!(
        result.sentences[0].terminator,
        index::SentenceTerminator::UtteranceEnd
    ));
}

/// Most spontaneous speech has no terminal punctuation. Dropping those
/// sentences would drop the words.
#[test]
fn an_unpunctuated_utterance_still_produces_a_sentence() {
    let result = indexed(&two_utterances());
    assert_eq!(result.sentences[1].word_count.get(), 2);
    assert!(!matches!(
        result.sentences[1].terminator,
        index::SentenceTerminator::Punctuation
    ));
}

/// "The recording ran out" and "the speaker stopped" are different facts, and
/// only the second is evidence about the speech. The distinguishing question
/// is whether real silence followed the last word.
#[test]
fn a_recording_that_ran_out_is_a_weaker_boundary_than_a_speaker_stopping() {
    let mut document = two_utterances();
    // Half a tick-second of quiet after the last word: the speaker stopped.
    document.coverage.end_ticks = 900_000;
    assert!(matches!(
        indexed(&document).sentences.last().unwrap().terminator,
        index::SentenceTerminator::UtteranceEnd
    ));
    // The file ends almost on the last word: the recording ran out.
    document.coverage.end_ticks = 140_000;
    assert!(matches!(
        indexed(&document).sentences.last().unwrap().terminator,
        index::SentenceTerminator::CoverageEnd
    ));
}

#[test]
fn every_unit_lies_inside_coverage_and_resolves_to_words() {
    let document = two_utterances();
    let result = indexed(&document);
    let words = u64::try_from(document.words.len()).unwrap();
    for utterance in &result.utterances {
        assert!(utterance.start_ticks >= result.coverage.start_ticks);
        assert!(utterance.end_ticks <= result.coverage.end_ticks);
        assert!(utterance.first_word_index + utterance.word_count.get() <= words);
    }
    for sentence in &result.sentences {
        assert!(sentence.start_ticks >= result.coverage.start_ticks);
        assert!(sentence.end_ticks <= result.coverage.end_ticks);
        assert!(sentence.first_word_index + sentence.word_count.get() <= words);
        assert!(sentence.utterance_index < u64::try_from(result.utterances.len()).unwrap());
    }
    for topic in &result.topics {
        assert!(
            topic.first_sentence_index + topic.sentence_count.get()
                <= u64::try_from(result.sentences.len()).unwrap()
        );
    }
}

/// Utterances tile the words: every word belongs to exactly one, in order.
#[test]
fn the_utterances_account_for_every_word_exactly_once() {
    let document = two_utterances();
    let result = indexed(&document);
    let mut next = 0u64;
    for utterance in &result.utterances {
        assert_eq!(utterance.first_word_index, next);
        next += utterance.word_count.get();
    }
    assert_eq!(next, u64::try_from(document.words.len()).unwrap());
}

/// Topics tile the sentences for the same reason.
#[test]
fn the_topics_account_for_every_sentence_exactly_once() {
    let result = indexed(&two_utterances());
    let mut next = 0u64;
    for topic in &result.topics {
        assert_eq!(topic.first_sentence_index, next);
        next += topic.sentence_count.get();
    }
    assert_eq!(next, u64::try_from(result.sentences.len()).unwrap());
}

#[test]
fn a_pause_is_measured_from_the_neighbour_or_from_coverage() {
    let result = indexed(&two_utterances());
    // The first utterance starts at the very beginning, so nothing precedes it.
    assert_eq!(result.utterances[0].pause_before_ticks, 0);
    // Ninety thousand ticks is the one-second gap between the two.
    assert_eq!(result.utterances[0].pause_after_ticks, 90_000);
    assert_eq!(result.utterances[1].pause_before_ticks, 90_000);
    assert_eq!(result.utterances[1].pause_after_ticks, 45_000);
}

#[test]
fn speaking_rate_is_words_over_the_span_they_occupy() {
    let result = indexed(&two_utterances());
    // Three words across 27000 ticks is 0.3 s, which is 600 a minute.
    assert_eq!(result.utterances[0].words_per_minute, 600.0);
}

/// A silence shorter than the threshold is a breath, not a boundary — unless
/// voice activity itself called it a silence, in which case it is.
#[test]
fn voice_activity_can_end_an_utterance_the_threshold_would_not() {
    let mut document = two_utterances();
    document.words[3].start_ticks = 36_000;
    document.words[3].end_ticks = 45_000;
    document.words[4].start_ticks = 45_000;
    document.words[4].end_ticks = 54_000;
    document.segments[1].start_ticks = 36_000;
    document.segments[1].end_ticks = 54_000;
    // Nine thousand ticks of gap, a third of the threshold.
    document.silences = Vec::new();
    assert_eq!(indexed(&document).utterances.len(), 1);
    document.silences = vec![transcript::Interval {
        start_ticks: 27_000,
        end_ticks: 36_000,
    }];
    assert_eq!(indexed(&document).utterances.len(), 2);
}

#[test]
fn a_transcript_nobody_analyzed_is_refused_rather_than_indexed_as_empty() {
    let mut document = two_utterances();
    document.coverage.analyzed = false;
    assert!(matches!(
        build(
            &document,
            None,
            Inputs {
                transcript: TRANSCRIPT_ID,
                shots: None,
            },
            Parameters::DEFAULT,
            "clipmill-evidence-test@0.0.1",
        ),
        Err(super::IndexError::NotAnalyzed)
    ));
}

#[test]
fn the_same_transcript_indexes_to_the_same_document_twice() {
    let document = two_utterances();
    assert_eq!(
        serde_json::to_value(indexed(&document)).unwrap(),
        serde_json::to_value(indexed(&document)).unwrap()
    );
}

#[test]
fn re_tuning_a_parameter_changes_the_document_it_is_recorded_in() {
    // Voice activity found no silence here, so the threshold is the only thing
    // deciding — which is what this test is about.
    let mut document = two_utterances();
    document.silences = Vec::new();
    let coarse = build(
        &document,
        None,
        Inputs {
            transcript: TRANSCRIPT_ID,
            shots: None,
        },
        Parameters {
            utterance_gap_ticks: 180_000,
            ..Parameters::DEFAULT
        },
        "clipmill-evidence-test@0.0.1",
    )
    .expect("the index builds");
    assert_eq!(coarse.utterances.len(), 1, "a longer threshold merges them");
    assert_eq!(coarse.segmentation.utterance_gap_ticks, 180_000);
    assert_ne!(
        coarse.segmentation.utterance_gap_ticks,
        indexed(&document).segmentation.utterance_gap_ticks
    );
}

/// Shot cuts and silences answer one question for the boundary lattice, so
/// they arrive as one ordered list — and an edge outside the analyzed range is
/// an edge nobody may use.
#[test]
fn shot_cuts_join_the_silences_and_coverage_clips_both() {
    let document = two_utterances();
    let shots = shots_at(&[9_000, 63_000, 500_000]);
    let result = build(
        &document,
        Some(&shots),
        Inputs {
            transcript: TRANSCRIPT_ID,
            shots: Some(SHOTS_ID),
        },
        Parameters::DEFAULT,
        "clipmill-evidence-test@0.0.1",
    )
    .expect("the index builds");

    let kinds = result
        .edges
        .iter()
        .map(|edge| (edge.start_ticks, edge.end_ticks, edge.kind))
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            (9_000, 9_000, index::EdgeKind::ShotCut),
            (27_000, 117_000, index::EdgeKind::Silence),
            (63_000, 63_000, index::EdgeKind::ShotCut),
        ],
        "coverage ends at 180000, so the cut at 500000 is not an edge here"
    );
    assert_eq!(
        result
            .inputs
            .shots_artifact_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        Some(SHOTS_ID.to_owned())
    );
}

/// A source with no video has no cuts, and that is a different document from
/// one whose cuts were simply not looked for.
#[test]
fn an_index_without_shots_names_no_shots_artifact() {
    let result = indexed(&two_utterances());
    assert!(result.inputs.shots_artifact_id.is_none());
    assert!(
        result
            .edges
            .iter()
            .all(|edge| matches!(edge.kind, index::EdgeKind::Silence))
    );
}

#[test]
fn shots_of_another_source_are_refused() {
    let mut shots = shots_at(&[9_000]);
    shots.source_fingerprint =
        "sha256:0000000000000000000000000000000000000000000000000000000000000001"
            .parse()
            .expect("a fixture digest");
    assert!(matches!(
        build(
            &two_utterances(),
            Some(&shots),
            Inputs {
                transcript: TRANSCRIPT_ID,
                shots: Some(SHOTS_ID),
            },
            Parameters::DEFAULT,
            "clipmill-evidence-test@0.0.1",
        ),
        Err(super::IndexError::MismatchedSources)
    ));
}

const SHOTS_ID: &str = "sha256:9c0f000000000000000000000000000000000000000000000000000000000031";

fn shots_at(cuts: &[u64]) -> clipmill_contracts::schemas::evidence_shots::EvidenceShots {
    use clipmill_contracts::schemas::evidence_shots as shots;
    shots::EvidenceShots {
        schema_version: serde_json::json!("clipmill.evidence.shots.v1"),
        source_fingerprint: FINGERPRINT.parse().expect("a fixture digest"),
        proxy_artifact_id: SHOTS_ID.parse().expect("a fixture digest"),
        producer: shots::Producer {
            stage: "detect-shots".parse().expect("a stage name"),
            implementation: "clipmill-worker-shots@0.1.0".parse().expect("a name"),
            model_digest: None,
            calibration: None,
        },
        detection: shots::EvidenceShotsDetection {
            threshold: 27.0,
            min_shot_ticks: 45_045,
            analysis_height: 180,
            frame_rate: shots::Timebase {
                num: std::num::NonZeroU64::new(30_000).unwrap_or(std::num::NonZeroU64::MIN),
                den: std::num::NonZeroU64::new(1_001).unwrap_or(std::num::NonZeroU64::MIN),
            },
            decoder: "ffmpeg-8.1.2-btb-n8.1.2".parse().expect("a decoder name"),
            weights: None,
        },
        coverage: shots::Coverage {
            start_ticks: 0,
            end_ticks: 180_000,
            analyzed: true,
            sampling_plan: None,
        },
        cuts: cuts
            .iter()
            .map(|at| shots::Cut {
                t_ticks: *at,
                score: 50.0,
                confidence: shots::Confidence { p50: 0.9, p10: 0.4 },
            })
            .collect(),
        shots: Vec::new(),
        invalid_regions: Vec::new(),
    }
}
