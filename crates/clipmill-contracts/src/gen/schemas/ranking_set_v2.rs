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
#[doc = "`Comparison`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"a\","]
#[doc = "    \"b\","]
#[doc = "    \"reasons\","]
#[doc = "    \"share_a\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"a\": {"]
#[doc = "      \"$ref\": \"#/$defs/plan_id\""]
#[doc = "    },"]
#[doc = "    \"b\": {"]
#[doc = "      \"$ref\": \"#/$defs/plan_id\""]
#[doc = "    },"]
#[doc = "    \"reasons\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      },"]
#[doc = "      \"maxItems\": 2,"]
#[doc = "      \"minItems\": 2"]
#[doc = "    },"]
#[doc = "    \"share_a\": {"]
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
pub struct Comparison {
    pub a: PlanId,
    pub b: PlanId,
    pub reasons: [ComparisonReasonsItem; 2usize],
    pub share_a: f64,
}
impl Comparison {
    pub fn builder() -> builder::Comparison {
        Default::default()
    }
}
#[doc = "`ComparisonReasonsItem`"]
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
pub struct ComparisonReasonsItem(::std::string::String);
impl ::std::ops::Deref for ComparisonReasonsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ComparisonReasonsItem> for ::std::string::String {
    fn from(value: ComparisonReasonsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ComparisonReasonsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ComparisonReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ComparisonReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ComparisonReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ComparisonReasonsItem {
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
    pub hook: QualityHook,
    pub issues: ::std::vec::Vec<QualityIssuesItem>,
    pub joins: ::std::vec::Vec<JoinReview>,
    pub repaired: bool,
    pub summary: QualitySummary,
    pub verdict: QualityVerdict,
}
impl Quality {
    pub fn builder() -> builder::Quality {
        Default::default()
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
#[doc = "`Ranked`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"chapter\","]
#[doc = "    \"keep_score\","]
#[doc = "    \"plan\","]
#[doc = "    \"quality\","]
#[doc = "    \"reasons\","]
#[doc = "    \"strength\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"chapter\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"keep_score\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 100.0,"]
#[doc = "      \"minimum\": 0.0"]
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
#[doc = "    },"]
#[doc = "    \"strength\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Ranked {
    pub chapter: RankedChapter,
    pub keep_score: i64,
    #[doc = "A clipmill.mind.clip_plan.v1 document with its validated source geometry."]
    pub plan: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    pub quality: Quality,
    pub reasons: ::std::vec::Vec<RankedReasonsItem>,
    pub strength: f64,
}
impl Ranked {
    pub fn builder() -> builder::Ranked {
        Default::default()
    }
}
#[doc = "`RankedChapter`"]
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
pub struct RankedChapter(::std::string::String);
impl ::std::ops::Deref for RankedChapter {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankedChapter> for ::std::string::String {
    fn from(value: RankedChapter) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankedChapter {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RankedChapter {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankedChapter {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankedChapter {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankedChapter {
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
#[doc = "`RankedReasonsItem`"]
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
pub struct RankedReasonsItem(::std::string::String);
impl ::std::ops::Deref for RankedReasonsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankedReasonsItem> for ::std::string::String {
    fn from(value: RankedReasonsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankedReasonsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RankedReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankedReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankedReasonsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankedReasonsItem {
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
#[doc = "Distinct independently reviewed plans in editorial order, with counterbalanced comparisons, Bradley–Terry strengths and a reason for any shortfall."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.ranking.set.v2.json\","]
#[doc = "  \"title\": \"RankingSetV2\","]
#[doc = "  \"description\": \"Distinct independently reviewed plans in editorial order, with counterbalanced comparisons, Bradley–Terry strengths and a reason for any shortfall.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"clips\","]
#[doc = "    \"comparisons\","]
#[doc = "    \"editorial_prompt\","]
#[doc = "    \"inputs\","]
#[doc = "    \"requested_count\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"clips\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/ranked\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"comparisons\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/comparison\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"cutoff_reason\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"editorial_prompt\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 1000"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"$ref\": \"#/$defs/sha256\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"requested_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 100.0,"]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.ranking.set.v2\""]
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
pub struct RankingSetV2 {
    pub clips: ::std::vec::Vec<Ranked>,
    pub comparisons: ::std::vec::Vec<Comparison>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub cutoff_reason: ::std::option::Option<RankingSetV2CutoffReason>,
    pub editorial_prompt: RankingSetV2EditorialPrompt,
    pub inputs: ::std::collections::HashMap<::std::string::String, Sha256>,
    pub requested_count: ::std::num::NonZeroU64,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
}
impl RankingSetV2 {
    pub fn builder() -> builder::RankingSetV2 {
        Default::default()
    }
}
#[doc = "`RankingSetV2CutoffReason`"]
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
pub struct RankingSetV2CutoffReason(::std::string::String);
impl ::std::ops::Deref for RankingSetV2CutoffReason {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankingSetV2CutoffReason> for ::std::string::String {
    fn from(value: RankingSetV2CutoffReason) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankingSetV2CutoffReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RankingSetV2CutoffReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankingSetV2CutoffReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankingSetV2CutoffReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankingSetV2CutoffReason {
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
#[doc = "`RankingSetV2EditorialPrompt`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 1000"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RankingSetV2EditorialPrompt(::std::string::String);
impl ::std::ops::Deref for RankingSetV2EditorialPrompt {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankingSetV2EditorialPrompt> for ::std::string::String {
    fn from(value: RankingSetV2EditorialPrompt) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankingSetV2EditorialPrompt {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 1000usize {
            return Err("longer than 1000 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RankingSetV2EditorialPrompt {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankingSetV2EditorialPrompt {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankingSetV2EditorialPrompt {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankingSetV2EditorialPrompt {
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
    pub struct Comparison {
        a: ::std::result::Result<super::PlanId, ::std::string::String>,
        b: ::std::result::Result<super::PlanId, ::std::string::String>,
        reasons:
            ::std::result::Result<[super::ComparisonReasonsItem; 2usize], ::std::string::String>,
        share_a: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Comparison {
        fn default() -> Self {
            Self {
                a: Err("no value supplied for a".to_string()),
                b: Err("no value supplied for b".to_string()),
                reasons: Err("no value supplied for reasons".to_string()),
                share_a: Err("no value supplied for share_a".to_string()),
            }
        }
    }
    impl Comparison {
        pub fn a<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PlanId>,
            T::Error: ::std::fmt::Display,
        {
            self.a = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for a: {e}"));
            self
        }
        pub fn b<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PlanId>,
            T::Error: ::std::fmt::Display,
        {
            self.b = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for b: {e}"));
            self
        }
        pub fn reasons<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<[super::ComparisonReasonsItem; 2usize]>,
            T::Error: ::std::fmt::Display,
        {
            self.reasons = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reasons: {e}"));
            self
        }
        pub fn share_a<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.share_a = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for share_a: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Comparison> for super::Comparison {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Comparison,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                a: value.a?,
                b: value.b?,
                reasons: value.reasons?,
                share_a: value.share_a?,
            })
        }
    }
    impl ::std::convert::From<super::Comparison> for Comparison {
        fn from(value: super::Comparison) -> Self {
            Self {
                a: Ok(value.a),
                b: Ok(value.b),
                reasons: Ok(value.reasons),
                share_a: Ok(value.share_a),
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
    pub struct Quality {
        hook: ::std::result::Result<super::QualityHook, ::std::string::String>,
        issues:
            ::std::result::Result<::std::vec::Vec<super::QualityIssuesItem>, ::std::string::String>,
        joins: ::std::result::Result<::std::vec::Vec<super::JoinReview>, ::std::string::String>,
        repaired: ::std::result::Result<bool, ::std::string::String>,
        summary: ::std::result::Result<super::QualitySummary, ::std::string::String>,
        verdict: ::std::result::Result<super::QualityVerdict, ::std::string::String>,
    }
    impl ::std::default::Default for Quality {
        fn default() -> Self {
            Self {
                hook: Err("no value supplied for hook".to_string()),
                issues: Err("no value supplied for issues".to_string()),
                joins: Err("no value supplied for joins".to_string()),
                repaired: Err("no value supplied for repaired".to_string()),
                summary: Err("no value supplied for summary".to_string()),
                verdict: Err("no value supplied for verdict".to_string()),
            }
        }
    }
    impl Quality {
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
                hook: value.hook?,
                issues: value.issues?,
                joins: value.joins?,
                repaired: value.repaired?,
                summary: value.summary?,
                verdict: value.verdict?,
            })
        }
    }
    impl ::std::convert::From<super::Quality> for Quality {
        fn from(value: super::Quality) -> Self {
            Self {
                hook: Ok(value.hook),
                issues: Ok(value.issues),
                joins: Ok(value.joins),
                repaired: Ok(value.repaired),
                summary: Ok(value.summary),
                verdict: Ok(value.verdict),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Ranked {
        chapter: ::std::result::Result<super::RankedChapter, ::std::string::String>,
        keep_score: ::std::result::Result<i64, ::std::string::String>,
        plan: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        quality: ::std::result::Result<super::Quality, ::std::string::String>,
        reasons:
            ::std::result::Result<::std::vec::Vec<super::RankedReasonsItem>, ::std::string::String>,
        strength: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Ranked {
        fn default() -> Self {
            Self {
                chapter: Err("no value supplied for chapter".to_string()),
                keep_score: Err("no value supplied for keep_score".to_string()),
                plan: Err("no value supplied for plan".to_string()),
                quality: Err("no value supplied for quality".to_string()),
                reasons: Err("no value supplied for reasons".to_string()),
                strength: Err("no value supplied for strength".to_string()),
            }
        }
    }
    impl Ranked {
        pub fn chapter<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankedChapter>,
            T::Error: ::std::fmt::Display,
        {
            self.chapter = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for chapter: {e}"));
            self
        }
        pub fn keep_score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.keep_score = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for keep_score: {e}"));
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
            T: ::std::convert::TryInto<::std::vec::Vec<super::RankedReasonsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.reasons = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reasons: {e}"));
            self
        }
        pub fn strength<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.strength = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for strength: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Ranked> for super::Ranked {
        type Error = super::error::ConversionError;
        fn try_from(value: Ranked) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                chapter: value.chapter?,
                keep_score: value.keep_score?,
                plan: value.plan?,
                quality: value.quality?,
                reasons: value.reasons?,
                strength: value.strength?,
            })
        }
    }
    impl ::std::convert::From<super::Ranked> for Ranked {
        fn from(value: super::Ranked) -> Self {
            Self {
                chapter: Ok(value.chapter),
                keep_score: Ok(value.keep_score),
                plan: Ok(value.plan),
                quality: Ok(value.quality),
                reasons: Ok(value.reasons),
                strength: Ok(value.strength),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RankingSetV2 {
        clips: ::std::result::Result<::std::vec::Vec<super::Ranked>, ::std::string::String>,
        comparisons:
            ::std::result::Result<::std::vec::Vec<super::Comparison>, ::std::string::String>,
        cutoff_reason: ::std::result::Result<
            ::std::option::Option<super::RankingSetV2CutoffReason>,
            ::std::string::String,
        >,
        editorial_prompt:
            ::std::result::Result<super::RankingSetV2EditorialPrompt, ::std::string::String>,
        inputs: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, super::Sha256>,
            ::std::string::String,
        >,
        requested_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for RankingSetV2 {
        fn default() -> Self {
            Self {
                clips: Err("no value supplied for clips".to_string()),
                comparisons: Err("no value supplied for comparisons".to_string()),
                cutoff_reason: Ok(Default::default()),
                editorial_prompt: Err("no value supplied for editorial_prompt".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                requested_count: Err("no value supplied for requested_count".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl RankingSetV2 {
        pub fn clips<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Ranked>>,
            T::Error: ::std::fmt::Display,
        {
            self.clips = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for clips: {e}"));
            self
        }
        pub fn comparisons<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Comparison>>,
            T::Error: ::std::fmt::Display,
        {
            self.comparisons = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for comparisons: {e}"));
            self
        }
        pub fn cutoff_reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::RankingSetV2CutoffReason>>,
            T::Error: ::std::fmt::Display,
        {
            self.cutoff_reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cutoff_reason: {e}"));
            self
        }
        pub fn editorial_prompt<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankingSetV2EditorialPrompt>,
            T::Error: ::std::fmt::Display,
        {
            self.editorial_prompt = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for editorial_prompt: {e}"));
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
        pub fn requested_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.requested_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for requested_count: {e}"));
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
    impl ::std::convert::TryFrom<RankingSetV2> for super::RankingSetV2 {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RankingSetV2,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                clips: value.clips?,
                comparisons: value.comparisons?,
                cutoff_reason: value.cutoff_reason?,
                editorial_prompt: value.editorial_prompt?,
                inputs: value.inputs?,
                requested_count: value.requested_count?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::RankingSetV2> for RankingSetV2 {
        fn from(value: super::RankingSetV2) -> Self {
            Self {
                clips: Ok(value.clips),
                comparisons: Ok(value.comparisons),
                cutoff_reason: Ok(value.cutoff_reason),
                editorial_prompt: Ok(value.editorial_prompt),
                inputs: Ok(value.inputs),
                requested_count: Ok(value.requested_count),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
}
