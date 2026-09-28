//! The render profile: every knob that decides what the encoder produces.
//!
//! The profile is part of the render recipe, so changing any of it produces a
//! different artifact rather than quietly different pixels under the same
//! content address. The default profile is a value rather
//! than a constant so the recipe can carry it and the manifest can state it.

use clipmill_edit_ir::{CaptionTrack, FrameShape};
use serde::{Deserialize, Serialize};

use crate::timing::FrameRate;

/// The identifier the daemon records in the render recipe.
pub const PROFILE_ID: &str = "clipmill.render.vertical_1080x1920.v1";
/// The caption style a project gets without choosing. It is the caption
/// engine's own default rather than a second opinion held here: a document
/// names a style, an unknown name is refused rather than defaulted, and a
/// default that were not itself a preset would be a name nothing could resolve.
pub const DEFAULT_STYLE_REF: &str = clipmill_captions::DEFAULT_STYLE_REF;
/// Font family name that must match the pinned font file's internal name.
pub const FONT_FAMILY: &str = "Inter";
/// Where the executor stages the pinned font, relative to the working
/// directory FFmpeg runs in. Nothing else may be visible to libass.
pub const FONTS_DIR: &str = "fonts";
/// The frame height caption styles are designed at. Sizes, margins, outline
/// and shadow are all stated at this height and libass scales them to the
/// output, so a larger or smaller render keeps the same proportions.
pub const DESIGN_HEIGHT: i64 = 1_920;

/// The ASS script resolution for an output frame: the design height, and the
/// width that keeps the output's shape.
pub fn design_resolution(width: i64, height: i64) -> (i64, i64) {
    if height <= 0 {
        return (width, height);
    }
    let scaled = (width * DESIGN_HEIGHT + height / 2) / height;
    (scaled, DESIGN_HEIGHT)
}

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

    /// The colour as an inline override tag takes it: `&HBBGGRR&`, no alpha.
    pub fn to_ass_override(self) -> String {
        format!("&H{:02X}{:02X}{:02X}&", self.blue, self.green, self.red)
    }

    pub(crate) fn from_hex(value: &str) -> Option<Self> {
        let digits = value.strip_prefix('#')?;
        if digits.len() != 6 {
            return None;
        }
        Some(Self::opaque(
            u8::from_str_radix(&digits[0..2], 16).ok()?,
            u8::from_str_radix(&digits[2..4], 16).ok()?,
            u8::from_str_radix(&digits[4..6], 16).ok()?,
        ))
    }
}

/// The supported subset of ASS styling. Line *breaking* is deliberately
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
    /// Whether the words sit on an opaque plate rather than under an outline.
    /// The only one of the three presets that stays legible over a bright,
    /// busy, moving background, and the reason libass is told a border style
    /// rather than always the same one.
    pub boxed: bool,
    pub margin_horizontal: u32,
    /// Distance from the frame edge the anchored region keeps clear.
    pub margin_vertical: u32,
    /// How the spoken word is marked, when the cue marks it.
    pub highlight: clipmill_edit_ir::HighlightStyle,
    /// The colour key words are set in.
    pub accent: Colour,
}

impl CaptionStyle {
    /// Resolve the document's preset and its saved clip-wide adjustments.
    pub fn for_track(track: &CaptionTrack) -> Option<Self> {
        let mut style = Self::from_preset(clipmill_captions::preset(&track.style_ref)?);
        let options = &track.options;
        if let Some(size) = options.font_size {
            style.font_size = size;
        }
        if let Some(value) = &options.spoken {
            style.spoken = Colour::from_hex(value)?;
        }
        if let Some(value) = &options.unspoken {
            style.unspoken = Colour::from_hex(value)?;
        }
        if let Some(value) = &options.outline {
            style.outline = Colour::from_hex(value)?;
        }
        if let Some(value) = &options.accent {
            style.accent = Colour::from_hex(value)?;
        }
        if options.highlight_spoken_word == Some(true) && style.spoken == style.unspoken {
            style.spoken = Colour::opaque(0xFF, 0xD6, 0x5C);
        }
        if options.highlight_spoken_word == Some(false) {
            style.spoken = style.unspoken;
        }
        if let Some(family) = &options.font_family {
            let face = clipmill_captions::font(family)?;
            face.family.clone_into(&mut style.font_family);
            style.bold = face.bold;
        }
        if let Some(width) = options.outline_width {
            style.outline_width = width;
        }
        if let Some(depth) = options.shadow_depth {
            style.shadow_depth = depth;
        }
        if let Some(opacity) = options.plate_opacity
            && style.boxed
        {
            let opacity = u8::try_from(opacity.min(100) * 255 / 100).unwrap_or(u8::MAX);
            style.outline.transparency = u8::MAX - opacity;
            style.shadow.transparency = u8::MAX - opacity;
        }
        style.highlight = options.highlight_style.unwrap_or_default();
        Some(style)
    }
    /// The default look: heavy outline, no plate, high contrast, and a spoken
    /// colour distinct enough to read as motion on a phone screen.
    ///
    /// Built from the caption engine's preset rather than restated here, so
    /// there is exactly one place the numbers live.
    pub fn default_preset() -> Self {
        clipmill_captions::preset(DEFAULT_STYLE_REF).map_or_else(
            // Unreachable while the default is a member of its own family, and
            // a test holds that. A style is still better than no captions.
            || Self {
                style_ref: DEFAULT_STYLE_REF.to_owned(),
                font_family: FONT_FAMILY.to_owned(),
                font_size: 84,
                spoken: Colour::opaque(0xFF, 0xD6, 0x5C),
                unspoken: Colour::opaque(0xFF, 0xFF, 0xFF),
                outline: Colour::opaque(0x00, 0x00, 0x00),
                shadow: Colour::opaque(0x00, 0x00, 0x00),
                outline_width: 5,
                shadow_depth: 2,
                bold: true,
                boxed: false,
                margin_horizontal: 90,
                margin_vertical: 260,
                highlight: clipmill_edit_ir::HighlightStyle::Fill,
                accent: Colour::opaque(0x4A, 0xDE, 0x80),
            },
            Self::from_preset,
        )
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
            caption_style: CaptionStyle::default_preset(),
            fit_background_sigma: 40,
        }
    }
}

/// The output sizes a creator may choose, named by the height of the 9:16
/// frame at that size: 1080p, 1440p and 4K. Another shape at the same size
/// keeps the short side — 1080 × 1080 square, 1920 × 1080 landscape.
pub const OUTPUT_HEIGHTS: [i64; 3] = [1_920, 2_560, 3_840];

impl RenderProfile {
    pub fn rate(&self) -> FrameRate {
        self.frame_rate.into()
    }

    /// The default profile in another shape, at another frame rate and size.
    ///
    /// `height` names the size as the 9:16 frame's height at it (see
    /// [`OUTPUT_HEIGHTS`]). Only the picture's shape, size and clock change.
    /// Caption styles are stated at the design height and libass scales them,
    /// and the letterbox blur is scaled here so a larger frame is the same
    /// picture, sharper.
    pub fn for_output(shape: FrameShape, height: i64, frame_rate: FrameRateSpec) -> Option<Self> {
        if !OUTPUT_HEIGHTS.contains(&height) || frame_rate.num <= 0 || frame_rate.den <= 0 {
            return None;
        }
        let base = Self::default();
        let short = height * base.width / base.height;
        let (width, height) = shape.frame(short);
        let sigma = u32::try_from(i64::from(base.fit_background_sigma) * short / base.width)
            .unwrap_or(base.fit_background_sigma);
        let profile_id = if (width, height) == (base.width, base.height) {
            base.profile_id.clone()
        } else {
            let name = match shape {
                FrameShape::Vertical => "vertical",
                FrameShape::Portrait => "portrait",
                FrameShape::Square => "square",
                FrameShape::Landscape => "landscape",
            };
            format!("clipmill.render.{name}_{width}x{height}.v1")
        };
        Some(Self {
            profile_id,
            width,
            height,
            frame_rate,
            fit_background_sigma: sigma,
            ..base
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{Colour, FrameShape, RenderProfile};

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
    fn larger_outputs_keep_the_shape_and_scale_the_fill() {
        use super::FrameRateSpec;
        let rate = FrameRateSpec {
            num: 24_000,
            den: 1_001,
        };
        let four_k = RenderProfile::for_output(FrameShape::Vertical, 3_840, rate).expect("4K");
        assert_eq!((four_k.width, four_k.height), (2_160, 3_840));
        assert_eq!(four_k.fit_background_sigma, 80);
        assert_eq!(four_k.frame_rate, rate);
        assert_ne!(four_k.profile_id, RenderProfile::default().profile_id);
        let same = RenderProfile::for_output(
            FrameShape::Vertical,
            1_920,
            RenderProfile::default().frame_rate,
        )
        .expect("default size");
        assert_eq!(same, RenderProfile::default());
        assert!(RenderProfile::for_output(FrameShape::Vertical, 1_000, rate).is_none());
    }

    #[test]
    fn every_shape_keeps_the_short_side_of_its_size() {
        use super::FrameShape;
        let rate = RenderProfile::default().frame_rate;
        let sized = |shape, height| {
            let profile = RenderProfile::for_output(shape, height, rate).expect("offered");
            (profile.width, profile.height, profile.profile_id)
        };
        assert_eq!(
            sized(FrameShape::Portrait, 1_920),
            (
                1_080,
                1_350,
                "clipmill.render.portrait_1080x1350.v1".to_owned()
            )
        );
        assert_eq!(
            sized(FrameShape::Square, 2_560),
            (
                1_440,
                1_440,
                "clipmill.render.square_1440x1440.v1".to_owned()
            )
        );
        assert_eq!(
            sized(FrameShape::Landscape, 1_920),
            (
                1_920,
                1_080,
                "clipmill.render.landscape_1920x1080.v1".to_owned()
            )
        );
        assert_eq!(
            sized(FrameShape::Landscape, 3_840),
            (
                3_840,
                2_160,
                "clipmill.render.landscape_3840x2160.v1".to_owned()
            )
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
