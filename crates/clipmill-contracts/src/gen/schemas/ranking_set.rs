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
#[doc = "Where this clip should actually be cut, chosen by scoring every legal pair in the candidate's lattice. The runner-up ships beside it because the optimizer's second choice is frequently the editor's first, and a boundary alternative one click away is cheaper than re-running the search."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Where this clip should actually be cut, chosen by scoring every legal pair in the candidate's lattice. The runner-up ships beside it because the optimizer's second choice is frequently the editor's first, and a boundary alternative one click away is cheaper than re-running the search.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"chosen\","]
#[doc = "    \"score\","]
#[doc = "    \"terms\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"alternative\": {"]
#[doc = "      \"description\": \"The runner-up, absent only when the lattice offered exactly one legal pair.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"interval\","]
#[doc = "        \"score\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"interval\": {"]
#[doc = "          \"$ref\": \"#/$defs/interval\""]
#[doc = "        },"]
#[doc = "        \"score\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"chosen\": {"]
#[doc = "      \"$ref\": \"#/$defs/interval\""]
#[doc = "    },"]
#[doc = "    \"considered\": {"]
#[doc = "      \"description\": \"How many legal pairs were scored. The design expects this to stay under a few hundred, which is why the search is exhaustive rather than heuristic.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"terms\": {"]
#[doc = "      \"description\": \"The seven weighted terms behind the choice, so a boundary a user disagrees with can be argued with.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/boundary_term\""]
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
pub struct Boundary {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub alternative: ::std::option::Option<BoundaryAlternative>,
    pub chosen: Interval,
    #[doc = "How many legal pairs were scored. The design expects this to stay under a few hundred, which is why the search is exhaustive rather than heuristic."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub considered: ::std::option::Option<::std::num::NonZeroU64>,
    pub score: f64,
    #[doc = "The seven weighted terms behind the choice, so a boundary a user disagrees with can be argued with."]
    pub terms: ::std::vec::Vec<BoundaryTerm>,
}
impl Boundary {
    pub fn builder() -> builder::Boundary {
        Default::default()
    }
}
#[doc = "The runner-up, absent only when the lattice offered exactly one legal pair."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The runner-up, absent only when the lattice offered exactly one legal pair.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"interval\","]
#[doc = "    \"score\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"interval\": {"]
#[doc = "      \"$ref\": \"#/$defs/interval\""]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct BoundaryAlternative {
    pub interval: Interval,
    pub score: f64,
}
impl BoundaryAlternative {
    pub fn builder() -> builder::BoundaryAlternative {
        Default::default()
    }
}
#[doc = "`BoundaryTerm`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"name\","]
#[doc = "    \"value\","]
#[doc = "    \"weight\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"name\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"completeness\","]
#[doc = "        \"hook\","]
#[doc = "        \"payoff\","]
#[doc = "        \"continuity\","]
#[doc = "        \"deadair\","]
#[doc = "        \"abrupt\","]
#[doc = "        \"context_debt\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"value\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"weight\": {"]
#[doc = "      \"description\": \"Signed: the first four terms reward and the last three penalise, and carrying the sign here means a reader does not have to know which is which.\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct BoundaryTerm {
    pub name: BoundaryTermName,
    pub value: f64,
    #[doc = "Signed: the first four terms reward and the last three penalise, and carrying the sign here means a reader does not have to know which is which."]
    pub weight: f64,
}
impl BoundaryTerm {
    pub fn builder() -> builder::BoundaryTerm {
        Default::default()
    }
}
#[doc = "`BoundaryTermName`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"completeness\","]
#[doc = "    \"hook\","]
#[doc = "    \"payoff\","]
#[doc = "    \"continuity\","]
#[doc = "    \"deadair\","]
#[doc = "    \"abrupt\","]
#[doc = "    \"context_debt\""]
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
pub enum BoundaryTermName {
    #[serde(rename = "completeness")]
    Completeness,
    #[serde(rename = "hook")]
    Hook,
    #[serde(rename = "payoff")]
    Payoff,
    #[serde(rename = "continuity")]
    Continuity,
    #[serde(rename = "deadair")]
    Deadair,
    #[serde(rename = "abrupt")]
    Abrupt,
    #[serde(rename = "context_debt")]
    ContextDebt,
}
impl ::std::fmt::Display for BoundaryTermName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Completeness => f.write_str("completeness"),
            Self::Hook => f.write_str("hook"),
            Self::Payoff => f.write_str("payoff"),
            Self::Continuity => f.write_str("continuity"),
            Self::Deadair => f.write_str("deadair"),
            Self::Abrupt => f.write_str("abrupt"),
            Self::ContextDebt => f.write_str("context_debt"),
        }
    }
}
impl ::std::str::FromStr for BoundaryTermName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "completeness" => Ok(Self::Completeness),
            "hook" => Ok(Self::Hook),
            "payoff" => Ok(Self::Payoff),
            "continuity" => Ok(Self::Continuity),
            "deadair" => Ok(Self::Deadair),
            "abrupt" => Ok(Self::Abrupt),
            "context_debt" => Ok(Self::ContextDebt),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for BoundaryTermName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for BoundaryTermName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for BoundaryTermName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CandidateId`"]
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
pub struct CandidateId(::std::string::String);
impl ::std::ops::Deref for CandidateId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CandidateId> for ::std::string::String {
    fn from(value: CandidateId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CandidateId {
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
impl ::std::convert::TryFrom<&str> for CandidateId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CandidateId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CandidateId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CandidateId {
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
#[doc = "`EvidenceReference`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"index\","]
#[doc = "    \"kind\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"utterance\","]
#[doc = "        \"sentence\","]
#[doc = "        \"topic\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReference {
    pub index: u64,
    pub kind: EvidenceReferenceKind,
}
impl EvidenceReference {
    pub fn builder() -> builder::EvidenceReference {
        Default::default()
    }
}
#[doc = "`EvidenceReferenceKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"utterance\","]
#[doc = "    \"sentence\","]
#[doc = "    \"topic\""]
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
pub enum EvidenceReferenceKind {
    #[serde(rename = "utterance")]
    Utterance,
    #[serde(rename = "sentence")]
    Sentence,
    #[serde(rename = "topic")]
    Topic,
}
impl ::std::fmt::Display for EvidenceReferenceKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Utterance => f.write_str("utterance"),
            Self::Sentence => f.write_str("sentence"),
            Self::Topic => f.write_str("topic"),
        }
    }
}
impl ::std::str::FromStr for EvidenceReferenceKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "utterance" => Ok(Self::Utterance),
            "sentence" => Ok(Self::Sentence),
            "topic" => Ok(Self::Topic),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EvidenceReferenceKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EvidenceReferenceKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EvidenceReferenceKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "One axis of the score card. A factor this phase cannot measure is reported `available: false` with a stated reason and contributes nothing to the score — not zero, which would read as a measurement of badness, and not a neutral default, which would read as a measurement at all."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"One axis of the score card. A factor this phase cannot measure is reported `available: false` with a stated reason and contributes nothing to the score — not zero, which would read as a measurement of badness, and not a neutral default, which would read as a measurement at all.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"available\","]
#[doc = "    \"name\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"available\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"evidence\": {"]
#[doc = "      \"description\": \"What the factor was read from, so a user asking 'why is the hook strong?' gets the sentence rather than the number again.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/evidence_reference\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"hook\","]
#[doc = "        \"flow\","]
#[doc = "        \"value\","]
#[doc = "        \"prompt_relevance\","]
#[doc = "        \"novelty\","]
#[doc = "        \"evidence\","]
#[doc = "        \"craft\","]
#[doc = "        \"feasibility\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"unavailable_reason\": {"]
#[doc = "      \"description\": \"Present when not available. Why nothing measured this axis, e.g. 'no prompt was given' or 'this source carries no audio'.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"value\": {"]
#[doc = "      \"description\": \"Present when available. Zero to one before weighting.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"weight\": {"]
#[doc = "      \"description\": \"Present when available. What this factor contributed to q(c), before the penalties.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Factor {
    pub available: bool,
    #[doc = "What the factor was read from, so a user asking 'why is the hook strong?' gets the sentence rather than the number again."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub evidence: ::std::vec::Vec<EvidenceReference>,
    pub name: FactorName,
    #[doc = "Present when not available. Why nothing measured this axis, e.g. 'no prompt was given' or 'this source carries no audio'."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub unavailable_reason: ::std::option::Option<FactorUnavailableReason>,
    #[doc = "Present when available. Zero to one before weighting."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub value: ::std::option::Option<f64>,
    #[doc = "Present when available. What this factor contributed to q(c), before the penalties."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub weight: ::std::option::Option<f64>,
}
impl Factor {
    pub fn builder() -> builder::Factor {
        Default::default()
    }
}
#[doc = "`FactorName`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"hook\","]
#[doc = "    \"flow\","]
#[doc = "    \"value\","]
#[doc = "    \"prompt_relevance\","]
#[doc = "    \"novelty\","]
#[doc = "    \"evidence\","]
#[doc = "    \"craft\","]
#[doc = "    \"feasibility\""]
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
pub enum FactorName {
    #[serde(rename = "hook")]
    Hook,
    #[serde(rename = "flow")]
    Flow,
    #[serde(rename = "value")]
    Value,
    #[serde(rename = "prompt_relevance")]
    PromptRelevance,
    #[serde(rename = "novelty")]
    Novelty,
    #[serde(rename = "evidence")]
    Evidence,
    #[serde(rename = "craft")]
    Craft,
    #[serde(rename = "feasibility")]
    Feasibility,
}
impl ::std::fmt::Display for FactorName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Hook => f.write_str("hook"),
            Self::Flow => f.write_str("flow"),
            Self::Value => f.write_str("value"),
            Self::PromptRelevance => f.write_str("prompt_relevance"),
            Self::Novelty => f.write_str("novelty"),
            Self::Evidence => f.write_str("evidence"),
            Self::Craft => f.write_str("craft"),
            Self::Feasibility => f.write_str("feasibility"),
        }
    }
}
impl ::std::str::FromStr for FactorName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "hook" => Ok(Self::Hook),
            "flow" => Ok(Self::Flow),
            "value" => Ok(Self::Value),
            "prompt_relevance" => Ok(Self::PromptRelevance),
            "novelty" => Ok(Self::Novelty),
            "evidence" => Ok(Self::Evidence),
            "craft" => Ok(Self::Craft),
            "feasibility" => Ok(Self::Feasibility),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for FactorName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for FactorName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for FactorName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Present when not available. Why nothing measured this axis, e.g. 'no prompt was given' or 'this source carries no audio'."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Present when not available. Why nothing measured this axis, e.g. 'no prompt was given' or 'this source carries no audio'.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct FactorUnavailableReason(::std::string::String);
impl ::std::ops::Deref for FactorUnavailableReason {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<FactorUnavailableReason> for ::std::string::String {
    fn from(value: FactorUnavailableReason) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for FactorUnavailableReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for FactorUnavailableReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for FactorUnavailableReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for FactorUnavailableReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for FactorUnavailableReason {
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
#[doc = "`FilteredCandidate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidate_id\","]
#[doc = "    \"reason\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidate_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/candidate_id\""]
#[doc = "    },"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"excluded_by_discovery\","]
#[doc = "        \"no_legal_boundary\","]
#[doc = "        \"below_floor\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FilteredCandidate {
    pub candidate_id: CandidateId,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub detail: ::std::option::Option<FilteredCandidateDetail>,
    pub reason: FilteredCandidateReason,
}
impl FilteredCandidate {
    pub fn builder() -> builder::FilteredCandidate {
        Default::default()
    }
}
#[doc = "`FilteredCandidateDetail`"]
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
pub struct FilteredCandidateDetail(::std::string::String);
impl ::std::ops::Deref for FilteredCandidateDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<FilteredCandidateDetail> for ::std::string::String {
    fn from(value: FilteredCandidateDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for FilteredCandidateDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for FilteredCandidateDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for FilteredCandidateDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for FilteredCandidateDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for FilteredCandidateDetail {
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
#[doc = "`FilteredCandidateReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"excluded_by_discovery\","]
#[doc = "    \"no_legal_boundary\","]
#[doc = "    \"below_floor\""]
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
pub enum FilteredCandidateReason {
    #[serde(rename = "excluded_by_discovery")]
    ExcludedByDiscovery,
    #[serde(rename = "no_legal_boundary")]
    NoLegalBoundary,
    #[serde(rename = "below_floor")]
    BelowFloor,
}
impl ::std::fmt::Display for FilteredCandidateReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ExcludedByDiscovery => f.write_str("excluded_by_discovery"),
            Self::NoLegalBoundary => f.write_str("no_legal_boundary"),
            Self::BelowFloor => f.write_str("below_floor"),
        }
    }
}
impl ::std::str::FromStr for FilteredCandidateReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "excluded_by_discovery" => Ok(Self::ExcludedByDiscovery),
            "no_legal_boundary" => Ok(Self::NoLegalBoundary),
            "below_floor" => Ok(Self::BelowFloor),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for FilteredCandidateReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for FilteredCandidateReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for FilteredCandidateReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
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
#[doc = "`Penalty`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"reason\","]
#[doc = "    \"value\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"description\": \"'repetition' is overlap with a candidate already ranked above this one. 'context_debt' is a clip that opens on something the viewer has not been told. 'rights_risk' cannot fire at this phase — there is no rights ledger — and is listed so the shape does not change when there is one.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"repetition\","]
#[doc = "        \"context_debt\","]
#[doc = "        \"rights_risk\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"value\": {"]
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
pub struct Penalty {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub detail: ::std::option::Option<PenaltyDetail>,
    #[doc = "'repetition' is overlap with a candidate already ranked above this one. 'context_debt' is a clip that opens on something the viewer has not been told. 'rights_risk' cannot fire at this phase — there is no rights ledger — and is listed so the shape does not change when there is one."]
    pub reason: PenaltyReason,
    pub value: f64,
}
impl Penalty {
    pub fn builder() -> builder::Penalty {
        Default::default()
    }
}
#[doc = "`PenaltyDetail`"]
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
pub struct PenaltyDetail(::std::string::String);
impl ::std::ops::Deref for PenaltyDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PenaltyDetail> for ::std::string::String {
    fn from(value: PenaltyDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for PenaltyDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PenaltyDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PenaltyDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PenaltyDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PenaltyDetail {
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
#[doc = "'repetition' is overlap with a candidate already ranked above this one. 'context_debt' is a clip that opens on something the viewer has not been told. 'rights_risk' cannot fire at this phase — there is no rights ledger — and is listed so the shape does not change when there is one."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"'repetition' is overlap with a candidate already ranked above this one. 'context_debt' is a clip that opens on something the viewer has not been told. 'rights_risk' cannot fire at this phase — there is no rights ledger — and is listed so the shape does not change when there is one.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"repetition\","]
#[doc = "    \"context_debt\","]
#[doc = "    \"rights_risk\""]
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
pub enum PenaltyReason {
    #[serde(rename = "repetition")]
    Repetition,
    #[serde(rename = "context_debt")]
    ContextDebt,
    #[serde(rename = "rights_risk")]
    RightsRisk,
}
impl ::std::fmt::Display for PenaltyReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Repetition => f.write_str("repetition"),
            Self::ContextDebt => f.write_str("context_debt"),
            Self::RightsRisk => f.write_str("rights_risk"),
        }
    }
}
impl ::std::str::FromStr for PenaltyReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "repetition" => Ok(Self::Repetition),
            "context_debt" => Ok(Self::ContextDebt),
            "rights_risk" => Ok(Self::RightsRisk),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for PenaltyReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PenaltyReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PenaltyReason {
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
pub struct Producer {
    pub implementation: ProducerImplementation,
    pub stage: ProducerStage,
}
impl Producer {
    pub fn builder() -> builder::Producer {
        Default::default()
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
#[doc = "`Ranked`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"boundary\","]
#[doc = "    \"candidate_id\","]
#[doc = "    \"cluster_id\","]
#[doc = "    \"display_score\","]
#[doc = "    \"factors\","]
#[doc = "    \"penalties\","]
#[doc = "    \"rank\","]
#[doc = "    \"score\","]
#[doc = "    \"uncertainty\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"boundary\": {"]
#[doc = "      \"$ref\": \"#/$defs/boundary\""]
#[doc = "    },"]
#[doc = "    \"candidate_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/candidate_id\""]
#[doc = "    },"]
#[doc = "    \"cluster_id\": {"]
#[doc = "      \"description\": \"Echoed from discovery, so the interface can offer the cluster's alternatives without opening the candidate set.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^cl_[0-9a-f]{16}$\""]
#[doc = "    },"]
#[doc = "    \"display_score\": {"]
#[doc = "      \"description\": \"The 0-99 the interface shows: a percentile within this cohort, not a probability and not comparable across recordings. Named `display_score` rather than `score` so nobody stores it as one.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 99.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"factors\": {"]
#[doc = "      \"description\": \"The score card, decomposed. Each factor names itself, so a card that grew a factor and a card that lost one are distinguishable documents.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/factor\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"penalties\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/penalty\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"rank\": {"]
#[doc = "      \"description\": \"Position in the cohort, from one. Ties are broken by candidate id, so the order does not depend on which proposer ran first.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"description\": \"The raw q(c) the percentile came from, kept so a re-tune can be reasoned about without recomputing and so two candidates a percentile rounded together can still be told apart.\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"uncertainty\": {"]
#[doc = "      \"$ref\": \"#/$defs/uncertainty\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Ranked {
    pub boundary: Boundary,
    pub candidate_id: CandidateId,
    #[doc = "Echoed from discovery, so the interface can offer the cluster's alternatives without opening the candidate set."]
    pub cluster_id: RankedClusterId,
    #[doc = "The 0-99 the interface shows: a percentile within this cohort, not a probability and not comparable across recordings. Named `display_score` rather than `score` so nobody stores it as one."]
    pub display_score: i64,
    #[doc = "The score card, decomposed. Each factor names itself, so a card that grew a factor and a card that lost one are distinguishable documents."]
    pub factors: ::std::vec::Vec<Factor>,
    pub penalties: ::std::vec::Vec<Penalty>,
    #[doc = "Position in the cohort, from one. Ties are broken by candidate id, so the order does not depend on which proposer ran first."]
    pub rank: ::std::num::NonZeroU64,
    #[doc = "The raw q(c) the percentile came from, kept so a re-tune can be reasoned about without recomputing and so two candidates a percentile rounded together can still be told apart."]
    pub score: f64,
    pub uncertainty: Uncertainty,
}
impl Ranked {
    pub fn builder() -> builder::Ranked {
        Default::default()
    }
}
#[doc = "Echoed from discovery, so the interface can offer the cluster's alternatives without opening the candidate set."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Echoed from discovery, so the interface can offer the cluster's alternatives without opening the candidate set.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^cl_[0-9a-f]{16}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RankedClusterId(::std::string::String);
impl ::std::ops::Deref for RankedClusterId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankedClusterId> for ::std::string::String {
    fn from(value: RankedClusterId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankedClusterId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^cl_[0-9a-f]{16}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^cl_[0-9a-f]{16}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RankedClusterId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankedClusterId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankedClusterId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankedClusterId {
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
#[doc = "Which nominated clips are worth showing, in what order, and cut where (book ch. 16). Three decisions live here that discovery deliberately left open: what each candidate is worth, where its boundaries should actually fall, and which subset of a cohort a user should see. Every one of them is shown rather than asserted — the score is decomposed into named factors with the evidence behind them, the chosen boundary is published beside the runner-up it beat, and a set smaller than the one requested says why instead of padding itself."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.ranking.set.v1.json\","]
#[doc = "  \"title\": \"RankingSet\","]
#[doc = "  \"description\": \"Which nominated clips are worth showing, in what order, and cut where (book ch. 16). Three decisions live here that discovery deliberately left open: what each candidate is worth, where its boundaries should actually fall, and which subset of a cohort a user should see. Every one of them is shown rather than asserted — the score is decomposed into named factors with the evidence behind them, the chosen boundary is published beside the runner-up it beat, and a set smaller than the one requested says why instead of padding itself.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cohort\","]
#[doc = "    \"inputs\","]
#[doc = "    \"producer\","]
#[doc = "    \"requested\","]
#[doc = "    \"rubric\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"selected\","]
#[doc = "    \"shortfall\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cohort\": {"]
#[doc = "      \"description\": \"Every candidate that survived the stage-one filters, scored and ordered. Percentiles are within this list, which is what makes the displayed number an editorial index rather than a probability.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/ranked\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"filtered\": {"]
#[doc = "      \"description\": \"Candidates the stage-one filters removed before scoring, with the reason. Kept so the interface can answer 'what happened to that one?' rather than the candidate simply vanishing between two documents.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/filtered_candidate\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"description\": \"What was read. The candidate set is the authority for every candidate id below; the index and transcript are what the factors were measured against.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"candidates_artifact_id\","]
#[doc = "        \"index_artifact_id\","]
#[doc = "        \"transcript_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"candidates_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"index_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"transcript_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"requested\": {"]
#[doc = "      \"description\": \"How many clips the caller asked for, and the diversity trade-off used to pick them.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"count\","]
#[doc = "        \"diversity\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"count\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"diversity\": {"]
#[doc = "          \"description\": \"The lambda in maximal marginal relevance: one takes the best clips whatever their overlap, zero takes the most different ones whatever their quality.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"rubric\": {"]
#[doc = "      \"description\": \"Which scoring rules produced these numbers. Hand-set weights calibrated against nothing yet, and named as such: the design calibrates them per genre against human labels, and until that has happened a version string is the only honest way to say which arithmetic ran. Part of the artifact key.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"boundary\","]
#[doc = "        \"scorer\","]
#[doc = "        \"selector\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"boundary\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"scorer\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"selector\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.ranking.set.v1\""]
#[doc = "    },"]
#[doc = "    \"selected\": {"]
#[doc = "      \"description\": \"The clips to show, in order. A subset of the cohort chosen for quality and difference together, so a user is not handed five versions of one moment.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"shortfall\": {"]
#[doc = "      \"description\": \"Why fewer clips were selected than requested. Empty when the request was met. Never padded: a recording that holds three good moments should return three and say so, because the fourth would be a clip the system does not believe in.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/shortfall_reason\""]
#[doc = "      }"]
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
pub struct RankingSet {
    #[doc = "Every candidate that survived the stage-one filters, scored and ordered. Percentiles are within this list, which is what makes the displayed number an editorial index rather than a probability."]
    pub cohort: ::std::vec::Vec<Ranked>,
    #[doc = "Candidates the stage-one filters removed before scoring, with the reason. Kept so the interface can answer 'what happened to that one?' rather than the candidate simply vanishing between two documents."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub filtered: ::std::vec::Vec<FilteredCandidate>,
    pub inputs: RankingSetInputs,
    pub producer: Producer,
    pub requested: RankingSetRequested,
    pub rubric: RankingSetRubric,
    pub schema_version: ::serde_json::Value,
    #[doc = "The clips to show, in order. A subset of the cohort chosen for quality and difference together, so a user is not handed five versions of one moment."]
    pub selected: ::std::vec::Vec<RankingSetSelectedItem>,
    #[doc = "Why fewer clips were selected than requested. Empty when the request was met. Never padded: a recording that holds three good moments should return three and say so, because the fourth would be a clip the system does not believe in."]
    pub shortfall: ::std::vec::Vec<ShortfallReason>,
    pub source_fingerprint: Sha256,
}
impl RankingSet {
    pub fn builder() -> builder::RankingSet {
        Default::default()
    }
}
#[doc = "What was read. The candidate set is the authority for every candidate id below; the index and transcript are what the factors were measured against."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was read. The candidate set is the authority for every candidate id below; the index and transcript are what the factors were measured against.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidates_artifact_id\","]
#[doc = "    \"index_artifact_id\","]
#[doc = "    \"transcript_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidates_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"index_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"transcript_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RankingSetInputs {
    pub candidates_artifact_id: Sha256,
    pub index_artifact_id: Sha256,
    pub transcript_artifact_id: Sha256,
}
impl RankingSetInputs {
    pub fn builder() -> builder::RankingSetInputs {
        Default::default()
    }
}
#[doc = "How many clips the caller asked for, and the diversity trade-off used to pick them."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How many clips the caller asked for, and the diversity trade-off used to pick them.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"count\","]
#[doc = "    \"diversity\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"diversity\": {"]
#[doc = "      \"description\": \"The lambda in maximal marginal relevance: one takes the best clips whatever their overlap, zero takes the most different ones whatever their quality.\","]
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
pub struct RankingSetRequested {
    pub count: ::std::num::NonZeroU64,
    #[doc = "The lambda in maximal marginal relevance: one takes the best clips whatever their overlap, zero takes the most different ones whatever their quality."]
    pub diversity: f64,
}
impl RankingSetRequested {
    pub fn builder() -> builder::RankingSetRequested {
        Default::default()
    }
}
#[doc = "Which scoring rules produced these numbers. Hand-set weights calibrated against nothing yet, and named as such: the design calibrates them per genre against human labels, and until that has happened a version string is the only honest way to say which arithmetic ran. Part of the artifact key."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Which scoring rules produced these numbers. Hand-set weights calibrated against nothing yet, and named as such: the design calibrates them per genre against human labels, and until that has happened a version string is the only honest way to say which arithmetic ran. Part of the artifact key.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"boundary\","]
#[doc = "    \"scorer\","]
#[doc = "    \"selector\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"boundary\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"scorer\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"selector\": {"]
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
pub struct RankingSetRubric {
    pub boundary: RankingSetRubricBoundary,
    pub scorer: RankingSetRubricScorer,
    pub selector: RankingSetRubricSelector,
}
impl RankingSetRubric {
    pub fn builder() -> builder::RankingSetRubric {
        Default::default()
    }
}
#[doc = "`RankingSetRubricBoundary`"]
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
pub struct RankingSetRubricBoundary(::std::string::String);
impl ::std::ops::Deref for RankingSetRubricBoundary {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankingSetRubricBoundary> for ::std::string::String {
    fn from(value: RankingSetRubricBoundary) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankingSetRubricBoundary {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RankingSetRubricBoundary {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankingSetRubricBoundary {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankingSetRubricBoundary {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankingSetRubricBoundary {
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
#[doc = "`RankingSetRubricScorer`"]
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
pub struct RankingSetRubricScorer(::std::string::String);
impl ::std::ops::Deref for RankingSetRubricScorer {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankingSetRubricScorer> for ::std::string::String {
    fn from(value: RankingSetRubricScorer) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankingSetRubricScorer {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RankingSetRubricScorer {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankingSetRubricScorer {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankingSetRubricScorer {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankingSetRubricScorer {
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
#[doc = "`RankingSetRubricSelector`"]
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
pub struct RankingSetRubricSelector(::std::string::String);
impl ::std::ops::Deref for RankingSetRubricSelector {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankingSetRubricSelector> for ::std::string::String {
    fn from(value: RankingSetRubricSelector) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankingSetRubricSelector {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RankingSetRubricSelector {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankingSetRubricSelector {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankingSetRubricSelector {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankingSetRubricSelector {
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
#[doc = "`RankingSetSelectedItem`"]
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
pub struct RankingSetSelectedItem(::std::string::String);
impl ::std::ops::Deref for RankingSetSelectedItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RankingSetSelectedItem> for ::std::string::String {
    fn from(value: RankingSetSelectedItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RankingSetSelectedItem {
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
impl ::std::convert::TryFrom<&str> for RankingSetSelectedItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RankingSetSelectedItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RankingSetSelectedItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RankingSetSelectedItem {
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
#[doc = "`ShortfallReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"count\","]
#[doc = "    \"reason\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"count\": {"]
#[doc = "      \"description\": \"How many of the requested clips this reason accounts for.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"cohort_exhausted\","]
#[doc = "        \"all_remaining_are_duplicates\","]
#[doc = "        \"all_remaining_below_bar\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShortfallReason {
    #[doc = "How many of the requested clips this reason accounts for."]
    pub count: ::std::num::NonZeroU64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub detail: ::std::option::Option<ShortfallReasonDetail>,
    pub reason: ShortfallReasonReason,
}
impl ShortfallReason {
    pub fn builder() -> builder::ShortfallReason {
        Default::default()
    }
}
#[doc = "`ShortfallReasonDetail`"]
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
pub struct ShortfallReasonDetail(::std::string::String);
impl ::std::ops::Deref for ShortfallReasonDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ShortfallReasonDetail> for ::std::string::String {
    fn from(value: ShortfallReasonDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ShortfallReasonDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ShortfallReasonDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ShortfallReasonDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ShortfallReasonDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ShortfallReasonDetail {
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
#[doc = "`ShortfallReasonReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"cohort_exhausted\","]
#[doc = "    \"all_remaining_are_duplicates\","]
#[doc = "    \"all_remaining_below_bar\""]
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
pub enum ShortfallReasonReason {
    #[serde(rename = "cohort_exhausted")]
    CohortExhausted,
    #[serde(rename = "all_remaining_are_duplicates")]
    AllRemainingAreDuplicates,
    #[serde(rename = "all_remaining_below_bar")]
    AllRemainingBelowBar,
}
impl ::std::fmt::Display for ShortfallReasonReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::CohortExhausted => f.write_str("cohort_exhausted"),
            Self::AllRemainingAreDuplicates => f.write_str("all_remaining_are_duplicates"),
            Self::AllRemainingBelowBar => f.write_str("all_remaining_below_bar"),
        }
    }
}
impl ::std::str::FromStr for ShortfallReasonReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "cohort_exhausted" => Ok(Self::CohortExhausted),
            "all_remaining_are_duplicates" => Ok(Self::AllRemainingAreDuplicates),
            "all_remaining_below_bar" => Ok(Self::AllRemainingBelowBar),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ShortfallReasonReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ShortfallReasonReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ShortfallReasonReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "How much this card should be trusted, and why. Translated by the interface into words rather than shown as a hidden shading of the score."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How much this card should be trusted, and why. Translated by the interface into words rather than shown as a hidden shading of the score.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"band\","]
#[doc = "    \"value\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"band\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"strong\","]
#[doc = "        \"promising\","]
#[doc = "        \"needs_review\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"value\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"warnings\": {"]
#[doc = "      \"description\": \"Factor-specific cautions, e.g. 'word timing here was interpolated rather than measured'.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Uncertainty {
    pub band: UncertaintyBand,
    pub value: f64,
    #[doc = "Factor-specific cautions, e.g. 'word timing here was interpolated rather than measured'."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub warnings: ::std::vec::Vec<UncertaintyWarningsItem>,
}
impl Uncertainty {
    pub fn builder() -> builder::Uncertainty {
        Default::default()
    }
}
#[doc = "`UncertaintyBand`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"strong\","]
#[doc = "    \"promising\","]
#[doc = "    \"needs_review\""]
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
pub enum UncertaintyBand {
    #[serde(rename = "strong")]
    Strong,
    #[serde(rename = "promising")]
    Promising,
    #[serde(rename = "needs_review")]
    NeedsReview,
}
impl ::std::fmt::Display for UncertaintyBand {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Strong => f.write_str("strong"),
            Self::Promising => f.write_str("promising"),
            Self::NeedsReview => f.write_str("needs_review"),
        }
    }
}
impl ::std::str::FromStr for UncertaintyBand {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "strong" => Ok(Self::Strong),
            "promising" => Ok(Self::Promising),
            "needs_review" => Ok(Self::NeedsReview),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for UncertaintyBand {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for UncertaintyBand {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UncertaintyBand {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`UncertaintyWarningsItem`"]
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
pub struct UncertaintyWarningsItem(::std::string::String);
impl ::std::ops::Deref for UncertaintyWarningsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<UncertaintyWarningsItem> for ::std::string::String {
    fn from(value: UncertaintyWarningsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for UncertaintyWarningsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for UncertaintyWarningsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for UncertaintyWarningsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UncertaintyWarningsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for UncertaintyWarningsItem {
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
    pub struct Boundary {
        alternative: ::std::result::Result<
            ::std::option::Option<super::BoundaryAlternative>,
            ::std::string::String,
        >,
        chosen: ::std::result::Result<super::Interval, ::std::string::String>,
        considered: ::std::result::Result<
            ::std::option::Option<::std::num::NonZeroU64>,
            ::std::string::String,
        >,
        score: ::std::result::Result<f64, ::std::string::String>,
        terms: ::std::result::Result<::std::vec::Vec<super::BoundaryTerm>, ::std::string::String>,
    }
    impl ::std::default::Default for Boundary {
        fn default() -> Self {
            Self {
                alternative: Ok(Default::default()),
                chosen: Err("no value supplied for chosen".to_string()),
                considered: Ok(Default::default()),
                score: Err("no value supplied for score".to_string()),
                terms: Err("no value supplied for terms".to_string()),
            }
        }
    }
    impl Boundary {
        pub fn alternative<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::BoundaryAlternative>>,
            T::Error: ::std::fmt::Display,
        {
            self.alternative = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for alternative: {e}"));
            self
        }
        pub fn chosen<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Interval>,
            T::Error: ::std::fmt::Display,
        {
            self.chosen = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for chosen: {e}"));
            self
        }
        pub fn considered<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::num::NonZeroU64>>,
            T::Error: ::std::fmt::Display,
        {
            self.considered = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for considered: {e}"));
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
        pub fn terms<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::BoundaryTerm>>,
            T::Error: ::std::fmt::Display,
        {
            self.terms = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for terms: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Boundary> for super::Boundary {
        type Error = super::error::ConversionError;
        fn try_from(value: Boundary) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                alternative: value.alternative?,
                chosen: value.chosen?,
                considered: value.considered?,
                score: value.score?,
                terms: value.terms?,
            })
        }
    }
    impl ::std::convert::From<super::Boundary> for Boundary {
        fn from(value: super::Boundary) -> Self {
            Self {
                alternative: Ok(value.alternative),
                chosen: Ok(value.chosen),
                considered: Ok(value.considered),
                score: Ok(value.score),
                terms: Ok(value.terms),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct BoundaryAlternative {
        interval: ::std::result::Result<super::Interval, ::std::string::String>,
        score: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for BoundaryAlternative {
        fn default() -> Self {
            Self {
                interval: Err("no value supplied for interval".to_string()),
                score: Err("no value supplied for score".to_string()),
            }
        }
    }
    impl BoundaryAlternative {
        pub fn interval<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Interval>,
            T::Error: ::std::fmt::Display,
        {
            self.interval = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for interval: {e}"));
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
    impl ::std::convert::TryFrom<BoundaryAlternative> for super::BoundaryAlternative {
        type Error = super::error::ConversionError;
        fn try_from(
            value: BoundaryAlternative,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                interval: value.interval?,
                score: value.score?,
            })
        }
    }
    impl ::std::convert::From<super::BoundaryAlternative> for BoundaryAlternative {
        fn from(value: super::BoundaryAlternative) -> Self {
            Self {
                interval: Ok(value.interval),
                score: Ok(value.score),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct BoundaryTerm {
        name: ::std::result::Result<super::BoundaryTermName, ::std::string::String>,
        value: ::std::result::Result<f64, ::std::string::String>,
        weight: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for BoundaryTerm {
        fn default() -> Self {
            Self {
                name: Err("no value supplied for name".to_string()),
                value: Err("no value supplied for value".to_string()),
                weight: Err("no value supplied for weight".to_string()),
            }
        }
    }
    impl BoundaryTerm {
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::BoundaryTermName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn value<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.value = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for value: {e}"));
            self
        }
        pub fn weight<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.weight = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for weight: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<BoundaryTerm> for super::BoundaryTerm {
        type Error = super::error::ConversionError;
        fn try_from(
            value: BoundaryTerm,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                name: value.name?,
                value: value.value?,
                weight: value.weight?,
            })
        }
    }
    impl ::std::convert::From<super::BoundaryTerm> for BoundaryTerm {
        fn from(value: super::BoundaryTerm) -> Self {
            Self {
                name: Ok(value.name),
                value: Ok(value.value),
                weight: Ok(value.weight),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EvidenceReference {
        index: ::std::result::Result<u64, ::std::string::String>,
        kind: ::std::result::Result<super::EvidenceReferenceKind, ::std::string::String>,
    }
    impl ::std::default::Default for EvidenceReference {
        fn default() -> Self {
            Self {
                index: Err("no value supplied for index".to_string()),
                kind: Err("no value supplied for kind".to_string()),
            }
        }
    }
    impl EvidenceReference {
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
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EvidenceReferenceKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EvidenceReference> for super::EvidenceReference {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EvidenceReference,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                index: value.index?,
                kind: value.kind?,
            })
        }
    }
    impl ::std::convert::From<super::EvidenceReference> for EvidenceReference {
        fn from(value: super::EvidenceReference) -> Self {
            Self {
                index: Ok(value.index),
                kind: Ok(value.kind),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Factor {
        available: ::std::result::Result<bool, ::std::string::String>,
        evidence:
            ::std::result::Result<::std::vec::Vec<super::EvidenceReference>, ::std::string::String>,
        name: ::std::result::Result<super::FactorName, ::std::string::String>,
        unavailable_reason: ::std::result::Result<
            ::std::option::Option<super::FactorUnavailableReason>,
            ::std::string::String,
        >,
        value: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        weight: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for Factor {
        fn default() -> Self {
            Self {
                available: Err("no value supplied for available".to_string()),
                evidence: Ok(Default::default()),
                name: Err("no value supplied for name".to_string()),
                unavailable_reason: Ok(Default::default()),
                value: Ok(Default::default()),
                weight: Ok(Default::default()),
            }
        }
    }
    impl Factor {
        pub fn available<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.available = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for available: {e}"));
            self
        }
        pub fn evidence<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::EvidenceReference>>,
            T::Error: ::std::fmt::Display,
        {
            self.evidence = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for evidence: {e}"));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::FactorName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn unavailable_reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::FactorUnavailableReason>>,
            T::Error: ::std::fmt::Display,
        {
            self.unavailable_reason = value.try_into().map_err(|e| {
                format!("error converting supplied value for unavailable_reason: {e}")
            });
            self
        }
        pub fn value<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.value = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for value: {e}"));
            self
        }
        pub fn weight<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.weight = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for weight: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Factor> for super::Factor {
        type Error = super::error::ConversionError;
        fn try_from(value: Factor) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                available: value.available?,
                evidence: value.evidence?,
                name: value.name?,
                unavailable_reason: value.unavailable_reason?,
                value: value.value?,
                weight: value.weight?,
            })
        }
    }
    impl ::std::convert::From<super::Factor> for Factor {
        fn from(value: super::Factor) -> Self {
            Self {
                available: Ok(value.available),
                evidence: Ok(value.evidence),
                name: Ok(value.name),
                unavailable_reason: Ok(value.unavailable_reason),
                value: Ok(value.value),
                weight: Ok(value.weight),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct FilteredCandidate {
        candidate_id: ::std::result::Result<super::CandidateId, ::std::string::String>,
        detail: ::std::result::Result<
            ::std::option::Option<super::FilteredCandidateDetail>,
            ::std::string::String,
        >,
        reason: ::std::result::Result<super::FilteredCandidateReason, ::std::string::String>,
    }
    impl ::std::default::Default for FilteredCandidate {
        fn default() -> Self {
            Self {
                candidate_id: Err("no value supplied for candidate_id".to_string()),
                detail: Ok(Default::default()),
                reason: Err("no value supplied for reason".to_string()),
            }
        }
    }
    impl FilteredCandidate {
        pub fn candidate_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CandidateId>,
            T::Error: ::std::fmt::Display,
        {
            self.candidate_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidate_id: {e}"));
            self
        }
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::FilteredCandidateDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::FilteredCandidateReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<FilteredCandidate> for super::FilteredCandidate {
        type Error = super::error::ConversionError;
        fn try_from(
            value: FilteredCandidate,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidate_id: value.candidate_id?,
                detail: value.detail?,
                reason: value.reason?,
            })
        }
    }
    impl ::std::convert::From<super::FilteredCandidate> for FilteredCandidate {
        fn from(value: super::FilteredCandidate) -> Self {
            Self {
                candidate_id: Ok(value.candidate_id),
                detail: Ok(value.detail),
                reason: Ok(value.reason),
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
    pub struct Penalty {
        detail: ::std::result::Result<
            ::std::option::Option<super::PenaltyDetail>,
            ::std::string::String,
        >,
        reason: ::std::result::Result<super::PenaltyReason, ::std::string::String>,
        value: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Penalty {
        fn default() -> Self {
            Self {
                detail: Ok(Default::default()),
                reason: Err("no value supplied for reason".to_string()),
                value: Err("no value supplied for value".to_string()),
            }
        }
    }
    impl Penalty {
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::PenaltyDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PenaltyReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
        pub fn value<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.value = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for value: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Penalty> for super::Penalty {
        type Error = super::error::ConversionError;
        fn try_from(value: Penalty) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                detail: value.detail?,
                reason: value.reason?,
                value: value.value?,
            })
        }
    }
    impl ::std::convert::From<super::Penalty> for Penalty {
        fn from(value: super::Penalty) -> Self {
            Self {
                detail: Ok(value.detail),
                reason: Ok(value.reason),
                value: Ok(value.value),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Producer {
        implementation: ::std::result::Result<super::ProducerImplementation, ::std::string::String>,
        stage: ::std::result::Result<super::ProducerStage, ::std::string::String>,
    }
    impl ::std::default::Default for Producer {
        fn default() -> Self {
            Self {
                implementation: Err("no value supplied for implementation".to_string()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl Producer {
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
                implementation: value.implementation?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::Producer> for Producer {
        fn from(value: super::Producer) -> Self {
            Self {
                implementation: Ok(value.implementation),
                stage: Ok(value.stage),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Ranked {
        boundary: ::std::result::Result<super::Boundary, ::std::string::String>,
        candidate_id: ::std::result::Result<super::CandidateId, ::std::string::String>,
        cluster_id: ::std::result::Result<super::RankedClusterId, ::std::string::String>,
        display_score: ::std::result::Result<i64, ::std::string::String>,
        factors: ::std::result::Result<::std::vec::Vec<super::Factor>, ::std::string::String>,
        penalties: ::std::result::Result<::std::vec::Vec<super::Penalty>, ::std::string::String>,
        rank: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        score: ::std::result::Result<f64, ::std::string::String>,
        uncertainty: ::std::result::Result<super::Uncertainty, ::std::string::String>,
    }
    impl ::std::default::Default for Ranked {
        fn default() -> Self {
            Self {
                boundary: Err("no value supplied for boundary".to_string()),
                candidate_id: Err("no value supplied for candidate_id".to_string()),
                cluster_id: Err("no value supplied for cluster_id".to_string()),
                display_score: Err("no value supplied for display_score".to_string()),
                factors: Err("no value supplied for factors".to_string()),
                penalties: Err("no value supplied for penalties".to_string()),
                rank: Err("no value supplied for rank".to_string()),
                score: Err("no value supplied for score".to_string()),
                uncertainty: Err("no value supplied for uncertainty".to_string()),
            }
        }
    }
    impl Ranked {
        pub fn boundary<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Boundary>,
            T::Error: ::std::fmt::Display,
        {
            self.boundary = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for boundary: {e}"));
            self
        }
        pub fn candidate_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CandidateId>,
            T::Error: ::std::fmt::Display,
        {
            self.candidate_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidate_id: {e}"));
            self
        }
        pub fn cluster_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankedClusterId>,
            T::Error: ::std::fmt::Display,
        {
            self.cluster_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cluster_id: {e}"));
            self
        }
        pub fn display_score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.display_score = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for display_score: {e}"));
            self
        }
        pub fn factors<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Factor>>,
            T::Error: ::std::fmt::Display,
        {
            self.factors = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for factors: {e}"));
            self
        }
        pub fn penalties<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Penalty>>,
            T::Error: ::std::fmt::Display,
        {
            self.penalties = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for penalties: {e}"));
            self
        }
        pub fn rank<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.rank = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rank: {e}"));
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
        pub fn uncertainty<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uncertainty>,
            T::Error: ::std::fmt::Display,
        {
            self.uncertainty = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for uncertainty: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Ranked> for super::Ranked {
        type Error = super::error::ConversionError;
        fn try_from(value: Ranked) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                boundary: value.boundary?,
                candidate_id: value.candidate_id?,
                cluster_id: value.cluster_id?,
                display_score: value.display_score?,
                factors: value.factors?,
                penalties: value.penalties?,
                rank: value.rank?,
                score: value.score?,
                uncertainty: value.uncertainty?,
            })
        }
    }
    impl ::std::convert::From<super::Ranked> for Ranked {
        fn from(value: super::Ranked) -> Self {
            Self {
                boundary: Ok(value.boundary),
                candidate_id: Ok(value.candidate_id),
                cluster_id: Ok(value.cluster_id),
                display_score: Ok(value.display_score),
                factors: Ok(value.factors),
                penalties: Ok(value.penalties),
                rank: Ok(value.rank),
                score: Ok(value.score),
                uncertainty: Ok(value.uncertainty),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RankingSet {
        cohort: ::std::result::Result<::std::vec::Vec<super::Ranked>, ::std::string::String>,
        filtered:
            ::std::result::Result<::std::vec::Vec<super::FilteredCandidate>, ::std::string::String>,
        inputs: ::std::result::Result<super::RankingSetInputs, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        requested: ::std::result::Result<super::RankingSetRequested, ::std::string::String>,
        rubric: ::std::result::Result<super::RankingSetRubric, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        selected: ::std::result::Result<
            ::std::vec::Vec<super::RankingSetSelectedItem>,
            ::std::string::String,
        >,
        shortfall:
            ::std::result::Result<::std::vec::Vec<super::ShortfallReason>, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for RankingSet {
        fn default() -> Self {
            Self {
                cohort: Err("no value supplied for cohort".to_string()),
                filtered: Ok(Default::default()),
                inputs: Err("no value supplied for inputs".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                requested: Err("no value supplied for requested".to_string()),
                rubric: Err("no value supplied for rubric".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                selected: Err("no value supplied for selected".to_string()),
                shortfall: Err("no value supplied for shortfall".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl RankingSet {
        pub fn cohort<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Ranked>>,
            T::Error: ::std::fmt::Display,
        {
            self.cohort = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cohort: {e}"));
            self
        }
        pub fn filtered<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::FilteredCandidate>>,
            T::Error: ::std::fmt::Display,
        {
            self.filtered = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for filtered: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankingSetInputs>,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
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
        pub fn requested<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankingSetRequested>,
            T::Error: ::std::fmt::Display,
        {
            self.requested = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for requested: {e}"));
            self
        }
        pub fn rubric<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankingSetRubric>,
            T::Error: ::std::fmt::Display,
        {
            self.rubric = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rubric: {e}"));
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
        pub fn selected<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::RankingSetSelectedItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.selected = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for selected: {e}"));
            self
        }
        pub fn shortfall<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ShortfallReason>>,
            T::Error: ::std::fmt::Display,
        {
            self.shortfall = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shortfall: {e}"));
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
    impl ::std::convert::TryFrom<RankingSet> for super::RankingSet {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RankingSet,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cohort: value.cohort?,
                filtered: value.filtered?,
                inputs: value.inputs?,
                producer: value.producer?,
                requested: value.requested?,
                rubric: value.rubric?,
                schema_version: value.schema_version?,
                selected: value.selected?,
                shortfall: value.shortfall?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::RankingSet> for RankingSet {
        fn from(value: super::RankingSet) -> Self {
            Self {
                cohort: Ok(value.cohort),
                filtered: Ok(value.filtered),
                inputs: Ok(value.inputs),
                producer: Ok(value.producer),
                requested: Ok(value.requested),
                rubric: Ok(value.rubric),
                schema_version: Ok(value.schema_version),
                selected: Ok(value.selected),
                shortfall: Ok(value.shortfall),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RankingSetInputs {
        candidates_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        index_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        transcript_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for RankingSetInputs {
        fn default() -> Self {
            Self {
                candidates_artifact_id: Err(
                    "no value supplied for candidates_artifact_id".to_string()
                ),
                index_artifact_id: Err("no value supplied for index_artifact_id".to_string()),
                transcript_artifact_id: Err(
                    "no value supplied for transcript_artifact_id".to_string()
                ),
            }
        }
    }
    impl RankingSetInputs {
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
        pub fn index_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.index_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for index_artifact_id: {e}"));
            self
        }
        pub fn transcript_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.transcript_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for transcript_artifact_id: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<RankingSetInputs> for super::RankingSetInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RankingSetInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidates_artifact_id: value.candidates_artifact_id?,
                index_artifact_id: value.index_artifact_id?,
                transcript_artifact_id: value.transcript_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::RankingSetInputs> for RankingSetInputs {
        fn from(value: super::RankingSetInputs) -> Self {
            Self {
                candidates_artifact_id: Ok(value.candidates_artifact_id),
                index_artifact_id: Ok(value.index_artifact_id),
                transcript_artifact_id: Ok(value.transcript_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RankingSetRequested {
        count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        diversity: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for RankingSetRequested {
        fn default() -> Self {
            Self {
                count: Err("no value supplied for count".to_string()),
                diversity: Err("no value supplied for diversity".to_string()),
            }
        }
    }
    impl RankingSetRequested {
        pub fn count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for count: {e}"));
            self
        }
        pub fn diversity<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.diversity = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for diversity: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RankingSetRequested> for super::RankingSetRequested {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RankingSetRequested,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                count: value.count?,
                diversity: value.diversity?,
            })
        }
    }
    impl ::std::convert::From<super::RankingSetRequested> for RankingSetRequested {
        fn from(value: super::RankingSetRequested) -> Self {
            Self {
                count: Ok(value.count),
                diversity: Ok(value.diversity),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RankingSetRubric {
        boundary: ::std::result::Result<super::RankingSetRubricBoundary, ::std::string::String>,
        scorer: ::std::result::Result<super::RankingSetRubricScorer, ::std::string::String>,
        selector: ::std::result::Result<super::RankingSetRubricSelector, ::std::string::String>,
    }
    impl ::std::default::Default for RankingSetRubric {
        fn default() -> Self {
            Self {
                boundary: Err("no value supplied for boundary".to_string()),
                scorer: Err("no value supplied for scorer".to_string()),
                selector: Err("no value supplied for selector".to_string()),
            }
        }
    }
    impl RankingSetRubric {
        pub fn boundary<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankingSetRubricBoundary>,
            T::Error: ::std::fmt::Display,
        {
            self.boundary = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for boundary: {e}"));
            self
        }
        pub fn scorer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankingSetRubricScorer>,
            T::Error: ::std::fmt::Display,
        {
            self.scorer = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for scorer: {e}"));
            self
        }
        pub fn selector<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RankingSetRubricSelector>,
            T::Error: ::std::fmt::Display,
        {
            self.selector = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for selector: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RankingSetRubric> for super::RankingSetRubric {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RankingSetRubric,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                boundary: value.boundary?,
                scorer: value.scorer?,
                selector: value.selector?,
            })
        }
    }
    impl ::std::convert::From<super::RankingSetRubric> for RankingSetRubric {
        fn from(value: super::RankingSetRubric) -> Self {
            Self {
                boundary: Ok(value.boundary),
                scorer: Ok(value.scorer),
                selector: Ok(value.selector),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ShortfallReason {
        count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        detail: ::std::result::Result<
            ::std::option::Option<super::ShortfallReasonDetail>,
            ::std::string::String,
        >,
        reason: ::std::result::Result<super::ShortfallReasonReason, ::std::string::String>,
    }
    impl ::std::default::Default for ShortfallReason {
        fn default() -> Self {
            Self {
                count: Err("no value supplied for count".to_string()),
                detail: Ok(Default::default()),
                reason: Err("no value supplied for reason".to_string()),
            }
        }
    }
    impl ShortfallReason {
        pub fn count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for count: {e}"));
            self
        }
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ShortfallReasonDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ShortfallReasonReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ShortfallReason> for super::ShortfallReason {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ShortfallReason,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                count: value.count?,
                detail: value.detail?,
                reason: value.reason?,
            })
        }
    }
    impl ::std::convert::From<super::ShortfallReason> for ShortfallReason {
        fn from(value: super::ShortfallReason) -> Self {
            Self {
                count: Ok(value.count),
                detail: Ok(value.detail),
                reason: Ok(value.reason),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Uncertainty {
        band: ::std::result::Result<super::UncertaintyBand, ::std::string::String>,
        value: ::std::result::Result<f64, ::std::string::String>,
        warnings: ::std::result::Result<
            ::std::vec::Vec<super::UncertaintyWarningsItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Uncertainty {
        fn default() -> Self {
            Self {
                band: Err("no value supplied for band".to_string()),
                value: Err("no value supplied for value".to_string()),
                warnings: Ok(Default::default()),
            }
        }
    }
    impl Uncertainty {
        pub fn band<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UncertaintyBand>,
            T::Error: ::std::fmt::Display,
        {
            self.band = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for band: {e}"));
            self
        }
        pub fn value<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.value = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for value: {e}"));
            self
        }
        pub fn warnings<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::UncertaintyWarningsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.warnings = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for warnings: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Uncertainty> for super::Uncertainty {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Uncertainty,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                band: value.band?,
                value: value.value?,
                warnings: value.warnings?,
            })
        }
    }
    impl ::std::convert::From<super::Uncertainty> for Uncertainty {
        fn from(value: super::Uncertainty) -> Self {
            Self {
                band: Ok(value.band),
                value: Ok(value.value),
                warnings: Ok(value.warnings),
            }
        }
    }
}
