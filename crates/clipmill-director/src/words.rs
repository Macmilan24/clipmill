//! Where a person may cut: at any edge between words, and never through one.
//!
//! The boundary lattice is the search's shortlist. It chose from it knowing
//! things about hooks and payoffs, and it is still the cut the director builds
//! when nobody moved anything. A person reviewing the clip may know better —
//! the sentence before is the setup, the last line is a tangent — so a hand-set
//! cut may land between any two words the recording has (R63).
//!
//! The one cut nobody may make is through a word, because that is the cut a
//! viewer hears. An edge that lands inside a word is moved out to that word's
//! own edge, keeping the whole word; an edge in the gap between two words stays
//! exactly where it was put.

use clipmill_contracts::schemas::speech_transcript::SpeechTranscript;

use crate::lattice::Boundary;

/// An edge of a cut that falls inside a word, and where it would keep it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Severed {
    /// `"start"` or `"end"`.
    pub edge: &'static str,
    pub word: String,
    /// The word's own edge on the side the cut is on: its start for a clip's
    /// start, its end for a clip's end.
    pub keep_at: i64,
}

/// The first edge of `boundary` that cuts through a word, if any.
///
/// The start is checked before the end, so a cut that severs two words is
/// reported by the one a person reads first.
pub fn severed(transcript: &SpeechTranscript, boundary: Boundary) -> Option<Severed> {
    let edges = [("start", boundary.start_ticks), ("end", boundary.end_ticks)];
    for (edge, tick) in edges {
        for word in &transcript.words {
            let (from, to) = span(word.start_ticks, word.end_ticks);
            if from < tick && tick < to {
                return Some(Severed {
                    edge,
                    word: word.text.to_string(),
                    keep_at: if edge == "start" { from } else { to },
                });
            }
        }
    }
    None
}

/// Move each edge that cuts through a word out to that word's edge.
///
/// Outward, never inward: a start inside a word moves to the word's start and
/// an end inside a word moves to its end, so the word is kept rather than
/// dropped. Repeated until nothing moves, because words whose timing was
/// interpolated can overlap, and leaving one word can land an edge inside the
/// one before it.
pub fn keep_whole_words(transcript: &SpeechTranscript, wanted: Boundary) -> Boundary {
    let mut kept = wanted;
    // Each pass either moves an edge strictly outward or stops, and an edge can
    // only leave each word once, so the word count bounds the passes.
    for _ in 0..=transcript.words.len() {
        let mut moved = false;
        for word in &transcript.words {
            let (from, to) = span(word.start_ticks, word.end_ticks);
            if from < kept.start_ticks && kept.start_ticks < to {
                kept.start_ticks = from;
                moved = true;
            }
            if from < kept.end_ticks && kept.end_ticks < to {
                kept.end_ticks = to;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    kept
}

fn span(start: u64, end: u64) -> (i64, i64) {
    (
        i64::try_from(start).unwrap_or(i64::MAX),
        i64::try_from(end).unwrap_or(i64::MAX),
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use clipmill_contracts::schemas::speech_transcript::SpeechTranscript;
    use serde_json::json;

    use super::{Severed, keep_whole_words, severed};
    use crate::lattice::Boundary;

    const SECOND: i64 = 90_000;

    /// Three words: `one` 1.0–1.5 s, `two` 2.0–2.5 s, and `three` 2.4–3.0 s,
    /// which overlaps `two` the way interpolated timing can.
    fn transcript() -> SpeechTranscript {
        let words: Vec<_> = [
            ("one", 90_000, 135_000),
            ("two", 180_000, 225_000),
            ("three", 216_000, 270_000),
        ]
        .iter()
        .enumerate()
        .map(|(index, (text, start, end))| {
            json!({
                "index": index, "segment_index": 0, "text": text,
                "start_ticks": start, "end_ticks": end,
                "confidence": { "p50": 0.9, "p10": 0.8 }, "timing": "aligned",
            })
        })
        .collect();
        serde_json::from_value(json!({
            "schema_version": "clipmill.speech.transcript.v1",
            "source_fingerprint": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "inputs": {
                "vad_artifact_id": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
                "asr_artifact_id": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
                "alignment_artifact_id": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
            },
            "producers": [{ "stage": "transcribe-source", "implementation": "test@1" }],
            "language": "en",
            "confidence": { "p50": 0.9, "p10": 0.8 },
            "coverage": {
                "start_ticks": 0, "end_ticks": 10 * SECOND, "analyzed": true,
                "speech_ticks": SECOND, "aligned_ticks": SECOND, "sampling_plan": "full",
            },
            "words": words,
            "segments": [],
            "silences": [],
            "invalid_regions": [],
        }))
        .expect("a transcript")
    }

    #[test]
    fn an_edge_between_words_stays_where_it_was_put() {
        let wanted = Boundary {
            start_ticks: 160_000,
            end_ticks: 150_000 + SECOND * 2,
        };
        let transcript = transcript();
        assert_eq!(keep_whole_words(&transcript, wanted), wanted);
        assert_eq!(severed(&transcript, wanted), None);
    }

    #[test]
    fn an_edge_inside_a_word_moves_out_to_keep_the_word() {
        let transcript = transcript();
        let kept = keep_whole_words(
            &transcript,
            Boundary {
                start_ticks: 100_000,
                end_ticks: 200_000,
            },
        );
        assert_eq!(kept.start_ticks, 90_000, "the start keeps all of `one`");
        // `two` ends at 225000, but `three` overlaps it, so the end moves on
        // until it is inside no word at all.
        assert_eq!(
            kept.end_ticks, 270_000,
            "the end keeps all of `two` and `three`"
        );
        assert_eq!(severed(&transcript, kept), None);
    }

    #[test]
    fn a_severed_edge_is_reported_with_the_word_and_where_to_keep_it() {
        let transcript = transcript();
        assert_eq!(
            severed(
                &transcript,
                Boundary {
                    start_ticks: 100_000,
                    end_ticks: 150_000,
                },
            ),
            Some(Severed {
                edge: "start",
                word: "one".to_owned(),
                keep_at: 90_000,
            })
        );
        assert_eq!(
            severed(
                &transcript,
                Boundary {
                    start_ticks: 150_000,
                    end_ticks: 200_000,
                },
            ),
            Some(Severed {
                edge: "end",
                word: "two".to_owned(),
                keep_at: 225_000,
            })
        );
    }
}
