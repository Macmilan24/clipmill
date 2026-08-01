//! Cue segmentation as an exact optimization.
//!
//! Line breaks are the craft of captioning, and they are also arithmetic. Every
//! consideration the book lists — reading speed, the quality of the break, not
//! orphaning an article, balanced lines, how long a cue is held, and never
//! spanning a cut — is a number attached to one candidate cue. A cue's cost
//! depends on which tokens it holds and nothing else, so the best partition of
//! the whole run is a shortest path over token boundaries, and dynamic
//! programming finds it exactly at a cost nobody will notice.
//!
//! "Exactly" is the load-bearing word. A greedy segmenter that fills a line and
//! moves on produces captions that look fine and read badly, and the difference
//! is invisible in review because every individual break is defensible. The
//! optimality property test in this module is what keeps that claim true: for
//! runs small enough to enumerate, the dynamic program's answer is compared
//! against every possible segmentation.
//!
//! ## What is hard and what is merely expensive
//!
//! Two constraints are absolute. A line may not exceed the profile's character
//! ceiling — that ceiling is how the safe area reaches this module — and a cue
//! may not span a shot cut *that falls in a silence*, because a caption that
//! survives a change of picture reads as a glitch. A cut that falls inside a
//! spoken word is a different thing: nothing can be done about it without
//! dropping the word, so it is allowed and reported rather than made
//! impossible.
//!
//! Everything else is a cost. Reading speed in particular is a cost and not a
//! constraint, because the alternative to a slightly fast cue is no cue, and a
//! viewer would rather read quickly than read nothing. The validator is what
//! reports the residue, and the gate is what refuses to ship it.

use crate::lexicon::Break;
use crate::profile::{Profile, TICKS_PER_SECOND};
use thiserror::Error;

/// One word, with everything the cost function needs to know about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub text: String,
    pub normalized: String,
    pub start_ticks: i64,
    pub end_ticks: i64,
    pub filler: bool,
    pub emphasis: bool,
    /// How good a break after this word would be, read from its punctuation.
    pub break_after: Break,
    /// Whether ending a line on this word would orphan it.
    pub orphans: bool,
}

impl Token {
    /// The rendered width, in the characters a viewer sees.
    pub fn characters(&self) -> usize {
        self.text.chars().count()
    }
}

/// A run of tokens on one rendered line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Line {
    pub first_token: usize,
    pub token_count: usize,
    pub characters: usize,
}

/// One caption, and the window it is held on screen for.
#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub first_token: usize,
    pub token_count: usize,
    pub start_ticks: i64,
    pub end_ticks: i64,
    pub lines: Vec<Line>,
    pub characters: usize,
    pub reading_rate_cps: f64,
}

/// The window a run of tokens is segmented inside. Its edges bind like cuts:
/// a cue may not cross them, because that is where the clip begins and ends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start_ticks: i64,
    pub end_ticks: i64,
}

/// What the dynamic program minimizes. Each term is normalized against the
/// thing it measures, so the weights compare like with like and a profile
/// change does not silently re-tune them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weights {
    /// Charged on how far a cue's characters-per-second exceeds the ceiling.
    pub reading_rate: f64,
    /// Charged on the width difference between a cue's two lines.
    pub line_balance: f64,
    /// Charged when a line or a cue ends on a word that binds forward.
    pub orphan: f64,
    /// Charged on how poor the break at the end of a cue is. This is also what
    /// stops the segmenter producing many tiny cues: every extra cue pays for
    /// the break it introduces.
    pub break_quality: f64,
    /// Charged when a cue cannot be held for the profile's minimum.
    pub short_cue: f64,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            reading_rate: 6.0,
            line_balance: 1.0,
            orphan: 2.0,
            break_quality: 3.0,
            short_cue: 4.0,
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SegmentError {
    #[error("the caption profile states numbers that cannot be met")]
    UnusableProfile,
    /// Two lines is the published standard and the shape this module solves
    /// exactly: the balance term compares a cue's lines against each other, so
    /// a third line is a different optimization rather than one more loop.
    /// Refused rather than approximated.
    #[error("this segmenter solves one- and two-line profiles exactly; {0} lines was asked for")]
    TooManyLines(usize),
    #[error("the span has no extent to segment inside")]
    EmptySpan,
}

/// Break the tokens into cues, optimally.
pub fn segment(
    tokens: &[Token],
    shot_cuts: &[i64],
    span: Span,
    profile: Profile,
    weights: Weights,
) -> Result<Vec<Cue>, SegmentError> {
    Ok(solve(tokens, shot_cuts, span, profile, weights)?.0)
}

/// The same segmentation with the cost that produced it, which is what makes
/// the optimality claim checkable rather than asserted.
pub fn solve(
    tokens: &[Token],
    shot_cuts: &[i64],
    span: Span,
    profile: Profile,
    weights: Weights,
) -> Result<(Vec<Cue>, f64), SegmentError> {
    if !profile.is_usable() {
        return Err(SegmentError::UnusableProfile);
    }
    if profile.max_lines > 2 {
        return Err(SegmentError::TooManyLines(profile.max_lines));
    }
    if span.end_ticks <= span.start_ticks {
        return Err(SegmentError::EmptySpan);
    }
    if tokens.is_empty() {
        return Ok((Vec::new(), 0.0));
    }

    let count = tokens.len();
    let mut best = vec![f64::INFINITY; count + 1];
    let mut previous = vec![usize::MAX; count + 1];
    best[0] = 0.0;

    for end in 1..=count {
        // Longest candidate first, so an exact tie resolves toward fewer cues.
        // Ties are otherwise a source of machine-dependent output, and these
        // cues reach a document addressed by content.
        let first = lowest_start(tokens, end, profile);
        for start in first..end {
            if !best[start].is_finite() {
                continue;
            }
            let Some((cost, _)) = cue(tokens, shot_cuts, span, profile, weights, start, end) else {
                continue;
            };
            let total = best[start] + cost;
            if total < best[end] {
                best[end] = total;
                previous[end] = start;
            }
        }
    }

    if !best[count].is_finite() {
        // Unreachable while a single token is always a legal cue, which
        // `lowest_start` guarantees. Kept as an empty answer rather than a
        // panic: a caller deserves a document that says there are no cues.
        return Ok((Vec::new(), 0.0));
    }

    let mut boundaries = vec![count];
    let mut at = count;
    while at > 0 {
        at = previous[at];
        boundaries.push(at);
    }
    boundaries.reverse();
    let mut cues = Vec::with_capacity(boundaries.len().saturating_sub(1));
    for window in boundaries.windows(2) {
        let (start, end) = (window[0], window[1]);
        if let Some((_, built)) = cue(tokens, shot_cuts, span, profile, weights, start, end) {
            cues.push(built);
        }
    }
    Ok((cues, best[count]))
}

/// The earliest token a cue ending at `end` may start from.
///
/// Bounded by the profile's capacity, and never above `end - 1`: a single token
/// is always a legal cue even when it is wider than the ceiling, because a word
/// longer than a line has to go somewhere and refusing it would leave the
/// recording uncaptionable.
fn lowest_start(tokens: &[Token], end: usize, profile: Profile) -> usize {
    let capacity = profile.capacity();
    let mut characters = 0_usize;
    let mut start = end;
    while start > 0 {
        let candidate = start - 1;
        let width = tokens[candidate].characters() + usize::from(candidate + 1 < end);
        if start < end && characters + width > capacity {
            break;
        }
        characters += width;
        start = candidate;
    }
    start
}

/// A candidate cue: its cost, and the cue itself. `None` when the tokens cannot
/// legally be one cue.
fn cue(
    tokens: &[Token],
    shot_cuts: &[i64],
    span: Span,
    profile: Profile,
    weights: Weights,
    start: usize,
    end: usize,
) -> Option<(f64, Cue)> {
    let run = &tokens[start..end];
    let speech_start = run.first()?.start_ticks;
    if crosses_a_cut_in_silence(run, shot_cuts) {
        return None;
    }
    let (lines, layout_cost) = layout(run, start, profile, weights)?;

    let characters: usize = run.iter().map(Token::characters).sum::<usize>() + (run.len() - 1);
    let display_end = window_end(tokens, shot_cuts, span, profile, start, end, characters);
    let duration = (display_end - speech_start).max(1);
    let rate = width(characters) / seconds(duration);

    let over = (rate - profile.reading_rate_cps).max(0.0) / profile.reading_rate_cps;
    let short = seconds((profile.min_duration_ticks - duration).max(0))
        / seconds(profile.min_duration_ticks);
    let last = run.last()?;
    let cost = layout_cost
        + weights.reading_rate * over * over
        + weights.short_cue * short
        + weights.break_quality * last.break_after.penalty()
        + if last.orphans { weights.orphan } else { 0.0 };

    Some((
        cost,
        Cue {
            first_token: start,
            token_count: end - start,
            start_ticks: speech_start,
            end_ticks: display_end,
            lines,
            characters,
            reading_rate_cps: rate,
        },
    ))
}

/// The timebase as a double, written out rather than cast, so the conversions
/// below are arithmetic and not a lint to be silenced.
const PER_SECOND: f64 = 90_000.0;
const _: () = assert!(TICKS_PER_SECOND == 90_000);

/// Ticks as seconds. A duration here is bounded by the profile's maximum, which
/// is seconds rather than centuries — nowhere near where a double stops being
/// exact about integers.
#[allow(
    clippy::cast_precision_loss,
    reason = "a duration in ticks, bounded by the profile's maximum"
)]
fn seconds(at: i64) -> f64 {
    at as f64 / PER_SECOND
}

/// Seconds as ticks, rounded up so a cue is never held a tick less than a
/// reader was promised.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a duration in seconds, bounded by the profile's maximum"
)]
fn ticks(at: f64) -> i64 {
    (at * PER_SECOND).ceil() as i64
}

/// A character count as a number to divide by.
#[allow(
    clippy::cast_precision_loss,
    reason = "characters on at most two lines"
)]
fn width(characters: usize) -> f64 {
    characters as f64
}

/// Whether a cut falls in a silence inside this run.
///
/// A cut between two words means the picture changed while nobody was speaking,
/// and a cue that survives it reads as a glitch. A cut inside a word is the
/// unavoidable case and is deliberately not counted here.
fn crosses_a_cut_in_silence(run: &[Token], shot_cuts: &[i64]) -> bool {
    run.windows(2).any(|pair| {
        let (gap_start, gap_end) = (pair[0].end_ticks, pair[1].start_ticks);
        shot_cuts
            .iter()
            .any(|cut| *cut >= gap_start && *cut <= gap_end && gap_end > gap_start)
    })
}

/// When the cue leaves the screen.
///
/// A cue may be held past its speech so a reader can finish it, which is what
/// makes the reading-rate ceiling reachable at all. What it may never do is
/// crowd the next cue, outlive the profile's maximum, cross a cut, or leave the
/// span. Every one of those limits is a function of this cue's own tokens, so
/// the cost stays local and the dynamic program stays exact.
fn window_end(
    tokens: &[Token],
    shot_cuts: &[i64],
    span: Span,
    profile: Profile,
    start: usize,
    end: usize,
    characters: usize,
) -> i64 {
    let speech_start = tokens[start].start_ticks;
    let speech_end = tokens[start..end]
        .iter()
        .map(|token| token.end_ticks)
        .max()
        .unwrap_or(speech_start);

    let next = tokens.get(end).map_or(span.end_ticks, |token| {
        token.start_ticks - profile.min_gap_ticks
    });
    let after_cut = shot_cuts
        .iter()
        .copied()
        .filter(|cut| *cut >= speech_end)
        .min()
        .unwrap_or(i64::MAX);
    let ceiling = next
        .min(after_cut)
        .min(speech_start.saturating_add(profile.max_duration_ticks))
        .min(span.end_ticks);

    let readable = ticks(width(characters) / profile.reading_rate_cps);
    let wanted = speech_start
        .saturating_add(profile.min_duration_ticks.max(readable))
        .max(speech_end);
    // Never shorter than the speech itself: a caption that disappears while the
    // word is still being said is worse than one that is briefly crowded.
    wanted.min(ceiling).max(speech_end)
}

/// The best way to break a run across the profile's lines.
///
/// Exhaustive over the one split point a two-line profile allows, which is what
/// makes it exact. The balance term compares the two lines against each other,
/// which is exactly why a third line would need a different formulation rather
/// than another loop — and why one is refused rather than approximated.
fn layout(
    run: &[Token],
    offset: usize,
    profile: Profile,
    weights: Weights,
) -> Option<(Vec<Line>, f64)> {
    let widths: Vec<usize> = run.iter().map(Token::characters).collect();
    let single = width_of(&widths, 0, run.len());

    let mut best: Option<(Vec<Line>, f64)> = None;
    if single <= profile.max_line_characters || run.len() == 1 {
        // One line: nothing to balance and nothing to orphan, because the cue's
        // own final break is charged by the caller.
        best = Some((
            vec![Line {
                first_token: offset,
                token_count: run.len(),
                characters: single,
            }],
            0.0,
        ));
    }
    if profile.max_lines >= 2 {
        for split in 1..run.len() {
            let top = width_of(&widths, 0, split);
            let bottom = width_of(&widths, split, run.len());
            if top > profile.max_line_characters || bottom > profile.max_line_characters {
                continue;
            }
            let imbalance = width(top.abs_diff(bottom)) / width(profile.max_line_characters);
            let cost = weights.line_balance * imbalance
                + if run[split - 1].orphans {
                    weights.orphan
                } else {
                    0.0
                };
            if best.as_ref().is_none_or(|(_, current)| cost < *current) {
                best = Some((
                    vec![
                        Line {
                            first_token: offset,
                            token_count: split,
                            characters: top,
                        },
                        Line {
                            first_token: offset + split,
                            token_count: run.len() - split,
                            characters: bottom,
                        },
                    ],
                    cost,
                ));
            }
        }
    }
    best
}

/// The rendered width of `widths[from..to]` joined by single spaces.
fn width_of(widths: &[usize], from: usize, to: usize) -> usize {
    if to <= from {
        return 0;
    }
    widths[from..to].iter().sum::<usize>() + (to - from - 1)
}

#[cfg(test)]
mod tests;
