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
#[doc = "`ComparisonCase`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"broken\","]
#[doc = "    \"broken_first\","]
#[doc = "    \"kind\","]
#[doc = "    \"whole\","]
#[doc = "    \"whole_first\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"broken\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit_span\""]
#[doc = "    },"]
#[doc = "    \"broken_first\": {"]
#[doc = "      \"description\": \"The probability that version A is the better short when A is the broken passage.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"cut_off_ending\","]
#[doc = "        \"dangling_start\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"whole\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit_span\""]
#[doc = "    },"]
#[doc = "    \"whole_first\": {"]
#[doc = "      \"description\": \"The probability that version A is the better short when A is the whole passage.\","]
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
pub struct ComparisonCase {
    pub broken: UnitSpan,
    #[doc = "The probability that version A is the better short when A is the broken passage."]
    pub broken_first: f64,
    pub kind: ComparisonCaseKind,
    pub whole: UnitSpan,
    #[doc = "The probability that version A is the better short when A is the whole passage."]
    pub whole_first: f64,
}
impl ComparisonCase {
    pub fn builder() -> builder::ComparisonCase {
        Default::default()
    }
}
#[doc = "`ComparisonCaseKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"cut_off_ending\","]
#[doc = "    \"dangling_start\""]
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
pub enum ComparisonCaseKind {
    #[serde(rename = "cut_off_ending")]
    CutOffEnding,
    #[serde(rename = "dangling_start")]
    DanglingStart,
}
impl ::std::fmt::Display for ComparisonCaseKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::CutOffEnding => f.write_str("cut_off_ending"),
            Self::DanglingStart => f.write_str("dangling_start"),
        }
    }
}
impl ::std::str::FromStr for ComparisonCaseKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "cut_off_ending" => Ok(Self::CutOffEnding),
            "dangling_start" => Ok(Self::DanglingStart),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ComparisonCaseKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ComparisonCaseKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ComparisonCaseKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`JoinReview`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"after\","]
#[doc = "    \"note\","]
#[doc = "    \"verdict\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"after\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"note\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"verdict\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"pass\","]
#[doc = "        \"reject\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct JoinReview {
    pub after: JoinReviewAfter,
    pub note: JoinReviewNote,
    pub verdict: JoinReviewVerdict,
}
impl JoinReview {
    pub fn builder() -> builder::JoinReview {
        Default::default()
    }
}
#[doc = "`JoinReviewAfter`"]
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
pub struct JoinReviewAfter(::std::string::String);
impl ::std::ops::Deref for JoinReviewAfter {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<JoinReviewAfter> for ::std::string::String {
    fn from(value: JoinReviewAfter) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for JoinReviewAfter {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for JoinReviewAfter {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JoinReviewAfter {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JoinReviewAfter {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for JoinReviewAfter {
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
#[doc = "`JoinReviewNote`"]
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
pub struct JoinReviewNote(::std::string::String);
impl ::std::ops::Deref for JoinReviewNote {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<JoinReviewNote> for ::std::string::String {
    fn from(value: JoinReviewNote) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for JoinReviewNote {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for JoinReviewNote {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JoinReviewNote {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JoinReviewNote {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for JoinReviewNote {
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
#[doc = "`JoinReviewVerdict`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"pass\","]
#[doc = "    \"reject\""]
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
pub enum JoinReviewVerdict {
    #[serde(rename = "pass")]
    Pass,
    #[serde(rename = "reject")]
    Reject,
}
impl ::std::fmt::Display for JoinReviewVerdict {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Pass => f.write_str("pass"),
            Self::Reject => f.write_str("reject"),
        }
    }
}
impl ::std::str::FromStr for JoinReviewVerdict {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "pass" => Ok(Self::Pass),
            "reject" => Ok(Self::Reject),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for JoinReviewVerdict {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JoinReviewVerdict {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JoinReviewVerdict {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Independent review of each proposed cut, including internal removals and the verdict after at most one repair."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.mind.quality_report.v1.json\","]
#[doc = "  \"title\": \"MindQualityReport\","]
#[doc = "  \"description\": \"Independent review of each proposed cut, including internal removals and the verdict after at most one repair.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"inputs\","]
#[doc = "    \"reviews\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"comparison_check\": {"]
#[doc = "      \"description\": \"How the judge that compares an edited clip with its plain version measured on pairs whose answer the recording shows: a passage that ends on a finished thought against the same passage cut off while its speaker goes on, or opened where a line points back at the one before it. Each pair is asked in both orders. The comparison is used only when it passed; without it, an edit whose plain version passed review ships as the plain version. Absent when no edited clip was accepted.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"answered_by\","]
#[doc = "        \"cases\","]
#[doc = "        \"passed\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"answered_by\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"model\","]
#[doc = "            \"agent\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"cases\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/comparison_case\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"passed\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"separation\": {"]
#[doc = "          \"description\": \"The share of (whole version first, broken version first) question pairs scored the right way round: 0.5 is chance, 1 is perfect.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"$ref\": \"#/$defs/sha256\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"questions\": {"]
#[doc = "      \"description\": \"Questions without an answer from a connected agent. An agent answers them through a new run of the same recording; empty or absent when every question was answered.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/question\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"removal_checks\": {"]
#[doc = "      \"description\": \"How the instruments that guard a removal by meaning measured on removals whose effect the recording shows: taking out a question before its answer, or a line the speaker then points back at, leaves something dangling; taking out filler or an abandoned start does not. An instrument is used only when it passed. Absent when no clip had a passage to consider.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"properties\": {"]
#[doc = "        \"judge\": {"]
#[doc = "          \"$ref\": \"#/$defs/removal_check\""]
#[doc = "        },"]
#[doc = "        \"meter\": {"]
#[doc = "          \"$ref\": \"#/$defs/removal_check\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"reviews\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/review\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.mind.quality_report.v1\""]
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
pub struct MindQualityReport {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub comparison_check: ::std::option::Option<MindQualityReportComparisonCheck>,
    pub inputs: ::std::collections::HashMap<::std::string::String, Sha256>,
    #[doc = "Questions without an answer from a connected agent. An agent answers them through a new run of the same recording; empty or absent when every question was answered."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub questions: ::std::vec::Vec<Question>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub removal_checks: ::std::option::Option<MindQualityReportRemovalChecks>,
    pub reviews: ::std::vec::Vec<Review>,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
}
impl MindQualityReport {
    pub fn builder() -> builder::MindQualityReport {
        Default::default()
    }
}
#[doc = "How the judge that compares an edited clip with its plain version measured on pairs whose answer the recording shows: a passage that ends on a finished thought against the same passage cut off while its speaker goes on, or opened where a line points back at the one before it. Each pair is asked in both orders. The comparison is used only when it passed; without it, an edit whose plain version passed review ships as the plain version. Absent when no edited clip was accepted."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How the judge that compares an edited clip with its plain version measured on pairs whose answer the recording shows: a passage that ends on a finished thought against the same passage cut off while its speaker goes on, or opened where a line points back at the one before it. Each pair is asked in both orders. The comparison is used only when it passed; without it, an edit whose plain version passed review ships as the plain version. Absent when no edited clip was accepted.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"answered_by\","]
#[doc = "    \"cases\","]
#[doc = "    \"passed\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"answered_by\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"model\","]
#[doc = "        \"agent\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"cases\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/comparison_case\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"passed\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"separation\": {"]
#[doc = "      \"description\": \"The share of (whole version first, broken version first) question pairs scored the right way round: 0.5 is chance, 1 is perfect.\","]
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
pub struct MindQualityReportComparisonCheck {
    pub answered_by: MindQualityReportComparisonCheckAnsweredBy,
    pub cases: ::std::vec::Vec<ComparisonCase>,
    pub passed: bool,
    #[doc = "The share of (whole version first, broken version first) question pairs scored the right way round: 0.5 is chance, 1 is perfect."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub separation: ::std::option::Option<f64>,
}
impl MindQualityReportComparisonCheck {
    pub fn builder() -> builder::MindQualityReportComparisonCheck {
        Default::default()
    }
}
#[doc = "`MindQualityReportComparisonCheckAnsweredBy`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"model\","]
#[doc = "    \"agent\""]
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
pub enum MindQualityReportComparisonCheckAnsweredBy {
    #[serde(rename = "model")]
    Model,
    #[serde(rename = "agent")]
    Agent,
}
impl ::std::fmt::Display for MindQualityReportComparisonCheckAnsweredBy {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Model => f.write_str("model"),
            Self::Agent => f.write_str("agent"),
        }
    }
}
impl ::std::str::FromStr for MindQualityReportComparisonCheckAnsweredBy {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "model" => Ok(Self::Model),
            "agent" => Ok(Self::Agent),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindQualityReportComparisonCheckAnsweredBy {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String>
    for MindQualityReportComparisonCheckAnsweredBy
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindQualityReportComparisonCheckAnsweredBy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "How the instruments that guard a removal by meaning measured on removals whose effect the recording shows: taking out a question before its answer, or a line the speaker then points back at, leaves something dangling; taking out filler or an abandoned start does not. An instrument is used only when it passed. Absent when no clip had a passage to consider."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How the instruments that guard a removal by meaning measured on removals whose effect the recording shows: taking out a question before its answer, or a line the speaker then points back at, leaves something dangling; taking out filler or an abandoned start does not. An instrument is used only when it passed. Absent when no clip had a passage to consider.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"judge\": {"]
#[doc = "      \"$ref\": \"#/$defs/removal_check\""]
#[doc = "    },"]
#[doc = "    \"meter\": {"]
#[doc = "      \"$ref\": \"#/$defs/removal_check\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindQualityReportRemovalChecks {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub judge: ::std::option::Option<RemovalCheck>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub meter: ::std::option::Option<RemovalCheck>,
}
impl ::std::default::Default for MindQualityReportRemovalChecks {
    fn default() -> Self {
        Self {
            judge: Default::default(),
            meter: Default::default(),
        }
    }
}
impl MindQualityReportRemovalChecks {
    pub fn builder() -> builder::MindQualityReportRemovalChecks {
        Default::default()
    }
}
#[doc = "`PlanId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^plan_[0-9a-f]{16,64}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PlanId(::std::string::String);
impl ::std::ops::Deref for PlanId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PlanId> for ::std::string::String {
    fn from(value: PlanId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for PlanId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^plan_[0-9a-f]{16,64}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^plan_[0-9a-f]{16,64}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PlanId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PlanId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PlanId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PlanId {
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
#[doc = "`Quality`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"hook\","]
#[doc = "    \"issues\","]
#[doc = "    \"joins\","]
#[doc = "    \"repaired\","]
#[doc = "    \"summary\","]
#[doc = "    \"verdict\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"compared\": {"]
#[doc = "      \"description\": \"The edited clip set against its plain version: the longest part played whole, with nothing left out. An edit ships only when the comparison prefers it in both orders, or when the plain version cannot ship.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"kept\","]
#[doc = "        \"plain_plan_id\","]
#[doc = "        \"reason\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"edit_first\": {"]
#[doc = "          \"description\": \"The probability that the edit is the better short, asked with the edit as version A.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"kept\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"edit\","]
#[doc = "            \"plain\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"plain_first\": {"]
#[doc = "          \"description\": \"The probability that the plain version is the better short, asked with it as version A.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"plain_plan_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/plan_id\""]
#[doc = "        },"]
#[doc = "        \"reason\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"edit_preferred\","]
#[doc = "            \"plain_preferred\","]
#[doc = "            \"not_measured\","]
#[doc = "            \"plain_failed_review\","]
#[doc = "            \"plain_out_of_length\","]
#[doc = "            \"awaiting_agent\""]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"hook\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"strong\","]
#[doc = "        \"adequate\","]
#[doc = "        \"weak\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"issues\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"joins\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/join_review\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"repaired\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"summary\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"tightened\": {"]
#[doc = "      \"description\": \"A passage was left out by meaning after the clip was accepted, and the shorter clip was reviewed and accepted in turn.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"verdict\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"accept\","]
#[doc = "        \"reject\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Quality {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub compared: ::std::option::Option<QualityCompared>,
    pub hook: QualityHook,
    pub issues: ::std::vec::Vec<QualityIssuesItem>,
    pub joins: ::std::vec::Vec<JoinReview>,
    pub repaired: bool,
    pub summary: QualitySummary,
    #[doc = "A passage was left out by meaning after the clip was accepted, and the shorter clip was reviewed and accepted in turn."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub tightened: ::std::option::Option<bool>,
    pub verdict: QualityVerdict,
}
impl Quality {
    pub fn builder() -> builder::Quality {
        Default::default()
    }
}
#[doc = "The edited clip set against its plain version: the longest part played whole, with nothing left out. An edit ships only when the comparison prefers it in both orders, or when the plain version cannot ship."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The edited clip set against its plain version: the longest part played whole, with nothing left out. An edit ships only when the comparison prefers it in both orders, or when the plain version cannot ship.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"kept\","]
#[doc = "    \"plain_plan_id\","]
#[doc = "    \"reason\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"edit_first\": {"]
#[doc = "      \"description\": \"The probability that the edit is the better short, asked with the edit as version A.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"kept\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"edit\","]
#[doc = "        \"plain\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"plain_first\": {"]
#[doc = "      \"description\": \"The probability that the plain version is the better short, asked with it as version A.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"plain_plan_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/plan_id\""]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"edit_preferred\","]
#[doc = "        \"plain_preferred\","]
#[doc = "        \"not_measured\","]
#[doc = "        \"plain_failed_review\","]
#[doc = "        \"plain_out_of_length\","]
#[doc = "        \"awaiting_agent\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct QualityCompared {
    #[doc = "The probability that the edit is the better short, asked with the edit as version A."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub edit_first: ::std::option::Option<f64>,
    pub kept: QualityComparedKept,
    #[doc = "The probability that the plain version is the better short, asked with it as version A."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub plain_first: ::std::option::Option<f64>,
    pub plain_plan_id: PlanId,
    pub reason: QualityComparedReason,
}
impl QualityCompared {
    pub fn builder() -> builder::QualityCompared {
        Default::default()
    }
}
#[doc = "`QualityComparedKept`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"edit\","]
#[doc = "    \"plain\""]
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
pub enum QualityComparedKept {
    #[serde(rename = "edit")]
    Edit,
    #[serde(rename = "plain")]
    Plain,
}
impl ::std::fmt::Display for QualityComparedKept {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Edit => f.write_str("edit"),
            Self::Plain => f.write_str("plain"),
        }
    }
}
impl ::std::str::FromStr for QualityComparedKept {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "edit" => Ok(Self::Edit),
            "plain" => Ok(Self::Plain),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for QualityComparedKept {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QualityComparedKept {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QualityComparedKept {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`QualityComparedReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"edit_preferred\","]
#[doc = "    \"plain_preferred\","]
#[doc = "    \"not_measured\","]
#[doc = "    \"plain_failed_review\","]
#[doc = "    \"plain_out_of_length\","]
#[doc = "    \"awaiting_agent\""]
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
pub enum QualityComparedReason {
    #[serde(rename = "edit_preferred")]
    EditPreferred,
    #[serde(rename = "plain_preferred")]
    PlainPreferred,
    #[serde(rename = "not_measured")]
    NotMeasured,
    #[serde(rename = "plain_failed_review")]
    PlainFailedReview,
    #[serde(rename = "plain_out_of_length")]
    PlainOutOfLength,
    #[serde(rename = "awaiting_agent")]
    AwaitingAgent,
}
impl ::std::fmt::Display for QualityComparedReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::EditPreferred => f.write_str("edit_preferred"),
            Self::PlainPreferred => f.write_str("plain_preferred"),
            Self::NotMeasured => f.write_str("not_measured"),
            Self::PlainFailedReview => f.write_str("plain_failed_review"),
            Self::PlainOutOfLength => f.write_str("plain_out_of_length"),
            Self::AwaitingAgent => f.write_str("awaiting_agent"),
        }
    }
}
impl ::std::str::FromStr for QualityComparedReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "edit_preferred" => Ok(Self::EditPreferred),
            "plain_preferred" => Ok(Self::PlainPreferred),
            "not_measured" => Ok(Self::NotMeasured),
            "plain_failed_review" => Ok(Self::PlainFailedReview),
            "plain_out_of_length" => Ok(Self::PlainOutOfLength),
            "awaiting_agent" => Ok(Self::AwaitingAgent),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for QualityComparedReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QualityComparedReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QualityComparedReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`QualityHook`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"strong\","]
#[doc = "    \"adequate\","]
#[doc = "    \"weak\""]
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
pub enum QualityHook {
    #[serde(rename = "strong")]
    Strong,
    #[serde(rename = "adequate")]
    Adequate,
    #[serde(rename = "weak")]
    Weak,
}
impl ::std::fmt::Display for QualityHook {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Strong => f.write_str("strong"),
            Self::Adequate => f.write_str("adequate"),
            Self::Weak => f.write_str("weak"),
        }
    }
}
impl ::std::str::FromStr for QualityHook {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "strong" => Ok(Self::Strong),
            "adequate" => Ok(Self::Adequate),
            "weak" => Ok(Self::Weak),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for QualityHook {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QualityHook {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QualityHook {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`QualityIssuesItem`"]
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
pub struct QualityIssuesItem(::std::string::String);
impl ::std::ops::Deref for QualityIssuesItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<QualityIssuesItem> for ::std::string::String {
    fn from(value: QualityIssuesItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for QualityIssuesItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for QualityIssuesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QualityIssuesItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QualityIssuesItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for QualityIssuesItem {
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
#[doc = "`QualitySummary`"]
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
pub struct QualitySummary(::std::string::String);
impl ::std::ops::Deref for QualitySummary {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<QualitySummary> for ::std::string::String {
    fn from(value: QualitySummary) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for QualitySummary {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for QualitySummary {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QualitySummary {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QualitySummary {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for QualitySummary {
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
#[doc = "`QualityVerdict`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"accept\","]
#[doc = "    \"reject\""]
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
pub enum QualityVerdict {
    #[serde(rename = "accept")]
    Accept,
    #[serde(rename = "reject")]
    Reject,
}
impl ::std::fmt::Display for QualityVerdict {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Accept => f.write_str("accept"),
            Self::Reject => f.write_str("reject"),
        }
    }
}
impl ::std::str::FromStr for QualityVerdict {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "accept" => Ok(Self::Accept),
            "reject" => Ok(Self::Reject),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for QualityVerdict {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QualityVerdict {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QualityVerdict {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "A narrow question this run asked, or would ask, of its judge, for a connected agent to answer. The list never says which questions are known cases of a self-check: an agent's answers are measured on those before any of them is used."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A narrow question this run asked, or would ask, of its judge, for a connected agent to answer. The list never says which questions are known cases of a self-check: an agent's answers are measured on those before any of them is used.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"kind\","]
#[doc = "    \"question_id\","]
#[doc = "    \"text\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"kind\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"judge.ending\","]
#[doc = "        \"judge.removal\","]
#[doc = "        \"judge.compare\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"question_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^q_[0-9a-f]{24}$\""]
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
pub struct Question {
    pub kind: QuestionKind,
    pub question_id: QuestionQuestionId,
    pub text: QuestionText,
}
impl Question {
    pub fn builder() -> builder::Question {
        Default::default()
    }
}
#[doc = "`QuestionKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"judge.ending\","]
#[doc = "    \"judge.removal\","]
#[doc = "    \"judge.compare\""]
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
pub enum QuestionKind {
    #[serde(rename = "judge.ending")]
    JudgeEnding,
    #[serde(rename = "judge.removal")]
    JudgeRemoval,
    #[serde(rename = "judge.compare")]
    JudgeCompare,
}
impl ::std::fmt::Display for QuestionKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::JudgeEnding => f.write_str("judge.ending"),
            Self::JudgeRemoval => f.write_str("judge.removal"),
            Self::JudgeCompare => f.write_str("judge.compare"),
        }
    }
}
impl ::std::str::FromStr for QuestionKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "judge.ending" => Ok(Self::JudgeEnding),
            "judge.removal" => Ok(Self::JudgeRemoval),
            "judge.compare" => Ok(Self::JudgeCompare),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for QuestionKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QuestionKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QuestionKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`QuestionQuestionId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^q_[0-9a-f]{24}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct QuestionQuestionId(::std::string::String);
impl ::std::ops::Deref for QuestionQuestionId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<QuestionQuestionId> for ::std::string::String {
    fn from(value: QuestionQuestionId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for QuestionQuestionId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^q_[0-9a-f]{24}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^q_[0-9a-f]{24}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for QuestionQuestionId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QuestionQuestionId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QuestionQuestionId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for QuestionQuestionId {
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
#[doc = "`QuestionText`"]
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
pub struct QuestionText(::std::string::String);
impl ::std::ops::Deref for QuestionText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<QuestionText> for ::std::string::String {
    fn from(value: QuestionText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for QuestionText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for QuestionText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for QuestionText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for QuestionText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for QuestionText {
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
#[doc = "`RemovalCheck`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cases\","]
#[doc = "    \"passed\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cases\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"expected\","]
#[doc = "          \"first_unit\","]
#[doc = "          \"last_unit\","]
#[doc = "          \"score\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"expected\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"dangling\","]
#[doc = "              \"intact\""]
#[doc = "            ]"]
#[doc = "          },"]
#[doc = "          \"first_unit\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"pattern\": \"^s[0-9]+$\""]
#[doc = "          },"]
#[doc = "          \"last_unit\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"pattern\": \"^s[0-9]+$\""]
#[doc = "          },"]
#[doc = "          \"score\": {"]
#[doc = "            \"description\": \"How much the instrument says the removal leaves dangling: for the meter, how much more surprising the next line becomes per token; for the judge, the probability it sees something missing.\","]
#[doc = "            \"type\": \"number\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"passed\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"separation\": {"]
#[doc = "      \"description\": \"The share of (dangling, intact) pairs the instrument scored the right way round: 0.5 is chance, 1 is perfect.\","]
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
pub struct RemovalCheck {
    pub cases: ::std::vec::Vec<RemovalCheckCasesItem>,
    pub passed: bool,
    #[doc = "The share of (dangling, intact) pairs the instrument scored the right way round: 0.5 is chance, 1 is perfect."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub separation: ::std::option::Option<f64>,
}
impl RemovalCheck {
    pub fn builder() -> builder::RemovalCheck {
        Default::default()
    }
}
#[doc = "`RemovalCheckCasesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"expected\","]
#[doc = "    \"first_unit\","]
#[doc = "    \"last_unit\","]
#[doc = "    \"score\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"expected\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"dangling\","]
#[doc = "        \"intact\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"first_unit\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^s[0-9]+$\""]
#[doc = "    },"]
#[doc = "    \"last_unit\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^s[0-9]+$\""]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"description\": \"How much the instrument says the removal leaves dangling: for the meter, how much more surprising the next line becomes per token; for the judge, the probability it sees something missing.\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RemovalCheckCasesItem {
    pub expected: RemovalCheckCasesItemExpected,
    pub first_unit: RemovalCheckCasesItemFirstUnit,
    pub last_unit: RemovalCheckCasesItemLastUnit,
    #[doc = "How much the instrument says the removal leaves dangling: for the meter, how much more surprising the next line becomes per token; for the judge, the probability it sees something missing."]
    pub score: f64,
}
impl RemovalCheckCasesItem {
    pub fn builder() -> builder::RemovalCheckCasesItem {
        Default::default()
    }
}
#[doc = "`RemovalCheckCasesItemExpected`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"dangling\","]
#[doc = "    \"intact\""]
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
pub enum RemovalCheckCasesItemExpected {
    #[serde(rename = "dangling")]
    Dangling,
    #[serde(rename = "intact")]
    Intact,
}
impl ::std::fmt::Display for RemovalCheckCasesItemExpected {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Dangling => f.write_str("dangling"),
            Self::Intact => f.write_str("intact"),
        }
    }
}
impl ::std::str::FromStr for RemovalCheckCasesItemExpected {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "dangling" => Ok(Self::Dangling),
            "intact" => Ok(Self::Intact),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for RemovalCheckCasesItemExpected {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RemovalCheckCasesItemExpected {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RemovalCheckCasesItemExpected {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`RemovalCheckCasesItemFirstUnit`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^s[0-9]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RemovalCheckCasesItemFirstUnit(::std::string::String);
impl ::std::ops::Deref for RemovalCheckCasesItemFirstUnit {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RemovalCheckCasesItemFirstUnit> for ::std::string::String {
    fn from(value: RemovalCheckCasesItemFirstUnit) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RemovalCheckCasesItemFirstUnit {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^s[0-9]+$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^s[0-9]+$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RemovalCheckCasesItemFirstUnit {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RemovalCheckCasesItemFirstUnit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RemovalCheckCasesItemFirstUnit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RemovalCheckCasesItemFirstUnit {
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
#[doc = "`RemovalCheckCasesItemLastUnit`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^s[0-9]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RemovalCheckCasesItemLastUnit(::std::string::String);
impl ::std::ops::Deref for RemovalCheckCasesItemLastUnit {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RemovalCheckCasesItemLastUnit> for ::std::string::String {
    fn from(value: RemovalCheckCasesItemLastUnit) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RemovalCheckCasesItemLastUnit {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^s[0-9]+$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^s[0-9]+$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RemovalCheckCasesItemLastUnit {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RemovalCheckCasesItemLastUnit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RemovalCheckCasesItemLastUnit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RemovalCheckCasesItemLastUnit {
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
#[doc = "`Review`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidate_id\","]
#[doc = "    \"chapter\","]
#[doc = "    \"plan\","]
#[doc = "    \"quality\","]
#[doc = "    \"reasons\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidate_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/plan_id\""]
#[doc = "    },"]
#[doc = "    \"chapter\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"plan\": {"]
#[doc = "      \"description\": \"A clipmill.mind.clip_plan.v1 document with its validated source geometry.\","]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"quality\": {"]
#[doc = "      \"$ref\": \"#/$defs/quality\""]
#[doc = "    },"]
#[doc = "    \"reasons\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
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
pub struct Review {
    pub candidate_id: PlanId,
    pub chapter: ReviewChapter,
    #[doc = "A clipmill.mind.clip_plan.v1 document with its validated source geometry."]
    pub plan: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    pub quality: Quality,
    pub reasons: ::std::vec::Vec<ReviewReasonsItem>,
}
impl Review {
    pub fn builder() -> builder::Review {
        Default::default()
    }
}
#[doc = "`ReviewChapter`"]
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
pub struct ReviewChapter(::std::string::String);
impl ::std::ops::Deref for ReviewChapter {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ReviewChapter> for ::std::string::String {
    fn from(value: ReviewChapter) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ReviewChapter {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ReviewChapter {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ReviewChapter {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ReviewChapter {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ReviewChapter {
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
#[doc = "`ReviewReasonsItem`"]
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
pub struct ReviewReasonsItem(::std::string::String);
impl ::std::ops::Deref for ReviewReasonsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ReviewReasonsItem> for ::std::string::String {
    fn from(value: ReviewReasonsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ReviewReasonsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ReviewReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ReviewReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ReviewReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ReviewReasonsItem {
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
#[doc = "`UnitSpan`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"first_unit\","]
#[doc = "    \"last_unit\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"first_unit\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^s[0-9]+$\""]
#[doc = "    },"]
#[doc = "    \"last_unit\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^s[0-9]+$\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UnitSpan {
    pub first_unit: UnitSpanFirstUnit,
    pub last_unit: UnitSpanLastUnit,
}
impl UnitSpan {
    pub fn builder() -> builder::UnitSpan {
        Default::default()
    }
}
#[doc = "`UnitSpanFirstUnit`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^s[0-9]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct UnitSpanFirstUnit(::std::string::String);
impl ::std::ops::Deref for UnitSpanFirstUnit {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<UnitSpanFirstUnit> for ::std::string::String {
    fn from(value: UnitSpanFirstUnit) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for UnitSpanFirstUnit {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^s[0-9]+$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^s[0-9]+$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for UnitSpanFirstUnit {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for UnitSpanFirstUnit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UnitSpanFirstUnit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for UnitSpanFirstUnit {
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
#[doc = "`UnitSpanLastUnit`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^s[0-9]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct UnitSpanLastUnit(::std::string::String);
impl ::std::ops::Deref for UnitSpanLastUnit {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<UnitSpanLastUnit> for ::std::string::String {
    fn from(value: UnitSpanLastUnit) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for UnitSpanLastUnit {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^s[0-9]+$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^s[0-9]+$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for UnitSpanLastUnit {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for UnitSpanLastUnit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UnitSpanLastUnit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for UnitSpanLastUnit {
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
    pub struct ComparisonCase {
        broken: ::std::result::Result<super::UnitSpan, ::std::string::String>,
        broken_first: ::std::result::Result<f64, ::std::string::String>,
        kind: ::std::result::Result<super::ComparisonCaseKind, ::std::string::String>,
        whole: ::std::result::Result<super::UnitSpan, ::std::string::String>,
        whole_first: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for ComparisonCase {
        fn default() -> Self {
            Self {
                broken: Err("no value supplied for broken".to_string()),
                broken_first: Err("no value supplied for broken_first".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                whole: Err("no value supplied for whole".to_string()),
                whole_first: Err("no value supplied for whole_first".to_string()),
            }
        }
    }
    impl ComparisonCase {
        pub fn broken<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UnitSpan>,
            T::Error: ::std::fmt::Display,
        {
            self.broken = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for broken: {e}"));
            self
        }
        pub fn broken_first<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.broken_first = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for broken_first: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ComparisonCaseKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn whole<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UnitSpan>,
            T::Error: ::std::fmt::Display,
        {
            self.whole = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for whole: {e}"));
            self
        }
        pub fn whole_first<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.whole_first = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for whole_first: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ComparisonCase> for super::ComparisonCase {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ComparisonCase,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                broken: value.broken?,
                broken_first: value.broken_first?,
                kind: value.kind?,
                whole: value.whole?,
                whole_first: value.whole_first?,
            })
        }
    }
    impl ::std::convert::From<super::ComparisonCase> for ComparisonCase {
        fn from(value: super::ComparisonCase) -> Self {
            Self {
                broken: Ok(value.broken),
                broken_first: Ok(value.broken_first),
                kind: Ok(value.kind),
                whole: Ok(value.whole),
                whole_first: Ok(value.whole_first),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct JoinReview {
        after: ::std::result::Result<super::JoinReviewAfter, ::std::string::String>,
        note: ::std::result::Result<super::JoinReviewNote, ::std::string::String>,
        verdict: ::std::result::Result<super::JoinReviewVerdict, ::std::string::String>,
    }
    impl ::std::default::Default for JoinReview {
        fn default() -> Self {
            Self {
                after: Err("no value supplied for after".to_string()),
                note: Err("no value supplied for note".to_string()),
                verdict: Err("no value supplied for verdict".to_string()),
            }
        }
    }
    impl JoinReview {
        pub fn after<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JoinReviewAfter>,
            T::Error: ::std::fmt::Display,
        {
            self.after = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for after: {e}"));
            self
        }
        pub fn note<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JoinReviewNote>,
            T::Error: ::std::fmt::Display,
        {
            self.note = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for note: {e}"));
            self
        }
        pub fn verdict<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JoinReviewVerdict>,
            T::Error: ::std::fmt::Display,
        {
            self.verdict = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for verdict: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<JoinReview> for super::JoinReview {
        type Error = super::error::ConversionError;
        fn try_from(
            value: JoinReview,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                after: value.after?,
                note: value.note?,
                verdict: value.verdict?,
            })
        }
    }
    impl ::std::convert::From<super::JoinReview> for JoinReview {
        fn from(value: super::JoinReview) -> Self {
            Self {
                after: Ok(value.after),
                note: Ok(value.note),
                verdict: Ok(value.verdict),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindQualityReport {
        comparison_check: ::std::result::Result<
            ::std::option::Option<super::MindQualityReportComparisonCheck>,
            ::std::string::String,
        >,
        inputs: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, super::Sha256>,
            ::std::string::String,
        >,
        questions: ::std::result::Result<::std::vec::Vec<super::Question>, ::std::string::String>,
        removal_checks: ::std::result::Result<
            ::std::option::Option<super::MindQualityReportRemovalChecks>,
            ::std::string::String,
        >,
        reviews: ::std::result::Result<::std::vec::Vec<super::Review>, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for MindQualityReport {
        fn default() -> Self {
            Self {
                comparison_check: Ok(Default::default()),
                inputs: Err("no value supplied for inputs".to_string()),
                questions: Ok(Default::default()),
                removal_checks: Ok(Default::default()),
                reviews: Err("no value supplied for reviews".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl MindQualityReport {
        pub fn comparison_check<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::option::Option<super::MindQualityReportComparisonCheck>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.comparison_check = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for comparison_check: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::collections::HashMap<::std::string::String, super::Sha256>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
            self
        }
        pub fn questions<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Question>>,
            T::Error: ::std::fmt::Display,
        {
            self.questions = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for questions: {e}"));
            self
        }
        pub fn removal_checks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::option::Option<super::MindQualityReportRemovalChecks>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.removal_checks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for removal_checks: {e}"));
            self
        }
        pub fn reviews<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Review>>,
            T::Error: ::std::fmt::Display,
        {
            self.reviews = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reviews: {e}"));
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
    impl ::std::convert::TryFrom<MindQualityReport> for super::MindQualityReport {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindQualityReport,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                comparison_check: value.comparison_check?,
                inputs: value.inputs?,
                questions: value.questions?,
                removal_checks: value.removal_checks?,
                reviews: value.reviews?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::MindQualityReport> for MindQualityReport {
        fn from(value: super::MindQualityReport) -> Self {
            Self {
                comparison_check: Ok(value.comparison_check),
                inputs: Ok(value.inputs),
                questions: Ok(value.questions),
                removal_checks: Ok(value.removal_checks),
                reviews: Ok(value.reviews),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindQualityReportComparisonCheck {
        answered_by: ::std::result::Result<
            super::MindQualityReportComparisonCheckAnsweredBy,
            ::std::string::String,
        >,
        cases: ::std::result::Result<::std::vec::Vec<super::ComparisonCase>, ::std::string::String>,
        passed: ::std::result::Result<bool, ::std::string::String>,
        separation: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for MindQualityReportComparisonCheck {
        fn default() -> Self {
            Self {
                answered_by: Err("no value supplied for answered_by".to_string()),
                cases: Err("no value supplied for cases".to_string()),
                passed: Err("no value supplied for passed".to_string()),
                separation: Ok(Default::default()),
            }
        }
    }
    impl MindQualityReportComparisonCheck {
        pub fn answered_by<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindQualityReportComparisonCheckAnsweredBy>,
            T::Error: ::std::fmt::Display,
        {
            self.answered_by = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for answered_by: {e}"));
            self
        }
        pub fn cases<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ComparisonCase>>,
            T::Error: ::std::fmt::Display,
        {
            self.cases = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cases: {e}"));
            self
        }
        pub fn passed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.passed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for passed: {e}"));
            self
        }
        pub fn separation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.separation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for separation: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindQualityReportComparisonCheck>
        for super::MindQualityReportComparisonCheck
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindQualityReportComparisonCheck,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                answered_by: value.answered_by?,
                cases: value.cases?,
                passed: value.passed?,
                separation: value.separation?,
            })
        }
    }
    impl ::std::convert::From<super::MindQualityReportComparisonCheck>
        for MindQualityReportComparisonCheck
    {
        fn from(value: super::MindQualityReportComparisonCheck) -> Self {
            Self {
                answered_by: Ok(value.answered_by),
                cases: Ok(value.cases),
                passed: Ok(value.passed),
                separation: Ok(value.separation),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindQualityReportRemovalChecks {
        judge: ::std::result::Result<
            ::std::option::Option<super::RemovalCheck>,
            ::std::string::String,
        >,
        meter: ::std::result::Result<
            ::std::option::Option<super::RemovalCheck>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for MindQualityReportRemovalChecks {
        fn default() -> Self {
            Self {
                judge: Ok(Default::default()),
                meter: Ok(Default::default()),
            }
        }
    }
    impl MindQualityReportRemovalChecks {
        pub fn judge<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::RemovalCheck>>,
            T::Error: ::std::fmt::Display,
        {
            self.judge = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for judge: {e}"));
            self
        }
        pub fn meter<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::RemovalCheck>>,
            T::Error: ::std::fmt::Display,
        {
            self.meter = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for meter: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindQualityReportRemovalChecks>
        for super::MindQualityReportRemovalChecks
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindQualityReportRemovalChecks,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                judge: value.judge?,
                meter: value.meter?,
            })
        }
    }
    impl ::std::convert::From<super::MindQualityReportRemovalChecks>
        for MindQualityReportRemovalChecks
    {
        fn from(value: super::MindQualityReportRemovalChecks) -> Self {
            Self {
                judge: Ok(value.judge),
                meter: Ok(value.meter),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Quality {
        compared: ::std::result::Result<
            ::std::option::Option<super::QualityCompared>,
            ::std::string::String,
        >,
        hook: ::std::result::Result<super::QualityHook, ::std::string::String>,
        issues:
            ::std::result::Result<::std::vec::Vec<super::QualityIssuesItem>, ::std::string::String>,
        joins: ::std::result::Result<::std::vec::Vec<super::JoinReview>, ::std::string::String>,
        repaired: ::std::result::Result<bool, ::std::string::String>,
        summary: ::std::result::Result<super::QualitySummary, ::std::string::String>,
        tightened: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        verdict: ::std::result::Result<super::QualityVerdict, ::std::string::String>,
    }
    impl ::std::default::Default for Quality {
        fn default() -> Self {
            Self {
                compared: Ok(Default::default()),
                hook: Err("no value supplied for hook".to_string()),
                issues: Err("no value supplied for issues".to_string()),
                joins: Err("no value supplied for joins".to_string()),
                repaired: Err("no value supplied for repaired".to_string()),
                summary: Err("no value supplied for summary".to_string()),
                tightened: Ok(Default::default()),
                verdict: Err("no value supplied for verdict".to_string()),
            }
        }
    }
    impl Quality {
        pub fn compared<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::QualityCompared>>,
            T::Error: ::std::fmt::Display,
        {
            self.compared = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for compared: {e}"));
            self
        }
        pub fn hook<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::QualityHook>,
            T::Error: ::std::fmt::Display,
        {
            self.hook = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hook: {e}"));
            self
        }
        pub fn issues<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::QualityIssuesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.issues = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for issues: {e}"));
            self
        }
        pub fn joins<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::JoinReview>>,
            T::Error: ::std::fmt::Display,
        {
            self.joins = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for joins: {e}"));
            self
        }
        pub fn repaired<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.repaired = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for repaired: {e}"));
            self
        }
        pub fn summary<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::QualitySummary>,
            T::Error: ::std::fmt::Display,
        {
            self.summary = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for summary: {e}"));
            self
        }
        pub fn tightened<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.tightened = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tightened: {e}"));
            self
        }
        pub fn verdict<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::QualityVerdict>,
            T::Error: ::std::fmt::Display,
        {
            self.verdict = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for verdict: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Quality> for super::Quality {
        type Error = super::error::ConversionError;
        fn try_from(value: Quality) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                compared: value.compared?,
                hook: value.hook?,
                issues: value.issues?,
                joins: value.joins?,
                repaired: value.repaired?,
                summary: value.summary?,
                tightened: value.tightened?,
                verdict: value.verdict?,
            })
        }
    }
    impl ::std::convert::From<super::Quality> for Quality {
        fn from(value: super::Quality) -> Self {
            Self {
                compared: Ok(value.compared),
                hook: Ok(value.hook),
                issues: Ok(value.issues),
                joins: Ok(value.joins),
                repaired: Ok(value.repaired),
                summary: Ok(value.summary),
                tightened: Ok(value.tightened),
                verdict: Ok(value.verdict),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct QualityCompared {
        edit_first: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        kept: ::std::result::Result<super::QualityComparedKept, ::std::string::String>,
        plain_first: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        plain_plan_id: ::std::result::Result<super::PlanId, ::std::string::String>,
        reason: ::std::result::Result<super::QualityComparedReason, ::std::string::String>,
    }
    impl ::std::default::Default for QualityCompared {
        fn default() -> Self {
            Self {
                edit_first: Ok(Default::default()),
                kept: Err("no value supplied for kept".to_string()),
                plain_first: Ok(Default::default()),
                plain_plan_id: Err("no value supplied for plain_plan_id".to_string()),
                reason: Err("no value supplied for reason".to_string()),
            }
        }
    }
    impl QualityCompared {
        pub fn edit_first<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.edit_first = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for edit_first: {e}"));
            self
        }
        pub fn kept<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::QualityComparedKept>,
            T::Error: ::std::fmt::Display,
        {
            self.kept = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kept: {e}"));
            self
        }
        pub fn plain_first<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.plain_first = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for plain_first: {e}"));
            self
        }
        pub fn plain_plan_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PlanId>,
            T::Error: ::std::fmt::Display,
        {
            self.plain_plan_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for plain_plan_id: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::QualityComparedReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<QualityCompared> for super::QualityCompared {
        type Error = super::error::ConversionError;
        fn try_from(
            value: QualityCompared,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                edit_first: value.edit_first?,
                kept: value.kept?,
                plain_first: value.plain_first?,
                plain_plan_id: value.plain_plan_id?,
                reason: value.reason?,
            })
        }
    }
    impl ::std::convert::From<super::QualityCompared> for QualityCompared {
        fn from(value: super::QualityCompared) -> Self {
            Self {
                edit_first: Ok(value.edit_first),
                kept: Ok(value.kept),
                plain_first: Ok(value.plain_first),
                plain_plan_id: Ok(value.plain_plan_id),
                reason: Ok(value.reason),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Question {
        kind: ::std::result::Result<super::QuestionKind, ::std::string::String>,
        question_id: ::std::result::Result<super::QuestionQuestionId, ::std::string::String>,
        text: ::std::result::Result<super::QuestionText, ::std::string::String>,
    }
    impl ::std::default::Default for Question {
        fn default() -> Self {
            Self {
                kind: Err("no value supplied for kind".to_string()),
                question_id: Err("no value supplied for question_id".to_string()),
                text: Err("no value supplied for text".to_string()),
            }
        }
    }
    impl Question {
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::QuestionKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn question_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::QuestionQuestionId>,
            T::Error: ::std::fmt::Display,
        {
            self.question_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for question_id: {e}"));
            self
        }
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::QuestionText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Question> for super::Question {
        type Error = super::error::ConversionError;
        fn try_from(value: Question) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                kind: value.kind?,
                question_id: value.question_id?,
                text: value.text?,
            })
        }
    }
    impl ::std::convert::From<super::Question> for Question {
        fn from(value: super::Question) -> Self {
            Self {
                kind: Ok(value.kind),
                question_id: Ok(value.question_id),
                text: Ok(value.text),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RemovalCheck {
        cases: ::std::result::Result<
            ::std::vec::Vec<super::RemovalCheckCasesItem>,
            ::std::string::String,
        >,
        passed: ::std::result::Result<bool, ::std::string::String>,
        separation: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for RemovalCheck {
        fn default() -> Self {
            Self {
                cases: Err("no value supplied for cases".to_string()),
                passed: Err("no value supplied for passed".to_string()),
                separation: Ok(Default::default()),
            }
        }
    }
    impl RemovalCheck {
        pub fn cases<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::RemovalCheckCasesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.cases = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cases: {e}"));
            self
        }
        pub fn passed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.passed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for passed: {e}"));
            self
        }
        pub fn separation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.separation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for separation: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RemovalCheck> for super::RemovalCheck {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RemovalCheck,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cases: value.cases?,
                passed: value.passed?,
                separation: value.separation?,
            })
        }
    }
    impl ::std::convert::From<super::RemovalCheck> for RemovalCheck {
        fn from(value: super::RemovalCheck) -> Self {
            Self {
                cases: Ok(value.cases),
                passed: Ok(value.passed),
                separation: Ok(value.separation),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RemovalCheckCasesItem {
        expected:
            ::std::result::Result<super::RemovalCheckCasesItemExpected, ::std::string::String>,
        first_unit:
            ::std::result::Result<super::RemovalCheckCasesItemFirstUnit, ::std::string::String>,
        last_unit:
            ::std::result::Result<super::RemovalCheckCasesItemLastUnit, ::std::string::String>,
        score: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for RemovalCheckCasesItem {
        fn default() -> Self {
            Self {
                expected: Err("no value supplied for expected".to_string()),
                first_unit: Err("no value supplied for first_unit".to_string()),
                last_unit: Err("no value supplied for last_unit".to_string()),
                score: Err("no value supplied for score".to_string()),
            }
        }
    }
    impl RemovalCheckCasesItem {
        pub fn expected<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RemovalCheckCasesItemExpected>,
            T::Error: ::std::fmt::Display,
        {
            self.expected = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for expected: {e}"));
            self
        }
        pub fn first_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RemovalCheckCasesItemFirstUnit>,
            T::Error: ::std::fmt::Display,
        {
            self.first_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_unit: {e}"));
            self
        }
        pub fn last_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RemovalCheckCasesItemLastUnit>,
            T::Error: ::std::fmt::Display,
        {
            self.last_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for last_unit: {e}"));
            self
        }
        pub fn score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.score = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for score: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RemovalCheckCasesItem> for super::RemovalCheckCasesItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RemovalCheckCasesItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                expected: value.expected?,
                first_unit: value.first_unit?,
                last_unit: value.last_unit?,
                score: value.score?,
            })
        }
    }
    impl ::std::convert::From<super::RemovalCheckCasesItem> for RemovalCheckCasesItem {
        fn from(value: super::RemovalCheckCasesItem) -> Self {
            Self {
                expected: Ok(value.expected),
                first_unit: Ok(value.first_unit),
                last_unit: Ok(value.last_unit),
                score: Ok(value.score),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Review {
        candidate_id: ::std::result::Result<super::PlanId, ::std::string::String>,
        chapter: ::std::result::Result<super::ReviewChapter, ::std::string::String>,
        plan: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        quality: ::std::result::Result<super::Quality, ::std::string::String>,
        reasons:
            ::std::result::Result<::std::vec::Vec<super::ReviewReasonsItem>, ::std::string::String>,
    }
    impl ::std::default::Default for Review {
        fn default() -> Self {
            Self {
                candidate_id: Err("no value supplied for candidate_id".to_string()),
                chapter: Err("no value supplied for chapter".to_string()),
                plan: Err("no value supplied for plan".to_string()),
                quality: Err("no value supplied for quality".to_string()),
                reasons: Err("no value supplied for reasons".to_string()),
            }
        }
    }
    impl Review {
        pub fn candidate_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PlanId>,
            T::Error: ::std::fmt::Display,
        {
            self.candidate_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidate_id: {e}"));
            self
        }
        pub fn chapter<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ReviewChapter>,
            T::Error: ::std::fmt::Display,
        {
            self.chapter = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for chapter: {e}"));
            self
        }
        pub fn plan<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::serde_json::Map<::std::string::String, ::serde_json::Value>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.plan = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for plan: {e}"));
            self
        }
        pub fn quality<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Quality>,
            T::Error: ::std::fmt::Display,
        {
            self.quality = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for quality: {e}"));
            self
        }
        pub fn reasons<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ReviewReasonsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.reasons = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reasons: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Review> for super::Review {
        type Error = super::error::ConversionError;
        fn try_from(value: Review) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidate_id: value.candidate_id?,
                chapter: value.chapter?,
                plan: value.plan?,
                quality: value.quality?,
                reasons: value.reasons?,
            })
        }
    }
    impl ::std::convert::From<super::Review> for Review {
        fn from(value: super::Review) -> Self {
            Self {
                candidate_id: Ok(value.candidate_id),
                chapter: Ok(value.chapter),
                plan: Ok(value.plan),
                quality: Ok(value.quality),
                reasons: Ok(value.reasons),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct UnitSpan {
        first_unit: ::std::result::Result<super::UnitSpanFirstUnit, ::std::string::String>,
        last_unit: ::std::result::Result<super::UnitSpanLastUnit, ::std::string::String>,
    }
    impl ::std::default::Default for UnitSpan {
        fn default() -> Self {
            Self {
                first_unit: Err("no value supplied for first_unit".to_string()),
                last_unit: Err("no value supplied for last_unit".to_string()),
            }
        }
    }
    impl UnitSpan {
        pub fn first_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UnitSpanFirstUnit>,
            T::Error: ::std::fmt::Display,
        {
            self.first_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_unit: {e}"));
            self
        }
        pub fn last_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UnitSpanLastUnit>,
            T::Error: ::std::fmt::Display,
        {
            self.last_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for last_unit: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<UnitSpan> for super::UnitSpan {
        type Error = super::error::ConversionError;
        fn try_from(value: UnitSpan) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                first_unit: value.first_unit?,
                last_unit: value.last_unit?,
            })
        }
    }
    impl ::std::convert::From<super::UnitSpan> for UnitSpan {
        fn from(value: super::UnitSpan) -> Self {
            Self {
                first_unit: Ok(value.first_unit),
                last_unit: Ok(value.last_unit),
            }
        }
    }
}
