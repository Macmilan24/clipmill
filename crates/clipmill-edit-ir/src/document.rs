use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// The document version this crate reads and writes.
pub const IR_VERSION: &str = "ir/1";
/// The edit timebase denominator (decision D06): all time is integer ticks
/// at 1/90000, the common multiple of the broadcast and audio rates.
pub const TICKS_PER_SECOND: i64 = 90_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Timebase {
    pub num: i64,
    pub den: i64,
}

impl Default for Timebase {
    fn default() -> Self {
        Self {
            num: 1,
            den: TICKS_PER_SECOND,
        }
    }
}

/// An integer pixel rectangle in the source frame's coordinate space.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CropRect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

/// One control point of a segment's crop path, positioned in **segment-local**
/// ticks so trimming the segment's source window cannot silently re-time the
/// camera move.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CropKeyframe {
    pub t_ticks: i64,
    pub rect: CropRect,
    /// Interpolation from this keyframe to the next one.
    #[serde(default, skip_serializing_if = "CropEasing::is_linear")]
    pub easing: CropEasing,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CropEasing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl CropEasing {
    // Serde's skip_serializing_if callback takes a reference.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_linear(&self) -> bool {
        *self == Self::Linear
    }
}

/// The crop a path holds at a position along it.
///
/// The rectangle's position is interpolated between the keyframes on either
/// side and its size is held from the earlier one; before the first keyframe
/// and after the last the path holds flat. Positions are whatever unit the
/// caller keyed the path by — the renderer asks in frames, a trim asks in
/// ticks — so there is exactly one implementation of the arithmetic: this
/// one. A second, anywhere, is a parity bug with a head start.
#[must_use]
pub fn crop_along(path: &[(i64, CropRect)], at: i64) -> Option<CropRect> {
    let (first_at, first) = path.first()?;
    let (last_at, last) = path.last()?;
    if at <= *first_at {
        return Some(*first);
    }
    if at >= *last_at {
        return Some(*last);
    }
    for pair in path.windows(2) {
        let ((start, before), (end, after)) = (pair[0], pair[1]);
        if at < start || at >= end || end <= start {
            continue;
        }
        let span = end - start;
        let offset = at - start;
        return Some(CropRect {
            x: interpolate(before.x, after.x, offset, span),
            y: interpolate(before.y, after.y, offset, span),
            width: before.width,
            height: before.height,
        });
    }
    Some(*last)
}

/// Evaluate a saved path, including the easing owned by its outgoing point.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "media ticks and crop pixels stay within f64 precision; eased results are rounded to pixels"
)]
pub fn crop_along_keyframes(path: &[CropKeyframe], at: i64) -> Option<CropRect> {
    let first = path.first()?;
    let last = path.last()?;
    if at <= first.t_ticks {
        return Some(first.rect);
    }
    if at >= last.t_ticks {
        return Some(last.rect);
    }
    for pair in path.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        if at < before.t_ticks || at >= after.t_ticks {
            continue;
        }
        let offset = at - before.t_ticks;
        let span = after.t_ticks - before.t_ticks;
        let unit = offset as f64 / span as f64;
        let factor = match before.easing {
            CropEasing::Linear => unit,
            CropEasing::EaseIn => unit * unit,
            CropEasing::EaseOut => 1.0 - (1.0 - unit) * (1.0 - unit),
            CropEasing::EaseInOut => unit * unit * (3.0 - 2.0 * unit),
        };
        let ease = |from: i64, to: i64| -> i64 {
            if before.easing == CropEasing::Linear {
                interpolate(from, to, offset, span)
            } else {
                (from as f64 + (to - from) as f64 * factor).floor() as i64
            }
        };
        return Some(CropRect {
            x: ease(before.rect.x, after.rect.x),
            y: ease(before.rect.y, after.rect.y),
            width: ease(before.rect.width, after.rect.width),
            height: ease(before.rect.height, after.rect.height),
        });
    }
    Some(last.rect)
}

/// Linear interpolation in integers, rounded toward negative infinity so a
/// path evaluated forwards and backwards lands on the same pixel.
#[must_use]
pub fn interpolate(from: i64, to: i64, offset: i64, span: i64) -> i64 {
    if span <= 0 {
        return from;
    }
    from + ((to - from) * offset).div_euclid(span)
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutState {
    /// Crop to a single speaker and follow them along the crop path.
    SpeakerFill,
    /// Two deliberate viewports stacked, upper and lower; no active-speaker
    /// inference. The split sets how the height is shared.
    TwoUp,
    /// Letterbox the whole frame; the crop path is inert but preserved.
    #[default]
    Fit,
    /// A full picture — the followed crop, or the whole frame when there is no
    /// crop path — with the secondary crop inset in one corner. A screen with
    /// the speaker over it, or one person with the other's reactions.
    PictureInPicture,
}

/// What fills the frame around a fitted picture.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FitBackground {
    /// The picture itself, scaled to fill and blurred.
    Blur,
    /// One colour, `#RRGGBB`.
    Colour { colour: String },
}

/// The shape of the delivered frame. Crops are fitted to it and the render is
/// sized by it; captions keep their proportions at every shape.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameShape {
    /// 9:16, the frame of the vertical short-video apps.
    #[default]
    Vertical,
    /// 4:5, the tallest a feed shows whole.
    Portrait,
    /// 1:1.
    Square,
    /// 16:9, for a landscape player.
    Landscape,
}

impl FrameShape {
    /// Width to height, in lowest terms.
    pub const fn ratio(self) -> (u32, u32) {
        match self {
            Self::Vertical => (9, 16),
            Self::Portrait => (4, 5),
            Self::Square => (1, 1),
            Self::Landscape => (16, 9),
        }
    }

    /// The frame whose short side is `short`, in even pixels.
    pub fn frame(self, short: i64) -> (i64, i64) {
        let (width, height) = self.ratio();
        let (width, height) = (i64::from(width), i64::from(height));
        let long = |across: i64, along: i64| (short * along / across) & !1;
        if width <= height {
            (short, long(width, height))
        } else {
            (long(height, width), short)
        }
    }

    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde skip_serializing_if requires a reference"
    )]
    pub fn is_vertical(&self) -> bool {
        *self == Self::Vertical
    }
}

/// Whether two viewports sit side by side in a frame this size, rather than
/// one above the other: across a frame wider than it is tall.
pub const fn splits_across(width: i64, height: i64) -> bool {
    width > height
}

/// The corner a picture-in-picture inset sits in.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InsetCorner {
    TopLeft,
    #[default]
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Where a picture-in-picture inset sits and how large it is. It is square.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Inset {
    pub corner: InsetCorner,
    /// Its side as a share of the frame's short side, per mille.
    pub size: u16,
}

impl Default for Inset {
    fn default() -> Self {
        Self {
            corner: InsetCorner::TopRight,
            size: 360,
        }
    }
}

impl Inset {
    /// The sizes offered: from a corner badge to most of the width.
    pub const SIZES: std::ops::RangeInclusive<u16> = 200..=600;
    /// The gap between the inset and the frame's edges, per mille of the
    /// short side.
    pub const MARGIN: i64 = 40;

    /// The inset's square in pixels of a frame this size: `(x, y, side)`.
    pub fn place(self, width: i64, height: i64) -> (i64, i64, i64) {
        let short = width.min(height);
        let side = (short * i64::from(self.size) / 1_000) & !1;
        let margin = (short * Self::MARGIN / 1_000) & !1;
        let x = match self.corner {
            InsetCorner::TopLeft | InsetCorner::BottomLeft => margin,
            InsetCorner::TopRight | InsetCorner::BottomRight => width - margin - side,
        };
        // A tall frame keeps clear of the platforms' top bar and of the
        // caption and button block along the bottom; any other keeps its
        // margin at the top and clear of the captions at the bottom.
        let tall = height > width;
        let y = match self.corner {
            InsetCorner::TopLeft | InsetCorner::TopRight if tall => (height * 90 / 1_000) & !1,
            InsetCorner::TopLeft | InsetCorner::TopRight => margin,
            InsetCorner::BottomLeft | InsetCorner::BottomRight => {
                let clear = if tall { 260 } else { 200 };
                (height - (height * clear / 1_000) - side) & !1
            }
        };
        (x, y.max(0), side)
    }
}

/// A moment the camera moves in closer: the crop, whatever it is doing,
/// taken tighter about its own centre for a span of the section, then back.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Punch {
    /// Segment-local, like a crop keyframe.
    pub start_ticks: i64,
    pub end_ticks: i64,
    /// How much closer, in percent.
    pub zoom: u16,
}

impl Punch {
    /// How much closer a punch may go, in percent.
    pub const ZOOMS: std::ops::RangeInclusive<u16> = 105..=200;
    /// How long the move in or out takes: longer than one frame at any rate
    /// a recording is played at, so it is a snap rather than two keyframes
    /// on one frame.
    pub const MOVE_TICKS: i64 = 6_000;
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub state: LayoutState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub crop_path: Vec<CropKeyframe>,
    /// Lower viewport in a two-person composition and the inset of a
    /// picture-in-picture, inert in other layouts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_crop_path: Vec<CropKeyframe>,
    /// Two viewports: the first one's share of the frame, per mille — of the
    /// height when they are stacked, of the width when they sit side by side
    /// in a landscape frame. Absent is an even split; a screen share over a
    /// face is the first viewport at the recording's own shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split: Option<u16>,
    /// What fills around a fitted picture. Absent is the blurred picture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<FitBackground>,
    /// How far past fitting a fitted picture is zoomed, in percent, centred:
    /// the sides give way so the picture grows. Absent is 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zoom: Option<u16>,
    /// Where a picture-in-picture inset sits. Absent is the default corner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inset: Option<Inset>,
    /// Moments the followed crop moves in closer, in order and apart. They
    /// apply to a followed picture; the crop path under them is kept as it
    /// is, so taking them away leaves the framing that was there.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub punches: Vec<Punch>,
}

impl Layout {
    /// The split ratios offered, per mille of the height for the upper viewport.
    pub const SPLITS: std::ops::RangeInclusive<u16> = 250..=750;
    /// The zooms a fitted picture may take, in percent.
    pub const ZOOMS: std::ops::RangeInclusive<u16> = 100..=250;

    /// Old documents may request a followed crop without having saved one.
    /// Keep them editable, but make the missing framing explicit to delivery.
    pub fn needs_crop_repair(&self) -> bool {
        match self.state {
            LayoutState::Fit => false,
            LayoutState::SpeakerFill => self.crop_path.is_empty(),
            LayoutState::TwoUp => self.crop_path.is_empty() || self.secondary_crop_path.is_empty(),
            LayoutState::PictureInPicture => self.secondary_crop_path.is_empty(),
        }
    }

    /// The followed crop as it is drawn: the path, with every punch taken
    /// in about the crop's centre. Only a followed picture punches in.
    pub fn drawn_crop_path(&self) -> std::borrow::Cow<'_, [CropKeyframe]> {
        if self.punches.is_empty() || self.state != LayoutState::SpeakerFill {
            return std::borrow::Cow::Borrowed(&self.crop_path);
        }
        std::borrow::Cow::Owned(punched(&self.crop_path, &self.punches))
    }

    /// Punches in order, each at least two moves long, meeting the one
    /// before or at least two moves after it, inside the section.
    fn punches_are_valid(&self, duration: i64) -> bool {
        let span = 2 * Punch::MOVE_TICKS;
        let mut previous: Option<i64> = None;
        self.punches.iter().all(|punch| {
            let apart = previous
                .is_none_or(|end| punch.start_ticks == end || punch.start_ticks - end >= span);
            let valid = apart
                && punch.start_ticks >= 0
                && punch.end_ticks - punch.start_ticks >= span
                && punch.end_ticks <= duration
                && Punch::ZOOMS.contains(&punch.zoom);
            previous = Some(punch.end_ticks);
            valid
        })
    }

    /// The upper viewport's share of the height, per mille.
    pub fn split_permille(&self) -> u16 {
        self.split.unwrap_or(500)
    }

    /// The two viewports' lengths along a split of this length, first
    /// first. Both even, so each encodes.
    pub fn viewport_heights(&self, height: i64) -> (i64, i64) {
        let upper = (height * i64::from(self.split_permille()) / 1_000) & !1;
        (upper, height - upper)
    }

    /// The two viewports of a frame this size, as `(width, height)`, first
    /// first: stacked, or side by side when the frame is landscape.
    pub fn viewports(&self, width: i64, height: i64) -> ((i64, i64), (i64, i64)) {
        if splits_across(width, height) {
            let (first, second) = self.viewport_heights(width);
            ((first, height), (second, height))
        } else {
            let (first, second) = self.viewport_heights(height);
            ((width, first), (width, second))
        }
    }

    /// The fitted picture's zoom, in percent.
    pub fn zoom_percent(&self) -> u16 {
        self.zoom.unwrap_or(100)
    }

    /// The inset of a picture-in-picture.
    pub fn inset_or_default(&self) -> Inset {
        self.inset.unwrap_or_default()
    }

    /// Whether the style values are ones this layout can draw.
    fn style_is_valid(&self) -> bool {
        self.split.is_none_or(|split| Self::SPLITS.contains(&split))
            && self.zoom.is_none_or(|zoom| Self::ZOOMS.contains(&zoom))
            && self
                .inset
                .is_none_or(|inset| Inset::SIZES.contains(&inset.size))
            && self
                .background
                .as_ref()
                .is_none_or(|background| match background {
                    FitBackground::Blur => true,
                    FitBackground::Colour { colour } => is_hex_colour(colour),
                })
    }
}

/// `#RRGGBB`.
/// A crop path with each punch taken in: keyframes a move's length before
/// and at each edge, the crop in between tighter about its own centre, and
/// the path's own keyframes kept — tightened inside a punch — except where
/// one would fall within a move of those, which the move then stands for.
fn punched(path: &[CropKeyframe], punches: &[Punch]) -> Vec<CropKeyframe> {
    let inside = |t: i64| {
        punches
            .iter()
            .find(|punch| punch.start_ticks <= t && t < punch.end_ticks)
    };
    let at = |t: i64| crop_along_keyframes(path, t);
    let key = |t_ticks: i64, rect: CropRect| CropKeyframe {
        t_ticks,
        rect,
        easing: CropEasing::Linear,
    };
    let mut moves = Vec::with_capacity(punches.len() * 4);
    for punch in punches {
        let (start, end) = (punch.start_ticks, punch.end_ticks);
        let before = start - Punch::MOVE_TICKS;
        if before >= 0
            && inside(before).is_none()
            && let Some(rect) = at(before)
        {
            moves.push(key(before, rect));
        }
        if let Some(rect) = at(start) {
            moves.push(key(start, tighter(rect, punch.zoom)));
        }
        if let Some(rect) = at(end - Punch::MOVE_TICKS) {
            moves.push(key(end - Punch::MOVE_TICKS, tighter(rect, punch.zoom)));
        }
        if inside(end).is_none()
            && let Some(rect) = at(end)
        {
            moves.push(key(end, rect));
        }
    }
    let clear = |t: i64| {
        moves
            .iter()
            .all(|moved| (moved.t_ticks - t).abs() >= Punch::MOVE_TICKS)
    };
    let mut keys: Vec<CropKeyframe> = path
        .iter()
        .filter(|key| clear(key.t_ticks))
        .map(|key| CropKeyframe {
            rect: inside(key.t_ticks).map_or(key.rect, |punch| tighter(key.rect, punch.zoom)),
            ..*key
        })
        .collect();
    keys.extend(moves);
    keys.sort_by_key(|key| key.t_ticks);
    keys
}

/// A crop `zoom` percent closer, about its own centre, keeping its shape.
fn tighter(rect: CropRect, zoom: u16) -> CropRect {
    let scale = |length: i64| ((length * 100 / i64::from(zoom.max(100))) & !1).max(2);
    let (width, height) = (scale(rect.width), scale(rect.height));
    CropRect {
        x: rect.x + (rect.width - width) / 2,
        y: rect.y + (rect.height - height) / 2,
        width,
        height,
    }
}

/// Whether what is left of a punch is still long enough to be one.
fn lasts(punch: &Punch) -> bool {
    punch.end_ticks - punch.start_ticks >= 2 * Punch::MOVE_TICKS
}

/// Punches after a segment's source window moved: kept where the new window
/// still plays them, cut to it, and counted from its new start; one left too
/// short to be a punch is dropped.
pub(crate) fn retime_punches(
    punches: &[Punch],
    old_in: i64,
    new_in: i64,
    new_duration: i64,
) -> Vec<Punch> {
    punches
        .iter()
        .filter_map(|punch| {
            let start = (old_in + punch.start_ticks - new_in).max(0);
            let end = (old_in + punch.end_ticks - new_in).min(new_duration);
            Some(Punch {
                start_ticks: start,
                end_ticks: end,
                ..*punch
            })
            .filter(lasts)
        })
        .collect()
}

/// Punches either side of a split at `at`, the tail's counted from it.
pub(crate) fn split_punches(punches: &[Punch], at: i64) -> (Vec<Punch>, Vec<Punch>) {
    let head = punches
        .iter()
        .filter(|punch| punch.start_ticks < at)
        .map(|punch| Punch {
            end_ticks: punch.end_ticks.min(at),
            ..*punch
        })
        .filter(lasts)
        .collect();
    let tail = punches
        .iter()
        .filter(|punch| punch.end_ticks > at)
        .map(|punch| Punch {
            start_ticks: (punch.start_ticks - at).max(0),
            end_ticks: punch.end_ticks - at,
            ..*punch
        })
        .filter(lasts)
        .collect();
    (head, tail)
}

fn is_hex_colour(hex: &str) -> bool {
    hex.len() == 7 && hex.starts_with('#') && hex[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// One span of one source, placed on the program timeline by its position in
/// the segment list rather than by a stored offset.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VideoSegment {
    pub segment_id: String,
    /// `sha256:`-prefixed fingerprint of the registered source.
    pub source_fingerprint: String,
    pub in_ticks: i64,
    pub out_ticks: i64,
    pub layout: Layout,
}

impl VideoSegment {
    pub fn duration_ticks(&self) -> i64 {
        self.out_ticks.saturating_sub(self.in_ticks)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VideoTrack {
    /// The delivered frame's shape. Absent is 9:16.
    #[serde(default, skip_serializing_if = "FrameShape::is_vertical")]
    pub shape: FrameShape,
    /// Requested soft-cut duration. Zero preserves legacy hard cuts. The
    /// renderer bounds each blend by its incoming shot's frame allocation.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub transition_ticks: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub segments: Vec<VideoSegment>,
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if requires a reference"
)]
fn is_zero(value: &i64) -> bool {
    *value == 0
}

/// One word with its own timing, so text selection is time selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionWord {
    pub text: String,
    pub start_ticks: i64,
    pub end_ticks: i64,
    /// Which word this is, across both presentations.
    ///
    /// The reading cues and the burned-in cues are two groupings of one word
    /// list, and a correction belongs to the word, not to a grouping. The id
    /// is what lets one correction land in both; the projection mints it from
    /// the transcript's word index, and a document that predates ids is given
    /// them by [`EditDocument::assign_word_ids`] from timing. `None` is only
    /// ever a document nobody has migrated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word_id: Option<String>,
    /// A key word, set in the accent colour so it stands out of its line.
    #[serde(default, skip_serializing_if = "is_false")]
    pub emphasis: bool,
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if requires a reference"
)]
fn is_false(value: &bool) -> bool {
    !*value
}

/// One rendered line. Line breaks are **decided once and stored here** — the
/// parity keystone: preview and render must never re-wrap text independently,
/// because two wrappers eventually disagree about which word is on which line.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionLine {
    pub words: Vec<CaptionWord>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptionRegion {
    #[default]
    LowerSafe,
    UpperSafe,
    Center,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptionAnimation {
    #[default]
    None,
    /// Highlight each word as it is spoken, from the word timings.
    Karaoke,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionCue {
    pub cue_id: String,
    pub start_ticks: i64,
    pub end_ticks: i64,
    pub region: CaptionRegion,
    pub anim: CaptionAnimation,
    pub lines: Vec<CaptionLine>,
    /// Where this cue sits, when it was placed by hand rather than by its
    /// region. Overrides the region and the clip-wide position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<CaptionPosition>,
}

/// A caption's centre on the frame, in thousandths of its width and height.
///
/// Thousandths rather than pixels so a position means the same place at any
/// output size, and integers so two renders of one document agree exactly.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionPosition {
    pub x: u32,
    pub y: u32,
}

impl CaptionPosition {
    /// The largest coordinate: the right or bottom edge of the frame.
    pub const FULL: u32 = 1_000;

    pub fn is_valid(self) -> bool {
        self.x <= Self::FULL && self.y <= Self::FULL
    }
}

/// How the word being spoken is marked while its caption is on screen.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HighlightStyle {
    /// Words take the spoken colour as they are said and keep it.
    #[default]
    Fill,
    /// Only the word being said takes the spoken colour.
    Word,
    /// The word being said sits on a box in the spoken colour.
    Box,
    /// The word being said grows briefly, in the spoken colour.
    Pop,
    /// The word being said is underlined, in the spoken colour.
    Underline,
}

impl CaptionCue {
    /// Every word in reading order, flattened across the stored line breaks.
    pub fn words(&self) -> impl Iterator<Item = &CaptionWord> {
        self.lines.iter().flat_map(|line| line.words.iter())
    }

    pub fn word_count(&self) -> usize {
        self.lines.iter().map(|line| line.words.len()).sum()
    }

    /// Re-flow this cue's words into lines of the given word counts.
    pub(crate) fn reflow(&mut self, line_word_counts: &[usize]) -> Result<(), DocumentError> {
        let words = self.words().cloned().collect::<Vec<_>>();
        if line_word_counts.iter().sum::<usize>() != words.len() {
            return Err(DocumentError::LineBreaksDoNotCoverWords);
        }
        if line_word_counts.contains(&0) {
            return Err(DocumentError::EmptyCaptionLine);
        }
        let mut remaining = words.into_iter();
        self.lines = line_word_counts
            .iter()
            .map(|count| CaptionLine {
                words: remaining.by_ref().take(*count).collect(),
            })
            .collect();
        Ok(())
    }

    pub(crate) fn line_word_counts(&self) -> Vec<usize> {
        self.lines.iter().map(|line| line.words.len()).collect()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionTrack {
    /// Named style preset; the style itself lives in the caption presets, not
    /// in every document.
    pub style_ref: String,
    /// Clip-wide look adjustments over the named preset. The reading sidecar
    /// keeps the original word case; these affect the burned-in picture.
    #[serde(default, skip_serializing_if = "CaptionOptions::is_default")]
    pub options: CaptionOptions,
    /// What a **reader** gets. Every sidecar is written from this list and only
    /// this list, because a sidecar is what a viewer who cannot hear is left
    /// with — so it carries the conservative grouping, always.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cues: Vec<CaptionCue>,
    /// What a **watcher** gets, when the two should differ. The kinetic
    /// grouping is burned into the picture; absent, the reading cues are burned
    /// in instead, which is the behaviour every document had before this field
    /// existed.
    ///
    /// Two lists rather than one because the caption engine produces two
    /// groupings of one token array and this is where they would otherwise
    /// collapse back into one. A burn-in that inherited the reading grouping is
    /// merely conservative; a sidecar that inherited the kinetic one is the
    /// divergence the caption engine exists to prevent — so the asymmetry is
    /// deliberate and the sidecar side is the one that is never negotiable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub burn_in: Vec<CaptionCue>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptionCase {
    #[default]
    Original,
    Upper,
    Lower,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionOptions {
    /// Override the preset's spoken-word sweep without changing typography.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight_spoken_word: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spoken: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unspoken: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<String>,
    #[serde(default, skip_serializing_if = "is_original_case")]
    pub text_case: CaptionCase,
    /// How the spoken word is marked, when it is. Absent is the sweep.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight_style: Option<HighlightStyle>,
    /// One of the caption fonts, by family name. Absent is the look's own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// Outline thickness at the design height, in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline_width: Option<u32>,
    /// Drop-shadow offset at the design height, in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_depth: Option<u32>,
    /// How opaque a boxed look's plate is, in percent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plate_opacity: Option<u32>,
    /// The colour key words are set in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    /// Where every caption sits, unless a cue was placed on its own. Absent
    /// leaves each cue in its region.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<CaptionPosition>,
    /// The most words the on-screen captions were last grouped into, which
    /// is what the editor shows as chosen. The cues themselves carry the
    /// grouping; this only remembers the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words_on_screen: Option<u32>,
}

impl CaptionOptions {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

// Serde's skip_serializing_if callback takes a reference.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_original_case(value: &CaptionCase) -> bool {
    *value == CaptionCase::Original
}

/// Which of the two groupings a cue-scoped command is addressed to.
///
/// A cue id names a cue in one list; the same id can name a different cue in
/// the other, because each grouping numbers its own. A command that split or
/// merged "the cue called `cue_2`" therefore has to say which `cue_2`, and the
/// default is the reading list because that is what every command meant
/// before the burned-in list existed.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Presentation {
    /// The reading cues, from which every sidecar is written.
    #[default]
    Reading,
    /// The kinetic cues burned into the picture.
    BurnIn,
}

impl Presentation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reading => "reading",
            Self::BurnIn => "burn_in",
        }
    }
}

impl CaptionTrack {
    /// The cues that are burned into the picture: the kinetic grouping when the
    /// document carries one, and the reading cues when it does not.
    pub fn burned(&self) -> &[CaptionCue] {
        if self.burn_in.is_empty() {
            &self.cues
        } else {
            &self.burn_in
        }
    }

    /// Which list `burned()` answers from, so a surface showing those cues
    /// can address its cue-scoped commands to the same list.
    pub fn burned_presentation(&self) -> Presentation {
        if self.burn_in.is_empty() {
            Presentation::Reading
        } else {
            Presentation::BurnIn
        }
    }

    pub fn list(&self, presentation: Presentation) -> &[CaptionCue] {
        match presentation {
            Presentation::Reading => &self.cues,
            Presentation::BurnIn => &self.burn_in,
        }
    }

    pub fn list_mut(&mut self, presentation: Presentation) -> &mut Vec<CaptionCue> {
        match presentation {
            Presentation::Reading => &mut self.cues,
            Presentation::BurnIn => &mut self.burn_in,
        }
    }

    /// Every word of every cue in both groupings, mutably.
    pub fn words_mut(&mut self) -> impl Iterator<Item = &mut CaptionWord> {
        self.cues
            .iter_mut()
            .chain(self.burn_in.iter_mut())
            .flat_map(|cue| cue.lines.iter_mut())
            .flat_map(|line| line.words.iter_mut())
    }

    /// Every word of every cue in both groupings.
    pub fn words(&self) -> impl Iterator<Item = &CaptionWord> {
        self.cues
            .iter()
            .chain(self.burn_in.iter())
            .flat_map(|cue| cue.lines.iter())
            .flat_map(|line| line.words.iter())
    }
}

/// One automation point on the program-time gain curve. Gain is a measurement
/// in decibels and is legitimately real-valued; its *position* is ticks.
/// Everything one list of cues has to satisfy on its own.
///
/// Run per list rather than over both at once: the two groupings cover the same
/// span, so cue ids repeat across them and their windows interleave. Checking
/// them together would report every kinetic cue as a duplicate of a reading one.
fn validate_cues(cues: &[CaptionCue]) -> Result<(), DocumentError> {
    let mut seen_cues = Vec::with_capacity(cues.len());
    let mut seen_words: Vec<&str> = Vec::new();
    let mut previous_end: Option<i64> = None;
    for cue in cues {
        if cue.cue_id.is_empty() {
            return Err(DocumentError::EmptyIdentifier);
        }
        if seen_cues.contains(&cue.cue_id.as_str()) {
            return Err(DocumentError::DuplicateCue(cue.cue_id.clone()));
        }
        seen_cues.push(cue.cue_id.as_str());
        if cue.start_ticks < 0 || cue.end_ticks <= cue.start_ticks {
            return Err(DocumentError::EmptyCue(cue.cue_id.clone()));
        }
        if previous_end.is_some_and(|end| end > cue.start_ticks) {
            return Err(DocumentError::OverlappingCues(cue.cue_id.clone()));
        }
        previous_end = Some(cue.end_ticks);
        if cue.lines.is_empty() || cue.lines.iter().any(|line| line.words.is_empty()) {
            return Err(DocumentError::EmptyCaptionLine);
        }
        if cue.position.is_some_and(|position| !position.is_valid()) {
            return Err(DocumentError::InvalidCaptionOptions);
        }
        let mut word_cursor: Option<i64> = None;
        for word in cue.words() {
            if word.text.is_empty() || word.end_ticks <= word.start_ticks {
                return Err(DocumentError::EmptyCaptionWord(cue.cue_id.clone()));
            }
            if word.start_ticks < cue.start_ticks || word.end_ticks > cue.end_ticks {
                return Err(DocumentError::WordOutsideCue(cue.cue_id.clone()));
            }
            if word_cursor.is_some_and(|cursor| cursor > word.start_ticks) {
                return Err(DocumentError::UnorderedCaptionWords(cue.cue_id.clone()));
            }
            word_cursor = Some(word.end_ticks);
            if let Some(word_id) = &word.word_id {
                if word_id.is_empty() {
                    return Err(DocumentError::EmptyIdentifier);
                }
                if seen_words.contains(&word_id.as_str()) {
                    return Err(DocumentError::DuplicateWord(word_id.clone()));
                }
                seen_words.push(word_id.as_str());
            }
        }
    }
    Ok(())
}

/// The one thing two groupings of one word list may never disagree on.
///
/// A word carries the same id in both presentations because it is the same
/// word; a correction lands in both because it is addressed to the id. If the
/// two ever held different text under one id, a viewer would read one word
/// and a reader the other, which is the divergence the caption engine's whole
/// shape exists to make impossible — so it is refused here rather than
/// discovered on a sidecar.
fn validate_shared_words(track: &CaptionTrack) -> Result<(), DocumentError> {
    let mut reading: Vec<(&str, &str)> = Vec::new();
    for word in track.cues.iter().flat_map(CaptionCue::words) {
        if let Some(word_id) = &word.word_id {
            reading.push((word_id.as_str(), word.text.as_str()));
        }
    }
    for word in track.burn_in.iter().flat_map(CaptionCue::words) {
        let Some(word_id) = &word.word_id else {
            continue;
        };
        if let Some((_, text)) = reading.iter().find(|(id, _)| *id == word_id.as_str())
            && *text != word.text.as_str()
        {
            return Err(DocumentError::DivergentWord(word_id.clone()));
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GainPoint {
    pub t_ticks: i64,
    pub gain_db: f64,
}

/// The gain a curve holds at a program tick: linear in decibels between the
/// points on either side, held flat before the first and after the last, and
/// nothing when there is no automation. This is the rule the renderer's
/// volume expression writes, so a trim that keeps this value at its new
/// boundary keeps what would have been heard there.
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "ticks of a clip, far inside a double's exact integers"
)]
pub fn gain_at(curve: &[GainPoint], t_ticks: i64) -> Option<f64> {
    let first = curve.first()?;
    let last = curve.last()?;
    if t_ticks <= first.t_ticks {
        return Some(first.gain_db);
    }
    if t_ticks >= last.t_ticks {
        return Some(last.gain_db);
    }
    for pair in curve.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        if t_ticks < before.t_ticks || t_ticks >= after.t_ticks {
            continue;
        }
        let span = (after.t_ticks - before.t_ticks) as f64;
        let offset = (t_ticks - before.t_ticks) as f64;
        return Some(before.gain_db + (after.gain_db - before.gain_db) * offset / span);
    }
    Some(last.gain_db)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AudioTrack {
    pub target_lufs: f64,
    pub true_peak_dbtp: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gain_curve: Vec<GainPoint>,
    /// Music under the voice, dropping wherever someone speaks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<MusicBed>,
    /// The voice cleaned before it is mixed: rumble cut, noise lowered,
    /// level evened.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cleanup: Option<VoiceCleanup>,
}

impl Default for AudioTrack {
    fn default() -> Self {
        Self {
            target_lufs: -14.0,
            true_peak_dbtp: -1.0,
            gain_curve: Vec::new(),
            music: None,
            cleanup: None,
        }
    }
}

/// A sound played under the whole clip, from one of the clip's assets.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MusicBed {
    /// The sound's content hash, one of the clip's assets.
    pub asset: String,
    /// Its level where nobody speaks, in decibels.
    pub level_db: f64,
    /// How much further it drops under speech, in decibels.
    pub duck_db: f64,
    /// Where in the sound the clip starts.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub offset_ticks: i64,
}

impl MusicBed {
    pub const LEVELS: std::ops::RangeInclusive<f64> = -40.0..=0.0;
    pub const DUCKS: std::ops::RangeInclusive<f64> = -30.0..=0.0;

    fn is_valid(&self, assets: &[Asset]) -> bool {
        Self::LEVELS.contains(&self.level_db)
            && Self::DUCKS.contains(&self.duck_db)
            && self.offset_ticks >= 0
            && assets.iter().any(|asset| asset.hash == self.asset)
    }
}

/// How much the voice is cleaned.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceCleanup {
    /// A quiet room: rumble cut, a little hiss lowered, level evened.
    Light,
    /// A noisy one: more noise lowered, sibilance softened, level held.
    Strong,
}

/// Something laid over the program for a span of it: a title, a label.
///
/// Its span is program time, like a cue's, so a cut moves it with the
/// material around it and takes whatever it removed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Overlay {
    pub overlay_id: String,
    pub start_ticks: i64,
    pub end_ticks: i64,
    pub content: OverlayContent,
}

/// What an overlay shows.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OverlayContent {
    /// Words set in the clip's caption font, by the same renderer as the
    /// captions. Lines break where the text says, never elsewhere.
    Text {
        text: String,
        /// A hook opens the clip and names what it is about; any other text
        /// is a label.
        #[serde(default, skip_serializing_if = "TextRole::is_label")]
        role: TextRole,
        /// Where its centre sits, per mille of the frame's width and height.
        x: u16,
        y: u16,
        /// Its size at the 1920-pixel design height, as a caption's.
        size: u16,
        /// `#RRGGBB`.
        colour: String,
        /// An opaque plate behind it, `#RRGGBB`; absent draws an outline.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        plate: Option<String>,
    },
    /// A colour emoji, drawn from its pinned picture.
    Emoji {
        /// Its code point, as the pinned picture names it: `1f525`.
        emoji: String,
        /// Where its centre sits, per mille of the frame's width and height.
        x: u16,
        y: u16,
        /// Its side as a share of the frame's short side, per mille.
        size: u16,
    },
}

/// What a text overlay is for.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TextRole {
    Hook,
    #[default]
    Label,
}

impl TextRole {
    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde skip_serializing_if requires a reference"
    )]
    pub fn is_label(&self) -> bool {
        *self == Self::Label
    }
}

impl Overlay {
    /// The sizes a text may be set at, at the design height.
    pub const TEXT_SIZES: std::ops::RangeInclusive<u16> = 24..=240;

    /// Whether it is words, set by the caption renderer, rather than a picture.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self.content, OverlayContent::Text { .. })
    }

    /// The most characters a text may hold.
    pub const TEXT_LENGTH: usize = 160;
    /// The sizes an emoji may be, per mille of the frame's short side.
    pub const EMOJI_SIZES: std::ops::RangeInclusive<u16> = 60..=400;

    fn is_valid(&self) -> bool {
        match &self.content {
            OverlayContent::Text {
                text,
                x,
                y,
                size,
                colour,
                plate,
                ..
            } => {
                !text.trim().is_empty()
                    && text.chars().count() <= Self::TEXT_LENGTH
                    && !text
                        .chars()
                        .any(|character| matches!(character, '{' | '}' | '\\'))
                    && !text
                        .chars()
                        .any(|character| character.is_control() && character != '\n')
                    && *x <= 1_000
                    && *y <= 1_000
                    && Self::TEXT_SIZES.contains(size)
                    && is_hex_colour(colour)
                    && plate.as_deref().is_none_or(is_hex_colour)
            }
            OverlayContent::Emoji { emoji, x, y, size } => {
                (4..=40).contains(&emoji.len())
                    && emoji
                        .bytes()
                        .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f' | b'_'))
                    && *x <= 1_000
                    && *y <= 1_000
                    && Self::EMOJI_SIZES.contains(size)
            }
        }
    }
}

/// What marks a clip as its creator's, over every frame of it.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    /// A bar along one edge that fills as the clip plays.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressBar>,
    /// A picture in one corner, from an asset the clip lists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<Logo>,
}

/// The edge a progress bar runs along.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BarEdge {
    Top,
    #[default]
    Bottom,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressBar {
    /// `#RRGGBB`.
    pub colour: String,
    pub edge: BarEdge,
    /// Its thickness at the 1920-pixel design height.
    pub thickness: u16,
}

impl ProgressBar {
    pub const THICKNESSES: std::ops::RangeInclusive<u16> = 4..=40;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Logo {
    /// The picture's content hash, one of the clip's assets.
    pub asset: String,
    pub corner: InsetCorner,
    /// Its longer side as a share of the frame's short side, per mille.
    pub size: u16,
    /// How opaque it is, in percent.
    pub opacity: u8,
}

impl Logo {
    pub const SIZES: std::ops::RangeInclusive<u16> = 60..=300;
    pub const OPACITIES: std::ops::RangeInclusive<u8> = 20..=100;
}

impl Brand {
    pub fn is_empty(&self) -> bool {
        self.progress.is_none() && self.logo.is_none()
    }

    fn is_valid(&self, assets: &[Asset]) -> bool {
        self.progress.as_ref().is_none_or(|bar| {
            is_hex_colour(&bar.colour) && ProgressBar::THICKNESSES.contains(&bar.thickness)
        }) && self.logo.as_ref().is_none_or(|logo| {
            Logo::SIZES.contains(&logo.size)
                && Logo::OPACITIES.contains(&logo.opacity)
                && assets.iter().any(|asset| asset.hash == logo.asset)
        })
    }
}

/// An asset referenced by content hash, carrying the licence record that lets
/// the render manifest state its rights position without guessing.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub hash: String,
    pub license: String,
}

/// Why the director cut here. Kept out of every render path so that
/// explanation can never perturb pixels (book ch. 17).
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rationale {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EditDocument {
    pub version: String,
    pub timebase: Timebase,
    pub video: VideoTrack,
    pub captions: CaptionTrack,
    pub audio: AudioTrack,
    /// Titles and labels over the program, bottom first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlays: Vec<Overlay>,
    /// A progress bar and a logo over every frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<Brand>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<Asset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<Rationale>,
    /// What the clip is called, when somebody named it. Kept out of every
    /// render path like the rationale: renaming a clip is not a new picture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Default for EditDocument {
    fn default() -> Self {
        Self {
            version: IR_VERSION.to_owned(),
            timebase: Timebase::default(),
            video: VideoTrack::default(),
            captions: CaptionTrack::default(),
            audio: AudioTrack::default(),
            overlays: Vec::new(),
            brand: None,
            assets: Vec::new(),
            rationale: None,
            title: None,
        }
    }
}

impl EditDocument {
    pub fn from_canonical_json(bytes: &[u8]) -> Result<Self, DocumentError> {
        let document: Self = serde_json::from_slice(bytes)
            .map_err(|error| DocumentError::Json(error.to_string()))?;
        document.validate()?;
        Ok(document)
    }

    /// RFC 8785 canonical bytes — the durable and content-addressed form.
    pub fn to_canonical_json(&self) -> Result<Vec<u8>, DocumentError> {
        serde_json_canonicalizer::to_vec(self)
            .map_err(|error| DocumentError::Json(error.to_string()))
    }

    /// The document as the render and preview interpreters see it: everything
    /// except `rationale`. Two documents with equal projections must produce
    /// identical pixels, so re-explaining an edit can never invalidate a
    /// render cache or move a frame.
    pub fn render_projection(&self) -> Result<Value, DocumentError> {
        let mut value =
            serde_json::to_value(self).map_err(|error| DocumentError::Json(error.to_string()))?;
        if let Some(object) = value.as_object_mut() {
            object.remove("rationale");
            object.remove("title");
        }
        Ok(value)
    }

    /// Total program duration: segments are laid end to end, so this is the
    /// sum of their source windows.
    pub fn program_duration_ticks(&self) -> i64 {
        self.video
            .segments
            .iter()
            .map(VideoSegment::duration_ticks)
            .fold(0, i64::saturating_add)
    }

    /// Program start tick of each segment, in order.
    pub fn segment_program_starts(&self) -> Vec<i64> {
        let mut starts = Vec::with_capacity(self.video.segments.len());
        let mut cursor = 0_i64;
        for segment in &self.video.segments {
            starts.push(cursor);
            cursor = cursor.saturating_add(segment.duration_ticks());
        }
        starts
    }

    /// Map a program tick to the source it plays: `(segment index, source
    /// tick)`. This is the only sanctioned program-to-source conversion; the
    /// preview and the render compiler must not each invent their own.
    pub fn program_to_source(&self, program_ticks: i64) -> Option<(usize, i64)> {
        if program_ticks < 0 {
            return None;
        }
        let mut cursor = 0_i64;
        for (index, segment) in self.video.segments.iter().enumerate() {
            let duration = segment.duration_ticks();
            if program_ticks < cursor.saturating_add(duration) {
                let offset = program_ticks.saturating_sub(cursor);
                return Some((index, segment.in_ticks.saturating_add(offset)));
            }
            cursor = cursor.saturating_add(duration);
        }
        None
    }

    /// Inverse of [`Self::program_to_source`] for a tick inside one segment.
    pub fn source_to_program(&self, segment_index: usize, source_ticks: i64) -> Option<i64> {
        let segment = self.video.segments.get(segment_index)?;
        if source_ticks < segment.in_ticks || source_ticks >= segment.out_ticks {
            return None;
        }
        let start = *self.segment_program_starts().get(segment_index)?;
        Some(start.saturating_add(source_ticks.saturating_sub(segment.in_ticks)))
    }

    /// Position of a segment by identifier.
    pub fn segment_index(&self, segment_id: &str) -> Result<usize, DocumentError> {
        self.video
            .segments
            .iter()
            .position(|segment| segment.segment_id == segment_id)
            .ok_or_else(|| DocumentError::UnknownSegment(segment_id.to_owned()))
    }

    /// Position of a caption cue by identifier, in the named presentation.
    pub fn cue_index(
        &self,
        presentation: Presentation,
        cue_id: &str,
    ) -> Result<usize, DocumentError> {
        self.captions
            .list(presentation)
            .iter()
            .position(|cue| cue.cue_id == cue_id)
            .ok_or_else(|| DocumentError::UnknownCue(cue_id.to_owned()))
    }

    /// Give every word an identity, where it has none.
    ///
    /// The migration for documents that predate word ids. The two groupings
    /// are two arrangements of one word list, so a word is the same word in
    /// both when it starts at the same tick — timing is what the projection
    /// copied into each — and its id is derived from that tick. Words that
    /// already carry an id are left alone; the id is a name, not a position,
    /// and renaming would break a correction somebody addressed to it.
    /// Returns whether anything changed, so a caller can tell a migrated
    /// document from one that needed nothing.
    pub fn assign_word_ids(&mut self) -> bool {
        let mut changed = false;
        for word in self.captions.words_mut() {
            if word.word_id.is_none() {
                word.word_id = Some(format!("w@{}", word.start_ticks));
                changed = true;
            }
        }
        changed
    }

    /// Whether every word carries an identity.
    pub fn words_are_identified(&self) -> bool {
        self.captions.words().all(|word| word.word_id.is_some())
    }

    /// The text one word carries, in either presentation.
    pub fn word_text(&self, word_id: &str) -> Option<&str> {
        self.captions
            .words()
            .find(|word| word.word_id.as_deref() == Some(word_id))
            .map(|word| word.text.as_str())
    }

    /// Give one word new text, wherever it appears. Returns the text it had.
    pub(crate) fn set_word_text(
        &mut self,
        word_id: &str,
        text: &str,
    ) -> Result<String, DocumentError> {
        let mut previous: Option<String> = None;
        for word in self.captions.words_mut() {
            if word.word_id.as_deref() == Some(word_id) {
                let was = std::mem::replace(&mut word.text, text.to_owned());
                previous.get_or_insert(was);
            }
        }
        previous.ok_or_else(|| DocumentError::UnknownWord(word_id.to_owned()))
    }

    /// Derive an unused identifier from an existing one. Deterministic by
    /// construction: replaying a log must mint the same identifiers it minted
    /// live, so splits may never reach for a clock or a random source.
    pub(crate) fn derive_id(existing: &[String], base: &str) -> String {
        let mut suffix = 1_u32;
        loop {
            let candidate = format!("{base}~{suffix}");
            if !existing.iter().any(|value| value == &candidate) {
                return candidate;
            }
            suffix = suffix.saturating_add(1);
        }
    }

    /// Splice program-anchored content: remove `remove` ticks starting at
    /// `at`, then make room for `insert` ticks there.
    ///
    /// Captions and automation live in program time, so any operation that
    /// changes how much program time exists before a point must move them or
    /// the edit silently desynchronises. A word is dropped when the deleted
    /// span touches it at all — the time it occupied is gone, so keeping it
    /// would mean showing a word over speech that no longer plays.
    /// Returns whether anything was destroyed, which decides whether a caller
    /// can invert itself narrowly or must restore the prior arrangement.
    ///
    /// `program_end` is where the program ended before the splice, in the
    /// same coordinates: what the cut leaves after it is the material between
    /// `removed_end` and there, and a cut that reaches the end leaves nothing
    /// to carry a value into.
    pub(crate) fn splice_program_content(
        &mut self,
        at: i64,
        remove: i64,
        insert: i64,
        program_end: i64,
    ) -> bool {
        let removed_end = at.saturating_add(remove.max(0));
        let delta = insert.max(0).saturating_sub(remove.max(0));
        let words_before = self.caption_word_count();
        let cues_before = self.caption_cue_count();
        let gain_before = self.audio.gain_curve.len();
        let gain_was = self.audio.gain_curve.clone();
        let overlays_were = self.overlays.clone();
        if remove > 0 {
            // An overlay the cut fell inside closes up around it; one it
            // reached into from either side keeps what is left; one wholly
            // inside it is gone with the material it was over.
            for overlay in &mut self.overlays {
                if overlay.end_ticks <= at || overlay.start_ticks >= removed_end {
                    continue;
                }
                if at <= overlay.start_ticks {
                    overlay.start_ticks = removed_end.min(overlay.end_ticks);
                } else if removed_end >= overlay.end_ticks {
                    overlay.end_ticks = at;
                } else {
                    overlay.end_ticks = overlay.end_ticks.saturating_add(delta);
                }
            }
            self.overlays
                .retain(|overlay| overlay.end_ticks > overlay.start_ticks);
            // Both presentations: they are two groupings of one word list and
            // a word that no longer plays is gone from each. Splicing only the
            // reading cues left the burned-in ones showing a caption over
            // speech that had been cut — the picture and the sidecar
            // disagreeing about what was said.
            for presentation in [Presentation::Reading, Presentation::BurnIn] {
                let cues = self.captions.list_mut(presentation);
                for cue in cues.iter_mut() {
                    if cue.end_ticks <= at || cue.start_ticks >= removed_end {
                        continue;
                    }
                    for line in &mut cue.lines {
                        line.words
                            .retain(|word| word.end_ticks <= at || word.start_ticks >= removed_end);
                    }
                    cue.lines.retain(|line| !line.words.is_empty());
                    if !cue.lines.is_empty() {
                        // The cut side moves to the cut. The other side keeps
                        // the window the cue had: a cue is held past its words
                        // so it can be read, and a trim that took the words on
                        // one side is no reason to take the reading time on
                        // the other. A cue cut through the head starts with
                        // the picture rather than a beat after it; a cue the
                        // cut fell inside closes up around it.
                        let head_cut = at <= cue.start_ticks;
                        let tail_cut = removed_end >= cue.end_ticks;
                        if head_cut {
                            cue.start_ticks = cue.start_ticks.max(removed_end);
                        } else if tail_cut {
                            cue.end_ticks = cue.end_ticks.min(at);
                        } else {
                            cue.end_ticks = cue.end_ticks.saturating_add(delta);
                            for word in cue.lines.iter_mut().flat_map(|line| &mut line.words) {
                                if word.start_ticks >= removed_end {
                                    word.start_ticks = word.start_ticks.saturating_add(delta);
                                    word.end_ticks = word.end_ticks.saturating_add(delta);
                                }
                            }
                        }
                    }
                }
                cues.retain(|cue| !cue.lines.is_empty());
            }
            self.pin_gain_around(at, removed_end, removed_end < program_end);
            self.audio
                .gain_curve
                .retain(|point| point.t_ticks < at || point.t_ticks >= removed_end);
        }
        let destroyed = self.caption_word_count() != words_before
            || self.caption_cue_count() != cues_before
            || self.audio.gain_curve.len() != gain_before
            || self.audio.gain_curve != gain_was
            || self.overlays != overlays_were;
        if delta == 0 {
            return destroyed;
        }
        let shift_from = if delta < 0 { removed_end } else { at };
        for presentation in [Presentation::Reading, Presentation::BurnIn] {
            for cue in self.captions.list_mut(presentation).iter_mut() {
                if cue.start_ticks < shift_from {
                    continue;
                }
                cue.start_ticks = cue.start_ticks.saturating_add(delta).max(0);
                cue.end_ticks = cue.end_ticks.saturating_add(delta).max(0);
                for line in &mut cue.lines {
                    for word in &mut line.words {
                        word.start_ticks = word.start_ticks.saturating_add(delta).max(0);
                        word.end_ticks = word.end_ticks.saturating_add(delta).max(0);
                    }
                }
            }
        }
        for point in &mut self.audio.gain_curve {
            if point.t_ticks >= shift_from {
                point.t_ticks = point.t_ticks.saturating_add(delta).max(0);
            }
        }
        for overlay in &mut self.overlays {
            if overlay.start_ticks >= shift_from {
                overlay.start_ticks = overlay.start_ticks.saturating_add(delta).max(0);
                overlay.end_ticks = overlay.end_ticks.saturating_add(delta).max(0);
            }
        }
        destroyed
    }

    /// Keep what the gain curve was doing at the edges of a cut.
    ///
    /// The renderer holds the first point backwards and the last forwards
    /// and ramps between neighbours, so removing the points inside a cut is
    /// not the same as removing the cut: a ramp that crossed the boundary now
    /// starts from a different point, and a hold that a removed point defined
    /// is gone with it. Before the points inside are removed, the value the
    /// curve held at each edge is written down as a point of its own — at the
    /// end of the cut when material follows it and anything before it defined
    /// that value, and on the last tick before the cut when anything at or
    /// after it did. The kept material then sounds as it did; only the cut is
    /// gone.
    fn pin_gain_around(&mut self, at: i64, removed_end: i64, material_follows: bool) {
        let curve = &self.audio.gain_curve;
        if curve.is_empty() {
            return;
        }
        let mut pins = Vec::new();
        if material_follows
            && curve.iter().any(|point| point.t_ticks < removed_end)
            && !curve.iter().any(|point| point.t_ticks == removed_end)
            && let Some(gain_db) = gain_at(curve, removed_end)
        {
            pins.push(GainPoint {
                t_ticks: removed_end,
                gain_db,
            });
        }
        let last_kept = at - 1;
        if at > 0
            && curve.iter().any(|point| point.t_ticks >= at)
            && !curve.iter().any(|point| point.t_ticks == last_kept)
            && let Some(gain_db) = gain_at(curve, last_kept)
        {
            pins.push(GainPoint {
                t_ticks: last_kept,
                gain_db,
            });
        }
        if pins.is_empty() {
            return;
        }
        self.audio.gain_curve.extend(pins);
        self.audio.gain_curve.sort_by_key(|point| point.t_ticks);
    }

    /// Where an overlay sits in the stack.
    pub(crate) fn overlay_index(&self, overlay_id: &str) -> Result<usize, DocumentError> {
        self.overlays
            .iter()
            .position(|overlay| overlay.overlay_id == overlay_id)
            .ok_or_else(|| DocumentError::UnknownOverlay(overlay_id.to_owned()))
    }

    fn caption_word_count(&self) -> usize {
        self.captions.words().count()
    }

    fn caption_cue_count(&self) -> usize {
        self.captions.cues.len() + self.captions.burn_in.len()
    }

    /// Re-time a segment's crop path when its source window moves. Keyframes
    /// are stored segment-local, so a path point at local `t` sits at source
    /// time `old_in + t`; points that fall outside the new window are dropped
    /// rather than clamped, because a clamped keyframe is a camera move the
    /// user never asked for.
    ///
    /// What is never dropped is the crop itself. A keyframe before the new
    /// in point decided where the camera stood at that boundary — a static
    /// crop is one keyframe at zero, and advancing the head past it used to
    /// leave a `speaker_fill` segment with no path at all, which the preview
    /// drew as fit and the renderer refused. The crop the path held at each
    /// new boundary is evaluated and kept as a keyframe there, so the picture
    /// on the first and last frame is the picture that was there before.
    pub(crate) fn retime_crop_path(
        path: &[CropKeyframe],
        old_in: i64,
        new_in: i64,
        new_duration: i64,
    ) -> Vec<CropKeyframe> {
        let new_out = new_in.saturating_add(new_duration);
        let in_source: Vec<CropKeyframe> = path
            .iter()
            .map(|keyframe| CropKeyframe {
                t_ticks: old_in.saturating_add(keyframe.t_ticks),
                rect: keyframe.rect,
                easing: keyframe.easing,
            })
            .collect();
        let mut kept: Vec<CropKeyframe> = in_source
            .iter()
            .filter(|key| (new_in..=new_out).contains(&key.t_ticks))
            .map(|key| CropKeyframe {
                t_ticks: key.t_ticks - new_in,
                rect: key.rect,
                easing: key.easing,
            })
            .collect();
        if in_source.iter().any(|key| key.t_ticks < new_in)
            && kept.first().is_none_or(|keyframe| keyframe.t_ticks != 0)
            && let Some(rect) = crop_along_keyframes(&in_source, new_in)
        {
            kept.insert(
                0,
                CropKeyframe {
                    t_ticks: 0,
                    rect,
                    easing: CropEasing::Linear,
                },
            );
        }
        if in_source.iter().any(|key| key.t_ticks > new_out)
            && kept
                .last()
                .is_none_or(|keyframe| keyframe.t_ticks != new_duration)
            && let Some(rect) = crop_along_keyframes(&in_source, new_out)
        {
            kept.push(CropKeyframe {
                t_ticks: new_duration,
                rect,
                easing: CropEasing::Linear,
            });
        }
        kept
    }

    /// Every invariant the command engine promises to preserve. Commands
    /// validate after applying, so a rejected command leaves the caller's
    /// document untouched and the log can never contain a step that produces
    /// an unrenderable state.
    #[allow(clippy::too_many_lines)]
    pub fn validate(&self) -> Result<(), DocumentError> {
        if self.version != IR_VERSION {
            return Err(DocumentError::UnsupportedVersion(self.version.clone()));
        }
        if self.timebase.num != 1 || self.timebase.den != TICKS_PER_SECOND {
            return Err(DocumentError::UnsupportedTimebase);
        }
        if !(0..=22_500).contains(&self.video.transition_ticks) {
            return Err(DocumentError::InvalidTransitionDuration);
        }
        let mut seen_segments = Vec::with_capacity(self.video.segments.len());
        for segment in &self.video.segments {
            if segment.segment_id.is_empty() {
                return Err(DocumentError::EmptyIdentifier);
            }
            if seen_segments.contains(&segment.segment_id.as_str()) {
                return Err(DocumentError::DuplicateSegment(segment.segment_id.clone()));
            }
            seen_segments.push(segment.segment_id.as_str());
            if segment.in_ticks < 0 || segment.out_ticks <= segment.in_ticks {
                return Err(DocumentError::EmptySegment(segment.segment_id.clone()));
            }
            if !segment
                .source_fingerprint
                .strip_prefix("sha256:")
                .is_some_and(|digest| {
                    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
            {
                return Err(DocumentError::InvalidSourceFingerprint(
                    segment.segment_id.clone(),
                ));
            }
            let duration = segment.duration_ticks();
            if segment.layout.state == LayoutState::TwoUp
                && (segment.layout.crop_path.is_empty()
                    || segment.layout.secondary_crop_path.is_empty())
            {
                return Err(DocumentError::TwoUpWithoutCropPaths(
                    segment.segment_id.clone(),
                ));
            }
            if segment.layout.state == LayoutState::PictureInPicture
                && segment.layout.secondary_crop_path.is_empty()
            {
                return Err(DocumentError::InsetWithoutCropPath(
                    segment.segment_id.clone(),
                ));
            }
            if !segment.layout.style_is_valid() {
                return Err(DocumentError::InvalidLayoutStyle(
                    segment.segment_id.clone(),
                ));
            }
            if !segment.layout.punches_are_valid(segment.duration_ticks()) {
                return Err(DocumentError::InvalidPunch(segment.segment_id.clone()));
            }
            for path in [
                &segment.layout.crop_path,
                &segment.layout.secondary_crop_path,
            ] {
                let mut previous: Option<i64> = None;
                for keyframe in path {
                    if keyframe.t_ticks < 0 || keyframe.t_ticks > duration {
                        return Err(DocumentError::CropKeyframeOutOfSegment(
                            segment.segment_id.clone(),
                        ));
                    }
                    if previous.is_some_and(|value| value >= keyframe.t_ticks) {
                        return Err(DocumentError::UnorderedCropPath(segment.segment_id.clone()));
                    }
                    if keyframe.rect.width <= 0 || keyframe.rect.height <= 0 {
                        return Err(DocumentError::EmptyCropRect(segment.segment_id.clone()));
                    }
                    if keyframe.rect.x < 0 || keyframe.rect.y < 0 {
                        return Err(DocumentError::NegativeCropOrigin(
                            segment.segment_id.clone(),
                        ));
                    }
                    previous = Some(keyframe.t_ticks);
                }
            }
        }

        if self
            .title
            .as_ref()
            .is_some_and(|title| title.trim().is_empty() || title.chars().count() > 120)
        {
            return Err(DocumentError::InvalidTitle);
        }
        if self
            .brand
            .as_ref()
            .is_some_and(|brand| brand.is_empty() || !brand.is_valid(&self.assets))
        {
            return Err(DocumentError::InvalidBrand);
        }
        let mut seen_overlays = Vec::with_capacity(self.overlays.len());
        for overlay in &self.overlays {
            if overlay.overlay_id.is_empty() {
                return Err(DocumentError::EmptyIdentifier);
            }
            if seen_overlays.contains(&overlay.overlay_id.as_str()) {
                return Err(DocumentError::DuplicateOverlay(overlay.overlay_id.clone()));
            }
            seen_overlays.push(overlay.overlay_id.as_str());
            if overlay.start_ticks < 0 || overlay.end_ticks <= overlay.start_ticks {
                return Err(DocumentError::EmptyOverlay(overlay.overlay_id.clone()));
            }
            if !overlay.is_valid() {
                return Err(DocumentError::InvalidOverlay(overlay.overlay_id.clone()));
            }
        }
        validate_cues(&self.captions.cues)?;
        validate_cues(&self.captions.burn_in)?;
        validate_shared_words(&self.captions)?;
        if self
            .captions
            .options
            .font_size
            .is_some_and(|size| !(24..=160).contains(&size))
        {
            return Err(DocumentError::InvalidCaptionOptions);
        }
        let options = &self.captions.options;
        if options.outline_width.is_some_and(|width| width > 16)
            || options.shadow_depth.is_some_and(|depth| depth > 12)
            || options.plate_opacity.is_some_and(|opacity| opacity > 100)
            || options
                .words_on_screen
                .is_some_and(|words| !(1..=8).contains(&words))
            || options
                .position
                .is_some_and(|position| !position.is_valid())
            || options
                .font_family
                .as_deref()
                .is_some_and(|family| clipmill_captions::font(family).is_none())
        {
            return Err(DocumentError::InvalidCaptionOptions);
        }
        for colour in [
            &self.captions.options.spoken,
            &self.captions.options.unspoken,
            &self.captions.options.outline,
            &self.captions.options.accent,
        ] {
            if colour.as_ref().is_some_and(|hex| !is_hex_colour(hex)) {
                return Err(DocumentError::InvalidCaptionOptions);
            }
        }

        let mut previous_gain: Option<i64> = None;
        for point in &self.audio.gain_curve {
            if point.t_ticks < 0 {
                return Err(DocumentError::NegativeGainPosition);
            }
            if previous_gain.is_some_and(|value| value >= point.t_ticks) {
                return Err(DocumentError::UnorderedGainCurve);
            }
            if !point.gain_db.is_finite() {
                return Err(DocumentError::NonFiniteGain);
            }
            previous_gain = Some(point.t_ticks);
        }
        if !self.audio.target_lufs.is_finite() || !self.audio.true_peak_dbtp.is_finite() {
            return Err(DocumentError::NonFiniteGain);
        }
        if self
            .audio
            .music
            .as_ref()
            .is_some_and(|music| !music.is_valid(&self.assets))
        {
            return Err(DocumentError::InvalidMusic);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("caption size, colour, font or position is outside the supported range")]
    InvalidCaptionOptions,
    #[error("soft cuts must be between zero and 250 milliseconds")]
    InvalidTransitionDuration,
    #[error("a clip title has between one and 120 characters")]
    InvalidTitle,
    #[error("edit document version {0} is not supported")]
    UnsupportedVersion(String),
    #[error("edit documents must use the 1/90000 edit timebase")]
    UnsupportedTimebase,
    #[error("identifiers cannot be empty")]
    EmptyIdentifier,
    #[error("segment {0} appears more than once")]
    DuplicateSegment(String),
    #[error("segment {0} has an empty or negative source window")]
    EmptySegment(String),
    #[error("segment {0} does not carry a valid source fingerprint")]
    InvalidSourceFingerprint(String),
    #[error("segment {0} has a crop keyframe outside its own duration")]
    CropKeyframeOutOfSegment(String),
    #[error("segment {0} has an unordered or duplicated crop path")]
    UnorderedCropPath(String),
    #[error("segment {0} has an empty crop rectangle")]
    EmptyCropRect(String),
    #[error("segment {0} has a crop rectangle outside the frame origin")]
    NegativeCropOrigin(String),
    #[error("cue {0} appears more than once")]
    DuplicateCue(String),
    #[error("cue {0} has an empty or negative time span")]
    EmptyCue(String),
    #[error("cue {0} overlaps the preceding cue")]
    OverlappingCues(String),
    #[error("cue {0} carries an empty or untimed word")]
    EmptyCaptionWord(String),
    #[error("cue {0} carries a word outside its own span")]
    WordOutsideCue(String),
    #[error("cue {0} has unordered words")]
    UnorderedCaptionWords(String),
    #[error("word {0} appears more than once in one presentation")]
    DuplicateWord(String),
    #[error("word {0} reads differently in the two presentations")]
    DivergentWord(String),
    #[error("no word named {0}")]
    UnknownWord(String),
    #[error("caption lines must each carry at least one word")]
    EmptyCaptionLine,
    #[error("line breaks must cover exactly the cue's words")]
    LineBreaksDoNotCoverWords,
    #[error("gain automation cannot be positioned before zero")]
    NegativeGainPosition,
    #[error("gain automation must be ordered and unique in time")]
    UnorderedGainCurve,
    #[error("loudness and gain values must be finite")]
    NonFiniteGain,
    #[error(
        "the music's level, drop or start is outside what can be played, or it is not an asset of the clip"
    )]
    InvalidMusic,
    #[error("two-person layout on segment {0} requires both crop paths")]
    TwoUpWithoutCropPaths(String),
    #[error("picture-in-picture on segment {0} requires an inset crop path")]
    InsetWithoutCropPath(String),
    #[error("segment {0} asks for a split, zoom, inset or background it cannot draw")]
    InvalidLayoutStyle(String),
    #[error("segment {0} has punches out of order, overlapping, outside it or too close")]
    InvalidPunch(String),
    #[error(
        "the progress bar or logo is outside what can be drawn, or the logo is not an asset of the clip"
    )]
    InvalidBrand,
    #[error("overlay {0} appears more than once")]
    DuplicateOverlay(String),
    #[error("overlay {0} has an empty or negative time span")]
    EmptyOverlay(String),
    #[error("overlay {0} has text, a size, a colour or a place it cannot be drawn with")]
    InvalidOverlay(String),
    #[error("no overlay named {0}")]
    UnknownOverlay(String),
    #[error("no segment named {0}")]
    UnknownSegment(String),
    #[error("no cue named {0}")]
    UnknownCue(String),
    #[error("edit document is not valid JSON: {0}")]
    Json(String),
}
