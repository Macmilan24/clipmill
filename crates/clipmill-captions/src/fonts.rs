//! The typefaces a caption may be set in.
//!
//! Captions rasterize through libass, which finds a face by its family name
//! among the fonts it is shown. Each render is shown only the pinned files it
//! uses, so the list here is the whole universe of caption type: a family
//! that is not listed cannot be chosen, and a listed one names the exact file
//! `bom.toml` pins for it.
//!
//! Every face is a single weight. `bold` is the flag that selects it without
//! libass synthesizing a heavier one: Inter is pinned as its Bold, and the
//! display faces are drawn at the weight they were cut.

/// One caption typeface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptionFont {
    /// The font's own family name (name ID 1), which is what an ASS style
    /// names and what libass matches.
    pub family: &'static str,
    /// What a person picks it by.
    pub label: &'static str,
    /// The pinned file, in the caption fonts directory.
    pub file: &'static str,
    /// The ASS bold flag that selects this face as it is, with no synthesis.
    pub bold: bool,
    /// The licence the face is distributed under.
    pub license: &'static str,
}

/// The family every preset is set in, and the one a render always has.
pub const DEFAULT_FONT: &str = "Inter";

pub const FONTS: &[CaptionFont] = &[
    CaptionFont {
        family: "Inter",
        label: "Inter",
        file: "Inter-Bold.ttf",
        bold: true,
        license: "OFL-1.1",
    },
    CaptionFont {
        family: "Montserrat Black",
        label: "Montserrat",
        file: "Montserrat-Black.ttf",
        bold: false,
        license: "OFL-1.1",
    },
    CaptionFont {
        family: "Poppins ExtraBold",
        label: "Poppins",
        file: "Poppins-ExtraBold.ttf",
        bold: false,
        license: "OFL-1.1",
    },
    CaptionFont {
        family: "Anton",
        label: "Anton",
        file: "Anton-Regular.ttf",
        bold: false,
        license: "OFL-1.1",
    },
    CaptionFont {
        family: "Bebas Neue",
        label: "Bebas Neue",
        file: "BebasNeue-Regular.ttf",
        bold: false,
        license: "OFL-1.1",
    },
    CaptionFont {
        family: "Luckiest Guy",
        label: "Luckiest Guy",
        file: "LuckiestGuy-Regular.ttf",
        bold: false,
        license: "Apache-2.0",
    },
    CaptionFont {
        family: "DM Serif Display",
        label: "DM Serif",
        file: "DMSerifDisplay-Regular.ttf",
        bold: false,
        license: "OFL-1.1",
    },
];

/// The face a family name names, if it is one of the caption fonts.
pub fn font(family: &str) -> Option<&'static CaptionFont> {
    FONTS.iter().find(|font| font.family == family)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_FONT, FONTS, font};

    #[test]
    fn every_face_is_named_once_and_the_default_is_among_them() {
        assert!(font(DEFAULT_FONT).is_some());
        for (index, face) in FONTS.iter().enumerate() {
            assert!(
                FONTS[index + 1..]
                    .iter()
                    .all(|other| other.family != face.family && other.file != face.file),
                "{} is listed twice",
                face.family
            );
        }
        assert!(font("Comic Sans MS").is_none());
    }
}
