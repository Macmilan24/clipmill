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
#[doc = "Every legal place this clip could start and end. Discovery keeps the whole lattice rather than choosing: boundary optimization searches it in the ranking stage, and a discovery that pre-chose would be making a decision with less information than the stage that has to live with it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Every legal place this clip could start and end. Discovery keeps the whole lattice rather than choosing: boundary optimization searches it in the ranking stage, and a discovery that pre-chose would be making a decision with less information than the stage that has to live with it.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"ends\","]
#[doc = "    \"phi_rejects\","]
#[doc = "    \"starts\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"ends\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"integer\","]
#[doc = "        \"minimum\": 0.0"]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"phi_rejects\": {"]
#[doc = "      \"description\": \"What the legality predicate removed, counted by reason rather than enumerated. The pairs are the product of starts and ends, so listing every rejection would make the artifact quadratic in the recording's length to record something a consumer can recompute from the bounds above.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/phi_reject\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"starts\": {"]
#[doc = "      \"description\": \"Ascending, distinct, and already structurally legal: a tick that falls inside a word never appears here.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"integer\","]
#[doc = "        \"minimum\": 0.0"]
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
pub struct BoundaryLattice {
    pub ends: ::std::vec::Vec<u64>,
    #[doc = "What the legality predicate removed, counted by reason rather than enumerated. The pairs are the product of starts and ends, so listing every rejection would make the artifact quadratic in the recording's length to record something a consumer can recompute from the bounds above."]
    pub phi_rejects: ::std::vec::Vec<PhiReject>,
    #[doc = "Ascending, distinct, and already structurally legal: a tick that falls inside a word never appears here."]
    pub starts: ::std::vec::Vec<u64>,
}
impl BoundaryLattice {
    pub fn builder() -> builder::BoundaryLattice {
        Default::default()
    }
}
#[doc = "`Candidate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"boundary_lattice\","]
#[doc = "    \"cluster_id\","]
#[doc = "    \"evidence\","]
#[doc = "    \"exclusions\","]
#[doc = "    \"id\","]
#[doc = "    \"intervals\","]
#[doc = "    \"layout_requirements\","]
#[doc = "    \"prelim_score\","]
#[doc = "    \"proposer\","]
#[doc = "    \"roles\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"boundary_lattice\": {"]
#[doc = "      \"$ref\": \"#/$defs/boundary_lattice\""]
#[doc = "    },"]
#[doc = "    \"cluster_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/cluster_id\""]
#[doc = "    },"]
#[doc = "    \"evidence\": {"]
#[doc = "      \"description\": \"The index units this nomination rests on, ordered and distinct. Never empty: a candidate nobody can explain is a candidate ranking cannot defend.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/evidence_reference\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"exclusions\": {"]
#[doc = "      \"description\": \"Reasons this candidate must not be published even if it ranks well.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/exclusion\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"description\": \"Derived from the proposer and the intervals rather than from a counter, so two runs over the same recording name the same candidate and a reordering cannot rename one.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "    },"]
#[doc = "    \"intervals\": {"]
#[doc = "      \"description\": \"One for a contiguous clip; more than one for a compilation. Ordered and non-overlapping.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/interval\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"layout_requirements\": {"]
#[doc = "      \"description\": \"Layout capabilities a renderer must have for this candidate. Empty at this phase, and empty on purpose: the fit layout is always legal, so no nomination can be layout-infeasible yet. The field exists now so that adding speaker-fill or screen-share layouts later does not change the contract.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"prelim_score\": {"]
#[doc = "      \"description\": \"The proposer's own confidence, comparable only within a proposer. Ranking computes the score that is comparable across them; this one exists so a cluster can pick a representative before ranking runs.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"proposer\": {"]
#[doc = "      \"$ref\": \"#/$defs/proposer\""]
#[doc = "    },"]
#[doc = "    \"roles\": {"]
#[doc = "      \"description\": \"Which evidence opens the clip and which pays it off. Both are optional because not every nomination has both, and guessing one would put a narrative claim into a document that made no narrative measurement.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"properties\": {"]
#[doc = "        \"hook\": {"]
#[doc = "          \"$ref\": \"#/$defs/evidence_reference\""]
#[doc = "        },"]
#[doc = "        \"payoff\": {"]
#[doc = "          \"$ref\": \"#/$defs/evidence_reference\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub boundary_lattice: BoundaryLattice,
    pub cluster_id: ClusterId,
    #[doc = "The index units this nomination rests on, ordered and distinct. Never empty: a candidate nobody can explain is a candidate ranking cannot defend."]
    pub evidence: ::std::vec::Vec<EvidenceReference>,
    #[doc = "Reasons this candidate must not be published even if it ranks well."]
    pub exclusions: ::std::vec::Vec<Exclusion>,
    #[doc = "Derived from the proposer and the intervals rather than from a counter, so two runs over the same recording name the same candidate and a reordering cannot rename one."]
    pub id: CandidateId,
    #[doc = "One for a contiguous clip; more than one for a compilation. Ordered and non-overlapping."]
    pub intervals: ::std::vec::Vec<Interval>,
    #[doc = "Layout capabilities a renderer must have for this candidate. Empty at this phase, and empty on purpose: the fit layout is always legal, so no nomination can be layout-infeasible yet. The field exists now so that adding speaker-fill or screen-share layouts later does not change the contract."]
    pub layout_requirements: ::std::vec::Vec<CandidateLayoutRequirementsItem>,
    #[doc = "The proposer's own confidence, comparable only within a proposer. Ranking computes the score that is comparable across them; this one exists so a cluster can pick a representative before ranking runs."]
    pub prelim_score: f64,
    pub proposer: Proposer,
    pub roles: CandidateRoles,
}
impl Candidate {
    pub fn builder() -> builder::Candidate {
        Default::default()
    }
}
#[doc = "Derived from the proposer and the intervals rather than from a counter, so two runs over the same recording name the same candidate and a reordering cannot rename one."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Derived from the proposer and the intervals rather than from a counter, so two runs over the same recording name the same candidate and a reordering cannot rename one.\","]
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
#[doc = "`CandidateLayoutRequirementsItem`"]
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
pub struct CandidateLayoutRequirementsItem(::std::string::String);
impl ::std::ops::Deref for CandidateLayoutRequirementsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CandidateLayoutRequirementsItem> for ::std::string::String {
    fn from(value: CandidateLayoutRequirementsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CandidateLayoutRequirementsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CandidateLayoutRequirementsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CandidateLayoutRequirementsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CandidateLayoutRequirementsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CandidateLayoutRequirementsItem {
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
#[doc = "Which evidence opens the clip and which pays it off. Both are optional because not every nomination has both, and guessing one would put a narrative claim into a document that made no narrative measurement."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Which evidence opens the clip and which pays it off. Both are optional because not every nomination has both, and guessing one would put a narrative claim into a document that made no narrative measurement.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"hook\": {"]
#[doc = "      \"$ref\": \"#/$defs/evidence_reference\""]
#[doc = "    },"]
#[doc = "    \"payoff\": {"]
#[doc = "      \"$ref\": \"#/$defs/evidence_reference\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CandidateRoles {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub hook: ::std::option::Option<EvidenceReference>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub payoff: ::std::option::Option<EvidenceReference>,
}
impl ::std::default::Default for CandidateRoles {
    fn default() -> Self {
        Self {
            hook: Default::default(),
            payoff: Default::default(),
        }
    }
}
impl CandidateRoles {
    pub fn builder() -> builder::CandidateRoles {
        Default::default()
    }
}
#[doc = "`Cluster`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"members\","]
#[doc = "    \"representative\","]
#[doc = "    \"similarity\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"$ref\": \"#/$defs/cluster_id\""]
#[doc = "    },"]
#[doc = "    \"members\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"representative\": {"]
#[doc = "      \"description\": \"The member with the highest preliminary score, ties broken by candidate id so the choice does not depend on iteration order.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "    },"]
#[doc = "    \"similarity\": {"]
#[doc = "      \"description\": \"The weakest pairwise similarity inside the cluster, so a consumer can see how loose the grouping is. One for a cluster of one, which duplicates nothing.\","]
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
pub struct Cluster {
    pub id: ClusterId,
    pub members: ::std::vec::Vec<ClusterMembersItem>,
    #[doc = "The member with the highest preliminary score, ties broken by candidate id so the choice does not depend on iteration order."]
    pub representative: ClusterRepresentative,
    #[doc = "The weakest pairwise similarity inside the cluster, so a consumer can see how loose the grouping is. One for a cluster of one, which duplicates nothing."]
    pub similarity: f64,
}
impl Cluster {
    pub fn builder() -> builder::Cluster {
        Default::default()
    }
}
#[doc = "`ClusterId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^cl_[0-9a-f]{16}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClusterId(::std::string::String);
impl ::std::ops::Deref for ClusterId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClusterId> for ::std::string::String {
    fn from(value: ClusterId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClusterId {
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
impl ::std::convert::TryFrom<&str> for ClusterId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ClusterId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClusterId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClusterId {
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
#[doc = "`ClusterMembersItem`"]
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
pub struct ClusterMembersItem(::std::string::String);
impl ::std::ops::Deref for ClusterMembersItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClusterMembersItem> for ::std::string::String {
    fn from(value: ClusterMembersItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClusterMembersItem {
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
impl ::std::convert::TryFrom<&str> for ClusterMembersItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ClusterMembersItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClusterMembersItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClusterMembersItem {
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
#[doc = "The member with the highest preliminary score, ties broken by candidate id so the choice does not depend on iteration order."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The member with the highest preliminary score, ties broken by candidate id so the choice does not depend on iteration order.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^cand_[0-9a-f]{16}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClusterRepresentative(::std::string::String);
impl ::std::ops::Deref for ClusterRepresentative {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClusterRepresentative> for ::std::string::String {
    fn from(value: ClusterRepresentative) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClusterRepresentative {
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
impl ::std::convert::TryFrom<&str> for ClusterRepresentative {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ClusterRepresentative {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClusterRepresentative {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClusterRepresentative {
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
#[doc = "The index's analyzed range, echoed. A recording nobody indexed produces no candidates and says `analyzed` false — not an empty list that reads like a recording with nothing worth clipping in it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The index's analyzed range, echoed. A recording nobody indexed produces no candidates and says `analyzed` false — not an empty list that reads like a recording with nothing worth clipping in it.\","]
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
    pub start_ticks: u64,
}
impl Coverage {
    pub fn builder() -> builder::Coverage {
        Default::default()
    }
}
#[doc = "Spans of a recording worth considering as clips, nominated by a mesh of independent proposers (book ch. 15). Discovery does not decide which clip is good — ranking does — and it does not decide where a clip starts, because boundary optimization searches the lattice each candidate carries. What discovery guarantees is that the lattice is legal and wide, that every nomination can be walked back to the evidence behind it, and that near-duplicates are grouped rather than silently dropped. A proposer names the approximation it is making in its rubric string, because at this phase every one of them is an approximation and a consumer must be able to tell which."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.discovery.candidates.v1.json\","]
#[doc = "  \"title\": \"DiscoveryCandidates\","]
#[doc = "  \"description\": \"Spans of a recording worth considering as clips, nominated by a mesh of independent proposers (book ch. 15). Discovery does not decide which clip is good — ranking does — and it does not decide where a clip starts, because boundary optimization searches the lattice each candidate carries. What discovery guarantees is that the lattice is legal and wide, that every nomination can be walked back to the evidence behind it, and that near-duplicates are grouped rather than silently dropped. A proposer names the approximation it is making in its rubric string, because at this phase every one of them is an approximation and a consumer must be able to tell which.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidates\","]
#[doc = "    \"clusters\","]
#[doc = "    \"coverage\","]
#[doc = "    \"duration_target\","]
#[doc = "    \"inputs\","]
#[doc = "    \"producer\","]
#[doc = "    \"proposers\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidates\": {"]
#[doc = "      \"description\": \"Ordered by first interval start, then by candidate id, so the document does not depend on which proposer happened to run first.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/candidate\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"clusters\": {"]
#[doc = "      \"description\": \"Near-duplicate groupings. Every candidate belongs to exactly one cluster, including the ones that duplicate nothing. Ranking sends a cluster's representative forward and can still offer the alternatives, so a diversity decision is shown rather than silent.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/cluster\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"duration_target\": {"]
#[doc = "      \"description\": \"The platform range candidates were expanded against, as ticks. Part of the artifact key: asking for a different length is a different search, not a filter over this one.\","]
#[doc = "      \"$ref\": \"#/$defs/duration_range\""]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"description\": \"What was read. The index is the authority for every evidence reference below and the transcript for every word boundary; the loudness envelope is optional because a source with no audio has none, and prosody then contributes nothing rather than contributing a default.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"index_artifact_id\","]
#[doc = "        \"transcript_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"index_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"loudness_artifact_id\": {"]
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
#[doc = "    \"proposers\": {"]
#[doc = "      \"description\": \"One entry per proposer that ran, whether or not it found anything. A proposer that nominated nothing is a fact about the recording; a proposer missing from this list is a fact about the build, and the two must not look alike.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/proposer_run\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.discovery.candidates.v1\""]
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
pub struct DiscoveryCandidates {
    #[doc = "Ordered by first interval start, then by candidate id, so the document does not depend on which proposer happened to run first."]
    pub candidates: ::std::vec::Vec<Candidate>,
    #[doc = "Near-duplicate groupings. Every candidate belongs to exactly one cluster, including the ones that duplicate nothing. Ranking sends a cluster's representative forward and can still offer the alternatives, so a diversity decision is shown rather than silent."]
    pub clusters: ::std::vec::Vec<Cluster>,
    pub coverage: Coverage,
    #[doc = "The platform range candidates were expanded against, as ticks. Part of the artifact key: asking for a different length is a different search, not a filter over this one."]
    pub duration_target: DurationRange,
    pub inputs: DiscoveryCandidatesInputs,
    pub producer: Producer,
    #[doc = "One entry per proposer that ran, whether or not it found anything. A proposer that nominated nothing is a fact about the recording; a proposer missing from this list is a fact about the build, and the two must not look alike."]
    pub proposers: ::std::vec::Vec<ProposerRun>,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
}
impl DiscoveryCandidates {
    pub fn builder() -> builder::DiscoveryCandidates {
        Default::default()
    }
}
#[doc = "What was read. The index is the authority for every evidence reference below and the transcript for every word boundary; the loudness envelope is optional because a source with no audio has none, and prosody then contributes nothing rather than contributing a default."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was read. The index is the authority for every evidence reference below and the transcript for every word boundary; the loudness envelope is optional because a source with no audio has none, and prosody then contributes nothing rather than contributing a default.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"index_artifact_id\","]
#[doc = "    \"transcript_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"index_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"loudness_artifact_id\": {"]
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
pub struct DiscoveryCandidatesInputs {
    pub index_artifact_id: Sha256,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub loudness_artifact_id: ::std::option::Option<Sha256>,
    pub transcript_artifact_id: Sha256,
}
impl DiscoveryCandidatesInputs {
    pub fn builder() -> builder::DiscoveryCandidatesInputs {
        Default::default()
    }
}
#[doc = "`DurationRange`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"max_ticks\","]
#[doc = "    \"min_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"max_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"min_ticks\": {"]
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
pub struct DurationRange {
    pub max_ticks: ::std::num::NonZeroU64,
    pub min_ticks: ::std::num::NonZeroU64,
}
impl DurationRange {
    pub fn builder() -> builder::DurationRange {
        Default::default()
    }
}
#[doc = "A unit of the evidence index, named by kind and position rather than by an opaque id. The index resolves each to a transcript word range, so a claim here is walkable to the words somebody measured (Rule 14.1). Opaque evidence ids belong with the Phase 2 interval tables; inventing them now would mean a registry nothing yet reads."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A unit of the evidence index, named by kind and position rather than by an opaque id. The index resolves each to a transcript word range, so a claim here is walkable to the words somebody measured (Rule 14.1). Opaque evidence ids belong with the Phase 2 interval tables; inventing them now would mean a registry nothing yet reads.\","]
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
#[doc = "`Exclusion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"reason\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"invalid_region\","]
#[doc = "        \"below_coverage\","]
#[doc = "        \"rights_excluded\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Exclusion {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub detail: ::std::option::Option<ExclusionDetail>,
    pub reason: ExclusionReason,
}
impl Exclusion {
    pub fn builder() -> builder::Exclusion {
        Default::default()
    }
}
#[doc = "`ExclusionDetail`"]
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
pub struct ExclusionDetail(::std::string::String);
impl ::std::ops::Deref for ExclusionDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ExclusionDetail> for ::std::string::String {
    fn from(value: ExclusionDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ExclusionDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ExclusionDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ExclusionDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExclusionDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ExclusionDetail {
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
#[doc = "`ExclusionReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"invalid_region\","]
#[doc = "    \"below_coverage\","]
#[doc = "    \"rights_excluded\""]
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
pub enum ExclusionReason {
    #[serde(rename = "invalid_region")]
    InvalidRegion,
    #[serde(rename = "below_coverage")]
    BelowCoverage,
    #[serde(rename = "rights_excluded")]
    RightsExcluded,
}
impl ::std::fmt::Display for ExclusionReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::InvalidRegion => f.write_str("invalid_region"),
            Self::BelowCoverage => f.write_str("below_coverage"),
            Self::RightsExcluded => f.write_str("rights_excluded"),
        }
    }
}
impl ::std::str::FromStr for ExclusionReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "invalid_region" => Ok(Self::InvalidRegion),
            "below_coverage" => Ok(Self::BelowCoverage),
            "rights_excluded" => Ok(Self::RightsExcluded),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ExclusionReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ExclusionReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExclusionReason {
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
#[doc = "`PhiReject`"]
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
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"description\": \"'mid_word' is structural and removes the point itself. 'too_short' and 'too_long' remove a pair against the requested duration range. 'outside_coverage' removes a pair that leaves the analyzed span. The open-loop, identity, rights, and layout terms the design names are absent rather than always-passing, because a term that is recorded as never firing reads as a term that was checked.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"mid_word\","]
#[doc = "        \"too_short\","]
#[doc = "        \"too_long\","]
#[doc = "        \"outside_coverage\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PhiReject {
    pub count: ::std::num::NonZeroU64,
    #[doc = "'mid_word' is structural and removes the point itself. 'too_short' and 'too_long' remove a pair against the requested duration range. 'outside_coverage' removes a pair that leaves the analyzed span. The open-loop, identity, rights, and layout terms the design names are absent rather than always-passing, because a term that is recorded as never firing reads as a term that was checked."]
    pub reason: PhiRejectReason,
}
impl PhiReject {
    pub fn builder() -> builder::PhiReject {
        Default::default()
    }
}
#[doc = "'mid_word' is structural and removes the point itself. 'too_short' and 'too_long' remove a pair against the requested duration range. 'outside_coverage' removes a pair that leaves the analyzed span. The open-loop, identity, rights, and layout terms the design names are absent rather than always-passing, because a term that is recorded as never firing reads as a term that was checked."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"'mid_word' is structural and removes the point itself. 'too_short' and 'too_long' remove a pair against the requested duration range. 'outside_coverage' removes a pair that leaves the analyzed span. The open-loop, identity, rights, and layout terms the design names are absent rather than always-passing, because a term that is recorded as never firing reads as a term that was checked.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"mid_word\","]
#[doc = "    \"too_short\","]
#[doc = "    \"too_long\","]
#[doc = "    \"outside_coverage\""]
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
pub enum PhiRejectReason {
    #[serde(rename = "mid_word")]
    MidWord,
    #[serde(rename = "too_short")]
    TooShort,
    #[serde(rename = "too_long")]
    TooLong,
    #[serde(rename = "outside_coverage")]
    OutsideCoverage,
}
impl ::std::fmt::Display for PhiRejectReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::MidWord => f.write_str("mid_word"),
            Self::TooShort => f.write_str("too_short"),
            Self::TooLong => f.write_str("too_long"),
            Self::OutsideCoverage => f.write_str("outside_coverage"),
        }
    }
}
impl ::std::str::FromStr for PhiRejectReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "mid_word" => Ok(Self::MidWord),
            "too_short" => Ok(Self::TooShort),
            "too_long" => Ok(Self::TooLong),
            "outside_coverage" => Ok(Self::OutsideCoverage),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for PhiRejectReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PhiRejectReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PhiRejectReason {
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
#[doc = "`Proposer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"name\","]
#[doc = "    \"rubric\","]
#[doc = "    \"version\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"name\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"rubric\": {"]
#[doc = "      \"description\": \"What this proposer actually looked at, named so the approximation is legible: 'topic-span-open-close.v1' is not a narrative model, and calling it one in a field nobody reads is how a downstream stage comes to trust it as one. Changing the rubric changes the artifact key.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"version\": {"]
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
pub struct Proposer {
    pub name: ProposerName,
    #[doc = "What this proposer actually looked at, named so the approximation is legible: 'topic-span-open-close.v1' is not a narrative model, and calling it one in a field nobody reads is how a downstream stage comes to trust it as one. Changing the rubric changes the artifact key."]
    pub rubric: ProposerRubric,
    pub version: ProposerVersion,
}
impl Proposer {
    pub fn builder() -> builder::Proposer {
        Default::default()
    }
}
#[doc = "`ProposerName`"]
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
pub struct ProposerName(::std::string::String);
impl ::std::ops::Deref for ProposerName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposerName> for ::std::string::String {
    fn from(value: ProposerName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposerName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposerName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposerName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposerName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposerName {
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
#[doc = "What this proposer actually looked at, named so the approximation is legible: 'topic-span-open-close.v1' is not a narrative model, and calling it one in a field nobody reads is how a downstream stage comes to trust it as one. Changing the rubric changes the artifact key."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What this proposer actually looked at, named so the approximation is legible: 'topic-span-open-close.v1' is not a narrative model, and calling it one in a field nobody reads is how a downstream stage comes to trust it as one. Changing the rubric changes the artifact key.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProposerRubric(::std::string::String);
impl ::std::ops::Deref for ProposerRubric {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposerRubric> for ::std::string::String {
    fn from(value: ProposerRubric) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposerRubric {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposerRubric {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposerRubric {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposerRubric {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposerRubric {
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
#[doc = "`ProposerRun`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidates\","]
#[doc = "    \"proposer\","]
#[doc = "    \"seeds\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidates\": {"]
#[doc = "      \"description\": \"How many survived expansion and the legality test. Fewer than the seed count means the lattice had nothing legal to offer, which is a statement about the recording rather than about the proposer.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"floor_applied\": {"]
#[doc = "      \"description\": \"Present when the exploration floor kept nominations a score-only cut would have dropped. A transcript-heavy recording must not be able to starve a proposer with a different bias, and when that protection fires it is recorded rather than assumed.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"proposer\": {"]
#[doc = "      \"$ref\": \"#/$defs/proposer\""]
#[doc = "    },"]
#[doc = "    \"seeds\": {"]
#[doc = "      \"description\": \"Moments this proposer nominated before expansion.\","]
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
pub struct ProposerRun {
    #[doc = "How many survived expansion and the legality test. Fewer than the seed count means the lattice had nothing legal to offer, which is a statement about the recording rather than about the proposer."]
    pub candidates: u64,
    #[doc = "Present when the exploration floor kept nominations a score-only cut would have dropped. A transcript-heavy recording must not be able to starve a proposer with a different bias, and when that protection fires it is recorded rather than assumed."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub floor_applied: ::std::option::Option<::std::num::NonZeroU64>,
    pub proposer: Proposer,
    #[doc = "Moments this proposer nominated before expansion."]
    pub seeds: u64,
}
impl ProposerRun {
    pub fn builder() -> builder::ProposerRun {
        Default::default()
    }
}
#[doc = "`ProposerVersion`"]
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
pub struct ProposerVersion(::std::string::String);
impl ::std::ops::Deref for ProposerVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposerVersion> for ::std::string::String {
    fn from(value: ProposerVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposerVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposerVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposerVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposerVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposerVersion {
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
    pub struct BoundaryLattice {
        ends: ::std::result::Result<::std::vec::Vec<u64>, ::std::string::String>,
        phi_rejects:
            ::std::result::Result<::std::vec::Vec<super::PhiReject>, ::std::string::String>,
        starts: ::std::result::Result<::std::vec::Vec<u64>, ::std::string::String>,
    }
    impl ::std::default::Default for BoundaryLattice {
        fn default() -> Self {
            Self {
                ends: Err("no value supplied for ends".to_string()),
                phi_rejects: Err("no value supplied for phi_rejects".to_string()),
                starts: Err("no value supplied for starts".to_string()),
            }
        }
    }
    impl BoundaryLattice {
        pub fn ends<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.ends = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for ends: {e}"));
            self
        }
        pub fn phi_rejects<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::PhiReject>>,
            T::Error: ::std::fmt::Display,
        {
            self.phi_rejects = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for phi_rejects: {e}"));
            self
        }
        pub fn starts<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.starts = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for starts: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<BoundaryLattice> for super::BoundaryLattice {
        type Error = super::error::ConversionError;
        fn try_from(
            value: BoundaryLattice,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                ends: value.ends?,
                phi_rejects: value.phi_rejects?,
                starts: value.starts?,
            })
        }
    }
    impl ::std::convert::From<super::BoundaryLattice> for BoundaryLattice {
        fn from(value: super::BoundaryLattice) -> Self {
            Self {
                ends: Ok(value.ends),
                phi_rejects: Ok(value.phi_rejects),
                starts: Ok(value.starts),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Candidate {
        boundary_lattice: ::std::result::Result<super::BoundaryLattice, ::std::string::String>,
        cluster_id: ::std::result::Result<super::ClusterId, ::std::string::String>,
        evidence:
            ::std::result::Result<::std::vec::Vec<super::EvidenceReference>, ::std::string::String>,
        exclusions: ::std::result::Result<::std::vec::Vec<super::Exclusion>, ::std::string::String>,
        id: ::std::result::Result<super::CandidateId, ::std::string::String>,
        intervals: ::std::result::Result<::std::vec::Vec<super::Interval>, ::std::string::String>,
        layout_requirements: ::std::result::Result<
            ::std::vec::Vec<super::CandidateLayoutRequirementsItem>,
            ::std::string::String,
        >,
        prelim_score: ::std::result::Result<f64, ::std::string::String>,
        proposer: ::std::result::Result<super::Proposer, ::std::string::String>,
        roles: ::std::result::Result<super::CandidateRoles, ::std::string::String>,
    }
    impl ::std::default::Default for Candidate {
        fn default() -> Self {
            Self {
                boundary_lattice: Err("no value supplied for boundary_lattice".to_string()),
                cluster_id: Err("no value supplied for cluster_id".to_string()),
                evidence: Err("no value supplied for evidence".to_string()),
                exclusions: Err("no value supplied for exclusions".to_string()),
                id: Err("no value supplied for id".to_string()),
                intervals: Err("no value supplied for intervals".to_string()),
                layout_requirements: Err("no value supplied for layout_requirements".to_string()),
                prelim_score: Err("no value supplied for prelim_score".to_string()),
                proposer: Err("no value supplied for proposer".to_string()),
                roles: Err("no value supplied for roles".to_string()),
            }
        }
    }
    impl Candidate {
        pub fn boundary_lattice<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::BoundaryLattice>,
            T::Error: ::std::fmt::Display,
        {
            self.boundary_lattice = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for boundary_lattice: {e}"));
            self
        }
        pub fn cluster_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ClusterId>,
            T::Error: ::std::fmt::Display,
        {
            self.cluster_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cluster_id: {e}"));
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
        pub fn exclusions<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Exclusion>>,
            T::Error: ::std::fmt::Display,
        {
            self.exclusions = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for exclusions: {e}"));
            self
        }
        pub fn id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CandidateId>,
            T::Error: ::std::fmt::Display,
        {
            self.id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for id: {e}"));
            self
        }
        pub fn intervals<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Interval>>,
            T::Error: ::std::fmt::Display,
        {
            self.intervals = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for intervals: {e}"));
            self
        }
        pub fn layout_requirements<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CandidateLayoutRequirementsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.layout_requirements = value.try_into().map_err(|e| {
                format!("error converting supplied value for layout_requirements: {e}")
            });
            self
        }
        pub fn prelim_score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.prelim_score = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prelim_score: {e}"));
            self
        }
        pub fn proposer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Proposer>,
            T::Error: ::std::fmt::Display,
        {
            self.proposer = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for proposer: {e}"));
            self
        }
        pub fn roles<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CandidateRoles>,
            T::Error: ::std::fmt::Display,
        {
            self.roles = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for roles: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Candidate> for super::Candidate {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Candidate,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                boundary_lattice: value.boundary_lattice?,
                cluster_id: value.cluster_id?,
                evidence: value.evidence?,
                exclusions: value.exclusions?,
                id: value.id?,
                intervals: value.intervals?,
                layout_requirements: value.layout_requirements?,
                prelim_score: value.prelim_score?,
                proposer: value.proposer?,
                roles: value.roles?,
            })
        }
    }
    impl ::std::convert::From<super::Candidate> for Candidate {
        fn from(value: super::Candidate) -> Self {
            Self {
                boundary_lattice: Ok(value.boundary_lattice),
                cluster_id: Ok(value.cluster_id),
                evidence: Ok(value.evidence),
                exclusions: Ok(value.exclusions),
                id: Ok(value.id),
                intervals: Ok(value.intervals),
                layout_requirements: Ok(value.layout_requirements),
                prelim_score: Ok(value.prelim_score),
                proposer: Ok(value.proposer),
                roles: Ok(value.roles),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CandidateRoles {
        hook: ::std::result::Result<
            ::std::option::Option<super::EvidenceReference>,
            ::std::string::String,
        >,
        payoff: ::std::result::Result<
            ::std::option::Option<super::EvidenceReference>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for CandidateRoles {
        fn default() -> Self {
            Self {
                hook: Ok(Default::default()),
                payoff: Ok(Default::default()),
            }
        }
    }
    impl CandidateRoles {
        pub fn hook<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EvidenceReference>>,
            T::Error: ::std::fmt::Display,
        {
            self.hook = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hook: {e}"));
            self
        }
        pub fn payoff<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EvidenceReference>>,
            T::Error: ::std::fmt::Display,
        {
            self.payoff = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payoff: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CandidateRoles> for super::CandidateRoles {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CandidateRoles,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                hook: value.hook?,
                payoff: value.payoff?,
            })
        }
    }
    impl ::std::convert::From<super::CandidateRoles> for CandidateRoles {
        fn from(value: super::CandidateRoles) -> Self {
            Self {
                hook: Ok(value.hook),
                payoff: Ok(value.payoff),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Cluster {
        id: ::std::result::Result<super::ClusterId, ::std::string::String>,
        members: ::std::result::Result<
            ::std::vec::Vec<super::ClusterMembersItem>,
            ::std::string::String,
        >,
        representative: ::std::result::Result<super::ClusterRepresentative, ::std::string::String>,
        similarity: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Cluster {
        fn default() -> Self {
            Self {
                id: Err("no value supplied for id".to_string()),
                members: Err("no value supplied for members".to_string()),
                representative: Err("no value supplied for representative".to_string()),
                similarity: Err("no value supplied for similarity".to_string()),
            }
        }
    }
    impl Cluster {
        pub fn id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ClusterId>,
            T::Error: ::std::fmt::Display,
        {
            self.id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for id: {e}"));
            self
        }
        pub fn members<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ClusterMembersItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.members = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for members: {e}"));
            self
        }
        pub fn representative<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ClusterRepresentative>,
            T::Error: ::std::fmt::Display,
        {
            self.representative = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for representative: {e}"));
            self
        }
        pub fn similarity<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.similarity = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for similarity: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Cluster> for super::Cluster {
        type Error = super::error::ConversionError;
        fn try_from(value: Cluster) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                id: value.id?,
                members: value.members?,
                representative: value.representative?,
                similarity: value.similarity?,
            })
        }
    }
    impl ::std::convert::From<super::Cluster> for Cluster {
        fn from(value: super::Cluster) -> Self {
            Self {
                id: Ok(value.id),
                members: Ok(value.members),
                representative: Ok(value.representative),
                similarity: Ok(value.similarity),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Coverage {
        analyzed: ::std::result::Result<bool, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Coverage {
        fn default() -> Self {
            Self {
                analyzed: Err("no value supplied for analyzed".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
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
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Coverage> for Coverage {
        fn from(value: super::Coverage) -> Self {
            Self {
                analyzed: Ok(value.analyzed),
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DiscoveryCandidates {
        candidates: ::std::result::Result<::std::vec::Vec<super::Candidate>, ::std::string::String>,
        clusters: ::std::result::Result<::std::vec::Vec<super::Cluster>, ::std::string::String>,
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        duration_target: ::std::result::Result<super::DurationRange, ::std::string::String>,
        inputs: ::std::result::Result<super::DiscoveryCandidatesInputs, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        proposers:
            ::std::result::Result<::std::vec::Vec<super::ProposerRun>, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for DiscoveryCandidates {
        fn default() -> Self {
            Self {
                candidates: Err("no value supplied for candidates".to_string()),
                clusters: Err("no value supplied for clusters".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                duration_target: Err("no value supplied for duration_target".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                proposers: Err("no value supplied for proposers".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl DiscoveryCandidates {
        pub fn candidates<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Candidate>>,
            T::Error: ::std::fmt::Display,
        {
            self.candidates = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidates: {e}"));
            self
        }
        pub fn clusters<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Cluster>>,
            T::Error: ::std::fmt::Display,
        {
            self.clusters = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for clusters: {e}"));
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
        pub fn duration_target<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DurationRange>,
            T::Error: ::std::fmt::Display,
        {
            self.duration_target = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for duration_target: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DiscoveryCandidatesInputs>,
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
        pub fn proposers<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ProposerRun>>,
            T::Error: ::std::fmt::Display,
        {
            self.proposers = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for proposers: {e}"));
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
    impl ::std::convert::TryFrom<DiscoveryCandidates> for super::DiscoveryCandidates {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DiscoveryCandidates,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidates: value.candidates?,
                clusters: value.clusters?,
                coverage: value.coverage?,
                duration_target: value.duration_target?,
                inputs: value.inputs?,
                producer: value.producer?,
                proposers: value.proposers?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::DiscoveryCandidates> for DiscoveryCandidates {
        fn from(value: super::DiscoveryCandidates) -> Self {
            Self {
                candidates: Ok(value.candidates),
                clusters: Ok(value.clusters),
                coverage: Ok(value.coverage),
                duration_target: Ok(value.duration_target),
                inputs: Ok(value.inputs),
                producer: Ok(value.producer),
                proposers: Ok(value.proposers),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DiscoveryCandidatesInputs {
        index_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        loudness_artifact_id:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        transcript_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for DiscoveryCandidatesInputs {
        fn default() -> Self {
            Self {
                index_artifact_id: Err("no value supplied for index_artifact_id".to_string()),
                loudness_artifact_id: Ok(Default::default()),
                transcript_artifact_id: Err(
                    "no value supplied for transcript_artifact_id".to_string()
                ),
            }
        }
    }
    impl DiscoveryCandidatesInputs {
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
        pub fn loudness_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.loudness_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for loudness_artifact_id: {e}")
            });
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
    impl ::std::convert::TryFrom<DiscoveryCandidatesInputs> for super::DiscoveryCandidatesInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DiscoveryCandidatesInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                index_artifact_id: value.index_artifact_id?,
                loudness_artifact_id: value.loudness_artifact_id?,
                transcript_artifact_id: value.transcript_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::DiscoveryCandidatesInputs> for DiscoveryCandidatesInputs {
        fn from(value: super::DiscoveryCandidatesInputs) -> Self {
            Self {
                index_artifact_id: Ok(value.index_artifact_id),
                loudness_artifact_id: Ok(value.loudness_artifact_id),
                transcript_artifact_id: Ok(value.transcript_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DurationRange {
        max_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        min_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for DurationRange {
        fn default() -> Self {
            Self {
                max_ticks: Err("no value supplied for max_ticks".to_string()),
                min_ticks: Err("no value supplied for min_ticks".to_string()),
            }
        }
    }
    impl DurationRange {
        pub fn max_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max_ticks: {e}"));
            self
        }
        pub fn min_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for min_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DurationRange> for super::DurationRange {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DurationRange,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                max_ticks: value.max_ticks?,
                min_ticks: value.min_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::DurationRange> for DurationRange {
        fn from(value: super::DurationRange) -> Self {
            Self {
                max_ticks: Ok(value.max_ticks),
                min_ticks: Ok(value.min_ticks),
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
    pub struct Exclusion {
        detail: ::std::result::Result<
            ::std::option::Option<super::ExclusionDetail>,
            ::std::string::String,
        >,
        reason: ::std::result::Result<super::ExclusionReason, ::std::string::String>,
    }
    impl ::std::default::Default for Exclusion {
        fn default() -> Self {
            Self {
                detail: Ok(Default::default()),
                reason: Err("no value supplied for reason".to_string()),
            }
        }
    }
    impl Exclusion {
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ExclusionDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ExclusionReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Exclusion> for super::Exclusion {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Exclusion,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                detail: value.detail?,
                reason: value.reason?,
            })
        }
    }
    impl ::std::convert::From<super::Exclusion> for Exclusion {
        fn from(value: super::Exclusion) -> Self {
            Self {
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
    pub struct PhiReject {
        count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        reason: ::std::result::Result<super::PhiRejectReason, ::std::string::String>,
    }
    impl ::std::default::Default for PhiReject {
        fn default() -> Self {
            Self {
                count: Err("no value supplied for count".to_string()),
                reason: Err("no value supplied for reason".to_string()),
            }
        }
    }
    impl PhiReject {
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
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PhiRejectReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<PhiReject> for super::PhiReject {
        type Error = super::error::ConversionError;
        fn try_from(
            value: PhiReject,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                count: value.count?,
                reason: value.reason?,
            })
        }
    }
    impl ::std::convert::From<super::PhiReject> for PhiReject {
        fn from(value: super::PhiReject) -> Self {
            Self {
                count: Ok(value.count),
                reason: Ok(value.reason),
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
    pub struct Proposer {
        name: ::std::result::Result<super::ProposerName, ::std::string::String>,
        rubric: ::std::result::Result<super::ProposerRubric, ::std::string::String>,
        version: ::std::result::Result<super::ProposerVersion, ::std::string::String>,
    }
    impl ::std::default::Default for Proposer {
        fn default() -> Self {
            Self {
                name: Err("no value supplied for name".to_string()),
                rubric: Err("no value supplied for rubric".to_string()),
                version: Err("no value supplied for version".to_string()),
            }
        }
    }
    impl Proposer {
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposerName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn rubric<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposerRubric>,
            T::Error: ::std::fmt::Display,
        {
            self.rubric = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rubric: {e}"));
            self
        }
        pub fn version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposerVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for version: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Proposer> for super::Proposer {
        type Error = super::error::ConversionError;
        fn try_from(value: Proposer) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                name: value.name?,
                rubric: value.rubric?,
                version: value.version?,
            })
        }
    }
    impl ::std::convert::From<super::Proposer> for Proposer {
        fn from(value: super::Proposer) -> Self {
            Self {
                name: Ok(value.name),
                rubric: Ok(value.rubric),
                version: Ok(value.version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ProposerRun {
        candidates: ::std::result::Result<u64, ::std::string::String>,
        floor_applied: ::std::result::Result<
            ::std::option::Option<::std::num::NonZeroU64>,
            ::std::string::String,
        >,
        proposer: ::std::result::Result<super::Proposer, ::std::string::String>,
        seeds: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for ProposerRun {
        fn default() -> Self {
            Self {
                candidates: Err("no value supplied for candidates".to_string()),
                floor_applied: Ok(Default::default()),
                proposer: Err("no value supplied for proposer".to_string()),
                seeds: Err("no value supplied for seeds".to_string()),
            }
        }
    }
    impl ProposerRun {
        pub fn candidates<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.candidates = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidates: {e}"));
            self
        }
        pub fn floor_applied<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::num::NonZeroU64>>,
            T::Error: ::std::fmt::Display,
        {
            self.floor_applied = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for floor_applied: {e}"));
            self
        }
        pub fn proposer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Proposer>,
            T::Error: ::std::fmt::Display,
        {
            self.proposer = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for proposer: {e}"));
            self
        }
        pub fn seeds<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.seeds = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for seeds: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ProposerRun> for super::ProposerRun {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ProposerRun,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidates: value.candidates?,
                floor_applied: value.floor_applied?,
                proposer: value.proposer?,
                seeds: value.seeds?,
            })
        }
    }
    impl ::std::convert::From<super::ProposerRun> for ProposerRun {
        fn from(value: super::ProposerRun) -> Self {
            Self {
                candidates: Ok(value.candidates),
                floor_applied: Ok(value.floor_applied),
                proposer: Ok(value.proposer),
                seeds: Ok(value.seeds),
            }
        }
    }
}
