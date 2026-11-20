//! Where a cut may go: the pauses actually heard between words.
//!
//! A recogniser's word times say roughly where words start and end. They do
//! not say whether the audio between two words is quiet. In fast conversation
//! the gap between one speaker's words often holds another voice, a breath or
//! crosstalk, and word edges can be off by a frame or more. A cut placed on a
//! word edge then clips a syllable or lands mid-sound, which is heard as a
//! snap.
//!
//! So the lattice measures the audio. Every 10 ms frame gets a level, compared
//! with the noise floor and speech level around it. In each gap between two
//! words, the longest quiet run is found near the reported edges, and the cut
//! goes at its quietest frame, kept clear of both ends. A gap with a long
//! enough pause becomes a [`CutPoint`]. A gap without one is not a place to
//! cut, and [`crate::check`] rejects a plan that cuts there.

use std::num::NonZeroU64;

use clipmill_contracts::schemas::mind_clip_plan::MindClipPlan;

use crate::SECOND;

/// One level frame, 10 ms, in ticks.
pub const FRAME_TICKS: u64 = SECOND / 100;
/// How far past the recogniser's word edges a pause is looked for.
pub const SEARCH_TICKS: u64 = 12 * FRAME_TICKS;
/// Quiet frames kept on each side of a cut, so a short fade never reaches a
/// word.
pub const MARGIN_FRAMES: usize = 4;
/// A pause at least this long is a clean place to cut.
pub const CLEAN_TICKS: u64 = 30 * FRAME_TICKS;
/// The shortest pause a cut may use.
pub const USABLE_TICKS: u64 = 15 * FRAME_TICKS;
/// How close a boundary must be to a cut point to count as on it.
pub const TOLERANCE_TICKS: u64 = FRAME_TICKS / 2;
/// The event kind cut points are stored under in the Timeline Index.
pub const EVENT_KIND: &str = "cut_point";
/// The measurement's name and version, for artifact keys: change it when
/// the measurement changes.
pub const ALGORITHM: &str = "clipmill.mind.lattice.v1";
/// Samples per frame of the 16 kHz rendition.
pub const SAMPLES_PER_FRAME_16K: usize = 160;

/// Frames of context for the local floor and speech level: 10 s.
const CONTEXT_FRAMES: usize = 1_000;
/// The level of digital silence, which keeps the logarithm finite.
const FLOOR_RMS: f64 = 1e-7;
/// The percentile of levels taken as the noise floor around a frame.
const FLOOR_PERCENTILE: u32 = 10;
/// The percentile taken as the level of speech around a frame.
const SPEECH_PERCENTILE: u32 = 95;
/// How far below the local speech level a quiet frame must be, in dB.
const SPEECH_MARGIN_DB: f32 = 10.0;

/// How good a pause is for a cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Grade {
    /// At least 150 ms of quiet: a cut with a short crossfade.
    Usable,
    /// At least 300 ms of quiet.
    Clean,
}

impl Grade {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Usable => "usable",
            Self::Clean => "clean",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "usable" => Some(Self::Usable),
            "clean" => Some(Self::Clean),
            _ => None,
        }
    }
}

/// A place in a pause where a cut may go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CutPoint {
    /// Where the cut goes: the quietest frame of the pause, clear of its ends.
    pub at: u64,
    /// The measured pause around it.
    pub pause_start: u64,
    pub pause_end: u64,
    pub grade: Grade,
}

impl CutPoint {
    pub const fn pause_ticks(&self) -> u64 {
        self.pause_end.saturating_sub(self.pause_start)
    }

    /// The point as a Timeline Index event of kind [`EVENT_KIND`]: the pause
    /// it sits in, the cut's offset into the pause in ticks as the value
    /// (exact as a float), and the grade as the label.
    pub fn event_fields(&self) -> (u64, u64, f64, &'static str) {
        let offset = u32::try_from(self.at.saturating_sub(self.pause_start)).unwrap_or(u32::MAX);
        (
            self.pause_start,
            self.pause_end,
            f64::from(offset),
            self.grade.as_str(),
        )
    }

    /// A point read back from its event fields, or `None` when they are not
    /// a cut point's.
    pub fn from_event(
        start: u64,
        end: u64,
        value: Option<f64>,
        label: Option<&str>,
    ) -> Option<Self> {
        let grade = Grade::parse(label?)?;
        let value = value?;
        if !value.is_finite() || value < 0.0 || value > f64::from(u32::MAX) {
            return None;
        }
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "checked above to be a whole number of ticks within u32"
        )]
        let offset = value.round() as u64;
        let at = start.checked_add(offset)?;
        (at <= end).then_some(Self {
            at,
            pause_start: start,
            pause_end: end,
            grade,
        })
    }
}

/// The level of one frame of 16-bit samples, in dB relative to full scale.
pub fn frame_db(samples: &[i16]) -> f32 {
    let Ok(count) = u32::try_from(samples.len()) else {
        return level(FLOOR_RMS);
    };
    if count == 0 {
        return level(FLOOR_RMS);
    }
    let energy: f64 = samples
        .iter()
        .map(|&sample| {
            let value = f64::from(sample) / 32_768.0;
            value * value
        })
        .sum();
    level((energy / f64::from(count)).sqrt() + FLOOR_RMS)
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "a level in dB fits an f32 with room to spare"
)]
fn level(rms: f64) -> f32 {
    (20.0 * rms.log10()) as f32
}

/// Frame levels with the floor and speech level around each frame.
#[derive(Clone, Debug, Default)]
pub struct Levels {
    db: Vec<f32>,
    floor: Vec<f32>,
    speech: Vec<f32>,
}

impl Levels {
    /// Levels from raw per-frame dB, one value per [`FRAME_TICKS`].
    pub fn new(raw: &[f32]) -> Self {
        let db = median3(raw);
        let count = db.len();
        if count == 0 {
            return Self::default();
        }
        let blocks = (count / CONTEXT_FRAMES).max(1);
        let mut floors = Vec::with_capacity(blocks + 1);
        let mut speeches = Vec::with_capacity(blocks + 1);
        for block in 0..=blocks {
            let low = (block * CONTEXT_FRAMES).saturating_sub(CONTEXT_FRAMES / 2);
            let high = (block * CONTEXT_FRAMES + CONTEXT_FRAMES + CONTEXT_FRAMES / 2).min(count);
            let window = if low < high { &db[low..high] } else { &db[..] };
            let mut sorted = window.to_vec();
            sorted.sort_by(f32::total_cmp);
            floors.push(percentile(&sorted, FLOOR_PERCENTILE));
            speeches.push(percentile(&sorted, SPEECH_PERCENTILE));
        }
        let floor = interpolate(&floors, count);
        let speech = interpolate(&speeches, count);
        Self { db, floor, speech }
    }

    pub fn len(&self) -> usize {
        self.db.len()
    }

    pub fn is_empty(&self) -> bool {
        self.db.is_empty()
    }

    /// Whether a frame is quiet enough to count as part of a pause: near the
    /// local floor, and well below the local speech level, so a stretch with
    /// almost no pauses (where the floor is speech) is never all quiet.
    fn quiet(&self, frame: usize) -> bool {
        let (floor, speech) = (self.floor[frame], self.speech[frame]);
        let threshold = (floor + (0.3 * (speech - floor)).max(6.0)).min(speech - SPEECH_MARGIN_DB);
        self.db[frame] < threshold
    }
}

/// A three-frame median, so a one-frame click cannot split a pause.
fn median3(raw: &[f32]) -> Vec<f32> {
    let count = raw.len();
    (0..count)
        .map(|frame| {
            let mut three = [
                raw[frame.saturating_sub(1)],
                raw[frame],
                raw[(frame + 1).min(count - 1)],
            ];
            three.sort_by(f32::total_cmp);
            three[1]
        })
        .collect()
}

/// A percentile of sorted values, interpolating between neighbours.
fn percentile(sorted: &[f32], percent: u32) -> f32 {
    let Some(last) = sorted.len().checked_sub(1) else {
        return 0.0;
    };
    let percent = usize::try_from(percent).unwrap_or(100).min(100);
    let scaled = percent * last;
    let (low, remainder) = (scaled / 100, scaled % 100);
    let high = (low + 1).min(last);
    let fraction = f32::from(u8::try_from(remainder).unwrap_or(0)) / 100.0;
    sorted[low] + (sorted[high] - sorted[low]) * fraction
}

/// One value per frame, interpolated between per-block values.
fn interpolate(per_block: &[f32], count: usize) -> Vec<f32> {
    let last = per_block.len() - 1;
    (0..count)
        .map(|frame| {
            let block = frame / CONTEXT_FRAMES;
            if block >= last {
                return per_block[last];
            }
            let within = u16::try_from(frame % CONTEXT_FRAMES).unwrap_or(0);
            let total = u16::try_from(CONTEXT_FRAMES).unwrap_or(u16::MAX);
            let fraction = f32::from(within) / f32::from(total);
            per_block[block] + (per_block[block + 1] - per_block[block]) * fraction
        })
        .collect()
}

const fn frame_of(ticks: u64) -> u64 {
    ticks.saturating_add(FRAME_TICKS / 2) / FRAME_TICKS
}

/// Every place between two words where a cut may go, in time order.
///
/// `words` are `(start, end)` ticks in time order. For each gap the search
/// runs from just before the first word's reported end to just after the next
/// word's reported start, never past either word's middle.
pub fn measure(levels: &Levels, words: &[(u64, u64)]) -> Vec<CutPoint> {
    let frames = levels.len();
    let mut points = Vec::new();
    for pair in words.windows(2) {
        let ((left_start, left_end), (right_start, right_end)) = (pair[0], pair[1]);
        let left_middle = left_start + left_end.saturating_sub(left_start) / 2;
        let right_middle = right_start + right_end.saturating_sub(right_start) / 2;
        let low_ticks = left_end.saturating_sub(SEARCH_TICKS).max(left_middle);
        let high_ticks = (right_start + SEARCH_TICKS).min(right_middle);
        if high_ticks <= low_ticks {
            continue;
        }
        let (Ok(low), Ok(high)) = (
            usize::try_from(frame_of(low_ticks)),
            usize::try_from(frame_of(high_ticks)),
        ) else {
            break;
        };
        if low >= frames {
            break;
        }
        let high = high.max(low + 1).min(frames);
        let (first, end) = longest_quiet_run(levels, low, high);
        let length = u64::try_from(end - first).unwrap_or(0) * FRAME_TICKS;
        let grade = if length >= CLEAN_TICKS {
            Grade::Clean
        } else if length >= USABLE_TICKS {
            Grade::Usable
        } else {
            continue;
        };
        let cut = quietest(levels, first, end);
        let ticks = |frame: usize| u64::try_from(frame).unwrap_or(u64::MAX) * FRAME_TICKS;
        points.push(CutPoint {
            at: ticks(cut),
            pause_start: ticks(first),
            pause_end: ticks(end),
            grade,
        });
    }
    points
}

/// The quietest frame of a pause, kept [`MARGIN_FRAMES`] clear of both ends
/// (the first one when several tie), or its middle when it is too short.
fn quietest(levels: &Levels, first: usize, end: usize) -> usize {
    let (low, high) = (first + MARGIN_FRAMES, end.saturating_sub(MARGIN_FRAMES));
    if low >= high {
        return first + (end - first) / 2;
    }
    let mut best = low;
    for frame in low..high {
        if levels.db[frame] < levels.db[best] {
            best = frame;
        }
    }
    best
}

/// The longest run of quiet frames in `low..high`, as `(first, end)`.
fn longest_quiet_run(levels: &Levels, low: usize, high: usize) -> (usize, usize) {
    let mut best = (low, low);
    let mut start = None;
    for frame in low..high {
        let quiet = levels.quiet(frame);
        if quiet && start.is_none() {
            start = Some(frame);
        }
        if let Some(first) = start
            && (!quiet || frame + 1 == high)
        {
            let end = if quiet { frame + 1 } else { frame };
            if end - first > best.1 - best.0 {
                best = (first, end);
            }
            start = None;
        }
    }
    best
}

/// A recording's cut points, searchable by time.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lattice {
    points: Vec<CutPoint>,
}

impl Lattice {
    pub fn new(mut points: Vec<CutPoint>) -> Self {
        points.sort_by_key(|point| point.at);
        points.dedup_by_key(|point| point.at);
        Self { points }
    }

    pub fn points(&self) -> &[CutPoint] {
        &self.points
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// The cut point a boundary at `ticks` sits on, if any.
    pub fn at(&self, ticks: u64) -> Option<CutPoint> {
        let from = ticks.saturating_sub(TOLERANCE_TICKS);
        let index = self.points.partition_point(|point| point.at < from);
        self.points
            .get(index)
            .copied()
            .filter(|point| point.at <= ticks + TOLERANCE_TICKS)
    }

    /// The cut point in the gap a word edge at `ticks` faces: the gap after a
    /// word ending there, or before a word starting there.
    ///
    /// `words` are the recording's words in time order. There is none at the
    /// very start or end of the speech, or where the gap has no usable pause.
    pub fn edge(&self, words: &[(u64, u64)], ticks: u64) -> Option<CutPoint> {
        let next = words.partition_point(|&(start, _)| start < ticks);
        let previous = next.checked_sub(1)?;
        let (Some(&(left_start, left_end)), Some(&(right_start, right_end))) =
            (words.get(previous), words.get(next))
        else {
            return None;
        };
        let low = left_start + left_end.saturating_sub(left_start) / 2;
        let high = right_start + right_end.saturating_sub(right_start) / 2;
        let index = self.points.partition_point(|point| point.at < low);
        self.points
            .get(index)
            .copied()
            .filter(|point| point.at <= high)
    }
}

/// Move every edge of `plan` onto the cut point in the pause at that edge, so
/// a plan written from sentence or word times (by an agent or a person) cuts
/// where the audio is quiet. An edge already on a cut point stays. A start
/// inside a word moves to the pause before the word and an end inside one to
/// the pause after it, so no word is lost. An edge with no pause there is left
/// as it is, for [`crate::check`] to report. The plan's duration follows the
/// edges when it agreed with them before.
pub fn snap(plan: &mut MindClipPlan, lattice: &Lattice, words: &[(u64, u64)]) {
    let consistent = crate::programme_ticks(plan) == plan.duration_ticks.get();
    let place = |ticks: u64, opening: bool| -> u64 {
        if lattice.at(ticks).is_some() {
            return ticks;
        }
        let index = words.partition_point(|&(start, _)| start < ticks);
        let edge = index
            .checked_sub(1)
            .map(|previous| words[previous])
            .filter(|&(start, end)| start < ticks && ticks < end)
            .map_or(ticks, |(start, end)| if opening { start } else { end });
        lattice.edge(words, edge).map_or(ticks, |point| point.at)
    };
    let nonzero = |ticks: u64, fallback: NonZeroU64| NonZeroU64::new(ticks).unwrap_or(fallback);
    for part in &mut plan.parts {
        part.start_ticks = place(part.start_ticks, true);
        part.end_ticks = nonzero(place(part.end_ticks.get(), false), part.end_ticks);
        for range in &mut part.removed {
            range.start_ticks = place(range.start_ticks, false);
            range.end_ticks = nonzero(place(range.end_ticks.get(), true), range.end_ticks);
        }
    }
    if consistent && let Some(duration) = NonZeroU64::new(crate::programme_ticks(plan)) {
        plan.duration_ticks = duration;
    }
}

#[cfg(test)]
#[path = "lattice_tests.rs"]
mod tests;
