//! The caption looks a project can choose, and the reduced-motion twin every
//! one of them has.
//!
//! Three presets, because three is what a creator will actually look at.
//! **Clean** is the default: large, bold, outlined, with the karaoke sweep the
//! platform native styles have taught viewers to read. **Minimal** is the same
//! type without the sweep and without the weight, for footage that should carry
//! itself. **Boxed** puts an opaque plate behind the words, which is the only
//! one of the three that stays legible over a bright, busy, moving background.
//!
//! Every preset has a reduced-motion variant, and it is not an afterthought
//! toggle: vestibular disorders and motion sensitivity are real, `prefers-
//! reduced-motion` is a real signal, and a caption style with no still version
//! is a caption style some viewers cannot use. The variant is the same
//! typography with the animation removed, so choosing it changes how the words
//! arrive and never which words they are.
//!
//! Styling lives here rather than in the render because the presets are part of
//! the caption engine's contract: a document names a `style_ref`, and something
//! has to be able to say whether that name exists.

/// Straight RGBA, converted by whatever draws it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Colour {
    pub const fn opaque(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

/// How the words arrive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Animation {
    /// The words are simply there for the cue's duration.
    None,
    /// The highlight advances word by word, exactly when speech does.
    Karaoke,
}

/// How the text is separated from the picture behind it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Border {
    /// An outline and a drop shadow.
    Outline,
    /// An opaque plate behind the line.
    Box,
}

/// One named caption look.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preset {
    /// The name a document stores and a renderer resolves.
    pub style_ref: &'static str,
    /// What a person picking one sees.
    pub label: &'static str,
    pub font_family: &'static str,
    pub font_size: u32,
    pub bold: bool,
    /// The colour of a word once it has been spoken — and the only colour, in
    /// a preset with no sweep.
    pub spoken: Colour,
    /// The colour of a word not yet reached by the sweep.
    pub unspoken: Colour,
    pub outline: Colour,
    pub shadow: Colour,
    pub outline_width: u32,
    pub shadow_depth: u32,
    pub border: Border,
    pub margin_horizontal: u32,
    pub margin_vertical: u32,
    pub animation: Animation,
    /// The still twin of this preset, or `None` when this is the still twin.
    pub reduced_motion_of: Option<&'static str>,
}

impl Preset {
    /// Whether this preset is a reduced-motion variant.
    pub fn is_reduced_motion(&self) -> bool {
        self.reduced_motion_of.is_some()
    }
}

const WHITE: Colour = Colour::opaque(255, 255, 255);
const BLACK: Colour = Colour::opaque(0, 0, 0);
/// The sweep's leading colour. Warm, high-contrast against both white and the
/// black outline every preset carries.
const HIGHLIGHT: Colour = Colour::opaque(255, 214, 92);
/// The plate behind Boxed. Not pure black: a hard black rectangle over video
/// reads as a broken player, and eighty-five percent is where it stops looking
/// like a hole and starts looking like a caption.
const PLATE: Colour = Colour::new(12, 12, 14, 217);

pub const CLEAN: &str = "clipmill.captions.clean.v1";
pub const CLEAN_STILL: &str = "clipmill.captions.clean.reduced-motion.v1";
pub const MINIMAL: &str = "clipmill.captions.minimal.v1";
pub const MINIMAL_STILL: &str = "clipmill.captions.minimal.reduced-motion.v1";
pub const BOXED: &str = "clipmill.captions.boxed.v1";
pub const BOXED_STILL: &str = "clipmill.captions.boxed.reduced-motion.v1";

/// The default a project gets without choosing.
pub const DEFAULT_STYLE_REF: &str = CLEAN;

const CLEAN_PRESET: Preset = Preset {
    style_ref: CLEAN,
    label: "Clean",
    font_family: "Inter",
    font_size: 84,
    bold: true,
    spoken: WHITE,
    unspoken: HIGHLIGHT,
    outline: BLACK,
    shadow: BLACK,
    outline_width: 5,
    shadow_depth: 2,
    border: Border::Outline,
    margin_horizontal: 90,
    margin_vertical: 260,
    animation: Animation::Karaoke,
    reduced_motion_of: None,
};

const MINIMAL_PRESET: Preset = Preset {
    style_ref: MINIMAL,
    label: "Minimal",
    font_family: "Inter",
    font_size: 72,
    bold: false,
    spoken: WHITE,
    unspoken: WHITE,
    outline: BLACK,
    shadow: BLACK,
    outline_width: 3,
    shadow_depth: 1,
    border: Border::Outline,
    margin_horizontal: 96,
    margin_vertical: 240,
    animation: Animation::None,
    reduced_motion_of: None,
};

const BOXED_PRESET: Preset = Preset {
    style_ref: BOXED,
    label: "Boxed",
    font_family: "Inter",
    font_size: 76,
    bold: true,
    spoken: WHITE,
    unspoken: HIGHLIGHT,
    outline: PLATE,
    shadow: PLATE,
    outline_width: 4,
    shadow_depth: 0,
    border: Border::Box,
    margin_horizontal: 96,
    margin_vertical: 250,
    animation: Animation::Karaoke,
    reduced_motion_of: None,
};

/// Every preset, animated and still, in a stable order.
pub const PRESETS: &[Preset] = &[
    CLEAN_PRESET,
    Preset {
        style_ref: CLEAN_STILL,
        label: "Clean (reduced motion)",
        animation: Animation::None,
        // With no sweep there is no second colour to sweep to, and leaving the
        // highlight in place would mean the still variant rendered words in a
        // colour the animated one only ever showed in passing.
        unspoken: WHITE,
        reduced_motion_of: Some(CLEAN),
        ..CLEAN_PRESET
    },
    MINIMAL_PRESET,
    Preset {
        style_ref: MINIMAL_STILL,
        label: "Minimal (reduced motion)",
        animation: Animation::None,
        reduced_motion_of: Some(MINIMAL),
        ..MINIMAL_PRESET
    },
    BOXED_PRESET,
    Preset {
        style_ref: BOXED_STILL,
        label: "Boxed (reduced motion)",
        animation: Animation::None,
        unspoken: WHITE,
        reduced_motion_of: Some(BOXED),
        ..BOXED_PRESET
    },
];

/// The preset a `style_ref` names, or `None` when nothing does.
pub fn preset(style_ref: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|item| item.style_ref == style_ref)
}

/// The still twin of a preset, which is the preset itself when it has no
/// animation to remove.
pub fn reduced_motion(style_ref: &str) -> Option<&'static Preset> {
    let wanted = preset(style_ref)?;
    if matches!(wanted.animation, Animation::None) {
        return Some(wanted);
    }
    PRESETS
        .iter()
        .find(|item| item.reduced_motion_of == Some(wanted.style_ref))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        Animation, BOXED, CLEAN, DEFAULT_STYLE_REF, MINIMAL, PRESETS, preset, reduced_motion,
    };

    #[test]
    fn every_preset_has_a_still_variant_that_can_be_reached() {
        for item in PRESETS {
            let still = reduced_motion(item.style_ref).expect("a reduced-motion variant");
            assert!(matches!(still.animation, Animation::None));
        }
    }

    #[test]
    fn a_still_variant_changes_the_motion_and_not_the_type() {
        for named in [CLEAN, MINIMAL, BOXED] {
            let animated = preset(named).expect("the preset");
            let still = reduced_motion(named).expect("its still twin");
            assert_eq!(animated.font_family, still.font_family);
            assert_eq!(animated.font_size, still.font_size);
            assert_eq!(animated.spoken, still.spoken);
            assert_eq!(animated.border, still.border);
        }
    }

    #[test]
    fn a_still_variant_never_renders_a_colour_the_animation_only_passed_through() {
        for named in [CLEAN, BOXED] {
            let still = reduced_motion(named).expect("its still twin");
            assert_eq!(
                still.spoken, still.unspoken,
                "a still cue has no unsung state"
            );
        }
    }

    #[test]
    fn style_references_are_unique_and_the_default_is_one_of_them() {
        let mut names: Vec<&str> = PRESETS.iter().map(|item| item.style_ref).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "two presets share a name");
        assert!(preset(DEFAULT_STYLE_REF).is_some());
    }

    #[test]
    fn a_name_nothing_defines_resolves_to_nothing() {
        assert!(preset("clipmill.captions.invented.v1").is_none());
        assert!(reduced_motion("clipmill.captions.invented.v1").is_none());
    }
}
