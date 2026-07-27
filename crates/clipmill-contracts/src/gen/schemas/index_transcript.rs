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
#[doc = "`Confidence`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
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
#[doc = "The transcript's own analyzed range, echoed rather than recomputed. Every unit above lies inside it, and an index over a transcript that analyzed nothing is an index with no units and `analyzed` false — not an empty document that reads like a recording with no structure in it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The transcript's own analyzed range, echoed rather than recomputed. Every unit above lies inside it, and an index over a transcript that analyzed nothing is an index with no units and `analyzed` false — not an empty document that reads like a recording with no structure in it.\","]
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
#[doc = "`Edge`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"kind\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"silence\","]
#[doc = "        \"shot_cut\""]
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
pub struct Edge {
    pub end_ticks: u64,
    pub kind: EdgeKind,
    pub start_ticks: u64,
}
impl Edge {
    pub fn builder() -> builder::Edge {
        Default::default()
    }
}
#[doc = "`EdgeKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"silence\","]
#[doc = "    \"shot_cut\""]
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
pub enum EdgeKind {
    #[serde(rename = "silence")]
    Silence,
    #[serde(rename = "shot_cut")]
    ShotCut,
}
impl ::std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Silence => f.write_str("silence"),
            Self::ShotCut => f.write_str("shot_cut"),
        }
    }
}
impl ::std::str::FromStr for EdgeKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "silence" => Ok(Self::Silence),
            "shot_cut" => Ok(Self::ShotCut),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EdgeKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EdgeKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EdgeKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Structure read out of a transcript, so that discovery proposes spans instead of guessing at them (book ch. 14). Two levels only. L1 is what the recording itself states — utterances where voice activity heard a pause, sentences where the recognizer punctuated, and the edges a clip may start or end on. L2 is topics, and it is an approximation stated as one: lexical cohesion over the words, not a model that understands them. There is no L3 here and no open-loop detection; claiming either would be claiming comprehension this stage does not have. Every unit names the words it came from, so any claim can be walked back to the observation behind it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.index.transcript.v1.json\","]
#[doc = "  \"title\": \"IndexTranscript\","]
#[doc = "  \"description\": \"Structure read out of a transcript, so that discovery proposes spans instead of guessing at them (book ch. 14). Two levels only. L1 is what the recording itself states — utterances where voice activity heard a pause, sentences where the recognizer punctuated, and the edges a clip may start or end on. L2 is topics, and it is an approximation stated as one: lexical cohesion over the words, not a model that understands them. There is no L3 here and no open-loop detection; claiming either would be claiming comprehension this stage does not have. Every unit names the words it came from, so any claim can be walked back to the observation behind it.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coverage\","]
#[doc = "    \"edges\","]
#[doc = "    \"inputs\","]
#[doc = "    \"invalid_regions\","]
#[doc = "    \"language\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"segmentation\","]
#[doc = "    \"sentences\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"topics\","]
#[doc = "    \"utterances\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"edges\": {"]
#[doc = "      \"description\": \"Where a clip may start or end without severing anything. A silence is a span a boundary may land anywhere inside; a shot cut is an instant, and carries the same start and end. Both are in one list because the boundary lattice consumes them as one question.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/edge\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"description\": \"What was read. The transcript is the authority for every word index below; shot cuts are optional because a source with no video has none, and their absence is a different document rather than a silently shorter edge list.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"transcript_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"shots_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"transcript_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"invalid_regions\": {"]
#[doc = "      \"description\": \"Carried through from the transcript. An index built over interpolated word timing inherits that timing's uncertainty, and a consumer must not have to open the transcript to discover it.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/invalid_region\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"description\": \"Echoed from the transcript, because L2 is only as good as the stopword list matches it. An index built over a language the list does not cover is still a valid document; it is a weaker one, and a consumer should be able to tell without opening the transcript.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 16,"]
#[doc = "      \"minLength\": 2"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.index.transcript.v1\""]
#[doc = "    },"]
#[doc = "    \"segmentation\": {"]
#[doc = "      \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. Part of the artifact key: a different cutoff is a different reading of the same transcript, not a correction of this one.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"block_sentences\","]
#[doc = "        \"boundary_cutoff\","]
#[doc = "        \"stopwords\","]
#[doc = "        \"utterance_gap_ticks\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"block_sentences\": {"]
#[doc = "          \"description\": \"How many sentences on each side of a gap are compared when looking for a topic boundary. Larger reads more context and finds fewer, broader topics.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"boundary_cutoff\": {"]
#[doc = "          \"description\": \"The boundary threshold sits this many standard deviations below the mean depth score. Depth is high at a boundary, so a larger value lowers the bar and admits more topics; zero keeps only the above-average valleys.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"stopwords\": {"]
#[doc = "          \"description\": \"Identifier of the stopword list used, e.g. 'english-minimal.v1'. Named rather than embedded, because the list is what makes the cohesion score mean anything and a changed list is a changed observation.\","]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"utterance_gap_ticks\": {"]
#[doc = "          \"description\": \"A pause at least this long ends an utterance. Below it, the speaker is still talking.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"sentences\": {"]
#[doc = "      \"description\": \"Ordered by time, and never spanning an utterance. Each says how its end was decided, because a boundary the recognizer punctuated and a boundary that is merely where the speaker stopped are not equally strong evidence, and the boundary optimizer should not treat them as though they were.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/sentence\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"topics\": {"]
#[doc = "      \"description\": \"Contiguous runs of sentences that share vocabulary, covering every sentence exactly once. The honest name for these is 'lexical neighbourhoods': they are where the words changed, which is usually but not always where the subject changed.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/topic\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"utterances\": {"]
#[doc = "      \"description\": \"Runs of speech between the pauses voice activity found. The unit a viewer would call 'a thing someone said', and the one a clip boundary is safest against.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/utterance\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct IndexTranscript {
    pub coverage: Coverage,
    #[doc = "Where a clip may start or end without severing anything. A silence is a span a boundary may land anywhere inside; a shot cut is an instant, and carries the same start and end. Both are in one list because the boundary lattice consumes them as one question."]
    pub edges: ::std::vec::Vec<Edge>,
    pub inputs: IndexTranscriptInputs,
    #[doc = "Carried through from the transcript. An index built over interpolated word timing inherits that timing's uncertainty, and a consumer must not have to open the transcript to discover it."]
    pub invalid_regions: ::std::vec::Vec<InvalidRegion>,
    #[doc = "Echoed from the transcript, because L2 is only as good as the stopword list matches it. An index built over a language the list does not cover is still a valid document; it is a weaker one, and a consumer should be able to tell without opening the transcript."]
    pub language: IndexTranscriptLanguage,
    pub producer: Producer,
    pub schema_version: ::serde_json::Value,
    pub segmentation: IndexTranscriptSegmentation,
    #[doc = "Ordered by time, and never spanning an utterance. Each says how its end was decided, because a boundary the recognizer punctuated and a boundary that is merely where the speaker stopped are not equally strong evidence, and the boundary optimizer should not treat them as though they were."]
    pub sentences: ::std::vec::Vec<Sentence>,
    pub source_fingerprint: Sha256,
    #[doc = "Contiguous runs of sentences that share vocabulary, covering every sentence exactly once. The honest name for these is 'lexical neighbourhoods': they are where the words changed, which is usually but not always where the subject changed."]
    pub topics: ::std::vec::Vec<Topic>,
    #[doc = "Runs of speech between the pauses voice activity found. The unit a viewer would call 'a thing someone said', and the one a clip boundary is safest against."]
    pub utterances: ::std::vec::Vec<Utterance>,
}
impl IndexTranscript {
    pub fn builder() -> builder::IndexTranscript {
        Default::default()
    }
}
#[doc = "What was read. The transcript is the authority for every word index below; shot cuts are optional because a source with no video has none, and their absence is a different document rather than a silently shorter edge list."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was read. The transcript is the authority for every word index below; shot cuts are optional because a source with no video has none, and their absence is a different document rather than a silently shorter edge list.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"transcript_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"shots_artifact_id\": {"]
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
pub struct IndexTranscriptInputs {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub shots_artifact_id: ::std::option::Option<Sha256>,
    pub transcript_artifact_id: Sha256,
}
impl IndexTranscriptInputs {
    pub fn builder() -> builder::IndexTranscriptInputs {
        Default::default()
    }
}
#[doc = "Echoed from the transcript, because L2 is only as good as the stopword list matches it. An index built over a language the list does not cover is still a valid document; it is a weaker one, and a consumer should be able to tell without opening the transcript."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Echoed from the transcript, because L2 is only as good as the stopword list matches it. An index built over a language the list does not cover is still a valid document; it is a weaker one, and a consumer should be able to tell without opening the transcript.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 16,"]
#[doc = "  \"minLength\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct IndexTranscriptLanguage(::std::string::String);
impl ::std::ops::Deref for IndexTranscriptLanguage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<IndexTranscriptLanguage> for ::std::string::String {
    fn from(value: IndexTranscriptLanguage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for IndexTranscriptLanguage {
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
impl ::std::convert::TryFrom<&str> for IndexTranscriptLanguage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for IndexTranscriptLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for IndexTranscriptLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for IndexTranscriptLanguage {
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
#[doc = "The decision parameters, recorded so a later pass knows what to change rather than guessing. Part of the artifact key: a different cutoff is a different reading of the same transcript, not a correction of this one."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. Part of the artifact key: a different cutoff is a different reading of the same transcript, not a correction of this one.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"block_sentences\","]
#[doc = "    \"boundary_cutoff\","]
#[doc = "    \"stopwords\","]
#[doc = "    \"utterance_gap_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"block_sentences\": {"]
#[doc = "      \"description\": \"How many sentences on each side of a gap are compared when looking for a topic boundary. Larger reads more context and finds fewer, broader topics.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"boundary_cutoff\": {"]
#[doc = "      \"description\": \"The boundary threshold sits this many standard deviations below the mean depth score. Depth is high at a boundary, so a larger value lowers the bar and admits more topics; zero keeps only the above-average valleys.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"stopwords\": {"]
#[doc = "      \"description\": \"Identifier of the stopword list used, e.g. 'english-minimal.v1'. Named rather than embedded, because the list is what makes the cohesion score mean anything and a changed list is a changed observation.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"utterance_gap_ticks\": {"]
#[doc = "      \"description\": \"A pause at least this long ends an utterance. Below it, the speaker is still talking.\","]
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
pub struct IndexTranscriptSegmentation {
    #[doc = "How many sentences on each side of a gap are compared when looking for a topic boundary. Larger reads more context and finds fewer, broader topics."]
    pub block_sentences: ::std::num::NonZeroU64,
    #[doc = "The boundary threshold sits this many standard deviations below the mean depth score. Depth is high at a boundary, so a larger value lowers the bar and admits more topics; zero keeps only the above-average valleys."]
    pub boundary_cutoff: f64,
    #[doc = "Identifier of the stopword list used, e.g. 'english-minimal.v1'. Named rather than embedded, because the list is what makes the cohesion score mean anything and a changed list is a changed observation."]
    pub stopwords: IndexTranscriptSegmentationStopwords,
    #[doc = "A pause at least this long ends an utterance. Below it, the speaker is still talking."]
    pub utterance_gap_ticks: u64,
}
impl IndexTranscriptSegmentation {
    pub fn builder() -> builder::IndexTranscriptSegmentation {
        Default::default()
    }
}
#[doc = "Identifier of the stopword list used, e.g. 'english-minimal.v1'. Named rather than embedded, because the list is what makes the cohesion score mean anything and a changed list is a changed observation."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Identifier of the stopword list used, e.g. 'english-minimal.v1'. Named rather than embedded, because the list is what makes the cohesion score mean anything and a changed list is a changed observation.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct IndexTranscriptSegmentationStopwords(::std::string::String);
impl ::std::ops::Deref for IndexTranscriptSegmentationStopwords {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<IndexTranscriptSegmentationStopwords> for ::std::string::String {
    fn from(value: IndexTranscriptSegmentationStopwords) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for IndexTranscriptSegmentationStopwords {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for IndexTranscriptSegmentationStopwords {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for IndexTranscriptSegmentationStopwords {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for IndexTranscriptSegmentationStopwords {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for IndexTranscriptSegmentationStopwords {
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
#[doc = "`Keyword`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"count\","]
#[doc = "    \"term\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"term\": {"]
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
pub struct Keyword {
    pub count: ::std::num::NonZeroU64,
    pub term: KeywordTerm,
}
impl Keyword {
    pub fn builder() -> builder::Keyword {
        Default::default()
    }
}
#[doc = "`KeywordTerm`"]
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
pub struct KeywordTerm(::std::string::String);
impl ::std::ops::Deref for KeywordTerm {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<KeywordTerm> for ::std::string::String {
    fn from(value: KeywordTerm) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for KeywordTerm {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for KeywordTerm {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for KeywordTerm {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for KeywordTerm {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for KeywordTerm {
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
#[doc = "    \"confidence\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_word_index\","]
#[doc = "    \"index\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"terminator\","]
#[doc = "    \"text\","]
#[doc = "    \"utterance_index\","]
#[doc = "    \"word_count\","]
#[doc = "    \"words_per_minute\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"first_word_index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"terminator\": {"]
#[doc = "      \"description\": \"How this sentence ended. 'punctuation' is the recognizer's own full stop, question mark, or exclamation. 'utterance_end' is a speaker who stopped without one, which is most spontaneous speech. 'coverage_end' is the recording running out mid-sentence, and is the weakest boundary of the three.\","]
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
#[doc = "    \"utterance_index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"word_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"words_per_minute\": {"]
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
pub struct Sentence {
    pub confidence: Confidence,
    pub end_ticks: u64,
    pub first_word_index: u64,
    pub index: u64,
    pub start_ticks: u64,
    #[doc = "How this sentence ended. 'punctuation' is the recognizer's own full stop, question mark, or exclamation. 'utterance_end' is a speaker who stopped without one, which is most spontaneous speech. 'coverage_end' is the recording running out mid-sentence, and is the weakest boundary of the three."]
    pub terminator: SentenceTerminator,
    pub text: SentenceText,
    pub utterance_index: u64,
    pub word_count: ::std::num::NonZeroU64,
    pub words_per_minute: f64,
}
impl Sentence {
    pub fn builder() -> builder::Sentence {
        Default::default()
    }
}
#[doc = "How this sentence ended. 'punctuation' is the recognizer's own full stop, question mark, or exclamation. 'utterance_end' is a speaker who stopped without one, which is most spontaneous speech. 'coverage_end' is the recording running out mid-sentence, and is the weakest boundary of the three."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How this sentence ended. 'punctuation' is the recognizer's own full stop, question mark, or exclamation. 'utterance_end' is a speaker who stopped without one, which is most spontaneous speech. 'coverage_end' is the recording running out mid-sentence, and is the weakest boundary of the three.\","]
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
#[doc = "`Topic`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_sentence_index\","]
#[doc = "    \"index\","]
#[doc = "    \"keywords\","]
#[doc = "    \"opening_depth\","]
#[doc = "    \"sentence_count\","]
#[doc = "    \"start_ticks\""]
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
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"keywords\": {"]
#[doc = "      \"description\": \"The terms this run of sentences uses more than the rest of the recording does. Ordered by weight and then alphabetically, so the list is the same on every machine.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/keyword\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"opening_depth\": {"]
#[doc = "      \"description\": \"How deep the lexical valley was at the boundary that opened this topic, summing the drop from the nearest peak on each side. Zero for the first topic, which nothing opened. This is the number the cutoff was compared against, kept so a re-tune can be reasoned about without recomputing.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"sentence_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
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
pub struct Topic {
    pub end_ticks: u64,
    pub first_sentence_index: u64,
    pub index: u64,
    #[doc = "The terms this run of sentences uses more than the rest of the recording does. Ordered by weight and then alphabetically, so the list is the same on every machine."]
    pub keywords: ::std::vec::Vec<Keyword>,
    #[doc = "How deep the lexical valley was at the boundary that opened this topic, summing the drop from the nearest peak on each side. Zero for the first topic, which nothing opened. This is the number the cutoff was compared against, kept so a re-tune can be reasoned about without recomputing."]
    pub opening_depth: f64,
    pub sentence_count: ::std::num::NonZeroU64,
    pub start_ticks: u64,
}
impl Topic {
    pub fn builder() -> builder::Topic {
        Default::default()
    }
}
#[doc = "`Utterance`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_word_index\","]
#[doc = "    \"index\","]
#[doc = "    \"pause_after_ticks\","]
#[doc = "    \"pause_before_ticks\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"text\","]
#[doc = "    \"word_count\","]
#[doc = "    \"words_per_minute\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"first_word_index\": {"]
#[doc = "      \"description\": \"Index into the transcript's word list. This is the provenance link: every claim here resolves to words somebody measured.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"pause_after_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"pause_before_ticks\": {"]
#[doc = "      \"description\": \"Silence between the previous utterance and this one; the leading silence for the first.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"word_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"words_per_minute\": {"]
#[doc = "      \"description\": \"Speaking rate over this utterance. A rate, not a time quantity: it cannot express a position on a timeline, which is what the integer-tick rule protects.\","]
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
pub struct Utterance {
    pub confidence: Confidence,
    pub end_ticks: u64,
    #[doc = "Index into the transcript's word list. This is the provenance link: every claim here resolves to words somebody measured."]
    pub first_word_index: u64,
    pub index: u64,
    pub pause_after_ticks: u64,
    #[doc = "Silence between the previous utterance and this one; the leading silence for the first."]
    pub pause_before_ticks: u64,
    pub start_ticks: u64,
    pub text: UtteranceText,
    pub word_count: ::std::num::NonZeroU64,
    #[doc = "Speaking rate over this utterance. A rate, not a time quantity: it cannot express a position on a timeline, which is what the integer-tick rule protects."]
    pub words_per_minute: f64,
}
impl Utterance {
    pub fn builder() -> builder::Utterance {
        Default::default()
    }
}
#[doc = "`UtteranceText`"]
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
pub struct UtteranceText(::std::string::String);
impl ::std::ops::Deref for UtteranceText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<UtteranceText> for ::std::string::String {
    fn from(value: UtteranceText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for UtteranceText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for UtteranceText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for UtteranceText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UtteranceText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for UtteranceText {
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
    pub struct Edge {
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        kind: ::std::result::Result<super::EdgeKind, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Edge {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Edge {
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
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EdgeKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
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
    impl ::std::convert::TryFrom<Edge> for super::Edge {
        type Error = super::error::ConversionError;
        fn try_from(value: Edge) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                kind: value.kind?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Edge> for Edge {
        fn from(value: super::Edge) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                kind: Ok(value.kind),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct IndexTranscript {
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        edges: ::std::result::Result<::std::vec::Vec<super::Edge>, ::std::string::String>,
        inputs: ::std::result::Result<super::IndexTranscriptInputs, ::std::string::String>,
        invalid_regions:
            ::std::result::Result<::std::vec::Vec<super::InvalidRegion>, ::std::string::String>,
        language: ::std::result::Result<super::IndexTranscriptLanguage, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        segmentation:
            ::std::result::Result<super::IndexTranscriptSegmentation, ::std::string::String>,
        sentences: ::std::result::Result<::std::vec::Vec<super::Sentence>, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        topics: ::std::result::Result<::std::vec::Vec<super::Topic>, ::std::string::String>,
        utterances: ::std::result::Result<::std::vec::Vec<super::Utterance>, ::std::string::String>,
    }
    impl ::std::default::Default for IndexTranscript {
        fn default() -> Self {
            Self {
                coverage: Err("no value supplied for coverage".to_string()),
                edges: Err("no value supplied for edges".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                invalid_regions: Err("no value supplied for invalid_regions".to_string()),
                language: Err("no value supplied for language".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                segmentation: Err("no value supplied for segmentation".to_string()),
                sentences: Err("no value supplied for sentences".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                topics: Err("no value supplied for topics".to_string()),
                utterances: Err("no value supplied for utterances".to_string()),
            }
        }
    }
    impl IndexTranscript {
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
        pub fn edges<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Edge>>,
            T::Error: ::std::fmt::Display,
        {
            self.edges = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for edges: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::IndexTranscriptInputs>,
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
            T: ::std::convert::TryInto<super::IndexTranscriptLanguage>,
            T::Error: ::std::fmt::Display,
        {
            self.language = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for language: {e}"));
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
        pub fn segmentation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::IndexTranscriptSegmentation>,
            T::Error: ::std::fmt::Display,
        {
            self.segmentation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segmentation: {e}"));
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
        pub fn topics<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Topic>>,
            T::Error: ::std::fmt::Display,
        {
            self.topics = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for topics: {e}"));
            self
        }
        pub fn utterances<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Utterance>>,
            T::Error: ::std::fmt::Display,
        {
            self.utterances = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for utterances: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<IndexTranscript> for super::IndexTranscript {
        type Error = super::error::ConversionError;
        fn try_from(
            value: IndexTranscript,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                coverage: value.coverage?,
                edges: value.edges?,
                inputs: value.inputs?,
                invalid_regions: value.invalid_regions?,
                language: value.language?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                segmentation: value.segmentation?,
                sentences: value.sentences?,
                source_fingerprint: value.source_fingerprint?,
                topics: value.topics?,
                utterances: value.utterances?,
            })
        }
    }
    impl ::std::convert::From<super::IndexTranscript> for IndexTranscript {
        fn from(value: super::IndexTranscript) -> Self {
            Self {
                coverage: Ok(value.coverage),
                edges: Ok(value.edges),
                inputs: Ok(value.inputs),
                invalid_regions: Ok(value.invalid_regions),
                language: Ok(value.language),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                segmentation: Ok(value.segmentation),
                sentences: Ok(value.sentences),
                source_fingerprint: Ok(value.source_fingerprint),
                topics: Ok(value.topics),
                utterances: Ok(value.utterances),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct IndexTranscriptInputs {
        shots_artifact_id:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        transcript_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for IndexTranscriptInputs {
        fn default() -> Self {
            Self {
                shots_artifact_id: Ok(Default::default()),
                transcript_artifact_id: Err(
                    "no value supplied for transcript_artifact_id".to_string()
                ),
            }
        }
    }
    impl IndexTranscriptInputs {
        pub fn shots_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.shots_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shots_artifact_id: {e}"));
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
    impl ::std::convert::TryFrom<IndexTranscriptInputs> for super::IndexTranscriptInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: IndexTranscriptInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                shots_artifact_id: value.shots_artifact_id?,
                transcript_artifact_id: value.transcript_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::IndexTranscriptInputs> for IndexTranscriptInputs {
        fn from(value: super::IndexTranscriptInputs) -> Self {
            Self {
                shots_artifact_id: Ok(value.shots_artifact_id),
                transcript_artifact_id: Ok(value.transcript_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct IndexTranscriptSegmentation {
        block_sentences: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        boundary_cutoff: ::std::result::Result<f64, ::std::string::String>,
        stopwords: ::std::result::Result<
            super::IndexTranscriptSegmentationStopwords,
            ::std::string::String,
        >,
        utterance_gap_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for IndexTranscriptSegmentation {
        fn default() -> Self {
            Self {
                block_sentences: Err("no value supplied for block_sentences".to_string()),
                boundary_cutoff: Err("no value supplied for boundary_cutoff".to_string()),
                stopwords: Err("no value supplied for stopwords".to_string()),
                utterance_gap_ticks: Err("no value supplied for utterance_gap_ticks".to_string()),
            }
        }
    }
    impl IndexTranscriptSegmentation {
        pub fn block_sentences<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.block_sentences = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for block_sentences: {e}"));
            self
        }
        pub fn boundary_cutoff<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.boundary_cutoff = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for boundary_cutoff: {e}"));
            self
        }
        pub fn stopwords<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::IndexTranscriptSegmentationStopwords>,
            T::Error: ::std::fmt::Display,
        {
            self.stopwords = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stopwords: {e}"));
            self
        }
        pub fn utterance_gap_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.utterance_gap_ticks = value.try_into().map_err(|e| {
                format!("error converting supplied value for utterance_gap_ticks: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<IndexTranscriptSegmentation> for super::IndexTranscriptSegmentation {
        type Error = super::error::ConversionError;
        fn try_from(
            value: IndexTranscriptSegmentation,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                block_sentences: value.block_sentences?,
                boundary_cutoff: value.boundary_cutoff?,
                stopwords: value.stopwords?,
                utterance_gap_ticks: value.utterance_gap_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::IndexTranscriptSegmentation> for IndexTranscriptSegmentation {
        fn from(value: super::IndexTranscriptSegmentation) -> Self {
            Self {
                block_sentences: Ok(value.block_sentences),
                boundary_cutoff: Ok(value.boundary_cutoff),
                stopwords: Ok(value.stopwords),
                utterance_gap_ticks: Ok(value.utterance_gap_ticks),
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
    pub struct Keyword {
        count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        term: ::std::result::Result<super::KeywordTerm, ::std::string::String>,
    }
    impl ::std::default::Default for Keyword {
        fn default() -> Self {
            Self {
                count: Err("no value supplied for count".to_string()),
                term: Err("no value supplied for term".to_string()),
            }
        }
    }
    impl Keyword {
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
        pub fn term<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::KeywordTerm>,
            T::Error: ::std::fmt::Display,
        {
            self.term = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for term: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Keyword> for super::Keyword {
        type Error = super::error::ConversionError;
        fn try_from(value: Keyword) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                count: value.count?,
                term: value.term?,
            })
        }
    }
    impl ::std::convert::From<super::Keyword> for Keyword {
        fn from(value: super::Keyword) -> Self {
            Self {
                count: Ok(value.count),
                term: Ok(value.term),
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
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        first_word_index: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        terminator: ::std::result::Result<super::SentenceTerminator, ::std::string::String>,
        text: ::std::result::Result<super::SentenceText, ::std::string::String>,
        utterance_index: ::std::result::Result<u64, ::std::string::String>,
        word_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        words_per_minute: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Sentence {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_word_index: Err("no value supplied for first_word_index".to_string()),
                index: Err("no value supplied for index".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                terminator: Err("no value supplied for terminator".to_string()),
                text: Err("no value supplied for text".to_string()),
                utterance_index: Err("no value supplied for utterance_index".to_string()),
                word_count: Err("no value supplied for word_count".to_string()),
                words_per_minute: Err("no value supplied for words_per_minute".to_string()),
            }
        }
    }
    impl Sentence {
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
        pub fn utterance_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.utterance_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for utterance_index: {e}"));
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
        pub fn words_per_minute<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.words_per_minute = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for words_per_minute: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Sentence> for super::Sentence {
        type Error = super::error::ConversionError;
        fn try_from(value: Sentence) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                end_ticks: value.end_ticks?,
                first_word_index: value.first_word_index?,
                index: value.index?,
                start_ticks: value.start_ticks?,
                terminator: value.terminator?,
                text: value.text?,
                utterance_index: value.utterance_index?,
                word_count: value.word_count?,
                words_per_minute: value.words_per_minute?,
            })
        }
    }
    impl ::std::convert::From<super::Sentence> for Sentence {
        fn from(value: super::Sentence) -> Self {
            Self {
                confidence: Ok(value.confidence),
                end_ticks: Ok(value.end_ticks),
                first_word_index: Ok(value.first_word_index),
                index: Ok(value.index),
                start_ticks: Ok(value.start_ticks),
                terminator: Ok(value.terminator),
                text: Ok(value.text),
                utterance_index: Ok(value.utterance_index),
                word_count: Ok(value.word_count),
                words_per_minute: Ok(value.words_per_minute),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Topic {
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        first_sentence_index: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        keywords: ::std::result::Result<::std::vec::Vec<super::Keyword>, ::std::string::String>,
        opening_depth: ::std::result::Result<f64, ::std::string::String>,
        sentence_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Topic {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_sentence_index: Err("no value supplied for first_sentence_index".to_string()),
                index: Err("no value supplied for index".to_string()),
                keywords: Err("no value supplied for keywords".to_string()),
                opening_depth: Err("no value supplied for opening_depth".to_string()),
                sentence_count: Err("no value supplied for sentence_count".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Topic {
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
        pub fn keywords<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Keyword>>,
            T::Error: ::std::fmt::Display,
        {
            self.keywords = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for keywords: {e}"));
            self
        }
        pub fn opening_depth<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.opening_depth = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for opening_depth: {e}"));
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
    }
    impl ::std::convert::TryFrom<Topic> for super::Topic {
        type Error = super::error::ConversionError;
        fn try_from(value: Topic) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                first_sentence_index: value.first_sentence_index?,
                index: value.index?,
                keywords: value.keywords?,
                opening_depth: value.opening_depth?,
                sentence_count: value.sentence_count?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Topic> for Topic {
        fn from(value: super::Topic) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                first_sentence_index: Ok(value.first_sentence_index),
                index: Ok(value.index),
                keywords: Ok(value.keywords),
                opening_depth: Ok(value.opening_depth),
                sentence_count: Ok(value.sentence_count),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Utterance {
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        first_word_index: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        pause_after_ticks: ::std::result::Result<u64, ::std::string::String>,
        pause_before_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        text: ::std::result::Result<super::UtteranceText, ::std::string::String>,
        word_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        words_per_minute: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Utterance {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_word_index: Err("no value supplied for first_word_index".to_string()),
                index: Err("no value supplied for index".to_string()),
                pause_after_ticks: Err("no value supplied for pause_after_ticks".to_string()),
                pause_before_ticks: Err("no value supplied for pause_before_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                text: Err("no value supplied for text".to_string()),
                word_count: Err("no value supplied for word_count".to_string()),
                words_per_minute: Err("no value supplied for words_per_minute".to_string()),
            }
        }
    }
    impl Utterance {
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
        pub fn pause_after_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.pause_after_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for pause_after_ticks: {e}"));
            self
        }
        pub fn pause_before_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.pause_before_ticks = value.try_into().map_err(|e| {
                format!("error converting supplied value for pause_before_ticks: {e}")
            });
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
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UtteranceText>,
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
        pub fn words_per_minute<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.words_per_minute = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for words_per_minute: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Utterance> for super::Utterance {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Utterance,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                end_ticks: value.end_ticks?,
                first_word_index: value.first_word_index?,
                index: value.index?,
                pause_after_ticks: value.pause_after_ticks?,
                pause_before_ticks: value.pause_before_ticks?,
                start_ticks: value.start_ticks?,
                text: value.text?,
                word_count: value.word_count?,
                words_per_minute: value.words_per_minute?,
            })
        }
    }
    impl ::std::convert::From<super::Utterance> for Utterance {
        fn from(value: super::Utterance) -> Self {
            Self {
                confidence: Ok(value.confidence),
                end_ticks: Ok(value.end_ticks),
                first_word_index: Ok(value.first_word_index),
                index: Ok(value.index),
                pause_after_ticks: Ok(value.pause_after_ticks),
                pause_before_ticks: Ok(value.pause_before_ticks),
                start_ticks: Ok(value.start_ticks),
                text: Ok(value.text),
                word_count: Ok(value.word_count),
                words_per_minute: Ok(value.words_per_minute),
            }
        }
    }
}
