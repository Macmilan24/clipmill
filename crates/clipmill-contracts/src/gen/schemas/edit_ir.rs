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
#[doc = "                \"emphasis\": {"]
#[doc = "                  \"description\": \"A key word, set in the accent colour so it stands out of its line.\","]
#[doc = "                  \"type\": \"boolean\""]
#[doc = "                },"]
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
#[doc = "                },"]
#[doc = "                \"word_id\": {"]
#[doc = "                  \"description\": \"Which word this is, shared by its occurrence in the reading cues and in the burned-in cues. A correction is addressed to the word, so it lands in both presentations. Absent only in a document that predates word identities; the daemon assigns them on migration.\","]
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
#[doc = "    \"position\": {"]
#[doc = "      \"description\": \"Where this cue sits, when it was placed by hand. Overrides the region and the clip-wide position.\","]
#[doc = "      \"$ref\": \"#/$defs/captionPosition\""]
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
    #[doc = "Where this cue sits, when it was placed by hand. Overrides the region and the clip-wide position."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub position: ::std::option::Option<CaptionPosition>,
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
#[doc = "          \"emphasis\": {"]
#[doc = "            \"description\": \"A key word, set in the accent colour so it stands out of its line.\","]
#[doc = "            \"type\": \"boolean\""]
#[doc = "          },"]
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
#[doc = "          },"]
#[doc = "          \"word_id\": {"]
#[doc = "            \"description\": \"Which word this is, shared by its occurrence in the reading cues and in the burned-in cues. A correction is addressed to the word, so it lands in both presentations. Absent only in a document that predates word identities; the daemon assigns them on migration.\","]
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
#[doc = "    \"emphasis\": {"]
#[doc = "      \"description\": \"A key word, set in the accent colour so it stands out of its line.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
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
#[doc = "    },"]
#[doc = "    \"word_id\": {"]
#[doc = "      \"description\": \"Which word this is, shared by its occurrence in the reading cues and in the burned-in cues. A correction is addressed to the word, so it lands in both presentations. Absent only in a document that predates word identities; the daemon assigns them on migration.\","]
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
    #[doc = "A key word, set in the accent colour so it stands out of its line."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub emphasis: ::std::option::Option<bool>,
    pub end_ticks: ::std::num::NonZeroU64,
    pub start_ticks: u64,
    pub text: CaptionCueLinesItemWordsItemText,
    #[doc = "Which word this is, shared by its occurrence in the reading cues and in the burned-in cues. A correction is addressed to the word, so it lands in both presentations. Absent only in a document that predates word identities; the daemon assigns them on migration."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub word_id: ::std::option::Option<CaptionCueLinesItemWordsItemWordId>,
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
#[doc = "Which word this is, shared by its occurrence in the reading cues and in the burned-in cues. A correction is addressed to the word, so it lands in both presentations. Absent only in a document that predates word identities; the daemon assigns them on migration."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Which word this is, shared by its occurrence in the reading cues and in the burned-in cues. A correction is addressed to the word, so it lands in both presentations. Absent only in a document that predates word identities; the daemon assigns them on migration.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CaptionCueLinesItemWordsItemWordId(::std::string::String);
impl ::std::ops::Deref for CaptionCueLinesItemWordsItemWordId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CaptionCueLinesItemWordsItemWordId> for ::std::string::String {
    fn from(value: CaptionCueLinesItemWordsItemWordId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CaptionCueLinesItemWordsItemWordId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CaptionCueLinesItemWordsItemWordId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCueLinesItemWordsItemWordId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCueLinesItemWordsItemWordId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CaptionCueLinesItemWordsItemWordId {
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
#[doc = "A caption's centre on the frame, in thousandths of its width and height."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A caption's centre on the frame, in thousandths of its width and height.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"x\","]
#[doc = "    \"y\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"x\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 1000.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"y\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 1000.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CaptionPosition {
    pub x: i64,
    pub y: i64,
}
impl CaptionPosition {
    pub fn builder() -> builder::CaptionPosition {
        Default::default()
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
#[doc = "        \"burn_in\": {"]
#[doc = "          \"description\": \"What a watcher gets, when the two should differ. The kinetic grouping is burned into the picture; absent, the reading cues are burned in instead. Two lists rather than one because the caption engine produces two groupings of one token array, and this is where they would otherwise collapse back into one — a burn-in that inherited the reading grouping is merely conservative, while a sidecar that inherited the kinetic one is the divergence that engine exists to prevent. The asymmetry is deliberate.\","]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/captionCue\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"cues\": {"]
#[doc = "          \"description\": \"What a reader gets. Every sidecar is written from this list and only this list, because a sidecar is what a viewer who cannot hear is left with — so it carries the conservative grouping, always.\","]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/captionCue\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"options\": {"]
#[doc = "          \"description\": \"Saved clip-wide overrides over the named preset. Case changes burned-in captions only; sidecars retain the spoken spelling.\","]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"properties\": {"]
#[doc = "            \"accent\": {"]
#[doc = "              \"description\": \"The colour key words are set in.\","]
#[doc = "              \"type\": \"string\","]
#[doc = "              \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "            },"]
#[doc = "            \"font_family\": {"]
#[doc = "              \"description\": \"One of the caption fonts, by family name. Absent is the look's own.\","]
#[doc = "              \"enum\": ["]
#[doc = "                \"Inter\","]
#[doc = "                \"Montserrat Black\","]
#[doc = "                \"Poppins ExtraBold\","]
#[doc = "                \"Anton\","]
#[doc = "                \"Bebas Neue\","]
#[doc = "                \"Luckiest Guy\","]
#[doc = "                \"DM Serif Display\""]
#[doc = "              ]"]
#[doc = "            },"]
#[doc = "            \"font_size\": {"]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 160.0,"]
#[doc = "              \"minimum\": 24.0"]
#[doc = "            },"]
#[doc = "            \"highlight_spoken_word\": {"]
#[doc = "              \"description\": \"Override the preset's spoken-word highlight independently of its typography.\","]
#[doc = "              \"type\": \"boolean\""]
#[doc = "            },"]
#[doc = "            \"highlight_style\": {"]
#[doc = "              \"description\": \"How the spoken word is marked. Absent is the sweep.\","]
#[doc = "              \"enum\": ["]
#[doc = "                \"fill\","]
#[doc = "                \"word\","]
#[doc = "                \"box\","]
#[doc = "                \"pop\","]
#[doc = "                \"underline\""]
#[doc = "              ]"]
#[doc = "            },"]
#[doc = "            \"outline\": {"]
#[doc = "              \"type\": \"string\","]
#[doc = "              \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "            },"]
#[doc = "            \"outline_width\": {"]
#[doc = "              \"description\": \"Outline thickness at the 1920-pixel design height.\","]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 16.0,"]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"plate_opacity\": {"]
#[doc = "              \"description\": \"How opaque a boxed look's plate is, in percent.\","]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 100.0,"]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"position\": {"]
#[doc = "              \"description\": \"Where every caption sits unless a cue was placed on its own. Absent leaves each cue in its region.\","]
#[doc = "              \"$ref\": \"#/$defs/captionPosition\""]
#[doc = "            },"]
#[doc = "            \"shadow_depth\": {"]
#[doc = "              \"description\": \"Drop-shadow offset at the 1920-pixel design height.\","]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 12.0,"]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"spoken\": {"]
#[doc = "              \"type\": \"string\","]
#[doc = "              \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "            },"]
#[doc = "            \"text_case\": {"]
#[doc = "              \"enum\": ["]
#[doc = "                \"original\","]
#[doc = "                \"upper\","]
#[doc = "                \"lower\""]
#[doc = "              ]"]
#[doc = "            },"]
#[doc = "            \"unspoken\": {"]
#[doc = "              \"type\": \"string\","]
#[doc = "              \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "            },"]
#[doc = "            \"words_on_screen\": {"]
#[doc = "              \"description\": \"The most words the on-screen captions were last grouped into.\","]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 8.0,"]
#[doc = "              \"minimum\": 1.0"]
#[doc = "            }"]
#[doc = "          },"]
#[doc = "          \"additionalProperties\": false"]
#[doc = "        },"]
#[doc = "        \"style_ref\": {"]
#[doc = "          \"description\": \"Named caption preset; the style itself lives with the presets, not in every document.\","]
#[doc = "          \"type\": \"string\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"overlays\": {"]
#[doc = "      \"description\": \"Titles and labels laid over the program, bottom first. Spans are program time, like a cue's: a cut moves an overlay with the material around it and removes whatever it cut.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/overlay\""]
#[doc = "      }"]
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
#[doc = "    \"title\": {"]
#[doc = "      \"description\": \"What the clip is called, when somebody named it. Never consumed by any render path, like the rationale.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 120,"]
#[doc = "      \"minLength\": 1"]
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
#[doc = "        },"]
#[doc = "        \"shape\": {"]
#[doc = "          \"description\": \"The delivered frame's shape: vertical 9:16, portrait 4:5, square 1:1 or landscape 16:9. Absent is vertical. Crops are fitted to it and the render is sized by it; two viewports sit side by side in a landscape frame.\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"vertical\","]
#[doc = "            \"portrait\","]
#[doc = "            \"square\","]
#[doc = "            \"landscape\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"transition_ticks\": {"]
#[doc = "          \"description\": \"Requested duration of soft cuts: hold the last outgoing composition over incoming video. Zero or absent preserves hard cuts. Effective duration is bounded by the incoming shot; audio, captions and program timing remain unchanged.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 22500.0,"]
#[doc = "          \"minimum\": 0.0"]
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
    #[doc = "Titles and labels laid over the program, bottom first. Spans are program time, like a cue's: a cut moves an overlay with the material around it and removes whatever it cut."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub overlays: ::std::vec::Vec<Overlay>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub rationale: ::std::option::Option<EditIrRationale>,
    pub timebase: EditIrTimebase,
    #[doc = "What the clip is called, when somebody named it. Never consumed by any render path, like the rationale."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub title: ::std::option::Option<EditIrTitle>,
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
#[doc = "    \"burn_in\": {"]
#[doc = "      \"description\": \"What a watcher gets, when the two should differ. The kinetic grouping is burned into the picture; absent, the reading cues are burned in instead. Two lists rather than one because the caption engine produces two groupings of one token array, and this is where they would otherwise collapse back into one — a burn-in that inherited the reading grouping is merely conservative, while a sidecar that inherited the kinetic one is the divergence that engine exists to prevent. The asymmetry is deliberate.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/captionCue\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"cues\": {"]
#[doc = "      \"description\": \"What a reader gets. Every sidecar is written from this list and only this list, because a sidecar is what a viewer who cannot hear is left with — so it carries the conservative grouping, always.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/captionCue\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"options\": {"]
#[doc = "      \"description\": \"Saved clip-wide overrides over the named preset. Case changes burned-in captions only; sidecars retain the spoken spelling.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"properties\": {"]
#[doc = "        \"accent\": {"]
#[doc = "          \"description\": \"The colour key words are set in.\","]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "        },"]
#[doc = "        \"font_family\": {"]
#[doc = "          \"description\": \"One of the caption fonts, by family name. Absent is the look's own.\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"Inter\","]
#[doc = "            \"Montserrat Black\","]
#[doc = "            \"Poppins ExtraBold\","]
#[doc = "            \"Anton\","]
#[doc = "            \"Bebas Neue\","]
#[doc = "            \"Luckiest Guy\","]
#[doc = "            \"DM Serif Display\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"font_size\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 160.0,"]
#[doc = "          \"minimum\": 24.0"]
#[doc = "        },"]
#[doc = "        \"highlight_spoken_word\": {"]
#[doc = "          \"description\": \"Override the preset's spoken-word highlight independently of its typography.\","]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"highlight_style\": {"]
#[doc = "          \"description\": \"How the spoken word is marked. Absent is the sweep.\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"fill\","]
#[doc = "            \"word\","]
#[doc = "            \"box\","]
#[doc = "            \"pop\","]
#[doc = "            \"underline\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"outline\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "        },"]
#[doc = "        \"outline_width\": {"]
#[doc = "          \"description\": \"Outline thickness at the 1920-pixel design height.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 16.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"plate_opacity\": {"]
#[doc = "          \"description\": \"How opaque a boxed look's plate is, in percent.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 100.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"position\": {"]
#[doc = "          \"description\": \"Where every caption sits unless a cue was placed on its own. Absent leaves each cue in its region.\","]
#[doc = "          \"$ref\": \"#/$defs/captionPosition\""]
#[doc = "        },"]
#[doc = "        \"shadow_depth\": {"]
#[doc = "          \"description\": \"Drop-shadow offset at the 1920-pixel design height.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 12.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"spoken\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "        },"]
#[doc = "        \"text_case\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"original\","]
#[doc = "            \"upper\","]
#[doc = "            \"lower\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"unspoken\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "        },"]
#[doc = "        \"words_on_screen\": {"]
#[doc = "          \"description\": \"The most words the on-screen captions were last grouped into.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 8.0,"]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
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
    #[doc = "What a watcher gets, when the two should differ. The kinetic grouping is burned into the picture; absent, the reading cues are burned in instead. Two lists rather than one because the caption engine produces two groupings of one token array, and this is where they would otherwise collapse back into one — a burn-in that inherited the reading grouping is merely conservative, while a sidecar that inherited the kinetic one is the divergence that engine exists to prevent. The asymmetry is deliberate."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub burn_in: ::std::vec::Vec<CaptionCue>,
    #[doc = "What a reader gets. Every sidecar is written from this list and only this list, because a sidecar is what a viewer who cannot hear is left with — so it carries the conservative grouping, always."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub cues: ::std::vec::Vec<CaptionCue>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub options: ::std::option::Option<EditIrCaptionsOptions>,
    #[doc = "Named caption preset; the style itself lives with the presets, not in every document."]
    pub style_ref: ::std::string::String,
}
impl EditIrCaptions {
    pub fn builder() -> builder::EditIrCaptions {
        Default::default()
    }
}
#[doc = "Saved clip-wide overrides over the named preset. Case changes burned-in captions only; sidecars retain the spoken spelling."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Saved clip-wide overrides over the named preset. Case changes burned-in captions only; sidecars retain the spoken spelling.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"accent\": {"]
#[doc = "      \"description\": \"The colour key words are set in.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "    },"]
#[doc = "    \"font_family\": {"]
#[doc = "      \"description\": \"One of the caption fonts, by family name. Absent is the look's own.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"Inter\","]
#[doc = "        \"Montserrat Black\","]
#[doc = "        \"Poppins ExtraBold\","]
#[doc = "        \"Anton\","]
#[doc = "        \"Bebas Neue\","]
#[doc = "        \"Luckiest Guy\","]
#[doc = "        \"DM Serif Display\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"font_size\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 160.0,"]
#[doc = "      \"minimum\": 24.0"]
#[doc = "    },"]
#[doc = "    \"highlight_spoken_word\": {"]
#[doc = "      \"description\": \"Override the preset's spoken-word highlight independently of its typography.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"highlight_style\": {"]
#[doc = "      \"description\": \"How the spoken word is marked. Absent is the sweep.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"fill\","]
#[doc = "        \"word\","]
#[doc = "        \"box\","]
#[doc = "        \"pop\","]
#[doc = "        \"underline\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"outline\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "    },"]
#[doc = "    \"outline_width\": {"]
#[doc = "      \"description\": \"Outline thickness at the 1920-pixel design height.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 16.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"plate_opacity\": {"]
#[doc = "      \"description\": \"How opaque a boxed look's plate is, in percent.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 100.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"position\": {"]
#[doc = "      \"description\": \"Where every caption sits unless a cue was placed on its own. Absent leaves each cue in its region.\","]
#[doc = "      \"$ref\": \"#/$defs/captionPosition\""]
#[doc = "    },"]
#[doc = "    \"shadow_depth\": {"]
#[doc = "      \"description\": \"Drop-shadow offset at the 1920-pixel design height.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 12.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"spoken\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "    },"]
#[doc = "    \"text_case\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"original\","]
#[doc = "        \"upper\","]
#[doc = "        \"lower\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"unspoken\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "    },"]
#[doc = "    \"words_on_screen\": {"]
#[doc = "      \"description\": \"The most words the on-screen captions were last grouped into.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 8.0,"]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditIrCaptionsOptions {
    #[doc = "The colour key words are set in."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub accent: ::std::option::Option<EditIrCaptionsOptionsAccent>,
    #[doc = "One of the caption fonts, by family name. Absent is the look's own."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub font_family: ::std::option::Option<EditIrCaptionsOptionsFontFamily>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub font_size: ::std::option::Option<i64>,
    #[doc = "Override the preset's spoken-word highlight independently of its typography."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub highlight_spoken_word: ::std::option::Option<bool>,
    #[doc = "How the spoken word is marked. Absent is the sweep."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub highlight_style: ::std::option::Option<EditIrCaptionsOptionsHighlightStyle>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub outline: ::std::option::Option<EditIrCaptionsOptionsOutline>,
    #[doc = "Outline thickness at the 1920-pixel design height."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub outline_width: ::std::option::Option<i64>,
    #[doc = "How opaque a boxed look's plate is, in percent."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub plate_opacity: ::std::option::Option<i64>,
    #[doc = "Where every caption sits unless a cue was placed on its own. Absent leaves each cue in its region."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub position: ::std::option::Option<CaptionPosition>,
    #[doc = "Drop-shadow offset at the 1920-pixel design height."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub shadow_depth: ::std::option::Option<i64>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub spoken: ::std::option::Option<EditIrCaptionsOptionsSpoken>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub text_case: ::std::option::Option<EditIrCaptionsOptionsTextCase>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub unspoken: ::std::option::Option<EditIrCaptionsOptionsUnspoken>,
    #[doc = "The most words the on-screen captions were last grouped into."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub words_on_screen: ::std::option::Option<::std::num::NonZeroU64>,
}
impl ::std::default::Default for EditIrCaptionsOptions {
    fn default() -> Self {
        Self {
            accent: Default::default(),
            font_family: Default::default(),
            font_size: Default::default(),
            highlight_spoken_word: Default::default(),
            highlight_style: Default::default(),
            outline: Default::default(),
            outline_width: Default::default(),
            plate_opacity: Default::default(),
            position: Default::default(),
            shadow_depth: Default::default(),
            spoken: Default::default(),
            text_case: Default::default(),
            unspoken: Default::default(),
            words_on_screen: Default::default(),
        }
    }
}
impl EditIrCaptionsOptions {
    pub fn builder() -> builder::EditIrCaptionsOptions {
        Default::default()
    }
}
#[doc = "The colour key words are set in."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The colour key words are set in.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EditIrCaptionsOptionsAccent(::std::string::String);
impl ::std::ops::Deref for EditIrCaptionsOptionsAccent {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EditIrCaptionsOptionsAccent> for ::std::string::String {
    fn from(value: EditIrCaptionsOptionsAccent) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EditIrCaptionsOptionsAccent {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^#[0-9a-fA-F]{6}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^#[0-9a-fA-F]{6}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EditIrCaptionsOptionsAccent {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrCaptionsOptionsAccent {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrCaptionsOptionsAccent {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EditIrCaptionsOptionsAccent {
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
#[doc = "One of the caption fonts, by family name. Absent is the look's own."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"One of the caption fonts, by family name. Absent is the look's own.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"Inter\","]
#[doc = "    \"Montserrat Black\","]
#[doc = "    \"Poppins ExtraBold\","]
#[doc = "    \"Anton\","]
#[doc = "    \"Bebas Neue\","]
#[doc = "    \"Luckiest Guy\","]
#[doc = "    \"DM Serif Display\""]
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
pub enum EditIrCaptionsOptionsFontFamily {
    Inter,
    #[serde(rename = "Montserrat Black")]
    MontserratBlack,
    #[serde(rename = "Poppins ExtraBold")]
    PoppinsExtraBold,
    Anton,
    #[serde(rename = "Bebas Neue")]
    BebasNeue,
    #[serde(rename = "Luckiest Guy")]
    LuckiestGuy,
    #[serde(rename = "DM Serif Display")]
    DmSerifDisplay,
}
impl ::std::fmt::Display for EditIrCaptionsOptionsFontFamily {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Inter => f.write_str("Inter"),
            Self::MontserratBlack => f.write_str("Montserrat Black"),
            Self::PoppinsExtraBold => f.write_str("Poppins ExtraBold"),
            Self::Anton => f.write_str("Anton"),
            Self::BebasNeue => f.write_str("Bebas Neue"),
            Self::LuckiestGuy => f.write_str("Luckiest Guy"),
            Self::DmSerifDisplay => f.write_str("DM Serif Display"),
        }
    }
}
impl ::std::str::FromStr for EditIrCaptionsOptionsFontFamily {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "Inter" => Ok(Self::Inter),
            "Montserrat Black" => Ok(Self::MontserratBlack),
            "Poppins ExtraBold" => Ok(Self::PoppinsExtraBold),
            "Anton" => Ok(Self::Anton),
            "Bebas Neue" => Ok(Self::BebasNeue),
            "Luckiest Guy" => Ok(Self::LuckiestGuy),
            "DM Serif Display" => Ok(Self::DmSerifDisplay),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EditIrCaptionsOptionsFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrCaptionsOptionsFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrCaptionsOptionsFontFamily {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "How the spoken word is marked. Absent is the sweep."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How the spoken word is marked. Absent is the sweep.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"fill\","]
#[doc = "    \"word\","]
#[doc = "    \"box\","]
#[doc = "    \"pop\","]
#[doc = "    \"underline\""]
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
pub enum EditIrCaptionsOptionsHighlightStyle {
    #[serde(rename = "fill")]
    Fill,
    #[serde(rename = "word")]
    Word,
    #[serde(rename = "box")]
    Box,
    #[serde(rename = "pop")]
    Pop,
    #[serde(rename = "underline")]
    Underline,
}
impl ::std::fmt::Display for EditIrCaptionsOptionsHighlightStyle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Fill => f.write_str("fill"),
            Self::Word => f.write_str("word"),
            Self::Box => f.write_str("box"),
            Self::Pop => f.write_str("pop"),
            Self::Underline => f.write_str("underline"),
        }
    }
}
impl ::std::str::FromStr for EditIrCaptionsOptionsHighlightStyle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "fill" => Ok(Self::Fill),
            "word" => Ok(Self::Word),
            "box" => Ok(Self::Box),
            "pop" => Ok(Self::Pop),
            "underline" => Ok(Self::Underline),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EditIrCaptionsOptionsHighlightStyle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrCaptionsOptionsHighlightStyle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrCaptionsOptionsHighlightStyle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`EditIrCaptionsOptionsOutline`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EditIrCaptionsOptionsOutline(::std::string::String);
impl ::std::ops::Deref for EditIrCaptionsOptionsOutline {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EditIrCaptionsOptionsOutline> for ::std::string::String {
    fn from(value: EditIrCaptionsOptionsOutline) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EditIrCaptionsOptionsOutline {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^#[0-9a-fA-F]{6}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^#[0-9a-fA-F]{6}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EditIrCaptionsOptionsOutline {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrCaptionsOptionsOutline {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrCaptionsOptionsOutline {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EditIrCaptionsOptionsOutline {
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
#[doc = "`EditIrCaptionsOptionsSpoken`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EditIrCaptionsOptionsSpoken(::std::string::String);
impl ::std::ops::Deref for EditIrCaptionsOptionsSpoken {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EditIrCaptionsOptionsSpoken> for ::std::string::String {
    fn from(value: EditIrCaptionsOptionsSpoken) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EditIrCaptionsOptionsSpoken {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^#[0-9a-fA-F]{6}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^#[0-9a-fA-F]{6}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EditIrCaptionsOptionsSpoken {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrCaptionsOptionsSpoken {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrCaptionsOptionsSpoken {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EditIrCaptionsOptionsSpoken {
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
#[doc = "`EditIrCaptionsOptionsTextCase`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"original\","]
#[doc = "    \"upper\","]
#[doc = "    \"lower\""]
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
pub enum EditIrCaptionsOptionsTextCase {
    #[serde(rename = "original")]
    Original,
    #[serde(rename = "upper")]
    Upper,
    #[serde(rename = "lower")]
    Lower,
}
impl ::std::fmt::Display for EditIrCaptionsOptionsTextCase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Original => f.write_str("original"),
            Self::Upper => f.write_str("upper"),
            Self::Lower => f.write_str("lower"),
        }
    }
}
impl ::std::str::FromStr for EditIrCaptionsOptionsTextCase {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "original" => Ok(Self::Original),
            "upper" => Ok(Self::Upper),
            "lower" => Ok(Self::Lower),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EditIrCaptionsOptionsTextCase {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrCaptionsOptionsTextCase {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrCaptionsOptionsTextCase {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`EditIrCaptionsOptionsUnspoken`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^#[0-9a-fA-F]{6}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EditIrCaptionsOptionsUnspoken(::std::string::String);
impl ::std::ops::Deref for EditIrCaptionsOptionsUnspoken {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EditIrCaptionsOptionsUnspoken> for ::std::string::String {
    fn from(value: EditIrCaptionsOptionsUnspoken) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EditIrCaptionsOptionsUnspoken {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^#[0-9a-fA-F]{6}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^#[0-9a-fA-F]{6}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EditIrCaptionsOptionsUnspoken {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrCaptionsOptionsUnspoken {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrCaptionsOptionsUnspoken {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EditIrCaptionsOptionsUnspoken {
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
#[doc = "What the clip is called, when somebody named it. Never consumed by any render path, like the rationale."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What the clip is called, when somebody named it. Never consumed by any render path, like the rationale.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 120,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EditIrTitle(::std::string::String);
impl ::std::ops::Deref for EditIrTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EditIrTitle> for ::std::string::String {
    fn from(value: EditIrTitle) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EditIrTitle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 120usize {
            return Err("longer than 120 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EditIrTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EditIrTitle {
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
#[doc = "    },"]
#[doc = "    \"shape\": {"]
#[doc = "      \"description\": \"The delivered frame's shape: vertical 9:16, portrait 4:5, square 1:1 or landscape 16:9. Absent is vertical. Crops are fitted to it and the render is sized by it; two viewports sit side by side in a landscape frame.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"vertical\","]
#[doc = "        \"portrait\","]
#[doc = "        \"square\","]
#[doc = "        \"landscape\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"transition_ticks\": {"]
#[doc = "      \"description\": \"Requested duration of soft cuts: hold the last outgoing composition over incoming video. Zero or absent preserves hard cuts. Effective duration is bounded by the incoming shot; audio, captions and program timing remain unchanged.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 22500.0,"]
#[doc = "      \"minimum\": 0.0"]
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
    #[doc = "The delivered frame's shape: vertical 9:16, portrait 4:5, square 1:1 or landscape 16:9. Absent is vertical. Crops are fitted to it and the render is sized by it; two viewports sit side by side in a landscape frame."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub shape: ::std::option::Option<EditIrVideoShape>,
    #[doc = "Requested duration of soft cuts: hold the last outgoing composition over incoming video. Zero or absent preserves hard cuts. Effective duration is bounded by the incoming shot; audio, captions and program timing remain unchanged."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub transition_ticks: ::std::option::Option<i64>,
}
impl ::std::default::Default for EditIrVideo {
    fn default() -> Self {
        Self {
            segments: Default::default(),
            shape: Default::default(),
            transition_ticks: Default::default(),
        }
    }
}
impl EditIrVideo {
    pub fn builder() -> builder::EditIrVideo {
        Default::default()
    }
}
#[doc = "The delivered frame's shape: vertical 9:16, portrait 4:5, square 1:1 or landscape 16:9. Absent is vertical. Crops are fitted to it and the render is sized by it; two viewports sit side by side in a landscape frame."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The delivered frame's shape: vertical 9:16, portrait 4:5, square 1:1 or landscape 16:9. Absent is vertical. Crops are fitted to it and the render is sized by it; two viewports sit side by side in a landscape frame.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"vertical\","]
#[doc = "    \"portrait\","]
#[doc = "    \"square\","]
#[doc = "    \"landscape\""]
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
pub enum EditIrVideoShape {
    #[serde(rename = "vertical")]
    Vertical,
    #[serde(rename = "portrait")]
    Portrait,
    #[serde(rename = "square")]
    Square,
    #[serde(rename = "landscape")]
    Landscape,
}
impl ::std::fmt::Display for EditIrVideoShape {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Vertical => f.write_str("vertical"),
            Self::Portrait => f.write_str("portrait"),
            Self::Square => f.write_str("square"),
            Self::Landscape => f.write_str("landscape"),
        }
    }
}
impl ::std::str::FromStr for EditIrVideoShape {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "vertical" => Ok(Self::Vertical),
            "portrait" => Ok(Self::Portrait),
            "square" => Ok(Self::Square),
            "landscape" => Ok(Self::Landscape),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EditIrVideoShape {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditIrVideoShape {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditIrVideoShape {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`HexColour`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^#[0-9A-Fa-f]{6}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct HexColour(::std::string::String);
impl ::std::ops::Deref for HexColour {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<HexColour> for ::std::string::String {
    fn from(value: HexColour) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for HexColour {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^#[0-9A-Fa-f]{6}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^#[0-9A-Fa-f]{6}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for HexColour {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for HexColour {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for HexColour {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for HexColour {
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
#[doc = "Something laid over the program for a span of it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Something laid over the program for a span of it.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"content\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"overlay_id\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"content\": {"]
#[doc = "      \"oneOf\": ["]
#[doc = "        {"]
#[doc = "          \"description\": \"Words set in the clip's caption font by the caption renderer. Lines break only where the text says.\","]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"required\": ["]
#[doc = "            \"colour\","]
#[doc = "            \"kind\","]
#[doc = "            \"size\","]
#[doc = "            \"text\","]
#[doc = "            \"x\","]
#[doc = "            \"y\""]
#[doc = "          ],"]
#[doc = "          \"properties\": {"]
#[doc = "            \"colour\": {"]
#[doc = "              \"$ref\": \"#/$defs/hexColour\""]
#[doc = "            },"]
#[doc = "            \"kind\": {"]
#[doc = "              \"const\": \"text\""]
#[doc = "            },"]
#[doc = "            \"plate\": {"]
#[doc = "              \"description\": \"An opaque plate behind the text. Absent draws an outline.\","]
#[doc = "              \"$ref\": \"#/$defs/hexColour\""]
#[doc = "            },"]
#[doc = "            \"role\": {"]
#[doc = "              \"description\": \"A hook opens the clip and names what it is about. Absent is a label.\","]
#[doc = "              \"enum\": ["]
#[doc = "                \"hook\","]
#[doc = "                \"label\""]
#[doc = "              ]"]
#[doc = "            },"]
#[doc = "            \"size\": {"]
#[doc = "              \"description\": \"Its size at the 1920-pixel design height.\","]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 240.0,"]
#[doc = "              \"minimum\": 24.0"]
#[doc = "            },"]
#[doc = "            \"text\": {"]
#[doc = "              \"type\": \"string\","]
#[doc = "              \"maxLength\": 160,"]
#[doc = "              \"minLength\": 1,"]
#[doc = "              \"pattern\": \"^[^{}\\\\\\\\]*$\""]
#[doc = "            },"]
#[doc = "            \"x\": {"]
#[doc = "              \"description\": \"Where its centre sits, per mille of the frame's width.\","]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 1000.0,"]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"y\": {"]
#[doc = "              \"description\": \"Where its centre sits, per mille of the frame's height.\","]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 1000.0,"]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            }"]
#[doc = "          },"]
#[doc = "          \"additionalProperties\": false"]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"overlay_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
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
pub struct Overlay {
    pub content: OverlayContent,
    pub end_ticks: ::std::num::NonZeroU64,
    pub overlay_id: OverlayOverlayId,
    pub start_ticks: u64,
}
impl Overlay {
    pub fn builder() -> builder::Overlay {
        Default::default()
    }
}
#[doc = "`OverlayContent`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"description\": \"Words set in the clip's caption font by the caption renderer. Lines break only where the text says.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"colour\","]
#[doc = "        \"kind\","]
#[doc = "        \"size\","]
#[doc = "        \"text\","]
#[doc = "        \"x\","]
#[doc = "        \"y\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"colour\": {"]
#[doc = "          \"$ref\": \"#/$defs/hexColour\""]
#[doc = "        },"]
#[doc = "        \"kind\": {"]
#[doc = "          \"const\": \"text\""]
#[doc = "        },"]
#[doc = "        \"plate\": {"]
#[doc = "          \"description\": \"An opaque plate behind the text. Absent draws an outline.\","]
#[doc = "          \"$ref\": \"#/$defs/hexColour\""]
#[doc = "        },"]
#[doc = "        \"role\": {"]
#[doc = "          \"description\": \"A hook opens the clip and names what it is about. Absent is a label.\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"hook\","]
#[doc = "            \"label\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"size\": {"]
#[doc = "          \"description\": \"Its size at the 1920-pixel design height.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 240.0,"]
#[doc = "          \"minimum\": 24.0"]
#[doc = "        },"]
#[doc = "        \"text\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"maxLength\": 160,"]
#[doc = "          \"minLength\": 1,"]
#[doc = "          \"pattern\": \"^[^{}\\\\\\\\]*$\""]
#[doc = "        },"]
#[doc = "        \"x\": {"]
#[doc = "          \"description\": \"Where its centre sits, per mille of the frame's width.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 1000.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"y\": {"]
#[doc = "          \"description\": \"Where its centre sits, per mille of the frame's height.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 1000.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum OverlayContent {
    #[doc = "Words set in the clip's caption font by the caption renderer. Lines break only where the text says."]
    #[serde(rename = "text")]
    Text {
        colour: HexColour,
        #[doc = "An opaque plate behind the text. Absent draws an outline."]
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        plate: ::std::option::Option<HexColour>,
        #[doc = "A hook opens the clip and names what it is about. Absent is a label."]
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        role: ::std::option::Option<OverlayContentRole>,
        #[doc = "Its size at the 1920-pixel design height."]
        size: i64,
        text: OverlayContentText,
        #[doc = "Where its centre sits, per mille of the frame's width."]
        x: i64,
        #[doc = "Where its centre sits, per mille of the frame's height."]
        y: i64,
    },
}
#[doc = "A hook opens the clip and names what it is about. Absent is a label."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A hook opens the clip and names what it is about. Absent is a label.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"hook\","]
#[doc = "    \"label\""]
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
pub enum OverlayContentRole {
    #[serde(rename = "hook")]
    Hook,
    #[serde(rename = "label")]
    Label,
}
impl ::std::fmt::Display for OverlayContentRole {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Hook => f.write_str("hook"),
            Self::Label => f.write_str("label"),
        }
    }
}
impl ::std::str::FromStr for OverlayContentRole {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "hook" => Ok(Self::Hook),
            "label" => Ok(Self::Label),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for OverlayContentRole {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for OverlayContentRole {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for OverlayContentRole {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`OverlayContentText`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 160,"]
#[doc = "  \"minLength\": 1,"]
#[doc = "  \"pattern\": \"^[^{}\\\\\\\\]*$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct OverlayContentText(::std::string::String);
impl ::std::ops::Deref for OverlayContentText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<OverlayContentText> for ::std::string::String {
    fn from(value: OverlayContentText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for OverlayContentText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 160usize {
            return Err("longer than 160 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[^{}\\\\]*$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[^{}\\\\]*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for OverlayContentText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for OverlayContentText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for OverlayContentText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for OverlayContentText {
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
#[doc = "`OverlayOverlayId`"]
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
pub struct OverlayOverlayId(::std::string::String);
impl ::std::ops::Deref for OverlayOverlayId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<OverlayOverlayId> for ::std::string::String {
    fn from(value: OverlayOverlayId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for OverlayOverlayId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for OverlayOverlayId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for OverlayOverlayId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for OverlayOverlayId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for OverlayOverlayId {
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
#[doc = "        \"background\": {"]
#[doc = "          \"description\": \"What fills around a fitted picture. Absent is the picture itself, blurred.\","]
#[doc = "          \"oneOf\": ["]
#[doc = "            {"]
#[doc = "              \"type\": \"object\","]
#[doc = "              \"required\": ["]
#[doc = "                \"kind\""]
#[doc = "              ],"]
#[doc = "              \"properties\": {"]
#[doc = "                \"kind\": {"]
#[doc = "                  \"const\": \"blur\""]
#[doc = "                }"]
#[doc = "              },"]
#[doc = "              \"additionalProperties\": false"]
#[doc = "            },"]
#[doc = "            {"]
#[doc = "              \"type\": \"object\","]
#[doc = "              \"required\": ["]
#[doc = "                \"colour\","]
#[doc = "                \"kind\""]
#[doc = "              ],"]
#[doc = "              \"properties\": {"]
#[doc = "                \"colour\": {"]
#[doc = "                  \"type\": \"string\","]
#[doc = "                  \"pattern\": \"^#[0-9A-Fa-f]{6}$\""]
#[doc = "                },"]
#[doc = "                \"kind\": {"]
#[doc = "                  \"const\": \"colour\""]
#[doc = "                }"]
#[doc = "              },"]
#[doc = "              \"additionalProperties\": false"]
#[doc = "            }"]
#[doc = "          ]"]
#[doc = "        },"]
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
#[doc = "              \"easing\": {"]
#[doc = "                \"enum\": ["]
#[doc = "                  \"linear\","]
#[doc = "                  \"ease_in\","]
#[doc = "                  \"ease_out\","]
#[doc = "                  \"ease_in_out\""]
#[doc = "                ]"]
#[doc = "              },"]
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
#[doc = "        \"inset\": {"]
#[doc = "          \"description\": \"Where a picture_in_picture inset sits: a corner, and its side as a share of the frame's short side, per mille. It is square.\","]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"required\": ["]
#[doc = "            \"corner\","]
#[doc = "            \"size\""]
#[doc = "          ],"]
#[doc = "          \"properties\": {"]
#[doc = "            \"corner\": {"]
#[doc = "              \"enum\": ["]
#[doc = "                \"top_left\","]
#[doc = "                \"top_right\","]
#[doc = "                \"bottom_left\","]
#[doc = "                \"bottom_right\""]
#[doc = "              ]"]
#[doc = "            },"]
#[doc = "            \"size\": {"]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 600.0,"]
#[doc = "              \"minimum\": 200.0"]
#[doc = "            }"]
#[doc = "          },"]
#[doc = "          \"additionalProperties\": false"]
#[doc = "        },"]
#[doc = "        \"punches\": {"]
#[doc = "          \"description\": \"Moments a followed crop moves in closer, in order and apart, segment-local like its keyframes. The crop path under them is kept as it is. Each lasts at least two moves (12000 ticks) and meets the one before or starts at least two moves after it.\","]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"end_ticks\","]
#[doc = "              \"start_ticks\","]
#[doc = "              \"zoom\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"end_ticks\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 1.0"]
#[doc = "              },"]
#[doc = "              \"start_ticks\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              },"]
#[doc = "              \"zoom\": {"]
#[doc = "                \"description\": \"How much closer, in percent.\","]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"maximum\": 200.0,"]
#[doc = "                \"minimum\": 105.0"]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"secondary_crop_path\": {"]
#[doc = "          \"description\": \"Lower viewport crop keyframes for a two_up composition, or the inset of a picture_in_picture. The primary path fills the upper viewport; both paths use segment-local ticks.\","]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"rect\","]
#[doc = "              \"t_ticks\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"easing\": {"]
#[doc = "                \"enum\": ["]
#[doc = "                  \"linear\","]
#[doc = "                  \"ease_in\","]
#[doc = "                  \"ease_out\","]
#[doc = "                  \"ease_in_out\""]
#[doc = "                ]"]
#[doc = "              },"]
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
#[doc = "        \"split\": {"]
#[doc = "          \"description\": \"two_up: the first viewport's share of the frame, per mille: of the height when the viewports are stacked, of the width side by side in a landscape frame. Absent is an even split; a screen share over a face is the first viewport at the recording's own shape.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 750.0,"]
#[doc = "          \"minimum\": 250.0"]
#[doc = "        },"]
#[doc = "        \"state\": {"]
#[doc = "          \"description\": \"picture_in_picture draws the full picture (the crop path, or the whole frame when it is empty) with the secondary path inset in one corner.\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"speaker_fill\","]
#[doc = "            \"fit\","]
#[doc = "            \"two_up\","]
#[doc = "            \"picture_in_picture\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"zoom\": {"]
#[doc = "          \"description\": \"How far past fitting a fitted picture is zoomed, in percent, about its centre. Absent is 100.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 250.0,"]
#[doc = "          \"minimum\": 100.0"]
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
#[doc = "    \"background\": {"]
#[doc = "      \"description\": \"What fills around a fitted picture. Absent is the picture itself, blurred.\","]
#[doc = "      \"oneOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"required\": ["]
#[doc = "            \"kind\""]
#[doc = "          ],"]
#[doc = "          \"properties\": {"]
#[doc = "            \"kind\": {"]
#[doc = "              \"const\": \"blur\""]
#[doc = "            }"]
#[doc = "          },"]
#[doc = "          \"additionalProperties\": false"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"required\": ["]
#[doc = "            \"colour\","]
#[doc = "            \"kind\""]
#[doc = "          ],"]
#[doc = "          \"properties\": {"]
#[doc = "            \"colour\": {"]
#[doc = "              \"type\": \"string\","]
#[doc = "              \"pattern\": \"^#[0-9A-Fa-f]{6}$\""]
#[doc = "            },"]
#[doc = "            \"kind\": {"]
#[doc = "              \"const\": \"colour\""]
#[doc = "            }"]
#[doc = "          },"]
#[doc = "          \"additionalProperties\": false"]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
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
#[doc = "          \"easing\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"linear\","]
#[doc = "              \"ease_in\","]
#[doc = "              \"ease_out\","]
#[doc = "              \"ease_in_out\""]
#[doc = "            ]"]
#[doc = "          },"]
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
#[doc = "    \"inset\": {"]
#[doc = "      \"description\": \"Where a picture_in_picture inset sits: a corner, and its side as a share of the frame's short side, per mille. It is square.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"corner\","]
#[doc = "        \"size\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"corner\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"top_left\","]
#[doc = "            \"top_right\","]
#[doc = "            \"bottom_left\","]
#[doc = "            \"bottom_right\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"size\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 600.0,"]
#[doc = "          \"minimum\": 200.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"punches\": {"]
#[doc = "      \"description\": \"Moments a followed crop moves in closer, in order and apart, segment-local like its keyframes. The crop path under them is kept as it is. Each lasts at least two moves (12000 ticks) and meets the one before or starts at least two moves after it.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"end_ticks\","]
#[doc = "          \"start_ticks\","]
#[doc = "          \"zoom\""]
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
#[doc = "          \"zoom\": {"]
#[doc = "            \"description\": \"How much closer, in percent.\","]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"maximum\": 200.0,"]
#[doc = "            \"minimum\": 105.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"secondary_crop_path\": {"]
#[doc = "      \"description\": \"Lower viewport crop keyframes for a two_up composition, or the inset of a picture_in_picture. The primary path fills the upper viewport; both paths use segment-local ticks.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"rect\","]
#[doc = "          \"t_ticks\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"easing\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"linear\","]
#[doc = "              \"ease_in\","]
#[doc = "              \"ease_out\","]
#[doc = "              \"ease_in_out\""]
#[doc = "            ]"]
#[doc = "          },"]
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
#[doc = "    \"split\": {"]
#[doc = "      \"description\": \"two_up: the first viewport's share of the frame, per mille: of the height when the viewports are stacked, of the width side by side in a landscape frame. Absent is an even split; a screen share over a face is the first viewport at the recording's own shape.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 750.0,"]
#[doc = "      \"minimum\": 250.0"]
#[doc = "    },"]
#[doc = "    \"state\": {"]
#[doc = "      \"description\": \"picture_in_picture draws the full picture (the crop path, or the whole frame when it is empty) with the secondary path inset in one corner.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"speaker_fill\","]
#[doc = "        \"fit\","]
#[doc = "        \"two_up\","]
#[doc = "        \"picture_in_picture\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"zoom\": {"]
#[doc = "      \"description\": \"How far past fitting a fitted picture is zoomed, in percent, about its centre. Absent is 100.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 250.0,"]
#[doc = "      \"minimum\": 100.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VideoSegmentLayout {
    #[doc = "What fills around a fitted picture. Absent is the picture itself, blurred."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub background: ::std::option::Option<VideoSegmentLayoutBackground>,
    #[doc = "Crop keyframes in segment-local ticks, so trimming the source window cannot silently re-time the camera move."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub crop_path: ::std::vec::Vec<VideoSegmentLayoutCropPathItem>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub inset: ::std::option::Option<VideoSegmentLayoutInset>,
    #[doc = "Moments a followed crop moves in closer, in order and apart, segment-local like its keyframes. The crop path under them is kept as it is. Each lasts at least two moves (12000 ticks) and meets the one before or starts at least two moves after it."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub punches: ::std::vec::Vec<VideoSegmentLayoutPunchesItem>,
    #[doc = "Lower viewport crop keyframes for a two_up composition, or the inset of a picture_in_picture. The primary path fills the upper viewport; both paths use segment-local ticks."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub secondary_crop_path: ::std::vec::Vec<VideoSegmentLayoutSecondaryCropPathItem>,
    #[doc = "two_up: the first viewport's share of the frame, per mille: of the height when the viewports are stacked, of the width side by side in a landscape frame. Absent is an even split; a screen share over a face is the first viewport at the recording's own shape."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub split: ::std::option::Option<i64>,
    #[doc = "picture_in_picture draws the full picture (the crop path, or the whole frame when it is empty) with the secondary path inset in one corner."]
    pub state: VideoSegmentLayoutState,
    #[doc = "How far past fitting a fitted picture is zoomed, in percent, about its centre. Absent is 100."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub zoom: ::std::option::Option<i64>,
}
impl VideoSegmentLayout {
    pub fn builder() -> builder::VideoSegmentLayout {
        Default::default()
    }
}
#[doc = "What fills around a fitted picture. Absent is the picture itself, blurred."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What fills around a fitted picture. Absent is the picture itself, blurred.\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"kind\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"kind\": {"]
#[doc = "          \"const\": \"blur\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"colour\","]
#[doc = "        \"kind\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"colour\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"pattern\": \"^#[0-9A-Fa-f]{6}$\""]
#[doc = "        },"]
#[doc = "        \"kind\": {"]
#[doc = "          \"const\": \"colour\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(tag = "kind", content = "colour")]
pub enum VideoSegmentLayoutBackground {
    #[serde(rename = "blur")]
    Blur,
    #[serde(rename = "colour")]
    Colour(VideoSegmentLayoutBackgroundColour),
}
impl ::std::convert::From<VideoSegmentLayoutBackgroundColour> for VideoSegmentLayoutBackground {
    fn from(value: VideoSegmentLayoutBackgroundColour) -> Self {
        Self::Colour(value)
    }
}
#[doc = "`VideoSegmentLayoutBackgroundColour`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^#[0-9A-Fa-f]{6}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct VideoSegmentLayoutBackgroundColour(::std::string::String);
impl ::std::ops::Deref for VideoSegmentLayoutBackgroundColour {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<VideoSegmentLayoutBackgroundColour> for ::std::string::String {
    fn from(value: VideoSegmentLayoutBackgroundColour) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for VideoSegmentLayoutBackgroundColour {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^#[0-9A-Fa-f]{6}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^#[0-9A-Fa-f]{6}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for VideoSegmentLayoutBackgroundColour {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for VideoSegmentLayoutBackgroundColour {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for VideoSegmentLayoutBackgroundColour {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for VideoSegmentLayoutBackgroundColour {
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
#[doc = "    \"easing\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"linear\","]
#[doc = "        \"ease_in\","]
#[doc = "        \"ease_out\","]
#[doc = "        \"ease_in_out\""]
#[doc = "      ]"]
#[doc = "    },"]
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
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub easing: ::std::option::Option<VideoSegmentLayoutCropPathItemEasing>,
    pub rect: CropRect,
    pub t_ticks: u64,
}
impl VideoSegmentLayoutCropPathItem {
    pub fn builder() -> builder::VideoSegmentLayoutCropPathItem {
        Default::default()
    }
}
#[doc = "`VideoSegmentLayoutCropPathItemEasing`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"linear\","]
#[doc = "    \"ease_in\","]
#[doc = "    \"ease_out\","]
#[doc = "    \"ease_in_out\""]
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
pub enum VideoSegmentLayoutCropPathItemEasing {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "ease_in")]
    EaseIn,
    #[serde(rename = "ease_out")]
    EaseOut,
    #[serde(rename = "ease_in_out")]
    EaseInOut,
}
impl ::std::fmt::Display for VideoSegmentLayoutCropPathItemEasing {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Linear => f.write_str("linear"),
            Self::EaseIn => f.write_str("ease_in"),
            Self::EaseOut => f.write_str("ease_out"),
            Self::EaseInOut => f.write_str("ease_in_out"),
        }
    }
}
impl ::std::str::FromStr for VideoSegmentLayoutCropPathItemEasing {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "linear" => Ok(Self::Linear),
            "ease_in" => Ok(Self::EaseIn),
            "ease_out" => Ok(Self::EaseOut),
            "ease_in_out" => Ok(Self::EaseInOut),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for VideoSegmentLayoutCropPathItemEasing {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for VideoSegmentLayoutCropPathItemEasing {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for VideoSegmentLayoutCropPathItemEasing {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Where a picture_in_picture inset sits: a corner, and its side as a share of the frame's short side, per mille. It is square."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Where a picture_in_picture inset sits: a corner, and its side as a share of the frame's short side, per mille. It is square.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"corner\","]
#[doc = "    \"size\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"corner\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"top_left\","]
#[doc = "        \"top_right\","]
#[doc = "        \"bottom_left\","]
#[doc = "        \"bottom_right\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"size\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 600.0,"]
#[doc = "      \"minimum\": 200.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VideoSegmentLayoutInset {
    pub corner: VideoSegmentLayoutInsetCorner,
    pub size: i64,
}
impl VideoSegmentLayoutInset {
    pub fn builder() -> builder::VideoSegmentLayoutInset {
        Default::default()
    }
}
#[doc = "`VideoSegmentLayoutInsetCorner`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"top_left\","]
#[doc = "    \"top_right\","]
#[doc = "    \"bottom_left\","]
#[doc = "    \"bottom_right\""]
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
pub enum VideoSegmentLayoutInsetCorner {
    #[serde(rename = "top_left")]
    TopLeft,
    #[serde(rename = "top_right")]
    TopRight,
    #[serde(rename = "bottom_left")]
    BottomLeft,
    #[serde(rename = "bottom_right")]
    BottomRight,
}
impl ::std::fmt::Display for VideoSegmentLayoutInsetCorner {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TopLeft => f.write_str("top_left"),
            Self::TopRight => f.write_str("top_right"),
            Self::BottomLeft => f.write_str("bottom_left"),
            Self::BottomRight => f.write_str("bottom_right"),
        }
    }
}
impl ::std::str::FromStr for VideoSegmentLayoutInsetCorner {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "top_left" => Ok(Self::TopLeft),
            "top_right" => Ok(Self::TopRight),
            "bottom_left" => Ok(Self::BottomLeft),
            "bottom_right" => Ok(Self::BottomRight),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for VideoSegmentLayoutInsetCorner {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for VideoSegmentLayoutInsetCorner {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for VideoSegmentLayoutInsetCorner {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`VideoSegmentLayoutPunchesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"zoom\""]
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
#[doc = "    \"zoom\": {"]
#[doc = "      \"description\": \"How much closer, in percent.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 200.0,"]
#[doc = "      \"minimum\": 105.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VideoSegmentLayoutPunchesItem {
    pub end_ticks: ::std::num::NonZeroU64,
    pub start_ticks: u64,
    #[doc = "How much closer, in percent."]
    pub zoom: i64,
}
impl VideoSegmentLayoutPunchesItem {
    pub fn builder() -> builder::VideoSegmentLayoutPunchesItem {
        Default::default()
    }
}
#[doc = "`VideoSegmentLayoutSecondaryCropPathItem`"]
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
#[doc = "    \"easing\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"linear\","]
#[doc = "        \"ease_in\","]
#[doc = "        \"ease_out\","]
#[doc = "        \"ease_in_out\""]
#[doc = "      ]"]
#[doc = "    },"]
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
pub struct VideoSegmentLayoutSecondaryCropPathItem {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub easing: ::std::option::Option<VideoSegmentLayoutSecondaryCropPathItemEasing>,
    pub rect: CropRect,
    pub t_ticks: u64,
}
impl VideoSegmentLayoutSecondaryCropPathItem {
    pub fn builder() -> builder::VideoSegmentLayoutSecondaryCropPathItem {
        Default::default()
    }
}
#[doc = "`VideoSegmentLayoutSecondaryCropPathItemEasing`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"linear\","]
#[doc = "    \"ease_in\","]
#[doc = "    \"ease_out\","]
#[doc = "    \"ease_in_out\""]
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
pub enum VideoSegmentLayoutSecondaryCropPathItemEasing {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "ease_in")]
    EaseIn,
    #[serde(rename = "ease_out")]
    EaseOut,
    #[serde(rename = "ease_in_out")]
    EaseInOut,
}
impl ::std::fmt::Display for VideoSegmentLayoutSecondaryCropPathItemEasing {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Linear => f.write_str("linear"),
            Self::EaseIn => f.write_str("ease_in"),
            Self::EaseOut => f.write_str("ease_out"),
            Self::EaseInOut => f.write_str("ease_in_out"),
        }
    }
}
impl ::std::str::FromStr for VideoSegmentLayoutSecondaryCropPathItemEasing {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "linear" => Ok(Self::Linear),
            "ease_in" => Ok(Self::EaseIn),
            "ease_out" => Ok(Self::EaseOut),
            "ease_in_out" => Ok(Self::EaseInOut),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for VideoSegmentLayoutSecondaryCropPathItemEasing {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String>
    for VideoSegmentLayoutSecondaryCropPathItemEasing
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String>
    for VideoSegmentLayoutSecondaryCropPathItemEasing
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "picture_in_picture draws the full picture (the crop path, or the whole frame when it is empty) with the secondary path inset in one corner."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"picture_in_picture draws the full picture (the crop path, or the whole frame when it is empty) with the secondary path inset in one corner.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"speaker_fill\","]
#[doc = "    \"fit\","]
#[doc = "    \"two_up\","]
#[doc = "    \"picture_in_picture\""]
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
    #[serde(rename = "two_up")]
    TwoUp,
    #[serde(rename = "picture_in_picture")]
    PictureInPicture,
}
impl ::std::fmt::Display for VideoSegmentLayoutState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::SpeakerFill => f.write_str("speaker_fill"),
            Self::Fit => f.write_str("fit"),
            Self::TwoUp => f.write_str("two_up"),
            Self::PictureInPicture => f.write_str("picture_in_picture"),
        }
    }
}
impl ::std::str::FromStr for VideoSegmentLayoutState {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "speaker_fill" => Ok(Self::SpeakerFill),
            "fit" => Ok(Self::Fit),
            "two_up" => Ok(Self::TwoUp),
            "picture_in_picture" => Ok(Self::PictureInPicture),
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
        position: ::std::result::Result<
            ::std::option::Option<super::CaptionPosition>,
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
                position: Ok(Default::default()),
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
        pub fn position<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CaptionPosition>>,
            T::Error: ::std::fmt::Display,
        {
            self.position = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for position: {e}"));
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
                position: value.position?,
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
                position: Ok(value.position),
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
        emphasis: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        text: ::std::result::Result<super::CaptionCueLinesItemWordsItemText, ::std::string::String>,
        word_id: ::std::result::Result<
            ::std::option::Option<super::CaptionCueLinesItemWordsItemWordId>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for CaptionCueLinesItemWordsItem {
        fn default() -> Self {
            Self {
                emphasis: Ok(Default::default()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                text: Err("no value supplied for text".to_string()),
                word_id: Ok(Default::default()),
            }
        }
    }
    impl CaptionCueLinesItemWordsItem {
        pub fn emphasis<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.emphasis = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for emphasis: {e}"));
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
        pub fn word_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::option::Option<super::CaptionCueLinesItemWordsItemWordId>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.word_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for word_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionCueLinesItemWordsItem> for super::CaptionCueLinesItemWordsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCueLinesItemWordsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                emphasis: value.emphasis?,
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
                text: value.text?,
                word_id: value.word_id?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCueLinesItemWordsItem> for CaptionCueLinesItemWordsItem {
        fn from(value: super::CaptionCueLinesItemWordsItem) -> Self {
            Self {
                emphasis: Ok(value.emphasis),
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
                text: Ok(value.text),
                word_id: Ok(value.word_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CaptionPosition {
        x: ::std::result::Result<i64, ::std::string::String>,
        y: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for CaptionPosition {
        fn default() -> Self {
            Self {
                x: Err("no value supplied for x".to_string()),
                y: Err("no value supplied for y".to_string()),
            }
        }
    }
    impl CaptionPosition {
        pub fn x<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.x = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for x: {e}"));
            self
        }
        pub fn y<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.y = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for y: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionPosition> for super::CaptionPosition {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionPosition,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                x: value.x?,
                y: value.y?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionPosition> for CaptionPosition {
        fn from(value: super::CaptionPosition) -> Self {
            Self {
                x: Ok(value.x),
                y: Ok(value.y),
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
        overlays: ::std::result::Result<::std::vec::Vec<super::Overlay>, ::std::string::String>,
        rationale: ::std::result::Result<
            ::std::option::Option<super::EditIrRationale>,
            ::std::string::String,
        >,
        timebase: ::std::result::Result<super::EditIrTimebase, ::std::string::String>,
        title:
            ::std::result::Result<::std::option::Option<super::EditIrTitle>, ::std::string::String>,
        version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        video: ::std::result::Result<super::EditIrVideo, ::std::string::String>,
    }
    impl ::std::default::Default for EditIr {
        fn default() -> Self {
            Self {
                assets: Ok(Default::default()),
                audio: Err("no value supplied for audio".to_string()),
                captions: Err("no value supplied for captions".to_string()),
                overlays: Ok(Default::default()),
                rationale: Ok(Default::default()),
                timebase: Err("no value supplied for timebase".to_string()),
                title: Ok(Default::default()),
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
        pub fn overlays<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Overlay>>,
            T::Error: ::std::fmt::Display,
        {
            self.overlays = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for overlays: {e}"));
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
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrTitle>>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {e}"));
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
                overlays: value.overlays?,
                rationale: value.rationale?,
                timebase: value.timebase?,
                title: value.title?,
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
                overlays: Ok(value.overlays),
                rationale: Ok(value.rationale),
                timebase: Ok(value.timebase),
                title: Ok(value.title),
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
        burn_in: ::std::result::Result<::std::vec::Vec<super::CaptionCue>, ::std::string::String>,
        cues: ::std::result::Result<::std::vec::Vec<super::CaptionCue>, ::std::string::String>,
        options: ::std::result::Result<
            ::std::option::Option<super::EditIrCaptionsOptions>,
            ::std::string::String,
        >,
        style_ref: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrCaptions {
        fn default() -> Self {
            Self {
                burn_in: Ok(Default::default()),
                cues: Ok(Default::default()),
                options: Ok(Default::default()),
                style_ref: Err("no value supplied for style_ref".to_string()),
            }
        }
    }
    impl EditIrCaptions {
        pub fn burn_in<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CaptionCue>>,
            T::Error: ::std::fmt::Display,
        {
            self.burn_in = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for burn_in: {e}"));
            self
        }
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
        pub fn options<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrCaptionsOptions>>,
            T::Error: ::std::fmt::Display,
        {
            self.options = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for options: {e}"));
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
                burn_in: value.burn_in?,
                cues: value.cues?,
                options: value.options?,
                style_ref: value.style_ref?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrCaptions> for EditIrCaptions {
        fn from(value: super::EditIrCaptions) -> Self {
            Self {
                burn_in: Ok(value.burn_in),
                cues: Ok(value.cues),
                options: Ok(value.options),
                style_ref: Ok(value.style_ref),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditIrCaptionsOptions {
        accent: ::std::result::Result<
            ::std::option::Option<super::EditIrCaptionsOptionsAccent>,
            ::std::string::String,
        >,
        font_family: ::std::result::Result<
            ::std::option::Option<super::EditIrCaptionsOptionsFontFamily>,
            ::std::string::String,
        >,
        font_size: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        highlight_spoken_word:
            ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        highlight_style: ::std::result::Result<
            ::std::option::Option<super::EditIrCaptionsOptionsHighlightStyle>,
            ::std::string::String,
        >,
        outline: ::std::result::Result<
            ::std::option::Option<super::EditIrCaptionsOptionsOutline>,
            ::std::string::String,
        >,
        outline_width: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        plate_opacity: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        position: ::std::result::Result<
            ::std::option::Option<super::CaptionPosition>,
            ::std::string::String,
        >,
        shadow_depth: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        spoken: ::std::result::Result<
            ::std::option::Option<super::EditIrCaptionsOptionsSpoken>,
            ::std::string::String,
        >,
        text_case: ::std::result::Result<
            ::std::option::Option<super::EditIrCaptionsOptionsTextCase>,
            ::std::string::String,
        >,
        unspoken: ::std::result::Result<
            ::std::option::Option<super::EditIrCaptionsOptionsUnspoken>,
            ::std::string::String,
        >,
        words_on_screen: ::std::result::Result<
            ::std::option::Option<::std::num::NonZeroU64>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for EditIrCaptionsOptions {
        fn default() -> Self {
            Self {
                accent: Ok(Default::default()),
                font_family: Ok(Default::default()),
                font_size: Ok(Default::default()),
                highlight_spoken_word: Ok(Default::default()),
                highlight_style: Ok(Default::default()),
                outline: Ok(Default::default()),
                outline_width: Ok(Default::default()),
                plate_opacity: Ok(Default::default()),
                position: Ok(Default::default()),
                shadow_depth: Ok(Default::default()),
                spoken: Ok(Default::default()),
                text_case: Ok(Default::default()),
                unspoken: Ok(Default::default()),
                words_on_screen: Ok(Default::default()),
            }
        }
    }
    impl EditIrCaptionsOptions {
        pub fn accent<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrCaptionsOptionsAccent>>,
            T::Error: ::std::fmt::Display,
        {
            self.accent = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for accent: {e}"));
            self
        }
        pub fn font_family<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::option::Option<super::EditIrCaptionsOptionsFontFamily>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.font_family = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for font_family: {e}"));
            self
        }
        pub fn font_size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.font_size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for font_size: {e}"));
            self
        }
        pub fn highlight_spoken_word<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.highlight_spoken_word = value.try_into().map_err(|e| {
                format!("error converting supplied value for highlight_spoken_word: {e}")
            });
            self
        }
        pub fn highlight_style<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::option::Option<super::EditIrCaptionsOptionsHighlightStyle>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.highlight_style = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for highlight_style: {e}"));
            self
        }
        pub fn outline<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrCaptionsOptionsOutline>>,
            T::Error: ::std::fmt::Display,
        {
            self.outline = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outline: {e}"));
            self
        }
        pub fn outline_width<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.outline_width = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outline_width: {e}"));
            self
        }
        pub fn plate_opacity<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.plate_opacity = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for plate_opacity: {e}"));
            self
        }
        pub fn position<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CaptionPosition>>,
            T::Error: ::std::fmt::Display,
        {
            self.position = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for position: {e}"));
            self
        }
        pub fn shadow_depth<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.shadow_depth = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shadow_depth: {e}"));
            self
        }
        pub fn spoken<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrCaptionsOptionsSpoken>>,
            T::Error: ::std::fmt::Display,
        {
            self.spoken = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for spoken: {e}"));
            self
        }
        pub fn text_case<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrCaptionsOptionsTextCase>>,
            T::Error: ::std::fmt::Display,
        {
            self.text_case = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text_case: {e}"));
            self
        }
        pub fn unspoken<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrCaptionsOptionsUnspoken>>,
            T::Error: ::std::fmt::Display,
        {
            self.unspoken = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for unspoken: {e}"));
            self
        }
        pub fn words_on_screen<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::num::NonZeroU64>>,
            T::Error: ::std::fmt::Display,
        {
            self.words_on_screen = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for words_on_screen: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditIrCaptionsOptions> for super::EditIrCaptionsOptions {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditIrCaptionsOptions,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                accent: value.accent?,
                font_family: value.font_family?,
                font_size: value.font_size?,
                highlight_spoken_word: value.highlight_spoken_word?,
                highlight_style: value.highlight_style?,
                outline: value.outline?,
                outline_width: value.outline_width?,
                plate_opacity: value.plate_opacity?,
                position: value.position?,
                shadow_depth: value.shadow_depth?,
                spoken: value.spoken?,
                text_case: value.text_case?,
                unspoken: value.unspoken?,
                words_on_screen: value.words_on_screen?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrCaptionsOptions> for EditIrCaptionsOptions {
        fn from(value: super::EditIrCaptionsOptions) -> Self {
            Self {
                accent: Ok(value.accent),
                font_family: Ok(value.font_family),
                font_size: Ok(value.font_size),
                highlight_spoken_word: Ok(value.highlight_spoken_word),
                highlight_style: Ok(value.highlight_style),
                outline: Ok(value.outline),
                outline_width: Ok(value.outline_width),
                plate_opacity: Ok(value.plate_opacity),
                position: Ok(value.position),
                shadow_depth: Ok(value.shadow_depth),
                spoken: Ok(value.spoken),
                text_case: Ok(value.text_case),
                unspoken: Ok(value.unspoken),
                words_on_screen: Ok(value.words_on_screen),
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
        shape: ::std::result::Result<
            ::std::option::Option<super::EditIrVideoShape>,
            ::std::string::String,
        >,
        transition_ticks: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
    }
    impl ::std::default::Default for EditIrVideo {
        fn default() -> Self {
            Self {
                segments: Ok(Default::default()),
                shape: Ok(Default::default()),
                transition_ticks: Ok(Default::default()),
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
        pub fn shape<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditIrVideoShape>>,
            T::Error: ::std::fmt::Display,
        {
            self.shape = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shape: {e}"));
            self
        }
        pub fn transition_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.transition_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transition_ticks: {e}"));
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
                shape: value.shape?,
                transition_ticks: value.transition_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::EditIrVideo> for EditIrVideo {
        fn from(value: super::EditIrVideo) -> Self {
            Self {
                segments: Ok(value.segments),
                shape: Ok(value.shape),
                transition_ticks: Ok(value.transition_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Overlay {
        content: ::std::result::Result<super::OverlayContent, ::std::string::String>,
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        overlay_id: ::std::result::Result<super::OverlayOverlayId, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Overlay {
        fn default() -> Self {
            Self {
                content: Err("no value supplied for content".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                overlay_id: Err("no value supplied for overlay_id".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Overlay {
        pub fn content<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::OverlayContent>,
            T::Error: ::std::fmt::Display,
        {
            self.content = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for content: {e}"));
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
        pub fn overlay_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::OverlayOverlayId>,
            T::Error: ::std::fmt::Display,
        {
            self.overlay_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for overlay_id: {e}"));
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
    impl ::std::convert::TryFrom<Overlay> for super::Overlay {
        type Error = super::error::ConversionError;
        fn try_from(value: Overlay) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                content: value.content?,
                end_ticks: value.end_ticks?,
                overlay_id: value.overlay_id?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Overlay> for Overlay {
        fn from(value: super::Overlay) -> Self {
            Self {
                content: Ok(value.content),
                end_ticks: Ok(value.end_ticks),
                overlay_id: Ok(value.overlay_id),
                start_ticks: Ok(value.start_ticks),
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
        background: ::std::result::Result<
            ::std::option::Option<super::VideoSegmentLayoutBackground>,
            ::std::string::String,
        >,
        crop_path: ::std::result::Result<
            ::std::vec::Vec<super::VideoSegmentLayoutCropPathItem>,
            ::std::string::String,
        >,
        inset: ::std::result::Result<
            ::std::option::Option<super::VideoSegmentLayoutInset>,
            ::std::string::String,
        >,
        punches: ::std::result::Result<
            ::std::vec::Vec<super::VideoSegmentLayoutPunchesItem>,
            ::std::string::String,
        >,
        secondary_crop_path: ::std::result::Result<
            ::std::vec::Vec<super::VideoSegmentLayoutSecondaryCropPathItem>,
            ::std::string::String,
        >,
        split: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        state: ::std::result::Result<super::VideoSegmentLayoutState, ::std::string::String>,
        zoom: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSegmentLayout {
        fn default() -> Self {
            Self {
                background: Ok(Default::default()),
                crop_path: Ok(Default::default()),
                inset: Ok(Default::default()),
                punches: Ok(Default::default()),
                secondary_crop_path: Ok(Default::default()),
                split: Ok(Default::default()),
                state: Err("no value supplied for state".to_string()),
                zoom: Ok(Default::default()),
            }
        }
    }
    impl VideoSegmentLayout {
        pub fn background<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::VideoSegmentLayoutBackground>>,
            T::Error: ::std::fmt::Display,
        {
            self.background = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for background: {e}"));
            self
        }
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
        pub fn inset<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::VideoSegmentLayoutInset>>,
            T::Error: ::std::fmt::Display,
        {
            self.inset = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inset: {e}"));
            self
        }
        pub fn punches<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::VideoSegmentLayoutPunchesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.punches = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for punches: {e}"));
            self
        }
        pub fn secondary_crop_path<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<super::VideoSegmentLayoutSecondaryCropPathItem>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.secondary_crop_path = value.try_into().map_err(|e| {
                format!("error converting supplied value for secondary_crop_path: {e}")
            });
            self
        }
        pub fn split<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.split = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for split: {e}"));
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
        pub fn zoom<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.zoom = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for zoom: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<VideoSegmentLayout> for super::VideoSegmentLayout {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VideoSegmentLayout,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                background: value.background?,
                crop_path: value.crop_path?,
                inset: value.inset?,
                punches: value.punches?,
                secondary_crop_path: value.secondary_crop_path?,
                split: value.split?,
                state: value.state?,
                zoom: value.zoom?,
            })
        }
    }
    impl ::std::convert::From<super::VideoSegmentLayout> for VideoSegmentLayout {
        fn from(value: super::VideoSegmentLayout) -> Self {
            Self {
                background: Ok(value.background),
                crop_path: Ok(value.crop_path),
                inset: Ok(value.inset),
                punches: Ok(value.punches),
                secondary_crop_path: Ok(value.secondary_crop_path),
                split: Ok(value.split),
                state: Ok(value.state),
                zoom: Ok(value.zoom),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VideoSegmentLayoutCropPathItem {
        easing: ::std::result::Result<
            ::std::option::Option<super::VideoSegmentLayoutCropPathItemEasing>,
            ::std::string::String,
        >,
        rect: ::std::result::Result<super::CropRect, ::std::string::String>,
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSegmentLayoutCropPathItem {
        fn default() -> Self {
            Self {
                easing: Ok(Default::default()),
                rect: Err("no value supplied for rect".to_string()),
                t_ticks: Err("no value supplied for t_ticks".to_string()),
            }
        }
    }
    impl VideoSegmentLayoutCropPathItem {
        pub fn easing<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::option::Option<super::VideoSegmentLayoutCropPathItemEasing>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.easing = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for easing: {e}"));
            self
        }
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
                easing: value.easing?,
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
                easing: Ok(value.easing),
                rect: Ok(value.rect),
                t_ticks: Ok(value.t_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VideoSegmentLayoutInset {
        corner: ::std::result::Result<super::VideoSegmentLayoutInsetCorner, ::std::string::String>,
        size: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSegmentLayoutInset {
        fn default() -> Self {
            Self {
                corner: Err("no value supplied for corner".to_string()),
                size: Err("no value supplied for size".to_string()),
            }
        }
    }
    impl VideoSegmentLayoutInset {
        pub fn corner<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::VideoSegmentLayoutInsetCorner>,
            T::Error: ::std::fmt::Display,
        {
            self.corner = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for corner: {e}"));
            self
        }
        pub fn size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for size: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<VideoSegmentLayoutInset> for super::VideoSegmentLayoutInset {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VideoSegmentLayoutInset,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                corner: value.corner?,
                size: value.size?,
            })
        }
    }
    impl ::std::convert::From<super::VideoSegmentLayoutInset> for VideoSegmentLayoutInset {
        fn from(value: super::VideoSegmentLayoutInset) -> Self {
            Self {
                corner: Ok(value.corner),
                size: Ok(value.size),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VideoSegmentLayoutPunchesItem {
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        zoom: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSegmentLayoutPunchesItem {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                zoom: Err("no value supplied for zoom".to_string()),
            }
        }
    }
    impl VideoSegmentLayoutPunchesItem {
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
        pub fn zoom<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.zoom = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for zoom: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<VideoSegmentLayoutPunchesItem>
        for super::VideoSegmentLayoutPunchesItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VideoSegmentLayoutPunchesItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
                zoom: value.zoom?,
            })
        }
    }
    impl ::std::convert::From<super::VideoSegmentLayoutPunchesItem> for VideoSegmentLayoutPunchesItem {
        fn from(value: super::VideoSegmentLayoutPunchesItem) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
                zoom: Ok(value.zoom),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VideoSegmentLayoutSecondaryCropPathItem {
        easing: ::std::result::Result<
            ::std::option::Option<super::VideoSegmentLayoutSecondaryCropPathItemEasing>,
            ::std::string::String,
        >,
        rect: ::std::result::Result<super::CropRect, ::std::string::String>,
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSegmentLayoutSecondaryCropPathItem {
        fn default() -> Self {
            Self {
                easing: Ok(Default::default()),
                rect: Err("no value supplied for rect".to_string()),
                t_ticks: Err("no value supplied for t_ticks".to_string()),
            }
        }
    }
    impl VideoSegmentLayoutSecondaryCropPathItem {
        pub fn easing<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::option::Option<super::VideoSegmentLayoutSecondaryCropPathItemEasing>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.easing = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for easing: {e}"));
            self
        }
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
    impl ::std::convert::TryFrom<VideoSegmentLayoutSecondaryCropPathItem>
        for super::VideoSegmentLayoutSecondaryCropPathItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VideoSegmentLayoutSecondaryCropPathItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                easing: value.easing?,
                rect: value.rect?,
                t_ticks: value.t_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::VideoSegmentLayoutSecondaryCropPathItem>
        for VideoSegmentLayoutSecondaryCropPathItem
    {
        fn from(value: super::VideoSegmentLayoutSecondaryCropPathItem) -> Self {
            Self {
                easing: Ok(value.easing),
                rect: Ok(value.rect),
                t_ticks: Ok(value.t_ticks),
            }
        }
    }
}
