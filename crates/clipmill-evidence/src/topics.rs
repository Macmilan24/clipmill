//! L2: where the vocabulary changed.
//!
//! This is the honest P1 approximation of "scenes and topics", and the honesty
//! is the point. No model reads these words. What is measured is lexical
//! cohesion — whether the sentences on either side of a gap draw on the same
//! vocabulary — by the method Hearst called `TextTiling`: score every gap, find
//! the valleys, and keep the ones deep enough to be worth calling a boundary.
//!
//! That finds where the words changed. Usually the subject changed too, which
//! is why it is useful; sometimes it did not, which is why the published unit
//! is described as a lexical neighbourhood rather than as a topic the system
//! understood. A stage that claimed comprehension here would be making a
//! promise the next stage would then rely on.
//!
//! Everything is ordered explicitly. Token counts live in a `BTreeMap`, ties
//! break alphabetically, and the greedy boundary selection sorts by depth with
//! the gap index as the tiebreak — because a topic list that depended on hash
//! iteration order would be a different document on every run.

use std::collections::{BTreeMap, BTreeSet};

use crate::stopwords;

/// Counted content words for one sentence.
pub(crate) type Counts = BTreeMap<String, u64>;

/// One run of sentences that share vocabulary.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Topic {
    pub first_sentence: usize,
    pub sentence_count: usize,
    /// The depth of the valley that opened it; zero for the first, which
    /// nothing opened.
    pub opening_depth: f64,
    pub keywords: Vec<(String, u64)>,
}

/// How many terms a topic publishes.
const KEYWORDS: usize = 5;
/// Below this length a token is punctuation or an interjection, not a term.
const MIN_TERM_CHARS: usize = 2;
/// A valley shallower than this is noise in the cosine, not a change of
/// vocabulary.
///
/// The relative test below — a depth above the mean less half a standard
/// deviation — is the classic one, and on its own it always finds something:
/// with any spread at all, some gap is the deepest, and being the deepest of
/// three near-identical numbers says nothing. Depth runs from zero to two, so
/// this floor asks for a drop of under a tenth of the range before a boundary
/// is even considered. It is what keeps four sentences about one subject from
/// being reported as two subjects.
const MIN_DEPTH: f64 = 0.15;

/// The content words of one sentence, lowercased and stripped.
///
/// Anything that is not a letter or a digit is dropped rather than replaced,
/// so "don't" counts as one term and "tick." as the same term as "tick" — the
/// recogniser's punctuation must not split a vocabulary in half.
pub(crate) fn tokenize(text: &str) -> Counts {
    let mut counts = Counts::new();
    for raw in text.split_whitespace() {
        let token = raw
            .chars()
            .filter(|character| character.is_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>();
        if token.chars().count() < MIN_TERM_CHARS || stopwords::is_stopword(&token) {
            continue;
        }
        *counts.entry(token).or_insert(0) += 1;
    }
    counts
}

/// Segment a sequence of sentences into topics.
///
/// `block` is how many sentences on each side of a gap are compared; `cutoff`
/// is how many standard deviations below the mean depth the boundary threshold
/// sits, so a larger cutoff lowers the bar and admits more boundaries.
pub(crate) fn segment(sentences: &[Counts], block: usize, cutoff: f64) -> Vec<Topic> {
    if sentences.is_empty() {
        return Vec::new();
    }
    let block = block.max(1);
    let boundaries = boundaries(sentences, block, cutoff);
    let depth_at = boundaries.iter().copied().collect::<BTreeMap<_, _>>();

    let mut topics = Vec::new();
    let mut first = 0usize;
    let mut opening = 0.0;
    let mut starts = boundaries.iter().map(|(gap, _)| *gap).collect::<Vec<_>>();
    starts.sort_unstable();
    for gap in starts {
        let count = gap + 1 - first;
        topics.push(Topic {
            first_sentence: first,
            sentence_count: count,
            opening_depth: opening,
            keywords: Vec::new(),
        });
        first = gap + 1;
        opening = depth_at.get(&gap).copied().unwrap_or(0.0);
    }
    topics.push(Topic {
        first_sentence: first,
        sentence_count: sentences.len() - first,
        opening_depth: opening,
        keywords: Vec::new(),
    });

    let weights = term_weights(&topics, sentences);
    for topic in &mut topics {
        topic.keywords = keywords(topic, sentences, &weights);
    }
    topics
}

/// The gaps deep enough to be boundaries, with their depths.
///
/// Taken deepest first so that a shallow valley never displaces the real
/// boundary beside it, and separated by at least one block so that two
/// boundaries cannot be decided from overlapping evidence.
fn boundaries(sentences: &[Counts], block: usize, cutoff: f64) -> Vec<(usize, f64)> {
    // The cutoff is a statement about a distribution, and a handful of gaps is
    // not one. Below a block's worth on each side there is nowhere for the
    // comparison window to move, so every depth is measuring the same few
    // sentences and the deepest of them means nothing.
    if sentences.len() < 2 || sentences.len() - 1 < 2 * block {
        return Vec::new();
    }
    let similarity = (0..sentences.len() - 1)
        .map(|gap| {
            let left = merge(sentences, gap.saturating_sub(block - 1), gap);
            let right = merge(sentences, gap + 1, (gap + block).min(sentences.len() - 1));
            cosine(&left, &right)
        })
        .collect::<Vec<_>>();
    let depths = (0..similarity.len())
        .map(|gap| depth(&similarity, gap))
        .collect::<Vec<_>>();

    #[allow(
        clippy::cast_precision_loss,
        reason = "a gap count well inside f64's exact integer range"
    )]
    let total = depths.len() as f64;
    let mean = depths.iter().sum::<f64>() / total;
    let variance = depths
        .iter()
        .map(|value| (value - mean) * (value - mean))
        .sum::<f64>()
        / total;
    let threshold = mean - cutoff * variance.sqrt();

    let mut candidates = depths
        .iter()
        .enumerate()
        .filter(|(_, depth)| **depth > threshold && **depth >= MIN_DEPTH)
        .map(|(gap, depth)| (gap, *depth))
        .collect::<Vec<_>>();
    // Deepest first, and where two are equally deep the earlier gap wins, so
    // the selection below cannot depend on sort stability.
    candidates.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });

    let mut taken: Vec<(usize, f64)> = Vec::new();
    for (gap, depth) in candidates {
        if taken.iter().any(|(other, _)| gap.abs_diff(*other) < block) {
            continue;
        }
        taken.push((gap, depth));
    }
    taken
}

/// The drop from the nearest peak on each side, which is what makes a valley a
/// valley rather than merely a low number.
fn depth(similarity: &[f64], gap: usize) -> f64 {
    let here = similarity[gap];
    let mut left = here;
    let mut at = gap;
    while at > 0 && similarity[at - 1] >= left {
        at -= 1;
        left = similarity[at];
    }
    let mut right = here;
    let mut at = gap;
    while at + 1 < similarity.len() && similarity[at + 1] >= right {
        at += 1;
        right = similarity[at];
    }
    (left - here) + (right - here)
}

fn merge(sentences: &[Counts], from: usize, to: usize) -> Counts {
    let mut merged = Counts::new();
    for sentence in &sentences[from..=to.min(sentences.len() - 1)] {
        for (term, count) in sentence {
            *merged.entry(term.clone()).or_insert(0) += count;
        }
    }
    merged
}

/// Cosine similarity over two term-count vectors.
///
/// Two blocks with no content words at all are treated as perfectly similar:
/// nothing changed, because there was nothing to change. One empty and one not
/// is the opposite — a vocabulary appeared or vanished — and scores zero.
fn cosine(left: &Counts, right: &Counts) -> f64 {
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "term counts well inside f64's exact integer range"
    )]
    let magnitude = |counts: &Counts| {
        counts
            .values()
            .map(|count| (*count as f64) * (*count as f64))
            .sum::<f64>()
            .sqrt()
    };
    #[allow(
        clippy::cast_precision_loss,
        reason = "term counts well inside f64's exact integer range"
    )]
    let dot = left
        .iter()
        .filter_map(|(term, count)| {
            right
                .get(term)
                .map(|other| (*count as f64) * (*other as f64))
        })
        .sum::<f64>();
    let norms = magnitude(left) * magnitude(right);
    if norms == 0.0 { 0.0 } else { dot / norms }
}

/// How much each term distinguishes one topic from the rest.
///
/// A term every topic uses says nothing about any of them, however often it is
/// said. This is the ordinary inverse-document-frequency correction with
/// topics as the documents, and with so few documents it mostly reorders ties —
/// which is exactly where a frequency-only ranking looked arbitrary.
fn term_weights(topics: &[Topic], sentences: &[Counts]) -> BTreeMap<String, f64> {
    let mut containing: BTreeMap<String, usize> = BTreeMap::new();
    for topic in topics {
        let terms = topic_terms(topic, sentences)
            .into_keys()
            .collect::<BTreeSet<_>>();
        for term in terms {
            *containing.entry(term).or_insert(0) += 1;
        }
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "a topic count well inside f64's exact integer range"
    )]
    let total = topics.len() as f64;
    containing
        .into_iter()
        .map(|(term, count)| {
            #[allow(
                clippy::cast_precision_loss,
                reason = "a topic count well inside f64's exact integer range"
            )]
            let documents = count as f64;
            (term, (1.0 + total / documents).ln())
        })
        .collect()
}

fn topic_terms(topic: &Topic, sentences: &[Counts]) -> Counts {
    merge(
        sentences,
        topic.first_sentence,
        topic.first_sentence + topic.sentence_count - 1,
    )
}

fn keywords(
    topic: &Topic,
    sentences: &[Counts],
    weights: &BTreeMap<String, f64>,
) -> Vec<(String, u64)> {
    let mut scored = topic_terms(topic, sentences)
        .into_iter()
        .map(|(term, count)| {
            #[allow(
                clippy::cast_precision_loss,
                reason = "term counts well inside f64's exact integer range"
            )]
            let frequency = count as f64;
            let score = frequency * weights.get(&term).copied().unwrap_or(1.0);
            (score, term, count)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .0
            .total_cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
    });
    scored
        .into_iter()
        .take(KEYWORDS)
        .map(|(_, term, count)| (term, count))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Counts, segment, tokenize};

    fn sentences(texts: &[&str]) -> Vec<Counts> {
        texts.iter().map(|text| tokenize(text)).collect()
    }

    #[test]
    fn punctuation_does_not_split_a_vocabulary_in_half() {
        assert_eq!(tokenize("tick."), tokenize("tick"));
        assert_eq!(tokenize("Don't"), tokenize("dont"));
        // Stopwords and single characters carry no topic and are dropped.
        assert!(tokenize("the a I of").is_empty());
    }

    #[test]
    fn one_sentence_is_one_topic_that_nothing_opened() {
        let topics = segment(&sentences(&["rendering the timeline"]), 2, 0.5);
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].first_sentence, 0);
        assert_eq!(topics[0].sentence_count, 1);
        assert!(topics[0].opening_depth == 0.0);
    }

    #[test]
    fn no_sentences_is_no_topics_rather_than_one_empty_topic() {
        assert!(segment(&[], 2, 0.5).is_empty());
    }

    /// The property the whole level exists for: two subjects spoken one after
    /// another are two neighbourhoods, and the boundary is where they meet.
    #[test]
    fn a_change_of_vocabulary_opens_a_topic() {
        let topics = segment(
            &sentences(&[
                "the renderer draws frames onto the timeline",
                "frames render onto the timeline in order",
                "rendering frames keeps the timeline honest",
                "the aligner measures word timing against audio",
                "word timing comes from measuring the audio",
                "measuring audio gives the aligner word timing",
            ]),
            2,
            0.5,
        );
        assert_eq!(topics.len(), 2, "{topics:#?}");
        assert_eq!(topics[0].first_sentence, 0);
        assert_eq!(topics[1].first_sentence, 3);
        assert!(topics[1].opening_depth > 0.0);
        // Every sentence belongs to exactly one topic, and none is lost.
        assert_eq!(
            topics
                .iter()
                .map(|topic| topic.sentence_count)
                .sum::<usize>(),
            6
        );
    }

    /// One subject throughout is one neighbourhood, however many sentences it
    /// takes. A segmenter that always finds a boundary is a segmenter that has
    /// found nothing.
    #[test]
    fn unchanging_vocabulary_stays_one_topic() {
        let topics = segment(
            &sentences(&[
                "the renderer draws frames",
                "frames are drawn by the renderer",
                "drawing frames is what the renderer does",
                "the renderer draws every frame",
            ]),
            2,
            0.5,
        );
        assert_eq!(topics.len(), 1, "{topics:#?}");
    }

    /// The relative test always finds something: with any spread at all, some
    /// gap is the deepest. Two guards stop that from being reported as a
    /// discovery — too few gaps for the statistics to mean anything, and a
    /// floor under how shallow a valley may be.
    #[test]
    fn too_few_sentences_to_measure_is_one_topic_rather_than_a_guess() {
        // The same six sentences that segment cleanly at a block of two: one
        // subject, then another.
        let text = sentences(&[
            "the renderer draws frames onto the timeline",
            "frames render onto the timeline in order",
            "rendering frames keeps the timeline honest",
            "the aligner measures word timing against audio",
            "word timing comes from measuring the audio",
            "measuring audio gives the aligner word timing",
        ]);
        assert_eq!(segment(&text, 2, 0.5).len(), 2);
        // A block of three wants six gaps and there are five, so the window
        // never moves off the same evidence and nothing is claimed. The words
        // did not change; what changed is whether they could be measured.
        assert_eq!(segment(&text, 3, 0.5).len(), 1);
    }

    #[test]
    fn the_same_sentences_segment_the_same_way_twice() {
        let text = sentences(&[
            "the renderer draws frames onto the timeline",
            "frames render onto the timeline in order",
            "the aligner measures word timing against audio",
            "word timing comes from measuring the audio",
        ]);
        assert_eq!(segment(&text, 2, 0.5), segment(&text, 2, 0.5));
    }

    /// Keywords are the terms a consumer will show a user, so their order has
    /// to be a fact about the words rather than about a hash table.
    #[test]
    fn keywords_are_ranked_then_alphabetical() {
        let topics = segment(
            &sentences(&["timeline timeline renderer renderer frames alpha"]),
            2,
            0.5,
        );
        let terms = topics[0]
            .keywords
            .iter()
            .map(|(term, _)| term.as_str())
            .collect::<Vec<_>>();
        assert_eq!(terms[0], "renderer");
        assert_eq!(terms[1], "timeline");
        // Equal weight below the top two, so alphabetical decides.
        assert_eq!(&terms[2..], ["alpha", "frames"]);
    }
}
