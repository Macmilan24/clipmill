//! Cutting a transcript into the windows a model reads.
//!
//! A model that reads a whole hour at once proposes worse moments than one
//! that reads it a few minutes at a time with the neighbourhood in view, and
//! it cannot cite what it read with any precision. So the index's sentences
//! are cut into windows of about a target size, on sentence boundaries only,
//! each overlapping the last so that a moment on the seam is seen whole by
//! one of them, with a little context on either side that the model may read
//! but not propose from. Where a topic boundary the index found falls near
//! the end of a window, the window ends there instead: a topic is where a
//! moment is likeliest to close.
//!
//! Everything is arithmetic over the index and reproducible from it; the
//! budget reaches the artifact key, so a re-cut is a different reading of the
//! same transcript rather than a correction of this one.

use std::num::NonZeroU64;

use clipmill_contracts::schemas::{
    editorial_windows::{self as doc, EditorialWindows},
    index_transcript::{IndexTranscript, SentenceTerminator},
};

#[derive(Debug, thiserror::Error)]
pub enum WindowsError {
    #[error("the index was built over a transcript that was never analyzed")]
    NotAnalyzed,
    #[error("{field} is not a well-formed content address")]
    MalformedAddress { field: &'static str },
    #[error("the index's {field} does not fit the windows contract: {detail}")]
    Contract { field: &'static str, detail: String },
}

/// The decision parameters, all of which reach the artifact key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Budget {
    /// How many words a window's core aims to hold.
    pub target_words: u64,
    /// At least this many words of one window's tail begin the next.
    pub overlap_words: u64,
    /// Sentences offered as context on each side of the core.
    pub context_sentences: u64,
    /// Extend a core far enough to finish any eligible moment that starts in it.
    /// Zero retains the legacy word-only overlap for old callers.
    pub max_clip_ticks: u64,
}

impl Budget {
    /// A nominal 600-word core plus enough following sentences to finish
    /// a 90-second moment beginning anywhere in that core. Two sentences
    /// around the resulting core provide context rather than clip endpoints.
    pub const DEFAULT: Self = Self {
        target_words: 600,
        overlap_words: 80,
        context_sentences: 2,
        max_clip_ticks: 90 * 90_000,
    };
}

impl Default for Budget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The artifacts the windows were cut from.
#[derive(Clone, Copy, Debug)]
pub struct Inputs<'a> {
    pub index: &'a str,
    pub transcript: &'a str,
}

/// One window as the cut decides it, before it is written down: sentence
/// positions into the index's list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Cut {
    first: usize,
    count: usize,
}

/// Where each window's core begins and ends, over sentence word counts.
///
/// Greedy from the front: a window takes sentences until the next would push
/// it past the target, ends early at a topic boundary in its last third, and
/// the next window starts far enough back to overlap by the budget — but
/// always at least one sentence further on than this one started, so the
/// cut makes progress on any input.
fn cut(word_counts: &[u64], topic_starts: &[usize], budget: Budget) -> Vec<Cut> {
    let mut cuts = Vec::new();
    let mut first = 0;
    while first < word_counts.len() {
        // Fill to the target; a single sentence past it still forms a window.
        let mut end = first + 1;
        let mut words = word_counts[first];
        while end < word_counts.len() && words + word_counts[end] <= budget.target_words {
            words += word_counts[end];
            end += 1;
        }
        // A topic that opens inside the window's last third closes the window
        // before it, when what remains before the topic is still a fair
        // window on its own.
        let filled: u64 = word_counts[first..end].iter().sum();
        if let Some(boundary) = topic_starts
            .iter()
            .copied()
            .filter(|start| *start > first && *start < end)
            .find(|start| {
                let before: u64 = word_counts[first..*start].iter().sum();
                before * 3 >= filled * 2
            })
        {
            end = boundary;
        }
        cuts.push(Cut {
            first,
            count: end - first,
        });
        if end >= word_counts.len() {
            break;
        }
        // The next window begins inside this one's tail: back up over whole
        // sentences until the overlap is covered, never past this window's
        // first sentence and never onto it.
        let mut next = end;
        let mut overlap = 0;
        while next > first + 1 && overlap < budget.overlap_words {
            next -= 1;
            overlap += word_counts[next];
        }
        first = next;
    }
    cuts
}

/// Extend nominal word-sized cores by at most one clip-duration tail.
///
/// Each sentence begins in a nominal core. Including every legal end for that
/// core's final start also includes every legal end for all its earlier starts.
/// This covers sparse dialogue and fast speech without pretending a fixed word
/// overlap is a fixed duration. Nominal cores still advance by the word budget;
/// we do not create one model call per sentence when speech is unusually dense.
fn cover_duration(cuts: Vec<Cut>, times: &[(u64, u64)], max_ticks: u64) -> Vec<Cut> {
    if max_ticks == 0 {
        return cuts;
    }
    let mut extended: Vec<Cut> = Vec::new();
    for cut in cuts {
        let nominal_end = cut.first + cut.count;
        let latest_start = times[nominal_end - 1].0;
        let mut end = nominal_end;
        while end < times.len() && times[end].1.saturating_sub(latest_start) <= max_ticks {
            end += 1;
        }
        // An already complete tail needs no second call over a subset of it.
        if extended
            .last()
            .is_some_and(|previous| previous.first + previous.count >= end)
        {
            continue;
        }
        extended.push(Cut {
            first: cut.first,
            count: end - cut.first,
        });
    }
    extended
}

/// Cut the index into windows.
///
/// The index is the authority for every sentence and topic here; nothing is
/// invented that a consumer could not derive from it. An index with no
/// sentences cuts into no windows, which is the honest state of a recording
/// nobody spoke in.
pub fn windows(
    index: &IndexTranscript,
    inputs: Inputs<'_>,
    budget: Budget,
    implementation: &str,
) -> Result<EditorialWindows, WindowsError> {
    if !index.coverage.analyzed {
        return Err(WindowsError::NotAnalyzed);
    }
    let sentences = sentences_of(index)?;
    let word_counts: Vec<u64> = sentences
        .iter()
        .map(|sentence| sentence.word_count.get())
        .collect();
    let topic_starts: Vec<usize> = index
        .topics
        .iter()
        .filter_map(|topic| usize::try_from(topic.first_sentence_index).ok())
        .collect();
    let times: Vec<_> = sentences
        .iter()
        .map(|sentence| (sentence.start_ticks, sentence.end_ticks))
        .collect();
    let windows = cover_duration(
        cut(&word_counts, &topic_starts, budget),
        &times,
        budget.max_clip_ticks,
    )
    .into_iter()
    .enumerate()
    .map(|(position, cut)| window_of(index, &sentences, budget, position, cut))
    .collect();

    Ok(EditorialWindows {
        content_profile: doc::EditorialWindowsContentProfile::Interview,
        schema_version: serde_json::json!("clipmill.editorial.windows.v1"),
        source_fingerprint: address(index.source_fingerprint.as_str(), "source_fingerprint")?,
        inputs: doc::EditorialWindowsInputs {
            index_artifact_id: address(inputs.index, "index_artifact_id")?,
            transcript_artifact_id: address(inputs.transcript, "transcript_artifact_id")?,
        },
        producer: doc::Producer {
            stage: crate::STAGE.parse().map_err(|_| WindowsError::Contract {
                field: "producer stage",
                detail: "empty".to_owned(),
            })?,
            implementation: implementation.parse().map_err(|_| WindowsError::Contract {
                field: "producer implementation",
                detail: "empty".to_owned(),
            })?,
        },
        language: index
            .language
            .as_str()
            .parse()
            .map_err(|_| WindowsError::Contract {
                field: "language",
                detail: index.language.to_string(),
            })?,
        budget: doc::EditorialWindowsBudget {
            target_words: non_zero_u64(budget.target_words.max(1)),
            overlap_words: budget.overlap_words,
            context_sentences: budget.context_sentences,
            max_clip_ticks: budget.max_clip_ticks,
        },
        coverage: doc::Coverage {
            start_ticks: index.coverage.start_ticks,
            end_ticks: index.coverage.end_ticks,
            analyzed: index.coverage.analyzed,
        },
        sentences,
        windows,
        outline: outline_of(index)?,
        invalid_regions: invalid_regions_of(index),
    })
}

/// The index's sentences, restated in the windows document's own terms.
fn sentences_of(index: &IndexTranscript) -> Result<Vec<doc::Sentence>, WindowsError> {
    index
        .sentences
        .iter()
        .map(|sentence| {
            Ok(doc::Sentence {
                index: sentence.index,
                start_ticks: sentence.start_ticks,
                end_ticks: sentence.end_ticks,
                first_word_index: sentence.first_word_index,
                word_count: sentence.word_count,
                text: sentence
                    .text
                    .as_str()
                    .parse()
                    .map_err(|_| WindowsError::Contract {
                        field: "sentence text",
                        detail: format!("sentence {} has no text", sentence.index),
                    })?,
                terminator: match sentence.terminator {
                    SentenceTerminator::Punctuation => doc::SentenceTerminator::Punctuation,
                    SentenceTerminator::UtteranceEnd => doc::SentenceTerminator::UtteranceEnd,
                    SentenceTerminator::CoverageEnd => doc::SentenceTerminator::CoverageEnd,
                },
            })
        })
        .collect()
}

/// One window written down: its core's ticks and words, the topics the core
/// touches, and the context on either side — as much as the budget allows
/// and as much as the recording has.
fn window_of(
    index: &IndexTranscript,
    sentences: &[doc::Sentence],
    budget: Budget,
    position: usize,
    cut: Cut,
) -> doc::Window {
    let last = cut.first + cut.count;
    let core = &sentences[cut.first..last];
    let context = usize::try_from(budget.context_sentences).unwrap_or(usize::MAX);
    let topic_indexes = index
        .topics
        .iter()
        .filter(|topic| {
            let start = usize::try_from(topic.first_sentence_index).unwrap_or(usize::MAX);
            let end = start
                .saturating_add(usize::try_from(topic.sentence_count.get()).unwrap_or(usize::MAX));
            start < last && end > cut.first
        })
        .map(|topic| topic.index)
        .collect();
    doc::Window {
        index: as_u64(position),
        first_sentence_index: as_u64(cut.first),
        sentence_count: non_zero(cut.count),
        context_before_sentences: as_u64(cut.first.min(context)),
        context_after_sentences: as_u64((sentences.len() - last).min(context)),
        start_ticks: core[0].start_ticks,
        end_ticks: core[core.len() - 1].end_ticks,
        word_count: non_zero_u64(core.iter().map(|sentence| sentence.word_count.get()).sum()),
        topic_indexes,
    }
}

/// The outline: one line per topic the index found, with its keywords.
fn outline_of(index: &IndexTranscript) -> Result<Vec<doc::OutlineLine>, WindowsError> {
    index
        .topics
        .iter()
        .map(|topic| {
            Ok(doc::OutlineLine {
                topic_index: topic.index,
                first_sentence_index: topic.first_sentence_index,
                sentence_count: topic.sentence_count,
                start_ticks: topic.start_ticks,
                end_ticks: topic.end_ticks,
                keywords: topic
                    .keywords
                    .iter()
                    .map(|keyword| {
                        keyword
                            .term
                            .as_str()
                            .parse()
                            .map_err(|_| WindowsError::Contract {
                                field: "topic keyword",
                                detail: format!("topic {} has an empty keyword", topic.index),
                            })
                    })
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect()
}

/// The regions the index warned about, carried through so the validator can
/// refuse a proposal that lands on one without opening the index.
fn invalid_regions_of(index: &IndexTranscript) -> Vec<doc::InvalidRegion> {
    use clipmill_contracts::schemas::index_transcript::InvalidRegionReason as Theirs;
    index
        .invalid_regions
        .iter()
        .map(|region| doc::InvalidRegion {
            start_ticks: region.start_ticks,
            end_ticks: region.end_ticks,
            reason: match region.reason {
                Theirs::NotAnalyzed => doc::InvalidRegionReason::NotAnalyzed,
                Theirs::NoAudio => doc::InvalidRegionReason::NoAudio,
                Theirs::DecodeFailed => doc::InvalidRegionReason::DecodeFailed,
                Theirs::AlignmentUnavailable => doc::InvalidRegionReason::AlignmentUnavailable,
                Theirs::TimingInterpolated => doc::InvalidRegionReason::TimingInterpolated,
            },
            detail: region
                .detail
                .as_ref()
                .and_then(|detail| detail.as_str().parse().ok()),
        })
        .collect()
}

fn address(value: &str, field: &'static str) -> Result<doc::Sha256, WindowsError> {
    value
        .parse()
        .map_err(|_| WindowsError::MalformedAddress { field })
}

fn as_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn non_zero(value: usize) -> NonZeroU64 {
    non_zero_u64(as_u64(value))
}

fn non_zero_u64(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap_or(NonZeroU64::MIN)
}

#[cfg(test)]
mod tests {
    use super::{Budget, Cut, cover_duration, cut};

    fn cuts(
        word_counts: &[u64],
        topics: &[usize],
        target: u64,
        overlap: u64,
    ) -> Vec<(usize, usize)> {
        cut(
            word_counts,
            topics,
            Budget {
                target_words: target,
                overlap_words: overlap,
                context_sentences: 2,
                max_clip_ticks: 0,
            },
        )
        .into_iter()
        .map(|Cut { first, count }| (first, count))
        .collect()
    }

    #[test]
    fn nothing_cuts_into_nothing() {
        assert!(cuts(&[], &[], 100, 10).is_empty());
    }

    #[test]
    fn a_short_recording_is_one_window() {
        assert_eq!(cuts(&[10, 20, 30], &[], 100, 10), [(0, 3)]);
    }

    #[test]
    fn windows_fill_to_the_target_and_overlap_by_whole_sentences() {
        // Ten sentences of ten words, target thirty, overlap ten: each window
        // holds three sentences and the next begins on the last of them.
        let counts = [10; 10];
        assert_eq!(
            cuts(&counts, &[], 30, 10),
            [(0, 3), (2, 3), (4, 3), (6, 3), (8, 2)]
        );
    }

    #[test]
    fn a_sentence_longer_than_the_target_still_gets_a_window() {
        assert_eq!(cuts(&[5, 500, 5], &[], 100, 10), [(0, 1), (1, 1), (2, 1)]);
    }

    #[test]
    fn a_topic_opening_in_the_last_third_closes_the_window_before_it() {
        // Six sentences of ten words, target sixty would take all six; a topic
        // opening at sentence four, with forty words before it (two thirds of
        // sixty), ends the first window there.
        assert_eq!(cuts(&[10; 6], &[4], 60, 10), [(0, 4), (3, 3)]);
        // A topic too early in the window — only one sentence before it —
        // does not: a window of one sentence is not a fair window.
        assert_eq!(cuts(&[10; 6], &[1], 60, 10), [(0, 6)]);
    }

    #[test]
    fn the_cut_always_makes_progress() {
        // An overlap wider than a window cannot walk backwards: the next
        // window begins at least one sentence on.
        let counts = [10; 5];
        let found = cuts(&counts, &[], 20, 1_000);
        for pair in found.windows(2) {
            assert!(pair[1].0 > pair[0].0, "{found:?}");
        }
        assert_eq!(found.last().map(|(first, count)| first + count), Some(5));
    }
    #[test]
    fn a_seventy_second_moment_crossing_a_word_core_seam_is_proposable() {
        let counts = [100; 30];
        let times: Vec<_> = (0..30)
            .map(|i| (i * 10 * 90_000, (i + 1) * 10 * 90_000))
            .collect();
        let nominal = cut(&counts, &[], Budget::DEFAULT);
        assert!(
            !nominal
                .iter()
                .any(|c| c.first <= 4 && c.first + c.count > 10)
        );
        let covered = cover_duration(nominal, &times, 90 * 90_000);
        assert!(
            covered
                .iter()
                .any(|c| c.first <= 4 && c.first + c.count > 10)
        );
        assert_eq!(times[10].1 - times[4].0, 70 * 90_000);
    }

    #[test]
    fn every_duration_valid_sentence_span_fits_for_sparse_and_dense_dialogue() {
        for seconds_per_sentence in [1, 3, 10, 40] {
            let counts = [30; 180];
            let times: Vec<_> = (0..180)
                .map(|i| {
                    (
                        i * seconds_per_sentence * 90_000,
                        (i + 1) * seconds_per_sentence * 90_000 - 1,
                    )
                })
                .collect();
            let budget = Budget {
                target_words: 600,
                overlap_words: 80,
                context_sentences: 2,
                max_clip_ticks: 90 * 90_000,
            };
            let nominal = cut(&counts, &[18, 39, 60], budget);
            let nominal_count = nominal.len();
            let covered = cover_duration(nominal, &times, budget.max_clip_ticks);
            assert!(
                covered.len() <= nominal_count,
                "duration coverage does not multiply model calls"
            );
            for first in 0..times.len() {
                for last in first..times.len() {
                    let duration = times[last].1 - times[first].0;
                    if (20 * 90_000..=90 * 90_000).contains(&duration) {
                        assert!(
                            covered
                                .iter()
                                .any(|c| c.first <= first && c.first + c.count > last),
                            "missing {first}:{last} at density {seconds_per_sentence}"
                        );
                    }
                }
            }
        }
    }
}
