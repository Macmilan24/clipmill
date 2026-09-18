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
#[doc = "`Call`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"duration_millis\","]
#[doc = "    \"index\","]
#[doc = "    \"job\","]
#[doc = "    \"model\","]
#[doc = "    \"outcome\","]
#[doc = "    \"prompt_version\","]
#[doc = "    \"request_text\","]
#[doc = "    \"route\","]
#[doc = "    \"started_unix_millis\","]
#[doc = "    \"tokens\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidate_id\": {"]
#[doc = "      \"description\": \"The candidate a review or look call was about.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "    },"]
#[doc = "    \"cost_micro_usd\": {"]
#[doc = "      \"description\": \"What this call cost on the cloud route, from the provider's published prices at the time.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"duration_millis\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"frames\": {"]
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
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"job\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"propose\","]
#[doc = "        \"review\","]
#[doc = "        \"look\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"model\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"name\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"digest\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"name\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"provider\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"outcome\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"answered\","]
#[doc = "        \"none\","]
#[doc = "        \"malformed\","]
#[doc = "        \"failed\","]
#[doc = "        \"cancelled\","]
#[doc = "        \"budget\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"prompt_version\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"request_text\": {"]
#[doc = "      \"description\": \"The prompt as the model received it, text only; frames shown to a visual model are named by `frames` rather than embedded.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"response_text\": {"]
#[doc = "      \"description\": \"The reply as it came back, verbatim, even when it was malformed — especially then.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"route\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"local\","]
#[doc = "        \"cloud\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"started_unix_millis\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"tokens\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"input\","]
#[doc = "        \"output\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"input\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"output\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"window_index\": {"]
#[doc = "      \"description\": \"The window a propose call was over.\","]
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
pub struct Call {
    #[doc = "The candidate a review or look call was about."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub candidate_id: ::std::option::Option<CallCandidateId>,
    #[doc = "What this call cost on the cloud route, from the provider's published prices at the time."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub cost_micro_usd: ::std::option::Option<u64>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub detail: ::std::option::Option<CallDetail>,
    pub duration_millis: u64,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub frames: ::std::vec::Vec<CallFramesItem>,
    pub index: u64,
    pub job: CallJob,
    pub model: CallModel,
    pub outcome: CallOutcome,
    pub prompt_version: CallPromptVersion,
    #[doc = "The prompt as the model received it, text only; frames shown to a visual model are named by `frames` rather than embedded."]
    pub request_text: ::std::string::String,
    #[doc = "The reply as it came back, verbatim, even when it was malformed — especially then."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub response_text: ::std::option::Option<::std::string::String>,
    pub route: CallRoute,
    pub started_unix_millis: u64,
    pub tokens: CallTokens,
    #[doc = "The window a propose call was over."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub window_index: ::std::option::Option<u64>,
}
impl Call {
    pub fn builder() -> builder::Call {
        Default::default()
    }
}
#[doc = "The candidate a review or look call was about."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The candidate a review or look call was about.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CallCandidateId(::std::string::String);
impl ::std::ops::Deref for CallCandidateId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CallCandidateId> for ::std::string::String {
    fn from(value: CallCandidateId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CallCandidateId {
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
impl ::std::convert::TryFrom<&str> for CallCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CallCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CallCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CallCandidateId {
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
#[doc = "`CallDetail`"]
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
pub struct CallDetail(::std::string::String);
impl ::std::ops::Deref for CallDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CallDetail> for ::std::string::String {
    fn from(value: CallDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CallDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CallDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CallDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CallDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CallDetail {
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
#[doc = "`CallFramesItem`"]
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
pub struct CallFramesItem {
    pub t_ticks: u64,
}
impl CallFramesItem {
    pub fn builder() -> builder::CallFramesItem {
        Default::default()
    }
}
#[doc = "`CallJob`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"propose\","]
#[doc = "    \"review\","]
#[doc = "    \"look\""]
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
pub enum CallJob {
    #[serde(rename = "propose")]
    Propose,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "look")]
    Look,
}
impl ::std::fmt::Display for CallJob {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Propose => f.write_str("propose"),
            Self::Review => f.write_str("review"),
            Self::Look => f.write_str("look"),
        }
    }
}
impl ::std::str::FromStr for CallJob {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "propose" => Ok(Self::Propose),
            "review" => Ok(Self::Review),
            "look" => Ok(Self::Look),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CallJob {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CallJob {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CallJob {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CallModel`"]
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
pub struct CallModel {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub digest: ::std::option::Option<Sha256>,
    pub name: CallModelName,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub provider: ::std::option::Option<CallModelProvider>,
}
impl CallModel {
    pub fn builder() -> builder::CallModel {
        Default::default()
    }
}
#[doc = "`CallModelName`"]
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
pub struct CallModelName(::std::string::String);
impl ::std::ops::Deref for CallModelName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CallModelName> for ::std::string::String {
    fn from(value: CallModelName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CallModelName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CallModelName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CallModelName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CallModelName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CallModelName {
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
#[doc = "`CallModelProvider`"]
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
pub struct CallModelProvider(::std::string::String);
impl ::std::ops::Deref for CallModelProvider {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CallModelProvider> for ::std::string::String {
    fn from(value: CallModelProvider) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CallModelProvider {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CallModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CallModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CallModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CallModelProvider {
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
#[doc = "`CallOutcome`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"answered\","]
#[doc = "    \"none\","]
#[doc = "    \"malformed\","]
#[doc = "    \"failed\","]
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
pub enum CallOutcome {
    #[serde(rename = "answered")]
    Answered,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "malformed")]
    Malformed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "budget")]
    Budget,
}
impl ::std::fmt::Display for CallOutcome {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Answered => f.write_str("answered"),
            Self::None => f.write_str("none"),
            Self::Malformed => f.write_str("malformed"),
            Self::Failed => f.write_str("failed"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Budget => f.write_str("budget"),
        }
    }
}
impl ::std::str::FromStr for CallOutcome {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "answered" => Ok(Self::Answered),
            "none" => Ok(Self::None),
            "malformed" => Ok(Self::Malformed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "budget" => Ok(Self::Budget),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CallOutcome {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CallOutcome {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CallOutcome {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CallPromptVersion`"]
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
pub struct CallPromptVersion(::std::string::String);
impl ::std::ops::Deref for CallPromptVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CallPromptVersion> for ::std::string::String {
    fn from(value: CallPromptVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CallPromptVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CallPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CallPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CallPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CallPromptVersion {
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
#[doc = "`CallRoute`"]
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
pub enum CallRoute {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "cloud")]
    Cloud,
}
impl ::std::fmt::Display for CallRoute {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Local => f.write_str("local"),
            Self::Cloud => f.write_str("cloud"),
        }
    }
}
impl ::std::str::FromStr for CallRoute {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "local" => Ok(Self::Local),
            "cloud" => Ok(Self::Cloud),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CallRoute {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CallRoute {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CallRoute {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CallTokens`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"input\","]
#[doc = "    \"output\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"input\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"output\": {"]
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
pub struct CallTokens {
    pub input: u64,
    pub output: u64,
}
impl CallTokens {
    pub fn builder() -> builder::CallTokens {
        Default::default()
    }
}
#[doc = "Every call an editorial stage made to a model, with what was sent and what came back, kept locally for diagnosis (plan, Milestone 2). This is the document to open when a proposal is strange: the exact prompt, the exact reply, the model, the route, the tokens, and on the cloud route the spend against the run's budget. It is never on the shell's readable list and never leaves the machine in an export or an archive. Credentials are not in it by construction — a call carries the request text after the transport layer, and the transport layer is where a key lives."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.editorial.trace.v1.json\","]
#[doc = "  \"title\": \"EditorialTrace\","]
#[doc = "  \"description\": \"Every call an editorial stage made to a model, with what was sent and what came back, kept locally for diagnosis (plan, Milestone 2). This is the document to open when a proposal is strange: the exact prompt, the exact reply, the model, the route, the tokens, and on the cloud route the spend against the run's budget. It is never on the shell's readable list and never leaves the machine in an export or an archive. Credentials are not in it by construction — a call carries the request text after the transport layer, and the transport layer is where a key lives.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"calls\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"budget\": {"]
#[doc = "      \"description\": \"On the cloud route: the cap the run was given and what it spent, in millionths of a US dollar so the arithmetic is exact. Absent on the local route, which spends nothing.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"cap_micro_usd\","]
#[doc = "        \"spent_micro_usd\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"cap_micro_usd\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"spent_micro_usd\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"calls\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/call\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"implementation\","]
#[doc = "        \"stage\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"implementation\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"stage\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.editorial.trace.v1\""]
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
pub struct EditorialTrace {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub budget: ::std::option::Option<EditorialTraceBudget>,
    pub calls: ::std::vec::Vec<Call>,
    pub producer: EditorialTraceProducer,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
}
impl EditorialTrace {
    pub fn builder() -> builder::EditorialTrace {
        Default::default()
    }
}
#[doc = "On the cloud route: the cap the run was given and what it spent, in millionths of a US dollar so the arithmetic is exact. Absent on the local route, which spends nothing."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"On the cloud route: the cap the run was given and what it spent, in millionths of a US dollar so the arithmetic is exact. Absent on the local route, which spends nothing.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cap_micro_usd\","]
#[doc = "    \"spent_micro_usd\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cap_micro_usd\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"spent_micro_usd\": {"]
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
pub struct EditorialTraceBudget {
    pub cap_micro_usd: u64,
    pub spent_micro_usd: u64,
}
impl EditorialTraceBudget {
    pub fn builder() -> builder::EditorialTraceBudget {
        Default::default()
    }
}
#[doc = "`EditorialTraceProducer`"]
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
#[doc = "    \"implementation\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
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
pub struct EditorialTraceProducer {
    pub implementation: EditorialTraceProducerImplementation,
    pub stage: EditorialTraceProducerStage,
}
impl EditorialTraceProducer {
    pub fn builder() -> builder::EditorialTraceProducer {
        Default::default()
    }
}
#[doc = "`EditorialTraceProducerImplementation`"]
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
pub struct EditorialTraceProducerImplementation(::std::string::String);
impl ::std::ops::Deref for EditorialTraceProducerImplementation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EditorialTraceProducerImplementation> for ::std::string::String {
    fn from(value: EditorialTraceProducerImplementation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EditorialTraceProducerImplementation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EditorialTraceProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditorialTraceProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditorialTraceProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EditorialTraceProducerImplementation {
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
#[doc = "`EditorialTraceProducerStage`"]
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
pub struct EditorialTraceProducerStage(::std::string::String);
impl ::std::ops::Deref for EditorialTraceProducerStage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EditorialTraceProducerStage> for ::std::string::String {
    fn from(value: EditorialTraceProducerStage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EditorialTraceProducerStage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EditorialTraceProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditorialTraceProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditorialTraceProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EditorialTraceProducerStage {
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
    pub struct Call {
        candidate_id: ::std::result::Result<
            ::std::option::Option<super::CallCandidateId>,
            ::std::string::String,
        >,
        cost_micro_usd: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        detail:
            ::std::result::Result<::std::option::Option<super::CallDetail>, ::std::string::String>,
        duration_millis: ::std::result::Result<u64, ::std::string::String>,
        frames:
            ::std::result::Result<::std::vec::Vec<super::CallFramesItem>, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        job: ::std::result::Result<super::CallJob, ::std::string::String>,
        model: ::std::result::Result<super::CallModel, ::std::string::String>,
        outcome: ::std::result::Result<super::CallOutcome, ::std::string::String>,
        prompt_version: ::std::result::Result<super::CallPromptVersion, ::std::string::String>,
        request_text: ::std::result::Result<::std::string::String, ::std::string::String>,
        response_text: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        route: ::std::result::Result<super::CallRoute, ::std::string::String>,
        started_unix_millis: ::std::result::Result<u64, ::std::string::String>,
        tokens: ::std::result::Result<super::CallTokens, ::std::string::String>,
        window_index: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
    }
    impl ::std::default::Default for Call {
        fn default() -> Self {
            Self {
                candidate_id: Ok(Default::default()),
                cost_micro_usd: Ok(Default::default()),
                detail: Ok(Default::default()),
                duration_millis: Err("no value supplied for duration_millis".to_string()),
                frames: Ok(Default::default()),
                index: Err("no value supplied for index".to_string()),
                job: Err("no value supplied for job".to_string()),
                model: Err("no value supplied for model".to_string()),
                outcome: Err("no value supplied for outcome".to_string()),
                prompt_version: Err("no value supplied for prompt_version".to_string()),
                request_text: Err("no value supplied for request_text".to_string()),
                response_text: Ok(Default::default()),
                route: Err("no value supplied for route".to_string()),
                started_unix_millis: Err("no value supplied for started_unix_millis".to_string()),
                tokens: Err("no value supplied for tokens".to_string()),
                window_index: Ok(Default::default()),
            }
        }
    }
    impl Call {
        pub fn candidate_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CallCandidateId>>,
            T::Error: ::std::fmt::Display,
        {
            self.candidate_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidate_id: {e}"));
            self
        }
        pub fn cost_micro_usd<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.cost_micro_usd = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cost_micro_usd: {e}"));
            self
        }
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CallDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn duration_millis<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.duration_millis = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for duration_millis: {e}"));
            self
        }
        pub fn frames<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CallFramesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.frames = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frames: {e}"));
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
        pub fn job<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CallJob>,
            T::Error: ::std::fmt::Display,
        {
            self.job = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for job: {e}"));
            self
        }
        pub fn model<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CallModel>,
            T::Error: ::std::fmt::Display,
        {
            self.model = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model: {e}"));
            self
        }
        pub fn outcome<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CallOutcome>,
            T::Error: ::std::fmt::Display,
        {
            self.outcome = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outcome: {e}"));
            self
        }
        pub fn prompt_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CallPromptVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt_version: {e}"));
            self
        }
        pub fn request_text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.request_text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_text: {e}"));
            self
        }
        pub fn response_text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.response_text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for response_text: {e}"));
            self
        }
        pub fn route<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CallRoute>,
            T::Error: ::std::fmt::Display,
        {
            self.route = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for route: {e}"));
            self
        }
        pub fn started_unix_millis<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.started_unix_millis = value.try_into().map_err(|e| {
                format!("error converting supplied value for started_unix_millis: {e}")
            });
            self
        }
        pub fn tokens<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CallTokens>,
            T::Error: ::std::fmt::Display,
        {
            self.tokens = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tokens: {e}"));
            self
        }
        pub fn window_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.window_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for window_index: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Call> for super::Call {
        type Error = super::error::ConversionError;
        fn try_from(value: Call) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidate_id: value.candidate_id?,
                cost_micro_usd: value.cost_micro_usd?,
                detail: value.detail?,
                duration_millis: value.duration_millis?,
                frames: value.frames?,
                index: value.index?,
                job: value.job?,
                model: value.model?,
                outcome: value.outcome?,
                prompt_version: value.prompt_version?,
                request_text: value.request_text?,
                response_text: value.response_text?,
                route: value.route?,
                started_unix_millis: value.started_unix_millis?,
                tokens: value.tokens?,
                window_index: value.window_index?,
            })
        }
    }
    impl ::std::convert::From<super::Call> for Call {
        fn from(value: super::Call) -> Self {
            Self {
                candidate_id: Ok(value.candidate_id),
                cost_micro_usd: Ok(value.cost_micro_usd),
                detail: Ok(value.detail),
                duration_millis: Ok(value.duration_millis),
                frames: Ok(value.frames),
                index: Ok(value.index),
                job: Ok(value.job),
                model: Ok(value.model),
                outcome: Ok(value.outcome),
                prompt_version: Ok(value.prompt_version),
                request_text: Ok(value.request_text),
                response_text: Ok(value.response_text),
                route: Ok(value.route),
                started_unix_millis: Ok(value.started_unix_millis),
                tokens: Ok(value.tokens),
                window_index: Ok(value.window_index),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CallFramesItem {
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for CallFramesItem {
        fn default() -> Self {
            Self {
                t_ticks: Err("no value supplied for t_ticks".to_string()),
            }
        }
    }
    impl CallFramesItem {
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
    impl ::std::convert::TryFrom<CallFramesItem> for super::CallFramesItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CallFramesItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                t_ticks: value.t_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::CallFramesItem> for CallFramesItem {
        fn from(value: super::CallFramesItem) -> Self {
            Self {
                t_ticks: Ok(value.t_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CallModel {
        digest: ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        name: ::std::result::Result<super::CallModelName, ::std::string::String>,
        provider: ::std::result::Result<
            ::std::option::Option<super::CallModelProvider>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for CallModel {
        fn default() -> Self {
            Self {
                digest: Ok(Default::default()),
                name: Err("no value supplied for name".to_string()),
                provider: Ok(Default::default()),
            }
        }
    }
    impl CallModel {
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
            T: ::std::convert::TryInto<super::CallModelName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn provider<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CallModelProvider>>,
            T::Error: ::std::fmt::Display,
        {
            self.provider = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for provider: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CallModel> for super::CallModel {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CallModel,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                digest: value.digest?,
                name: value.name?,
                provider: value.provider?,
            })
        }
    }
    impl ::std::convert::From<super::CallModel> for CallModel {
        fn from(value: super::CallModel) -> Self {
            Self {
                digest: Ok(value.digest),
                name: Ok(value.name),
                provider: Ok(value.provider),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CallTokens {
        input: ::std::result::Result<u64, ::std::string::String>,
        output: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for CallTokens {
        fn default() -> Self {
            Self {
                input: Err("no value supplied for input".to_string()),
                output: Err("no value supplied for output".to_string()),
            }
        }
    }
    impl CallTokens {
        pub fn input<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.input = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for input: {e}"));
            self
        }
        pub fn output<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.output = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for output: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CallTokens> for super::CallTokens {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CallTokens,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                input: value.input?,
                output: value.output?,
            })
        }
    }
    impl ::std::convert::From<super::CallTokens> for CallTokens {
        fn from(value: super::CallTokens) -> Self {
            Self {
                input: Ok(value.input),
                output: Ok(value.output),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialTrace {
        budget: ::std::result::Result<
            ::std::option::Option<super::EditorialTraceBudget>,
            ::std::string::String,
        >,
        calls: ::std::result::Result<::std::vec::Vec<super::Call>, ::std::string::String>,
        producer: ::std::result::Result<super::EditorialTraceProducer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialTrace {
        fn default() -> Self {
            Self {
                budget: Ok(Default::default()),
                calls: Err("no value supplied for calls".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl EditorialTrace {
        pub fn budget<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EditorialTraceBudget>>,
            T::Error: ::std::fmt::Display,
        {
            self.budget = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for budget: {e}"));
            self
        }
        pub fn calls<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Call>>,
            T::Error: ::std::fmt::Display,
        {
            self.calls = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for calls: {e}"));
            self
        }
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialTraceProducer>,
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
    impl ::std::convert::TryFrom<EditorialTrace> for super::EditorialTrace {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialTrace,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                budget: value.budget?,
                calls: value.calls?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialTrace> for EditorialTrace {
        fn from(value: super::EditorialTrace) -> Self {
            Self {
                budget: Ok(value.budget),
                calls: Ok(value.calls),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialTraceBudget {
        cap_micro_usd: ::std::result::Result<u64, ::std::string::String>,
        spent_micro_usd: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialTraceBudget {
        fn default() -> Self {
            Self {
                cap_micro_usd: Err("no value supplied for cap_micro_usd".to_string()),
                spent_micro_usd: Err("no value supplied for spent_micro_usd".to_string()),
            }
        }
    }
    impl EditorialTraceBudget {
        pub fn cap_micro_usd<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.cap_micro_usd = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cap_micro_usd: {e}"));
            self
        }
        pub fn spent_micro_usd<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.spent_micro_usd = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for spent_micro_usd: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditorialTraceBudget> for super::EditorialTraceBudget {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialTraceBudget,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cap_micro_usd: value.cap_micro_usd?,
                spent_micro_usd: value.spent_micro_usd?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialTraceBudget> for EditorialTraceBudget {
        fn from(value: super::EditorialTraceBudget) -> Self {
            Self {
                cap_micro_usd: Ok(value.cap_micro_usd),
                spent_micro_usd: Ok(value.spent_micro_usd),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialTraceProducer {
        implementation: ::std::result::Result<
            super::EditorialTraceProducerImplementation,
            ::std::string::String,
        >,
        stage: ::std::result::Result<super::EditorialTraceProducerStage, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialTraceProducer {
        fn default() -> Self {
            Self {
                implementation: Err("no value supplied for implementation".to_string()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl EditorialTraceProducer {
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialTraceProducerImplementation>,
            T::Error: ::std::fmt::Display,
        {
            self.implementation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for implementation: {e}"));
            self
        }
        pub fn stage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialTraceProducerStage>,
            T::Error: ::std::fmt::Display,
        {
            self.stage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stage: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditorialTraceProducer> for super::EditorialTraceProducer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialTraceProducer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                implementation: value.implementation?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialTraceProducer> for EditorialTraceProducer {
        fn from(value: super::EditorialTraceProducer) -> Self {
            Self {
                implementation: Ok(value.implementation),
                stage: Ok(value.stage),
            }
        }
    }
}
