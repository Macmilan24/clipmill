//! The cutter: candidate clips built the way an editor builds them.
//!
//! A clip that is one continuous span of the recording, start to finish, is
//! the weakest thing an editor makes. The cutter reads what the mind knows
//! about a recording (its events and the links between them in the Recording
//! Memory, and the editor's markup of every sentence) and proposes plans of
//! parts instead:
//!
//! - **tightened**: one moment with its tangents, retakes and filler cut out;
//! - **stitch**: a question and its answer, a setup and its payoff, a promise
//!   and its delivery, joined across the minutes between them;
//! - **cold open**: the moment's strongest line lifted to the front, then the
//!   moment from its start;
//! - **theme montage**: short moments about one person, product
//!   or idea;
//! - **straight**: a moment that needs nothing removed.
//!
//! Every part starts and ends on a sentence edge, a part never opens on a line
//! that needs an earlier one, and every plan passes [`crate::check`] before it
//! is returned. When the recording's pauses were measured, every edge and
//! every removal sits on a [`crate::lattice::CutPoint`] in the pause at that
//! sentence edge, and a sentence edge with no usable pause is not used: a cut
//! there would land in speech. A clip does not end where the same speaker
//! carries the thought on ("and…", "but…"), nor open on a line that points
//! back ("but…", "that's why…"). Choosing between candidates is the planner's and the ranker's
//! job; the score here only orders them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::ops::RangeInclusive;

use clipmill_contracts::schemas::mind_clip_plan::MindClipPlan;
use clipmill_contracts::schemas::mind_memory::{EventKind, EventMood, LinkKind, MindMemory};
use clipmill_contracts::schemas::mind_unit_scores::{Level, MindUnitScores, RowTrim};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::lattice::Lattice;
use crate::script::Source;
use crate::{ADJACENT_GAP_TICKS, Bounds, MIN_MONTAGE_PARTS, MIN_PART_TICKS, SECOND, check};

/// How far back a line's dependency may be pulled into the same part.
pub const NEEDS_REACH_UNITS: usize = 10;
/// How far past an event's end the cutter looks for a landing.
pub const EXTEND_UNITS: usize = 8;
/// The longest line a cold open lifts to the front.
pub const MAX_LIFT_TICKS: u64 = 10 * SECOND;
/// The shortest moment worth a place in a montage.
pub const MIN_MONTAGE_ITEM_TICKS: u64 = 3 * SECOND;
const MIN_SCOUTED_MONTAGE_MOMENTS: usize = 3;
/// The most moments in a montage scouted without an exhaustive brief.
pub const MAX_MONTAGE_PARTS: usize = 5;
// The clip-plan contract allows twelve parts. A requested exhaustive montage
// may need more moments than the five normally scouted for an unsolicited one.
const MAX_REQUESTED_MONTAGE_PARTS: usize = 12;
/// Linked moments closer than this are one moment, not a stitch: jumping a
/// few sentences to reach an answer cuts out what led to it for nothing.
pub const STITCH_MIN_GAP_TICKS: u64 = 45 * SECOND;
/// Two candidates sharing this much of what they play are one candidate;
/// the better-scored stays.
pub const DUPLICATE_SHARE_PERCENT: u64 = 50;
/// The most candidates the cutter hands on, best first.
pub const MAX_CANDIDATES: usize = 30;
/// The most of a moment a tight cut may leave out. Markup that would cut
/// more is not trusted for that moment: an editor tightens a moment, they do
/// not hollow it out.
pub const MAX_CUT_PERCENT: u64 = 35;
/// The least a removal must save. A shorter one is a visible jump for almost
/// nothing gained.
pub const MIN_REMOVAL_TICKS: u64 = 5 * SECOND / 2;
/// One removal per this much of a moment at most: two in thirty seconds.
pub const REMOVAL_SPACING_TICKS: u64 = 15 * SECOND;
/// The audio crossfade at a join between parts. The cut sits in a measured
/// pause, so the fade only has to hide the change of room sound.
pub const JOIN_CROSSFADE_MS: u64 = 20;
/// Words a sentence starts with when it carries the previous thought on: a
/// clip does not end before one said by the same speaker, nor open on one.
const CARRIES_ON: [&str; 11] = [
    "and", "but", "because", "cause", "'cause", "or", "which", "although", "though", "whereas",
    "plus",
];
/// Openings that point back at something said before.
const POINTS_BACK: [&str; 5] = [
    "that's why",
    "that is why",
    "as i said",
    "like i said",
    "as i was saying",
];

/// Everything the cutter reads.
#[derive(Clone, Copy, Debug)]
pub struct Material<'a> {
    pub source_fingerprint: &'a str,
    pub source: &'a Source,
    /// Every word as `(start, end)`, in time order.
    pub words: &'a [(u64, u64)],
    /// Where a cut may go, when the recording's pauses were measured. Without
    /// it, cuts fall on sentence edges as the words give them.
    pub lattice: Option<&'a Lattice>,
    pub scores: Option<&'a MindUnitScores>,
    pub memory: Option<&'a MindMemory>,
    /// The programme length the brief asks for, `(min, max)` ticks.
    pub length: (u64, u64),
}

/// Which scout found a candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scout {
    Event,
    Link,
    ColdOpen,
    Theme,
    Span,
    Brief,
    Reaction,
}

/// A plan the cutter proposes, with a provisional score for ordering.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub plan: MindClipPlan,
    /// Other validated forms of the same moment the planner can choose instead.
    pub alternatives: Vec<MindClipPlan>,
    pub scout: Scout,
    pub score: i64,
}

#[derive(Clone, Debug)]
struct Unit {
    index: u64,
    start: u64,
    end: u64,
    text: String,
    hook: u8,
    landing: u8,
    /// Why a tight cut may drop it, as a plan's removal reason.
    cut: Option<&'static str>,
    /// The position of the earlier unit it cannot be understood without.
    needs: Option<usize>,
    speaker: Option<String>,
    /// Where a part may start at this line: the cut point in the pause before
    /// it, or the line's own start when pauses were not measured. `None` when
    /// there is no pause before it to cut in.
    open: Option<u64>,
    /// Where a part may end after this line, likewise.
    close: Option<u64>,
}

fn first_word(text: &str) -> String {
    text.split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|c: char| !c.is_alphanumeric() && c != '\'')
        .to_lowercase()
}

/// Whether a line carries the one before it on, judged by how it starts.
fn carries_on(text: &str) -> bool {
    CARRIES_ON.contains(&first_word(text).as_str())
}

/// Whether a line opens by pointing back at what came before.
fn points_back(text: &str) -> bool {
    let lower = text.trim_start().to_lowercase();
    carries_on(text) || POINTS_BACK.iter().any(|phrase| lower.starts_with(phrase))
}

struct Table {
    units: Vec<Unit>,
    position: BTreeMap<u64, usize>,
}

const fn level(level: Level) -> u8 {
    match level {
        Level::None => 0,
        Level::Fair => 1,
        Level::Good => 2,
        Level::Strong => 3,
    }
}

const fn removal_reason(trim: RowTrim) -> Option<&'static str> {
    match trim {
        RowTrim::Keep => None,
        RowTrim::Tangent | RowTrim::Aside => Some("tangent"),
        RowTrim::Retake => Some("repetition"),
        RowTrim::FalseStart => Some("false_start"),
        RowTrim::Filler => Some("filler"),
        RowTrim::AdRead => Some("sponsor"),
    }
}

fn unit_index(unit: &str) -> Option<u64> {
    unit.strip_prefix('s')?.parse().ok()
}

impl Table {
    fn new(material: &Material<'_>) -> Self {
        let mut units: Vec<Unit> = material
            .source
            .sentences
            .iter()
            .map(|s| {
                let edge = |ticks: u64| match material.lattice {
                    Some(lattice) => lattice.edge(material.words, ticks).map(|point| point.at),
                    None => Some(ticks),
                };
                Unit {
                    index: s.index,
                    start: s.start_ticks,
                    end: s.end_ticks,
                    text: s.text.trim().to_owned(),
                    hook: 0,
                    landing: 0,
                    cut: None,
                    needs: None,
                    speaker: s.speaker.clone(),
                    open: edge(s.start_ticks),
                    close: edge(s.end_ticks),
                }
            })
            .collect();
        let position: BTreeMap<u64, usize> = units
            .iter()
            .enumerate()
            .map(|(place, unit)| (unit.index, place))
            .collect();
        for row in material.scores.map_or(&[][..], |s| &s.units) {
            let Some(&place) = unit_index(&row.unit).and_then(|i| position.get(&i)) else {
                continue;
            };
            let needs = row
                .needs
                .as_ref()
                .and_then(|u| unit_index(u))
                .and_then(|i| position.get(&i).copied())
                .filter(|&earlier| earlier < place);
            let unit = &mut units[place];
            unit.hook = level(row.hook);
            unit.landing = level(row.landing);
            unit.cut = removal_reason(row.trim);
            unit.needs = needs;
        }
        // Older scorers occasionally called an entire passage a false start.
        // Do not remove a run of completed thoughts under a disfluency label.
        for run in units.chunk_by_mut(|a, b| a.cut == b.cut) {
            if run.len() > 2 && matches!(run[0].cut, Some("false_start" | "filler" | "repetition"))
            {
                for unit in run {
                    unit.cut = None;
                }
            }
        }
        for unit in &mut units {
            if unit.hook >= 2 || unit.landing >= 2 {
                unit.cut = None;
            }
        }
        Self { units, position }
    }

    /// The same recording with every cut mark set aside, so a moment the
    /// markup over-marked still has its plain version.
    fn plain(&self) -> Self {
        Self {
            units: self
                .units
                .iter()
                .cloned()
                .map(|unit| Unit { cut: None, ..unit })
                .collect(),
            position: self.position.clone(),
        }
    }

    fn place(&self, unit: &str) -> Option<usize> {
        unit_index(unit).and_then(|i| self.position.get(&i).copied())
    }

    /// Where a part starting at line `a` starts.
    fn first(&self, a: usize) -> u64 {
        self.units[a].open.unwrap_or(self.units[a].start)
    }

    /// Where a part ending at line `b` ends.
    fn last(&self, b: usize) -> u64 {
        self.units[b].close.unwrap_or(self.units[b].end)
    }

    /// Whether a part may end after line `b`: there is a pause to cut in,
    /// and the same speaker does not carry the thought on in the next line.
    fn may_end(&self, b: usize) -> bool {
        let unit = &self.units[b];
        if unit.close.is_none() {
            return false;
        }
        self.units.get(b + 1).is_none_or(|next| {
            let same_voice =
                unit.speaker.is_none() || next.speaker.is_none() || unit.speaker == next.speaker;
            let close = next.start.saturating_sub(unit.end) <= ADJACENT_GAP_TICKS;
            !(same_voice && close && carries_on(&next.text))
        })
    }

    /// Whether a part may start at line `a`: there is a pause to cut in, and
    /// the line does not open by pointing back at an earlier one.
    fn may_open(&self, a: usize) -> bool {
        let unit = &self.units[a];
        unit.open.is_some() && unit.needs.is_none() && !points_back(&unit.text)
    }

    /// Ranges a tight cut leaves out of `a..=b`: runs of cuttable units
    /// strictly inside it, from the cut after the kept line before to the cut
    /// before the kept line after. A run with no pause to cut in at either
    /// end stays, as does one that would save too little; and no more than
    /// one removal per [`REMOVAL_SPACING_TICKS`] is made, the longest first.
    fn removals(&self, a: usize, b: usize) -> Vec<(u64, u64, &'static str)> {
        let mut found = Vec::new();
        let mut place = a + 1;
        while place < b {
            let Some(reason) = self.units[place].cut else {
                place += 1;
                continue;
            };
            let mut last = place;
            while last + 1 < b && self.units[last + 1].cut.is_some() {
                last += 1;
            }
            if let (Some(start), Some(end)) =
                (self.units[place - 1].close, self.units[last + 1].open)
                && end.saturating_sub(start) >= MIN_REMOVAL_TICKS
            {
                found.push((start, end, reason));
            }
            place = last + 1;
        }
        let span = self.last(b).saturating_sub(self.first(a));
        let allowed = usize::try_from((span / REMOVAL_SPACING_TICKS).max(1)).unwrap_or(usize::MAX);
        if found.len() > allowed {
            found.sort_by_key(|&(start, end, _)| (std::cmp::Reverse(end - start), start));
            found.truncate(allowed);
            found.sort_by_key(|&(start, _, _)| start);
        }
        found
    }

    /// How long `a..=b` plays once its removals are taken out.
    fn kept(&self, a: usize, b: usize) -> u64 {
        let span = self.last(b).saturating_sub(self.first(a));
        let removed: u64 = self.removals(a, b).iter().map(|(s, e, _)| e - s).sum();
        span.saturating_sub(removed)
    }

    /// The source evidence a cut can actually play from this event.
    fn kept_edges(&self, (mut a, mut b): (usize, usize)) -> Option<(usize, usize)> {
        while a < b && self.units[a].cut.is_some() {
            a += 1;
        }
        while b > a && self.units[b].cut.is_some() {
            b -= 1;
        }
        self.units[a].cut.is_none().then_some((a, b))
    }

    /// The best span inside or around `a..=b` whose kept length is within
    /// `(min, max)`: open on the strongest standalone line, close on the
    /// strongest landing.
    fn fit(&self, a: usize, b: usize, length: (u64, u64)) -> Option<(usize, usize)> {
        self.fit_anchored((a, b), (a, b), length)
    }

    /// Keep audible evidence from both events, including any nearby context
    /// needed for a standalone opening.
    fn fit_anchored(
        &self,
        first: (usize, usize),
        last: (usize, usize),
        length: (u64, u64),
    ) -> Option<(usize, usize)> {
        let (a, last_start) = self.kept_edges(first)?;
        let (first_end, b) = self.kept_edges(last)?;
        let mut opening = a;
        while let Some(earlier) = self.units[opening].needs {
            if a - earlier > NEEDS_REACH_UNITS {
                // Try an independent opening inside this event instead.
                opening = a;
                break;
            }
            opening = earlier;
        }
        // Where a moment starts is the reader's judgment: open within its
        // first quarter, and later only when nothing earlier fits the length.
        // A strong line deep inside a moment is what a cold open is for.
        let near = (a + (b - a) / 4).min(last_start);
        self.search((a, b), opening..=near, first_end, length)
            .or_else(|| self.search((a, b), opening..=last_start, first_end, length))
    }

    fn search(
        &self,
        (a, b): (usize, usize),
        openings: RangeInclusive<usize>,
        first_end: usize,
        (min, max): (u64, u64),
    ) -> Option<(usize, usize)> {
        let reach = (b + EXTEND_UNITS).min(self.units.len() - 1);
        let mut best: Option<(i64, usize, usize)> = None;
        for start in openings {
            // A line that depends on an earlier one cannot open a part.
            if !self.may_open(start) || self.units[start].cut.is_some() {
                continue;
            }
            for end in start.max(first_end)..=reach {
                if self.units[end].cut.is_some() || !self.may_end(end) {
                    continue;
                }
                let kept = self.kept(start, end);
                if kept > max {
                    break;
                }
                if kept < min {
                    continue;
                }
                let span = self.last(end).saturating_sub(self.first(start));
                let removed = span.saturating_sub(kept);
                if u128::from(removed) * 100 > u128::from(span) * u128::from(MAX_CUT_PERCENT) {
                    continue;
                }
                // Prefer covering the moment, then strong edges, then length.
                let covered = i64::try_from(end.min(b) + 1 - start.max(a)).unwrap_or(0);
                let past = i64::try_from(end.saturating_sub(b)).unwrap_or(0);
                let score = 4 * i64::from(self.units[start].hook)
                    + 4 * i64::from(self.units[end].landing)
                    + 2 * covered
                    - past
                    - i64::from(start != a);
                if best.is_none_or(|(s, _, _)| score > s) {
                    best = Some((score, start, end));
                }
            }
        }
        best.map(|(_, start, end)| (start, end))
    }

    /// The strongest short stretch of `a..=b` that plays for at most `cap`.
    fn excerpt(&self, a: usize, b: usize, cap: u64) -> Option<(usize, usize)> {
        self.fit(a, b, (MIN_MONTAGE_ITEM_TICKS, cap))
    }

    fn edges(&self, a: usize, b: usize) -> i64 {
        2 * i64::from(self.units[a].hook) + 2 * i64::from(self.units[b].landing)
    }
}

/// One part as the cutter lays it out.
struct Piece {
    a: usize,
    b: usize,
    role: &'static str,
    why: String,
    /// A line inside the part that a cold open already played first, taken
    /// out here so it is not heard twice.
    lifted: Option<usize>,
}

/// The ranges a piece leaves out: its tight cuts, and the line a cold open
/// lifted to the front. `None` when the lifted line has no pause to cut at.
fn piece_removals(table: &Table, piece: &Piece) -> Option<Vec<(u64, u64, &'static str)>> {
    let mut removed = table.removals(piece.a, piece.b);
    if let Some(lift) = piece.lifted {
        let start = table.units.get(lift.checked_sub(1)?)?.close?;
        let end = table.units.get(lift + 1)?.open?;
        if removed.iter().any(|&(s, e, _)| s < end && start < e) {
            return None;
        }
        removed.push((start, end, LIFTED));
        removed.sort_by_key(|&(start, _, _)| start);
    }
    Some(removed)
}

/// The removal reason for a line a cold open lifted to the front.
const LIFTED: &str = "lifted";

/// Identify the playable edit across analyses, independent of its title or rating.
/// Roles, order and every omitted interval distinguish different cuts of the same moment.
fn plan_id(source: &str, form: &str, pieces: &[Piece], table: &Table) -> String {
    let mut key = format!("{source}|{form}");
    for piece in pieces {
        let _ = write!(
            key,
            "|{}:{}:{}",
            piece.role,
            table.first(piece.a),
            table.last(piece.b)
        );
        for (start, end, _) in piece_removals(table, piece).unwrap_or_default() {
            let _ = write!(key, "|cut:{start}:{end}");
        }
    }
    let digest = Sha256::digest(key.as_bytes());
    format!("plan_{}", hex::encode(&digest[..12]))
}

fn clipped(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(limit - 1).collect();
    out.push('…');
    out
}

struct Draft<'p> {
    form: &'static str,
    pieces: Vec<Piece>,
    /// The evidence for the join after each piece but the last.
    links: Vec<Option<String>>,
    title: Option<&'p str>,
    evidence: Vec<String>,
}

fn assemble(material: &Material<'_>, table: &Table, draft: &Draft<'_>) -> Option<MindClipPlan> {
    let mut parts = Vec::new();
    let mut duration = 0;
    for (number, piece) in draft.pieces.iter().enumerate() {
        let (start, end) = (table.units[piece.a].open?, table.units[piece.b].close?);
        let ranges = piece_removals(table, piece)?;
        let removed: Vec<Value> = ranges
            .iter()
            .map(|&(start, end, reason)| {
                if reason == LIFTED {
                    json!({
                        "start_ticks": start,
                        "end_ticks": end,
                        "reason": "repetition",
                        "note": "Heard first, as the hook",
                    })
                } else {
                    json!({ "start_ticks": start, "end_ticks": end, "reason": reason })
                }
            })
            .collect();
        let cut: u64 = ranges.iter().map(|(s, e, _)| e - s).sum();
        duration += end.saturating_sub(start).saturating_sub(cut);
        let units: Vec<String> = (piece.a..=piece.b)
            .filter(|&place| {
                Some(place) != piece.lifted
                    && (place == piece.a || place == piece.b || table.units[place].cut.is_none())
            })
            .map(|place| format!("s{}", table.units[place].index))
            .collect();
        let mut part = json!({
            "part_id": format!("p{}", number + 1),
            "role": piece.role,
            "start_ticks": start,
            "end_ticks": end,
            "units": units,
            "why": clipped(&piece.why, 400),
        });
        if !removed.is_empty() {
            part["removed"] = Value::Array(removed);
        }
        parts.push(part);
    }
    let mut joins = Vec::new();
    for (number, pair) in draft.pieces.windows(2).enumerate() {
        let (before, after) = (&pair[0], &pair[1]);
        let (end, start) = (table.last(before.b), table.first(after.a));
        let kind = if start < end {
            "reorder"
        } else if start - end <= ADJACENT_GAP_TICKS {
            "adjacent"
        } else {
            "distant"
        };
        let mut join = json!({ "after": format!("p{}", number + 1), "kind": kind });
        if kind != "adjacent" {
            join["crossfade_ms"] = json!(JOIN_CROSSFADE_MS);
            join["room_tone"] = json!(true);
        }
        if kind == "distant" {
            join["framing"] = json!("punch_in");
        }
        if let Some(link) = draft.links.get(number).cloned().flatten() {
            join["link"] = json!(clipped(&link, 200));
        }
        joins.push(join);
    }
    let first = &table.units[draft.pieces.first()?.a];
    let mut plan = json!({
        "schema_version": "clipmill.mind.clip_plan.v1",
        "source_fingerprint": material.source_fingerprint,
        "plan_id": plan_id(material.source_fingerprint, draft.form, &draft.pieces, table),
        "form": draft.form,
        "hook_text": clipped(&first.text, 160),
        "parts": parts,
        "joins": joins,
        "duration_ticks": duration,
        "evidence": draft.evidence,
        "provenance": { "by": "mind" },
    });
    if let Some(title) = draft.title {
        plan["title"] = json!(clipped(title, 120));
    }
    if first.text.is_empty() {
        plan.as_object_mut()?.remove("hook_text");
    }
    let plan: MindClipPlan = serde_json::from_value(plan).ok()?;
    let words = material.words;
    let bounds = Bounds {
        words,
        lattice: material.lattice,
        duration: Some(material.length),
        source_ticks: material.source.duration_ticks,
    };
    check(&plan, &bounds).is_empty().then_some(plan)
}

/// The form a single moment takes: tightened if anything is cut out of it.
fn single_form(table: &Table, a: usize, b: usize) -> &'static str {
    if table.removals(a, b).is_empty() {
        "straight"
    } else {
        "tightened"
    }
}

const fn clip_worthy(kind: EventKind) -> bool {
    matches!(
        kind,
        EventKind::Story
            | EventKind::Claim
            | EventKind::Insight
            | EventKind::Joke
            | EventKind::Answer
            | EventKind::Explanation
            | EventKind::DemoStep
            | EventKind::Reveal
            | EventKind::HotTake
    )
}

const fn mood_bonus(mood: EventMood) -> i64 {
    match mood {
        EventMood::Funny | EventMood::Surprising | EventMood::Emotional => 2,
        EventMood::Tense | EventMood::Inspiring => 1,
        EventMood::Serious | EventMood::Neutral => 0,
    }
}

const fn roles(kind: LinkKind) -> Option<(&'static str, &'static str)> {
    match kind {
        LinkKind::QuestionAnswer => Some(("question", "answer")),
        LinkKind::SetupPayoff | LinkKind::PromiseDelivery | LinkKind::Callback => {
            Some(("setup", "payoff"))
        }
        LinkKind::ClaimEvidence => Some(("body", "proof")),
        LinkKind::Contradiction => None,
    }
}

/// Every candidate the scouts find, checked, without duplicates, best first.
pub fn candidates(material: &Material<'_>) -> Vec<Candidate> {
    candidates_for_brief(material, "")
}

/// Search the entire script for a requested topic before limiting the candidate pool.
pub fn candidates_for_brief(material: &Material<'_>, prompt: &str) -> Vec<Candidate> {
    let table = Table::new(material);
    if table.units.is_empty() {
        return Vec::new();
    }
    // Try the original moments alongside the marked ones. With no cut marks
    // the passes are identical, so scan and score the recording only once.
    let plain = table
        .units
        .iter()
        .any(|unit| unit.cut.is_some())
        .then(|| table.plain());
    let versions = std::iter::once(&table).chain(plain.as_ref());
    let mut found = Vec::new();
    if let Some(memory) = material.memory {
        for version in versions.clone() {
            events(material, version, memory, &mut found);
            links(material, version, memory, &mut found);
            themes(material, version, memory, &mut found);
        }
    }
    reactions(material, &table, &mut found);
    brief_scout(material, &table, prompt, &mut found);
    if found.is_empty() {
        for version in versions {
            spans(material, version, &mut found);
        }
    }
    cold_opens(material, &table, &mut found);

    found.sort_by(|x, y| {
        y.score
            .cmp(&x.score)
            .then_with(|| x.plan.plan_id.as_str().cmp(y.plan.plan_id.as_str()))
    });
    let mut seen = BTreeSet::new();
    found.retain(|candidate| seen.insert(candidate.plan.plan_id.to_string()));
    if let Some(memory) = material.memory
        && found.len() > MAX_CANDIDATES
        && !memory.chapters.is_empty()
    {
        // Give every chapter a first nomination before filling the bounded pool
        // with a second from the same chapter. Earlier scores break round ties.
        let mut counts = vec![0; memory.chapters.len() + 1];
        let mut rounds = Vec::with_capacity(found.len());
        for (position, candidate) in found.into_iter().enumerate() {
            let anchor = candidate
                .plan
                .parts
                .iter()
                .max_by_key(|p| p.end_ticks.get() - p.start_ticks)
                .map_or(0, |p| p.start_ticks);
            let chapter = memory
                .chapters
                .partition_point(|c| c.end_ticks.get() <= anchor);
            rounds.push((counts[chapter], position, candidate));
            counts[chapter] += 1;
        }
        rounds.sort_unstable_by_key(|(round, position, _)| (*round, *position));
        found = rounds
            .into_iter()
            .map(|(_, _, candidate)| candidate)
            .collect();
    }
    let mut selected = distinct(found);
    selected.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.plan.plan_id.cmp(&b.plan.plan_id))
    });
    selected
}

/// What a plan plays, as source ranges with its removals taken out.
fn played(plan: &MindClipPlan) -> Vec<(u64, u64)> {
    plan.parts.iter().flat_map(crate::played).collect()
}

fn shared(a: &[(u64, u64)], b: &[(u64, u64)]) -> u64 {
    a.iter()
        .flat_map(|x| b.iter().map(move |y| (x, y)))
        .map(|(x, y)| x.1.min(y.1).saturating_sub(x.0.max(y.0)))
        .sum()
}

/// The best candidates that are not the same clip again, at most
/// [`MAX_CANDIDATES`]: one that shares half of what it plays with a better
/// one becomes an alternative if it tells the same moment in a different form.
fn distinct(found: Vec<Candidate>) -> Vec<Candidate> {
    let mut kept: Vec<(Candidate, Vec<(u64, u64)>)> = Vec::new();
    for candidate in found {
        let ranges = played(&candidate.plan);
        let length: u64 = ranges.iter().map(|(a, b)| b - a).sum();
        let repeat = kept.iter_mut().find(|(primary, other)| {
            let other_length: u64 = other.iter().map(|(a, b)| b - a).sum();
            // A theme montage is a different editorial proposition from any of
            // its individual stories, even if a long item accounts for most of it.
            let montage =
                clipmill_contracts::schemas::mind_clip_plan::MindClipPlanForm::ThemeMontage;
            (primary.plan.form == montage) == (candidate.plan.form == montage)
                && shared(&ranges, other) * 100
                    >= DUPLICATE_SHARE_PERCENT * length.min(other_length).max(1)
        });
        if let Some((primary, _)) = repeat {
            let alternate = candidate.plan;
            if primary.plan.form != alternate.form
                && primary.alternatives.len() < 3
                && !primary
                    .alternatives
                    .iter()
                    .any(|p| p.form == alternate.form)
            {
                primary.alternatives.push(alternate);
            }
        } else if kept.len() < MAX_CANDIDATES {
            kept.push((candidate, ranges));
        }
    }
    kept.into_iter().map(|(candidate, _)| candidate).collect()
}

/// Each clip-worthy event as one moment, tightened where it can be.
fn events(material: &Material<'_>, table: &Table, memory: &MindMemory, found: &mut Vec<Candidate>) {
    for event in memory.events.iter().filter(|e| clip_worthy(e.kind)) {
        let (Some(a), Some(b)) = (
            table.place(&event.first_unit),
            table.place(&event.last_unit),
        ) else {
            continue;
        };
        let Some((a, b)) = table.fit(a, b, material.length) else {
            continue;
        };
        let form = single_form(table, a, b);
        let draft = Draft {
            form,
            pieces: vec![Piece {
                lifted: None,
                a,
                b,
                role: "body",
                why: format!("{}: {}", event.kind, *event.title),
            }],
            links: Vec::new(),
            title: Some(&event.title),
            evidence: vec![event.event_id.to_string()],
        };
        if let Some(plan) = assemble(material, table, &draft) {
            found.push(Candidate {
                plan,
                alternatives: Vec::new(),
                scout: Scout::Event,
                score: table.edges(a, b) + mood_bonus(event.mood) + i64::from(form == "tightened"),
            });
        }
    }
}

/// Linked events: one moment when they are close, a stitch when they are not.
fn links(material: &Material<'_>, table: &Table, memory: &MindMemory, found: &mut Vec<Candidate>) {
    let spans: BTreeMap<&str, (usize, usize, &str)> = memory
        .events
        .iter()
        .filter_map(|e| {
            Some((
                e.event_id.as_str(),
                (
                    table.place(&e.first_unit)?,
                    table.place(&e.last_unit)?,
                    e.title.as_str(),
                ),
            ))
        })
        .collect();
    for link in &memory.links {
        let Some((first_role, second_role)) = roles(link.kind) else {
            continue;
        };
        let (Some(&(fa, fb, from_title)), Some(&(ta, tb, to_title))) = (
            spans.get(link.from_event.as_str()),
            spans.get(link.to_event.as_str()),
        ) else {
            continue;
        };
        if fb >= ta {
            continue;
        }
        let evidence = vec![
            link.link_id.to_string(),
            link.from_event.to_string(),
            link.to_event.to_string(),
        ];
        let gap = table.units[ta].start.saturating_sub(table.units[fb].end);
        let candidate = if gap < STITCH_MIN_GAP_TICKS {
            joined(material, table, [(fa, fb), (ta, tb)], from_title, evidence)
        } else {
            stitched(
                material,
                table,
                [
                    (fa, fb, first_role, from_title),
                    (ta, tb, second_role, to_title),
                ],
                &format!(
                    "{} {}: {}>{}",
                    *link.link_id, link.kind, *link.from_event, *link.to_event
                ),
                evidence,
            )
        };
        found.extend(candidate);
    }
}

fn joined(
    material: &Material<'_>,
    table: &Table,
    [first, last]: [(usize, usize); 2],
    title: &str,
    evidence: Vec<String>,
) -> Option<Candidate> {
    let (a, b) = table.fit_anchored(first, last, material.length)?;
    let form = single_form(table, a, b);
    let plan = assemble(
        material,
        table,
        &Draft {
            form,
            pieces: vec![Piece {
                lifted: None,
                a,
                b,
                role: "body",
                why: title.to_owned(),
            }],
            links: Vec::new(),
            title: Some(title),
            evidence,
        },
    )?;
    Some(Candidate {
        plan,
        alternatives: Vec::new(),
        scout: Scout::Link,
        score: table.edges(a, b) + 2,
    })
}

fn stitched(
    material: &Material<'_>,
    table: &Table,
    ends: [(usize, usize, &'static str, &str); 2],
    link: &str,
    evidence: Vec<String>,
) -> Option<Candidate> {
    let (min, max) = material.length;
    let [
        (fa, fb, first_role, from_title),
        (ta, tb, second_role, to_title),
    ] = ends;
    // The first half sets up: keep it short so the payoff has the time.
    let (fa, fb) = table.fit(fa, fb, (MIN_PART_TICKS, max * 2 / 5))?;
    let first = table.kept(fa, fb);
    // Keep the entire payoff when it fits. A longer answer may end on the
    // best available sentence boundary below the remaining time budget.
    let room = max.saturating_sub(first);
    let whole = table.kept(ta, tb);
    let mut minimum = min.saturating_sub(first).max(MIN_PART_TICKS);
    if whole <= room {
        minimum = minimum.max(whole);
    }
    let (ta, tb) = table.fit(ta, tb, (minimum, room))?;
    let plan = assemble(
        material,
        table,
        &Draft {
            form: "stitch",
            pieces: vec![
                Piece {
                    lifted: None,
                    a: fa,
                    b: fb,
                    role: first_role,
                    why: from_title.to_owned(),
                },
                Piece {
                    lifted: None,
                    a: ta,
                    b: tb,
                    role: second_role,
                    why: to_title.to_owned(),
                },
            ],
            links: vec![Some(link.to_owned())],
            title: Some(from_title),
            evidence,
        },
    )?;
    Some(Candidate {
        plan,
        alternatives: Vec::new(),
        scout: Scout::Link,
        score: table.edges(fa, tb) + 4,
    })
}

/// Whether a line names any of `names` (already lower-cased).
fn mentions(text: &str, names: &[String]) -> bool {
    let text = text.to_lowercase();
    names.iter().any(|name| text.contains(name.as_str()))
}

/// Three or more moments about one named thing, in the order they were said.
///
/// Each moment is cut around a line that names the thing, and the moments come
/// from more than one chapter: a montage is a thread through the recording,
/// not one passage chopped up.
fn themes(material: &Material<'_>, table: &Table, memory: &MindMemory, found: &mut Vec<Candidate>) {
    let events: BTreeMap<&str, (usize, usize, Option<&str>)> = memory
        .events
        .iter()
        .filter(|e| clip_worthy(e.kind))
        .filter_map(|e| {
            Some((
                e.event_id.as_str(),
                (
                    table.place(&e.first_unit)?,
                    table.place(&e.last_unit)?,
                    e.chapter_id.as_ref().map(|c| c.as_str()),
                ),
            ))
        })
        .collect();
    let (min, max) = material.length;
    for entity in &memory.entities {
        let names: Vec<String> = std::iter::once(entity.name.as_str())
            .chain(entity.aliases.iter().map(|alias| alias.as_str()))
            .map(str::to_lowercase)
            .filter(|name| name.chars().count() >= 3)
            .collect();
        // The line in each event that names it.
        let mut sightings: Vec<(usize, &str, Option<&str>)> = entity
            .events
            .iter()
            .filter_map(|id| {
                let &(a, b, chapter) = events.get(id.as_str())?;
                let line = (a..=b).find(|&place| mentions(&table.units[place].text, &names))?;
                Some((line, id.as_str(), chapter))
            })
            .collect();
        sightings.sort_unstable();
        sightings.dedup_by_key(|line| line.0);
        let chapters: BTreeSet<&str> = sightings.iter().filter_map(|line| line.2).collect();
        if sightings.len() < MIN_SCOUTED_MONTAGE_MOMENTS || chapters.len() < 2 {
            continue;
        }
        sightings.truncate(MAX_MONTAGE_PARTS);
        let cap = max / u64::try_from(sightings.len()).unwrap_or(1);
        let mut pieces: Vec<Piece> = Vec::new();
        let mut evidence = vec![entity.entity_id.to_string()];
        for &(line, id, _) in &sightings {
            let Some((a, b)) = table.excerpt(line, line, cap) else {
                continue;
            };
            // Moments, not one passage in pieces: each after the last ends.
            if pieces.last().is_some_and(|last| a <= last.b + 1) {
                continue;
            }
            pieces.push(Piece {
                lifted: None,
                a,
                b,
                role: "item",
                why: format!("{} on {}", *entity.name, id),
            });
            evidence.push(id.to_owned());
        }
        if pieces.len() < MIN_SCOUTED_MONTAGE_MOMENTS {
            continue;
        }
        let total: u64 = pieces.iter().map(|p| table.kept(p.a, p.b)).sum();
        if total < min {
            continue;
        }
        let link = format!("theme:{} {}", *entity.entity_id, *entity.name);
        let score = pieces.iter().map(|p| table.edges(p.a, p.b)).sum::<i64>()
            / i64::try_from(pieces.len()).unwrap_or(1)
            + 1;
        let links = vec![Some(link); pieces.len() - 1];
        let title = format!("Every time: {}", *entity.name);
        let draft = Draft {
            form: "theme_montage",
            pieces,
            links,
            title: Some(&title),
            evidence,
        };
        found.extend(montage_candidate(
            material,
            table,
            draft,
            Scout::Theme,
            score,
        ));
    }
}

/// Offer a complete thought around a sentence, keeping the requested line in playback.
fn around(
    material: &Material<'_>,
    table: &Table,
    line: usize,
    scout: Scout,
    reason: &str,
    bonus: i64,
) -> Option<Candidate> {
    let first = line.saturating_sub(EXTEND_UNITS);
    let (a, b) = table.fit(first, line, material.length)?;
    if b < line || table.units[line].cut.is_some() {
        return None;
    }
    let evidence = format!("s{}", table.units[line].index);
    let draft = Draft {
        form: single_form(table, a, b),
        pieces: vec![Piece {
            lifted: None,
            a,
            b,
            role: "body",
            why: reason.to_owned(),
        }],
        links: Vec::new(),
        title: None,
        evidence: vec![evidence],
    };
    assemble(material, table, &draft).map(|plan| Candidate {
        plan,
        alternatives: Vec::new(),
        scout,
        score: table.edges(a, b) + bonus,
    })
}

fn reactions(material: &Material<'_>, table: &Table, found: &mut Vec<Candidate>) {
    for cue in &material.source.cues {
        if !matches!(
            cue.kind.as_str(),
            "laughter" | "applause" | "cheering" | "gasp"
        ) {
            continue;
        }
        let Some(line) = table
            .units
            .partition_point(|u| u.start <= cue.start_ticks)
            .checked_sub(1)
        else {
            continue;
        };
        if cue.start_ticks.saturating_sub(table.units[line].end) > 5 * SECOND {
            continue;
        }
        let reason = format!(
            "The recording contains {} at the end of this line",
            cue.kind
        );
        found.extend(around(material, table, line, Scout::Reaction, &reason, 2));
    }
}

fn brief_terms(prompt: &str) -> BTreeSet<String> {
    // These common editing directions describe the desired cut rather than its subject.
    const DIRECTIONS: &[&str] = &[
        "about",
        "again",
        "all",
        "and",
        "another",
        "any",
        "around",
        "clip",
        "clips",
        "could",
        "discuss",
        "discusses",
        "every",
        "find",
        "from",
        "give",
        "include",
        "like",
        "make",
        "mention",
        "mentioned",
        "mentions",
        "moment",
        "moments",
        "montage",
        "more",
        "only",
        "part",
        "parts",
        "please",
        "says",
        "short",
        "shorter",
        "show",
        "speaker",
        "speakers",
        "talk",
        "talked",
        "talks",
        "talking",
        "that",
        "the",
        "their",
        "them",
        "they",
        "this",
        "time",
        "times",
        "when",
        "where",
        "with",
        "would",
        "you",
    ];
    prompt
        .split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|word| word.chars().count() >= 3 && !DIRECTIONS.contains(&word.as_str()))
        .take(20)
        .collect()
}

fn brief_scout(material: &Material<'_>, table: &Table, prompt: &str, found: &mut Vec<Candidate>) {
    let terms = brief_terms(prompt);
    if terms.is_empty() {
        return;
    }
    let matches = table
        .units
        .iter()
        .enumerate()
        .filter_map(|(line, u)| {
            let words = u
                .text
                .split(|c: char| !c.is_alphanumeric())
                .map(str::to_lowercase)
                .collect::<BTreeSet<_>>();
            let count = terms.intersection(&words).count();
            (count != 0).then_some((line, count))
        })
        .collect::<Vec<_>>();
    let reason = "A sentence in this cut matches the requested topic";
    // More matching words provide a stronger lead; ties retain source order.
    let mut best = matches.clone();
    best.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    for (line, n) in best.into_iter().take(MAX_CANDIDATES * 2) {
        found.extend(around(
            material,
            table,
            line,
            Scout::Brief,
            reason,
            15 + i64::try_from(n).unwrap_or(0),
        ));
    }
    let lower = prompt.to_lowercase();
    if lower.contains("every time") || lower.contains("all the") || lower.contains("montage") {
        let lines = matches.iter().map(|(line, _)| *line).collect::<Vec<_>>();
        brief_montage(material, table, &lines, &terms, found);
    }
}

fn brief_montage(
    material: &Material<'_>,
    table: &Table,
    matches: &[usize],
    terms: &BTreeSet<String>,
    found: &mut Vec<Candidate>,
) {
    let count = matches.len().min(MAX_REQUESTED_MONTAGE_PARTS);
    if count < MIN_MONTAGE_PARTS {
        return;
    }
    // A global tangent can be the exact passage explicitly requested. Keep
    // complete source sentences for each mention; the independent critic will
    // check the resulting joins against their original context.
    let mut quoted = Table::new(material);
    for unit in &mut quoted.units {
        unit.cut = None;
    }
    let n = u64::try_from(count).unwrap_or(1);
    let target = material.length.0.div_ceil(n).max(MIN_MONTAGE_ITEM_TICKS);
    let cap = material.length.1 / n;
    let mut pieces: Vec<Piece> = Vec::new();
    let mut evidence = Vec::new();
    for &line in matches {
        let (mut a, mut b) = spoken_sentence(table, line);
        let prior = a.checked_sub(1).map(|last| spoken_sentence(table, last).0);
        // A complete sentence can still have a condition or numeric correction
        // in the next sentence. An explicit request for every mention needs both.
        while let Some(next) = table.units.get(b + 1)
            && continues_claim(&next.text)
            && next.start.saturating_sub(table.units[b].end) <= ADJACENT_GAP_TICKS
        {
            b = spoken_sentence(table, b + 1).1;
            if quoted.kept(a, b) > material.length.1 {
                return;
            }
        }
        let following = (b + 1 < table.units.len()).then(|| spoken_sentence(table, b + 1).1);
        if let Some(left) = prior
            && (table.units[a].needs == a.checked_sub(1) || quoted.kept(a, b) < target)
            && quoted.kept(left, b) <= cap
        {
            a = left;
        }
        if let Some(right) = following
            && quoted.kept(a, b) < target
            && quoted.kept(a, right) <= cap
        {
            b = right;
        }
        if quoted.kept(a, b) > material.length.1 {
            return;
        }
        if let Some(previous) = pieces.last_mut()
            && (a <= previous.b
                || (a == previous.b + 1
                    && table.units[a]
                        .start
                        .saturating_sub(table.units[previous.b].end)
                        <= ADJACENT_GAP_TICKS))
        {
            // Keep consecutive mentions in their original uninterrupted passage.
            // A mention already inside it still contributes its own citation.
            previous.b = previous.b.max(b);
        } else {
            if pieces.len() == MAX_REQUESTED_MONTAGE_PARTS {
                // A partial set of mentions cannot fulfil an exhaustive request.
                return;
            }
            pieces.push(Piece {
                lifted: None,
                a,
                b,
                role: "item",
                why: "This passage contains a mention of the requested theme".to_owned(),
            });
        }
        evidence.push(format!("s{}", table.units[line].index));
    }
    let regions = coalesce_connectives(&quoted, pieces, material.length.1);
    if regions.len() < MIN_MONTAGE_PARTS {
        return;
    }
    let theme = terms.iter().cloned().collect::<Vec<_>>().join(" ");
    let links = vec![Some(format!("theme:{theme}")); regions.len() - 1];
    let draft = Draft {
        form: "theme_montage",
        pieces: regions,
        links,
        title: Some(&theme),
        evidence,
    };
    found.extend(montage_candidate(
        material,
        &quoted,
        draft,
        Scout::Brief,
        50,
    ));
}

/// Keep a short connective and its original pause between neighbouring
/// moments, preserving them as one continuous source region.
fn coalesce_connectives(table: &Table, pieces: Vec<Piece>, max_ticks: u64) -> Vec<Piece> {
    let mut duration: u64 = pieces.iter().map(|p| table.kept(p.a, p.b)).sum();
    let mut regions: Vec<Piece> = Vec::new();
    for piece in pieces {
        if let Some(previous) = regions.last_mut()
            && piece.a == previous.b + 2
            && table.units[piece.a - 1]
                .text
                .to_lowercase()
                .starts_with("and ")
            && table.units[piece.a]
                .start
                .saturating_sub(table.units[piece.a - 1].end)
                <= ADJACENT_GAP_TICKS
        {
            let added = table.units[piece.a]
                .start
                .saturating_sub(table.units[previous.b].end);
            if duration.saturating_add(added) <= max_ticks {
                previous.b = piece.b;
                duration += added;
                continue;
            }
        }
        regions.push(piece);
    }
    regions
}

/// Explicit conditions and consequences can extend a punctuated source claim.
fn continues_claim(text: &str) -> bool {
    let lower = text.trim_start().to_lowercase();
    [
        "but ",
        "except ",
        "only ",
        "unless ",
        "provided ",
        "as long as ",
        "excluding ",
        "including ",
        "that is why ",
        "that's why ",
        "in other words",
        "and not ",
        "add the ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

/// ASR segments may end mid-sentence. Read through the first real punctuation
/// and keep neighbouring words of the same sentence on both sides of the mention.
fn spoken_sentence(table: &Table, line: usize) -> (usize, usize) {
    let ends = |i: usize| {
        table.units[i]
            .text
            .trim_end_matches(['"', '\'', '”', '’', ')'])
            .ends_with(['.', '!', '?', '。', '！', '？'])
    };
    let mut a = line;
    let mut b = line;
    while a > 0
        && line - a < EXTEND_UNITS
        && !ends(a - 1)
        && table.units[a].start.saturating_sub(table.units[a - 1].end) <= ADJACENT_GAP_TICKS
    {
        a -= 1;
    }
    while b + 1 < table.units.len()
        && b - line < EXTEND_UNITS
        && !ends(b)
        && table.units[b + 1].start.saturating_sub(table.units[b].end) <= ADJACENT_GAP_TICKS
    {
        b += 1;
    }
    (a, b)
}

/// Let the planner choose which sighting opens a montage. Every offered ordering
/// has the same common theme, cited source passages and verified word boundaries.
fn montage_candidate(
    material: &Material<'_>,
    table: &Table,
    mut draft: Draft<'_>,
    scout: Scout,
    score: i64,
) -> Option<Candidate> {
    let plan = assemble(material, table, &draft)?;
    let mut alternatives = Vec::new();
    for _ in 1..draft.pieces.len().min(4) {
        draft.pieces.rotate_left(1);
        if let Some(reordered) = assemble(material, table, &draft) {
            alternatives.push(reordered);
        }
    }
    Some(Candidate {
        plan,
        alternatives,
        scout,
        score,
    })
}

/// Without a memory: moments that open on a hook and close on a landing.
fn spans(material: &Material<'_>, table: &Table, found: &mut Vec<Candidate>) {
    let mut taken = 0;
    let mut place = 0;
    while place < table.units.len() {
        let unit = &table.units[place];
        if unit.hook < 2 || unit.needs.is_some() || unit.cut.is_some() || place < taken {
            place += 1;
            continue;
        }
        if let Some((a, b)) = table.fit(place, place, material.length) {
            let form = single_form(table, a, b);
            let draft = Draft {
                form,
                pieces: vec![Piece {
                    lifted: None,
                    a,
                    b,
                    role: "body",
                    why: "Opens on a strong line and ends on a landing".to_owned(),
                }],
                links: Vec::new(),
                title: None,
                evidence: vec![format!("s{}", unit.index)],
            };
            if let Some(plan) = assemble(material, table, &draft) {
                found.push(Candidate {
                    plan,
                    alternatives: Vec::new(),
                    scout: Scout::Span,
                    score: table.edges(a, b),
                });
                taken = b + 1;
            }
        }
        place += 1;
    }
}

/// The strongest line of a single moment, lifted to open it.
fn cold_opens(material: &Material<'_>, table: &Table, found: &mut Vec<Candidate>) {
    let (min, max) = material.length;
    let mut opened = Vec::new();
    for candidate in found.iter() {
        let [part] = candidate.plan.parts.as_slice() else {
            continue;
        };
        let (Some(a), Some(b)) = (
            part.units.first().and_then(|u| table.place(u)),
            part.units.last().and_then(|u| table.place(u)),
        ) else {
            continue;
        };
        // The lifted line is taken out of the body, so it needs a pause to
        // cut at on both sides and kept neighbours, and it is never the
        // body's own first or last line.
        let Some(lift) = (a + 2..b)
            .filter(|&place| {
                let unit = &table.units[place];
                unit.hook == 3
                    && unit.needs.is_none()
                    && unit.cut.is_none()
                    && table.units[place - 1].cut.is_none()
                    && table.units[place + 1].cut.is_none()
                    && unit.open.is_some()
                    && unit.close.is_some()
                    && unit.end - unit.start <= MAX_LIFT_TICKS
                    && unit.end - unit.start >= MIN_PART_TICKS
            })
            .max_by_key(|&place| (table.units[place].hook, std::cmp::Reverse(place)))
        else {
            continue;
        };
        // The line moves rather than repeats, so the clip plays as long as
        // the body would.
        let Some((a, b)) = table.fit(a, b, (min, max)) else {
            continue;
        };
        if !(a + 1..b).contains(&lift) {
            continue;
        }
        let draft = Draft {
            form: "cold_open",
            pieces: vec![
                Piece {
                    a: lift,
                    b: lift,
                    role: "hook",
                    why: "The strongest line, heard first".to_owned(),
                    lifted: None,
                },
                Piece {
                    a,
                    b,
                    role: "body",
                    why: "Then the moment from its start, without the line again".to_owned(),
                    lifted: Some(lift),
                },
            ],
            links: vec![None],
            title: candidate.plan.title.as_deref().map(String::as_str),
            evidence: candidate
                .plan
                .evidence
                .iter()
                .map(|e| e.to_string())
                .collect(),
        };
        if let Some(plan) = assemble(material, table, &draft) {
            opened.push(Candidate {
                plan,
                alternatives: Vec::new(),
                scout: Scout::ColdOpen,
                score: candidate.score,
            });
        }
    }
    found.extend(opened);
}

#[cfg(test)]
#[path = "cutter_tests.rs"]
mod tests;
