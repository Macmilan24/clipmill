//! L1: the structure the recording states about itself.
//!
//! Nothing here is inferred. An utterance is where voice activity heard a
//! pause; a sentence is where the recognizer put a full stop, or where the
//! speaker stopped; an edge is a silence somebody measured or a cut somebody
//! detected. That is the whole level, and it is deliberately the whole level —
//! everything that requires reading meaning into the words belongs to L2,
//! where it can be labelled as the approximation it is.
//!
//! The one subtlety is punctuation. Forced alignment scores acoustic tokens,
//! so the aligner returns `tick` where the recognizer wrote `tick.` — the
//! transcript's word list has had the punctuation stripped out of it. The full
//! stop still exists, in the recognizer segment's own text, and pairing the
//! two back up is what lets a sentence boundary be an observation rather than
//! a guess. Where the two disagree about how many tokens a segment holds, that
//! pairing is refused rather than approximated, and the utterance's end
//! carries the boundary instead.

use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::speech_transcript as transcript;

use crate::confidence::distribution;

/// Ticks per second, the timebase every interval in the system uses (D06).
pub(crate) const TICKS_PER_SECOND: u64 = 90_000;

/// A run of words between two pauses, as indices into the transcript.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Run {
    pub first: usize,
    pub count: usize,
}

impl Run {
    pub(crate) fn last(self) -> usize {
        self.first + self.count - 1
    }
}

/// Where the speaker stopped.
///
/// A boundary is taken where the gap to the next word is at least
/// `utterance_gap_ticks`, or where voice activity reported a silence starting
/// inside that gap. The second clause matters when the detector was tuned
/// finer than this stage: a pause the recording's own analysis called a
/// silence is a pause, whatever threshold is written here.
pub(crate) fn utterance_runs(
    words: &[transcript::Word],
    silences: &[transcript::Interval],
    gap: u64,
) -> Vec<Run> {
    if words.is_empty() {
        return Vec::new();
    }
    let mut runs = Vec::new();
    let mut first = 0usize;
    for position in 0..words.len() - 1 {
        let here = words[position].end_ticks;
        let next = words[position + 1].start_ticks;
        let quiet = next.saturating_sub(here) >= gap
            || silences
                .iter()
                .any(|silence| silence.start_ticks >= here && silence.start_ticks < next);
        if quiet {
            runs.push(Run {
                first,
                count: position + 1 - first,
            });
            first = position + 1;
        }
    }
    runs.push(Run {
        first,
        count: words.len() - first,
    });
    runs
}

/// The characters a recognizer ends a sentence with.
const TERMINATORS: [char; 4] = ['.', '!', '?', '…'];

/// Whether the word at this transcript index was punctuated as a sentence end.
///
/// Two places are consulted, in order: the word's own text, in case a producer
/// kept the punctuation, and then the recognizer segment's text at the same
/// offset. The segment is only trusted where its whitespace token count equals
/// its declared word count — otherwise the two are describing different things
/// and lining them up by position would attach a full stop to whichever word
/// happened to sit at that offset.
pub(crate) fn is_sentence_end(
    words: &[transcript::Word],
    segments: &[transcript::Segment],
    at: usize,
) -> bool {
    let Some(word) = words.get(at) else {
        return false;
    };
    if ends_a_sentence(word.text.as_str()) {
        return true;
    }
    let Some(segment) = segments
        .iter()
        .find(|segment| segment.index == word.segment_index)
    else {
        return false;
    };
    let tokens = segment.text.split_whitespace().collect::<Vec<_>>();
    let count = usize::try_from(segment.word_count).unwrap_or(usize::MAX);
    if tokens.len() != count {
        return false;
    }
    let Ok(first) = usize::try_from(segment.first_word_index) else {
        return false;
    };
    let Some(offset) = at.checked_sub(first) else {
        return false;
    };
    tokens.get(offset).copied().is_some_and(ends_a_sentence)
}

fn ends_a_sentence(text: &str) -> bool {
    text.trim_end_matches(['"', '\'', ')', ']', '»', '”'])
        .chars()
        .next_back()
        .is_some_and(|last| TERMINATORS.contains(&last))
}

/// The sentences inside one utterance, and how each of them ended.
///
/// Every utterance yields at least one sentence: a speaker who stopped without
/// punctuating still finished saying something, and dropping it would lose the
/// words. The last one is labelled by how the recording ended rather than by
/// how the sentence did, which is the honest reading.
pub(crate) fn sentence_runs(
    words: &[transcript::Word],
    segments: &[transcript::Segment],
    utterance: Run,
) -> Vec<(Run, index::SentenceTerminator)> {
    let mut sentences = Vec::new();
    let mut first = utterance.first;
    for position in utterance.first..=utterance.last() {
        let punctuated = is_sentence_end(words, segments, position);
        let last_in_utterance = position == utterance.last();
        if !punctuated && !last_in_utterance {
            continue;
        }
        sentences.push((
            Run {
                first,
                count: position + 1 - first,
            },
            if punctuated {
                index::SentenceTerminator::Punctuation
            } else {
                index::SentenceTerminator::UtteranceEnd
            },
        ));
        first = position + 1;
    }
    sentences
}

/// Where a clip may start or end without severing anything.
///
/// Silences and shot cuts land in one list because the boundary lattice asks
/// one question of them. Both are clipped to the analyzed range: an edge
/// outside coverage is an edge nobody may use, and publishing it would invite
/// a consumer to.
pub(crate) fn edges(
    silences: &[transcript::Interval],
    cuts: &[u64],
    coverage: (u64, u64),
) -> Vec<index::Edge> {
    let (start, end) = coverage;
    let mut edges = Vec::new();
    for silence in silences {
        let low = silence.start_ticks.max(start);
        let high = silence.end_ticks.min(end);
        if low <= high {
            edges.push(index::Edge {
                start_ticks: low,
                end_ticks: high,
                kind: index::EdgeKind::Silence,
            });
        }
    }
    for cut in cuts {
        if *cut >= start && *cut <= end {
            edges.push(index::Edge {
                start_ticks: *cut,
                end_ticks: *cut,
                kind: index::EdgeKind::ShotCut,
            });
        }
    }
    edges.sort_by_key(|edge| {
        (
            edge.start_ticks,
            edge.end_ticks,
            matches!(edge.kind, index::EdgeKind::ShotCut),
        )
    });
    edges.dedup_by(|left, right| {
        left.start_ticks == right.start_ticks
            && left.end_ticks == right.end_ticks
            && left.kind == right.kind
    });
    edges
}

/// Words per minute over a span, or zero for a span with no duration.
///
/// A rate rather than a time quantity, which is why it is allowed to be a
/// float at all: it cannot express a position on a timeline, so no two stages
/// can disagree about where something happened because of it.
pub(crate) fn words_per_minute(word_count: usize, start: u64, end: u64) -> f64 {
    let span = end.saturating_sub(start);
    if span == 0 {
        return 0.0;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "counts and tick spans well inside f64's exact integer range"
    )]
    let rate = (word_count as f64) * 60.0 * (TICKS_PER_SECOND as f64) / (span as f64);
    rate
}

/// The joined text of a run of words.
pub(crate) fn text_of(words: &[transcript::Word], run: Run) -> String {
    words[run.first..=run.last()]
        .iter()
        .map(|word| word.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The confidence of a run of words, by the shared quantile.
pub(crate) fn confidence_of(words: &[transcript::Word], run: Run) -> index::Confidence {
    let values = words[run.first..=run.last()]
        .iter()
        .map(|word| word.confidence.p50)
        .collect::<Vec<_>>();
    let low = words[run.first..=run.last()]
        .iter()
        .map(|word| word.confidence.p10)
        .collect::<Vec<_>>();
    let (p50, _) = distribution(&values);
    let (_, p10) = distribution(&low);
    index::Confidence { p50, p10 }
}
