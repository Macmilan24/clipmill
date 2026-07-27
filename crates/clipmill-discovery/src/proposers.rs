//! The mesh: three ways of noticing that something is worth clipping.
//!
//! Genre diversity defeats any single model (book ch. 15). What makes a Q&A
//! moment complete is discourse structure; what makes a strong opinion
//! memorable is phrasing and delivery; neither shares features with the other.
//! So discovery is a portfolio of independent proposers emitting one common
//! contract, and a proposer that finds nothing is a fact about the recording
//! rather than a gap in the mesh.
//!
//! Three of the design's ten run here. The other seven need signals this phase
//! does not have — face affect, motion, OCR, speaker turns — and a proposer
//! that guessed at them would be nominating on evidence it never measured.
//!
//! Every one of these three is an approximation, and each says so in its
//! rubric string. That string reaches the artifact key, so a proposer whose
//! method changes cannot quietly reuse a cached candidate set, and a reader of
//! a published document can tell that `topic-span-open-close.v1` is a topic
//! span rather than a narrative model.

use std::collections::{BTreeMap, BTreeSet};

use clipmill_contracts::schemas::discovery_candidates as contract;
use clipmill_contracts::schemas::index_transcript as index;

use crate::prosody::Prosody;

/// One nominated moment, before expansion turns it into an interval.
#[derive(Clone, Debug)]
pub(crate) struct Seed {
    /// The span the proposer is pointing at, which expansion grows outward.
    pub interval: (u64, u64),
    pub evidence: Vec<contract::EvidenceReference>,
    pub hook: Option<contract::EvidenceReference>,
    pub payoff: Option<contract::EvidenceReference>,
    /// Comparable only within a proposer.
    pub score: f64,
}

fn sentence_reference(at: usize) -> contract::EvidenceReference {
    contract::EvidenceReference {
        kind: contract::EvidenceReferenceKind::Sentence,
        index: crate::as_u64(at),
    }
}

fn topic_reference(at: usize) -> contract::EvidenceReference {
    contract::EvidenceReference {
        kind: contract::EvidenceReferenceKind::Topic,
        index: crate::as_u64(at),
    }
}

/// A proposer's identity, carried into every candidate it nominates.
pub(crate) fn identity(name: &str, rubric: &str) -> contract::Proposer {
    contract::Proposer {
        name: crate::literal(name),
        rubric: crate::literal(rubric),
        version: crate::literal(VERSION),
    }
}

/// Bumped when any proposer's method changes, which invalidates exactly the
/// candidate sets that were nominated by the old one.
pub const VERSION: &str = "1.0.0";

pub(crate) const NARRATIVE: &str = "narrative-arc";
pub(crate) const NARRATIVE_RUBRIC: &str = "topic-span-open-close.v1";
pub(crate) const INSIGHT: &str = "insight-quote";
pub(crate) const INSIGHT_RUBRIC: &str = "tfidf-specificity-claim-prosody.v1";
pub(crate) const QUESTION: &str = "question-answer";
pub(crate) const QUESTION_RUBRIC: &str = "punctuation-and-wh-pattern.v1";

/// Narrative arc, approximated by topic span.
///
/// The design wants a setup–reveal–resolution mini-story, which needs event
/// structure and open-loop detection that nothing here measures. What is
/// measurable is that a run of sentences shares vocabulary and then stops
/// doing so, and a topic that opens and closes is the shape a mini-story
/// leaves behind even when nobody has identified the story. The rubric says
/// `topic-span-open-close`, not `narrative`, because those are different
/// claims and the second one would be believed.
///
/// The hook is the topic's opening sentence — the one a viewer hears first,
/// whether or not it is a good hook. The payoff is its most information-dense
/// close: the last third's highest-scoring sentence by the same novelty
/// measure the insight proposer uses, because a topic that ends on filler
/// should not have that filler labelled as its payoff.
pub(crate) fn narrative_arc(document: &index::IndexTranscript, novelty: &Novelty) -> Vec<Seed> {
    document
        .topics
        .iter()
        .enumerate()
        .filter_map(|(position, topic)| {
            let first = usize::try_from(topic.first_sentence_index).ok()?;
            let count = usize::try_from(topic.sentence_count.get()).ok()?;
            let members = document.sentences.get(first..first + count)?;
            // A topic of one sentence has no arc to speak of: it opens and
            // closes on the same words, and nominating it as a mini-story
            // would be nominating the sentence twice.
            if members.len() < 2 {
                return None;
            }
            let closing_from = first + (count * 2) / 3;
            let payoff = (closing_from..first + count)
                .max_by(|left, right| {
                    novelty
                        .of(*left)
                        .total_cmp(&novelty.of(*right))
                        .then_with(|| right.cmp(left))
                })
                .unwrap_or(first + count - 1);
            let mut evidence = vec![topic_reference(position)];
            evidence.extend((first..first + count).map(sentence_reference));
            Some(Seed {
                interval: (topic.start_ticks, topic.end_ticks),
                evidence,
                hook: Some(sentence_reference(first)),
                payoff: Some(sentence_reference(payoff)),
                // A topic the segmenter was confident about is a topic whose
                // boundaries mean something. Depth runs to two; a whole
                // opening is worth as much as a deep one.
                score: crate::clamp_unit(if position == 0 {
                    0.5
                } else {
                    0.25 + topic.opening_depth / 2.0
                }),
            })
        })
        .collect()
}

/// Insight or quote: a sentence worth hearing on its own.
///
/// Four measurable proxies, summed with stated weights. None of them knows
/// what was said; together they favour a sentence that uses words the rest of
/// the recording does not, names something concrete, states a position, and
/// was delivered with emphasis.
pub(crate) fn insight_quote(
    document: &index::IndexTranscript,
    novelty: &Novelty,
    prosody: &Prosody,
) -> Vec<Seed> {
    let scored = document
        .sentences
        .iter()
        .enumerate()
        .map(|(position, sentence)| {
            let text = sentence.text.as_str();
            let score = 0.35 * novelty.of(position)
                + 0.25 * specificity(text)
                + 0.20 * claim_language(text)
                + 0.20 * prosody.emphasis(sentence);
            (position, crate::clamp_unit(score))
        })
        .collect::<Vec<_>>();
    // Only sentences that stand out against this recording. An absolute bar
    // would nominate everything in an emphatic recording and nothing in a flat
    // one, which is a statement about the microphone rather than the content.
    let mean = if scored.is_empty() {
        0.0
    } else {
        crate::as_f64(scored.len()).recip() * scored.iter().map(|(_, score)| score).sum::<f64>()
    };
    scored
        .into_iter()
        .filter(|(position, score)| {
            *score > mean && document.sentences[*position].word_count.get() >= MIN_QUOTE_WORDS
        })
        .map(|(position, score)| {
            let sentence = &document.sentences[position];
            Seed {
                interval: (sentence.start_ticks, sentence.end_ticks),
                evidence: vec![sentence_reference(position)],
                hook: Some(sentence_reference(position)),
                payoff: None,
                score,
            }
        })
        .collect()
}

/// Under this a sentence is an interjection, not a quote.
const MIN_QUOTE_WORDS: u64 = 4;

/// Question and answer, without knowing who is speaking.
///
/// The design's version reads speaker turns. There is no diarization at this
/// phase, so this one cannot tell a question the host asked from one the guest
/// asked rhetorically, and it says so: the rubric is
/// `punctuation-and-wh-pattern`. What it can do is find a question and the
/// stretch that follows it, which is where an answer is if there is one.
///
/// The answer window closes at whichever comes first: the next question, the
/// end of the topic, or a pause long enough that the subject has changed.
pub(crate) fn question_answer(document: &index::IndexTranscript) -> Vec<Seed> {
    let questions = document
        .sentences
        .iter()
        .enumerate()
        .filter(|(_, sentence)| is_question(sentence))
        .map(|(position, _)| position)
        .collect::<BTreeSet<_>>();

    questions
        .iter()
        .filter_map(|position| {
            let question = &document.sentences[*position];
            let topic_end = topic_of(document, *position)
                .map_or(document.sentences.len(), |topic| topic.1)
                .min(document.sentences.len());
            let next_question = questions
                .range(position + 1..)
                .next()
                .copied()
                .unwrap_or(document.sentences.len());
            let mut last = *position;
            for at in position + 1..topic_end.min(next_question) {
                let sentence = &document.sentences[at];
                let previous = &document.sentences[at - 1];
                if sentence.start_ticks.saturating_sub(previous.end_ticks) >= ANSWER_GAP_TICKS {
                    break;
                }
                last = at;
            }
            // A question nobody answered is not a self-contained Q&A, and
            // clipping it would publish a setup with no payoff.
            if last == *position {
                return None;
            }
            let answer = &document.sentences[last];
            Some(Seed {
                interval: (question.start_ticks, answer.end_ticks),
                evidence: (*position..=last).map(sentence_reference).collect(),
                hook: Some(sentence_reference(*position)),
                payoff: Some(sentence_reference(last)),
                // Longer answers are more likely to be complete, with
                // diminishing returns: the difference between one sentence and
                // three matters, between eight and ten it does not.
                score: crate::clamp_unit(0.4 + 0.15 * crate::as_f64(last - position).sqrt()),
            })
        })
        .collect()
}

/// A pause this long between two sentences ends an answer: whatever comes
/// after it is a new thought, not the rest of this one. Two seconds — long
/// enough that it is not a breath, short enough that a considered pause mid-
/// answer does not truncate the clip.
const ANSWER_GAP_TICKS: u64 = 180_000;

/// The sentence range of the topic containing a sentence.
fn topic_of(document: &index::IndexTranscript, sentence: usize) -> Option<(usize, usize)> {
    document.topics.iter().find_map(|topic| {
        let first = usize::try_from(topic.first_sentence_index).ok()?;
        let count = usize::try_from(topic.sentence_count.get()).ok()?;
        (sentence >= first && sentence < first + count).then_some((first, first + count))
    })
}

/// Whether a sentence asks something.
///
/// Punctuation first, because a recognizer that wrote a question mark heard a
/// question. Failing that, the wh- and auxiliary-inversion openings that carry
/// most spoken questions in English — which is a real limit, and the reason
/// the rubric names the pattern rather than claiming question detection.
fn is_question(sentence: &index::Sentence) -> bool {
    let text = sentence.text.as_str();
    if text.trim_end().ends_with('?') {
        return true;
    }
    let opening = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase();
    matches!(
        opening.as_str(),
        "what"
            | "why"
            | "how"
            | "when"
            | "where"
            | "who"
            | "whom"
            | "whose"
            | "which"
            | "is"
            | "are"
            | "was"
            | "were"
            | "do"
            | "does"
            | "did"
            | "can"
            | "could"
            | "will"
            | "would"
            | "should"
            | "have"
            | "has"
            | "had"
    )
}

/// How much of a sentence is concrete: numbers, and words a speaker
/// capitalized mid-sentence, which in a recognizer's output is roughly where
/// the proper nouns are.
///
/// Roughly. A recognizer capitalizes inconsistently and the first word of a
/// sentence is capitalized for grammatical reasons, so that one is skipped.
/// This is a proxy for named entities, not a recognizer of them.
fn specificity(text: &str) -> f64 {
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return 0.0;
    }
    let concrete = tokens
        .iter()
        .enumerate()
        .filter(|(position, token)| {
            token.chars().any(|character| character.is_ascii_digit())
                || (*position > 0 && token.chars().next().is_some_and(char::is_uppercase))
        })
        .count();
    (crate::as_f64(concrete) / crate::as_f64(tokens.len()) * 3.0).min(1.0)
}

/// Whether a sentence states a position rather than narrating one.
///
/// A small lexicon, deliberately. A larger one starts encoding what the author
/// of the list thinks an opinion sounds like, and a learned classifier is a
/// model this phase does not run.
const CLAIM_WORDS: &[&str] = &[
    "always",
    "believe",
    "best",
    "biggest",
    "cannot",
    "critical",
    "essential",
    "every",
    "fundamental",
    "important",
    "key",
    "matters",
    "must",
    "never",
    "nobody",
    "point",
    "problem",
    "reason",
    "should",
    "think",
    "truth",
    "why",
    "worst",
    "wrong",
];

fn claim_language(text: &str) -> f64 {
    let hits = text
        .split_whitespace()
        .filter(|token| {
            let word = token
                .trim_matches(|character: char| !character.is_alphanumeric())
                .to_lowercase();
            CLAIM_WORDS.binary_search(&word.as_str()).is_ok()
        })
        .count();
    (crate::as_f64(hits) / 2.0).min(1.0)
}

/// How unusual each sentence's vocabulary is against the whole recording.
///
/// Ordinary tf-idf with sentences as documents, computed once and shared by
/// the two proposers that need it. Sharing matters beyond speed: a payoff the
/// narrative proposer picked and a quote the insight proposer nominated should
/// be scored by the same measure, or the two would disagree about which
/// sentence in a topic carries the most.
pub(crate) struct Novelty {
    scores: Vec<f64>,
}

impl Novelty {
    pub(crate) fn measure(document: &index::IndexTranscript) -> Self {
        let tokenized = document
            .sentences
            .iter()
            .map(|sentence| crate::tokens(sentence.text.as_str()))
            .collect::<Vec<_>>();
        let mut containing: BTreeMap<&str, usize> = BTreeMap::new();
        for terms in &tokenized {
            for term in terms.keys() {
                *containing.entry(term.as_str()).or_insert(0) += 1;
            }
        }
        let total = crate::as_f64(tokenized.len().max(1));
        let scores = tokenized
            .iter()
            .map(|terms| {
                if terms.is_empty() {
                    return 0.0;
                }
                let weight = terms
                    .iter()
                    .map(|(term, count)| {
                        let documents = crate::as_f64(
                            containing.get(term.as_str()).copied().unwrap_or(1).max(1),
                        );
                        crate::ticks_f64(*count) * (total / documents).ln()
                    })
                    .sum::<f64>();
                // Per term, so a long sentence is not novel merely by being
                // long, and normalized against a ceiling that a term appearing
                // once in a ten-sentence recording would reach.
                let per_term = weight / crate::as_f64(terms.len());
                (per_term / total.ln().max(1.0)).min(1.0)
            })
            .collect();
        Self { scores }
    }

    pub(crate) fn of(&self, sentence: usize) -> f64 {
        self.scores.get(sentence).copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests;
