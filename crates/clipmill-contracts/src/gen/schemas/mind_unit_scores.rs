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
#[doc = "`Level`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"none\","]
#[doc = "    \"fair\","]
#[doc = "    \"good\","]
#[doc = "    \"strong\""]
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
pub enum Level {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "fair")]
    Fair,
    #[serde(rename = "good")]
    Good,
    #[serde(rename = "strong")]
    Strong,
}
impl ::std::fmt::Display for Level {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::None => f.write_str("none"),
            Self::Fair => f.write_str("fair"),
            Self::Good => f.write_str("good"),
            Self::Strong => f.write_str("strong"),
        }
    }
}
impl ::std::str::FromStr for Level {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "none" => Ok(Self::None),
            "fair" => Ok(Self::Fair),
            "good" => Ok(Self::Good),
            "strong" => Ok(Self::Strong),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Level {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Level {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Level {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "An editor's markup of every sentence in a recording: which lines would open a clip well, which lines land a thought, which ranges a tight cut can leave out (a tangent, a retake, a false start, filler, an aside, an ad read) and which sentences cannot stand without an earlier one. The model marks only what stands out, a window of the annotated script at a time, citing sentences it was shown; code writes the row for every sentence, so a sentence with nothing marked is an ordinary line, not a missing one. Windows the model could not mark are listed under `unscored`. This is what lets the cutter tighten a moment and start a clip on its strongest line."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.mind.unit_scores.v1.json\","]
#[doc = "  \"title\": \"MindUnitScores\","]
#[doc = "  \"description\": \"An editor's markup of every sentence in a recording: which lines would open a clip well, which lines land a thought, which ranges a tight cut can leave out (a tangent, a retake, a false start, filler, an aside, an ad read) and which sentences cannot stand without an earlier one. The model marks only what stands out, a window of the annotated script at a time, citing sentences it was shown; code writes the row for every sentence, so a sentence with nothing marked is an ordinary line, not a missing one. Windows the model could not mark are listed under `unscored`. This is what lets the cutter tighten a moment and start a clip on its strongest line.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"script\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"units\","]
#[doc = "    \"unscored\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"producer\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"implementation\","]
#[doc = "        \"model\","]
#[doc = "        \"prompt_version\","]
#[doc = "        \"stage\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"implementation\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"model\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"prompt_version\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"stage\": {"]
#[doc = "          \"const\": \"mind-score\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.mind.unit_scores.v1\""]
#[doc = "    },"]
#[doc = "    \"script\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"digest\","]
#[doc = "        \"level\","]
#[doc = "        \"windows\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"digest\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"level\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"full\","]
#[doc = "            \"compact\","]
#[doc = "            \"outline\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"windows\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"units\": {"]
#[doc = "      \"description\": \"One row per sentence, in recording order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/row\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"unscored\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"first_unit\","]
#[doc = "          \"last_unit\","]
#[doc = "          \"message\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"first_unit\": {"]
#[doc = "            \"$ref\": \"#/$defs/unit\""]
#[doc = "          },"]
#[doc = "          \"last_unit\": {"]
#[doc = "            \"$ref\": \"#/$defs/unit\""]
#[doc = "          },"]
#[doc = "          \"message\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"maxLength\": 400,"]
#[doc = "            \"minLength\": 1"]
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
pub struct MindUnitScores {
    pub producer: MindUnitScoresProducer,
    pub schema_version: ::serde_json::Value,
    pub script: MindUnitScoresScript,
    pub source_fingerprint: Sha256,
    #[doc = "One row per sentence, in recording order."]
    pub units: ::std::vec::Vec<Row>,
    pub unscored: ::std::vec::Vec<MindUnitScoresUnscoredItem>,
}
impl MindUnitScores {
    pub fn builder() -> builder::MindUnitScores {
        Default::default()
    }
}
#[doc = "`MindUnitScoresProducer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"implementation\","]
#[doc = "    \"model\","]
#[doc = "    \"prompt_version\","]
#[doc = "    \"stage\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"implementation\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"model\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"prompt_version\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"stage\": {"]
#[doc = "      \"const\": \"mind-score\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindUnitScoresProducer {
    pub implementation: MindUnitScoresProducerImplementation,
    pub model: MindUnitScoresProducerModel,
    pub prompt_version: MindUnitScoresProducerPromptVersion,
    pub stage: ::serde_json::Value,
}
impl MindUnitScoresProducer {
    pub fn builder() -> builder::MindUnitScoresProducer {
        Default::default()
    }
}
#[doc = "`MindUnitScoresProducerImplementation`"]
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
pub struct MindUnitScoresProducerImplementation(::std::string::String);
impl ::std::ops::Deref for MindUnitScoresProducerImplementation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindUnitScoresProducerImplementation> for ::std::string::String {
    fn from(value: MindUnitScoresProducerImplementation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindUnitScoresProducerImplementation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindUnitScoresProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindUnitScoresProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindUnitScoresProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindUnitScoresProducerImplementation {
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
#[doc = "`MindUnitScoresProducerModel`"]
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
pub struct MindUnitScoresProducerModel(::std::string::String);
impl ::std::ops::Deref for MindUnitScoresProducerModel {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindUnitScoresProducerModel> for ::std::string::String {
    fn from(value: MindUnitScoresProducerModel) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindUnitScoresProducerModel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindUnitScoresProducerModel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindUnitScoresProducerModel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindUnitScoresProducerModel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindUnitScoresProducerModel {
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
#[doc = "`MindUnitScoresProducerPromptVersion`"]
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
pub struct MindUnitScoresProducerPromptVersion(::std::string::String);
impl ::std::ops::Deref for MindUnitScoresProducerPromptVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindUnitScoresProducerPromptVersion> for ::std::string::String {
    fn from(value: MindUnitScoresProducerPromptVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindUnitScoresProducerPromptVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindUnitScoresProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindUnitScoresProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindUnitScoresProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindUnitScoresProducerPromptVersion {
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
#[doc = "`MindUnitScoresScript`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"digest\","]
#[doc = "    \"level\","]
#[doc = "    \"windows\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"digest\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"level\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"full\","]
#[doc = "        \"compact\","]
#[doc = "        \"outline\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"windows\": {"]
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
pub struct MindUnitScoresScript {
    pub digest: Sha256,
    pub level: MindUnitScoresScriptLevel,
    pub windows: ::std::num::NonZeroU64,
}
impl MindUnitScoresScript {
    pub fn builder() -> builder::MindUnitScoresScript {
        Default::default()
    }
}
#[doc = "`MindUnitScoresScriptLevel`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"full\","]
#[doc = "    \"compact\","]
#[doc = "    \"outline\""]
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
pub enum MindUnitScoresScriptLevel {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "compact")]
    Compact,
    #[serde(rename = "outline")]
    Outline,
}
impl ::std::fmt::Display for MindUnitScoresScriptLevel {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Full => f.write_str("full"),
            Self::Compact => f.write_str("compact"),
            Self::Outline => f.write_str("outline"),
        }
    }
}
impl ::std::str::FromStr for MindUnitScoresScriptLevel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "full" => Ok(Self::Full),
            "compact" => Ok(Self::Compact),
            "outline" => Ok(Self::Outline),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindUnitScoresScriptLevel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindUnitScoresScriptLevel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindUnitScoresScriptLevel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindUnitScoresUnscoredItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"first_unit\","]
#[doc = "    \"last_unit\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"first_unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    },"]
#[doc = "    \"last_unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 400,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindUnitScoresUnscoredItem {
    pub first_unit: Unit,
    pub last_unit: Unit,
    pub message: MindUnitScoresUnscoredItemMessage,
}
impl MindUnitScoresUnscoredItem {
    pub fn builder() -> builder::MindUnitScoresUnscoredItem {
        Default::default()
    }
}
#[doc = "`MindUnitScoresUnscoredItemMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 400,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindUnitScoresUnscoredItemMessage(::std::string::String);
impl ::std::ops::Deref for MindUnitScoresUnscoredItemMessage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindUnitScoresUnscoredItemMessage> for ::std::string::String {
    fn from(value: MindUnitScoresUnscoredItemMessage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindUnitScoresUnscoredItemMessage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 400usize {
            return Err("longer than 400 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindUnitScoresUnscoredItemMessage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindUnitScoresUnscoredItemMessage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindUnitScoresUnscoredItemMessage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindUnitScoresUnscoredItemMessage {
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
#[doc = "`Row`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"hook\","]
#[doc = "    \"landing\","]
#[doc = "    \"trim\","]
#[doc = "    \"unit\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"hook\": {"]
#[doc = "      \"description\": \"How well the line would open a clip for someone who has heard nothing before it, from `none` to `strong`: a line people stop scrolling for.\","]
#[doc = "      \"$ref\": \"#/$defs/level\""]
#[doc = "    },"]
#[doc = "    \"landing\": {"]
#[doc = "      \"description\": \"How well the line ends a thought, from `none` (it leaves things hanging) to `strong` (a clean conclusion, punchline or turn).\","]
#[doc = "      \"$ref\": \"#/$defs/level\""]
#[doc = "    },"]
#[doc = "    \"needs\": {"]
#[doc = "      \"description\": \"An earlier sentence this one cannot be understood without: the thing a pronoun points at, the question it answers.\","]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    },"]
#[doc = "    \"trim\": {"]
#[doc = "      \"description\": \"`keep`, or why a tight cut may leave the line out.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"keep\","]
#[doc = "        \"tangent\","]
#[doc = "        \"retake\","]
#[doc = "        \"false_start\","]
#[doc = "        \"filler\","]
#[doc = "        \"aside\","]
#[doc = "        \"ad_read\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[doc = "How well the line would open a clip for someone who has heard nothing before it, from `none` to `strong`: a line people stop scrolling for."]
    pub hook: Level,
    #[doc = "How well the line ends a thought, from `none` (it leaves things hanging) to `strong` (a clean conclusion, punchline or turn)."]
    pub landing: Level,
    #[doc = "An earlier sentence this one cannot be understood without: the thing a pronoun points at, the question it answers."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub needs: ::std::option::Option<Unit>,
    #[doc = "`keep`, or why a tight cut may leave the line out."]
    pub trim: RowTrim,
    pub unit: Unit,
}
impl Row {
    pub fn builder() -> builder::Row {
        Default::default()
    }
}
#[doc = "`keep`, or why a tight cut may leave the line out."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"`keep`, or why a tight cut may leave the line out.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"keep\","]
#[doc = "    \"tangent\","]
#[doc = "    \"retake\","]
#[doc = "    \"false_start\","]
#[doc = "    \"filler\","]
#[doc = "    \"aside\","]
#[doc = "    \"ad_read\""]
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
pub enum RowTrim {
    #[serde(rename = "keep")]
    Keep,
    #[serde(rename = "tangent")]
    Tangent,
    #[serde(rename = "retake")]
    Retake,
    #[serde(rename = "false_start")]
    FalseStart,
    #[serde(rename = "filler")]
    Filler,
    #[serde(rename = "aside")]
    Aside,
    #[serde(rename = "ad_read")]
    AdRead,
}
impl ::std::fmt::Display for RowTrim {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Keep => f.write_str("keep"),
            Self::Tangent => f.write_str("tangent"),
            Self::Retake => f.write_str("retake"),
            Self::FalseStart => f.write_str("false_start"),
            Self::Filler => f.write_str("filler"),
            Self::Aside => f.write_str("aside"),
            Self::AdRead => f.write_str("ad_read"),
        }
    }
}
impl ::std::str::FromStr for RowTrim {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "keep" => Ok(Self::Keep),
            "tangent" => Ok(Self::Tangent),
            "retake" => Ok(Self::Retake),
            "false_start" => Ok(Self::FalseStart),
            "filler" => Ok(Self::Filler),
            "aside" => Ok(Self::Aside),
            "ad_read" => Ok(Self::AdRead),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for RowTrim {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RowTrim {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RowTrim {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
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
#[doc = "`Unit`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^s(0|[1-9][0-9]{0,8})$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Unit(::std::string::String);
impl ::std::ops::Deref for Unit {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Unit> for ::std::string::String {
    fn from(value: Unit) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Unit {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^s(0|[1-9][0-9]{0,8})$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^s(0|[1-9][0-9]{0,8})$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for Unit {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Unit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Unit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Unit {
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
    pub struct MindUnitScores {
        producer: ::std::result::Result<super::MindUnitScoresProducer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        script: ::std::result::Result<super::MindUnitScoresScript, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        units: ::std::result::Result<::std::vec::Vec<super::Row>, ::std::string::String>,
        unscored: ::std::result::Result<
            ::std::vec::Vec<super::MindUnitScoresUnscoredItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for MindUnitScores {
        fn default() -> Self {
            Self {
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                script: Err("no value supplied for script".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                units: Err("no value supplied for units".to_string()),
                unscored: Err("no value supplied for unscored".to_string()),
            }
        }
    }
    impl MindUnitScores {
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindUnitScoresProducer>,
            T::Error: ::std::fmt::Display,
        {
            self.producer = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for producer: {e}"));
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
        pub fn script<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindUnitScoresScript>,
            T::Error: ::std::fmt::Display,
        {
            self.script = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for script: {e}"));
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
        pub fn units<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Row>>,
            T::Error: ::std::fmt::Display,
        {
            self.units = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for units: {e}"));
            self
        }
        pub fn unscored<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindUnitScoresUnscoredItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.unscored = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for unscored: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindUnitScores> for super::MindUnitScores {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindUnitScores,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                producer: value.producer?,
                schema_version: value.schema_version?,
                script: value.script?,
                source_fingerprint: value.source_fingerprint?,
                units: value.units?,
                unscored: value.unscored?,
            })
        }
    }
    impl ::std::convert::From<super::MindUnitScores> for MindUnitScores {
        fn from(value: super::MindUnitScores) -> Self {
            Self {
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                script: Ok(value.script),
                source_fingerprint: Ok(value.source_fingerprint),
                units: Ok(value.units),
                unscored: Ok(value.unscored),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindUnitScoresProducer {
        implementation: ::std::result::Result<
            super::MindUnitScoresProducerImplementation,
            ::std::string::String,
        >,
        model: ::std::result::Result<super::MindUnitScoresProducerModel, ::std::string::String>,
        prompt_version: ::std::result::Result<
            super::MindUnitScoresProducerPromptVersion,
            ::std::string::String,
        >,
        stage: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for MindUnitScoresProducer {
        fn default() -> Self {
            Self {
                implementation: Err("no value supplied for implementation".to_string()),
                model: Err("no value supplied for model".to_string()),
                prompt_version: Err("no value supplied for prompt_version".to_string()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl MindUnitScoresProducer {
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindUnitScoresProducerImplementation>,
            T::Error: ::std::fmt::Display,
        {
            self.implementation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for implementation: {e}"));
            self
        }
        pub fn model<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindUnitScoresProducerModel>,
            T::Error: ::std::fmt::Display,
        {
            self.model = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model: {e}"));
            self
        }
        pub fn prompt_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindUnitScoresProducerPromptVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt_version: {e}"));
            self
        }
        pub fn stage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.stage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stage: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindUnitScoresProducer> for super::MindUnitScoresProducer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindUnitScoresProducer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                implementation: value.implementation?,
                model: value.model?,
                prompt_version: value.prompt_version?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::MindUnitScoresProducer> for MindUnitScoresProducer {
        fn from(value: super::MindUnitScoresProducer) -> Self {
            Self {
                implementation: Ok(value.implementation),
                model: Ok(value.model),
                prompt_version: Ok(value.prompt_version),
                stage: Ok(value.stage),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindUnitScoresScript {
        digest: ::std::result::Result<super::Sha256, ::std::string::String>,
        level: ::std::result::Result<super::MindUnitScoresScriptLevel, ::std::string::String>,
        windows: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for MindUnitScoresScript {
        fn default() -> Self {
            Self {
                digest: Err("no value supplied for digest".to_string()),
                level: Err("no value supplied for level".to_string()),
                windows: Err("no value supplied for windows".to_string()),
            }
        }
    }
    impl MindUnitScoresScript {
        pub fn digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for digest: {e}"));
            self
        }
        pub fn level<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindUnitScoresScriptLevel>,
            T::Error: ::std::fmt::Display,
        {
            self.level = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for level: {e}"));
            self
        }
        pub fn windows<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.windows = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for windows: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindUnitScoresScript> for super::MindUnitScoresScript {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindUnitScoresScript,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                digest: value.digest?,
                level: value.level?,
                windows: value.windows?,
            })
        }
    }
    impl ::std::convert::From<super::MindUnitScoresScript> for MindUnitScoresScript {
        fn from(value: super::MindUnitScoresScript) -> Self {
            Self {
                digest: Ok(value.digest),
                level: Ok(value.level),
                windows: Ok(value.windows),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindUnitScoresUnscoredItem {
        first_unit: ::std::result::Result<super::Unit, ::std::string::String>,
        last_unit: ::std::result::Result<super::Unit, ::std::string::String>,
        message:
            ::std::result::Result<super::MindUnitScoresUnscoredItemMessage, ::std::string::String>,
    }
    impl ::std::default::Default for MindUnitScoresUnscoredItem {
        fn default() -> Self {
            Self {
                first_unit: Err("no value supplied for first_unit".to_string()),
                last_unit: Err("no value supplied for last_unit".to_string()),
                message: Err("no value supplied for message".to_string()),
            }
        }
    }
    impl MindUnitScoresUnscoredItem {
        pub fn first_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.first_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_unit: {e}"));
            self
        }
        pub fn last_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.last_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for last_unit: {e}"));
            self
        }
        pub fn message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindUnitScoresUnscoredItemMessage>,
            T::Error: ::std::fmt::Display,
        {
            self.message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for message: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindUnitScoresUnscoredItem> for super::MindUnitScoresUnscoredItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindUnitScoresUnscoredItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                first_unit: value.first_unit?,
                last_unit: value.last_unit?,
                message: value.message?,
            })
        }
    }
    impl ::std::convert::From<super::MindUnitScoresUnscoredItem> for MindUnitScoresUnscoredItem {
        fn from(value: super::MindUnitScoresUnscoredItem) -> Self {
            Self {
                first_unit: Ok(value.first_unit),
                last_unit: Ok(value.last_unit),
                message: Ok(value.message),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Row {
        hook: ::std::result::Result<super::Level, ::std::string::String>,
        landing: ::std::result::Result<super::Level, ::std::string::String>,
        needs: ::std::result::Result<::std::option::Option<super::Unit>, ::std::string::String>,
        trim: ::std::result::Result<super::RowTrim, ::std::string::String>,
        unit: ::std::result::Result<super::Unit, ::std::string::String>,
    }
    impl ::std::default::Default for Row {
        fn default() -> Self {
            Self {
                hook: Err("no value supplied for hook".to_string()),
                landing: Err("no value supplied for landing".to_string()),
                needs: Ok(Default::default()),
                trim: Err("no value supplied for trim".to_string()),
                unit: Err("no value supplied for unit".to_string()),
            }
        }
    }
    impl Row {
        pub fn hook<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Level>,
            T::Error: ::std::fmt::Display,
        {
            self.hook = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hook: {e}"));
            self
        }
        pub fn landing<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Level>,
            T::Error: ::std::fmt::Display,
        {
            self.landing = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for landing: {e}"));
            self
        }
        pub fn needs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Unit>>,
            T::Error: ::std::fmt::Display,
        {
            self.needs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for needs: {e}"));
            self
        }
        pub fn trim<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RowTrim>,
            T::Error: ::std::fmt::Display,
        {
            self.trim = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for trim: {e}"));
            self
        }
        pub fn unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for unit: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Row> for super::Row {
        type Error = super::error::ConversionError;
        fn try_from(value: Row) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                hook: value.hook?,
                landing: value.landing?,
                needs: value.needs?,
                trim: value.trim?,
                unit: value.unit?,
            })
        }
    }
    impl ::std::convert::From<super::Row> for Row {
        fn from(value: super::Row) -> Self {
            Self {
                hook: Ok(value.hook),
                landing: Ok(value.landing),
                needs: Ok(value.needs),
                trim: Ok(value.trim),
                unit: Ok(value.unit),
            }
        }
    }
}
