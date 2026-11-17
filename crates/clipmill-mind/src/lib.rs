//! The editorial mind's deterministic rules (decisions R75, R76).
//!
//! A language model proposes a clip as a plan of parts; this crate decides
//! whether the plan is one ClipMill will make. Models are good at judging what
//! is worth keeping and poor at exact times, so every time, join and form rule
//! is checked here, in code, and every problem found is returned at once, in
//! words a person (or a repairing model) can act on.
//!
//! Times are integer ticks at 1/90000 s of the source.

use std::collections::BTreeSet;

use clipmill_contracts::schemas::mind_clip_plan::{
    JoinKind, MindClipPlan, MindClipPlanForm, Part, PartRole,
};

/// Ticks per second.
pub const SECOND: u64 = 90_000;

/// The shortest part worth a cut: anything shorter reads as a glitch.
pub const MIN_PART_TICKS: u64 = 108_000;

/// A reaction (a laugh, a look, "wow") may be shorter than a spoken part.
pub const MIN_REACTION_TICKS: u64 = 36_000;

/// The widest gap in the source an `adjacent` join may step over: a breath or
/// a pause, not a skipped passage.
pub const ADJACENT_GAP_TICKS: u64 = 3 * SECOND;

/// The fewest parts a theme montage is made of.
pub const MIN_MONTAGE_PARTS: usize = 3;

/// What a plan is checked against besides itself.
#[derive(Clone, Copy, Debug)]
pub struct Bounds<'a> {
    /// Every recognised word as `(start, end)`, in time order. A part may not
    /// begin or end inside one. Empty means the recording has no words (silent
    /// footage), where any boundary is allowed.
    pub words: &'a [(u64, u64)],
    /// The programme length the brief asks for, as `(min, max)` ticks.
    pub duration: Option<(u64, u64)>,
    /// The recording's length; no part may run past it.
    pub source_ticks: u64,
}

/// One reason a plan cannot be made as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub code: &'static str,
    /// The part or join it concerns, when one.
    pub at: Option<String>,
    pub message: String,
}

fn violation(code: &'static str, at: Option<&str>, message: String) -> Violation {
    Violation {
        code,
        at: at.map(str::to_owned),
        message,
    }
}

/// How long a part plays: its range less what was removed from it.
pub fn kept_ticks(part: &Part) -> u64 {
    let length = part.end_ticks.get().saturating_sub(part.start_ticks);
    let removed: u64 = part
        .removed
        .iter()
        .map(|range| range.end_ticks.get().saturating_sub(range.start_ticks))
        .sum();
    length.saturating_sub(removed)
}

/// The plan's programme length: every part's kept length.
pub fn programme_ticks(plan: &MindClipPlan) -> u64 {
    plan.parts.iter().map(kept_ticks).sum()
}

/// Every reason `plan` cannot be made, or nothing when it can.
pub fn check(plan: &MindClipPlan, bounds: &Bounds<'_>) -> Vec<Violation> {
    let mut found = Vec::new();
    if plan.parts.is_empty() {
        found.push(violation(
            "no_parts",
            None,
            "A plan needs at least one part.".to_owned(),
        ));
        return found;
    }
    check_parts(plan, bounds, &mut found);
    check_overlaps(plan, &mut found);
    check_joins(plan, &mut found);
    check_form(plan, &mut found);
    check_duration(plan, bounds, &mut found);
    found
}

fn check_parts(plan: &MindClipPlan, bounds: &Bounds<'_>, found: &mut Vec<Violation>) {
    let mut ids = BTreeSet::new();
    for part in &plan.parts {
        let id = part.part_id.as_str();
        if !ids.insert(id) {
            found.push(violation(
                "duplicate_part",
                Some(id),
                format!("Part {id} appears twice."),
            ));
        }
        let (start, end) = (part.start_ticks, part.end_ticks.get());
        if end <= start {
            found.push(violation(
                "empty_part",
                Some(id),
                format!("Part {id} ends before it starts."),
            ));
            continue;
        }
        if end > bounds.source_ticks {
            found.push(violation(
                "past_source",
                Some(id),
                format!("Part {id} runs past the end of the recording."),
            ));
        }
        for (edge, at) in [("start", start), ("end", end)] {
            if let Some(word) = severs(bounds.words, at) {
                found.push(violation(
                    "cuts_word",
                    Some(id),
                    format!(
                        "Part {id}'s {edge} at {} falls inside a word ({}–{}); move it to a word boundary.",
                        clock(at),
                        clock(word.0),
                        clock(word.1)
                    ),
                ));
            }
        }
        let mut previous_end = start;
        for range in &part.removed {
            let (cut_start, cut_end) = (range.start_ticks, range.end_ticks.get());
            if cut_start < previous_end || cut_end <= cut_start || cut_end > end {
                found.push(violation(
                    "bad_removal",
                    Some(id),
                    format!(
                        "A range removed from part {id} ({}–{}) is outside the part, empty, or overlaps another.",
                        clock(cut_start),
                        clock(cut_end)
                    ),
                ));
                continue;
            }
            if cut_start == start || cut_end == end {
                found.push(violation(
                    "removal_at_edge",
                    Some(id),
                    format!(
                        "Part {id} removes a range at its own edge; move the part's boundary instead."
                    ),
                ));
            }
            for at in [cut_start, cut_end] {
                if severs(bounds.words, at).is_some() {
                    found.push(violation(
                        "cuts_word",
                        Some(id),
                        format!(
                            "A cut inside part {id} at {} falls inside a word.",
                            clock(at)
                        ),
                    ));
                }
            }
            previous_end = cut_end;
        }
        let minimum = if part.role == PartRole::Reaction {
            MIN_REACTION_TICKS
        } else {
            MIN_PART_TICKS
        };
        if kept_ticks(part) < minimum {
            found.push(violation(
                "part_too_short",
                Some(id),
                format!(
                    "Part {id} plays for {}, shorter than {} — too short to read as anything but a glitch.",
                    duration(kept_ticks(part)),
                    duration(minimum)
                ),
            ));
        }
    }
}

fn check_overlaps(plan: &MindClipPlan, found: &mut Vec<Violation>) {
    // No moment plays twice, except the line a cold open lifts to the front,
    // which is meant to be heard again in context.
    let lifted = (plan.form == MindClipPlanForm::ColdOpen)
        .then(|| plan.parts.first())
        .flatten()
        .filter(|part| part.role == PartRole::Hook)
        .map(|part| part.part_id.as_str());
    let mut ranges: Vec<&Part> = plan
        .parts
        .iter()
        .filter(|part| Some(part.part_id.as_str()) != lifted)
        .collect();
    ranges.sort_by_key(|part| (part.start_ticks, part.end_ticks.get()));
    for pair in ranges.windows(2) {
        if pair[1].start_ticks < pair[0].end_ticks.get() {
            found.push(violation(
                "parts_overlap",
                Some(pair[1].part_id.as_str()),
                format!(
                    "Parts {} and {} overlap in the recording, so some of it would play twice.",
                    pair[0].part_id.as_str(),
                    pair[1].part_id.as_str()
                ),
            ));
        }
    }
}

fn check_joins(plan: &MindClipPlan, found: &mut Vec<Violation>) {
    if plan.joins.len() + 1 != plan.parts.len() {
        found.push(violation(
            "join_count",
            None,
            format!(
                "{} parts need {} joins, not {}.",
                plan.parts.len(),
                plan.parts.len() - 1,
                plan.joins.len()
            ),
        ));
        return;
    }
    for (index, join) in plan.joins.iter().enumerate() {
        let (current, next) = (&plan.parts[index], &plan.parts[index + 1]);
        let id = join.after.as_str();
        if id != current.part_id.as_str() {
            found.push(violation(
                "join_order",
                Some(id),
                format!(
                    "Join {} should follow part {}, the part before it.",
                    index + 1,
                    current.part_id.as_str()
                ),
            ));
            continue;
        }
        let forward = next.start_ticks >= current.end_ticks.get();
        let gap = next.start_ticks.saturating_sub(current.end_ticks.get());
        let geometry = if !forward {
            JoinKind::Reorder
        } else if gap <= ADJACENT_GAP_TICKS {
            JoinKind::Adjacent
        } else {
            JoinKind::Distant
        };
        if join.kind != geometry {
            found.push(violation(
                "join_kind",
                Some(id),
                format!(
                    "The join after part {id} is marked {} but the next part {}.",
                    kind_name(join.kind),
                    match geometry {
                        JoinKind::Reorder => "comes earlier in the recording (a reorder)",
                        JoinKind::Adjacent => "continues the recording (adjacent)",
                        JoinKind::Distant => "comes from elsewhere in the recording (distant)",
                    }
                ),
            ));
        }
        let lifted_hook =
            index == 0 && plan.form == MindClipPlanForm::ColdOpen && current.role == PartRole::Hook;
        let needs_link = match geometry {
            JoinKind::Distant => true,
            JoinKind::Reorder => !lifted_hook,
            JoinKind::Adjacent => false,
        };
        if needs_link && join.link.is_none() {
            found.push(violation(
                "unjustified_join",
                Some(id),
                format!(
                    "The join after part {id} jumps across the recording without the evidence that links the two parts."
                ),
            ));
        }
    }
}

fn check_form(plan: &MindClipPlan, found: &mut Vec<Violation>) {
    let parts = plan.parts.len();
    let removes = plan.parts.iter().any(|part| !part.removed.is_empty());
    let all_adjacent = plan
        .joins
        .iter()
        .all(|join| join.kind == JoinKind::Adjacent);
    let problem = match plan.form {
        MindClipPlanForm::Straight => {
            (parts != 1 || removes).then_some("A straight cut is one part with nothing removed.")
        }
        MindClipPlanForm::Tightened => (!all_adjacent || (parts == 1 && !removes))
            .then_some("A tightened cut keeps one moment in order and removes something from it."),
        MindClipPlanForm::Stitch => (parts < 2
            || !plan.joins.iter().any(|join| join.kind == JoinKind::Distant))
        .then_some("A stitch joins parts from different places in the recording."),
        MindClipPlanForm::ColdOpen => (parts < 2
            || plan.parts[0].role != PartRole::Hook
            || plan.joins.first().map(|join| join.kind) != Some(JoinKind::Reorder))
        .then_some("A cold open starts with a hook lifted from later, then goes back."),
        MindClipPlanForm::ThemeMontage => {
            (parts < MIN_MONTAGE_PARTS).then_some("A theme montage needs at least three moments.")
        }
    };
    if let Some(message) = problem {
        found.push(violation("form", None, message.to_owned()));
    }
}

fn check_duration(plan: &MindClipPlan, bounds: &Bounds<'_>, found: &mut Vec<Violation>) {
    let playing = programme_ticks(plan);
    if plan.duration_ticks.get() != playing {
        found.push(violation(
            "duration_mismatch",
            None,
            format!(
                "The plan says it lasts {} but its parts play for {}.",
                duration(plan.duration_ticks.get()),
                duration(playing)
            ),
        ));
    }
    if let Some((minimum, maximum)) = bounds.duration
        && !(minimum..=maximum).contains(&playing)
    {
        found.push(violation(
            "duration_range",
            None,
            format!(
                "The clip plays for {}, outside the {}–{} asked for.",
                duration(playing),
                duration(minimum),
                duration(maximum)
            ),
        ));
    }
}

/// The word a boundary at `at` would cut through, if any.
fn severs(words: &[(u64, u64)], at: u64) -> Option<(u64, u64)> {
    let index = words.partition_point(|&(start, _)| start < at);
    // Only the word that starts before `at` can contain it.
    index
        .checked_sub(1)
        .map(|previous| words[previous])
        .filter(|&(start, end)| start < at && at < end)
}

const fn kind_name(kind: JoinKind) -> &'static str {
    match kind {
        JoinKind::Adjacent => "adjacent",
        JoinKind::Reorder => "a reorder",
        JoinKind::Distant => "distant",
    }
}

/// `m:ss.t` for messages.
fn clock(ticks: u64) -> String {
    let tenths = ticks / 9_000;
    format!("{}:{:02}.{}", tenths / 600, (tenths / 10) % 60, tenths % 10)
}

/// `12.3 s` for messages.
fn duration(ticks: u64) -> String {
    let tenths = ticks / 9_000;
    format!("{}.{} s", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests;
