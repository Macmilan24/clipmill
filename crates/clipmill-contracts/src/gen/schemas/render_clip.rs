#![allow(clippy::redundant_closure_call)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::clone_on_copy)]

#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
#[doc = "An ASS colour. Alpha is inverted there, so 0 transparency is opaque."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"An ASS colour. Alpha is inverted there, so 0 transparency is opaque.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"blue\","]
#[doc = "    \"green\","]
#[doc = "    \"red\","]
#[doc = "    \"transparency\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"blue\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 255.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"green\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 255.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"red\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 255.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"transparency\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 255.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Colour {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub transparency: u8,
}
impl Colour {
    pub fn builder() -> builder::Colour {
        Default::default()
    }
}
#[doc = "`MeasuredLoudness`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"integrated_lufs\","]
#[doc = "    \"loudness_range_lu\","]
#[doc = "    \"true_peak_dbtp\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"integrated_lufs\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"loudness_range_lu\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"true_peak_dbtp\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MeasuredLoudness {
    pub integrated_lufs: f64,
    pub loudness_range_lu: f64,
    pub true_peak_dbtp: f64,
}
impl MeasuredLoudness {
    pub fn builder() -> builder::MeasuredLoudness {
        Default::default()
    }
}
#[doc = "Every knob that decides what the encoder produces. The profile is part of the render recipe, so changing any of it produces a different artifact rather than different pixels under the same content address."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Every knob that decides what the encoder produces. The profile is part of the render recipe, so changing any of it produces a different artifact rather than different pixels under the same content address.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"audio_bitrate\","]
#[doc = "    \"audio_channels\","]
#[doc = "    \"audio_codec\","]
#[doc = "    \"audio_sample_rate\","]
#[doc = "    \"caption_style\","]
#[doc = "    \"crf\","]
#[doc = "    \"fit_background_sigma\","]
#[doc = "    \"frame_rate\","]
#[doc = "    \"height\","]
#[doc = "    \"loudness\","]
#[doc = "    \"pixel_format\","]
#[doc = "    \"preset\","]
#[doc = "    \"profile_id\","]
#[doc = "    \"video_codec\","]
#[doc = "    \"width\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"audio_bitrate\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"audio_channels\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"audio_codec\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"audio_sample_rate\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"caption_style\": {"]
#[doc = "      \"description\": \"The subset of ASS styling Phase 1 exposes. Line breaking is deliberately absent: breaks are decided once and stored in the Edit IR, and libass is configured never to re-wrap.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"bold\","]
#[doc = "        \"font_family\","]
#[doc = "        \"font_size\","]
#[doc = "        \"margin_horizontal\","]
#[doc = "        \"margin_vertical\","]
#[doc = "        \"outline\","]
#[doc = "        \"outline_width\","]
#[doc = "        \"shadow\","]
#[doc = "        \"shadow_depth\","]
#[doc = "        \"spoken\","]
#[doc = "        \"style_ref\","]
#[doc = "        \"unspoken\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"bold\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"font_family\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"font_size\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"margin_horizontal\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"margin_vertical\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"outline\": {"]
#[doc = "          \"$ref\": \"#/$defs/colour\""]
#[doc = "        },"]
#[doc = "        \"outline_width\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"shadow\": {"]
#[doc = "          \"$ref\": \"#/$defs/colour\""]
#[doc = "        },"]
#[doc = "        \"shadow_depth\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"spoken\": {"]
#[doc = "          \"$ref\": \"#/$defs/colour\""]
#[doc = "        },"]
#[doc = "        \"style_ref\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"unspoken\": {"]
#[doc = "          \"$ref\": \"#/$defs/colour\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"crf\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 63.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"fit_background_sigma\": {"]
#[doc = "      \"description\": \"Blur applied to the filled background behind a letterboxed frame.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"frame_rate\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"den\","]
#[doc = "        \"num\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"den\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"num\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"height\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 2.0"]
#[doc = "    },"]
#[doc = "    \"loudness\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"integrated_lufs\","]
#[doc = "        \"range_lu\","]
#[doc = "        \"true_peak_dbtp\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"integrated_lufs\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"range_lu\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"true_peak_dbtp\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"pixel_format\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"preset\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"profile_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"video_codec\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"width\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 2.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub audio_bitrate: ::std::num::NonZeroU64,
    pub audio_channels: ::std::num::NonZeroU64,
    pub audio_codec: ProfileAudioCodec,
    pub audio_sample_rate: ::std::num::NonZeroU64,
    pub caption_style: ProfileCaptionStyle,
    pub crf: i64,
    #[doc = "Blur applied to the filled background behind a letterboxed frame."]
    pub fit_background_sigma: u64,
    pub frame_rate: ProfileFrameRate,
    pub height: i64,
    pub loudness: ProfileLoudness,
    pub pixel_format: ProfilePixelFormat,
    pub preset: ProfilePreset,
    pub profile_id: ProfileProfileId,
    pub video_codec: ProfileVideoCodec,
    pub width: i64,
}
impl Profile {
    pub fn builder() -> builder::Profile {
        Default::default()
    }
}
#[doc = "`ProfileAudioCodec`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProfileAudioCodec(::std::string::String);
impl ::std::ops::Deref for ProfileAudioCodec {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProfileAudioCodec> for ::std::string::String {
    fn from(value: ProfileAudioCodec) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProfileAudioCodec {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProfileAudioCodec {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProfileAudioCodec {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProfileAudioCodec {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProfileAudioCodec {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "The subset of ASS styling Phase 1 exposes. Line breaking is deliberately absent: breaks are decided once and stored in the Edit IR, and libass is configured never to re-wrap."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The subset of ASS styling Phase 1 exposes. Line breaking is deliberately absent: breaks are decided once and stored in the Edit IR, and libass is configured never to re-wrap.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"bold\","]
#[doc = "    \"font_family\","]
#[doc = "    \"font_size\","]
#[doc = "    \"margin_horizontal\","]
#[doc = "    \"margin_vertical\","]
#[doc = "    \"outline\","]
#[doc = "    \"outline_width\","]
#[doc = "    \"shadow\","]
#[doc = "    \"shadow_depth\","]
#[doc = "    \"spoken\","]
#[doc = "    \"style_ref\","]
#[doc = "    \"unspoken\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bold\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"font_family\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"font_size\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"margin_horizontal\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"margin_vertical\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"outline\": {"]
#[doc = "      \"$ref\": \"#/$defs/colour\""]
#[doc = "    },"]
#[doc = "    \"outline_width\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"shadow\": {"]
#[doc = "      \"$ref\": \"#/$defs/colour\""]
#[doc = "    },"]
#[doc = "    \"shadow_depth\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"spoken\": {"]
#[doc = "      \"$ref\": \"#/$defs/colour\""]
#[doc = "    },"]
#[doc = "    \"style_ref\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"unspoken\": {"]
#[doc = "      \"$ref\": \"#/$defs/colour\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProfileCaptionStyle {
    pub bold: bool,
    pub font_family: ProfileCaptionStyleFontFamily,
    pub font_size: ::std::num::NonZeroU64,
    pub margin_horizontal: u64,
    pub margin_vertical: u64,
    pub outline: Colour,
    pub outline_width: u64,
    pub shadow: Colour,
    pub shadow_depth: u64,
    pub spoken: Colour,
    pub style_ref: ProfileCaptionStyleStyleRef,
    pub unspoken: Colour,
}
impl ProfileCaptionStyle {
    pub fn builder() -> builder::ProfileCaptionStyle {
        Default::default()
    }
}
#[doc = "`ProfileCaptionStyleFontFamily`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProfileCaptionStyleFontFamily(::std::string::String);
impl ::std::ops::Deref for ProfileCaptionStyleFontFamily {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProfileCaptionStyleFontFamily> for ::std::string::String {
    fn from(value: ProfileCaptionStyleFontFamily) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProfileCaptionStyleFontFamily {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProfileCaptionStyleFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProfileCaptionStyleFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProfileCaptionStyleFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProfileCaptionStyleFontFamily {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`ProfileCaptionStyleStyleRef`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProfileCaptionStyleStyleRef(::std::string::String);
impl ::std::ops::Deref for ProfileCaptionStyleStyleRef {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProfileCaptionStyleStyleRef> for ::std::string::String {
    fn from(value: ProfileCaptionStyleStyleRef) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProfileCaptionStyleStyleRef {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProfileCaptionStyleStyleRef {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProfileCaptionStyleStyleRef {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProfileCaptionStyleStyleRef {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProfileCaptionStyleStyleRef {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`ProfileFrameRate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"den\","]
#[doc = "    \"num\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"den\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"num\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProfileFrameRate {
    pub den: ::std::num::NonZeroU64,
    pub num: ::std::num::NonZeroU64,
}
impl ProfileFrameRate {
    pub fn builder() -> builder::ProfileFrameRate {
        Default::default()
    }
}
#[doc = "`ProfileLoudness`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"integrated_lufs\","]
#[doc = "    \"range_lu\","]
#[doc = "    \"true_peak_dbtp\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"integrated_lufs\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"range_lu\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"true_peak_dbtp\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProfileLoudness {
    pub integrated_lufs: f64,
    pub range_lu: f64,
    pub true_peak_dbtp: f64,
}
impl ProfileLoudness {
    pub fn builder() -> builder::ProfileLoudness {
        Default::default()
    }
}
#[doc = "`ProfilePixelFormat`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProfilePixelFormat(::std::string::String);
impl ::std::ops::Deref for ProfilePixelFormat {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProfilePixelFormat> for ::std::string::String {
    fn from(value: ProfilePixelFormat) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProfilePixelFormat {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProfilePixelFormat {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProfilePixelFormat {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProfilePixelFormat {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProfilePixelFormat {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`ProfilePreset`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProfilePreset(::std::string::String);
impl ::std::ops::Deref for ProfilePreset {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProfilePreset> for ::std::string::String {
    fn from(value: ProfilePreset) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProfilePreset {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProfilePreset {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProfilePreset {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProfilePreset {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProfilePreset {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`ProfileProfileId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProfileProfileId(::std::string::String);
impl ::std::ops::Deref for ProfileProfileId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProfileProfileId> for ::std::string::String {
    fn from(value: ProfileProfileId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProfileProfileId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProfileProfileId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProfileProfileId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProfileProfileId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProfileProfileId {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`ProfileVideoCodec`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProfileVideoCodec(::std::string::String);
impl ::std::ops::Deref for ProfileVideoCodec {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProfileVideoCodec> for ::std::string::String {
    fn from(value: ProfileVideoCodec) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProfileVideoCodec {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProfileVideoCodec {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProfileVideoCodec {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProfileVideoCodec {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProfileVideoCodec {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "The render manifest that ships beside a finished clip (book ch. 17, appendix B). It answers, without anyone re-deriving anything: which Edit IR produced these pixels, which engine and font rasterized them, what the loudness actually measured, what rights position the user attested, and where a model's work appears. Rendering is model-free by construction, so anything listed under ai_use_summary entered through the document, never through the renderer."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.render.clip.v1.json\","]
#[doc = "  \"title\": \"RenderClipManifest\","]
#[doc = "  \"description\": \"The render manifest that ships beside a finished clip (book ch. 17, appendix B). It answers, without anyone re-deriving anything: which Edit IR produced these pixels, which engine and font rasterized them, what the loudness actually measured, what rights position the user attested, and where a model's work appears. Rendering is model-free by construction, so anything listed under ai_use_summary entered through the document, never through the renderer.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"ai_use_summary\","]
#[doc = "    \"caption_windows\","]
#[doc = "    \"determinism\","]
#[doc = "    \"engine\","]
#[doc = "    \"input_source_fingerprints\","]
#[doc = "    \"ir_artifact_id\","]
#[doc = "    \"ir_hash\","]
#[doc = "    \"loudness\","]
#[doc = "    \"outputs\","]
#[doc = "    \"profile\","]
#[doc = "    \"program\","]
#[doc = "    \"rights\","]
#[doc = "    \"schema_version\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"ai_use_summary\": {"]
#[doc = "      \"description\": \"Where a model's work appears in the result. Supplied by whoever authored the document rather than inferred here: a disclosure a renderer guessed is a disclosure nobody checked.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"assistance\","]
#[doc = "        \"generated\","]
#[doc = "        \"requires_youtube_ai_disclosure\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"assistance\": {"]
#[doc = "          \"description\": \"Model work that shaped existing footage, e.g. asr_captions, reframe.\","]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"generated\": {"]
#[doc = "          \"description\": \"Synthesised imagery or audio.\","]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"requires_youtube_ai_disclosure\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"caption_windows\": {"]
#[doc = "      \"description\": \"The frames each cue occupies in the finished file, so checking captions against the IR is reading a number rather than re-deriving one.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"cue_id\","]
#[doc = "          \"end_frame\","]
#[doc = "          \"first_frame\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"cue_id\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          },"]
#[doc = "          \"end_frame\": {"]
#[doc = "            \"description\": \"Exclusive: the first frame the cue is no longer drawn on.\","]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"first_frame\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"determinism\": {"]
#[doc = "      \"description\": \"byte_stable where the encoder reproduces bytes for identical inputs on this platform; semantic where only the decoded result is guaranteed to match.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"byte_stable\","]
#[doc = "        \"semantic\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"engine\": {"]
#[doc = "      \"description\": \"Everything outside the document that could change the output.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"app\","]
#[doc = "        \"ffmpeg\","]
#[doc = "        \"font_family\","]
#[doc = "        \"font_sha256\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"app\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"ffmpeg\": {"]
#[doc = "          \"description\": \"Pinned FFmpeg substrate identity (decision R4).\","]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"font_family\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"font_sha256\": {"]
#[doc = "          \"description\": \"Digest of the single font libass was allowed to see.\","]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"input_source_fingerprints\": {"]
#[doc = "      \"description\": \"Every source that contributed frames, in document order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/sha256\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"ir_artifact_id\": {"]
#[doc = "      \"description\": \"Content address of that snapshot artifact.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"ir_hash\": {"]
#[doc = "      \"description\": \"Digest of the canonical Edit IR snapshot this render read. The snapshot carries the render projection only, so re-explaining an edit cannot change this value.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"loudness\": {"]
#[doc = "      \"description\": \"Targets and both measurements. The output figure is re-decoded from the finished file rather than predicted from the filter's arguments.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"measured_input\","]
#[doc = "        \"measured_output\","]
#[doc = "        \"target_lufs\","]
#[doc = "        \"target_true_peak_dbtp\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"measured_input\": {"]
#[doc = "          \"$ref\": \"#/$defs/measured_loudness\""]
#[doc = "        },"]
#[doc = "        \"measured_output\": {"]
#[doc = "          \"$ref\": \"#/$defs/measured_loudness\""]
#[doc = "        },"]
#[doc = "        \"target_lufs\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"target_true_peak_dbtp\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"outputs\": {"]
#[doc = "      \"description\": \"Every published file, with the digest a recipient can verify.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"bytes\","]
#[doc = "          \"path\","]
#[doc = "          \"sha256\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"bytes\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"path\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          },"]
#[doc = "          \"sha256\": {"]
#[doc = "            \"$ref\": \"#/$defs/sha256\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"profile\": {"]
#[doc = "      \"$ref\": \"#/$defs/profile\""]
#[doc = "    },"]
#[doc = "    \"program\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"duration_ticks\","]
#[doc = "        \"frame_count\","]
#[doc = "        \"segments\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"duration_ticks\": {"]
#[doc = "          \"description\": \"Program duration in ticks at 1/90000.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"frame_count\": {"]
#[doc = "          \"description\": \"Frames the encoder was pinned to produce, and the count the finished file was verified to carry.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"segments\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"frame_count\","]
#[doc = "              \"in_ticks\","]
#[doc = "              \"layout\","]
#[doc = "              \"out_ticks\","]
#[doc = "              \"segment_id\","]
#[doc = "              \"source_fingerprint\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"frame_count\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              },"]
#[doc = "              \"in_ticks\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              },"]
#[doc = "              \"layout\": {"]
#[doc = "                \"enum\": ["]
#[doc = "                  \"fit\","]
#[doc = "                  \"speaker_fill\""]
#[doc = "                ]"]
#[doc = "              },"]
#[doc = "              \"out_ticks\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              },"]
#[doc = "              \"segment_id\": {"]
#[doc = "                \"type\": \"string\","]
#[doc = "                \"minLength\": 1"]
#[doc = "              },"]
#[doc = "              \"source_fingerprint\": {"]
#[doc = "                \"$ref\": \"#/$defs/sha256\""]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          }"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"rights\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"gates_passed\","]
#[doc = "        \"source_attestation\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"gates_passed\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"source_attestation\": {"]
#[doc = "          \"description\": \"What the user attested about the footage, echoed verbatim.\","]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.render.clip.v1\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifest {
    pub ai_use_summary: RenderClipManifestAiUseSummary,
    #[doc = "The frames each cue occupies in the finished file, so checking captions against the IR is reading a number rather than re-deriving one."]
    pub caption_windows: ::std::vec::Vec<RenderClipManifestCaptionWindowsItem>,
    #[doc = "byte_stable where the encoder reproduces bytes for identical inputs on this platform; semantic where only the decoded result is guaranteed to match."]
    pub determinism: RenderClipManifestDeterminism,
    pub engine: RenderClipManifestEngine,
    #[doc = "Every source that contributed frames, in document order."]
    pub input_source_fingerprints: ::std::vec::Vec<Sha256>,
    #[doc = "Content address of that snapshot artifact."]
    pub ir_artifact_id: RenderClipManifestIrArtifactId,
    #[doc = "Digest of the canonical Edit IR snapshot this render read. The snapshot carries the render projection only, so re-explaining an edit cannot change this value."]
    pub ir_hash: Sha256,
    pub loudness: RenderClipManifestLoudness,
    #[doc = "Every published file, with the digest a recipient can verify."]
    pub outputs: ::std::vec::Vec<RenderClipManifestOutputsItem>,
    pub profile: Profile,
    pub program: RenderClipManifestProgram,
    pub rights: RenderClipManifestRights,
    pub schema_version: ::serde_json::Value,
}
impl RenderClipManifest {
    pub fn builder() -> builder::RenderClipManifest {
        Default::default()
    }
}
#[doc = "Where a model's work appears in the result. Supplied by whoever authored the document rather than inferred here: a disclosure a renderer guessed is a disclosure nobody checked."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Where a model's work appears in the result. Supplied by whoever authored the document rather than inferred here: a disclosure a renderer guessed is a disclosure nobody checked.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"assistance\","]
#[doc = "    \"generated\","]
#[doc = "    \"requires_youtube_ai_disclosure\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"assistance\": {"]
#[doc = "      \"description\": \"Model work that shaped existing footage, e.g. asr_captions, reframe.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"generated\": {"]
#[doc = "      \"description\": \"Synthesised imagery or audio.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"requires_youtube_ai_disclosure\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifestAiUseSummary {
    #[doc = "Model work that shaped existing footage, e.g. asr_captions, reframe."]
    pub assistance: ::std::vec::Vec<RenderClipManifestAiUseSummaryAssistanceItem>,
    #[doc = "Synthesised imagery or audio."]
    pub generated: ::std::vec::Vec<RenderClipManifestAiUseSummaryGeneratedItem>,
    pub requires_youtube_ai_disclosure: bool,
}
impl RenderClipManifestAiUseSummary {
    pub fn builder() -> builder::RenderClipManifestAiUseSummary {
        Default::default()
    }
}
#[doc = "`RenderClipManifestAiUseSummaryAssistanceItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestAiUseSummaryAssistanceItem(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestAiUseSummaryAssistanceItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestAiUseSummaryAssistanceItem> for ::std::string::String {
    fn from(value: RenderClipManifestAiUseSummaryAssistanceItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestAiUseSummaryAssistanceItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestAiUseSummaryAssistanceItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String>
    for RenderClipManifestAiUseSummaryAssistanceItem
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String>
    for RenderClipManifestAiUseSummaryAssistanceItem
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestAiUseSummaryAssistanceItem {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`RenderClipManifestAiUseSummaryGeneratedItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestAiUseSummaryGeneratedItem(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestAiUseSummaryGeneratedItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestAiUseSummaryGeneratedItem> for ::std::string::String {
    fn from(value: RenderClipManifestAiUseSummaryGeneratedItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestAiUseSummaryGeneratedItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestAiUseSummaryGeneratedItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String>
    for RenderClipManifestAiUseSummaryGeneratedItem
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String>
    for RenderClipManifestAiUseSummaryGeneratedItem
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestAiUseSummaryGeneratedItem {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`RenderClipManifestCaptionWindowsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cue_id\","]
#[doc = "    \"end_frame\","]
#[doc = "    \"first_frame\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cue_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"end_frame\": {"]
#[doc = "      \"description\": \"Exclusive: the first frame the cue is no longer drawn on.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"first_frame\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifestCaptionWindowsItem {
    pub cue_id: RenderClipManifestCaptionWindowsItemCueId,
    #[doc = "Exclusive: the first frame the cue is no longer drawn on."]
    pub end_frame: u64,
    pub first_frame: u64,
}
impl RenderClipManifestCaptionWindowsItem {
    pub fn builder() -> builder::RenderClipManifestCaptionWindowsItem {
        Default::default()
    }
}
#[doc = "`RenderClipManifestCaptionWindowsItemCueId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestCaptionWindowsItemCueId(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestCaptionWindowsItemCueId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestCaptionWindowsItemCueId> for ::std::string::String {
    fn from(value: RenderClipManifestCaptionWindowsItemCueId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestCaptionWindowsItemCueId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestCaptionWindowsItemCueId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestCaptionWindowsItemCueId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestCaptionWindowsItemCueId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestCaptionWindowsItemCueId {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "byte_stable where the encoder reproduces bytes for identical inputs on this platform; semantic where only the decoded result is guaranteed to match."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"byte_stable where the encoder reproduces bytes for identical inputs on this platform; semantic where only the decoded result is guaranteed to match.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"byte_stable\","]
#[doc = "    \"semantic\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum RenderClipManifestDeterminism {
    #[serde(rename = "byte_stable")]
    ByteStable,
    #[serde(rename = "semantic")]
    Semantic,
}
impl ::std::fmt::Display for RenderClipManifestDeterminism {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ByteStable => f.write_str("byte_stable"),
            Self::Semantic => f.write_str("semantic"),
        }
    }
}
impl ::std::str::FromStr for RenderClipManifestDeterminism {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "byte_stable" => Ok(Self::ByteStable),
            "semantic" => Ok(Self::Semantic),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestDeterminism {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestDeterminism {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestDeterminism {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Everything outside the document that could change the output."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Everything outside the document that could change the output.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"app\","]
#[doc = "    \"ffmpeg\","]
#[doc = "    \"font_family\","]
#[doc = "    \"font_sha256\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"app\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"ffmpeg\": {"]
#[doc = "      \"description\": \"Pinned FFmpeg substrate identity (decision R4).\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"font_family\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"font_sha256\": {"]
#[doc = "      \"description\": \"Digest of the single font libass was allowed to see.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifestEngine {
    pub app: RenderClipManifestEngineApp,
    #[doc = "Pinned FFmpeg substrate identity (decision R4)."]
    pub ffmpeg: RenderClipManifestEngineFfmpeg,
    pub font_family: RenderClipManifestEngineFontFamily,
    #[doc = "Digest of the single font libass was allowed to see."]
    pub font_sha256: Sha256,
}
impl RenderClipManifestEngine {
    pub fn builder() -> builder::RenderClipManifestEngine {
        Default::default()
    }
}
#[doc = "`RenderClipManifestEngineApp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestEngineApp(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestEngineApp {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestEngineApp> for ::std::string::String {
    fn from(value: RenderClipManifestEngineApp) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestEngineApp {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestEngineApp {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestEngineApp {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestEngineApp {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestEngineApp {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "Pinned FFmpeg substrate identity (decision R4)."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Pinned FFmpeg substrate identity (decision R4).\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestEngineFfmpeg(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestEngineFfmpeg {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestEngineFfmpeg> for ::std::string::String {
    fn from(value: RenderClipManifestEngineFfmpeg) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestEngineFfmpeg {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestEngineFfmpeg {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestEngineFfmpeg {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestEngineFfmpeg {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestEngineFfmpeg {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`RenderClipManifestEngineFontFamily`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestEngineFontFamily(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestEngineFontFamily {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestEngineFontFamily> for ::std::string::String {
    fn from(value: RenderClipManifestEngineFontFamily) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestEngineFontFamily {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestEngineFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestEngineFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestEngineFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestEngineFontFamily {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "Content address of that snapshot artifact."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Content address of that snapshot artifact.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestIrArtifactId(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestIrArtifactId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestIrArtifactId> for ::std::string::String {
    fn from(value: RenderClipManifestIrArtifactId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestIrArtifactId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestIrArtifactId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestIrArtifactId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestIrArtifactId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestIrArtifactId {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "Targets and both measurements. The output figure is re-decoded from the finished file rather than predicted from the filter's arguments."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Targets and both measurements. The output figure is re-decoded from the finished file rather than predicted from the filter's arguments.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"measured_input\","]
#[doc = "    \"measured_output\","]
#[doc = "    \"target_lufs\","]
#[doc = "    \"target_true_peak_dbtp\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"measured_input\": {"]
#[doc = "      \"$ref\": \"#/$defs/measured_loudness\""]
#[doc = "    },"]
#[doc = "    \"measured_output\": {"]
#[doc = "      \"$ref\": \"#/$defs/measured_loudness\""]
#[doc = "    },"]
#[doc = "    \"target_lufs\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"target_true_peak_dbtp\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifestLoudness {
    pub measured_input: MeasuredLoudness,
    pub measured_output: MeasuredLoudness,
    pub target_lufs: f64,
    pub target_true_peak_dbtp: f64,
}
impl RenderClipManifestLoudness {
    pub fn builder() -> builder::RenderClipManifestLoudness {
        Default::default()
    }
}
#[doc = "`RenderClipManifestOutputsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"bytes\","]
#[doc = "    \"path\","]
#[doc = "    \"sha256\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bytes\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"path\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"sha256\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifestOutputsItem {
    pub bytes: u64,
    pub path: RenderClipManifestOutputsItemPath,
    pub sha256: Sha256,
}
impl RenderClipManifestOutputsItem {
    pub fn builder() -> builder::RenderClipManifestOutputsItem {
        Default::default()
    }
}
#[doc = "`RenderClipManifestOutputsItemPath`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestOutputsItemPath(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestOutputsItemPath {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestOutputsItemPath> for ::std::string::String {
    fn from(value: RenderClipManifestOutputsItemPath) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestOutputsItemPath {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestOutputsItemPath {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestOutputsItemPath {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestOutputsItemPath {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestOutputsItemPath {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`RenderClipManifestProgram`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"duration_ticks\","]
#[doc = "    \"frame_count\","]
#[doc = "    \"segments\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"duration_ticks\": {"]
#[doc = "      \"description\": \"Program duration in ticks at 1/90000.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"frame_count\": {"]
#[doc = "      \"description\": \"Frames the encoder was pinned to produce, and the count the finished file was verified to carry.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"segments\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"frame_count\","]
#[doc = "          \"in_ticks\","]
#[doc = "          \"layout\","]
#[doc = "          \"out_ticks\","]
#[doc = "          \"segment_id\","]
#[doc = "          \"source_fingerprint\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"frame_count\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"in_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"layout\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"fit\","]
#[doc = "              \"speaker_fill\""]
#[doc = "            ]"]
#[doc = "          },"]
#[doc = "          \"out_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"segment_id\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          },"]
#[doc = "          \"source_fingerprint\": {"]
#[doc = "            \"$ref\": \"#/$defs/sha256\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifestProgram {
    #[doc = "Program duration in ticks at 1/90000."]
    pub duration_ticks: u64,
    #[doc = "Frames the encoder was pinned to produce, and the count the finished file was verified to carry."]
    pub frame_count: u64,
    pub segments: ::std::vec::Vec<RenderClipManifestProgramSegmentsItem>,
}
impl RenderClipManifestProgram {
    pub fn builder() -> builder::RenderClipManifestProgram {
        Default::default()
    }
}
#[doc = "`RenderClipManifestProgramSegmentsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"frame_count\","]
#[doc = "    \"in_ticks\","]
#[doc = "    \"layout\","]
#[doc = "    \"out_ticks\","]
#[doc = "    \"segment_id\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"frame_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"in_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"layout\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"fit\","]
#[doc = "        \"speaker_fill\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"out_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"segment_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifestProgramSegmentsItem {
    pub frame_count: u64,
    pub in_ticks: u64,
    pub layout: RenderClipManifestProgramSegmentsItemLayout,
    pub out_ticks: u64,
    pub segment_id: RenderClipManifestProgramSegmentsItemSegmentId,
    pub source_fingerprint: Sha256,
}
impl RenderClipManifestProgramSegmentsItem {
    pub fn builder() -> builder::RenderClipManifestProgramSegmentsItem {
        Default::default()
    }
}
#[doc = "`RenderClipManifestProgramSegmentsItemLayout`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"fit\","]
#[doc = "    \"speaker_fill\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum RenderClipManifestProgramSegmentsItemLayout {
    #[serde(rename = "fit")]
    Fit,
    #[serde(rename = "speaker_fill")]
    SpeakerFill,
}
impl ::std::fmt::Display for RenderClipManifestProgramSegmentsItemLayout {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Fit => f.write_str("fit"),
            Self::SpeakerFill => f.write_str("speaker_fill"),
        }
    }
}
impl ::std::str::FromStr for RenderClipManifestProgramSegmentsItemLayout {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "fit" => Ok(Self::Fit),
            "speaker_fill" => Ok(Self::SpeakerFill),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestProgramSegmentsItemLayout {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String>
    for RenderClipManifestProgramSegmentsItemLayout
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String>
    for RenderClipManifestProgramSegmentsItemLayout
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`RenderClipManifestProgramSegmentsItemSegmentId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestProgramSegmentsItemSegmentId(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestProgramSegmentsItemSegmentId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestProgramSegmentsItemSegmentId>
    for ::std::string::String
{
    fn from(value: RenderClipManifestProgramSegmentsItemSegmentId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestProgramSegmentsItemSegmentId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestProgramSegmentsItemSegmentId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String>
    for RenderClipManifestProgramSegmentsItemSegmentId
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String>
    for RenderClipManifestProgramSegmentsItemSegmentId
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestProgramSegmentsItemSegmentId {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`RenderClipManifestRights`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"gates_passed\","]
#[doc = "    \"source_attestation\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"gates_passed\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"source_attestation\": {"]
#[doc = "      \"description\": \"What the user attested about the footage, echoed verbatim.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderClipManifestRights {
    pub gates_passed: ::std::vec::Vec<RenderClipManifestRightsGatesPassedItem>,
    #[doc = "What the user attested about the footage, echoed verbatim."]
    pub source_attestation: RenderClipManifestRightsSourceAttestation,
}
impl RenderClipManifestRights {
    pub fn builder() -> builder::RenderClipManifestRights {
        Default::default()
    }
}
#[doc = "`RenderClipManifestRightsGatesPassedItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestRightsGatesPassedItem(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestRightsGatesPassedItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestRightsGatesPassedItem> for ::std::string::String {
    fn from(value: RenderClipManifestRightsGatesPassedItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestRightsGatesPassedItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestRightsGatesPassedItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestRightsGatesPassedItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestRightsGatesPassedItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestRightsGatesPassedItem {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "What the user attested about the footage, echoed verbatim."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What the user attested about the footage, echoed verbatim.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RenderClipManifestRightsSourceAttestation(::std::string::String);
impl ::std::ops::Deref for RenderClipManifestRightsSourceAttestation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RenderClipManifestRightsSourceAttestation> for ::std::string::String {
    fn from(value: RenderClipManifestRightsSourceAttestation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RenderClipManifestRightsSourceAttestation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RenderClipManifestRightsSourceAttestation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RenderClipManifestRightsSourceAttestation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RenderClipManifestRightsSourceAttestation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RenderClipManifestRightsSourceAttestation {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`Sha256`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^sha256:[0-9a-f]{64}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Sha256(::std::string::String);
impl ::std::ops::Deref for Sha256 {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Sha256> for ::std::string::String {
    fn from(value: Sha256) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Sha256 {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^sha256:[0-9a-f]{64}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^sha256:[0-9a-f]{64}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for Sha256 {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Sha256 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Sha256 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Sha256 {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct Colour {
        blue: ::std::result::Result<u8, ::std::string::String>,
        green: ::std::result::Result<u8, ::std::string::String>,
        red: ::std::result::Result<u8, ::std::string::String>,
        transparency: ::std::result::Result<u8, ::std::string::String>,
    }
    impl ::std::default::Default for Colour {
        fn default() -> Self {
            Self {
                blue: Err("no value supplied for blue".to_string()),
                green: Err("no value supplied for green".to_string()),
                red: Err("no value supplied for red".to_string()),
                transparency: Err("no value supplied for transparency".to_string()),
            }
        }
    }
    impl Colour {
        pub fn blue<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u8>,
            T::Error: ::std::fmt::Display,
        {
            self.blue = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for blue: {e}"));
            self
        }
        pub fn green<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u8>,
            T::Error: ::std::fmt::Display,
        {
            self.green = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for green: {e}"));
            self
        }
        pub fn red<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u8>,
            T::Error: ::std::fmt::Display,
        {
            self.red = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for red: {e}"));
            self
        }
        pub fn transparency<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u8>,
            T::Error: ::std::fmt::Display,
        {
            self.transparency = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transparency: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Colour> for super::Colour {
        type Error = super::error::ConversionError;
        fn try_from(value: Colour) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                blue: value.blue?,
                green: value.green?,
                red: value.red?,
                transparency: value.transparency?,
            })
        }
    }
    impl ::std::convert::From<super::Colour> for Colour {
        fn from(value: super::Colour) -> Self {
            Self {
                blue: Ok(value.blue),
                green: Ok(value.green),
                red: Ok(value.red),
                transparency: Ok(value.transparency),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MeasuredLoudness {
        integrated_lufs: ::std::result::Result<f64, ::std::string::String>,
        loudness_range_lu: ::std::result::Result<f64, ::std::string::String>,
        true_peak_dbtp: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for MeasuredLoudness {
        fn default() -> Self {
            Self {
                integrated_lufs: Err("no value supplied for integrated_lufs".to_string()),
                loudness_range_lu: Err("no value supplied for loudness_range_lu".to_string()),
                true_peak_dbtp: Err("no value supplied for true_peak_dbtp".to_string()),
            }
        }
    }
    impl MeasuredLoudness {
        pub fn integrated_lufs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.integrated_lufs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for integrated_lufs: {e}"));
            self
        }
        pub fn loudness_range_lu<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.loudness_range_lu = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for loudness_range_lu: {e}"));
            self
        }
        pub fn true_peak_dbtp<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.true_peak_dbtp = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for true_peak_dbtp: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MeasuredLoudness> for super::MeasuredLoudness {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MeasuredLoudness,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                integrated_lufs: value.integrated_lufs?,
                loudness_range_lu: value.loudness_range_lu?,
                true_peak_dbtp: value.true_peak_dbtp?,
            })
        }
    }
    impl ::std::convert::From<super::MeasuredLoudness> for MeasuredLoudness {
        fn from(value: super::MeasuredLoudness) -> Self {
            Self {
                integrated_lufs: Ok(value.integrated_lufs),
                loudness_range_lu: Ok(value.loudness_range_lu),
                true_peak_dbtp: Ok(value.true_peak_dbtp),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Profile {
        audio_bitrate: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        audio_channels: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        audio_codec: ::std::result::Result<super::ProfileAudioCodec, ::std::string::String>,
        audio_sample_rate: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        caption_style: ::std::result::Result<super::ProfileCaptionStyle, ::std::string::String>,
        crf: ::std::result::Result<i64, ::std::string::String>,
        fit_background_sigma: ::std::result::Result<u64, ::std::string::String>,
        frame_rate: ::std::result::Result<super::ProfileFrameRate, ::std::string::String>,
        height: ::std::result::Result<i64, ::std::string::String>,
        loudness: ::std::result::Result<super::ProfileLoudness, ::std::string::String>,
        pixel_format: ::std::result::Result<super::ProfilePixelFormat, ::std::string::String>,
        preset: ::std::result::Result<super::ProfilePreset, ::std::string::String>,
        profile_id: ::std::result::Result<super::ProfileProfileId, ::std::string::String>,
        video_codec: ::std::result::Result<super::ProfileVideoCodec, ::std::string::String>,
        width: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for Profile {
        fn default() -> Self {
            Self {
                audio_bitrate: Err("no value supplied for audio_bitrate".to_string()),
                audio_channels: Err("no value supplied for audio_channels".to_string()),
                audio_codec: Err("no value supplied for audio_codec".to_string()),
                audio_sample_rate: Err("no value supplied for audio_sample_rate".to_string()),
                caption_style: Err("no value supplied for caption_style".to_string()),
                crf: Err("no value supplied for crf".to_string()),
                fit_background_sigma: Err("no value supplied for fit_background_sigma".to_string()),
                frame_rate: Err("no value supplied for frame_rate".to_string()),
                height: Err("no value supplied for height".to_string()),
                loudness: Err("no value supplied for loudness".to_string()),
                pixel_format: Err("no value supplied for pixel_format".to_string()),
                preset: Err("no value supplied for preset".to_string()),
                profile_id: Err("no value supplied for profile_id".to_string()),
                video_codec: Err("no value supplied for video_codec".to_string()),
                width: Err("no value supplied for width".to_string()),
            }
        }
    }
    impl Profile {
        pub fn audio_bitrate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.audio_bitrate = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio_bitrate: {e}"));
            self
        }
        pub fn audio_channels<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.audio_channels = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio_channels: {e}"));
            self
        }
        pub fn audio_codec<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfileAudioCodec>,
            T::Error: ::std::fmt::Display,
        {
            self.audio_codec = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio_codec: {e}"));
            self
        }
        pub fn audio_sample_rate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.audio_sample_rate = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio_sample_rate: {e}"));
            self
        }
        pub fn caption_style<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfileCaptionStyle>,
            T::Error: ::std::fmt::Display,
        {
            self.caption_style = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for caption_style: {e}"));
            self
        }
        pub fn crf<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.crf = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for crf: {e}"));
            self
        }
        pub fn fit_background_sigma<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.fit_background_sigma = value.try_into().map_err(|e| {
                format!("error converting supplied value for fit_background_sigma: {e}")
            });
            self
        }
        pub fn frame_rate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfileFrameRate>,
            T::Error: ::std::fmt::Display,
        {
            self.frame_rate = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frame_rate: {e}"));
            self
        }
        pub fn height<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.height = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for height: {e}"));
            self
        }
        pub fn loudness<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfileLoudness>,
            T::Error: ::std::fmt::Display,
        {
            self.loudness = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for loudness: {e}"));
            self
        }
        pub fn pixel_format<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfilePixelFormat>,
            T::Error: ::std::fmt::Display,
        {
            self.pixel_format = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for pixel_format: {e}"));
            self
        }
        pub fn preset<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfilePreset>,
            T::Error: ::std::fmt::Display,
        {
            self.preset = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for preset: {e}"));
            self
        }
        pub fn profile_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfileProfileId>,
            T::Error: ::std::fmt::Display,
        {
            self.profile_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for profile_id: {e}"));
            self
        }
        pub fn video_codec<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfileVideoCodec>,
            T::Error: ::std::fmt::Display,
        {
            self.video_codec = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for video_codec: {e}"));
            self
        }
        pub fn width<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.width = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for width: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Profile> for super::Profile {
        type Error = super::error::ConversionError;
        fn try_from(value: Profile) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                audio_bitrate: value.audio_bitrate?,
                audio_channels: value.audio_channels?,
                audio_codec: value.audio_codec?,
                audio_sample_rate: value.audio_sample_rate?,
                caption_style: value.caption_style?,
                crf: value.crf?,
                fit_background_sigma: value.fit_background_sigma?,
                frame_rate: value.frame_rate?,
                height: value.height?,
                loudness: value.loudness?,
                pixel_format: value.pixel_format?,
                preset: value.preset?,
                profile_id: value.profile_id?,
                video_codec: value.video_codec?,
                width: value.width?,
            })
        }
    }
    impl ::std::convert::From<super::Profile> for Profile {
        fn from(value: super::Profile) -> Self {
            Self {
                audio_bitrate: Ok(value.audio_bitrate),
                audio_channels: Ok(value.audio_channels),
                audio_codec: Ok(value.audio_codec),
                audio_sample_rate: Ok(value.audio_sample_rate),
                caption_style: Ok(value.caption_style),
                crf: Ok(value.crf),
                fit_background_sigma: Ok(value.fit_background_sigma),
                frame_rate: Ok(value.frame_rate),
                height: Ok(value.height),
                loudness: Ok(value.loudness),
                pixel_format: Ok(value.pixel_format),
                preset: Ok(value.preset),
                profile_id: Ok(value.profile_id),
                video_codec: Ok(value.video_codec),
                width: Ok(value.width),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ProfileCaptionStyle {
        bold: ::std::result::Result<bool, ::std::string::String>,
        font_family:
            ::std::result::Result<super::ProfileCaptionStyleFontFamily, ::std::string::String>,
        font_size: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        margin_horizontal: ::std::result::Result<u64, ::std::string::String>,
        margin_vertical: ::std::result::Result<u64, ::std::string::String>,
        outline: ::std::result::Result<super::Colour, ::std::string::String>,
        outline_width: ::std::result::Result<u64, ::std::string::String>,
        shadow: ::std::result::Result<super::Colour, ::std::string::String>,
        shadow_depth: ::std::result::Result<u64, ::std::string::String>,
        spoken: ::std::result::Result<super::Colour, ::std::string::String>,
        style_ref: ::std::result::Result<super::ProfileCaptionStyleStyleRef, ::std::string::String>,
        unspoken: ::std::result::Result<super::Colour, ::std::string::String>,
    }
    impl ::std::default::Default for ProfileCaptionStyle {
        fn default() -> Self {
            Self {
                bold: Err("no value supplied for bold".to_string()),
                font_family: Err("no value supplied for font_family".to_string()),
                font_size: Err("no value supplied for font_size".to_string()),
                margin_horizontal: Err("no value supplied for margin_horizontal".to_string()),
                margin_vertical: Err("no value supplied for margin_vertical".to_string()),
                outline: Err("no value supplied for outline".to_string()),
                outline_width: Err("no value supplied for outline_width".to_string()),
                shadow: Err("no value supplied for shadow".to_string()),
                shadow_depth: Err("no value supplied for shadow_depth".to_string()),
                spoken: Err("no value supplied for spoken".to_string()),
                style_ref: Err("no value supplied for style_ref".to_string()),
                unspoken: Err("no value supplied for unspoken".to_string()),
            }
        }
    }
    impl ProfileCaptionStyle {
        pub fn bold<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.bold = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for bold: {e}"));
            self
        }
        pub fn font_family<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfileCaptionStyleFontFamily>,
            T::Error: ::std::fmt::Display,
        {
            self.font_family = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for font_family: {e}"));
            self
        }
        pub fn font_size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.font_size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for font_size: {e}"));
            self
        }
        pub fn margin_horizontal<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.margin_horizontal = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for margin_horizontal: {e}"));
            self
        }
        pub fn margin_vertical<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.margin_vertical = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for margin_vertical: {e}"));
            self
        }
        pub fn outline<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Colour>,
            T::Error: ::std::fmt::Display,
        {
            self.outline = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outline: {e}"));
            self
        }
        pub fn outline_width<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.outline_width = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outline_width: {e}"));
            self
        }
        pub fn shadow<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Colour>,
            T::Error: ::std::fmt::Display,
        {
            self.shadow = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shadow: {e}"));
            self
        }
        pub fn shadow_depth<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.shadow_depth = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shadow_depth: {e}"));
            self
        }
        pub fn spoken<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Colour>,
            T::Error: ::std::fmt::Display,
        {
            self.spoken = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for spoken: {e}"));
            self
        }
        pub fn style_ref<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProfileCaptionStyleStyleRef>,
            T::Error: ::std::fmt::Display,
        {
            self.style_ref = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for style_ref: {e}"));
            self
        }
        pub fn unspoken<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Colour>,
            T::Error: ::std::fmt::Display,
        {
            self.unspoken = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for unspoken: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ProfileCaptionStyle> for super::ProfileCaptionStyle {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ProfileCaptionStyle,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                bold: value.bold?,
                font_family: value.font_family?,
                font_size: value.font_size?,
                margin_horizontal: value.margin_horizontal?,
                margin_vertical: value.margin_vertical?,
                outline: value.outline?,
                outline_width: value.outline_width?,
                shadow: value.shadow?,
                shadow_depth: value.shadow_depth?,
                spoken: value.spoken?,
                style_ref: value.style_ref?,
                unspoken: value.unspoken?,
            })
        }
    }
    impl ::std::convert::From<super::ProfileCaptionStyle> for ProfileCaptionStyle {
        fn from(value: super::ProfileCaptionStyle) -> Self {
            Self {
                bold: Ok(value.bold),
                font_family: Ok(value.font_family),
                font_size: Ok(value.font_size),
                margin_horizontal: Ok(value.margin_horizontal),
                margin_vertical: Ok(value.margin_vertical),
                outline: Ok(value.outline),
                outline_width: Ok(value.outline_width),
                shadow: Ok(value.shadow),
                shadow_depth: Ok(value.shadow_depth),
                spoken: Ok(value.spoken),
                style_ref: Ok(value.style_ref),
                unspoken: Ok(value.unspoken),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ProfileFrameRate {
        den: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        num: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for ProfileFrameRate {
        fn default() -> Self {
            Self {
                den: Err("no value supplied for den".to_string()),
                num: Err("no value supplied for num".to_string()),
            }
        }
    }
    impl ProfileFrameRate {
        pub fn den<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.den = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for den: {e}"));
            self
        }
        pub fn num<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.num = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for num: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ProfileFrameRate> for super::ProfileFrameRate {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ProfileFrameRate,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                den: value.den?,
                num: value.num?,
            })
        }
    }
    impl ::std::convert::From<super::ProfileFrameRate> for ProfileFrameRate {
        fn from(value: super::ProfileFrameRate) -> Self {
            Self {
                den: Ok(value.den),
                num: Ok(value.num),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ProfileLoudness {
        integrated_lufs: ::std::result::Result<f64, ::std::string::String>,
        range_lu: ::std::result::Result<f64, ::std::string::String>,
        true_peak_dbtp: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for ProfileLoudness {
        fn default() -> Self {
            Self {
                integrated_lufs: Err("no value supplied for integrated_lufs".to_string()),
                range_lu: Err("no value supplied for range_lu".to_string()),
                true_peak_dbtp: Err("no value supplied for true_peak_dbtp".to_string()),
            }
        }
    }
    impl ProfileLoudness {
        pub fn integrated_lufs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.integrated_lufs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for integrated_lufs: {e}"));
            self
        }
        pub fn range_lu<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.range_lu = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for range_lu: {e}"));
            self
        }
        pub fn true_peak_dbtp<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.true_peak_dbtp = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for true_peak_dbtp: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ProfileLoudness> for super::ProfileLoudness {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ProfileLoudness,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                integrated_lufs: value.integrated_lufs?,
                range_lu: value.range_lu?,
                true_peak_dbtp: value.true_peak_dbtp?,
            })
        }
    }
    impl ::std::convert::From<super::ProfileLoudness> for ProfileLoudness {
        fn from(value: super::ProfileLoudness) -> Self {
            Self {
                integrated_lufs: Ok(value.integrated_lufs),
                range_lu: Ok(value.range_lu),
                true_peak_dbtp: Ok(value.true_peak_dbtp),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifest {
        ai_use_summary:
            ::std::result::Result<super::RenderClipManifestAiUseSummary, ::std::string::String>,
        caption_windows: ::std::result::Result<
            ::std::vec::Vec<super::RenderClipManifestCaptionWindowsItem>,
            ::std::string::String,
        >,
        determinism:
            ::std::result::Result<super::RenderClipManifestDeterminism, ::std::string::String>,
        engine: ::std::result::Result<super::RenderClipManifestEngine, ::std::string::String>,
        input_source_fingerprints:
            ::std::result::Result<::std::vec::Vec<super::Sha256>, ::std::string::String>,
        ir_artifact_id:
            ::std::result::Result<super::RenderClipManifestIrArtifactId, ::std::string::String>,
        ir_hash: ::std::result::Result<super::Sha256, ::std::string::String>,
        loudness: ::std::result::Result<super::RenderClipManifestLoudness, ::std::string::String>,
        outputs: ::std::result::Result<
            ::std::vec::Vec<super::RenderClipManifestOutputsItem>,
            ::std::string::String,
        >,
        profile: ::std::result::Result<super::Profile, ::std::string::String>,
        program: ::std::result::Result<super::RenderClipManifestProgram, ::std::string::String>,
        rights: ::std::result::Result<super::RenderClipManifestRights, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for RenderClipManifest {
        fn default() -> Self {
            Self {
                ai_use_summary: Err("no value supplied for ai_use_summary".to_string()),
                caption_windows: Err("no value supplied for caption_windows".to_string()),
                determinism: Err("no value supplied for determinism".to_string()),
                engine: Err("no value supplied for engine".to_string()),
                input_source_fingerprints: Err(
                    "no value supplied for input_source_fingerprints".to_string()
                ),
                ir_artifact_id: Err("no value supplied for ir_artifact_id".to_string()),
                ir_hash: Err("no value supplied for ir_hash".to_string()),
                loudness: Err("no value supplied for loudness".to_string()),
                outputs: Err("no value supplied for outputs".to_string()),
                profile: Err("no value supplied for profile".to_string()),
                program: Err("no value supplied for program".to_string()),
                rights: Err("no value supplied for rights".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
            }
        }
    }
    impl RenderClipManifest {
        pub fn ai_use_summary<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestAiUseSummary>,
            T::Error: ::std::fmt::Display,
        {
            self.ai_use_summary = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for ai_use_summary: {e}"));
            self
        }
        pub fn caption_windows<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<super::RenderClipManifestCaptionWindowsItem>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.caption_windows = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for caption_windows: {e}"));
            self
        }
        pub fn determinism<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestDeterminism>,
            T::Error: ::std::fmt::Display,
        {
            self.determinism = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for determinism: {e}"));
            self
        }
        pub fn engine<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestEngine>,
            T::Error: ::std::fmt::Display,
        {
            self.engine = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for engine: {e}"));
            self
        }
        pub fn input_source_fingerprints<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.input_source_fingerprints = value.try_into().map_err(|e| {
                format!("error converting supplied value for input_source_fingerprints: {e}")
            });
            self
        }
        pub fn ir_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestIrArtifactId>,
            T::Error: ::std::fmt::Display,
        {
            self.ir_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for ir_artifact_id: {e}"));
            self
        }
        pub fn ir_hash<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.ir_hash = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for ir_hash: {e}"));
            self
        }
        pub fn loudness<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestLoudness>,
            T::Error: ::std::fmt::Display,
        {
            self.loudness = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for loudness: {e}"));
            self
        }
        pub fn outputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::RenderClipManifestOutputsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.outputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outputs: {e}"));
            self
        }
        pub fn profile<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Profile>,
            T::Error: ::std::fmt::Display,
        {
            self.profile = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for profile: {e}"));
            self
        }
        pub fn program<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestProgram>,
            T::Error: ::std::fmt::Display,
        {
            self.program = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for program: {e}"));
            self
        }
        pub fn rights<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestRights>,
            T::Error: ::std::fmt::Display,
        {
            self.rights = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rights: {e}"));
            self
        }
        pub fn schema_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.schema_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for schema_version: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifest> for super::RenderClipManifest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifest,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                ai_use_summary: value.ai_use_summary?,
                caption_windows: value.caption_windows?,
                determinism: value.determinism?,
                engine: value.engine?,
                input_source_fingerprints: value.input_source_fingerprints?,
                ir_artifact_id: value.ir_artifact_id?,
                ir_hash: value.ir_hash?,
                loudness: value.loudness?,
                outputs: value.outputs?,
                profile: value.profile?,
                program: value.program?,
                rights: value.rights?,
                schema_version: value.schema_version?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifest> for RenderClipManifest {
        fn from(value: super::RenderClipManifest) -> Self {
            Self {
                ai_use_summary: Ok(value.ai_use_summary),
                caption_windows: Ok(value.caption_windows),
                determinism: Ok(value.determinism),
                engine: Ok(value.engine),
                input_source_fingerprints: Ok(value.input_source_fingerprints),
                ir_artifact_id: Ok(value.ir_artifact_id),
                ir_hash: Ok(value.ir_hash),
                loudness: Ok(value.loudness),
                outputs: Ok(value.outputs),
                profile: Ok(value.profile),
                program: Ok(value.program),
                rights: Ok(value.rights),
                schema_version: Ok(value.schema_version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifestAiUseSummary {
        assistance: ::std::result::Result<
            ::std::vec::Vec<super::RenderClipManifestAiUseSummaryAssistanceItem>,
            ::std::string::String,
        >,
        generated: ::std::result::Result<
            ::std::vec::Vec<super::RenderClipManifestAiUseSummaryGeneratedItem>,
            ::std::string::String,
        >,
        requires_youtube_ai_disclosure: ::std::result::Result<bool, ::std::string::String>,
    }
    impl ::std::default::Default for RenderClipManifestAiUseSummary {
        fn default() -> Self {
            Self {
                assistance: Err("no value supplied for assistance".to_string()),
                generated: Err("no value supplied for generated".to_string()),
                requires_youtube_ai_disclosure: Err(
                    "no value supplied for requires_youtube_ai_disclosure".to_string(),
                ),
            }
        }
    }
    impl RenderClipManifestAiUseSummary {
        pub fn assistance<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<super::RenderClipManifestAiUseSummaryAssistanceItem>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.assistance = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for assistance: {e}"));
            self
        }
        pub fn generated<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<super::RenderClipManifestAiUseSummaryGeneratedItem>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.generated = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for generated: {e}"));
            self
        }
        pub fn requires_youtube_ai_disclosure<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.requires_youtube_ai_disclosure = value.try_into().map_err(|e| {
                format!("error converting supplied value for requires_youtube_ai_disclosure: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifestAiUseSummary>
        for super::RenderClipManifestAiUseSummary
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifestAiUseSummary,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                assistance: value.assistance?,
                generated: value.generated?,
                requires_youtube_ai_disclosure: value.requires_youtube_ai_disclosure?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifestAiUseSummary>
        for RenderClipManifestAiUseSummary
    {
        fn from(value: super::RenderClipManifestAiUseSummary) -> Self {
            Self {
                assistance: Ok(value.assistance),
                generated: Ok(value.generated),
                requires_youtube_ai_disclosure: Ok(value.requires_youtube_ai_disclosure),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifestCaptionWindowsItem {
        cue_id: ::std::result::Result<
            super::RenderClipManifestCaptionWindowsItemCueId,
            ::std::string::String,
        >,
        end_frame: ::std::result::Result<u64, ::std::string::String>,
        first_frame: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for RenderClipManifestCaptionWindowsItem {
        fn default() -> Self {
            Self {
                cue_id: Err("no value supplied for cue_id".to_string()),
                end_frame: Err("no value supplied for end_frame".to_string()),
                first_frame: Err("no value supplied for first_frame".to_string()),
            }
        }
    }
    impl RenderClipManifestCaptionWindowsItem {
        pub fn cue_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestCaptionWindowsItemCueId>,
            T::Error: ::std::fmt::Display,
        {
            self.cue_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cue_id: {e}"));
            self
        }
        pub fn end_frame<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_frame = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_frame: {e}"));
            self
        }
        pub fn first_frame<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_frame = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_frame: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifestCaptionWindowsItem>
        for super::RenderClipManifestCaptionWindowsItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifestCaptionWindowsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cue_id: value.cue_id?,
                end_frame: value.end_frame?,
                first_frame: value.first_frame?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifestCaptionWindowsItem>
        for RenderClipManifestCaptionWindowsItem
    {
        fn from(value: super::RenderClipManifestCaptionWindowsItem) -> Self {
            Self {
                cue_id: Ok(value.cue_id),
                end_frame: Ok(value.end_frame),
                first_frame: Ok(value.first_frame),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifestEngine {
        app: ::std::result::Result<super::RenderClipManifestEngineApp, ::std::string::String>,
        ffmpeg: ::std::result::Result<super::RenderClipManifestEngineFfmpeg, ::std::string::String>,
        font_family:
            ::std::result::Result<super::RenderClipManifestEngineFontFamily, ::std::string::String>,
        font_sha256: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for RenderClipManifestEngine {
        fn default() -> Self {
            Self {
                app: Err("no value supplied for app".to_string()),
                ffmpeg: Err("no value supplied for ffmpeg".to_string()),
                font_family: Err("no value supplied for font_family".to_string()),
                font_sha256: Err("no value supplied for font_sha256".to_string()),
            }
        }
    }
    impl RenderClipManifestEngine {
        pub fn app<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestEngineApp>,
            T::Error: ::std::fmt::Display,
        {
            self.app = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for app: {e}"));
            self
        }
        pub fn ffmpeg<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestEngineFfmpeg>,
            T::Error: ::std::fmt::Display,
        {
            self.ffmpeg = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for ffmpeg: {e}"));
            self
        }
        pub fn font_family<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestEngineFontFamily>,
            T::Error: ::std::fmt::Display,
        {
            self.font_family = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for font_family: {e}"));
            self
        }
        pub fn font_sha256<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.font_sha256 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for font_sha256: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifestEngine> for super::RenderClipManifestEngine {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifestEngine,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                app: value.app?,
                ffmpeg: value.ffmpeg?,
                font_family: value.font_family?,
                font_sha256: value.font_sha256?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifestEngine> for RenderClipManifestEngine {
        fn from(value: super::RenderClipManifestEngine) -> Self {
            Self {
                app: Ok(value.app),
                ffmpeg: Ok(value.ffmpeg),
                font_family: Ok(value.font_family),
                font_sha256: Ok(value.font_sha256),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifestLoudness {
        measured_input: ::std::result::Result<super::MeasuredLoudness, ::std::string::String>,
        measured_output: ::std::result::Result<super::MeasuredLoudness, ::std::string::String>,
        target_lufs: ::std::result::Result<f64, ::std::string::String>,
        target_true_peak_dbtp: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for RenderClipManifestLoudness {
        fn default() -> Self {
            Self {
                measured_input: Err("no value supplied for measured_input".to_string()),
                measured_output: Err("no value supplied for measured_output".to_string()),
                target_lufs: Err("no value supplied for target_lufs".to_string()),
                target_true_peak_dbtp: Err(
                    "no value supplied for target_true_peak_dbtp".to_string()
                ),
            }
        }
    }
    impl RenderClipManifestLoudness {
        pub fn measured_input<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MeasuredLoudness>,
            T::Error: ::std::fmt::Display,
        {
            self.measured_input = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for measured_input: {e}"));
            self
        }
        pub fn measured_output<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MeasuredLoudness>,
            T::Error: ::std::fmt::Display,
        {
            self.measured_output = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for measured_output: {e}"));
            self
        }
        pub fn target_lufs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.target_lufs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for target_lufs: {e}"));
            self
        }
        pub fn target_true_peak_dbtp<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.target_true_peak_dbtp = value.try_into().map_err(|e| {
                format!("error converting supplied value for target_true_peak_dbtp: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifestLoudness> for super::RenderClipManifestLoudness {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifestLoudness,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                measured_input: value.measured_input?,
                measured_output: value.measured_output?,
                target_lufs: value.target_lufs?,
                target_true_peak_dbtp: value.target_true_peak_dbtp?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifestLoudness> for RenderClipManifestLoudness {
        fn from(value: super::RenderClipManifestLoudness) -> Self {
            Self {
                measured_input: Ok(value.measured_input),
                measured_output: Ok(value.measured_output),
                target_lufs: Ok(value.target_lufs),
                target_true_peak_dbtp: Ok(value.target_true_peak_dbtp),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifestOutputsItem {
        bytes: ::std::result::Result<u64, ::std::string::String>,
        path:
            ::std::result::Result<super::RenderClipManifestOutputsItemPath, ::std::string::String>,
        sha256: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for RenderClipManifestOutputsItem {
        fn default() -> Self {
            Self {
                bytes: Err("no value supplied for bytes".to_string()),
                path: Err("no value supplied for path".to_string()),
                sha256: Err("no value supplied for sha256".to_string()),
            }
        }
    }
    impl RenderClipManifestOutputsItem {
        pub fn bytes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.bytes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for bytes: {e}"));
            self
        }
        pub fn path<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestOutputsItemPath>,
            T::Error: ::std::fmt::Display,
        {
            self.path = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for path: {e}"));
            self
        }
        pub fn sha256<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.sha256 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sha256: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifestOutputsItem>
        for super::RenderClipManifestOutputsItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifestOutputsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                bytes: value.bytes?,
                path: value.path?,
                sha256: value.sha256?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifestOutputsItem> for RenderClipManifestOutputsItem {
        fn from(value: super::RenderClipManifestOutputsItem) -> Self {
            Self {
                bytes: Ok(value.bytes),
                path: Ok(value.path),
                sha256: Ok(value.sha256),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifestProgram {
        duration_ticks: ::std::result::Result<u64, ::std::string::String>,
        frame_count: ::std::result::Result<u64, ::std::string::String>,
        segments: ::std::result::Result<
            ::std::vec::Vec<super::RenderClipManifestProgramSegmentsItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for RenderClipManifestProgram {
        fn default() -> Self {
            Self {
                duration_ticks: Err("no value supplied for duration_ticks".to_string()),
                frame_count: Err("no value supplied for frame_count".to_string()),
                segments: Err("no value supplied for segments".to_string()),
            }
        }
    }
    impl RenderClipManifestProgram {
        pub fn duration_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.duration_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for duration_ticks: {e}"));
            self
        }
        pub fn frame_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.frame_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frame_count: {e}"));
            self
        }
        pub fn segments<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<super::RenderClipManifestProgramSegmentsItem>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.segments = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segments: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifestProgram> for super::RenderClipManifestProgram {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifestProgram,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                duration_ticks: value.duration_ticks?,
                frame_count: value.frame_count?,
                segments: value.segments?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifestProgram> for RenderClipManifestProgram {
        fn from(value: super::RenderClipManifestProgram) -> Self {
            Self {
                duration_ticks: Ok(value.duration_ticks),
                frame_count: Ok(value.frame_count),
                segments: Ok(value.segments),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifestProgramSegmentsItem {
        frame_count: ::std::result::Result<u64, ::std::string::String>,
        in_ticks: ::std::result::Result<u64, ::std::string::String>,
        layout: ::std::result::Result<
            super::RenderClipManifestProgramSegmentsItemLayout,
            ::std::string::String,
        >,
        out_ticks: ::std::result::Result<u64, ::std::string::String>,
        segment_id: ::std::result::Result<
            super::RenderClipManifestProgramSegmentsItemSegmentId,
            ::std::string::String,
        >,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for RenderClipManifestProgramSegmentsItem {
        fn default() -> Self {
            Self {
                frame_count: Err("no value supplied for frame_count".to_string()),
                in_ticks: Err("no value supplied for in_ticks".to_string()),
                layout: Err("no value supplied for layout".to_string()),
                out_ticks: Err("no value supplied for out_ticks".to_string()),
                segment_id: Err("no value supplied for segment_id".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl RenderClipManifestProgramSegmentsItem {
        pub fn frame_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.frame_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frame_count: {e}"));
            self
        }
        pub fn in_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.in_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for in_ticks: {e}"));
            self
        }
        pub fn layout<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestProgramSegmentsItemLayout>,
            T::Error: ::std::fmt::Display,
        {
            self.layout = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for layout: {e}"));
            self
        }
        pub fn out_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.out_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for out_ticks: {e}"));
            self
        }
        pub fn segment_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestProgramSegmentsItemSegmentId>,
            T::Error: ::std::fmt::Display,
        {
            self.segment_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segment_id: {e}"));
            self
        }
        pub fn source_fingerprint<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.source_fingerprint = value.try_into().map_err(|e| {
                format!("error converting supplied value for source_fingerprint: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifestProgramSegmentsItem>
        for super::RenderClipManifestProgramSegmentsItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifestProgramSegmentsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                frame_count: value.frame_count?,
                in_ticks: value.in_ticks?,
                layout: value.layout?,
                out_ticks: value.out_ticks?,
                segment_id: value.segment_id?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifestProgramSegmentsItem>
        for RenderClipManifestProgramSegmentsItem
    {
        fn from(value: super::RenderClipManifestProgramSegmentsItem) -> Self {
            Self {
                frame_count: Ok(value.frame_count),
                in_ticks: Ok(value.in_ticks),
                layout: Ok(value.layout),
                out_ticks: Ok(value.out_ticks),
                segment_id: Ok(value.segment_id),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RenderClipManifestRights {
        gates_passed: ::std::result::Result<
            ::std::vec::Vec<super::RenderClipManifestRightsGatesPassedItem>,
            ::std::string::String,
        >,
        source_attestation: ::std::result::Result<
            super::RenderClipManifestRightsSourceAttestation,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for RenderClipManifestRights {
        fn default() -> Self {
            Self {
                gates_passed: Err("no value supplied for gates_passed".to_string()),
                source_attestation: Err("no value supplied for source_attestation".to_string()),
            }
        }
    }
    impl RenderClipManifestRights {
        pub fn gates_passed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<super::RenderClipManifestRightsGatesPassedItem>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.gates_passed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for gates_passed: {e}"));
            self
        }
        pub fn source_attestation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RenderClipManifestRightsSourceAttestation>,
            T::Error: ::std::fmt::Display,
        {
            self.source_attestation = value.try_into().map_err(|e| {
                format!("error converting supplied value for source_attestation: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<RenderClipManifestRights> for super::RenderClipManifestRights {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RenderClipManifestRights,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                gates_passed: value.gates_passed?,
                source_attestation: value.source_attestation?,
            })
        }
    }
    impl ::std::convert::From<super::RenderClipManifestRights> for RenderClipManifestRights {
        fn from(value: super::RenderClipManifestRights) -> Self {
            Self {
                gates_passed: Ok(value.gates_passed),
                source_attestation: Ok(value.source_attestation),
            }
        }
    }
}
