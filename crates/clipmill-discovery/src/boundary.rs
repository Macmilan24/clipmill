//! Where the clip is actually cut.
//!
//! Boundaries get their own stage because they are where a viewer measures the
//! product's competence in the first five seconds (book ch. 16). Discovery
//! deliberately declined to choose — it published the whole legal lattice — and
//! this is the stage with enough information to pick from it.
//!
//! Every legal pair is scored:
//!
//! ```text
//! J(i,j) = w1·completeness + w2·hook + w3·payoff + w4·continuity
//!        − w5·deadair − w6·abrupt − w7·contextdebt
//! ```
//!
//! Exhaustively, because the lattice is small — the design expects
//! `|A|×|B| < 400`, and a search that small should not be approximated. The
//! runner-up is published beside the winner: the optimizer's second choice is
//! frequently the editor's first, and a boundary alternative one click away is
//! cheaper than re-running anything.
//!
//! The speech rules the design names are here as terms rather than as filters,
//! with one exception. "Never start mid-word" is structural and was already
//! enforced in discovery, so no pair reaching this module can break it. The
//! others — never open on an unresolvable pronoun, never open an answer without
//! its question, end after closure rather than before the re-explanation — are
//! preferences a strong enough hook can outweigh, which is exactly what the
//! design's own worked example does when it prefers a cold open.

use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::ranking_set as contract;

/// Hand-set, and named so. The design calibrates these per genre against human
/// labels; until that has happened, a version string is the only honest way to
/// say which arithmetic ran.
pub const RUBRIC: &str = "boundary-j-handset.v1";

const W_COMPLETENESS: f64 = 1.0;
const W_HOOK: f64 = 1.2;
const W_PAYOFF: f64 = 0.8;
const W_CONTINUITY: f64 = 0.6;
const W_DEADAIR: f64 = 0.9;
const W_ABRUPT: f64 = 1.5;
const W_CONTEXT_DEBT: f64 = 0.7;

/// Openings that refer to something the viewer has not been shown.
///
/// A clip that starts "and that's why it works" is asking the viewer to
/// remember a sentence they never heard. Deliberately a small list of the
/// pronouns and connectives that do this most; coreference resolution is a
/// model this phase does not run, and a longer list would be guessing more
/// confidently rather than more accurately.
const UNRESOLVED_OPENINGS: &[&str] = &[
    "and", "anyway", "because", "besides", "but", "he", "her", "him", "his", "however", "it",
    "its", "she", "so", "that", "their", "them", "then", "these", "they", "this", "those", "thus",
    "which", "who",
];

/// Everything the optimizer needs about the recording, gathered once.
pub(crate) struct Context<'a> {
    pub sentences: &'a [index::Sentence],
    pub utterances: &'a [index::Utterance],
    pub edges: &'a [index::Edge],
}

/// The chosen interval, the runner-up, and the arithmetic behind both.
#[derive(Clone, Debug)]
pub(crate) struct Choice {
    pub chosen: (u64, u64),
    pub score: f64,
    pub terms: Vec<contract::BoundaryTerm>,
    pub alternative: Option<((u64, u64), f64)>,
    pub considered: u64,
}

/// Score every legal pair in the lattice and keep the best two.
///
/// Legality is re-tested here rather than assumed: discovery published points
/// that each pair with *something* legal, which is not the same as every pair
/// being legal. Returns `None` only when nothing is legal at all, which the
/// caller reports as a filtered candidate rather than swallowing.
pub(crate) fn optimize(
    starts: &[u64],
    ends: &[u64],
    duration: (u64, u64),
    context: &Context<'_>,
) -> Option<Choice> {
    let mut best: Option<((u64, u64), f64)> = None;
    let mut runner_up: Option<((u64, u64), f64)> = None;
    let mut considered = 0u64;
    for start in starts {
        for end in ends {
            if !legal(*start, *end, duration) {
                continue;
            }
            considered += 1;
            let score = judge(*start, *end, context).total();
            // Ties break toward the earlier start and then the earlier end, so
            // the winner does not depend on iteration order.
            // Exact comparison on purpose: the tiebreak has to be
            // deterministic, and a tolerance would make which pair wins depend
            // on how close two unrelated scores happened to land.
            let better_than = |other: &Option<((u64, u64), f64)>| match other {
                None => true,
                Some((interval, best_score)) => match score.total_cmp(best_score) {
                    std::cmp::Ordering::Greater => true,
                    std::cmp::Ordering::Equal => (*start, *end) < (interval.0, interval.1),
                    std::cmp::Ordering::Less => false,
                },
            };
            if better_than(&best) {
                runner_up = best;
                best = Some(((*start, *end), score));
            } else if better_than(&runner_up) {
                runner_up = Some(((*start, *end), score));
            }
        }
    }
    let (chosen, score) = best?;
    Some(Choice {
        chosen,
        score: crate::published(score),
        terms: judge(chosen.0, chosen.1, context).published(),
        alternative: runner_up.map(|(interval, score)| (interval, crate::published(score))),
        considered,
    })
}

fn legal(start: u64, end: u64, duration: (u64, u64)) -> bool {
    start < end && {
        let span = end - start;
        span >= duration.0 && span <= duration.1
    }
}

/// The seven terms, unweighted.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Terms {
    completeness: f64,
    hook: f64,
    payoff: f64,
    continuity: f64,
    deadair: f64,
    abrupt: f64,
    context_debt: f64,
}

impl Terms {
    fn total(self) -> f64 {
        W_COMPLETENESS * self.completeness
            + W_HOOK * self.hook
            + W_PAYOFF * self.payoff
            + W_CONTINUITY * self.continuity
            - W_DEADAIR * self.deadair
            - W_ABRUPT * self.abrupt
            - W_CONTEXT_DEBT * self.context_debt
    }

    fn published(self) -> Vec<contract::BoundaryTerm> {
        // Signed weights, so a reader does not have to know which four reward
        // and which three penalise.
        [
            (
                contract::BoundaryTermName::Completeness,
                self.completeness,
                W_COMPLETENESS,
            ),
            (contract::BoundaryTermName::Hook, self.hook, W_HOOK),
            (contract::BoundaryTermName::Payoff, self.payoff, W_PAYOFF),
            (
                contract::BoundaryTermName::Continuity,
                self.continuity,
                W_CONTINUITY,
            ),
            (
                contract::BoundaryTermName::Deadair,
                self.deadair,
                -W_DEADAIR,
            ),
            (contract::BoundaryTermName::Abrupt, self.abrupt, -W_ABRUPT),
            (
                contract::BoundaryTermName::ContextDebt,
                self.context_debt,
                -W_CONTEXT_DEBT,
            ),
        ]
        .into_iter()
        .map(|(name, value, weight)| contract::BoundaryTerm {
            name,
            value: crate::clamp_unit(value),
            weight,
        })
        .collect()
    }
}

/// Measure one pair.
pub(crate) fn judge(start: u64, end: u64, context: &Context<'_>) -> Terms {
    let span = end.saturating_sub(start);
    if span == 0 {
        return Terms::default();
    }
    let inside = context
        .sentences
        .iter()
        .filter(|sentence| sentence.start_ticks < end && sentence.end_ticks > start)
        .collect::<Vec<_>>();
    let Some(first) = inside.first() else {
        // A span holding no sentence is silence. Legal, and almost certainly
        // not what anyone wants, which the terms say rather than a filter.
        return Terms {
            deadair: 1.0,
            ..Terms::default()
        };
    };
    let last = inside[inside.len() - 1];

    // Completeness: how much of the span is whole sentences rather than the
    // halves of ones the boundary cut through.
    let whole = inside
        .iter()
        .filter(|sentence| sentence.start_ticks >= start && sentence.end_ticks <= end)
        .map(|sentence| sentence.end_ticks - sentence.start_ticks)
        .sum::<u64>();
    let completeness = crate::ticks_f64(whole) / crate::ticks_f64(span);

    // Abrupt: starting or ending part-way through a sentence. The heaviest
    // penalty, because it is the failure a viewer notices in the first second.
    let opens_clean = f64::from(u8::from(first.start_ticks >= start));
    let closes_clean = f64::from(u8::from(last.end_ticks <= end));
    let abrupt = 1.0 - 0.5 * (opens_clean + closes_clean);

    // Hook: what the clip opens on. A punctuated sentence that begins the span
    // is a clean opening; a fast one is a better one.
    let hook = if first.start_ticks >= start {
        let rate = (first.words_per_minute / 200.0).clamp(0.0, 1.0);
        0.6 + 0.4 * rate
    } else {
        0.0
    };

    // Payoff: ending on something the recognizer punctuated is a closed
    // thought. Ending merely where the speaker stopped is weaker, and running
    // out of recording is weaker still.
    let payoff = match last.terminator {
        index::SentenceTerminator::Punctuation if last.end_ticks <= end => 1.0,
        index::SentenceTerminator::UtteranceEnd if last.end_ticks <= end => 0.6,
        index::SentenceTerminator::CoverageEnd if last.end_ticks <= end => 0.3,
        _ => 0.0,
    };

    // Continuity: a boundary that lands on a shot cut or in a silence is one
    // the footage already makes for you.
    let continuity = 0.5 * (edge_at(context.edges, start) + edge_at(context.edges, end));

    // Dead air: speech that does not fill the span. Measured against the
    // utterances rather than the sentences, because the gaps between sentences
    // inside one utterance are breaths, not silence.
    let spoken = context
        .utterances
        .iter()
        .filter_map(|utterance| {
            let low = utterance.start_ticks.max(start);
            let high = utterance.end_ticks.min(end);
            (high > low).then(|| high - low)
        })
        .sum::<u64>();
    let deadair = 1.0 - crate::ticks_f64(spoken) / crate::ticks_f64(span);

    // Context debt: opening on a word that refers to something the viewer has
    // not been shown.
    let context_debt = f64::from(u8::from(opens_unresolved(first.text.as_str())));

    Terms {
        completeness: crate::clamp_unit(completeness),
        hook,
        payoff,
        continuity,
        deadair: crate::clamp_unit(deadair),
        abrupt,
        context_debt,
    }
}

/// Whether an edge sits at this tick — a shot cut exactly on it, or a silence
/// containing it.
fn edge_at(edges: &[index::Edge], at: u64) -> f64 {
    let hit = edges.iter().any(|edge| match edge.kind {
        index::EdgeKind::ShotCut => edge.start_ticks == at,
        index::EdgeKind::Silence => at >= edge.start_ticks && at <= edge.end_ticks,
    });
    f64::from(u8::from(hit))
}

pub(crate) fn opens_unresolved(text: &str) -> bool {
    let opening = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase();
    UNRESOLVED_OPENINGS.binary_search(&opening.as_str()).is_ok()
}

#[cfg(test)]
mod tests;
