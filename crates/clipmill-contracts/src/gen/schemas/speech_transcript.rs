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
#[doc = "`Confidence`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
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
#[doc = "Three separate numbers because they answer three separate questions: how much of the recording was examined, how much of it was speech, and how much of that speech has timing anyone measured."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Three separate numbers because they answer three separate questions: how much of the recording was examined, how much of it was speech, and how much of that speech has timing anyone measured.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"aligned_ticks\","]
#[doc = "    \"analyzed\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"speech_ticks\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"aligned_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"analyzed\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"sampling_plan\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"speech_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
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
    pub aligned_ticks: u64,
    pub analyzed: bool,
    pub end_ticks: u64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub sampling_plan: ::std::option::Option<CoverageSamplingPlan>,
    pub speech_ticks: u64,
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
#[doc = "`Interval`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
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
pub struct Interval {
    pub end_ticks: u64,
    pub start_ticks: u64,
}
impl Interval {
    pub fn builder() -> builder::Interval {
        Default::default()
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
#[doc = "        \"no_audio\","]
#[doc = "        \"decode_failed\","]
#[doc = "        \"alignment_unavailable\","]
#[doc = "        \"timing_interpolated\""]
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
#[doc = "    \"no_audio\","]
#[doc = "    \"decode_failed\","]
#[doc = "    \"alignment_unavailable\","]
#[doc = "    \"timing_interpolated\""]
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
    #[serde(rename = "no_audio")]
    NoAudio,
    #[serde(rename = "decode_failed")]
    DecodeFailed,
    #[serde(rename = "alignment_unavailable")]
    AlignmentUnavailable,
    #[serde(rename = "timing_interpolated")]
    TimingInterpolated,
}
impl ::std::fmt::Display for InvalidRegionReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::NotAnalyzed => f.write_str("not_analyzed"),
            Self::NoAudio => f.write_str("no_audio"),
            Self::DecodeFailed => f.write_str("decode_failed"),
            Self::AlignmentUnavailable => f.write_str("alignment_unavailable"),
            Self::TimingInterpolated => f.write_str("timing_interpolated"),
        }
    }
}
impl ::std::str::FromStr for InvalidRegionReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "not_analyzed" => Ok(Self::NotAnalyzed),
            "no_audio" => Ok(Self::NoAudio),
            "decode_failed" => Ok(Self::DecodeFailed),
            "alignment_unavailable" => Ok(Self::AlignmentUnavailable),
            "timing_interpolated" => Ok(Self::TimingInterpolated),
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
#[doc = "`Segment`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_word_index\","]
#[doc = "    \"index\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"text\","]
#[doc = "    \"word_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"first_word_index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"word_count\": {"]
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
pub struct Segment {
    pub confidence: Confidence,
    pub end_ticks: u64,
    pub first_word_index: u64,
    pub index: u64,
    pub start_ticks: u64,
    pub text: ::std::string::String,
    pub word_count: u64,
}
impl Segment {
    pub fn builder() -> builder::Segment {
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
#[doc = "The speech chain's single published observation: voice activity, recognized text, and measured word timing fused into one document that downstream stages read instead of the three artifacts behind it (book ch. 13, App. B). It carries what an observation is required to carry — rational intervals, a confidence distribution rather than a bare scalar, an explicit coverage statement, every producing model's identity, and the regions known to be invalid — so that discovery, ranking, captions, and the editor can request evidence by minimum quality and say why when it is missing."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.speech.transcript.v1.json\","]
#[doc = "  \"title\": \"SpeechTranscript\","]
#[doc = "  \"description\": \"The speech chain's single published observation: voice activity, recognized text, and measured word timing fused into one document that downstream stages read instead of the three artifacts behind it (book ch. 13, App. B). It carries what an observation is required to carry — rational intervals, a confidence distribution rather than a bare scalar, an explicit coverage statement, every producing model's identity, and the regions known to be invalid — so that discovery, ranking, captions, and the editor can request evidence by minimum quality and say why when it is missing.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"coverage\","]
#[doc = "    \"inputs\","]
#[doc = "    \"invalid_regions\","]
#[doc = "    \"language\","]
#[doc = "    \"producers\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"segments\","]
#[doc = "    \"silences\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"words\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"description\": \"Over the words below. Ranking reads p10, not p50, when it decides whether a quote is safe to put on screen.\","]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"description\": \"The three artifacts fused here, so any claim in this document can be walked back to the pass that made it.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"alignment_artifact_id\","]
#[doc = "        \"asr_artifact_id\","]
#[doc = "        \"vad_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"alignment_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"asr_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"audio_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"vad_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"invalid_regions\": {"]
#[doc = "      \"description\": \"Spans this transcript does not vouch for, including any whose word timing was interpolated rather than measured. The boundary optimizer refuses to place an edit inside one.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/invalid_region\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 16,"]
#[doc = "      \"minLength\": 2"]
#[doc = "    },"]
#[doc = "    \"language_confidence\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"producers\": {"]
#[doc = "      \"description\": \"One entry per contributing stage, each with the digest of the model that ran. A transcript that cannot name the models behind it cannot support the AI-use disclosure a render manifest has to make.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/producer\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.speech.transcript.v1\""]
#[doc = "    },"]
#[doc = "    \"segments\": {"]
#[doc = "      \"description\": \"Recognizer segments, as index ranges into the word list rather than duplicated text, so the two can never disagree.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/segment\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"silences\": {"]
#[doc = "      \"description\": \"Carried through from voice activity: these are where a cut is allowed to land without severing speech.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/interval\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"words\": {"]
#[doc = "      \"description\": \"The authoritative word list, ordered by time. Every word carries timing and says how that timing was obtained.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/word\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpeechTranscript {
    #[doc = "Over the words below. Ranking reads p10, not p50, when it decides whether a quote is safe to put on screen."]
    pub confidence: Confidence,
    pub coverage: Coverage,
    pub inputs: SpeechTranscriptInputs,
    #[doc = "Spans this transcript does not vouch for, including any whose word timing was interpolated rather than measured. The boundary optimizer refuses to place an edit inside one."]
    pub invalid_regions: ::std::vec::Vec<InvalidRegion>,
    pub language: SpeechTranscriptLanguage,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub language_confidence: ::std::option::Option<f64>,
    #[doc = "One entry per contributing stage, each with the digest of the model that ran. A transcript that cannot name the models behind it cannot support the AI-use disclosure a render manifest has to make."]
    pub producers: ::std::vec::Vec<Producer>,
    pub schema_version: ::serde_json::Value,
    #[doc = "Recognizer segments, as index ranges into the word list rather than duplicated text, so the two can never disagree."]
    pub segments: ::std::vec::Vec<Segment>,
    #[doc = "Carried through from voice activity: these are where a cut is allowed to land without severing speech."]
    pub silences: ::std::vec::Vec<Interval>,
    pub source_fingerprint: Sha256,
    #[doc = "The authoritative word list, ordered by time. Every word carries timing and says how that timing was obtained."]
    pub words: ::std::vec::Vec<Word>,
}
impl SpeechTranscript {
    pub fn builder() -> builder::SpeechTranscript {
        Default::default()
    }
}
#[doc = "The three artifacts fused here, so any claim in this document can be walked back to the pass that made it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The three artifacts fused here, so any claim in this document can be walked back to the pass that made it.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"alignment_artifact_id\","]
#[doc = "    \"asr_artifact_id\","]
#[doc = "    \"vad_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"alignment_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"asr_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"audio_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"vad_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpeechTranscriptInputs {
    pub alignment_artifact_id: Sha256,
    pub asr_artifact_id: Sha256,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub audio_artifact_id: ::std::option::Option<Sha256>,
    pub vad_artifact_id: Sha256,
}
impl SpeechTranscriptInputs {
    pub fn builder() -> builder::SpeechTranscriptInputs {
        Default::default()
    }
}
#[doc = "`SpeechTranscriptLanguage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 16,"]
#[doc = "  \"minLength\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct SpeechTranscriptLanguage(::std::string::String);
impl ::std::ops::Deref for SpeechTranscriptLanguage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<SpeechTranscriptLanguage> for ::std::string::String {
    fn from(value: SpeechTranscriptLanguage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for SpeechTranscriptLanguage {
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
impl ::std::convert::TryFrom<&str> for SpeechTranscriptLanguage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SpeechTranscriptLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SpeechTranscriptLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for SpeechTranscriptLanguage {
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
#[doc = "`Word`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"index\","]
#[doc = "    \"segment_index\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"text\","]
#[doc = "    \"timing\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"segment_index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"timing\": {"]
#[doc = "      \"description\": \"How the interval was obtained. 'aligned' was measured against the audio; 'interpolated' was spread across a span the aligner could not place, and every such word's span also appears in invalid_regions. Dropping the text would lose what was said; presenting it as measured would be a lie — labelling it is the only honest option.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"aligned\","]
#[doc = "        \"interpolated\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Word {
    pub confidence: Confidence,
    pub end_ticks: u64,
    pub index: u64,
    pub segment_index: u64,
    pub start_ticks: u64,
    pub text: WordText,
    #[doc = "How the interval was obtained. 'aligned' was measured against the audio; 'interpolated' was spread across a span the aligner could not place, and every such word's span also appears in invalid_regions. Dropping the text would lose what was said; presenting it as measured would be a lie — labelling it is the only honest option."]
    pub timing: WordTiming,
}
impl Word {
    pub fn builder() -> builder::Word {
        Default::default()
    }
}
#[doc = "`WordText`"]
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
pub struct WordText(::std::string::String);
impl ::std::ops::Deref for WordText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<WordText> for ::std::string::String {
    fn from(value: WordText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for WordText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for WordText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for WordText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for WordText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WordText {
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
#[doc = "How the interval was obtained. 'aligned' was measured against the audio; 'interpolated' was spread across a span the aligner could not place, and every such word's span also appears in invalid_regions. Dropping the text would lose what was said; presenting it as measured would be a lie — labelling it is the only honest option."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How the interval was obtained. 'aligned' was measured against the audio; 'interpolated' was spread across a span the aligner could not place, and every such word's span also appears in invalid_regions. Dropping the text would lose what was said; presenting it as measured would be a lie — labelling it is the only honest option.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"aligned\","]
#[doc = "    \"interpolated\""]
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
pub enum WordTiming {
    #[serde(rename = "aligned")]
    Aligned,
    #[serde(rename = "interpolated")]
    Interpolated,
}
impl ::std::fmt::Display for WordTiming {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Aligned => f.write_str("aligned"),
            Self::Interpolated => f.write_str("interpolated"),
        }
    }
}
impl ::std::str::FromStr for WordTiming {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "aligned" => Ok(Self::Aligned),
            "interpolated" => Ok(Self::Interpolated),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for WordTiming {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for WordTiming {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for WordTiming {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
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
        aligned_ticks: ::std::result::Result<u64, ::std::string::String>,
        analyzed: ::std::result::Result<bool, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        sampling_plan: ::std::result::Result<
            ::std::option::Option<super::CoverageSamplingPlan>,
            ::std::string::String,
        >,
        speech_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Coverage {
        fn default() -> Self {
            Self {
                aligned_ticks: Err("no value supplied for aligned_ticks".to_string()),
                analyzed: Err("no value supplied for analyzed".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                sampling_plan: Ok(Default::default()),
                speech_ticks: Err("no value supplied for speech_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Coverage {
        pub fn aligned_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.aligned_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for aligned_ticks: {e}"));
            self
        }
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
        pub fn speech_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.speech_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for speech_ticks: {e}"));
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
                aligned_ticks: value.aligned_ticks?,
                analyzed: value.analyzed?,
                end_ticks: value.end_ticks?,
                sampling_plan: value.sampling_plan?,
                speech_ticks: value.speech_ticks?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Coverage> for Coverage {
        fn from(value: super::Coverage) -> Self {
            Self {
                aligned_ticks: Ok(value.aligned_ticks),
                analyzed: Ok(value.analyzed),
                end_ticks: Ok(value.end_ticks),
                sampling_plan: Ok(value.sampling_plan),
                speech_ticks: Ok(value.speech_ticks),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Interval {
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Interval {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Interval {
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
    impl ::std::convert::TryFrom<Interval> for super::Interval {
        type Error = super::error::ConversionError;
        fn try_from(value: Interval) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Interval> for Interval {
        fn from(value: super::Interval) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
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
    pub struct Segment {
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        first_word_index: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        text: ::std::result::Result<::std::string::String, ::std::string::String>,
        word_count: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Segment {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_word_index: Err("no value supplied for first_word_index".to_string()),
                index: Err("no value supplied for index".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                text: Err("no value supplied for text".to_string()),
                word_count: Err("no value supplied for word_count".to_string()),
            }
        }
    }
    impl Segment {
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
        pub fn first_word_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_word_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_word_index: {e}"));
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
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
        pub fn word_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.word_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for word_count: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Segment> for super::Segment {
        type Error = super::error::ConversionError;
        fn try_from(value: Segment) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                end_ticks: value.end_ticks?,
                first_word_index: value.first_word_index?,
                index: value.index?,
                start_ticks: value.start_ticks?,
                text: value.text?,
                word_count: value.word_count?,
            })
        }
    }
    impl ::std::convert::From<super::Segment> for Segment {
        fn from(value: super::Segment) -> Self {
            Self {
                confidence: Ok(value.confidence),
                end_ticks: Ok(value.end_ticks),
                first_word_index: Ok(value.first_word_index),
                index: Ok(value.index),
                start_ticks: Ok(value.start_ticks),
                text: Ok(value.text),
                word_count: Ok(value.word_count),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SpeechTranscript {
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        inputs: ::std::result::Result<super::SpeechTranscriptInputs, ::std::string::String>,
        invalid_regions:
            ::std::result::Result<::std::vec::Vec<super::InvalidRegion>, ::std::string::String>,
        language: ::std::result::Result<super::SpeechTranscriptLanguage, ::std::string::String>,
        language_confidence:
            ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        producers: ::std::result::Result<::std::vec::Vec<super::Producer>, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        segments: ::std::result::Result<::std::vec::Vec<super::Segment>, ::std::string::String>,
        silences: ::std::result::Result<::std::vec::Vec<super::Interval>, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        words: ::std::result::Result<::std::vec::Vec<super::Word>, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechTranscript {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                invalid_regions: Err("no value supplied for invalid_regions".to_string()),
                language: Err("no value supplied for language".to_string()),
                language_confidence: Ok(Default::default()),
                producers: Err("no value supplied for producers".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                segments: Err("no value supplied for segments".to_string()),
                silences: Err("no value supplied for silences".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                words: Err("no value supplied for words".to_string()),
            }
        }
    }
    impl SpeechTranscript {
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
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SpeechTranscriptInputs>,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
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
            T: ::std::convert::TryInto<super::SpeechTranscriptLanguage>,
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
        pub fn producers<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Producer>>,
            T::Error: ::std::fmt::Display,
        {
            self.producers = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for producers: {e}"));
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
            T: ::std::convert::TryInto<::std::vec::Vec<super::Segment>>,
            T::Error: ::std::fmt::Display,
        {
            self.segments = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segments: {e}"));
            self
        }
        pub fn silences<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Interval>>,
            T::Error: ::std::fmt::Display,
        {
            self.silences = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for silences: {e}"));
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
        pub fn words<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Word>>,
            T::Error: ::std::fmt::Display,
        {
            self.words = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for words: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SpeechTranscript> for super::SpeechTranscript {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechTranscript,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                coverage: value.coverage?,
                inputs: value.inputs?,
                invalid_regions: value.invalid_regions?,
                language: value.language?,
                language_confidence: value.language_confidence?,
                producers: value.producers?,
                schema_version: value.schema_version?,
                segments: value.segments?,
                silences: value.silences?,
                source_fingerprint: value.source_fingerprint?,
                words: value.words?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechTranscript> for SpeechTranscript {
        fn from(value: super::SpeechTranscript) -> Self {
            Self {
                confidence: Ok(value.confidence),
                coverage: Ok(value.coverage),
                inputs: Ok(value.inputs),
                invalid_regions: Ok(value.invalid_regions),
                language: Ok(value.language),
                language_confidence: Ok(value.language_confidence),
                producers: Ok(value.producers),
                schema_version: Ok(value.schema_version),
                segments: Ok(value.segments),
                silences: Ok(value.silences),
                source_fingerprint: Ok(value.source_fingerprint),
                words: Ok(value.words),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SpeechTranscriptInputs {
        alignment_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        asr_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        audio_artifact_id:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        vad_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechTranscriptInputs {
        fn default() -> Self {
            Self {
                alignment_artifact_id: Err(
                    "no value supplied for alignment_artifact_id".to_string()
                ),
                asr_artifact_id: Err("no value supplied for asr_artifact_id".to_string()),
                audio_artifact_id: Ok(Default::default()),
                vad_artifact_id: Err("no value supplied for vad_artifact_id".to_string()),
            }
        }
    }
    impl SpeechTranscriptInputs {
        pub fn alignment_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.alignment_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for alignment_artifact_id: {e}")
            });
            self
        }
        pub fn asr_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.asr_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for asr_artifact_id: {e}"));
            self
        }
        pub fn audio_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.audio_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio_artifact_id: {e}"));
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
    impl ::std::convert::TryFrom<SpeechTranscriptInputs> for super::SpeechTranscriptInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechTranscriptInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                alignment_artifact_id: value.alignment_artifact_id?,
                asr_artifact_id: value.asr_artifact_id?,
                audio_artifact_id: value.audio_artifact_id?,
                vad_artifact_id: value.vad_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechTranscriptInputs> for SpeechTranscriptInputs {
        fn from(value: super::SpeechTranscriptInputs) -> Self {
            Self {
                alignment_artifact_id: Ok(value.alignment_artifact_id),
                asr_artifact_id: Ok(value.asr_artifact_id),
                audio_artifact_id: Ok(value.audio_artifact_id),
                vad_artifact_id: Ok(value.vad_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Word {
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        segment_index: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        text: ::std::result::Result<super::WordText, ::std::string::String>,
        timing: ::std::result::Result<super::WordTiming, ::std::string::String>,
    }
    impl ::std::default::Default for Word {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                index: Err("no value supplied for index".to_string()),
                segment_index: Err("no value supplied for segment_index".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                text: Err("no value supplied for text".to_string()),
                timing: Err("no value supplied for timing".to_string()),
            }
        }
    }
    impl Word {
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
        pub fn segment_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.segment_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segment_index: {e}"));
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
            T: ::std::convert::TryInto<super::WordText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
        pub fn timing<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::WordTiming>,
            T::Error: ::std::fmt::Display,
        {
            self.timing = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timing: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Word> for super::Word {
        type Error = super::error::ConversionError;
        fn try_from(value: Word) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                end_ticks: value.end_ticks?,
                index: value.index?,
                segment_index: value.segment_index?,
                start_ticks: value.start_ticks?,
                text: value.text?,
                timing: value.timing?,
            })
        }
    }
    impl ::std::convert::From<super::Word> for Word {
        fn from(value: super::Word) -> Self {
            Self {
                confidence: Ok(value.confidence),
                end_ticks: Ok(value.end_ticks),
                index: Ok(value.index),
                segment_index: Ok(value.segment_index),
                start_ticks: Ok(value.start_ticks),
                text: Ok(value.text),
                timing: Ok(value.timing),
            }
        }
    }
}
