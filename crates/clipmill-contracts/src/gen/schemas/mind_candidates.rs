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
#[doc = "The clips the cutter proposes for one recording, best first by a provisional score: plans of parts (tightened, stitched, cold-opened, montaged or straight) built from the Recording Memory and the sentence markup. Each plan is a `clipmill.mind.clip_plan.v1` document that already passed the plan checks against the recording's words and the length asked for. The planner and the ranker decide between them; the score here only orders them."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.mind.candidates.v1.json\","]
#[doc = "  \"title\": \"MindCandidates\","]
#[doc = "  \"description\": \"The clips the cutter proposes for one recording, best first by a provisional score: plans of parts (tightened, stitched, cold-opened, montaged or straight) built from the Recording Memory and the sentence markup. Each plan is a `clipmill.mind.clip_plan.v1` document that already passed the plan checks against the recording's words and the length asked for. The planner and the ranker decide between them; the score here only orders them.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"candidates\","]
#[doc = "    \"inputs\","]
#[doc = "    \"length\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"candidates\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"plan\","]
#[doc = "          \"score\","]
#[doc = "          \"scout\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"alternatives\": {"]
#[doc = "            \"description\": \"Other validated forms of the same moment, available to the planner for comparison and one repair.\","]
#[doc = "            \"type\": \"array\","]
#[doc = "            \"items\": {"]
#[doc = "              \"type\": \"object\""]
#[doc = "            },"]
#[doc = "            \"maxItems\": 3"]
#[doc = "          },"]
#[doc = "          \"plan\": {"]
#[doc = "            \"description\": \"A clipmill.mind.clip_plan.v1 document.\","]
#[doc = "            \"type\": \"object\""]
#[doc = "          },"]
#[doc = "          \"score\": {"]
#[doc = "            \"type\": \"integer\""]
#[doc = "          },"]
#[doc = "          \"scout\": {"]
#[doc = "            \"description\": \"What found it: an event, a link between events, a strong line lifted to open a moment, a theme across moments, or a hook-to-landing span, a match to the brief, or a sound from the audience.\","]
#[doc = "            \"enum\": ["]
#[doc = "              \"event\","]
#[doc = "              \"link\","]
#[doc = "              \"cold_open\","]
#[doc = "              \"theme\","]
#[doc = "              \"span\","]
#[doc = "              \"brief\","]
#[doc = "              \"reaction\""]
#[doc = "            ]"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"description\": \"The artifacts the cutter read.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"script_artifact_id\","]
#[doc = "        \"timeline_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"memory_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"scores_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"script_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"timeline_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"legal_ends\": {"]
#[doc = "      \"description\": \"Where a part may end after each sentence the cutter allows to close a clip: the sentence id and the cut point in the pause after it, in ticks. Sentences missing here cannot end a clip. Absent from candidates written before the cutter measured pauses.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"type\": \"integer\","]
#[doc = "        \"minimum\": 0.0"]
#[doc = "      },"]
#[doc = "      \"propertyNames\": {"]
#[doc = "        \"pattern\": \"^s[0-9]+$\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"length\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"max_ticks\","]
#[doc = "        \"min_ticks\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"max_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"min_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.mind.candidates.v1\""]
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
pub struct MindCandidates {
    pub candidates: ::std::vec::Vec<MindCandidatesCandidatesItem>,
    pub inputs: MindCandidatesInputs,
    #[doc = "Where a part may end after each sentence the cutter allows to close a clip: the sentence id and the cut point in the pause after it, in ticks. Sentences missing here cannot end a clip. Absent from candidates written before the cutter measured pauses."]
    #[serde(
        default,
        skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
    )]
    pub legal_ends: ::std::collections::HashMap<MindCandidatesLegalEndsKey, u64>,
    pub length: MindCandidatesLength,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
}
impl MindCandidates {
    pub fn builder() -> builder::MindCandidates {
        Default::default()
    }
}
#[doc = "`MindCandidatesCandidatesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"plan\","]
#[doc = "    \"score\","]
#[doc = "    \"scout\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"alternatives\": {"]
#[doc = "      \"description\": \"Other validated forms of the same moment, available to the planner for comparison and one repair.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 3"]
#[doc = "    },"]
#[doc = "    \"plan\": {"]
#[doc = "      \"description\": \"A clipmill.mind.clip_plan.v1 document.\","]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"scout\": {"]
#[doc = "      \"description\": \"What found it: an event, a link between events, a strong line lifted to open a moment, a theme across moments, or a hook-to-landing span, a match to the brief, or a sound from the audience.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"event\","]
#[doc = "        \"link\","]
#[doc = "        \"cold_open\","]
#[doc = "        \"theme\","]
#[doc = "        \"span\","]
#[doc = "        \"brief\","]
#[doc = "        \"reaction\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindCandidatesCandidatesItem {
    #[doc = "Other validated forms of the same moment, available to the planner for comparison and one repair."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub alternatives:
        ::std::vec::Vec<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[doc = "A clipmill.mind.clip_plan.v1 document."]
    pub plan: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    pub score: i64,
    #[doc = "What found it: an event, a link between events, a strong line lifted to open a moment, a theme across moments, or a hook-to-landing span, a match to the brief, or a sound from the audience."]
    pub scout: MindCandidatesCandidatesItemScout,
}
impl MindCandidatesCandidatesItem {
    pub fn builder() -> builder::MindCandidatesCandidatesItem {
        Default::default()
    }
}
#[doc = "What found it: an event, a link between events, a strong line lifted to open a moment, a theme across moments, or a hook-to-landing span, a match to the brief, or a sound from the audience."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What found it: an event, a link between events, a strong line lifted to open a moment, a theme across moments, or a hook-to-landing span, a match to the brief, or a sound from the audience.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"event\","]
#[doc = "    \"link\","]
#[doc = "    \"cold_open\","]
#[doc = "    \"theme\","]
#[doc = "    \"span\","]
#[doc = "    \"brief\","]
#[doc = "    \"reaction\""]
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
pub enum MindCandidatesCandidatesItemScout {
    #[serde(rename = "event")]
    Event,
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "cold_open")]
    ColdOpen,
    #[serde(rename = "theme")]
    Theme,
    #[serde(rename = "span")]
    Span,
    #[serde(rename = "brief")]
    Brief,
    #[serde(rename = "reaction")]
    Reaction,
}
impl ::std::fmt::Display for MindCandidatesCandidatesItemScout {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Event => f.write_str("event"),
            Self::Link => f.write_str("link"),
            Self::ColdOpen => f.write_str("cold_open"),
            Self::Theme => f.write_str("theme"),
            Self::Span => f.write_str("span"),
            Self::Brief => f.write_str("brief"),
            Self::Reaction => f.write_str("reaction"),
        }
    }
}
impl ::std::str::FromStr for MindCandidatesCandidatesItemScout {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "event" => Ok(Self::Event),
            "link" => Ok(Self::Link),
            "cold_open" => Ok(Self::ColdOpen),
            "theme" => Ok(Self::Theme),
            "span" => Ok(Self::Span),
            "brief" => Ok(Self::Brief),
            "reaction" => Ok(Self::Reaction),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindCandidatesCandidatesItemScout {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindCandidatesCandidatesItemScout {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindCandidatesCandidatesItemScout {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "The artifacts the cutter read."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The artifacts the cutter read.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"script_artifact_id\","]
#[doc = "    \"timeline_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"memory_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"scores_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"script_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"timeline_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindCandidatesInputs {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub memory_artifact_id: ::std::option::Option<Sha256>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub scores_artifact_id: ::std::option::Option<Sha256>,
    pub script_artifact_id: Sha256,
    pub timeline_artifact_id: Sha256,
}
impl MindCandidatesInputs {
    pub fn builder() -> builder::MindCandidatesInputs {
        Default::default()
    }
}
#[doc = "`MindCandidatesLegalEndsKey`"]
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
pub struct MindCandidatesLegalEndsKey(::std::string::String);
impl ::std::ops::Deref for MindCandidatesLegalEndsKey {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindCandidatesLegalEndsKey> for ::std::string::String {
    fn from(value: MindCandidatesLegalEndsKey) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindCandidatesLegalEndsKey {
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
impl ::std::convert::TryFrom<&str> for MindCandidatesLegalEndsKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindCandidatesLegalEndsKey {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindCandidatesLegalEndsKey {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindCandidatesLegalEndsKey {
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
#[doc = "`MindCandidatesLength`"]
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
pub struct MindCandidatesLength {
    pub max_ticks: ::std::num::NonZeroU64,
    pub min_ticks: ::std::num::NonZeroU64,
}
impl MindCandidatesLength {
    pub fn builder() -> builder::MindCandidatesLength {
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
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct MindCandidates {
        candidates: ::std::result::Result<
            ::std::vec::Vec<super::MindCandidatesCandidatesItem>,
            ::std::string::String,
        >,
        inputs: ::std::result::Result<super::MindCandidatesInputs, ::std::string::String>,
        legal_ends: ::std::result::Result<
            ::std::collections::HashMap<super::MindCandidatesLegalEndsKey, u64>,
            ::std::string::String,
        >,
        length: ::std::result::Result<super::MindCandidatesLength, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for MindCandidates {
        fn default() -> Self {
            Self {
                candidates: Err("no value supplied for candidates".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                legal_ends: Ok(Default::default()),
                length: Err("no value supplied for length".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl MindCandidates {
        pub fn candidates<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindCandidatesCandidatesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.candidates = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for candidates: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindCandidatesInputs>,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
            self
        }
        pub fn legal_ends<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::collections::HashMap<super::MindCandidatesLegalEndsKey, u64>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.legal_ends = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for legal_ends: {e}"));
            self
        }
        pub fn length<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindCandidatesLength>,
            T::Error: ::std::fmt::Display,
        {
            self.length = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for length: {e}"));
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
    impl ::std::convert::TryFrom<MindCandidates> for super::MindCandidates {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindCandidates,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                candidates: value.candidates?,
                inputs: value.inputs?,
                legal_ends: value.legal_ends?,
                length: value.length?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::MindCandidates> for MindCandidates {
        fn from(value: super::MindCandidates) -> Self {
            Self {
                candidates: Ok(value.candidates),
                inputs: Ok(value.inputs),
                legal_ends: Ok(value.legal_ends),
                length: Ok(value.length),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindCandidatesCandidatesItem {
        alternatives: ::std::result::Result<
            ::std::vec::Vec<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
            ::std::string::String,
        >,
        plan: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        score: ::std::result::Result<i64, ::std::string::String>,
        scout:
            ::std::result::Result<super::MindCandidatesCandidatesItemScout, ::std::string::String>,
    }
    impl ::std::default::Default for MindCandidatesCandidatesItem {
        fn default() -> Self {
            Self {
                alternatives: Ok(Default::default()),
                plan: Err("no value supplied for plan".to_string()),
                score: Err("no value supplied for score".to_string()),
                scout: Err("no value supplied for scout".to_string()),
            }
        }
    }
    impl MindCandidatesCandidatesItem {
        pub fn alternatives<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.alternatives = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for alternatives: {e}"));
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
        pub fn score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.score = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for score: {e}"));
            self
        }
        pub fn scout<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindCandidatesCandidatesItemScout>,
            T::Error: ::std::fmt::Display,
        {
            self.scout = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for scout: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindCandidatesCandidatesItem> for super::MindCandidatesCandidatesItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindCandidatesCandidatesItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                alternatives: value.alternatives?,
                plan: value.plan?,
                score: value.score?,
                scout: value.scout?,
            })
        }
    }
    impl ::std::convert::From<super::MindCandidatesCandidatesItem> for MindCandidatesCandidatesItem {
        fn from(value: super::MindCandidatesCandidatesItem) -> Self {
            Self {
                alternatives: Ok(value.alternatives),
                plan: Ok(value.plan),
                score: Ok(value.score),
                scout: Ok(value.scout),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindCandidatesInputs {
        memory_artifact_id:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        scores_artifact_id:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        script_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        timeline_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for MindCandidatesInputs {
        fn default() -> Self {
            Self {
                memory_artifact_id: Ok(Default::default()),
                scores_artifact_id: Ok(Default::default()),
                script_artifact_id: Err("no value supplied for script_artifact_id".to_string()),
                timeline_artifact_id: Err("no value supplied for timeline_artifact_id".to_string()),
            }
        }
    }
    impl MindCandidatesInputs {
        pub fn memory_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.memory_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for memory_artifact_id: {e}")
            });
            self
        }
        pub fn scores_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.scores_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for scores_artifact_id: {e}")
            });
            self
        }
        pub fn script_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.script_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for script_artifact_id: {e}")
            });
            self
        }
        pub fn timeline_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.timeline_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for timeline_artifact_id: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<MindCandidatesInputs> for super::MindCandidatesInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindCandidatesInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                memory_artifact_id: value.memory_artifact_id?,
                scores_artifact_id: value.scores_artifact_id?,
                script_artifact_id: value.script_artifact_id?,
                timeline_artifact_id: value.timeline_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::MindCandidatesInputs> for MindCandidatesInputs {
        fn from(value: super::MindCandidatesInputs) -> Self {
            Self {
                memory_artifact_id: Ok(value.memory_artifact_id),
                scores_artifact_id: Ok(value.scores_artifact_id),
                script_artifact_id: Ok(value.script_artifact_id),
                timeline_artifact_id: Ok(value.timeline_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindCandidatesLength {
        max_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        min_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for MindCandidatesLength {
        fn default() -> Self {
            Self {
                max_ticks: Err("no value supplied for max_ticks".to_string()),
                min_ticks: Err("no value supplied for min_ticks".to_string()),
            }
        }
    }
    impl MindCandidatesLength {
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
    impl ::std::convert::TryFrom<MindCandidatesLength> for super::MindCandidatesLength {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindCandidatesLength,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                max_ticks: value.max_ticks?,
                min_ticks: value.min_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::MindCandidatesLength> for MindCandidatesLength {
        fn from(value: super::MindCandidatesLength) -> Self {
            Self {
                max_ticks: Ok(value.max_ticks),
                min_ticks: Ok(value.min_ticks),
            }
        }
    }
}
