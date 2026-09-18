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
#[doc = "What the editorial model made of each candidate once it saw it with the context around it (plan, Milestone 2): whether the moment stands on its own, and if not, why — an unresolved reference, a question that was never asked, a payoff that is not there, an ad read, an irrelevant introduction, an omission that changes the meaning — and whether understanding it depends on what is on screen. The answer is a status a person can act on and reasons a person can read, in place of a number nobody calibrated. A malformed reply or a model failure is recorded as exactly that, distinct from a rejection: the ranking treats an unjudged candidate as unjudged, not as bad."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.editorial.judgments.v1.json\","]
#[doc = "  \"title\": \"EditorialJudgments\","]
#[doc = "  \"description\": \"What the editorial model made of each candidate once it saw it with the context around it (plan, Milestone 2): whether the moment stands on its own, and if not, why — an unresolved reference, a question that was never asked, a payoff that is not there, an ad read, an irrelevant introduction, an omission that changes the meaning — and whether understanding it depends on what is on screen. The answer is a status a person can act on and reasons a person can read, in place of a number nobody calibrated. A malformed reply or a model failure is recorded as exactly that, distinct from a rejection: the ranking treats an unjudged candidate as unjudged, not as bad.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidates\","]
#[doc = "    \"inputs\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidates\": {"]
#[doc = "      \"description\": \"One entry per candidate the review was asked about, in the candidates document's order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/judgment\""]
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
#[doc = "        \"candidates_artifact_id\","]
#[doc = "        \"windows_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"candidates_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"windows_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/model_producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.editorial.judgments.v1\""]
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
pub struct EditorialJudgments {
    #[doc = "One entry per candidate the review was asked about, in the candidates document's order."]
    pub candidates: ::std::vec::Vec<Judgment>,
    #[doc = "The editorial rubric selected for this run; older artifacts use interview."]
    #[serde(default = "defaults::editorial_judgments_content_profile")]
    pub content_profile: EditorialJudgmentsContentProfile,
    pub inputs: EditorialJudgmentsInputs,
    pub producer: ModelProducer,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
}
impl EditorialJudgments {
    pub fn builder() -> builder::EditorialJudgments {
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
pub enum EditorialJudgmentsContentProfile {
    #[serde(rename = "interview")]
    Interview,
    #[serde(rename = "scripted")]
    Scripted,
}
impl ::std::fmt::Display for EditorialJudgmentsContentProfile {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Interview => f.write_str("interview"),
            Self::Scripted => f.write_str("scripted"),
        }
    }
}
impl ::std::str::FromStr for EditorialJudgmentsContentProfile {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "interview" => Ok(Self::Interview),
            "scripted" => Ok(Self::Scripted),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EditorialJudgmentsContentProfile {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditorialJudgmentsContentProfile {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditorialJudgmentsContentProfile {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::default::Default for EditorialJudgmentsContentProfile {
    fn default() -> Self {
        EditorialJudgmentsContentProfile::Interview
    }
}
#[doc = "`EditorialJudgmentsInputs`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidates_artifact_id\","]
#[doc = "    \"windows_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidates_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"windows_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditorialJudgmentsInputs {
    pub candidates_artifact_id: Sha256,
    pub windows_artifact_id: Sha256,
}
impl EditorialJudgmentsInputs {
    pub fn builder() -> builder::EditorialJudgmentsInputs {
        Default::default()
    }
}
#[doc = "`Judgment`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidate_id\","]
#[doc = "    \"context\","]
#[doc = "    \"outcome\","]
#[doc = "    \"reasons\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidate_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "    },"]
#[doc = "    \"context\": {"]
#[doc = "      \"description\": \"The sentences the candidate was judged with: its own and the neighbours around it.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"first_sentence_index\","]
#[doc = "        \"sentence_count\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"first_sentence_index\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"sentence_count\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
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
#[doc = "    \"outcome\": {"]
#[doc = "      \"description\": \"Whether the model answered at all. Only 'answered' carries a status; the other two are run problems to show as such.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"answered\","]
#[doc = "        \"malformed\","]
#[doc = "        \"failed\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"reasons\": {"]
#[doc = "      \"description\": \"Why, each with a code a screen can key off and a sentence a person can read. Empty for an acceptance with nothing to say, and for a judgment that was never made.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/reason\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"description\": \"'accepted' stands on its own; 'needs_review' has a reason a person should look at before it ships; 'rejected' should not be offered.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"accepted\","]
#[doc = "        \"needs_review\","]
#[doc = "        \"rejected\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"summary\": {"]
#[doc = "      \"description\": \"The model's one-sentence account of the moment, for the board.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"visual_dependency\": {"]
#[doc = "      \"description\": \"Whether understanding the moment depends on what is on screen — the flag that sends it to the visual check.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Judgment {
    pub candidate_id: JudgmentCandidateId,
    pub context: JudgmentContext,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub failure: ::std::option::Option<JudgmentFailure>,
    #[doc = "Whether the model answered at all. Only 'answered' carries a status; the other two are run problems to show as such."]
    pub outcome: JudgmentOutcome,
    #[doc = "Why, each with a code a screen can key off and a sentence a person can read. Empty for an acceptance with nothing to say, and for a judgment that was never made."]
    pub reasons: ::std::vec::Vec<Reason>,
    #[doc = "'accepted' stands on its own; 'needs_review' has a reason a person should look at before it ships; 'rejected' should not be offered."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub status: ::std::option::Option<JudgmentStatus>,
    #[doc = "The model's one-sentence account of the moment, for the board."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub summary: ::std::option::Option<JudgmentSummary>,
    #[doc = "Whether understanding the moment depends on what is on screen — the flag that sends it to the visual check."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub visual_dependency: ::std::option::Option<bool>,
}
impl Judgment {
    pub fn builder() -> builder::Judgment {
        Default::default()
    }
}
#[doc = "`JudgmentCandidateId`"]
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
pub struct JudgmentCandidateId(::std::string::String);
impl ::std::ops::Deref for JudgmentCandidateId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<JudgmentCandidateId> for ::std::string::String {
    fn from(value: JudgmentCandidateId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for JudgmentCandidateId {
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
impl ::std::convert::TryFrom<&str> for JudgmentCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JudgmentCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JudgmentCandidateId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for JudgmentCandidateId {
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
#[doc = "The sentences the candidate was judged with: its own and the neighbours around it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The sentences the candidate was judged with: its own and the neighbours around it.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"first_sentence_index\","]
#[doc = "    \"sentence_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"first_sentence_index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"sentence_count\": {"]
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
pub struct JudgmentContext {
    pub first_sentence_index: u64,
    pub sentence_count: ::std::num::NonZeroU64,
}
impl JudgmentContext {
    pub fn builder() -> builder::JudgmentContext {
        Default::default()
    }
}
#[doc = "`JudgmentFailure`"]
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
pub struct JudgmentFailure {
    pub detail: JudgmentFailureDetail,
    pub failure_class: JudgmentFailureFailureClass,
}
impl JudgmentFailure {
    pub fn builder() -> builder::JudgmentFailure {
        Default::default()
    }
}
#[doc = "`JudgmentFailureDetail`"]
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
pub struct JudgmentFailureDetail(::std::string::String);
impl ::std::ops::Deref for JudgmentFailureDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<JudgmentFailureDetail> for ::std::string::String {
    fn from(value: JudgmentFailureDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for JudgmentFailureDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for JudgmentFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JudgmentFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JudgmentFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for JudgmentFailureDetail {
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
#[doc = "`JudgmentFailureFailureClass`"]
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
pub enum JudgmentFailureFailureClass {
    #[serde(rename = "deterministic")]
    Deterministic,
    #[serde(rename = "transient")]
    Transient,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "budget")]
    Budget,
}
impl ::std::fmt::Display for JudgmentFailureFailureClass {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Deterministic => f.write_str("deterministic"),
            Self::Transient => f.write_str("transient"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Budget => f.write_str("budget"),
        }
    }
}
impl ::std::str::FromStr for JudgmentFailureFailureClass {
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
impl ::std::convert::TryFrom<&str> for JudgmentFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JudgmentFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JudgmentFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Whether the model answered at all. Only 'answered' carries a status; the other two are run problems to show as such."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Whether the model answered at all. Only 'answered' carries a status; the other two are run problems to show as such.\","]
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
pub enum JudgmentOutcome {
    #[serde(rename = "answered")]
    Answered,
    #[serde(rename = "malformed")]
    Malformed,
    #[serde(rename = "failed")]
    Failed,
}
impl ::std::fmt::Display for JudgmentOutcome {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Answered => f.write_str("answered"),
            Self::Malformed => f.write_str("malformed"),
            Self::Failed => f.write_str("failed"),
        }
    }
}
impl ::std::str::FromStr for JudgmentOutcome {
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
impl ::std::convert::TryFrom<&str> for JudgmentOutcome {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JudgmentOutcome {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JudgmentOutcome {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "'accepted' stands on its own; 'needs_review' has a reason a person should look at before it ships; 'rejected' should not be offered."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"'accepted' stands on its own; 'needs_review' has a reason a person should look at before it ships; 'rejected' should not be offered.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"accepted\","]
#[doc = "    \"needs_review\","]
#[doc = "    \"rejected\""]
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
pub enum JudgmentStatus {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "needs_review")]
    NeedsReview,
    #[serde(rename = "rejected")]
    Rejected,
}
impl ::std::fmt::Display for JudgmentStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Accepted => f.write_str("accepted"),
            Self::NeedsReview => f.write_str("needs_review"),
            Self::Rejected => f.write_str("rejected"),
        }
    }
}
impl ::std::str::FromStr for JudgmentStatus {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "accepted" => Ok(Self::Accepted),
            "needs_review" => Ok(Self::NeedsReview),
            "rejected" => Ok(Self::Rejected),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for JudgmentStatus {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JudgmentStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JudgmentStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "The model's one-sentence account of the moment, for the board."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The model's one-sentence account of the moment, for the board.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct JudgmentSummary(::std::string::String);
impl ::std::ops::Deref for JudgmentSummary {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<JudgmentSummary> for ::std::string::String {
    fn from(value: JudgmentSummary) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for JudgmentSummary {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for JudgmentSummary {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JudgmentSummary {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JudgmentSummary {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for JudgmentSummary {
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
#[doc = "`Reason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"detail\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"unresolved_reference\","]
#[doc = "        \"missing_question\","]
#[doc = "        \"incomplete_payoff\","]
#[doc = "        \"ad_read\","]
#[doc = "        \"irrelevant_intro\","]
#[doc = "        \"misleading_omission\","]
#[doc = "        \"visual_dependency\","]
#[doc = "        \"other\","]
#[doc = "        \"transcript_uncertain\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"detail\": {"]
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
pub struct Reason {
    pub code: ReasonCode,
    pub detail: ReasonDetail,
}
impl Reason {
    pub fn builder() -> builder::Reason {
        Default::default()
    }
}
#[doc = "`ReasonCode`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"unresolved_reference\","]
#[doc = "    \"missing_question\","]
#[doc = "    \"incomplete_payoff\","]
#[doc = "    \"ad_read\","]
#[doc = "    \"irrelevant_intro\","]
#[doc = "    \"misleading_omission\","]
#[doc = "    \"visual_dependency\","]
#[doc = "    \"other\","]
#[doc = "    \"transcript_uncertain\""]
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
pub enum ReasonCode {
    #[serde(rename = "unresolved_reference")]
    UnresolvedReference,
    #[serde(rename = "missing_question")]
    MissingQuestion,
    #[serde(rename = "incomplete_payoff")]
    IncompletePayoff,
    #[serde(rename = "ad_read")]
    AdRead,
    #[serde(rename = "irrelevant_intro")]
    IrrelevantIntro,
    #[serde(rename = "misleading_omission")]
    MisleadingOmission,
    #[serde(rename = "visual_dependency")]
    VisualDependency,
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "transcript_uncertain")]
    TranscriptUncertain,
}
impl ::std::fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::UnresolvedReference => f.write_str("unresolved_reference"),
            Self::MissingQuestion => f.write_str("missing_question"),
            Self::IncompletePayoff => f.write_str("incomplete_payoff"),
            Self::AdRead => f.write_str("ad_read"),
            Self::IrrelevantIntro => f.write_str("irrelevant_intro"),
            Self::MisleadingOmission => f.write_str("misleading_omission"),
            Self::VisualDependency => f.write_str("visual_dependency"),
            Self::Other => f.write_str("other"),
            Self::TranscriptUncertain => f.write_str("transcript_uncertain"),
        }
    }
}
impl ::std::str::FromStr for ReasonCode {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "unresolved_reference" => Ok(Self::UnresolvedReference),
            "missing_question" => Ok(Self::MissingQuestion),
            "incomplete_payoff" => Ok(Self::IncompletePayoff),
            "ad_read" => Ok(Self::AdRead),
            "irrelevant_intro" => Ok(Self::IrrelevantIntro),
            "misleading_omission" => Ok(Self::MisleadingOmission),
            "visual_dependency" => Ok(Self::VisualDependency),
            "other" => Ok(Self::Other),
            "transcript_uncertain" => Ok(Self::TranscriptUncertain),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ReasonCode {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ReasonCode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ReasonCode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`ReasonDetail`"]
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
pub struct ReasonDetail(::std::string::String);
impl ::std::ops::Deref for ReasonDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ReasonDetail> for ::std::string::String {
    fn from(value: ReasonDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ReasonDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ReasonDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ReasonDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ReasonDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ReasonDetail {
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
    pub struct EditorialJudgments {
        candidates: ::std::result::Result<::std::vec::Vec<super::Judgment>, ::std::string::String>,
        content_profile:
            ::std::result::Result<super::EditorialJudgmentsContentProfile, ::std::string::String>,
        inputs: ::std::result::Result<super::EditorialJudgmentsInputs, ::std::string::String>,
        producer: ::std::result::Result<super::ModelProducer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialJudgments {
        fn default() -> Self {
            Self {
                candidates: Err("no value supplied for candidates".to_string()),
                content_profile: Ok(super::defaults::editorial_judgments_content_profile()),
                inputs: Err("no value supplied for inputs".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl EditorialJudgments {
        pub fn candidates<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Judgment>>,
            T::Error: ::std::fmt::Display,
        {
            self.candidates = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidates: {e}"));
            self
        }
        pub fn content_profile<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialJudgmentsContentProfile>,
            T::Error: ::std::fmt::Display,
        {
            self.content_profile = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for content_profile: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialJudgmentsInputs>,
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
    impl ::std::convert::TryFrom<EditorialJudgments> for super::EditorialJudgments {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialJudgments,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidates: value.candidates?,
                content_profile: value.content_profile?,
                inputs: value.inputs?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialJudgments> for EditorialJudgments {
        fn from(value: super::EditorialJudgments) -> Self {
            Self {
                candidates: Ok(value.candidates),
                content_profile: Ok(value.content_profile),
                inputs: Ok(value.inputs),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialJudgmentsInputs {
        candidates_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        windows_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialJudgmentsInputs {
        fn default() -> Self {
            Self {
                candidates_artifact_id: Err(
                    "no value supplied for candidates_artifact_id".to_string()
                ),
                windows_artifact_id: Err("no value supplied for windows_artifact_id".to_string()),
            }
        }
    }
    impl EditorialJudgmentsInputs {
        pub fn candidates_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.candidates_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for candidates_artifact_id: {e}")
            });
            self
        }
        pub fn windows_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.windows_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for windows_artifact_id: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<EditorialJudgmentsInputs> for super::EditorialJudgmentsInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialJudgmentsInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidates_artifact_id: value.candidates_artifact_id?,
                windows_artifact_id: value.windows_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialJudgmentsInputs> for EditorialJudgmentsInputs {
        fn from(value: super::EditorialJudgmentsInputs) -> Self {
            Self {
                candidates_artifact_id: Ok(value.candidates_artifact_id),
                windows_artifact_id: Ok(value.windows_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Judgment {
        candidate_id: ::std::result::Result<super::JudgmentCandidateId, ::std::string::String>,
        context: ::std::result::Result<super::JudgmentContext, ::std::string::String>,
        failure: ::std::result::Result<
            ::std::option::Option<super::JudgmentFailure>,
            ::std::string::String,
        >,
        outcome: ::std::result::Result<super::JudgmentOutcome, ::std::string::String>,
        reasons: ::std::result::Result<::std::vec::Vec<super::Reason>, ::std::string::String>,
        status: ::std::result::Result<
            ::std::option::Option<super::JudgmentStatus>,
            ::std::string::String,
        >,
        summary: ::std::result::Result<
            ::std::option::Option<super::JudgmentSummary>,
            ::std::string::String,
        >,
        visual_dependency:
            ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
    }
    impl ::std::default::Default for Judgment {
        fn default() -> Self {
            Self {
                candidate_id: Err("no value supplied for candidate_id".to_string()),
                context: Err("no value supplied for context".to_string()),
                failure: Ok(Default::default()),
                outcome: Err("no value supplied for outcome".to_string()),
                reasons: Err("no value supplied for reasons".to_string()),
                status: Ok(Default::default()),
                summary: Ok(Default::default()),
                visual_dependency: Ok(Default::default()),
            }
        }
    }
    impl Judgment {
        pub fn candidate_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JudgmentCandidateId>,
            T::Error: ::std::fmt::Display,
        {
            self.candidate_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidate_id: {e}"));
            self
        }
        pub fn context<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JudgmentContext>,
            T::Error: ::std::fmt::Display,
        {
            self.context = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for context: {e}"));
            self
        }
        pub fn failure<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::JudgmentFailure>>,
            T::Error: ::std::fmt::Display,
        {
            self.failure = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for failure: {e}"));
            self
        }
        pub fn outcome<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JudgmentOutcome>,
            T::Error: ::std::fmt::Display,
        {
            self.outcome = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outcome: {e}"));
            self
        }
        pub fn reasons<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Reason>>,
            T::Error: ::std::fmt::Display,
        {
            self.reasons = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reasons: {e}"));
            self
        }
        pub fn status<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::JudgmentStatus>>,
            T::Error: ::std::fmt::Display,
        {
            self.status = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for status: {e}"));
            self
        }
        pub fn summary<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::JudgmentSummary>>,
            T::Error: ::std::fmt::Display,
        {
            self.summary = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for summary: {e}"));
            self
        }
        pub fn visual_dependency<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.visual_dependency = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for visual_dependency: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Judgment> for super::Judgment {
        type Error = super::error::ConversionError;
        fn try_from(value: Judgment) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidate_id: value.candidate_id?,
                context: value.context?,
                failure: value.failure?,
                outcome: value.outcome?,
                reasons: value.reasons?,
                status: value.status?,
                summary: value.summary?,
                visual_dependency: value.visual_dependency?,
            })
        }
    }
    impl ::std::convert::From<super::Judgment> for Judgment {
        fn from(value: super::Judgment) -> Self {
            Self {
                candidate_id: Ok(value.candidate_id),
                context: Ok(value.context),
                failure: Ok(value.failure),
                outcome: Ok(value.outcome),
                reasons: Ok(value.reasons),
                status: Ok(value.status),
                summary: Ok(value.summary),
                visual_dependency: Ok(value.visual_dependency),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct JudgmentContext {
        first_sentence_index: ::std::result::Result<u64, ::std::string::String>,
        sentence_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for JudgmentContext {
        fn default() -> Self {
            Self {
                first_sentence_index: Err("no value supplied for first_sentence_index".to_string()),
                sentence_count: Err("no value supplied for sentence_count".to_string()),
            }
        }
    }
    impl JudgmentContext {
        pub fn first_sentence_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_sentence_index = value.try_into().map_err(|e| {
                format!("error converting supplied value for first_sentence_index: {e}")
            });
            self
        }
        pub fn sentence_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.sentence_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sentence_count: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<JudgmentContext> for super::JudgmentContext {
        type Error = super::error::ConversionError;
        fn try_from(
            value: JudgmentContext,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                first_sentence_index: value.first_sentence_index?,
                sentence_count: value.sentence_count?,
            })
        }
    }
    impl ::std::convert::From<super::JudgmentContext> for JudgmentContext {
        fn from(value: super::JudgmentContext) -> Self {
            Self {
                first_sentence_index: Ok(value.first_sentence_index),
                sentence_count: Ok(value.sentence_count),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct JudgmentFailure {
        detail: ::std::result::Result<super::JudgmentFailureDetail, ::std::string::String>,
        failure_class:
            ::std::result::Result<super::JudgmentFailureFailureClass, ::std::string::String>,
    }
    impl ::std::default::Default for JudgmentFailure {
        fn default() -> Self {
            Self {
                detail: Err("no value supplied for detail".to_string()),
                failure_class: Err("no value supplied for failure_class".to_string()),
            }
        }
    }
    impl JudgmentFailure {
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JudgmentFailureDetail>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn failure_class<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JudgmentFailureFailureClass>,
            T::Error: ::std::fmt::Display,
        {
            self.failure_class = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for failure_class: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<JudgmentFailure> for super::JudgmentFailure {
        type Error = super::error::ConversionError;
        fn try_from(
            value: JudgmentFailure,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                detail: value.detail?,
                failure_class: value.failure_class?,
            })
        }
    }
    impl ::std::convert::From<super::JudgmentFailure> for JudgmentFailure {
        fn from(value: super::JudgmentFailure) -> Self {
            Self {
                detail: Ok(value.detail),
                failure_class: Ok(value.failure_class),
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
    #[derive(Clone, Debug)]
    pub struct Reason {
        code: ::std::result::Result<super::ReasonCode, ::std::string::String>,
        detail: ::std::result::Result<super::ReasonDetail, ::std::string::String>,
    }
    impl ::std::default::Default for Reason {
        fn default() -> Self {
            Self {
                code: Err("no value supplied for code".to_string()),
                detail: Err("no value supplied for detail".to_string()),
            }
        }
    }
    impl Reason {
        pub fn code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ReasonCode>,
            T::Error: ::std::fmt::Display,
        {
            self.code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for code: {e}"));
            self
        }
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ReasonDetail>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Reason> for super::Reason {
        type Error = super::error::ConversionError;
        fn try_from(value: Reason) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                code: value.code?,
                detail: value.detail?,
            })
        }
    }
    impl ::std::convert::From<super::Reason> for Reason {
        fn from(value: super::Reason) -> Self {
            Self {
                code: Ok(value.code),
                detail: Ok(value.detail),
            }
        }
    }
}
#[doc = r" Generation of default values for serde."]
pub mod defaults {
    pub(super) fn editorial_judgments_content_profile() -> super::EditorialJudgmentsContentProfile {
        super::EditorialJudgmentsContentProfile::Interview
    }
}
