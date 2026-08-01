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
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutState {
    /// Crop to a single speaker and follow them along the crop path.
    SpeakerFill,
    /// Letterbox the whole frame; the crop path is inert but preserved.
    #[default]
    Fit,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub state: LayoutState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub crop_path: Vec<CropKeyframe>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub segments: Vec<VideoSegment>,
}

/// One word with its own timing, so text selection is time selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionWord {
    pub text: String,
    pub start_ticks: i64,
    pub end_ticks: i64,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AudioTrack {
    pub target_lufs: f64,
    pub true_peak_dbtp: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gain_curve: Vec<GainPoint>,
}

impl Default for AudioTrack {
    fn default() -> Self {
        Self {
            target_lufs: -14.0,
            true_peak_dbtp: -1.0,
            gain_curve: Vec::new(),
        }
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<Asset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<Rationale>,
}

impl Default for EditDocument {
    fn default() -> Self {
        Self {
            version: IR_VERSION.to_owned(),
            timebase: Timebase::default(),
            video: VideoTrack::default(),
            captions: CaptionTrack::default(),
            audio: AudioTrack::default(),
            assets: Vec::new(),
            rationale: None,
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

    /// Position of a caption cue by identifier.
    pub fn cue_index(&self, cue_id: &str) -> Result<usize, DocumentError> {
        self.captions
            .cues
            .iter()
            .position(|cue| cue.cue_id == cue_id)
            .ok_or_else(|| DocumentError::UnknownCue(cue_id.to_owned()))
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
    pub(crate) fn splice_program_content(&mut self, at: i64, remove: i64, insert: i64) -> bool {
        let removed_end = at.saturating_add(remove.max(0));
        let delta = insert.max(0).saturating_sub(remove.max(0));
        let words_before = self.caption_word_count();
        let cues_before = self.captions.cues.len();
        let gain_before = self.audio.gain_curve.len();
        if remove > 0 {
            for cue in &mut self.captions.cues {
                if cue.end_ticks <= at || cue.start_ticks >= removed_end {
                    continue;
                }
                for line in &mut cue.lines {
                    line.words
                        .retain(|word| word.end_ticks <= at || word.start_ticks >= removed_end);
                }
                cue.lines.retain(|line| !line.words.is_empty());
                let bounds = cue
                    .words()
                    .fold(None::<(i64, i64)>, |bounds, word| match bounds {
                        None => Some((word.start_ticks, word.end_ticks)),
                        Some((start, end)) => {
                            Some((start.min(word.start_ticks), end.max(word.end_ticks)))
                        }
                    });
                if let Some((start, end)) = bounds {
                    cue.start_ticks = start;
                    cue.end_ticks = end;
                }
            }
            self.captions.cues.retain(|cue| !cue.lines.is_empty());
            self.audio
                .gain_curve
                .retain(|point| point.t_ticks < at || point.t_ticks >= removed_end);
        }
        let destroyed = self.caption_word_count() != words_before
            || self.captions.cues.len() != cues_before
            || self.audio.gain_curve.len() != gain_before;
        if delta == 0 {
            return destroyed;
        }
        let shift_from = if delta < 0 { removed_end } else { at };
        for cue in &mut self.captions.cues {
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
        for point in &mut self.audio.gain_curve {
            if point.t_ticks >= shift_from {
                point.t_ticks = point.t_ticks.saturating_add(delta).max(0);
            }
        }
        destroyed
    }

    fn caption_word_count(&self) -> usize {
        self.captions.cues.iter().map(CaptionCue::word_count).sum()
    }

    /// Re-time a segment's crop path when its source window moves. Keyframes
    /// are stored segment-local, so a path point at local `t` sits at source
    /// time `old_in + t`; points that fall outside the new window are dropped
    /// rather than clamped, because a clamped keyframe is a camera move the
    /// user never asked for.
    pub(crate) fn retime_crop_path(
        path: &[CropKeyframe],
        old_in: i64,
        new_in: i64,
        new_duration: i64,
    ) -> Vec<CropKeyframe> {
        path.iter()
            .filter_map(|keyframe| {
                let source = old_in.saturating_add(keyframe.t_ticks);
                let local = source.saturating_sub(new_in);
                (0..=new_duration).contains(&local).then_some(CropKeyframe {
                    t_ticks: local,
                    rect: keyframe.rect,
                })
            })
            .collect()
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
            let mut previous: Option<i64> = None;
            for keyframe in &segment.layout.crop_path {
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

        validate_cues(&self.captions.cues)?;
        validate_cues(&self.captions.burn_in)?;

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
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum DocumentError {
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
    #[error("no segment named {0}")]
    UnknownSegment(String),
    #[error("no cue named {0}")]
    UnknownCue(String),
    #[error("edit document is not valid JSON: {0}")]
    Json(String),
}
