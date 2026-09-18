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
        if remove > 0 {
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
            || self.audio.gain_curve != gain_was;
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
        let in_source: Vec<(i64, CropRect)> = path
            .iter()
            .map(|keyframe| (old_in.saturating_add(keyframe.t_ticks), keyframe.rect))
            .collect();
        let mut kept: Vec<CropKeyframe> = in_source
            .iter()
            .filter(|(source, _)| (new_in..=new_out).contains(source))
            .map(|(source, rect)| CropKeyframe {
                t_ticks: source - new_in,
                rect: *rect,
            })
            .collect();
        if in_source.iter().any(|(source, _)| *source < new_in)
            && kept.first().is_none_or(|keyframe| keyframe.t_ticks != 0)
            && let Some(rect) = crop_along(&in_source, new_in)
        {
            kept.insert(0, CropKeyframe { t_ticks: 0, rect });
        }
        if in_source.iter().any(|(source, _)| *source > new_out)
            && kept
                .last()
                .is_none_or(|keyframe| keyframe.t_ticks != new_duration)
            && let Some(rect) = crop_along(&in_source, new_out)
        {
            kept.push(CropKeyframe {
                t_ticks: new_duration,
                rect,
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
        validate_shared_words(&self.captions)?;

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
    #[error("no segment named {0}")]
    UnknownSegment(String),
    #[error("no cue named {0}")]
    UnknownCue(String),
    #[error("edit document is not valid JSON: {0}")]
    Json(String),
}
