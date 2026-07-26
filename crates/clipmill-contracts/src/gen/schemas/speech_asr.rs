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
#[doc = "`AsrSegment`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"hint_end_ticks\","]
#[doc = "    \"hint_start_ticks\","]
#[doc = "    \"index\","]
#[doc = "    \"text\","]
#[doc = "    \"tokens\","]
#[doc = "    \"vad_segment_index\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
#[doc = "    },"]
#[doc = "    \"hint_end_ticks\": {"]
#[doc = "      \"description\": \"Decoder bookkeeping, not word timing.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"hint_start_ticks\": {"]
#[doc = "      \"description\": \"Decoder bookkeeping, not word timing. Useful for batching and progress; never for cutting.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"tokens\": {"]
#[doc = "      \"description\": \"Per-token confidences, kept because a segment average hides the one hallucinated proper noun that ranking would otherwise quote.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/token\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"vad_segment_index\": {"]
#[doc = "      \"description\": \"Which speech segment this decode covered.\","]
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
pub struct AsrSegment {
    pub confidence: Confidence,
    #[doc = "Decoder bookkeeping, not word timing."]
    pub hint_end_ticks: u64,
    #[doc = "Decoder bookkeeping, not word timing. Useful for batching and progress; never for cutting."]
    pub hint_start_ticks: u64,
    pub index: u64,
    pub text: ::std::string::String,
    #[doc = "Per-token confidences, kept because a segment average hides the one hallucinated proper noun that ranking would otherwise quote."]
    pub tokens: ::std::vec::Vec<Token>,
    #[doc = "Which speech segment this decode covered."]
    pub vad_segment_index: u64,
}
impl AsrSegment {
    pub fn builder() -> builder::AsrSegment {
        Default::default()
    }
}
#[doc = "A distribution, not a scalar: p10 is what the segment is worth if the model is having a bad time, and downstream stages that widen uncertainty need both numbers."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A distribution, not a scalar: p10 is what the segment is worth if the model is having a bad time, and downstream stages that widen uncertainty need both numbers.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"p10\","]
#[doc = "    \"p50\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"p10\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"p50\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Confidence {
    pub p10: f64,
    pub p50: f64,
}
impl Confidence {
    pub fn builder() -> builder::Confidence {
        Default::default()
    }
}
#[doc = "What was actually decoded. Speech the pass never reached is not speech that turned out to be empty."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was actually decoded. Speech the pass never reached is not speech that turned out to be empty.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"analyzed\","]
#[doc = "    \"decoded_ticks\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"analyzed\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"decoded_ticks\": {"]
#[doc = "      \"description\": \"Speech duration actually handed to the recognizer.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"sampling_plan\": {"]
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
pub struct Coverage {
    pub analyzed: bool,
    #[doc = "Speech duration actually handed to the recognizer."]
    pub decoded_ticks: u64,
    pub end_ticks: u64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub sampling_plan: ::std::option::Option<CoverageSamplingPlan>,
    pub start_ticks: u64,
}
impl Coverage {
    pub fn builder() -> builder::Coverage {
        Default::default()
    }
}
#[doc = "`CoverageSamplingPlan`"]
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
pub struct CoverageSamplingPlan(::std::string::String);
impl ::std::ops::Deref for CoverageSamplingPlan {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CoverageSamplingPlan> for ::std::string::String {
    fn from(value: CoverageSamplingPlan) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CoverageSamplingPlan {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CoverageSamplingPlan {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CoverageSamplingPlan {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CoverageSamplingPlan {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CoverageSamplingPlan {
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
#[doc = "`InvalidRegion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"reason\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"not_analyzed\","]
#[doc = "        \"decode_failed\","]
#[doc = "        \"below_threshold\","]
#[doc = "        \"no_audio\""]
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
pub struct InvalidRegion {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub detail: ::std::option::Option<InvalidRegionDetail>,
    pub end_ticks: u64,
    pub reason: InvalidRegionReason,
    pub start_ticks: u64,
}
impl InvalidRegion {
    pub fn builder() -> builder::InvalidRegion {
        Default::default()
    }
}
#[doc = "`InvalidRegionDetail`"]
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
pub struct InvalidRegionDetail(::std::string::String);
impl ::std::ops::Deref for InvalidRegionDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<InvalidRegionDetail> for ::std::string::String {
    fn from(value: InvalidRegionDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for InvalidRegionDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for InvalidRegionDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for InvalidRegionDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for InvalidRegionDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for InvalidRegionDetail {
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
#[doc = "`InvalidRegionReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"not_analyzed\","]
#[doc = "    \"decode_failed\","]
#[doc = "    \"below_threshold\","]
#[doc = "    \"no_audio\""]
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
pub enum InvalidRegionReason {
    #[serde(rename = "not_analyzed")]
    NotAnalyzed,
    #[serde(rename = "decode_failed")]
    DecodeFailed,
    #[serde(rename = "below_threshold")]
    BelowThreshold,
    #[serde(rename = "no_audio")]
    NoAudio,
}
impl ::std::fmt::Display for InvalidRegionReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::NotAnalyzed => f.write_str("not_analyzed"),
            Self::DecodeFailed => f.write_str("decode_failed"),
            Self::BelowThreshold => f.write_str("below_threshold"),
            Self::NoAudio => f.write_str("no_audio"),
        }
    }
}
impl ::std::str::FromStr for InvalidRegionReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "not_analyzed" => Ok(Self::NotAnalyzed),
            "decode_failed" => Ok(Self::DecodeFailed),
            "below_threshold" => Ok(Self::BelowThreshold),
            "no_audio" => Ok(Self::NoAudio),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for InvalidRegionReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for InvalidRegionReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for InvalidRegionReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Producer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"implementation\","]
#[doc = "    \"stage\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"calibration\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"implementation\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"model_digest\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"stage\": {"]
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
pub struct Producer {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub calibration: ::std::option::Option<ProducerCalibration>,
    pub implementation: ProducerImplementation,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub model_digest: ::std::option::Option<Sha256>,
    pub stage: ProducerStage,
}
impl Producer {
    pub fn builder() -> builder::Producer {
        Default::default()
    }
}
#[doc = "`ProducerCalibration`"]
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
pub struct ProducerCalibration(::std::string::String);
impl ::std::ops::Deref for ProducerCalibration {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProducerCalibration> for ::std::string::String {
    fn from(value: ProducerCalibration) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProducerCalibration {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProducerCalibration {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProducerCalibration {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProducerCalibration {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProducerCalibration {
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
#[doc = "`ProducerImplementation`"]
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
pub struct ProducerImplementation(::std::string::String);
impl ::std::ops::Deref for ProducerImplementation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProducerImplementation> for ::std::string::String {
    fn from(value: ProducerImplementation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProducerImplementation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProducerImplementation {
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
#[doc = "`ProducerStage`"]
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
pub struct ProducerStage(::std::string::String);
impl ::std::ops::Deref for ProducerStage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProducerStage> for ::std::string::String {
    fn from(value: ProducerStage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProducerStage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProducerStage {
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
#[doc = "What was said, from a recognizer batched over voice activity (book ch. 13). Text and token confidence only: the intervals here are the decoder's own bookkeeping and are named as hints because word timing comes from forced alignment, never from decoder token positions. One contract serves every implementation — whisper.cpp on any CPU, an accelerated primary where one is available — so the daemon can choose between them by measured benchmark rather than by brand."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.speech.asr.v1.json\","]
#[doc = "  \"title\": \"SpeechAsr\","]
#[doc = "  \"description\": \"What was said, from a recognizer batched over voice activity (book ch. 13). Text and token confidence only: the intervals here are the decoder's own bookkeeping and are named as hints because word timing comes from forced alignment, never from decoder token positions. One contract serves every implementation — whisper.cpp on any CPU, an accelerated primary where one is available — so the daemon can choose between them by measured benchmark rather than by brand.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"audio_artifact_id\","]
#[doc = "    \"coverage\","]
#[doc = "    \"decoding\","]
#[doc = "    \"invalid_regions\","]
#[doc = "    \"language\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"segments\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"timing_authority\","]
#[doc = "    \"vad_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"audio_artifact_id\": {"]
#[doc = "      \"description\": \"The 16 kHz mono rendition this pass decoded, verified before use.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"decoding\": {"]
#[doc = "      \"description\": \"How the text was produced. Greedy decoding at temperature zero is not a quality preference — it is what makes two runs of the same audio agree, which is what lets a cached transcript be trusted.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"beam_size\","]
#[doc = "        \"conditioned_on_previous\","]
#[doc = "        \"strategy\","]
#[doc = "        \"temperature\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"beam_size\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"conditioned_on_previous\": {"]
#[doc = "          \"description\": \"Whether each window saw the previous window's text. False keeps one bad window from poisoning the rest.\","]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"initial_prompt_digest\": {"]
#[doc = "          \"description\": \"Digest of any vocabulary hint given to the decoder, so a hinted transcript is never confused with an unhinted one.\","]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"strategy\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"greedy\","]
#[doc = "            \"beam\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"temperature\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"invalid_regions\": {"]
#[doc = "      \"description\": \"Speech the recognizer could not turn into text. Explicit, because a downstream stage reading a gap as silence would place a cut in the middle of a sentence.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/invalid_region\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"description\": \"BCP 47 primary subtag of the recognized language.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 16,"]
#[doc = "      \"minLength\": 2"]
#[doc = "    },"]
#[doc = "    \"language_confidence\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.speech.asr.v1\""]
#[doc = "    },"]
#[doc = "    \"segments\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/asr_segment\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"timing_authority\": {"]
#[doc = "      \"description\": \"Stated once, at the top, so no consumer can reach the hint fields below without having read this: the intervals in this artifact are decoder bookkeeping. Word timing is the forced aligner's output and nothing else.\","]
#[doc = "      \"const\": \"forced_alignment\""]
#[doc = "    },"]
#[doc = "    \"vad_artifact_id\": {"]
#[doc = "      \"description\": \"The voice activity that bounded the decode. Recorded because re-running with different speech boundaries is a different observation.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpeechAsr {
    #[doc = "The 16 kHz mono rendition this pass decoded, verified before use."]
    pub audio_artifact_id: Sha256,
    pub coverage: Coverage,
    pub decoding: SpeechAsrDecoding,
    #[doc = "Speech the recognizer could not turn into text. Explicit, because a downstream stage reading a gap as silence would place a cut in the middle of a sentence."]
    pub invalid_regions: ::std::vec::Vec<InvalidRegion>,
    #[doc = "BCP 47 primary subtag of the recognized language."]
    pub language: SpeechAsrLanguage,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub language_confidence: ::std::option::Option<f64>,
    pub producer: Producer,
    pub schema_version: ::serde_json::Value,
    pub segments: ::std::vec::Vec<AsrSegment>,
    pub source_fingerprint: Sha256,
    #[doc = "Stated once, at the top, so no consumer can reach the hint fields below without having read this: the intervals in this artifact are decoder bookkeeping. Word timing is the forced aligner's output and nothing else."]
    pub timing_authority: ::serde_json::Value,
    #[doc = "The voice activity that bounded the decode. Recorded because re-running with different speech boundaries is a different observation."]
    pub vad_artifact_id: Sha256,
}
impl SpeechAsr {
    pub fn builder() -> builder::SpeechAsr {
        Default::default()
    }
}
#[doc = "How the text was produced. Greedy decoding at temperature zero is not a quality preference — it is what makes two runs of the same audio agree, which is what lets a cached transcript be trusted."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How the text was produced. Greedy decoding at temperature zero is not a quality preference — it is what makes two runs of the same audio agree, which is what lets a cached transcript be trusted.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"beam_size\","]
#[doc = "    \"conditioned_on_previous\","]
#[doc = "    \"strategy\","]
#[doc = "    \"temperature\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"beam_size\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"conditioned_on_previous\": {"]
#[doc = "      \"description\": \"Whether each window saw the previous window's text. False keeps one bad window from poisoning the rest.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"initial_prompt_digest\": {"]
#[doc = "      \"description\": \"Digest of any vocabulary hint given to the decoder, so a hinted transcript is never confused with an unhinted one.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"strategy\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"greedy\","]
#[doc = "        \"beam\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"temperature\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpeechAsrDecoding {
    pub beam_size: u64,
    #[doc = "Whether each window saw the previous window's text. False keeps one bad window from poisoning the rest."]
    pub conditioned_on_previous: bool,
    #[doc = "Digest of any vocabulary hint given to the decoder, so a hinted transcript is never confused with an unhinted one."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub initial_prompt_digest: ::std::option::Option<Sha256>,
    pub strategy: SpeechAsrDecodingStrategy,
    pub temperature: f64,
}
impl SpeechAsrDecoding {
    pub fn builder() -> builder::SpeechAsrDecoding {
        Default::default()
    }
}
#[doc = "`SpeechAsrDecodingStrategy`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"greedy\","]
#[doc = "    \"beam\""]
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
pub enum SpeechAsrDecodingStrategy {
    #[serde(rename = "greedy")]
    Greedy,
    #[serde(rename = "beam")]
    Beam,
}
impl ::std::fmt::Display for SpeechAsrDecodingStrategy {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Greedy => f.write_str("greedy"),
            Self::Beam => f.write_str("beam"),
        }
    }
}
impl ::std::str::FromStr for SpeechAsrDecodingStrategy {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "greedy" => Ok(Self::Greedy),
            "beam" => Ok(Self::Beam),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for SpeechAsrDecodingStrategy {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SpeechAsrDecodingStrategy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SpeechAsrDecodingStrategy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "BCP 47 primary subtag of the recognized language."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"BCP 47 primary subtag of the recognized language.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 16,"]
#[doc = "  \"minLength\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct SpeechAsrLanguage(::std::string::String);
impl ::std::ops::Deref for SpeechAsrLanguage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<SpeechAsrLanguage> for ::std::string::String {
    fn from(value: SpeechAsrLanguage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for SpeechAsrLanguage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 16usize {
            return Err("longer than 16 characters".into());
        }
        if value.chars().count() < 2usize {
            return Err("shorter than 2 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for SpeechAsrLanguage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SpeechAsrLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SpeechAsrLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for SpeechAsrLanguage {
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
#[doc = "`Token`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"text\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Token {
    pub confidence: f64,
    pub text: ::std::string::String,
}
impl Token {
    pub fn builder() -> builder::Token {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct AsrSegment {
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        hint_end_ticks: ::std::result::Result<u64, ::std::string::String>,
        hint_start_ticks: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        text: ::std::result::Result<::std::string::String, ::std::string::String>,
        tokens: ::std::result::Result<::std::vec::Vec<super::Token>, ::std::string::String>,
        vad_segment_index: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for AsrSegment {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                hint_end_ticks: Err("no value supplied for hint_end_ticks".to_string()),
                hint_start_ticks: Err("no value supplied for hint_start_ticks".to_string()),
                index: Err("no value supplied for index".to_string()),
                text: Err("no value supplied for text".to_string()),
                tokens: Err("no value supplied for tokens".to_string()),
                vad_segment_index: Err("no value supplied for vad_segment_index".to_string()),
            }
        }
    }
    impl AsrSegment {
        pub fn confidence<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Confidence>,
            T::Error: ::std::fmt::Display,
        {
            self.confidence = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for confidence: {e}"));
            self
        }
        pub fn hint_end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.hint_end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hint_end_ticks: {e}"));
            self
        }
        pub fn hint_start_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.hint_start_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hint_start_ticks: {e}"));
            self
        }
        pub fn index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for index: {e}"));
            self
        }
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
        pub fn tokens<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Token>>,
            T::Error: ::std::fmt::Display,
        {
            self.tokens = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tokens: {e}"));
            self
        }
        pub fn vad_segment_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.vad_segment_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for vad_segment_index: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AsrSegment> for super::AsrSegment {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AsrSegment,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                hint_end_ticks: value.hint_end_ticks?,
                hint_start_ticks: value.hint_start_ticks?,
                index: value.index?,
                text: value.text?,
                tokens: value.tokens?,
                vad_segment_index: value.vad_segment_index?,
            })
        }
    }
    impl ::std::convert::From<super::AsrSegment> for AsrSegment {
        fn from(value: super::AsrSegment) -> Self {
            Self {
                confidence: Ok(value.confidence),
                hint_end_ticks: Ok(value.hint_end_ticks),
                hint_start_ticks: Ok(value.hint_start_ticks),
                index: Ok(value.index),
                text: Ok(value.text),
                tokens: Ok(value.tokens),
                vad_segment_index: Ok(value.vad_segment_index),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Confidence {
        p10: ::std::result::Result<f64, ::std::string::String>,
        p50: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Confidence {
        fn default() -> Self {
            Self {
                p10: Err("no value supplied for p10".to_string()),
                p50: Err("no value supplied for p50".to_string()),
            }
        }
    }
    impl Confidence {
        pub fn p10<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.p10 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for p10: {e}"));
            self
        }
        pub fn p50<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.p50 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for p50: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Confidence> for super::Confidence {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Confidence,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                p10: value.p10?,
                p50: value.p50?,
            })
        }
    }
    impl ::std::convert::From<super::Confidence> for Confidence {
        fn from(value: super::Confidence) -> Self {
            Self {
                p10: Ok(value.p10),
                p50: Ok(value.p50),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Coverage {
        analyzed: ::std::result::Result<bool, ::std::string::String>,
        decoded_ticks: ::std::result::Result<u64, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        sampling_plan: ::std::result::Result<
            ::std::option::Option<super::CoverageSamplingPlan>,
            ::std::string::String,
        >,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Coverage {
        fn default() -> Self {
            Self {
                analyzed: Err("no value supplied for analyzed".to_string()),
                decoded_ticks: Err("no value supplied for decoded_ticks".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                sampling_plan: Ok(Default::default()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Coverage {
        pub fn analyzed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.analyzed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for analyzed: {e}"));
            self
        }
        pub fn decoded_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.decoded_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for decoded_ticks: {e}"));
            self
        }
        pub fn end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_ticks: {e}"));
            self
        }
        pub fn sampling_plan<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CoverageSamplingPlan>>,
            T::Error: ::std::fmt::Display,
        {
            self.sampling_plan = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sampling_plan: {e}"));
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
    impl ::std::convert::TryFrom<Coverage> for super::Coverage {
        type Error = super::error::ConversionError;
        fn try_from(value: Coverage) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                analyzed: value.analyzed?,
                decoded_ticks: value.decoded_ticks?,
                end_ticks: value.end_ticks?,
                sampling_plan: value.sampling_plan?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Coverage> for Coverage {
        fn from(value: super::Coverage) -> Self {
            Self {
                analyzed: Ok(value.analyzed),
                decoded_ticks: Ok(value.decoded_ticks),
                end_ticks: Ok(value.end_ticks),
                sampling_plan: Ok(value.sampling_plan),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct InvalidRegion {
        detail: ::std::result::Result<
            ::std::option::Option<super::InvalidRegionDetail>,
            ::std::string::String,
        >,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        reason: ::std::result::Result<super::InvalidRegionReason, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for InvalidRegion {
        fn default() -> Self {
            Self {
                detail: Ok(Default::default()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                reason: Err("no value supplied for reason".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl InvalidRegion {
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::InvalidRegionDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_ticks: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::InvalidRegionReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
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
    impl ::std::convert::TryFrom<InvalidRegion> for super::InvalidRegion {
        type Error = super::error::ConversionError;
        fn try_from(
            value: InvalidRegion,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                detail: value.detail?,
                end_ticks: value.end_ticks?,
                reason: value.reason?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::InvalidRegion> for InvalidRegion {
        fn from(value: super::InvalidRegion) -> Self {
            Self {
                detail: Ok(value.detail),
                end_ticks: Ok(value.end_ticks),
                reason: Ok(value.reason),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Producer {
        calibration: ::std::result::Result<
            ::std::option::Option<super::ProducerCalibration>,
            ::std::string::String,
        >,
        implementation: ::std::result::Result<super::ProducerImplementation, ::std::string::String>,
        model_digest:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        stage: ::std::result::Result<super::ProducerStage, ::std::string::String>,
    }
    impl ::std::default::Default for Producer {
        fn default() -> Self {
            Self {
                calibration: Ok(Default::default()),
                implementation: Err("no value supplied for implementation".to_string()),
                model_digest: Ok(Default::default()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl Producer {
        pub fn calibration<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ProducerCalibration>>,
            T::Error: ::std::fmt::Display,
        {
            self.calibration = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for calibration: {e}"));
            self
        }
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProducerImplementation>,
            T::Error: ::std::fmt::Display,
        {
            self.implementation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for implementation: {e}"));
            self
        }
        pub fn model_digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.model_digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model_digest: {e}"));
            self
        }
        pub fn stage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProducerStage>,
            T::Error: ::std::fmt::Display,
        {
            self.stage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stage: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Producer> for super::Producer {
        type Error = super::error::ConversionError;
        fn try_from(value: Producer) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                calibration: value.calibration?,
                implementation: value.implementation?,
                model_digest: value.model_digest?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::Producer> for Producer {
        fn from(value: super::Producer) -> Self {
            Self {
                calibration: Ok(value.calibration),
                implementation: Ok(value.implementation),
                model_digest: Ok(value.model_digest),
                stage: Ok(value.stage),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SpeechAsr {
        audio_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        decoding: ::std::result::Result<super::SpeechAsrDecoding, ::std::string::String>,
        invalid_regions:
            ::std::result::Result<::std::vec::Vec<super::InvalidRegion>, ::std::string::String>,
        language: ::std::result::Result<super::SpeechAsrLanguage, ::std::string::String>,
        language_confidence:
            ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        segments: ::std::result::Result<::std::vec::Vec<super::AsrSegment>, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        timing_authority: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        vad_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechAsr {
        fn default() -> Self {
            Self {
                audio_artifact_id: Err("no value supplied for audio_artifact_id".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                decoding: Err("no value supplied for decoding".to_string()),
                invalid_regions: Err("no value supplied for invalid_regions".to_string()),
                language: Err("no value supplied for language".to_string()),
                language_confidence: Ok(Default::default()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                segments: Err("no value supplied for segments".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                timing_authority: Err("no value supplied for timing_authority".to_string()),
                vad_artifact_id: Err("no value supplied for vad_artifact_id".to_string()),
            }
        }
    }
    impl SpeechAsr {
        pub fn audio_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.audio_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio_artifact_id: {e}"));
            self
        }
        pub fn coverage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Coverage>,
            T::Error: ::std::fmt::Display,
        {
            self.coverage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for coverage: {e}"));
            self
        }
        pub fn decoding<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SpeechAsrDecoding>,
            T::Error: ::std::fmt::Display,
        {
            self.decoding = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for decoding: {e}"));
            self
        }
        pub fn invalid_regions<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::InvalidRegion>>,
            T::Error: ::std::fmt::Display,
        {
            self.invalid_regions = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for invalid_regions: {e}"));
            self
        }
        pub fn language<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SpeechAsrLanguage>,
            T::Error: ::std::fmt::Display,
        {
            self.language = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for language: {e}"));
            self
        }
        pub fn language_confidence<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.language_confidence = value.try_into().map_err(|e| {
                format!("error converting supplied value for language_confidence: {e}")
            });
            self
        }
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Producer>,
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
        pub fn segments<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::AsrSegment>>,
            T::Error: ::std::fmt::Display,
        {
            self.segments = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segments: {e}"));
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
        pub fn timing_authority<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.timing_authority = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timing_authority: {e}"));
            self
        }
        pub fn vad_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.vad_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for vad_artifact_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SpeechAsr> for super::SpeechAsr {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechAsr,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                audio_artifact_id: value.audio_artifact_id?,
                coverage: value.coverage?,
                decoding: value.decoding?,
                invalid_regions: value.invalid_regions?,
                language: value.language?,
                language_confidence: value.language_confidence?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                segments: value.segments?,
                source_fingerprint: value.source_fingerprint?,
                timing_authority: value.timing_authority?,
                vad_artifact_id: value.vad_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechAsr> for SpeechAsr {
        fn from(value: super::SpeechAsr) -> Self {
            Self {
                audio_artifact_id: Ok(value.audio_artifact_id),
                coverage: Ok(value.coverage),
                decoding: Ok(value.decoding),
                invalid_regions: Ok(value.invalid_regions),
                language: Ok(value.language),
                language_confidence: Ok(value.language_confidence),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                segments: Ok(value.segments),
                source_fingerprint: Ok(value.source_fingerprint),
                timing_authority: Ok(value.timing_authority),
                vad_artifact_id: Ok(value.vad_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SpeechAsrDecoding {
        beam_size: ::std::result::Result<u64, ::std::string::String>,
        conditioned_on_previous: ::std::result::Result<bool, ::std::string::String>,
        initial_prompt_digest:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        strategy: ::std::result::Result<super::SpeechAsrDecodingStrategy, ::std::string::String>,
        temperature: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechAsrDecoding {
        fn default() -> Self {
            Self {
                beam_size: Err("no value supplied for beam_size".to_string()),
                conditioned_on_previous: Err(
                    "no value supplied for conditioned_on_previous".to_string()
                ),
                initial_prompt_digest: Ok(Default::default()),
                strategy: Err("no value supplied for strategy".to_string()),
                temperature: Err("no value supplied for temperature".to_string()),
            }
        }
    }
    impl SpeechAsrDecoding {
        pub fn beam_size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.beam_size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for beam_size: {e}"));
            self
        }
        pub fn conditioned_on_previous<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.conditioned_on_previous = value.try_into().map_err(|e| {
                format!("error converting supplied value for conditioned_on_previous: {e}")
            });
            self
        }
        pub fn initial_prompt_digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.initial_prompt_digest = value.try_into().map_err(|e| {
                format!("error converting supplied value for initial_prompt_digest: {e}")
            });
            self
        }
        pub fn strategy<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SpeechAsrDecodingStrategy>,
            T::Error: ::std::fmt::Display,
        {
            self.strategy = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for strategy: {e}"));
            self
        }
        pub fn temperature<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.temperature = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for temperature: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SpeechAsrDecoding> for super::SpeechAsrDecoding {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechAsrDecoding,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                beam_size: value.beam_size?,
                conditioned_on_previous: value.conditioned_on_previous?,
                initial_prompt_digest: value.initial_prompt_digest?,
                strategy: value.strategy?,
                temperature: value.temperature?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechAsrDecoding> for SpeechAsrDecoding {
        fn from(value: super::SpeechAsrDecoding) -> Self {
            Self {
                beam_size: Ok(value.beam_size),
                conditioned_on_previous: Ok(value.conditioned_on_previous),
                initial_prompt_digest: Ok(value.initial_prompt_digest),
                strategy: Ok(value.strategy),
                temperature: Ok(value.temperature),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Token {
        confidence: ::std::result::Result<f64, ::std::string::String>,
        text: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for Token {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                text: Err("no value supplied for text".to_string()),
            }
        }
    }
    impl Token {
        pub fn confidence<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.confidence = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for confidence: {e}"));
            self
        }
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Token> for super::Token {
        type Error = super::error::ConversionError;
        fn try_from(value: Token) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                text: value.text?,
            })
        }
    }
    impl ::std::convert::From<super::Token> for Token {
        fn from(value: super::Token) -> Self {
            Self {
                confidence: Ok(value.confidence),
                text: Ok(value.text),
            }
        }
    }
}
