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
//! - **theme montage**: three or more short moments about one person, product
//!   or idea;
//! - **straight**: a moment that needs nothing removed.
//!
//! Every part starts and ends on a sentence edge, a part never opens on a line
//! that needs an earlier one, and every plan passes [`crate::check`] before it
//! is returned. Choosing between candidates is the planner's and the ranker's
//! job; the score here only orders them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use clipmill_contracts::schemas::mind_clip_plan::MindClipPlan;
use clipmill_contracts::schemas::mind_memory::{EventKind, EventMood, LinkKind, MindMemory};
use clipmill_contracts::schemas::mind_unit_scores::{Level, MindUnitScores, RowTrim};
use serde_json::{Value, json};

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
/// The most moments a montage strings together.
pub const MAX_MONTAGE_PARTS: usize = 5;
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

/// Everything the cutter reads.
#[derive(Clone, Copy, Debug)]
pub struct Material<'a> {
    pub source_fingerprint: &'a str,
    pub source: &'a Source,
    /// Every word as `(start, end)`, in time order.
    pub words: &'a [(u64, u64)],
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
}

/// A plan the cutter proposes, with a provisional score for ordering.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub plan: MindClipPlan,
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
            .map(|s| Unit {
                index: s.index,
                start: s.start_ticks,
                end: s.end_ticks,
                text: s.text.trim().to_owned(),
                hook: 0,
                landing: 0,
                cut: None,
                needs: None,
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

    /// Ranges a tight cut leaves out of `a..=b`: runs of cuttable units
    /// strictly inside it, from the end of the kept line before to the start
    /// of the kept line after.
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
            let (start, end) = (self.units[place - 1].end, self.units[last + 1].start);
            if start < end {
                found.push((start, end, reason));
            }
            place = last + 1;
        }
        found
    }

    /// How long `a..=b` plays once its removals are taken out.
    fn kept(&self, a: usize, b: usize) -> u64 {
        let span = self.units[b].end.saturating_sub(self.units[a].start);
        let removed: u64 = self.removals(a, b).iter().map(|(s, e, _)| e - s).sum();
        span.saturating_sub(removed)
    }

    /// Moves the edges of `a..=b` off lines a tight cut would drop, and back
    /// over the lines its opening depends on.
    fn settle(&self, mut a: usize, mut b: usize) -> Option<(usize, usize)> {
        while a < b && self.units[a].cut.is_some() {
            a += 1;
        }
        while b > a && self.units[b].cut.is_some() {
            b -= 1;
        }
        while let Some(earlier) = self.units[a].needs {
            if a - earlier > NEEDS_REACH_UNITS {
                return None;
            }
            a = earlier;
        }
        Some((a, b))
    }

    /// The best span inside or around `a..=b` whose kept length is within
    /// `(min, max)`: open on the strongest standalone line, close on the
    /// strongest landing.
    fn fit(&self, a: usize, b: usize, length: (u64, u64)) -> Option<(usize, usize)> {
        let (a, b) = self.settle(a, b)?;
        // Where a moment starts is the reader's judgment: open within its
        // first quarter, and later only when nothing earlier fits the length.
        // A strong line deep inside a moment is what a cold open is for.
        let near = a + (b - a) / 4;
        self.search((a, b), near, length)
            .or_else(|| self.search((a, b), b, length))
    }

    fn search(
        &self,
        (a, b): (usize, usize),
        last_start: usize,
        (min, max): (u64, u64),
    ) -> Option<(usize, usize)> {
        let reach = (b + EXTEND_UNITS).min(self.units.len() - 1);
        let mut best: Option<(i64, usize, usize)> = None;
        for start in a..=last_start {
            // A line that depends on an earlier one cannot open a part.
            if self.units[start].needs.is_some() || self.units[start].cut.is_some() {
                continue;
            }
            for end in start..=reach {
                if self.units[end].cut.is_some() {
                    continue;
                }
                let kept = self.kept(start, end);
                if kept > max {
                    break;
                }
                if kept < min {
                    continue;
                }
                let span = self.units[end].end.saturating_sub(self.units[start].start);
                let removed = span.saturating_sub(kept);
                if u128::from(removed) * 100 > u128::from(span) * u128::from(MAX_CUT_PERCENT) {
                    continue;
                }
                // Prefer covering the moment, then strong edges, then length.
                let covered = i64::try_from(end.min(b) + 1 - start).unwrap_or(0);
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
}

/// A stable id from what a plan plays: FNV-1a over its form and ranges.
fn plan_id(form: &str, pieces: &[Piece], table: &Table) -> String {
    let mut text = form.to_owned();
    for piece in pieces {
        let _ = write!(
            text,
            "|{}-{}",
            table.units[piece.a].index, table.units[piece.b].index
        );
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("plan_{hash:016x}")
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
        let removed: Vec<Value> = table
            .removals(piece.a, piece.b)
            .into_iter()
            .map(|(start, end, reason)| {
                json!({ "start_ticks": start, "end_ticks": end, "reason": reason })
            })
            .collect();
        duration += table.kept(piece.a, piece.b);
        let units: Vec<String> = (piece.a..=piece.b)
            .filter(|&place| {
                place == piece.a || place == piece.b || table.units[place].cut.is_none()
            })
            .map(|place| format!("s{}", table.units[place].index))
            .collect();
        let mut part = json!({
            "part_id": format!("p{}", number + 1),
            "role": piece.role,
            "start_ticks": table.units[piece.a].start,
            "end_ticks": table.units[piece.b].end,
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
        let (end, start) = (table.units[before.b].end, table.units[after.a].start);
        let kind = if start < end {
            "reorder"
        } else if start - end <= ADJACENT_GAP_TICKS {
            "adjacent"
        } else {
            "distant"
        };
        let mut join = json!({ "after": format!("p{}", number + 1), "kind": kind });
        if kind != "adjacent" {
            join["crossfade_ms"] = json!(60);
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
        "plan_id": plan_id(draft.form, &draft.pieces, table),
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
    if found.is_empty() {
        for version in versions {
            spans(material, version, &mut found);
        }
    }
    cold_opens(material, &table, &mut found);

    let mut seen = BTreeSet::new();
    found.retain(|candidate: &Candidate| seen.insert(candidate.plan.plan_id.to_string()));
    found.sort_by(|x, y| {
        y.score
            .cmp(&x.score)
            .then_with(|| x.plan.plan_id.as_str().cmp(y.plan.plan_id.as_str()))
    });
    distinct(found)
}

/// What a plan plays, as source ranges with its removals taken out.
fn played(plan: &MindClipPlan) -> Vec<(u64, u64)> {
    let mut ranges = Vec::new();
    for part in &plan.parts {
        let mut at = part.start_ticks;
        for removed in &part.removed {
            if removed.start_ticks > at {
                ranges.push((at, removed.start_ticks));
            }
            at = at.max(removed.end_ticks.get());
        }
        if part.end_ticks.get() > at {
            ranges.push((at, part.end_ticks.get()));
        }
    }
    ranges
}

fn shared(a: &[(u64, u64)], b: &[(u64, u64)]) -> u64 {
    a.iter()
        .flat_map(|x| b.iter().map(move |y| (x, y)))
        .map(|(x, y)| x.1.min(y.1).saturating_sub(x.0.max(y.0)))
        .sum()
}

/// The best candidates that are not the same clip again, at most
/// [`MAX_CANDIDATES`]: one that shares half of what it plays with a better
/// one is left out.
fn distinct(found: Vec<Candidate>) -> Vec<Candidate> {
    let mut kept: Vec<(Candidate, Vec<(u64, u64)>)> = Vec::new();
    for candidate in found {
        if kept.len() == MAX_CANDIDATES {
            break;
        }
        let ranges = played(&candidate.plan);
        let length: u64 = ranges.iter().map(|(a, b)| b - a).sum();
        let repeats = kept.iter().any(|(_, other)| {
            let other_length: u64 = other.iter().map(|(a, b)| b - a).sum();
            shared(&ranges, other) * 100
                >= DUPLICATE_SHARE_PERCENT * length.min(other_length).max(1)
        });
        if !repeats {
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
            joined(material, table, (fa, tb), from_title, evidence)
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
    (a, b): (usize, usize),
    title: &str,
    evidence: Vec<String>,
) -> Option<Candidate> {
    let (a, b) = table.fit(a, b, material.length)?;
    let form = single_form(table, a, b);
    let plan = assemble(
        material,
        table,
        &Draft {
            form,
            pieces: vec![Piece {
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
    // The payoff is the reason for the clip: all of it, or as much as fits,
    // never a lone line cut from it.
    let room = max.saturating_sub(first);
    let whole = table.kept(ta, tb).min(room);
    let (ta, tb) = table.fit(
        ta,
        tb,
        (
            min.saturating_sub(first).max(whole).max(MIN_PART_TICKS),
            room,
        ),
    )?;
    let plan = assemble(
        material,
        table,
        &Draft {
            form: "stitch",
            pieces: vec![
                Piece {
                    a: fa,
                    b: fb,
                    role: first_role,
                    why: from_title.to_owned(),
                },
                Piece {
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
        if sightings.len() < MIN_MONTAGE_PARTS || chapters.len() < 2 {
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
                a,
                b,
                role: "item",
                why: format!("{} on {}", *entity.name, id),
            });
            evidence.push(id.to_owned());
        }
        if pieces.len() < MIN_MONTAGE_PARTS {
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
        if let Some(plan) = assemble(material, table, &draft) {
            found.push(Candidate {
                plan,
                scout: Scout::Theme,
                score,
            });
        }
    }
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
        let Some(lift) = (a + 2..=b)
            .filter(|&place| {
                let unit = &table.units[place];
                unit.hook == 3
                    && unit.needs.is_none()
                    && unit.cut.is_none()
                    && unit.end - unit.start <= MAX_LIFT_TICKS
                    && unit.end - unit.start >= MIN_PART_TICKS
            })
            .max_by_key(|&place| (table.units[place].hook, std::cmp::Reverse(place)))
        else {
            continue;
        };
        let line = table.kept(lift, lift);
        let Some((a, b)) = table.fit(a, b, (min.saturating_sub(line), max.saturating_sub(line)))
        else {
            continue;
        };
        if !(a..=b).contains(&lift) {
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
                },
                Piece {
                    a,
                    b,
                    role: "body",
                    why: "Then the moment from its start".to_owned(),
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
                scout: Scout::ColdOpen,
                score: candidate.score + 2,
            });
        }
    }
    found.extend(opened);
}

#[cfg(test)]
#[path = "cutter_tests.rs"]
mod tests;
