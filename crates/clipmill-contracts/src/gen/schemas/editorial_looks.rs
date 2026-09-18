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
#[doc = "`Check`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidate_id\","]
#[doc = "    \"frames\","]
#[doc = "    \"outcome\","]
#[doc = "    \"question\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"answer\": {"]
#[doc = "      \"description\": \"'yes' the thing referred to is visible; 'no' it is not; 'unclear' the frames do not settle it.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"yes\","]
#[doc = "        \"no\","]
#[doc = "        \"unclear\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"candidate_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "    },"]
#[doc = "    \"detail\": {"]
#[doc = "      \"description\": \"The model's one-sentence account of what it saw.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"failure\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"detail\","]
#[doc = "        \"failure_class\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"detail\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"failure_class\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"deterministic\","]
#[doc = "            \"transient\","]
#[doc = "            \"cancelled\","]
#[doc = "            \"budget\""]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"frames\": {"]
#[doc = "      \"description\": \"The frames shown, by their position in the recording, so the answer can be walked back to a picture.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"t_ticks\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"t_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"outcome\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"answered\","]
#[doc = "        \"malformed\","]
#[doc = "        \"failed\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"question\": {"]
#[doc = "      \"description\": \"What the model was asked about these frames, verbatim, so an answer is never read against a question nobody can see.\","]
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
pub struct Check {
    #[doc = "'yes' the thing referred to is visible; 'no' it is not; 'unclear' the frames do not settle it."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub answer: ::std::option::Option<CheckAnswer>,
    pub candidate_id: CheckCandidateId,
    #[doc = "The model's one-sentence account of what it saw."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub detail: ::std::option::Option<CheckDetail>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub failure: ::std::option::Option<CheckFailure>,
    #[doc = "The frames shown, by their position in the recording, so the answer can be walked back to a picture."]
    pub frames: ::std::vec::Vec<CheckFramesItem>,
    pub outcome: CheckOutcome,
    #[doc = "What the model was asked about these frames, verbatim, so an answer is never read against a question nobody can see."]
    pub question: CheckQuestion,
}
impl Check {
    pub fn builder() -> builder::Check {
        Default::default()
    }
}
#[doc = "'yes' the thing referred to is visible; 'no' it is not; 'unclear' the frames do not settle it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"'yes' the thing referred to is visible; 'no' it is not; 'unclear' the frames do not settle it.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"yes\","]
#[doc = "    \"no\","]
#[doc = "    \"unclear\""]
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
pub enum CheckAnswer {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "unclear")]
    Unclear,
}
impl ::std::fmt::Display for CheckAnswer {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Yes => f.write_str("yes"),
            Self::No => f.write_str("no"),
            Self::Unclear => f.write_str("unclear"),
        }
    }
}
impl ::std::str::FromStr for CheckAnswer {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "yes" => Ok(Self::Yes),
            "no" => Ok(Self::No),
            "unclear" => Ok(Self::Unclear),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CheckAnswer {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CheckAnswer {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CheckAnswer {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CheckCandidateId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CheckCandidateId(::std::string::String);
impl ::std::ops::Deref for CheckCandidateId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CheckCandidateId> for ::std::string::String {
    fn from(value: CheckCandidateId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CheckCandidateId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^cand_[0-9a-f]{16}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^cand_[0-9a-f]{16}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CheckCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CheckCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CheckCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CheckCandidateId {
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
#[doc = "The model's one-sentence account of what it saw."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The model's one-sentence account of what it saw.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CheckDetail(::std::string::String);
impl ::std::ops::Deref for CheckDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CheckDetail> for ::std::string::String {
    fn from(value: CheckDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CheckDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CheckDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CheckDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CheckDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CheckDetail {
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
#[doc = "`CheckFailure`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"detail\","]
#[doc = "    \"failure_class\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"failure_class\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"deterministic\","]
#[doc = "        \"transient\","]
#[doc = "        \"cancelled\","]
#[doc = "        \"budget\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckFailure {
    pub detail: CheckFailureDetail,
    pub failure_class: CheckFailureFailureClass,
}
impl CheckFailure {
    pub fn builder() -> builder::CheckFailure {
        Default::default()
    }
}
#[doc = "`CheckFailureDetail`"]
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
pub struct CheckFailureDetail(::std::string::String);
impl ::std::ops::Deref for CheckFailureDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CheckFailureDetail> for ::std::string::String {
    fn from(value: CheckFailureDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CheckFailureDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CheckFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CheckFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CheckFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CheckFailureDetail {
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
#[doc = "`CheckFailureFailureClass`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"deterministic\","]
#[doc = "    \"transient\","]
#[doc = "    \"cancelled\","]
#[doc = "    \"budget\""]
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
pub enum CheckFailureFailureClass {
    #[serde(rename = "deterministic")]
    Deterministic,
    #[serde(rename = "transient")]
    Transient,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "budget")]
    Budget,
}
impl ::std::fmt::Display for CheckFailureFailureClass {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Deterministic => f.write_str("deterministic"),
            Self::Transient => f.write_str("transient"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Budget => f.write_str("budget"),
        }
    }
}
impl ::std::str::FromStr for CheckFailureFailureClass {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "deterministic" => Ok(Self::Deterministic),
            "transient" => Ok(Self::Transient),
            "cancelled" => Ok(Self::Cancelled),
            "budget" => Ok(Self::Budget),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CheckFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CheckFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CheckFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CheckFramesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"t_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
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
pub struct CheckFramesItem {
    pub t_ticks: u64,
}
impl CheckFramesItem {
    pub fn builder() -> builder::CheckFramesItem {
        Default::default()
    }
}
#[doc = "`CheckOutcome`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"answered\","]
#[doc = "    \"malformed\","]
#[doc = "    \"failed\""]
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
pub enum CheckOutcome {
    #[serde(rename = "answered")]
    Answered,
    #[serde(rename = "malformed")]
    Malformed,
    #[serde(rename = "failed")]
    Failed,
}
impl ::std::fmt::Display for CheckOutcome {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Answered => f.write_str("answered"),
            Self::Malformed => f.write_str("malformed"),
            Self::Failed => f.write_str("failed"),
        }
    }
}
impl ::std::str::FromStr for CheckOutcome {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "answered" => Ok(Self::Answered),
            "malformed" => Ok(Self::Malformed),
            "failed" => Ok(Self::Failed),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CheckOutcome {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CheckOutcome {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CheckOutcome {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "What the model was asked about these frames, verbatim, so an answer is never read against a question nobody can see."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What the model was asked about these frames, verbatim, so an answer is never read against a question nobody can see.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CheckQuestion(::std::string::String);
impl ::std::ops::Deref for CheckQuestion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CheckQuestion> for ::std::string::String {
    fn from(value: CheckQuestion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CheckQuestion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CheckQuestion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CheckQuestion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CheckQuestion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CheckQuestion {
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
#[doc = "`Decoding`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"constrained\","]
#[doc = "    \"max_output_tokens\","]
#[doc = "    \"seed\","]
#[doc = "    \"temperature_milli\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"constrained\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"max_output_tokens\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"seed\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"temperature_milli\": {"]
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
pub struct Decoding {
    pub constrained: bool,
    pub max_output_tokens: ::std::num::NonZeroU64,
    pub seed: u64,
    pub temperature_milli: u64,
}
impl Decoding {
    pub fn builder() -> builder::Decoding {
        Default::default()
    }
}
#[doc = "What a visual model saw when the review said the picture mattered (plan, Milestone 2). One check per flagged candidate over a bounded set of frames from its span, one question, a short answer — is what is being talked about visible? — and the frames it was answered from, so the answer can be walked back to pictures. Not a frame-by-frame viewer: a candidate nobody flagged has no entry here, and a candidate the model could not look at says so rather than answering."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.editorial.looks.v1.json\","]
#[doc = "  \"title\": \"EditorialLooks\","]
#[doc = "  \"description\": \"What a visual model saw when the review said the picture mattered (plan, Milestone 2). One check per flagged candidate over a bounded set of frames from its span, one question, a short answer — is what is being talked about visible? — and the frames it was answered from, so the answer can be walked back to pictures. Not a frame-by-frame viewer: a candidate nobody flagged has no entry here, and a candidate the model could not look at says so rather than answering.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"checks\","]
#[doc = "    \"inputs\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"checks\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/check\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"content_profile\": {"]
#[doc = "      \"description\": \"The editorial rubric selected for this run; older artifacts use interview.\","]
#[doc = "      \"default\": \"interview\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"interview\","]
#[doc = "        \"scripted\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"frames_artifact_id\","]
#[doc = "        \"judgments_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"frames_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"judgments_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/model_producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.editorial.looks.v1\""]
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
pub struct EditorialLooks {
    pub checks: ::std::vec::Vec<Check>,
    #[doc = "The editorial rubric selected for this run; older artifacts use interview."]
    #[serde(default = "defaults::editorial_looks_content_profile")]
    pub content_profile: EditorialLooksContentProfile,
    pub inputs: EditorialLooksInputs,
    pub producer: ModelProducer,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
}
impl EditorialLooks {
    pub fn builder() -> builder::EditorialLooks {
        Default::default()
    }
}
#[doc = "The editorial rubric selected for this run; older artifacts use interview."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The editorial rubric selected for this run; older artifacts use interview.\","]
#[doc = "  \"default\": \"interview\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"interview\","]
#[doc = "    \"scripted\""]
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
pub enum EditorialLooksContentProfile {
    #[serde(rename = "interview")]
    Interview,
    #[serde(rename = "scripted")]
    Scripted,
}
impl ::std::fmt::Display for EditorialLooksContentProfile {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Interview => f.write_str("interview"),
            Self::Scripted => f.write_str("scripted"),
        }
    }
}
impl ::std::str::FromStr for EditorialLooksContentProfile {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "interview" => Ok(Self::Interview),
            "scripted" => Ok(Self::Scripted),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EditorialLooksContentProfile {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditorialLooksContentProfile {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditorialLooksContentProfile {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::default::Default for EditorialLooksContentProfile {
    fn default() -> Self {
        EditorialLooksContentProfile::Interview
    }
}
#[doc = "`EditorialLooksInputs`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"frames_artifact_id\","]
#[doc = "    \"judgments_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"frames_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"judgments_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditorialLooksInputs {
    pub frames_artifact_id: Sha256,
    pub judgments_artifact_id: Sha256,
}
impl EditorialLooksInputs {
    pub fn builder() -> builder::EditorialLooksInputs {
        Default::default()
    }
}
#[doc = "`Model`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"name\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"digest\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"provider\": {"]
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
pub struct Model {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub digest: ::std::option::Option<Sha256>,
    pub name: ModelName,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub provider: ::std::option::Option<ModelProvider>,
}
impl Model {
    pub fn builder() -> builder::Model {
        Default::default()
    }
}
#[doc = "`ModelName`"]
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
pub struct ModelName(::std::string::String);
impl ::std::ops::Deref for ModelName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelName> for ::std::string::String {
    fn from(value: ModelName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelName {
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
#[doc = "`ModelProducer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"decoding\","]
#[doc = "    \"implementation\","]
#[doc = "    \"model\","]
#[doc = "    \"prompt_version\","]
#[doc = "    \"route\","]
#[doc = "    \"stage\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"decoding\": {"]
#[doc = "      \"$ref\": \"#/$defs/decoding\""]
#[doc = "    },"]
#[doc = "    \"implementation\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"model\": {"]
#[doc = "      \"$ref\": \"#/$defs/model\""]
#[doc = "    },"]
#[doc = "    \"prompt_version\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"route\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"local\","]
#[doc = "        \"cloud\""]
#[doc = "      ]"]
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
pub struct ModelProducer {
    pub decoding: Decoding,
    pub implementation: ModelProducerImplementation,
    pub model: Model,
    pub prompt_version: ModelProducerPromptVersion,
    pub route: ModelProducerRoute,
    pub stage: ModelProducerStage,
}
impl ModelProducer {
    pub fn builder() -> builder::ModelProducer {
        Default::default()
    }
}
#[doc = "`ModelProducerImplementation`"]
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
pub struct ModelProducerImplementation(::std::string::String);
impl ::std::ops::Deref for ModelProducerImplementation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelProducerImplementation> for ::std::string::String {
    fn from(value: ModelProducerImplementation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelProducerImplementation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelProducerImplementation {
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
#[doc = "`ModelProducerPromptVersion`"]
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
pub struct ModelProducerPromptVersion(::std::string::String);
impl ::std::ops::Deref for ModelProducerPromptVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelProducerPromptVersion> for ::std::string::String {
    fn from(value: ModelProducerPromptVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelProducerPromptVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelProducerPromptVersion {
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
#[doc = "`ModelProducerRoute`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"local\","]
#[doc = "    \"cloud\""]
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
pub enum ModelProducerRoute {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "cloud")]
    Cloud,
}
impl ::std::fmt::Display for ModelProducerRoute {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Local => f.write_str("local"),
            Self::Cloud => f.write_str("cloud"),
        }
    }
}
impl ::std::str::FromStr for ModelProducerRoute {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "local" => Ok(Self::Local),
            "cloud" => Ok(Self::Cloud),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ModelProducerRoute {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProducerRoute {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProducerRoute {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`ModelProducerStage`"]
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
pub struct ModelProducerStage(::std::string::String);
impl ::std::ops::Deref for ModelProducerStage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelProducerStage> for ::std::string::String {
    fn from(value: ModelProducerStage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelProducerStage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelProducerStage {
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
#[doc = "`ModelProvider`"]
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
pub struct ModelProvider(::std::string::String);
impl ::std::ops::Deref for ModelProvider {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelProvider> for ::std::string::String {
    fn from(value: ModelProvider) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelProvider {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelProvider {
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
    pub struct Check {
        answer:
            ::std::result::Result<::std::option::Option<super::CheckAnswer>, ::std::string::String>,
        candidate_id: ::std::result::Result<super::CheckCandidateId, ::std::string::String>,
        detail:
            ::std::result::Result<::std::option::Option<super::CheckDetail>, ::std::string::String>,
        failure: ::std::result::Result<
            ::std::option::Option<super::CheckFailure>,
            ::std::string::String,
        >,
        frames:
            ::std::result::Result<::std::vec::Vec<super::CheckFramesItem>, ::std::string::String>,
        outcome: ::std::result::Result<super::CheckOutcome, ::std::string::String>,
        question: ::std::result::Result<super::CheckQuestion, ::std::string::String>,
    }
    impl ::std::default::Default for Check {
        fn default() -> Self {
            Self {
                answer: Ok(Default::default()),
                candidate_id: Err("no value supplied for candidate_id".to_string()),
                detail: Ok(Default::default()),
                failure: Ok(Default::default()),
                frames: Err("no value supplied for frames".to_string()),
                outcome: Err("no value supplied for outcome".to_string()),
                question: Err("no value supplied for question".to_string()),
            }
        }
    }
    impl Check {
        pub fn answer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CheckAnswer>>,
            T::Error: ::std::fmt::Display,
        {
            self.answer = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for answer: {e}"));
            self
        }
        pub fn candidate_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CheckCandidateId>,
            T::Error: ::std::fmt::Display,
        {
            self.candidate_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidate_id: {e}"));
            self
        }
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CheckDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn failure<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CheckFailure>>,
            T::Error: ::std::fmt::Display,
        {
            self.failure = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for failure: {e}"));
            self
        }
        pub fn frames<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CheckFramesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.frames = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frames: {e}"));
            self
        }
        pub fn outcome<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CheckOutcome>,
            T::Error: ::std::fmt::Display,
        {
            self.outcome = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outcome: {e}"));
            self
        }
        pub fn question<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CheckQuestion>,
            T::Error: ::std::fmt::Display,
        {
            self.question = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for question: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Check> for super::Check {
        type Error = super::error::ConversionError;
        fn try_from(value: Check) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                answer: value.answer?,
                candidate_id: value.candidate_id?,
                detail: value.detail?,
                failure: value.failure?,
                frames: value.frames?,
                outcome: value.outcome?,
                question: value.question?,
            })
        }
    }
    impl ::std::convert::From<super::Check> for Check {
        fn from(value: super::Check) -> Self {
            Self {
                answer: Ok(value.answer),
                candidate_id: Ok(value.candidate_id),
                detail: Ok(value.detail),
                failure: Ok(value.failure),
                frames: Ok(value.frames),
                outcome: Ok(value.outcome),
                question: Ok(value.question),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CheckFailure {
        detail: ::std::result::Result<super::CheckFailureDetail, ::std::string::String>,
        failure_class:
            ::std::result::Result<super::CheckFailureFailureClass, ::std::string::String>,
    }
    impl ::std::default::Default for CheckFailure {
        fn default() -> Self {
            Self {
                detail: Err("no value supplied for detail".to_string()),
                failure_class: Err("no value supplied for failure_class".to_string()),
            }
        }
    }
    impl CheckFailure {
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CheckFailureDetail>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn failure_class<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CheckFailureFailureClass>,
            T::Error: ::std::fmt::Display,
        {
            self.failure_class = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for failure_class: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CheckFailure> for super::CheckFailure {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CheckFailure,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                detail: value.detail?,
                failure_class: value.failure_class?,
            })
        }
    }
    impl ::std::convert::From<super::CheckFailure> for CheckFailure {
        fn from(value: super::CheckFailure) -> Self {
            Self {
                detail: Ok(value.detail),
                failure_class: Ok(value.failure_class),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CheckFramesItem {
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for CheckFramesItem {
        fn default() -> Self {
            Self {
                t_ticks: Err("no value supplied for t_ticks".to_string()),
            }
        }
    }
    impl CheckFramesItem {
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
    impl ::std::convert::TryFrom<CheckFramesItem> for super::CheckFramesItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CheckFramesItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                t_ticks: value.t_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::CheckFramesItem> for CheckFramesItem {
        fn from(value: super::CheckFramesItem) -> Self {
            Self {
                t_ticks: Ok(value.t_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Decoding {
        constrained: ::std::result::Result<bool, ::std::string::String>,
        max_output_tokens: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        seed: ::std::result::Result<u64, ::std::string::String>,
        temperature_milli: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Decoding {
        fn default() -> Self {
            Self {
                constrained: Err("no value supplied for constrained".to_string()),
                max_output_tokens: Err("no value supplied for max_output_tokens".to_string()),
                seed: Err("no value supplied for seed".to_string()),
                temperature_milli: Err("no value supplied for temperature_milli".to_string()),
            }
        }
    }
    impl Decoding {
        pub fn constrained<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.constrained = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for constrained: {e}"));
            self
        }
        pub fn max_output_tokens<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_output_tokens = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max_output_tokens: {e}"));
            self
        }
        pub fn seed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.seed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for seed: {e}"));
            self
        }
        pub fn temperature_milli<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.temperature_milli = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for temperature_milli: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Decoding> for super::Decoding {
        type Error = super::error::ConversionError;
        fn try_from(value: Decoding) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                constrained: value.constrained?,
                max_output_tokens: value.max_output_tokens?,
                seed: value.seed?,
                temperature_milli: value.temperature_milli?,
            })
        }
    }
    impl ::std::convert::From<super::Decoding> for Decoding {
        fn from(value: super::Decoding) -> Self {
            Self {
                constrained: Ok(value.constrained),
                max_output_tokens: Ok(value.max_output_tokens),
                seed: Ok(value.seed),
                temperature_milli: Ok(value.temperature_milli),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialLooks {
        checks: ::std::result::Result<::std::vec::Vec<super::Check>, ::std::string::String>,
        content_profile:
            ::std::result::Result<super::EditorialLooksContentProfile, ::std::string::String>,
        inputs: ::std::result::Result<super::EditorialLooksInputs, ::std::string::String>,
        producer: ::std::result::Result<super::ModelProducer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialLooks {
        fn default() -> Self {
            Self {
                checks: Err("no value supplied for checks".to_string()),
                content_profile: Ok(super::defaults::editorial_looks_content_profile()),
                inputs: Err("no value supplied for inputs".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl EditorialLooks {
        pub fn checks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Check>>,
            T::Error: ::std::fmt::Display,
        {
            self.checks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for checks: {e}"));
            self
        }
        pub fn content_profile<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialLooksContentProfile>,
            T::Error: ::std::fmt::Display,
        {
            self.content_profile = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for content_profile: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialLooksInputs>,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
            self
        }
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducer>,
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
    impl ::std::convert::TryFrom<EditorialLooks> for super::EditorialLooks {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialLooks,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                checks: value.checks?,
                content_profile: value.content_profile?,
                inputs: value.inputs?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialLooks> for EditorialLooks {
        fn from(value: super::EditorialLooks) -> Self {
            Self {
                checks: Ok(value.checks),
                content_profile: Ok(value.content_profile),
                inputs: Ok(value.inputs),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialLooksInputs {
        frames_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        judgments_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialLooksInputs {
        fn default() -> Self {
            Self {
                frames_artifact_id: Err("no value supplied for frames_artifact_id".to_string()),
                judgments_artifact_id: Err(
                    "no value supplied for judgments_artifact_id".to_string()
                ),
            }
        }
    }
    impl EditorialLooksInputs {
        pub fn frames_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.frames_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for frames_artifact_id: {e}")
            });
            self
        }
        pub fn judgments_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.judgments_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for judgments_artifact_id: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<EditorialLooksInputs> for super::EditorialLooksInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialLooksInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                frames_artifact_id: value.frames_artifact_id?,
                judgments_artifact_id: value.judgments_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialLooksInputs> for EditorialLooksInputs {
        fn from(value: super::EditorialLooksInputs) -> Self {
            Self {
                frames_artifact_id: Ok(value.frames_artifact_id),
                judgments_artifact_id: Ok(value.judgments_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Model {
        digest: ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        name: ::std::result::Result<super::ModelName, ::std::string::String>,
        provider: ::std::result::Result<
            ::std::option::Option<super::ModelProvider>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Model {
        fn default() -> Self {
            Self {
                digest: Ok(Default::default()),
                name: Err("no value supplied for name".to_string()),
                provider: Ok(Default::default()),
            }
        }
    }
    impl Model {
        pub fn digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for digest: {e}"));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn provider<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ModelProvider>>,
            T::Error: ::std::fmt::Display,
        {
            self.provider = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for provider: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Model> for super::Model {
        type Error = super::error::ConversionError;
        fn try_from(value: Model) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                digest: value.digest?,
                name: value.name?,
                provider: value.provider?,
            })
        }
    }
    impl ::std::convert::From<super::Model> for Model {
        fn from(value: super::Model) -> Self {
            Self {
                digest: Ok(value.digest),
                name: Ok(value.name),
                provider: Ok(value.provider),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ModelProducer {
        decoding: ::std::result::Result<super::Decoding, ::std::string::String>,
        implementation:
            ::std::result::Result<super::ModelProducerImplementation, ::std::string::String>,
        model: ::std::result::Result<super::Model, ::std::string::String>,
        prompt_version:
            ::std::result::Result<super::ModelProducerPromptVersion, ::std::string::String>,
        route: ::std::result::Result<super::ModelProducerRoute, ::std::string::String>,
        stage: ::std::result::Result<super::ModelProducerStage, ::std::string::String>,
    }
    impl ::std::default::Default for ModelProducer {
        fn default() -> Self {
            Self {
                decoding: Err("no value supplied for decoding".to_string()),
                implementation: Err("no value supplied for implementation".to_string()),
                model: Err("no value supplied for model".to_string()),
                prompt_version: Err("no value supplied for prompt_version".to_string()),
                route: Err("no value supplied for route".to_string()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl ModelProducer {
        pub fn decoding<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Decoding>,
            T::Error: ::std::fmt::Display,
        {
            self.decoding = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for decoding: {e}"));
            self
        }
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducerImplementation>,
            T::Error: ::std::fmt::Display,
        {
            self.implementation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for implementation: {e}"));
            self
        }
        pub fn model<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Model>,
            T::Error: ::std::fmt::Display,
        {
            self.model = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model: {e}"));
            self
        }
        pub fn prompt_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducerPromptVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt_version: {e}"));
            self
        }
        pub fn route<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducerRoute>,
            T::Error: ::std::fmt::Display,
        {
            self.route = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for route: {e}"));
            self
        }
        pub fn stage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducerStage>,
            T::Error: ::std::fmt::Display,
        {
            self.stage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stage: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ModelProducer> for super::ModelProducer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ModelProducer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                decoding: value.decoding?,
                implementation: value.implementation?,
                model: value.model?,
                prompt_version: value.prompt_version?,
                route: value.route?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::ModelProducer> for ModelProducer {
        fn from(value: super::ModelProducer) -> Self {
            Self {
                decoding: Ok(value.decoding),
                implementation: Ok(value.implementation),
                model: Ok(value.model),
                prompt_version: Ok(value.prompt_version),
                route: Ok(value.route),
                stage: Ok(value.stage),
            }
        }
    }
}
#[doc = r" Generation of default values for serde."]
pub mod defaults {
    pub(super) fn editorial_looks_content_profile() -> super::EditorialLooksContentProfile {
        super::EditorialLooksContentProfile::Interview
    }
}
