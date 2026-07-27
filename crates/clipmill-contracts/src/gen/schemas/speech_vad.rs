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
#[doc = "What was actually examined. A skipped pass, a below-threshold result, and a genuinely empty result are three different facts, and no downstream stage may read an empty segment list as 'nobody spoke'."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was actually examined. A skipped pass, a below-threshold result, and a genuinely empty result are three different facts, and no downstream stage may read an empty segment list as 'nobody spoke'.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"analyzed\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
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
#[doc = "        \"no_audio\","]
#[doc = "        \"not_analyzed\","]
#[doc = "        \"decode_failed\""]
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
#[doc = "    \"no_audio\","]
#[doc = "    \"not_analyzed\","]
#[doc = "    \"decode_failed\""]
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
    #[serde(rename = "no_audio")]
    NoAudio,
    #[serde(rename = "not_analyzed")]
    NotAnalyzed,
    #[serde(rename = "decode_failed")]
    DecodeFailed,
}
impl ::std::fmt::Display for InvalidRegionReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::NoAudio => f.write_str("no_audio"),
            Self::NotAnalyzed => f.write_str("not_analyzed"),
            Self::DecodeFailed => f.write_str("decode_failed"),
        }
    }
}
impl ::std::str::FromStr for InvalidRegionReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "no_audio" => Ok(Self::NoAudio),
            "not_analyzed" => Ok(Self::NotAnalyzed),
            "decode_failed" => Ok(Self::DecodeFailed),
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
#[doc = "      \"description\": \"Which calibration mapped raw model scores onto the confidences above.\","]
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
    #[doc = "Which calibration mapped raw model scores onto the confidences above."]
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
#[doc = "Which calibration mapped raw model scores onto the confidences above."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Which calibration mapped raw model scores onto the confidences above.\","]
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
#[doc = "`SpeechSegment`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
#[doc = "    },"]
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
pub struct SpeechSegment {
    pub confidence: Confidence,
    pub end_ticks: u64,
    pub start_ticks: u64,
}
impl SpeechSegment {
    pub fn builder() -> builder::SpeechSegment {
        Default::default()
    }
}
#[doc = "Where speech is, decided before anything transcribes it (book ch. 13). The first link in the speech chain: it bounds what ASR decodes, and its silence edges become candidate cut points for the boundary lattice. Every interval is integer ticks at 1/90000. Absence is explicit — a region nobody analyzed is not a region where nobody spoke."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.speech.vad.v1.json\","]
#[doc = "  \"title\": \"SpeechVad\","]
#[doc = "  \"description\": \"Where speech is, decided before anything transcribes it (book ch. 13). The first link in the speech chain: it bounds what ASR decodes, and its silence edges become candidate cut points for the boundary lattice. Every interval is integer ticks at 1/90000. Absence is explicit — a region nobody analyzed is not a region where nobody spoke.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"audio_artifact_id\","]
#[doc = "    \"coverage\","]
#[doc = "    \"detection\","]
#[doc = "    \"invalid_regions\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"segments\","]
#[doc = "    \"silences\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"speech_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"audio_artifact_id\": {"]
#[doc = "      \"description\": \"The 16 kHz mono rendition this pass read, verified before use.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"detection\": {"]
#[doc = "      \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"min_silence_ticks\","]
#[doc = "        \"min_speech_ticks\","]
#[doc = "        \"speech_pad_ticks\","]
#[doc = "        \"threshold\","]
#[doc = "        \"window_samples\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"min_silence_ticks\": {"]
#[doc = "          \"description\": \"A gap shorter than this does not end a segment.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"min_speech_ticks\": {"]
#[doc = "          \"description\": \"Segments shorter than this are dropped rather than reported.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"speech_pad_ticks\": {"]
#[doc = "          \"description\": \"Padding added to each side of a segment, so a decoder is not handed a word already in progress.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"threshold\": {"]
#[doc = "          \"description\": \"Speech probability at or above which a window counts as speech.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"window_samples\": {"]
#[doc = "          \"description\": \"Samples per scored window; the resolution of every boundary below.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"invalid_regions\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/invalid_region\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.speech.vad.v1\""]
#[doc = "    },"]
#[doc = "    \"segments\": {"]
#[doc = "      \"description\": \"Speech, in source time, ordered and non-overlapping.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/speech_segment\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"silences\": {"]
#[doc = "      \"description\": \"The gaps between speech, including any leading and trailing silence. Carried explicitly rather than left to be derived, because these are the edges the boundary lattice is allowed to cut on and a consumer should not have to reconstruct them correctly.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/interval\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"speech_ticks\": {"]
#[doc = "      \"description\": \"Total speech duration, so a consumer can state 'this recording is 8% speech' without walking the segments.\","]
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
pub struct SpeechVad {
    #[doc = "The 16 kHz mono rendition this pass read, verified before use."]
    pub audio_artifact_id: Sha256,
    pub coverage: Coverage,
    pub detection: SpeechVadDetection,
    pub invalid_regions: ::std::vec::Vec<InvalidRegion>,
    pub producer: Producer,
    pub schema_version: ::serde_json::Value,
    #[doc = "Speech, in source time, ordered and non-overlapping."]
    pub segments: ::std::vec::Vec<SpeechSegment>,
    #[doc = "The gaps between speech, including any leading and trailing silence. Carried explicitly rather than left to be derived, because these are the edges the boundary lattice is allowed to cut on and a consumer should not have to reconstruct them correctly."]
    pub silences: ::std::vec::Vec<Interval>,
    pub source_fingerprint: Sha256,
    #[doc = "Total speech duration, so a consumer can state 'this recording is 8% speech' without walking the segments."]
    pub speech_ticks: u64,
}
impl SpeechVad {
    pub fn builder() -> builder::SpeechVad {
        Default::default()
    }
}
#[doc = "The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"min_silence_ticks\","]
#[doc = "    \"min_speech_ticks\","]
#[doc = "    \"speech_pad_ticks\","]
#[doc = "    \"threshold\","]
#[doc = "    \"window_samples\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"min_silence_ticks\": {"]
#[doc = "      \"description\": \"A gap shorter than this does not end a segment.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"min_speech_ticks\": {"]
#[doc = "      \"description\": \"Segments shorter than this are dropped rather than reported.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"speech_pad_ticks\": {"]
#[doc = "      \"description\": \"Padding added to each side of a segment, so a decoder is not handed a word already in progress.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"threshold\": {"]
#[doc = "      \"description\": \"Speech probability at or above which a window counts as speech.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"window_samples\": {"]
#[doc = "      \"description\": \"Samples per scored window; the resolution of every boundary below.\","]
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
pub struct SpeechVadDetection {
    #[doc = "A gap shorter than this does not end a segment."]
    pub min_silence_ticks: u64,
    #[doc = "Segments shorter than this are dropped rather than reported."]
    pub min_speech_ticks: u64,
    #[doc = "Padding added to each side of a segment, so a decoder is not handed a word already in progress."]
    pub speech_pad_ticks: u64,
    #[doc = "Speech probability at or above which a window counts as speech."]
    pub threshold: f64,
    #[doc = "Samples per scored window; the resolution of every boundary below."]
    pub window_samples: ::std::num::NonZeroU64,
}
impl SpeechVadDetection {
    pub fn builder() -> builder::SpeechVadDetection {
        Default::default()
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
        analyzed: ::std::result::Result<bool, ::std::string::String>,
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
                end_ticks: Ok(value.end_ticks),
                sampling_plan: Ok(value.sampling_plan),
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
    pub struct SpeechSegment {
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechSegment {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl SpeechSegment {
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
    impl ::std::convert::TryFrom<SpeechSegment> for super::SpeechSegment {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechSegment,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechSegment> for SpeechSegment {
        fn from(value: super::SpeechSegment) -> Self {
            Self {
                confidence: Ok(value.confidence),
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SpeechVad {
        audio_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        detection: ::std::result::Result<super::SpeechVadDetection, ::std::string::String>,
        invalid_regions:
            ::std::result::Result<::std::vec::Vec<super::InvalidRegion>, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        segments:
            ::std::result::Result<::std::vec::Vec<super::SpeechSegment>, ::std::string::String>,
        silences: ::std::result::Result<::std::vec::Vec<super::Interval>, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        speech_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechVad {
        fn default() -> Self {
            Self {
                audio_artifact_id: Err("no value supplied for audio_artifact_id".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                detection: Err("no value supplied for detection".to_string()),
                invalid_regions: Err("no value supplied for invalid_regions".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                segments: Err("no value supplied for segments".to_string()),
                silences: Err("no value supplied for silences".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                speech_ticks: Err("no value supplied for speech_ticks".to_string()),
            }
        }
    }
    impl SpeechVad {
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
        pub fn detection<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SpeechVadDetection>,
            T::Error: ::std::fmt::Display,
        {
            self.detection = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detection: {e}"));
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
            T: ::std::convert::TryInto<::std::vec::Vec<super::SpeechSegment>>,
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
    }
    impl ::std::convert::TryFrom<SpeechVad> for super::SpeechVad {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechVad,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                audio_artifact_id: value.audio_artifact_id?,
                coverage: value.coverage?,
                detection: value.detection?,
                invalid_regions: value.invalid_regions?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                segments: value.segments?,
                silences: value.silences?,
                source_fingerprint: value.source_fingerprint?,
                speech_ticks: value.speech_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechVad> for SpeechVad {
        fn from(value: super::SpeechVad) -> Self {
            Self {
                audio_artifact_id: Ok(value.audio_artifact_id),
                coverage: Ok(value.coverage),
                detection: Ok(value.detection),
                invalid_regions: Ok(value.invalid_regions),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                segments: Ok(value.segments),
                silences: Ok(value.silences),
                source_fingerprint: Ok(value.source_fingerprint),
                speech_ticks: Ok(value.speech_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SpeechVadDetection {
        min_silence_ticks: ::std::result::Result<u64, ::std::string::String>,
        min_speech_ticks: ::std::result::Result<u64, ::std::string::String>,
        speech_pad_ticks: ::std::result::Result<u64, ::std::string::String>,
        threshold: ::std::result::Result<f64, ::std::string::String>,
        window_samples: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechVadDetection {
        fn default() -> Self {
            Self {
                min_silence_ticks: Err("no value supplied for min_silence_ticks".to_string()),
                min_speech_ticks: Err("no value supplied for min_speech_ticks".to_string()),
                speech_pad_ticks: Err("no value supplied for speech_pad_ticks".to_string()),
                threshold: Err("no value supplied for threshold".to_string()),
                window_samples: Err("no value supplied for window_samples".to_string()),
            }
        }
    }
    impl SpeechVadDetection {
        pub fn min_silence_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_silence_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for min_silence_ticks: {e}"));
            self
        }
        pub fn min_speech_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_speech_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for min_speech_ticks: {e}"));
            self
        }
        pub fn speech_pad_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.speech_pad_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for speech_pad_ticks: {e}"));
            self
        }
        pub fn threshold<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.threshold = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for threshold: {e}"));
            self
        }
        pub fn window_samples<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.window_samples = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for window_samples: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SpeechVadDetection> for super::SpeechVadDetection {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechVadDetection,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                min_silence_ticks: value.min_silence_ticks?,
                min_speech_ticks: value.min_speech_ticks?,
                speech_pad_ticks: value.speech_pad_ticks?,
                threshold: value.threshold?,
                window_samples: value.window_samples?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechVadDetection> for SpeechVadDetection {
        fn from(value: super::SpeechVadDetection) -> Self {
            Self {
                min_silence_ticks: Ok(value.min_silence_ticks),
                min_speech_ticks: Ok(value.min_speech_ticks),
                speech_pad_ticks: Ok(value.speech_pad_ticks),
                threshold: Ok(value.threshold),
                window_samples: Ok(value.window_samples),
            }
        }
    }
}
