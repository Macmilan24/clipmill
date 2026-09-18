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
#[doc = "The index's own analyzed range, echoed rather than recomputed."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The index's own analyzed range, echoed rather than recomputed.\","]
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
#[doc = "The transcript cut into the pieces an editorial model reads, one at a time, to propose moments (plan, Milestone 2). Windows are ranges into one sentence list rather than copies of it, so a proposal cites a sentence index that means the same thing in every window it appears in; they overlap so that a moment straddling a boundary is seen whole in at least one of them; and they never split a sentence. Every sentence is inside the core of at least one window — coverage is the property, not sampling. Context on either side of a window is offered to the model as context and is not proposable from that window. The outline is one line per topic, each citing the sentences it summarizes, and is context for a long recording, never a substitute for the words. Everything here is derived arithmetic over the index; no model has read anything yet."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.editorial.windows.v1.json\","]
#[doc = "  \"title\": \"EditorialWindows\","]
#[doc = "  \"description\": \"The transcript cut into the pieces an editorial model reads, one at a time, to propose moments (plan, Milestone 2). Windows are ranges into one sentence list rather than copies of it, so a proposal cites a sentence index that means the same thing in every window it appears in; they overlap so that a moment straddling a boundary is seen whole in at least one of them; and they never split a sentence. Every sentence is inside the core of at least one window — coverage is the property, not sampling. Context on either side of a window is offered to the model as context and is not proposable from that window. The outline is one line per topic, each citing the sentences it summarizes, and is context for a long recording, never a substitute for the words. Everything here is derived arithmetic over the index; no model has read anything yet.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"budget\","]
#[doc = "    \"coverage\","]
#[doc = "    \"inputs\","]
#[doc = "    \"invalid_regions\","]
#[doc = "    \"language\","]
#[doc = "    \"outline\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"sentences\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"windows\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"budget\": {"]
#[doc = "      \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. Part of the artifact key: a different budget is a different cut of the same transcript, not a correction of this one.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"context_sentences\","]
#[doc = "        \"overlap_words\","]
#[doc = "        \"target_words\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"context_sentences\": {"]
#[doc = "          \"description\": \"How many sentences on each side of the core are offered as context.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"overlap_words\": {"]
#[doc = "          \"description\": \"At least this many words of the previous window's tail begin the next window, so a moment on the seam is seen whole by one of them.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"target_words\": {"]
#[doc = "          \"description\": \"How many words a window's core aims to hold. A single sentence longer than this still forms a window on its own; a sentence is never split.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"description\": \"The index this was cut from, and the transcript the index's word indexes are the authority for — carried so a consumer can walk a word index back to a word without opening the index first.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"index_artifact_id\","]
#[doc = "        \"transcript_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"index_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"transcript_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"invalid_regions\": {"]
#[doc = "      \"description\": \"Echoed from the index: where the timing under these sentences is a guess, so a consumer that cuts on a word there knows what the cut is worth.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/invalid_region\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"description\": \"Echoed from the index, so the prompt can be written in the recording's language without opening it.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 16,"]
#[doc = "      \"minLength\": 2"]
#[doc = "    },"]
#[doc = "    \"outline\": {"]
#[doc = "      \"description\": \"One line per topic the index found, in order, each citing the sentences it stands for. Empty when the index found no topics.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/outline_line\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.editorial.windows.v1\""]
#[doc = "    },"]
#[doc = "    \"sentences\": {"]
#[doc = "      \"description\": \"Every sentence of the index, once, in order, with the words behind it. A window is ranges into this list.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/sentence\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"windows\": {"]
#[doc = "      \"description\": \"In order, each beginning at or before the previous one ends, and together covering every sentence.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/window\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditorialWindows {
    pub budget: EditorialWindowsBudget,
    pub coverage: Coverage,
    pub inputs: EditorialWindowsInputs,
    #[doc = "Echoed from the index: where the timing under these sentences is a guess, so a consumer that cuts on a word there knows what the cut is worth."]
    pub invalid_regions: ::std::vec::Vec<InvalidRegion>,
    #[doc = "Echoed from the index, so the prompt can be written in the recording's language without opening it."]
    pub language: EditorialWindowsLanguage,
    #[doc = "One line per topic the index found, in order, each citing the sentences it stands for. Empty when the index found no topics."]
    pub outline: ::std::vec::Vec<OutlineLine>,
    pub producer: Producer,
    pub schema_version: ::serde_json::Value,
    #[doc = "Every sentence of the index, once, in order, with the words behind it. A window is ranges into this list."]
    pub sentences: ::std::vec::Vec<Sentence>,
    pub source_fingerprint: Sha256,
    #[doc = "In order, each beginning at or before the previous one ends, and together covering every sentence."]
    pub windows: ::std::vec::Vec<Window>,
}
impl EditorialWindows {
    pub fn builder() -> builder::EditorialWindows {
        Default::default()
    }
}
#[doc = "The decision parameters, recorded so a later pass knows what to change rather than guessing. Part of the artifact key: a different budget is a different cut of the same transcript, not a correction of this one."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. Part of the artifact key: a different budget is a different cut of the same transcript, not a correction of this one.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"context_sentences\","]
#[doc = "    \"overlap_words\","]
#[doc = "    \"target_words\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"context_sentences\": {"]
#[doc = "      \"description\": \"How many sentences on each side of the core are offered as context.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"overlap_words\": {"]
#[doc = "      \"description\": \"At least this many words of the previous window's tail begin the next window, so a moment on the seam is seen whole by one of them.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"target_words\": {"]
#[doc = "      \"description\": \"How many words a window's core aims to hold. A single sentence longer than this still forms a window on its own; a sentence is never split.\","]
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
pub struct EditorialWindowsBudget {
    #[doc = "How many sentences on each side of the core are offered as context."]
    pub context_sentences: u64,
    #[doc = "At least this many words of the previous window's tail begin the next window, so a moment on the seam is seen whole by one of them."]
    pub overlap_words: u64,
    #[doc = "How many words a window's core aims to hold. A single sentence longer than this still forms a window on its own; a sentence is never split."]
    pub target_words: ::std::num::NonZeroU64,
}
impl EditorialWindowsBudget {
    pub fn builder() -> builder::EditorialWindowsBudget {
        Default::default()
    }
}
#[doc = "The index this was cut from, and the transcript the index's word indexes are the authority for — carried so a consumer can walk a word index back to a word without opening the index first."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The index this was cut from, and the transcript the index's word indexes are the authority for — carried so a consumer can walk a word index back to a word without opening the index first.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"index_artifact_id\","]
#[doc = "    \"transcript_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
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
pub struct EditorialWindowsInputs {
    pub index_artifact_id: Sha256,
    pub transcript_artifact_id: Sha256,
}
impl EditorialWindowsInputs {
    pub fn builder() -> builder::EditorialWindowsInputs {
        Default::default()
    }
}
#[doc = "Echoed from the index, so the prompt can be written in the recording's language without opening it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Echoed from the index, so the prompt can be written in the recording's language without opening it.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 16,"]
#[doc = "  \"minLength\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EditorialWindowsLanguage(::std::string::String);
impl ::std::ops::Deref for EditorialWindowsLanguage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EditorialWindowsLanguage> for ::std::string::String {
    fn from(value: EditorialWindowsLanguage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EditorialWindowsLanguage {
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
impl ::std::convert::TryFrom<&str> for EditorialWindowsLanguage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EditorialWindowsLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EditorialWindowsLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EditorialWindowsLanguage {
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
#[doc = "`OutlineLine`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_sentence_index\","]
#[doc = "    \"keywords\","]
#[doc = "    \"sentence_count\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"topic_index\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"first_sentence_index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"keywords\": {"]
#[doc = "      \"description\": \"The topic's terms, most frequent first, as the index counted them. The line a model reads is these words; a sentence it wants to cite is in the range above.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"sentence_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"topic_index\": {"]
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
pub struct OutlineLine {
    pub end_ticks: u64,
    pub first_sentence_index: u64,
    #[doc = "The topic's terms, most frequent first, as the index counted them. The line a model reads is these words; a sentence it wants to cite is in the range above."]
    pub keywords: ::std::vec::Vec<OutlineLineKeywordsItem>,
    pub sentence_count: ::std::num::NonZeroU64,
    pub start_ticks: u64,
    pub topic_index: u64,
}
impl OutlineLine {
    pub fn builder() -> builder::OutlineLine {
        Default::default()
    }
}
#[doc = "`OutlineLineKeywordsItem`"]
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
pub struct OutlineLineKeywordsItem(::std::string::String);
impl ::std::ops::Deref for OutlineLineKeywordsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<OutlineLineKeywordsItem> for ::std::string::String {
    fn from(value: OutlineLineKeywordsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for OutlineLineKeywordsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for OutlineLineKeywordsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for OutlineLineKeywordsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for OutlineLineKeywordsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for OutlineLineKeywordsItem {
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
#[doc = "`Sentence`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_word_index\","]
#[doc = "    \"index\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"terminator\","]
#[doc = "    \"text\","]
#[doc = "    \"word_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"first_word_index\": {"]
#[doc = "      \"description\": \"The transcript word index of the sentence's first word; word ids are `w` followed by this index, as the captions carry them.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"description\": \"The sentence's index in the index document, which is what a proposal cites.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"terminator\": {"]
#[doc = "      \"description\": \"How the sentence ended, as the index decided it: a boundary the recognizer punctuated is stronger evidence than one where the speaker merely stopped.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"punctuation\","]
#[doc = "        \"utterance_end\","]
#[doc = "        \"coverage_end\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"word_count\": {"]
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
pub struct Sentence {
    pub end_ticks: u64,
    #[doc = "The transcript word index of the sentence's first word; word ids are `w` followed by this index, as the captions carry them."]
    pub first_word_index: u64,
    #[doc = "The sentence's index in the index document, which is what a proposal cites."]
    pub index: u64,
    pub start_ticks: u64,
    #[doc = "How the sentence ended, as the index decided it: a boundary the recognizer punctuated is stronger evidence than one where the speaker merely stopped."]
    pub terminator: SentenceTerminator,
    pub text: SentenceText,
    pub word_count: ::std::num::NonZeroU64,
}
impl Sentence {
    pub fn builder() -> builder::Sentence {
        Default::default()
    }
}
#[doc = "How the sentence ended, as the index decided it: a boundary the recognizer punctuated is stronger evidence than one where the speaker merely stopped."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How the sentence ended, as the index decided it: a boundary the recognizer punctuated is stronger evidence than one where the speaker merely stopped.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"punctuation\","]
#[doc = "    \"utterance_end\","]
#[doc = "    \"coverage_end\""]
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
pub enum SentenceTerminator {
    #[serde(rename = "punctuation")]
    Punctuation,
    #[serde(rename = "utterance_end")]
    UtteranceEnd,
    #[serde(rename = "coverage_end")]
    CoverageEnd,
}
impl ::std::fmt::Display for SentenceTerminator {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Punctuation => f.write_str("punctuation"),
            Self::UtteranceEnd => f.write_str("utterance_end"),
            Self::CoverageEnd => f.write_str("coverage_end"),
        }
    }
}
impl ::std::str::FromStr for SentenceTerminator {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "punctuation" => Ok(Self::Punctuation),
            "utterance_end" => Ok(Self::UtteranceEnd),
            "coverage_end" => Ok(Self::CoverageEnd),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for SentenceTerminator {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SentenceTerminator {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SentenceTerminator {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`SentenceText`"]
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
pub struct SentenceText(::std::string::String);
impl ::std::ops::Deref for SentenceText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<SentenceText> for ::std::string::String {
    fn from(value: SentenceText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for SentenceText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for SentenceText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SentenceText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SentenceText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for SentenceText {
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
#[doc = "`Window`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"context_after_sentences\","]
#[doc = "    \"context_before_sentences\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_sentence_index\","]
#[doc = "    \"index\","]
#[doc = "    \"sentence_count\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"topic_indexes\","]
#[doc = "    \"word_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"context_after_sentences\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"context_before_sentences\": {"]
#[doc = "      \"description\": \"How many sentences before the core are offered as context, fewer than the budget only at the recording's start.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"first_sentence_index\": {"]
#[doc = "      \"description\": \"The first sentence of the core: the sentences a proposal from this window may cite.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"sentence_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"description\": \"Where the core begins and ends in the recording — the first sentence's start and the last sentence's end.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"topic_indexes\": {"]
#[doc = "      \"description\": \"The index's topics the core touches, in order — what the outline line for this window is.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"integer\","]
#[doc = "        \"minimum\": 0.0"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"word_count\": {"]
#[doc = "      \"description\": \"Words in the core, so a consumer can size a prompt without summing.\","]
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
pub struct Window {
    pub context_after_sentences: u64,
    #[doc = "How many sentences before the core are offered as context, fewer than the budget only at the recording's start."]
    pub context_before_sentences: u64,
    pub end_ticks: u64,
    #[doc = "The first sentence of the core: the sentences a proposal from this window may cite."]
    pub first_sentence_index: u64,
    pub index: u64,
    pub sentence_count: ::std::num::NonZeroU64,
    #[doc = "Where the core begins and ends in the recording — the first sentence's start and the last sentence's end."]
    pub start_ticks: u64,
    #[doc = "The index's topics the core touches, in order — what the outline line for this window is."]
    pub topic_indexes: ::std::vec::Vec<u64>,
    #[doc = "Words in the core, so a consumer can size a prompt without summing."]
    pub word_count: ::std::num::NonZeroU64,
}
impl Window {
    pub fn builder() -> builder::Window {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
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
    pub struct EditorialWindows {
        budget: ::std::result::Result<super::EditorialWindowsBudget, ::std::string::String>,
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        inputs: ::std::result::Result<super::EditorialWindowsInputs, ::std::string::String>,
        invalid_regions:
            ::std::result::Result<::std::vec::Vec<super::InvalidRegion>, ::std::string::String>,
        language: ::std::result::Result<super::EditorialWindowsLanguage, ::std::string::String>,
        outline: ::std::result::Result<::std::vec::Vec<super::OutlineLine>, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        sentences: ::std::result::Result<::std::vec::Vec<super::Sentence>, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        windows: ::std::result::Result<::std::vec::Vec<super::Window>, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialWindows {
        fn default() -> Self {
            Self {
                budget: Err("no value supplied for budget".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                invalid_regions: Err("no value supplied for invalid_regions".to_string()),
                language: Err("no value supplied for language".to_string()),
                outline: Err("no value supplied for outline".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                sentences: Err("no value supplied for sentences".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                windows: Err("no value supplied for windows".to_string()),
            }
        }
    }
    impl EditorialWindows {
        pub fn budget<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialWindowsBudget>,
            T::Error: ::std::fmt::Display,
        {
            self.budget = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for budget: {e}"));
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
            T: ::std::convert::TryInto<super::EditorialWindowsInputs>,
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
            T: ::std::convert::TryInto<super::EditorialWindowsLanguage>,
            T::Error: ::std::fmt::Display,
        {
            self.language = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for language: {e}"));
            self
        }
        pub fn outline<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::OutlineLine>>,
            T::Error: ::std::fmt::Display,
        {
            self.outline = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outline: {e}"));
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
        pub fn sentences<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Sentence>>,
            T::Error: ::std::fmt::Display,
        {
            self.sentences = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sentences: {e}"));
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
        pub fn windows<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Window>>,
            T::Error: ::std::fmt::Display,
        {
            self.windows = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for windows: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditorialWindows> for super::EditorialWindows {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialWindows,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                budget: value.budget?,
                coverage: value.coverage?,
                inputs: value.inputs?,
                invalid_regions: value.invalid_regions?,
                language: value.language?,
                outline: value.outline?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                sentences: value.sentences?,
                source_fingerprint: value.source_fingerprint?,
                windows: value.windows?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialWindows> for EditorialWindows {
        fn from(value: super::EditorialWindows) -> Self {
            Self {
                budget: Ok(value.budget),
                coverage: Ok(value.coverage),
                inputs: Ok(value.inputs),
                invalid_regions: Ok(value.invalid_regions),
                language: Ok(value.language),
                outline: Ok(value.outline),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                sentences: Ok(value.sentences),
                source_fingerprint: Ok(value.source_fingerprint),
                windows: Ok(value.windows),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialWindowsBudget {
        context_sentences: ::std::result::Result<u64, ::std::string::String>,
        overlap_words: ::std::result::Result<u64, ::std::string::String>,
        target_words: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialWindowsBudget {
        fn default() -> Self {
            Self {
                context_sentences: Err("no value supplied for context_sentences".to_string()),
                overlap_words: Err("no value supplied for overlap_words".to_string()),
                target_words: Err("no value supplied for target_words".to_string()),
            }
        }
    }
    impl EditorialWindowsBudget {
        pub fn context_sentences<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.context_sentences = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for context_sentences: {e}"));
            self
        }
        pub fn overlap_words<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.overlap_words = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for overlap_words: {e}"));
            self
        }
        pub fn target_words<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.target_words = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for target_words: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditorialWindowsBudget> for super::EditorialWindowsBudget {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialWindowsBudget,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                context_sentences: value.context_sentences?,
                overlap_words: value.overlap_words?,
                target_words: value.target_words?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialWindowsBudget> for EditorialWindowsBudget {
        fn from(value: super::EditorialWindowsBudget) -> Self {
            Self {
                context_sentences: Ok(value.context_sentences),
                overlap_words: Ok(value.overlap_words),
                target_words: Ok(value.target_words),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialWindowsInputs {
        index_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        transcript_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialWindowsInputs {
        fn default() -> Self {
            Self {
                index_artifact_id: Err("no value supplied for index_artifact_id".to_string()),
                transcript_artifact_id: Err(
                    "no value supplied for transcript_artifact_id".to_string()
                ),
            }
        }
    }
    impl EditorialWindowsInputs {
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
    impl ::std::convert::TryFrom<EditorialWindowsInputs> for super::EditorialWindowsInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialWindowsInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                index_artifact_id: value.index_artifact_id?,
                transcript_artifact_id: value.transcript_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialWindowsInputs> for EditorialWindowsInputs {
        fn from(value: super::EditorialWindowsInputs) -> Self {
            Self {
                index_artifact_id: Ok(value.index_artifact_id),
                transcript_artifact_id: Ok(value.transcript_artifact_id),
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
    pub struct OutlineLine {
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        first_sentence_index: ::std::result::Result<u64, ::std::string::String>,
        keywords: ::std::result::Result<
            ::std::vec::Vec<super::OutlineLineKeywordsItem>,
            ::std::string::String,
        >,
        sentence_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        topic_index: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for OutlineLine {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_sentence_index: Err("no value supplied for first_sentence_index".to_string()),
                keywords: Err("no value supplied for keywords".to_string()),
                sentence_count: Err("no value supplied for sentence_count".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                topic_index: Err("no value supplied for topic_index".to_string()),
            }
        }
    }
    impl OutlineLine {
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
        pub fn keywords<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::OutlineLineKeywordsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.keywords = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for keywords: {e}"));
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
        pub fn topic_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.topic_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for topic_index: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<OutlineLine> for super::OutlineLine {
        type Error = super::error::ConversionError;
        fn try_from(
            value: OutlineLine,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                first_sentence_index: value.first_sentence_index?,
                keywords: value.keywords?,
                sentence_count: value.sentence_count?,
                start_ticks: value.start_ticks?,
                topic_index: value.topic_index?,
            })
        }
    }
    impl ::std::convert::From<super::OutlineLine> for OutlineLine {
        fn from(value: super::OutlineLine) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                first_sentence_index: Ok(value.first_sentence_index),
                keywords: Ok(value.keywords),
                sentence_count: Ok(value.sentence_count),
                start_ticks: Ok(value.start_ticks),
                topic_index: Ok(value.topic_index),
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
    pub struct Sentence {
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        first_word_index: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        terminator: ::std::result::Result<super::SentenceTerminator, ::std::string::String>,
        text: ::std::result::Result<super::SentenceText, ::std::string::String>,
        word_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for Sentence {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_word_index: Err("no value supplied for first_word_index".to_string()),
                index: Err("no value supplied for index".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                terminator: Err("no value supplied for terminator".to_string()),
                text: Err("no value supplied for text".to_string()),
                word_count: Err("no value supplied for word_count".to_string()),
            }
        }
    }
    impl Sentence {
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
        pub fn terminator<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SentenceTerminator>,
            T::Error: ::std::fmt::Display,
        {
            self.terminator = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for terminator: {e}"));
            self
        }
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SentenceText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
        pub fn word_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.word_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for word_count: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Sentence> for super::Sentence {
        type Error = super::error::ConversionError;
        fn try_from(value: Sentence) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                first_word_index: value.first_word_index?,
                index: value.index?,
                start_ticks: value.start_ticks?,
                terminator: value.terminator?,
                text: value.text?,
                word_count: value.word_count?,
            })
        }
    }
    impl ::std::convert::From<super::Sentence> for Sentence {
        fn from(value: super::Sentence) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                first_word_index: Ok(value.first_word_index),
                index: Ok(value.index),
                start_ticks: Ok(value.start_ticks),
                terminator: Ok(value.terminator),
                text: Ok(value.text),
                word_count: Ok(value.word_count),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Window {
        context_after_sentences: ::std::result::Result<u64, ::std::string::String>,
        context_before_sentences: ::std::result::Result<u64, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        first_sentence_index: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        sentence_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        topic_indexes: ::std::result::Result<::std::vec::Vec<u64>, ::std::string::String>,
        word_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for Window {
        fn default() -> Self {
            Self {
                context_after_sentences: Err(
                    "no value supplied for context_after_sentences".to_string()
                ),
                context_before_sentences: Err(
                    "no value supplied for context_before_sentences".to_string()
                ),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_sentence_index: Err("no value supplied for first_sentence_index".to_string()),
                index: Err("no value supplied for index".to_string()),
                sentence_count: Err("no value supplied for sentence_count".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                topic_indexes: Err("no value supplied for topic_indexes".to_string()),
                word_count: Err("no value supplied for word_count".to_string()),
            }
        }
    }
    impl Window {
        pub fn context_after_sentences<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.context_after_sentences = value.try_into().map_err(|e| {
                format!("error converting supplied value for context_after_sentences: {e}")
            });
            self
        }
        pub fn context_before_sentences<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.context_before_sentences = value.try_into().map_err(|e| {
                format!("error converting supplied value for context_before_sentences: {e}")
            });
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
        pub fn topic_indexes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.topic_indexes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for topic_indexes: {e}"));
            self
        }
        pub fn word_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.word_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for word_count: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Window> for super::Window {
        type Error = super::error::ConversionError;
        fn try_from(value: Window) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                context_after_sentences: value.context_after_sentences?,
                context_before_sentences: value.context_before_sentences?,
                end_ticks: value.end_ticks?,
                first_sentence_index: value.first_sentence_index?,
                index: value.index?,
                sentence_count: value.sentence_count?,
                start_ticks: value.start_ticks?,
                topic_indexes: value.topic_indexes?,
                word_count: value.word_count?,
            })
        }
    }
    impl ::std::convert::From<super::Window> for Window {
        fn from(value: super::Window) -> Self {
            Self {
                context_after_sentences: Ok(value.context_after_sentences),
                context_before_sentences: Ok(value.context_before_sentences),
                end_ticks: Ok(value.end_ticks),
                first_sentence_index: Ok(value.first_sentence_index),
                index: Ok(value.index),
                sentence_count: Ok(value.sentence_count),
                start_ticks: Ok(value.start_ticks),
                topic_indexes: Ok(value.topic_indexes),
                word_count: Ok(value.word_count),
            }
        }
    }
}
