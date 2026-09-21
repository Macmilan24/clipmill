//! Language-specific caption limits.
//!
//! The English accessibility profile uses Netflix timed-text guidance: 42
//! characters per line, two lines, 20 characters per second, durations from five
//! sixths of a second to seven seconds, and two blank frames between cues.
//! Profiles remain data because character counts and reading rates vary by script;
//! unsupported languages are reported explicitly.

/// Ticks per second. The daemon's timebase throughout.
pub const TICKS_PER_SECOND: i64 = 90_000;

/// The numbers one grouping of tokens is held to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    /// Characters on one line, counted in Unicode scalar values of the text a
    /// viewer sees. This is also how the safe area reaches the segmenter: the
    /// ceiling is the width that fits inside it at the preset's type size, so
    /// a cue that respects the ceiling respects the safe area by construction.
    pub max_line_characters: usize,
    pub max_lines: usize,
    /// The ceiling a cue's characters-per-second may not exceed.
    pub reading_rate_cps: f64,
    pub min_duration_ticks: i64,
    pub max_duration_ticks: i64,
    /// Blank between consecutive cues, so two cues do not read as one that
    /// changed under the reader.
    pub min_gap_ticks: i64,
}

impl Profile {
    /// Netflix's English adult-programming numbers, and the profile every
    /// sidecar is held to.
    pub const ACCESSIBILITY_EN: Self = Self {
        max_line_characters: 42,
        max_lines: 2,
        reading_rate_cps: 20.0,
        // Five sixths of a second.
        min_duration_ticks: TICKS_PER_SECOND * 5 / 6,
        max_duration_ticks: TICKS_PER_SECOND * 7,
        // Two frames at 24, which is the standard's own unit for this one.
        min_gap_ticks: TICKS_PER_SECOND * 2 / 24,
    };

    /// The kinetic burn-in: a few words at a time, held briefly, deliberately
    /// faster than anything a sidecar may be.
    ///
    /// This is not a relaxed accessibility profile — it is a different job. A
    /// viewer scrolling with the sound off is reading a rhythm, and the words
    /// are the same words. What makes running hot acceptable is precisely that
    /// the conservative grouping exists beside it in the same document.
    pub const BURN_IN_EN: Self = Self {
        // One short line. The ceiling is in characters rather than words
        // because a line is as wide as its letters, and eighteen is where
        // ordinary English lands at one to three words. A rule that counted
        // words instead would set the same type at wildly different widths.
        max_line_characters: 18,
        max_lines: 1,
        reading_rate_cps: 40.0,
        min_duration_ticks: TICKS_PER_SECOND / 3,
        max_duration_ticks: TICKS_PER_SECOND * 3,
        min_gap_ticks: 0,
    };

    /// Whether these numbers can be met at all, checked once rather than
    /// rediscovered inside the segmenter's inner loop.
    pub(crate) fn is_usable(&self) -> bool {
        self.max_line_characters > 0
            && self.max_lines > 0
            && self.reading_rate_cps > 0.0
            && self.min_duration_ticks > 0
            && self.max_duration_ticks >= self.min_duration_ticks
            && self.min_gap_ticks >= 0
    }

    /// The most characters a single cue can hold.
    pub(crate) fn capacity(&self) -> usize {
        self.max_line_characters.saturating_mul(self.max_lines)
    }
}

/// Which language's numbers a document was segmented with.
///
/// Returning the pair rather than a single profile keeps the two intents
/// together: they are chosen once, from one language, and a build that let them
/// come from different places is a build where the burn-in and the sidecar can
/// describe different recordings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profiles {
    pub accessibility: Profile,
    pub burn_in: Profile,
}

/// The direction a script is read in, carried per document so a translated
/// track is a sibling rather than a schema change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Ltr,
    Rtl,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}

/// Scripts written right to left. Listed by the language subtag's script, so a
/// regional variant resolves the same way as its parent.
const RIGHT_TO_LEFT: &[&str] = &[
    "ar", "arc", "az-arab", "dv", "fa", "he", "ku-arab", "ps", "ur",
];

/// The profiles and direction for a language tag.
///
/// A language with no entry gets the English numbers and is **reported as
/// such** by the caller, because the alternative — quietly applying Latin
/// character counts to a script they do not describe — is the mistake the book
/// names by name. The numbers are a stated default, not a claim about the
/// language.
pub fn for_language(language: &str) -> (Profiles, Direction, bool) {
    let tag = language.to_ascii_lowercase();
    let primary = tag.split(['-', '_']).next().unwrap_or("").to_owned();
    let direction = if RIGHT_TO_LEFT
        .iter()
        .any(|entry| entry == &primary || tag.starts_with(entry))
    {
        Direction::Rtl
    } else {
        Direction::Ltr
    };
    let known = matches!(primary.as_str(), "en");
    (
        Profiles {
            accessibility: Profile::ACCESSIBILITY_EN,
            burn_in: Profile::BURN_IN_EN,
        },
        direction,
        known,
    )
}

#[cfg(test)]
mod tests {
    use super::{Direction, Profile, for_language};

    #[test]
    fn the_published_numbers_are_the_ones_recorded() {
        let profile = Profile::ACCESSIBILITY_EN;
        assert_eq!(profile.max_line_characters, 42);
        assert_eq!(profile.max_lines, 2);
        assert!((profile.reading_rate_cps - 20.0).abs() < f64::EPSILON);
        assert_eq!(profile.min_duration_ticks, 75_000);
        assert_eq!(profile.max_duration_ticks, 630_000);
        assert_eq!(profile.min_gap_ticks, 7_500);
    }

    #[test]
    fn the_burn_in_may_run_hotter_than_the_sidecar_but_not_wider() {
        let hot = Profile::BURN_IN_EN;
        let calm = Profile::ACCESSIBILITY_EN;
        assert!(hot.reading_rate_cps > calm.reading_rate_cps);
        assert!(hot.capacity() < calm.capacity());
        assert!(hot.is_usable() && calm.is_usable());
    }

    #[test]
    fn an_unknown_language_is_reported_rather_than_assumed() {
        let (_, direction, known) = for_language("ja");
        assert!(!known, "Japanese must not silently take English ceilings");
        assert_eq!(direction, Direction::Ltr);
    }

    #[test]
    fn right_to_left_scripts_are_recognized_through_their_region() {
        assert_eq!(for_language("ar").1, Direction::Rtl);
        assert_eq!(for_language("he-IL").1, Direction::Rtl);
        assert_eq!(for_language("en-GB").1, Direction::Ltr);
        assert!(for_language("en-GB").2);
    }

    #[test]
    fn numbers_that_cannot_be_met_are_refused_before_the_loop() {
        let mut broken = Profile::ACCESSIBILITY_EN;
        broken.max_lines = 0;
        assert!(!broken.is_usable());
        broken = Profile::ACCESSIBILITY_EN;
        broken.max_duration_ticks = broken.min_duration_ticks - 1;
        assert!(!broken.is_usable());
    }
}
