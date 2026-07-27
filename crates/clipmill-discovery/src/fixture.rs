//! Index and transcript documents built by hand, shared by every test here.
//!
//! Real recordings do not contain a topic with one sentence, a question nobody
//! answered, or a shot cut landing exactly inside a word. Those are the cases
//! that decide whether a lattice is legal and whether a nomination is honest,
//! so they are written out rather than waited for.
#![allow(clippy::expect_used)]

use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::speech_transcript as transcript;

pub(crate) const FINGERPRINT: &str =
    "sha256:31ab000000000000000000000000000000000000000000000000000000000007";
pub(crate) const INDEX_ID: &str =
    "sha256:1de0000000000000000000000000000000000000000000000000000000000011";
pub(crate) const TRANSCRIPT_ID: &str =
    "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
pub(crate) const IMPLEMENTATION: &str = "clipmill-discovery-test@0.0.1";

/// One second, in ticks.
pub(crate) const SECOND: u64 = 90_000;

fn confidence() -> index::Confidence {
    index::Confidence { p50: 0.9, p10: 0.8 }
}

fn nonzero(value: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(value).expect("a count of at least one")
}

/// A sentence occupying `[start, end)` with a word range and some text.
pub(crate) fn sentence(
    at: usize,
    utterance: usize,
    words: (u64, u64),
    span: (u64, u64),
    text: &str,
) -> index::Sentence {
    index::Sentence {
        index: at as u64,
        utterance_index: utterance as u64,
        start_ticks: span.0,
        end_ticks: span.1,
        first_word_index: words.0,
        word_count: nonzero(words.1),
        text: text.parse().expect("a sentence is never empty"),
        terminator: if text.trim_end().ends_with(['.', '?', '!']) {
            index::SentenceTerminator::Punctuation
        } else {
            index::SentenceTerminator::UtteranceEnd
        },
        words_per_minute: 150.0,
        confidence: confidence(),
    }
}

pub(crate) fn utterance(
    at: usize,
    words: (u64, u64),
    span: (u64, u64),
    text: &str,
) -> index::Utterance {
    index::Utterance {
        index: at as u64,
        start_ticks: span.0,
        end_ticks: span.1,
        first_word_index: words.0,
        word_count: nonzero(words.1),
        text: text.parse().expect("an utterance is never empty"),
        pause_before_ticks: 0,
        pause_after_ticks: 0,
        words_per_minute: 150.0,
        confidence: confidence(),
    }
}

pub(crate) fn topic(
    at: usize,
    sentences: (u64, u64),
    span: (u64, u64),
    depth: f64,
) -> index::Topic {
    index::Topic {
        index: at as u64,
        start_ticks: span.0,
        end_ticks: span.1,
        first_sentence_index: sentences.0,
        sentence_count: nonzero(sentences.1),
        opening_depth: depth,
        keywords: Vec::new(),
    }
}

pub(crate) fn shot_cut(at: u64) -> index::Edge {
    index::Edge {
        start_ticks: at,
        end_ticks: at,
        kind: index::EdgeKind::ShotCut,
    }
}

pub(crate) fn silence(span: (u64, u64)) -> index::Edge {
    index::Edge {
        start_ticks: span.0,
        end_ticks: span.1,
        kind: index::EdgeKind::Silence,
    }
}

pub(crate) fn word(at: u64, span: (u64, u64), text: &str) -> transcript::Word {
    transcript::Word {
        index: at,
        segment_index: 0,
        text: text.parse().expect("a word is never empty"),
        start_ticks: span.0,
        end_ticks: span.1,
        confidence: transcript::Confidence { p50: 0.9, p10: 0.8 },
        timing: transcript::WordTiming::Aligned,
    }
}

pub(crate) fn indexed(
    sentences: Vec<index::Sentence>,
    utterances: Vec<index::Utterance>,
    topics: Vec<index::Topic>,
    edges: Vec<index::Edge>,
    coverage: (u64, u64),
) -> index::IndexTranscript {
    index::IndexTranscript {
        schema_version: serde_json::json!("clipmill.index.transcript.v1"),
        source_fingerprint: FINGERPRINT.parse().expect("a fixture digest"),
        inputs: index::IndexTranscriptInputs {
            transcript_artifact_id: TRANSCRIPT_ID.parse().expect("a fixture digest"),
            shots_artifact_id: None,
        },
        producer: index::Producer {
            stage: "index-transcript".parse().expect("a stage name"),
            implementation: "clipmill-evidence-index@1.0.0".parse().expect("a name"),
        },
        language: "en".parse().expect("a language tag"),
        segmentation: index::IndexTranscriptSegmentation {
            utterance_gap_ticks: 27_000,
            block_sentences: nonzero(2),
            boundary_cutoff: 0.5,
            stopwords: "english-minimal.v1".parse().expect("a list name"),
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
        invalid_regions: Vec::new(),
    }
}

pub(crate) fn spoken(
    words: Vec<transcript::Word>,
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
        segments: Vec::new(),
        silences: Vec::new(),
        invalid_regions: Vec::new(),
    }
}

/// A four-minute interview: two topics, a question with an answer, and a quote.
///
/// Long enough that the fifteen-second floor and the three-minute ceiling both
/// bite somewhere, which is what makes it useful for the lattice tests.
pub(crate) struct Interview {
    pub index: index::IndexTranscript,
    pub transcript: transcript::SpeechTranscript,
}

pub(crate) fn interview() -> Interview {
    // Two topics, each a question followed by four sentences of answer. Laid
    // out end to end with a conversational gap inside a topic and a real pause
    // between them, because the answer window closes on a pause and a fixture
    // whose sentences sat ten seconds apart would test nothing but the break.
    const WITHIN: u64 = SECOND * 2 / 5;
    const BETWEEN: u64 = SECOND * 4;
    let lines: &[&str] = &[
        "What makes a renderer deterministic?",
        "The encoder settings are pinned to one profile.",
        "Every frame lands on the same tick on every machine.",
        "Nobody can reproduce a build that drifts.",
        "That is the whole reason the profile is frozen.",
        "How does alignment measure word timing?",
        "A CTC model scores each frame against the text.",
        "The peaks become word boundaries in ticks.",
        "Interpolated timing is labelled and never presented as measured.",
        "Precision here is what stops a cut landing inside a word.",
    ];
    let mut sentences = Vec::new();
    let mut utterances = Vec::new();
    let mut words = Vec::new();
    let mut next_word = 0u64;
    let mut at = 0u64;
    for (position, text) in lines.iter().enumerate() {
        if position > 0 {
            at += if position == 5 { BETWEEN } else { WITHIN };
        }
        let tokens = text.split_whitespace().count() as u64;
        let start = at;
        let end = start + tokens * (SECOND / 2);
        sentences.push(sentence(
            position,
            position,
            (next_word, tokens),
            (start, end),
            text,
        ));
        utterances.push(utterance(position, (next_word, tokens), (start, end), text));
        for (offset, token) in text.split_whitespace().enumerate() {
            let token_at = start + offset as u64 * (SECOND / 2);
            words.push(word(
                next_word + offset as u64,
                (token_at, token_at + SECOND / 2),
                token,
            ));
        }
        next_word += tokens;
        at = end;
    }
    let coverage = (0, at + SECOND);
    let topics = vec![
        topic(0, (0, 5), (0, sentences[4].end_ticks), 0.0),
        topic(
            1,
            (5, 5),
            (sentences[5].start_ticks, sentences[9].end_ticks),
            0.8,
        ),
    ];
    let edges = vec![
        silence((sentences[4].end_ticks, sentences[5].start_ticks)),
        // Lands in the pause between the topics, so it is a legal boundary.
        shot_cut(sentences[4].end_ticks + BETWEEN / 2),
    ];
    Interview {
        index: indexed(sentences, utterances, topics, edges, coverage),
        transcript: spoken(words, coverage),
    }
}
