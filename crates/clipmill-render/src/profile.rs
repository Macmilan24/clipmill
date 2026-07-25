//! The render profile: every knob that decides what the encoder produces.
//!
//! The profile is part of the render recipe, so changing any of it produces a
//! different artifact rather than quietly different pixels under the same
//! content address. Phase 1 ships exactly one profile; it is a value rather
//! than a constant so the recipe can carry it and the manifest can state it.

use serde::{Deserialize, Serialize};

use crate::timing::FrameRate;

/// The identifier the daemon records in the render recipe.
pub const PROFILE_ID: &str = "clipmill.render.vertical_1080x1920.v1";
/// The caption style Phase 1 renders with. W21 adds the preset family; the
/// document names a style and an unknown name is refused, never defaulted.
pub const DEFAULT_STYLE_REF: &str = "clipmill.captions.karaoke.v1";
/// Font family name that must match the pinned font file's internal name.
pub const FONT_FAMILY: &str = "Inter";
/// Where the executor stages the pinned font, relative to the working
/// directory FFmpeg runs in. Nothing else may be visible to libass.
pub const FONTS_DIR: &str = "fonts";

/// An ASS colour, written `&HAABBGGRR`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    /// ASS alpha is inverted: 0 is opaque.
    pub transparency: u8,
}

impl Colour {
    pub const fn opaque(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            transparency: 0,
        }
    }

    pub fn to_ass(self) -> String {
        format!(
            "&H{:02X}{:02X}{:02X}{:02X}",
            self.transparency, self.blue, self.green, self.red
        )
    }
}

/// The subset of ASS styling Phase 1 exposes. Line *breaking* is deliberately
/// absent: breaks are decided once and stored in the Edit IR, and the renderer
/// is configured never to re-wrap (book ch. 17, ch. 19).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CaptionStyle {
    pub style_ref: String,
    pub font_family: String,
    pub font_size: u32,
    /// Colour a word takes once it has been spoken.
    pub spoken: Colour,
    /// Colour a word carries before it is spoken.
    pub unspoken: Colour,
    pub outline: Colour,
    pub shadow: Colour,
    pub outline_width: u32,
    pub shadow_depth: u32,
    pub bold: bool,
    pub margin_horizontal: u32,
    /// Distance from the frame edge the anchored region keeps clear.
    pub margin_vertical: u32,
}

impl CaptionStyle {
    /// The Phase 1 style: heavy outline, no box, high contrast, and a spoken
    /// colour distinct enough to read as motion on a phone screen.
    pub fn karaoke_v1() -> Self {
        Self {
            style_ref: DEFAULT_STYLE_REF.to_owned(),
            font_family: FONT_FAMILY.to_owned(),
            font_size: 84,
            spoken: Colour::opaque(0xFF, 0xD8, 0x2E),
            unspoken: Colour::opaque(0xFF, 0xFF, 0xFF),
            outline: Colour::opaque(0x00, 0x00, 0x00),
            shadow: Colour::opaque(0x00, 0x00, 0x00),
            outline_width: 5,
            shadow_depth: 2,
            bold: true,
            margin_horizontal: 90,
            margin_vertical: 260,
        }
    }
}

/// Loudness targets. Both are measured quantities in decibels and are
/// legitimately real-valued; nothing here is a time.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct LoudnessTarget {
    pub integrated_lufs: f64,
    pub true_peak_dbtp: f64,
    pub range_lu: f64,
}

impl Default for LoudnessTarget {
    fn default() -> Self {
        Self {
            integrated_lufs: -14.0,
            true_peak_dbtp: -1.0,
            range_lu: 11.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RenderProfile {
    pub profile_id: String,
    pub width: i64,
    pub height: i64,
    pub frame_rate: FrameRateSpec,
    pub video_codec: String,
    pub crf: u32,
    pub preset: String,
    pub pixel_format: String,
    pub audio_codec: String,
    pub audio_bitrate: u32,
    pub audio_sample_rate: u32,
    pub audio_channels: u32,
    pub loudness: LoudnessTarget,
    pub caption_style: CaptionStyle,
    /// Blur applied to the filled background behind a letterboxed frame.
    pub fit_background_sigma: u32,
}

/// Serializable twin of [`FrameRate`]; the compiler works in the exact
/// rational, the manifest states it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FrameRateSpec {
    pub num: i64,
    pub den: i64,
}

impl From<FrameRateSpec> for FrameRate {
    fn from(value: FrameRateSpec) -> Self {
        Self {
            num: value.num,
            den: value.den,
        }
    }
}

impl Default for RenderProfile {
    fn default() -> Self {
        Self {
            profile_id: PROFILE_ID.to_owned(),
            width: 1_080,
            height: 1_920,
            frame_rate: FrameRateSpec {
                num: 30_000,
                den: 1_001,
            },
            video_codec: "libx264".to_owned(),
            crf: 18,
            preset: "medium".to_owned(),
            pixel_format: "yuv420p".to_owned(),
            audio_codec: "aac".to_owned(),
            audio_bitrate: 192_000,
            audio_sample_rate: 48_000,
            audio_channels: 2,
            loudness: LoudnessTarget::default(),
            caption_style: CaptionStyle::karaoke_v1(),
            fit_background_sigma: 40,
        }
    }
}

impl RenderProfile {
    pub fn rate(&self) -> FrameRate {
        self.frame_rate.into()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{Colour, RenderProfile};

    #[test]
    fn ass_colours_are_written_bgr_with_inverted_alpha() {
        assert_eq!(Colour::opaque(0xFF, 0xFF, 0xFF).to_ass(), "&H00FFFFFF");
        assert_eq!(Colour::opaque(0xFF, 0x00, 0x00).to_ass(), "&H000000FF");
        assert_eq!(
            Colour {
                red: 0x11,
                green: 0x22,
                blue: 0x33,
                transparency: 0x80,
            }
            .to_ass(),
            "&H80332211"
        );
    }

    #[test]
    fn the_phase_one_profile_is_vertical_and_frame_exact() {
        let profile = RenderProfile::default();
        assert_eq!((profile.width, profile.height), (1_080, 1_920));
        let rate = profile.rate();
        assert_eq!((rate.num, rate.den), (30_000, 1_001));
    }
}
