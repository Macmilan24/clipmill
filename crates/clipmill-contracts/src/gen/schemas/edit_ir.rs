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
#[doc = "`CaptionCue`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"anim\","]
#[doc = "    \"cue_id\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"lines\","]
#[doc = "    \"region\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"anim\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"none\","]
#[doc = "        \"karaoke\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"cue_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"lines\": {"]
#[doc = "      \"description\": \"Line breaks are decided once and stored here — the parity keystone. Preview and render must never re-wrap text independently.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"words\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"words\": {"]
#[doc = "            \"type\": \"array\","]
#[doc = "            \"items\": {"]
#[doc = "              \"type\": \"object\","]
#[doc = "              \"required\": ["]
#[doc = "                \"end_ticks\","]
#[doc = "                \"start_ticks\","]
#[doc = "                \"text\""]
#[doc = "              ],"]
#[doc = "              \"properties\": {"]
#[doc = "                \"end_ticks\": {"]
#[doc = "                  \"type\": \"integer\","]
#[doc = "                  \"minimum\": 1.0"]
#[doc = "                },"]
#[doc = "                \"start_ticks\": {"]
#[doc = "                  \"type\": \"integer\","]
#[doc = "                  \"minimum\": 0.0"]
#[doc = "                },"]
#[doc = "                \"text\": {"]
#[doc = "                  \"type\": \"string\","]
#[doc = "                  \"minLength\": 1"]
#[doc = "                }"]
#[doc = "              },"]
#[doc = "              \"additionalProperties\": false"]
#[doc = "            },"]
#[doc = "            \"minItems\": 1"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"region\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"lower_safe\","]
#[doc = "        \"upper_safe\","]
#[doc = "        \"center\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
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
pub struct CaptionCue {
    pub anim: CaptionCueAnim,
    pub cue_id: CaptionCueCueId,
    pub end_ticks: ::std::num::NonZeroU64,
    #[doc = "Line breaks are decided once and stored here — the parity keystone. Preview and render must never re-wrap text independently."]
    pub lines: ::std::vec::Vec<CaptionCueLinesItem>,
    pub region: CaptionCueRegion,
    pub start_ticks: u64,
}
impl CaptionCue {
    pub fn builder() -> builder::CaptionCue {
        Default::default()
    }
}
#[doc = "`CaptionCueAnim`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"none\","]
#[doc = "    \"karaoke\""]
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
pub enum CaptionCueAnim {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "karaoke")]
    Karaoke,
}
impl ::std::fmt::Display for CaptionCueAnim {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::None => f.write_str("none"),
            Self::Karaoke => f.write_str("karaoke"),
        }
    }
}
impl ::std::str::FromStr for CaptionCueAnim {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "none" => Ok(Self::None),
            "karaoke" => Ok(Self::Karaoke),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CaptionCueAnim {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCueAnim {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCueAnim {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CaptionCueCueId`"]
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
pub struct CaptionCueCueId(::std::string::String);
impl ::std::ops::Deref for CaptionCueCueId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CaptionCueCueId> for ::std::string::String {
    fn from(value: CaptionCueCueId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CaptionCueCueId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CaptionCueCueId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCueCueId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCueCueId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CaptionCueCueId {
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
#[doc = "`CaptionCueLinesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"words\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"words\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"end_ticks\","]
#[doc = "          \"start_ticks\","]
#[doc = "          \"text\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"end_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 1.0"]
#[doc = "          },"]
#[doc = "          \"start_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"text\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CaptionCueLinesItem {
    pub words: ::std::vec::Vec<CaptionCueLinesItemWordsItem>,
}
impl CaptionCueLinesItem {
    pub fn builder() -> builder::CaptionCueLinesItem {
        Default::default()
    }
}
#[doc = "`CaptionCueLinesItemWordsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"text\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
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
pub struct CaptionCueLinesItemWordsItem {
    pub end_ticks: ::std::num::NonZeroU64,
    pub start_ticks: u64,
    pub text: CaptionCueLinesItemWordsItemText,
}
impl CaptionCueLinesItemWordsItem {
    pub fn builder() -> builder::CaptionCueLinesItemWordsItem {
        Default::default()
    }
}
#[doc = "`CaptionCueLinesItemWordsItemText`"]
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
pub struct CaptionCueLinesItemWordsItemText(::std::string::String);
impl ::std::ops::Deref for CaptionCueLinesItemWordsItemText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CaptionCueLinesItemWordsItemText> for ::std::string::String {
    fn from(value: CaptionCueLinesItemWordsItemText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CaptionCueLinesItemWordsItemText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CaptionCueLinesItemWordsItemText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCueLinesItemWordsItemText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCueLinesItemWordsItemText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CaptionCueLinesItemWordsItemText {
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
#[doc = "`CaptionCueRegion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"lower_safe\","]
#[doc = "    \"upper_safe\","]
#[doc = "    \"center\""]
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
pub enum CaptionCueRegion {
    #[serde(rename = "lower_safe")]
    LowerSafe,
    #[serde(rename = "upper_safe")]
    UpperSafe,
    #[serde(rename = "center")]
    Center,
}
impl ::std::fmt::Display for CaptionCueRegion {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::LowerSafe => f.write_str("lower_safe"),
            Self::UpperSafe => f.write_str("upper_safe"),
            Self::Center => f.write_str("center"),
        }
    }
}
impl ::std::str::FromStr for CaptionCueRegion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "lower_safe" => Ok(Self::LowerSafe),
            "upper_safe" => Ok(Self::UpperSafe),
            "center" => Ok(Self::Center),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CaptionCueRegion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCueRegion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCueRegion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Integer pixel rectangle in the source frame's coordinate space."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Integer pixel rectangle in the source frame's coordinate space.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"height\","]
#[doc = "    \"width\","]
#[doc = "    \"x\","]
#[doc = "    \"y\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"height\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"width\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"x\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"y\": {"]
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
pub struct CropRect {
    pub height: ::std::num::NonZeroU64,
    pub width: ::std::num::NonZeroU64,
    pub x: u64,
    pub y: u64,
}
impl CropRect {
    pub fn builder() -> builder::CropRect {
        Default::default()
    }
}
#[doc = "The edit document (book ch. 17): a versioned, multi-track, non-destructive timeline that the preview, the render compiler, and later the NLE exporter all read. No subsystem may render, preview, or export from any other representation. All time is integer ticks at 1/90000 (D06); a segment's program position is the sum of the durations before it and is never stored, so a trim cannot leave a stale offset behind."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.edit_ir.v1.json\","]
#[doc = "  \"title\": \"EditIr\","]
#[doc = "  \"description\": \"The edit document (book ch. 17): a versioned, multi-track, non-destructive timeline that the preview, the render compiler, and later the NLE exporter all read. No subsystem may render, preview, or export from any other representation. All time is integer ticks at 1/90000 (D06); a segment's program position is the sum of the durations before it and is never stored, so a trim cannot leave a stale offset behind.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"audio\","]
#[doc = "    \"captions\","]
#[doc = "    \"timebase\","]
#[doc = "    \"version\","]
#[doc = "    \"video\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"assets\": {"]
#[doc = "      \"description\": \"Assets referenced by content hash, each carrying the licence record the render manifest echoes.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"hash\","]
#[doc = "          \"license\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"hash\": {"]
#[doc = "            \"$ref\": \"#/$defs/sha256\""]
#[doc = "          },"]
#[doc = "          \"license\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"audio\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"target_lufs\","]
#[doc = "        \"true_peak_dbtp\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"gain_curve\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"gain_db\","]
#[doc = "              \"t_ticks\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"gain_db\": {"]
#[doc = "                \"type\": \"number\""]
#[doc = "              },"]
#[doc = "              \"t_ticks\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"target_lufs\": {"]
#[doc = "          \"description\": \"Loudness target in LUFS. Loudness is a measurement, not a time, so it is legitimately real-valued.\","]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"true_peak_dbtp\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"captions\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"style_ref\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"cues\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/captionCue\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"style_ref\": {"]
#[doc = "          \"description\": \"Named caption preset; the style itself lives with the presets, not in every document.\","]
#[doc = "          \"type\": \"string\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"rationale\": {"]
#[doc = "      \"description\": \"Why the director cut here. Never consumed by any render path, so explanation can never perturb pixels.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"properties\": {"]
#[doc = "        \"candidate_id\": {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        \"decisions\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          }"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"timebase\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"den\","]
#[doc = "        \"num\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"den\": {"]
#[doc = "          \"const\": 90000"]
#[doc = "        },"]
#[doc = "        \"num\": {"]
#[doc = "          \"const\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"version\": {"]
#[doc = "      \"const\": \"ir/1\""]
#[doc = "    },"]
#[doc = "    \"video\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"properties\": {"]
#[doc = "        \"segments\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/videoSegment\""]
#[doc = "          }"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditIr {
    #[doc = "Assets referenced by content hash, each carrying the licence record the render manifest echoes."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub assets: ::std::vec::Vec<EditIrAssetsItem>,
    pub audio: EditIrAudio,
    pub captions: EditIrCaptions,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub rationale: ::std::option::Option<EditIrRationale>,
    pub timebase: EditIrTimebase,
    pub version: ::serde_json::Value,
    pub video: EditIrVideo,
}
impl EditIr {
    pub fn builder() -> builder::EditIr {
        Default::default()
    }
}
#[doc = "`EditIrAssetsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"hash\","]
#[doc = "    \"license\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"hash\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"license\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditIrAssetsItem {
    pub hash: Sha256,
    pub license: ::std::string::String,
}
impl EditIrAssetsItem {
    pub fn builder() -> builder::EditIrAssetsItem {
        Default::default()
    }
}
#[doc = "`EditIrAudio`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"target_lufs\","]
#[doc = "    \"true_peak_dbtp\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"gain_curve\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"gain_db\","]
#[doc = "          \"t_ticks\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"gain_db\": {"]
#[doc = "            \"type\": \"number\""]
#[doc = "          },"]
#[doc = "          \"t_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"target_lufs\": {"]
#[doc = "      \"description\": \"Loudness target in LUFS. Loudness is a measurement, not a time, so it is legitimately real-valued.\","]
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
pub struct EditIrAudio {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub gain_curve: ::std::vec::Vec<EditIrAudioGainCurveItem>,
    #[doc = "Loudness target in LUFS. Loudness is a measurement, not a time, so it is legitimately real-valued."]
    pub target_lufs: f64,
    pub true_peak_dbtp: f64,
}
impl EditIrAudio {
    pub fn builder() -> builder::EditIrAudio {
        Default::default()
    }
}
#[doc = "`EditIrAudioGainCurveItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"gain_db\","]
#[doc = "    \"t_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"gain_db\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"t_ticks\": {"]
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
pub struct EditIrAudioGainCurveItem {
    pub gain_db: f64,
    pub t_ticks: u64,
}
impl EditIrAudioGainCurveItem {
    pub fn builder() -> builder::EditIrAudioGainCurveItem {
        Default::default()
    }
}
#[doc = "`EditIrCaptions`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"style_ref\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cues\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/captionCue\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"style_ref\": {"]
#[doc = "      \"description\": \"Named caption preset; the style itself lives with the presets, not in every document.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditIrCaptions {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub cues: ::std::vec::Vec<CaptionCue>,
    #[doc = "Named caption preset; the style itself lives with the presets, not in every document."]
    pub style_ref: ::std::string::String,
}
impl EditIrCaptions {
    pub fn builder() -> builder::EditIrCaptions {
        Default::default()
    }
}
#[doc = "Why the director cut here. Never consumed by any render path, so explanation can never perturb pixels."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Why the director cut here. Never consumed by any render path, so explanation can never perturb pixels.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidate_id\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"decisions\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditIrRationale {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub candidate_id: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub decisions: ::std::vec::Vec<::std::string::String>,
}
impl ::std::default::Default for EditIrRationale {
    fn default() -> Self {
        Self {
            candidate_id: Default::default(),
            decisions: Default::default(),
        }
    }
}
impl EditIrRationale {
    pub fn builder() -> builder::EditIrRationale {
        Default::default()
    }
}
#[doc = "`EditIrTimebase`"]
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
#[doc = "      \"const\": 90000"]
#[doc = "    },"]
#[doc = "    \"num\": {"]
#[doc = "      \"const\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditIrTimebase {
    pub den: ::serde_json::Value,
    pub num: ::serde_json::Value,
}
impl EditIrTimebase {
    pub fn builder() -> builder::EditIrTimebase {
        Default::default()
    }
}
#[doc = "`EditIrVideo`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"segments\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/videoSegment\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditIrVideo {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub segments: ::std::vec::Vec<VideoSegment>,
}
impl ::std::default::Default for EditIrVideo {
    fn default() -> Self {
        Self {
            segments: Default::default(),
        }
    }
}
impl EditIrVideo {
    pub fn builder() -> builder::EditIrVideo {
        Default::default()
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
#[doc = "`VideoSegment`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"in_ticks\","]
#[doc = "    \"layout\","]
#[doc = "    \"out_ticks\","]
#[doc = "    \"segment_id\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"in_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"layout\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"state\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"crop_path\": {"]
#[doc = "          \"description\": \"Crop keyframes in segment-local ticks, so trimming the source window cannot silently re-time the camera move.\","]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"rect\","]
#[doc = "              \"t_ticks\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"rect\": {"]
#[doc = "                \"$ref\": \"#/$defs/cropRect\""]
#[doc = "              },"]
#[doc = "              \"t_ticks\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"state\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"speaker_fill\","]
#[doc = "            \"fit\""]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"out_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
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
pub struct VideoSegment {
    pub in_ticks: u64,
    pub layout: VideoSegmentLayout,
    pub out_ticks: ::std::num::NonZeroU64,
    pub segment_id: VideoSegmentSegmentId,
    pub source_fingerprint: Sha256,
}
impl VideoSegment {
    pub fn builder() -> builder::VideoSegment {
        Default::default()
    }
}
#[doc = "`VideoSegmentLayout`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"state\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"crop_path\": {"]
#[doc = "      \"description\": \"Crop keyframes in segment-local ticks, so trimming the source window cannot silently re-time the camera move.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"rect\","]
#[doc = "          \"t_ticks\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"rect\": {"]
#[doc = "            \"$ref\": \"#/$defs/cropRect\""]
#[doc = "          },"]
#[doc = "          \"t_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"state\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"speaker_fill\","]
#[doc = "        \"fit\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VideoSegmentLayout {
    #[doc = "Crop keyframes in segment-local ticks, so trimming the source window cannot silently re-time the camera move."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub crop_path: ::std::vec::Vec<VideoSegmentLayoutCropPathItem>,
    pub state: VideoSegmentLayoutState,
}
impl VideoSegmentLayout {
    pub fn builder() -> builder::VideoSegmentLayout {
        Default::default()
    }
}
#[doc = "`VideoSegmentLayoutCropPathItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"rect\","]
#[doc = "    \"t_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"rect\": {"]
#[doc = "      \"$ref\": \"#/$defs/cropRect\""]
#[doc = "    },"]
#[doc = "    \"t_ticks\": {"]
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
pub struct VideoSegmentLayoutCropPathItem {
    pub rect: CropRect,
    pub t_ticks: u64,
}
impl VideoSegmentLayoutCropPathItem {
    pub fn builder() -> builder::VideoSegmentLayoutCropPathItem {
        Default::default()
    }
}
#[doc = "`VideoSegmentLayoutState`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"speaker_fill\","]
#[doc = "    \"fit\""]
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
pub enum VideoSegmentLayoutState {
    #[serde(rename = "speaker_fill")]
    SpeakerFill,
    #[serde(rename = "fit")]
    Fit,
}
impl ::std::fmt::Display for VideoSegmentLayoutState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::SpeakerFill => f.write_str("speaker_fill"),
            Self::Fit => f.write_str("fit"),
        }
    }
}
impl ::std::str::FromStr for VideoSegmentLayoutState {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "speaker_fill" => Ok(Self::SpeakerFill),
            "fit" => Ok(Self::Fit),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for VideoSegmentLayoutState {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for VideoSegmentLayoutState {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for VideoSegmentLayoutState {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`VideoSegmentSegmentId`"]
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
pub struct VideoSegmentSegmentId(::std::string::String);
impl ::std::ops::Deref for VideoSegmentSegmentId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<VideoSegmentSegmentId> for ::std::string::String {
    fn from(value: VideoSegmentSegmentId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for VideoSegmentSegmentId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for VideoSegmentSegmentId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for VideoSegmentSegmentId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for VideoSegmentSegmentId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for VideoSegmentSegmentId {
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
    pub struct CaptionCue {
        anim: ::std::result::Result<super::CaptionCueAnim, ::std::string::String>,
        cue_id: ::std::result::Result<super::CaptionCueCueId, ::std::string::String>,
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        lines: ::std::result::Result<
            ::std::vec::Vec<super::CaptionCueLinesItem>,
            ::std::string::String,
        >,
        region: ::std::result::Result<super::CaptionCueRegion, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for CaptionCue {
        fn default() -> Self {
            Self {
                anim: Err("no value supplied for anim".to_string()),
                cue_id: Err("no value supplied for cue_id".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                lines: Err("no value supplied for lines".to_string()),
                region: Err("no value supplied for region".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl CaptionCue {
        pub fn anim<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCueAnim>,
            T::Error: ::std::fmt::Display,
        {
            self.anim = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for anim: {e}"));
            self
        }
        pub fn cue_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCueCueId>,
            T::Error: ::std::fmt::Display,
        {
            self.cue_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cue_id: {e}"));
            self
        }
        pub fn end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_ticks: {e}"));
            self
        }
        pub fn lines<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CaptionCueLinesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.lines = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for lines: {e}"));
            self
        }
        pub fn region<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCueRegion>,
            T::Error: ::std::fmt::Display,
        {
            self.region = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for region: {e}"));
            self
        }
        pub fn start_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.start_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for start_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionCue> for super::CaptionCue {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCue,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                anim: value.anim?,
                cue_id: value.cue_id?,
                end_ticks: value.end_ticks?,
                lines: value.lines?,
                region: value.region?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCue> for CaptionCue {
        fn from(value: super::CaptionCue) -> Self {
            Self {
                anim: Ok(value.anim),
                cue_id: Ok(value.cue_id),
                end_ticks: Ok(value.end_ticks),
                lines: Ok(value.lines),
                region: Ok(value.region),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CaptionCueLinesItem {
        words: ::std::result::Result<
            ::std::vec::Vec<super::CaptionCueLinesItemWordsItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for CaptionCueLinesItem {
        fn default() -> Self {
            Self {
                words: Err("no value supplied for words".to_string()),
            }
        }
    }
    impl CaptionCueLinesItem {
        pub fn words<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CaptionCueLinesItemWordsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.words = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for words: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionCueLinesItem> for super::CaptionCueLinesItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCueLinesItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                words: value.words?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCueLinesItem> for CaptionCueLinesItem {
        fn from(value: super::CaptionCueLinesItem) -> Self {
            Self {
                words: Ok(value.words),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CaptionCueLinesItemWordsItem {
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        text: ::std::result::Result<super::CaptionCueLinesItemWordsItemText, ::std::string::String>,
    }
    impl ::std::default::Default for CaptionCueLinesItemWordsItem {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                text: Err("no value supplied for text".to_string()),
            }
        }
    }
    impl CaptionCueLinesItemWordsItem {
        pub fn end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_ticks: {e}"));
            self
        }
        pub fn start_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.start_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for start_ticks: {e}"));
            self
        }
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCueLinesItemWordsItemText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionCueLinesItemWordsItem> for super::CaptionCueLinesItemWordsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCueLinesItemWordsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
                text: value.text?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCueLinesItemWordsItem> for CaptionCueLinesItemWordsItem {
        fn from(value: super::CaptionCueLinesItemWordsItem) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
                text: Ok(value.text),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CropRect {
        height: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        width: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        x: ::std::result::Result<u64, ::std::string::String>,
        y: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for CropRect {
        fn default() -> Self {
            Self {
                height: Err("no value supplied for height".to_string()),
                width: Err("no value supplied for width".to_string()),
                x: Err("no value supplied for x".to_string()),
                y: Err("no value supplied for y".to_string()),
            }
        }
    }
    impl CropRect {
        pub fn height<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.height = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for height: {e}"));
            self
        }
        pub fn width<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.width = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for width: {e}"));
            self
        }
        pub fn x<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.x = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for x: {e}"));
            self
        }
        pub fn y<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.y = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for y: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CropRect> for super::CropRect {
        type Error = super::error::ConversionError;
        fn try_from(value: CropRect) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                height: value.height?,
                width: value.width?,
                x: value.x?,
                y: value.y?,
            })
        }
    }
    impl ::std::convert::From<super::CropRect> for CropRect {
        fn from(value: super::CropRect) -> Self {
            Self {
                height: Ok(value.height),
                width: Ok(value.width),
                x: Ok(value.x),
                y: Ok(value.y),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIr {
        assets:
            ::std::result::Result<::std::vec::Vec<super::EditIrAssetsItem>, ::std::string::String>,
        audio: ::std::result::Result<super::EditIrAudio, ::std::string::String>,
        captions: ::std::result::Result<super::EditIrCaptions, ::std::string::String>,
        rationale: ::std::result::Result<
            ::std::option::Option<super::EditIrRationale>,
            ::std::string::String,
        >,
        timebase: ::std::result::Result<super::EditIrTimebase, ::std::string::String>,
        version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        video: ::std::result::Result<super::EditIrVideo, ::std::string::String>,
    }
    impl ::std::default::Default for EditIr {
        fn default() -> Self {
            Self {
                assets: Ok(Default::default()),
                audio: Err("no value supplied for audio".to_string()),
                captions: Err("no value supplied for captions".to_string()),
                rationale: Ok(Default::default()),
                timebase: Err("no value supplied for timebase".to_string()),
                version: Err("no value supplied for version".to_string()),
                video: Err("no value supplied for video".to_string()),
            }
        }
    }
    impl EditIr {
        pub fn assets<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::EditIrAssetsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.assets = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for assets: {e}"));
            self
        }
        pub fn audio<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditIrAudio>,
            T::Error: ::std::fmt::Display,
        {
            self.audio = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio: {e}"));
            self
        }
        pub fn captions<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditIrCaptions>,
            T::Error: ::std::fmt::Display,
        {
            self.captions = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for captions: {e}"));
            self
        }
        pub fn rationale<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrRationale>>,
            T::Error: ::std::fmt::Display,
        {
            self.rationale = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rationale: {e}"));
            self
        }
        pub fn timebase<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditIrTimebase>,
            T::Error: ::std::fmt::Display,
        {
            self.timebase = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timebase: {e}"));
            self
        }
        pub fn version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for version: {e}"));
            self
        }
        pub fn video<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditIrVideo>,
            T::Error: ::std::fmt::Display,
        {
            self.video = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for video: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditIr> for super::EditIr {
        type Error = super::error::ConversionError;
        fn try_from(value: EditIr) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                assets: value.assets?,
                audio: value.audio?,
                captions: value.captions?,
                rationale: value.rationale?,
                timebase: value.timebase?,
                version: value.version?,
                video: value.video?,
            })
        }
    }
    impl ::std::convert::From<super::EditIr> for EditIr {
        fn from(value: super::EditIr) -> Self {
            Self {
                assets: Ok(value.assets),
                audio: Ok(value.audio),
                captions: Ok(value.captions),
                rationale: Ok(value.rationale),
                timebase: Ok(value.timebase),
                version: Ok(value.version),
                video: Ok(value.video),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIrAssetsItem {
        hash: ::std::result::Result<super::Sha256, ::std::string::String>,
        license: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrAssetsItem {
        fn default() -> Self {
            Self {
                hash: Err("no value supplied for hash".to_string()),
                license: Err("no value supplied for license".to_string()),
            }
        }
    }
    impl EditIrAssetsItem {
        pub fn hash<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.hash = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hash: {e}"));
            self
        }
        pub fn license<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.license = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for license: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditIrAssetsItem> for super::EditIrAssetsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditIrAssetsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                hash: value.hash?,
                license: value.license?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrAssetsItem> for EditIrAssetsItem {
        fn from(value: super::EditIrAssetsItem) -> Self {
            Self {
                hash: Ok(value.hash),
                license: Ok(value.license),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIrAudio {
        gain_curve: ::std::result::Result<
            ::std::vec::Vec<super::EditIrAudioGainCurveItem>,
            ::std::string::String,
        >,
        target_lufs: ::std::result::Result<f64, ::std::string::String>,
        true_peak_dbtp: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrAudio {
        fn default() -> Self {
            Self {
                gain_curve: Ok(Default::default()),
                target_lufs: Err("no value supplied for target_lufs".to_string()),
                true_peak_dbtp: Err("no value supplied for true_peak_dbtp".to_string()),
            }
        }
    }
    impl EditIrAudio {
        pub fn gain_curve<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::EditIrAudioGainCurveItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.gain_curve = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for gain_curve: {e}"));
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
    impl ::std::convert::TryFrom<EditIrAudio> for super::EditIrAudio {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditIrAudio,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                gain_curve: value.gain_curve?,
                target_lufs: value.target_lufs?,
                true_peak_dbtp: value.true_peak_dbtp?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrAudio> for EditIrAudio {
        fn from(value: super::EditIrAudio) -> Self {
            Self {
                gain_curve: Ok(value.gain_curve),
                target_lufs: Ok(value.target_lufs),
                true_peak_dbtp: Ok(value.true_peak_dbtp),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIrAudioGainCurveItem {
        gain_db: ::std::result::Result<f64, ::std::string::String>,
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrAudioGainCurveItem {
        fn default() -> Self {
            Self {
                gain_db: Err("no value supplied for gain_db".to_string()),
                t_ticks: Err("no value supplied for t_ticks".to_string()),
            }
        }
    }
    impl EditIrAudioGainCurveItem {
        pub fn gain_db<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.gain_db = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for gain_db: {e}"));
            self
        }
        pub fn t_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.t_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for t_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditIrAudioGainCurveItem> for super::EditIrAudioGainCurveItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditIrAudioGainCurveItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                gain_db: value.gain_db?,
                t_ticks: value.t_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrAudioGainCurveItem> for EditIrAudioGainCurveItem {
        fn from(value: super::EditIrAudioGainCurveItem) -> Self {
            Self {
                gain_db: Ok(value.gain_db),
                t_ticks: Ok(value.t_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIrCaptions {
        cues: ::std::result::Result<::std::vec::Vec<super::CaptionCue>, ::std::string::String>,
        style_ref: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrCaptions {
        fn default() -> Self {
            Self {
                cues: Ok(Default::default()),
                style_ref: Err("no value supplied for style_ref".to_string()),
            }
        }
    }
    impl EditIrCaptions {
        pub fn cues<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CaptionCue>>,
            T::Error: ::std::fmt::Display,
        {
            self.cues = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cues: {e}"));
            self
        }
        pub fn style_ref<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.style_ref = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for style_ref: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditIrCaptions> for super::EditIrCaptions {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditIrCaptions,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cues: value.cues?,
                style_ref: value.style_ref?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrCaptions> for EditIrCaptions {
        fn from(value: super::EditIrCaptions) -> Self {
            Self {
                cues: Ok(value.cues),
                style_ref: Ok(value.style_ref),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIrRationale {
        candidate_id: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        decisions:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrRationale {
        fn default() -> Self {
            Self {
                candidate_id: Ok(Default::default()),
                decisions: Ok(Default::default()),
            }
        }
    }
    impl EditIrRationale {
        pub fn candidate_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.candidate_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidate_id: {e}"));
            self
        }
        pub fn decisions<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.decisions = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for decisions: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditIrRationale> for super::EditIrRationale {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditIrRationale,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidate_id: value.candidate_id?,
                decisions: value.decisions?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrRationale> for EditIrRationale {
        fn from(value: super::EditIrRationale) -> Self {
            Self {
                candidate_id: Ok(value.candidate_id),
                decisions: Ok(value.decisions),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIrTimebase {
        den: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        num: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrTimebase {
        fn default() -> Self {
            Self {
                den: Err("no value supplied for den".to_string()),
                num: Err("no value supplied for num".to_string()),
            }
        }
    }
    impl EditIrTimebase {
        pub fn den<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.den = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for den: {e}"));
            self
        }
        pub fn num<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.num = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for num: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditIrTimebase> for super::EditIrTimebase {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditIrTimebase,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                den: value.den?,
                num: value.num?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrTimebase> for EditIrTimebase {
        fn from(value: super::EditIrTimebase) -> Self {
            Self {
                den: Ok(value.den),
                num: Ok(value.num),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIrVideo {
        segments:
            ::std::result::Result<::std::vec::Vec<super::VideoSegment>, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrVideo {
        fn default() -> Self {
            Self {
                segments: Ok(Default::default()),
            }
        }
    }
    impl EditIrVideo {
        pub fn segments<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::VideoSegment>>,
            T::Error: ::std::fmt::Display,
        {
            self.segments = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segments: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditIrVideo> for super::EditIrVideo {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditIrVideo,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                segments: value.segments?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrVideo> for EditIrVideo {
        fn from(value: super::EditIrVideo) -> Self {
            Self {
                segments: Ok(value.segments),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VideoSegment {
        in_ticks: ::std::result::Result<u64, ::std::string::String>,
        layout: ::std::result::Result<super::VideoSegmentLayout, ::std::string::String>,
        out_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        segment_id: ::std::result::Result<super::VideoSegmentSegmentId, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSegment {
        fn default() -> Self {
            Self {
                in_ticks: Err("no value supplied for in_ticks".to_string()),
                layout: Err("no value supplied for layout".to_string()),
                out_ticks: Err("no value supplied for out_ticks".to_string()),
                segment_id: Err("no value supplied for segment_id".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl VideoSegment {
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
            T: ::std::convert::TryInto<super::VideoSegmentLayout>,
            T::Error: ::std::fmt::Display,
        {
            self.layout = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for layout: {e}"));
            self
        }
        pub fn out_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.out_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for out_ticks: {e}"));
            self
        }
        pub fn segment_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::VideoSegmentSegmentId>,
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
    impl ::std::convert::TryFrom<VideoSegment> for super::VideoSegment {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VideoSegment,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                in_ticks: value.in_ticks?,
                layout: value.layout?,
                out_ticks: value.out_ticks?,
                segment_id: value.segment_id?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::VideoSegment> for VideoSegment {
        fn from(value: super::VideoSegment) -> Self {
            Self {
                in_ticks: Ok(value.in_ticks),
                layout: Ok(value.layout),
                out_ticks: Ok(value.out_ticks),
                segment_id: Ok(value.segment_id),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VideoSegmentLayout {
        crop_path: ::std::result::Result<
            ::std::vec::Vec<super::VideoSegmentLayoutCropPathItem>,
            ::std::string::String,
        >,
        state: ::std::result::Result<super::VideoSegmentLayoutState, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSegmentLayout {
        fn default() -> Self {
            Self {
                crop_path: Ok(Default::default()),
                state: Err("no value supplied for state".to_string()),
            }
        }
    }
    impl VideoSegmentLayout {
        pub fn crop_path<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::VideoSegmentLayoutCropPathItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.crop_path = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for crop_path: {e}"));
            self
        }
        pub fn state<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::VideoSegmentLayoutState>,
            T::Error: ::std::fmt::Display,
        {
            self.state = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for state: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<VideoSegmentLayout> for super::VideoSegmentLayout {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VideoSegmentLayout,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                crop_path: value.crop_path?,
                state: value.state?,
            })
        }
    }
    impl ::std::convert::From<super::VideoSegmentLayout> for VideoSegmentLayout {
        fn from(value: super::VideoSegmentLayout) -> Self {
            Self {
                crop_path: Ok(value.crop_path),
                state: Ok(value.state),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VideoSegmentLayoutCropPathItem {
        rect: ::std::result::Result<super::CropRect, ::std::string::String>,
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSegmentLayoutCropPathItem {
        fn default() -> Self {
            Self {
                rect: Err("no value supplied for rect".to_string()),
                t_ticks: Err("no value supplied for t_ticks".to_string()),
            }
        }
    }
    impl VideoSegmentLayoutCropPathItem {
        pub fn rect<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CropRect>,
            T::Error: ::std::fmt::Display,
        {
            self.rect = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rect: {e}"));
            self
        }
        pub fn t_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.t_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for t_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<VideoSegmentLayoutCropPathItem>
        for super::VideoSegmentLayoutCropPathItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VideoSegmentLayoutCropPathItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                rect: value.rect?,
                t_ticks: value.t_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::VideoSegmentLayoutCropPathItem>
        for VideoSegmentLayoutCropPathItem
    {
        fn from(value: super::VideoSegmentLayoutCropPathItem) -> Self {
            Self {
                rect: Ok(value.rect),
                t_ticks: Ok(value.t_ticks),
            }
        }
    }
}
