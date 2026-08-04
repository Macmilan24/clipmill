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
#[doc = "What an editor says is worth clipping in one recording (book ch. 22). Ground truth here is plural by construction: a recording has moment *sets* rather than one right answer, each moment carries the alternative starts and ends that would also have been acceptable, and disagreement between annotators is kept rather than resolved — a moment two of three editors accept is a different fact from unanimity, and a scorer that averaged them away would report a number nobody could interpret. One document is one annotator's opinion of one source; agreement is computed across documents, never inside one."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.eval_annotation.v1.json\","]
#[doc = "  \"title\": \"EvalAnnotation\","]
#[doc = "  \"description\": \"What an editor says is worth clipping in one recording (book ch. 22). Ground truth here is plural by construction: a recording has moment *sets* rather than one right answer, each moment carries the alternative starts and ends that would also have been acceptable, and disagreement between annotators is kept rather than resolved — a moment two of three editors accept is a different fact from unanimity, and a scorer that averaged them away would report a number nobody could interpret. One document is one annotator's opinion of one source; agreement is computed across documents, never inside one.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"annotated_unix_millis\","]
#[doc = "    \"annotator_id\","]
#[doc = "    \"moments\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"timebase\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"annotated_unix_millis\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"annotator_id\": {"]
#[doc = "      \"description\": \"Who said this. Kept because disagreement is signal: the same recording annotated by two people is two documents, and the difference between them is what calibration reads.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"exclusions\": {"]
#[doc = "      \"description\": \"Spans that must never be offered, with the reason. Separate from simply not being a moment: not-a-moment is an absence, an exclusion is a claim.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/exclusion\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"item_id\": {"]
#[doc = "      \"description\": \"The corpus item, when the recording came from one. Absent for a source annotated outside a corpus.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"moments\": {"]
#[doc = "      \"description\": \"Every span this annotator would accept as a clip. An empty list is a real answer — a recording with nothing worth clipping is a fact a recall number has to be able to represent — so it is allowed and is not the same as an unannotated item.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/moment\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"notes\": {"]
#[doc = "      \"description\": \"Anything about the recording as a whole that shaped the annotation.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.eval_annotation.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"description\": \"The recording this is about, by content. A path would stop being true the moment the file moved.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"timebase\": {"]
#[doc = "      \"description\": \"The rational clock every tick in this document is counted against. Stated rather than assumed, so an annotation and a candidate set cannot silently disagree about what a tick is (decision D06).\","]
#[doc = "      \"$ref\": \"#/$defs/timebase\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EvalAnnotation {
    pub annotated_unix_millis: u64,
    #[doc = "Who said this. Kept because disagreement is signal: the same recording annotated by two people is two documents, and the difference between them is what calibration reads."]
    pub annotator_id: EvalAnnotationAnnotatorId,
    #[doc = "Spans that must never be offered, with the reason. Separate from simply not being a moment: not-a-moment is an absence, an exclusion is a claim."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub exclusions: ::std::vec::Vec<Exclusion>,
    #[doc = "The corpus item, when the recording came from one. Absent for a source annotated outside a corpus."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub item_id: ::std::option::Option<EvalAnnotationItemId>,
    #[doc = "Every span this annotator would accept as a clip. An empty list is a real answer — a recording with nothing worth clipping is a fact a recall number has to be able to represent — so it is allowed and is not the same as an unannotated item."]
    pub moments: ::std::vec::Vec<Moment>,
    #[doc = "Anything about the recording as a whole that shaped the annotation."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub notes: ::std::option::Option<::std::string::String>,
    pub schema_version: ::serde_json::Value,
    #[doc = "The recording this is about, by content. A path would stop being true the moment the file moved."]
    pub source_fingerprint: Sha256,
    #[doc = "The rational clock every tick in this document is counted against. Stated rather than assumed, so an annotation and a candidate set cannot silently disagree about what a tick is (decision D06)."]
    pub timebase: Timebase,
}
impl EvalAnnotation {
    pub fn builder() -> builder::EvalAnnotation {
        Default::default()
    }
}
#[doc = "Who said this. Kept because disagreement is signal: the same recording annotated by two people is two documents, and the difference between them is what calibration reads."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Who said this. Kept because disagreement is signal: the same recording annotated by two people is two documents, and the difference between them is what calibration reads.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EvalAnnotationAnnotatorId(::std::string::String);
impl ::std::ops::Deref for EvalAnnotationAnnotatorId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EvalAnnotationAnnotatorId> for ::std::string::String {
    fn from(value: EvalAnnotationAnnotatorId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EvalAnnotationAnnotatorId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EvalAnnotationAnnotatorId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EvalAnnotationAnnotatorId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EvalAnnotationAnnotatorId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EvalAnnotationAnnotatorId {
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
#[doc = "The corpus item, when the recording came from one. Absent for a source annotated outside a corpus."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The corpus item, when the recording came from one. Absent for a source annotated outside a corpus.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EvalAnnotationItemId(::std::string::String);
impl ::std::ops::Deref for EvalAnnotationItemId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EvalAnnotationItemId> for ::std::string::String {
    fn from(value: EvalAnnotationItemId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EvalAnnotationItemId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EvalAnnotationItemId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EvalAnnotationItemId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EvalAnnotationItemId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EvalAnnotationItemId {
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
#[doc = "`Exclusion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"reason\","]
#[doc = "    \"span\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"notes\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"description\": \"Why this span may not be offered. A closed vocabulary, because an exclusion that could say anything is an exclusion nothing downstream can act on.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"rights\","]
#[doc = "        \"personal_information\","]
#[doc = "        \"misleading_out_of_context\","]
#[doc = "        \"poor_audio\","]
#[doc = "        \"poor_picture\","]
#[doc = "        \"off_topic\","]
#[doc = "        \"duplicate_of_a_moment\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"span\": {"]
#[doc = "      \"$ref\": \"#/$defs/interval\""]
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
    pub notes: ::std::option::Option<::std::string::String>,
    #[doc = "Why this span may not be offered. A closed vocabulary, because an exclusion that could say anything is an exclusion nothing downstream can act on."]
    pub reason: ExclusionReason,
    pub span: Interval,
}
impl Exclusion {
    pub fn builder() -> builder::Exclusion {
        Default::default()
    }
}
#[doc = "Why this span may not be offered. A closed vocabulary, because an exclusion that could say anything is an exclusion nothing downstream can act on."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Why this span may not be offered. A closed vocabulary, because an exclusion that could say anything is an exclusion nothing downstream can act on.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"rights\","]
#[doc = "    \"personal_information\","]
#[doc = "    \"misleading_out_of_context\","]
#[doc = "    \"poor_audio\","]
#[doc = "    \"poor_picture\","]
#[doc = "    \"off_topic\","]
#[doc = "    \"duplicate_of_a_moment\""]
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
    #[serde(rename = "rights")]
    Rights,
    #[serde(rename = "personal_information")]
    PersonalInformation,
    #[serde(rename = "misleading_out_of_context")]
    MisleadingOutOfContext,
    #[serde(rename = "poor_audio")]
    PoorAudio,
    #[serde(rename = "poor_picture")]
    PoorPicture,
    #[serde(rename = "off_topic")]
    OffTopic,
    #[serde(rename = "duplicate_of_a_moment")]
    DuplicateOfAMoment,
}
impl ::std::fmt::Display for ExclusionReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Rights => f.write_str("rights"),
            Self::PersonalInformation => f.write_str("personal_information"),
            Self::MisleadingOutOfContext => f.write_str("misleading_out_of_context"),
            Self::PoorAudio => f.write_str("poor_audio"),
            Self::PoorPicture => f.write_str("poor_picture"),
            Self::OffTopic => f.write_str("off_topic"),
            Self::DuplicateOfAMoment => f.write_str("duplicate_of_a_moment"),
        }
    }
}
impl ::std::str::FromStr for ExclusionReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "rights" => Ok(Self::Rights),
            "personal_information" => Ok(Self::PersonalInformation),
            "misleading_out_of_context" => Ok(Self::MisleadingOutOfContext),
            "poor_audio" => Ok(Self::PoorAudio),
            "poor_picture" => Ok(Self::PoorPicture),
            "off_topic" => Ok(Self::OffTopic),
            "duplicate_of_a_moment" => Ok(Self::DuplicateOfAMoment),
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
#[doc = "`Moment`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"importance\","]
#[doc = "    \"moment_id\","]
#[doc = "    \"span\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"alternatives\": {"]
#[doc = "      \"description\": \"Other starts and ends the annotator would also have accepted for this same moment. These are what boundary-edge error is measured against: a cut that landed on an alternative is not an error, and scoring it as one would punish the system for agreeing with the editor.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/interval\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"hook_ticks\": {"]
#[doc = "      \"description\": \"Where the thing that makes somebody keep watching happens. Absent when the annotator could not point at one, which is different from it being at zero.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"importance\": {"]
#[doc = "      \"description\": \"How much it costs to miss this one. `essential` is a moment whose absence makes the result wrong; `strong` is one a good result contains; `acceptable` is one a result may contain without being better for it. Recall is reported per grade as well as overall, because a system that finds every acceptable moment and misses every essential one has a good aggregate number and is useless.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"essential\","]
#[doc = "        \"strong\","]
#[doc = "        \"acceptable\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"moment_id\": {"]
#[doc = "      \"description\": \"Stable within this document. Two annotators naming the same span do not have to agree on an id — agreement is computed from the spans, not the labels.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"notes\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"payoff_ticks\": {"]
#[doc = "      \"description\": \"Where the moment lands. Absent for the same reason as the hook.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"required_context\": {"]
#[doc = "      \"description\": \"What a viewer has to have heard for this to make sense, in the annotator's words. A moment that needs context the clip cannot carry is a moment worth finding and worth cutting differently.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"span\": {"]
#[doc = "      \"description\": \"The annotator's preferred cut. Scoring uses this as the moment's identity; the alternatives widen what counts as having found it.\","]
#[doc = "      \"$ref\": \"#/$defs/interval\""]
#[doc = "    },"]
#[doc = "    \"topic\": {"]
#[doc = "      \"description\": \"Free-text label, used for topic coverage rather than for scoring. Not a controlled vocabulary: imposing one before ten recordings have been annotated would be inventing a taxonomy from nothing.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Moment {
    #[doc = "Other starts and ends the annotator would also have accepted for this same moment. These are what boundary-edge error is measured against: a cut that landed on an alternative is not an error, and scoring it as one would punish the system for agreeing with the editor."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub alternatives: ::std::vec::Vec<Interval>,
    #[doc = "Where the thing that makes somebody keep watching happens. Absent when the annotator could not point at one, which is different from it being at zero."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub hook_ticks: ::std::option::Option<u64>,
    #[doc = "How much it costs to miss this one. `essential` is a moment whose absence makes the result wrong; `strong` is one a good result contains; `acceptable` is one a result may contain without being better for it. Recall is reported per grade as well as overall, because a system that finds every acceptable moment and misses every essential one has a good aggregate number and is useless."]
    pub importance: MomentImportance,
    #[doc = "Stable within this document. Two annotators naming the same span do not have to agree on an id — agreement is computed from the spans, not the labels."]
    pub moment_id: MomentMomentId,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub notes: ::std::option::Option<::std::string::String>,
    #[doc = "Where the moment lands. Absent for the same reason as the hook."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub payoff_ticks: ::std::option::Option<u64>,
    #[doc = "What a viewer has to have heard for this to make sense, in the annotator's words. A moment that needs context the clip cannot carry is a moment worth finding and worth cutting differently."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub required_context: ::std::option::Option<::std::string::String>,
    #[doc = "The annotator's preferred cut. Scoring uses this as the moment's identity; the alternatives widen what counts as having found it."]
    pub span: Interval,
    #[doc = "Free-text label, used for topic coverage rather than for scoring. Not a controlled vocabulary: imposing one before ten recordings have been annotated would be inventing a taxonomy from nothing."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub topic: ::std::option::Option<::std::string::String>,
}
impl Moment {
    pub fn builder() -> builder::Moment {
        Default::default()
    }
}
#[doc = "How much it costs to miss this one. `essential` is a moment whose absence makes the result wrong; `strong` is one a good result contains; `acceptable` is one a result may contain without being better for it. Recall is reported per grade as well as overall, because a system that finds every acceptable moment and misses every essential one has a good aggregate number and is useless."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How much it costs to miss this one. `essential` is a moment whose absence makes the result wrong; `strong` is one a good result contains; `acceptable` is one a result may contain without being better for it. Recall is reported per grade as well as overall, because a system that finds every acceptable moment and misses every essential one has a good aggregate number and is useless.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"essential\","]
#[doc = "    \"strong\","]
#[doc = "    \"acceptable\""]
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
pub enum MomentImportance {
    #[serde(rename = "essential")]
    Essential,
    #[serde(rename = "strong")]
    Strong,
    #[serde(rename = "acceptable")]
    Acceptable,
}
impl ::std::fmt::Display for MomentImportance {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Essential => f.write_str("essential"),
            Self::Strong => f.write_str("strong"),
            Self::Acceptable => f.write_str("acceptable"),
        }
    }
}
impl ::std::str::FromStr for MomentImportance {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "essential" => Ok(Self::Essential),
            "strong" => Ok(Self::Strong),
            "acceptable" => Ok(Self::Acceptable),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MomentImportance {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MomentImportance {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MomentImportance {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Stable within this document. Two annotators naming the same span do not have to agree on an id — agreement is computed from the spans, not the labels."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Stable within this document. Two annotators naming the same span do not have to agree on an id — agreement is computed from the spans, not the labels.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MomentMomentId(::std::string::String);
impl ::std::ops::Deref for MomentMomentId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MomentMomentId> for ::std::string::String {
    fn from(value: MomentMomentId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MomentMomentId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MomentMomentId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MomentMomentId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MomentMomentId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MomentMomentId {
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
#[doc = "`Timebase`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"den\","]
#[doc = "    \"num\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"den\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"num\": {"]
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
pub struct Timebase {
    pub den: ::std::num::NonZeroU64,
    pub num: ::std::num::NonZeroU64,
}
impl Timebase {
    pub fn builder() -> builder::Timebase {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct EvalAnnotation {
        annotated_unix_millis: ::std::result::Result<u64, ::std::string::String>,
        annotator_id:
            ::std::result::Result<super::EvalAnnotationAnnotatorId, ::std::string::String>,
        exclusions: ::std::result::Result<::std::vec::Vec<super::Exclusion>, ::std::string::String>,
        item_id: ::std::result::Result<
            ::std::option::Option<super::EvalAnnotationItemId>,
            ::std::string::String,
        >,
        moments: ::std::result::Result<::std::vec::Vec<super::Moment>, ::std::string::String>,
        notes: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        timebase: ::std::result::Result<super::Timebase, ::std::string::String>,
    }
    impl ::std::default::Default for EvalAnnotation {
        fn default() -> Self {
            Self {
                annotated_unix_millis: Err(
                    "no value supplied for annotated_unix_millis".to_string()
                ),
                annotator_id: Err("no value supplied for annotator_id".to_string()),
                exclusions: Ok(Default::default()),
                item_id: Ok(Default::default()),
                moments: Err("no value supplied for moments".to_string()),
                notes: Ok(Default::default()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                timebase: Err("no value supplied for timebase".to_string()),
            }
        }
    }
    impl EvalAnnotation {
        pub fn annotated_unix_millis<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.annotated_unix_millis = value.try_into().map_err(|e| {
                format!("error converting supplied value for annotated_unix_millis: {e}")
            });
            self
        }
        pub fn annotator_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EvalAnnotationAnnotatorId>,
            T::Error: ::std::fmt::Display,
        {
            self.annotator_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for annotator_id: {e}"));
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
        pub fn item_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EvalAnnotationItemId>>,
            T::Error: ::std::fmt::Display,
        {
            self.item_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for item_id: {e}"));
            self
        }
        pub fn moments<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Moment>>,
            T::Error: ::std::fmt::Display,
        {
            self.moments = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for moments: {e}"));
            self
        }
        pub fn notes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.notes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for notes: {e}"));
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
        pub fn timebase<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Timebase>,
            T::Error: ::std::fmt::Display,
        {
            self.timebase = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timebase: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EvalAnnotation> for super::EvalAnnotation {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EvalAnnotation,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                annotated_unix_millis: value.annotated_unix_millis?,
                annotator_id: value.annotator_id?,
                exclusions: value.exclusions?,
                item_id: value.item_id?,
                moments: value.moments?,
                notes: value.notes?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                timebase: value.timebase?,
            })
        }
    }
    impl ::std::convert::From<super::EvalAnnotation> for EvalAnnotation {
        fn from(value: super::EvalAnnotation) -> Self {
            Self {
                annotated_unix_millis: Ok(value.annotated_unix_millis),
                annotator_id: Ok(value.annotator_id),
                exclusions: Ok(value.exclusions),
                item_id: Ok(value.item_id),
                moments: Ok(value.moments),
                notes: Ok(value.notes),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                timebase: Ok(value.timebase),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Exclusion {
        notes: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        reason: ::std::result::Result<super::ExclusionReason, ::std::string::String>,
        span: ::std::result::Result<super::Interval, ::std::string::String>,
    }
    impl ::std::default::Default for Exclusion {
        fn default() -> Self {
            Self {
                notes: Ok(Default::default()),
                reason: Err("no value supplied for reason".to_string()),
                span: Err("no value supplied for span".to_string()),
            }
        }
    }
    impl Exclusion {
        pub fn notes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.notes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for notes: {e}"));
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
        pub fn span<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Interval>,
            T::Error: ::std::fmt::Display,
        {
            self.span = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for span: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Exclusion> for super::Exclusion {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Exclusion,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                notes: value.notes?,
                reason: value.reason?,
                span: value.span?,
            })
        }
    }
    impl ::std::convert::From<super::Exclusion> for Exclusion {
        fn from(value: super::Exclusion) -> Self {
            Self {
                notes: Ok(value.notes),
                reason: Ok(value.reason),
                span: Ok(value.span),
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
    pub struct Moment {
        alternatives:
            ::std::result::Result<::std::vec::Vec<super::Interval>, ::std::string::String>,
        hook_ticks: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        importance: ::std::result::Result<super::MomentImportance, ::std::string::String>,
        moment_id: ::std::result::Result<super::MomentMomentId, ::std::string::String>,
        notes: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        payoff_ticks: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        required_context: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        span: ::std::result::Result<super::Interval, ::std::string::String>,
        topic: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Moment {
        fn default() -> Self {
            Self {
                alternatives: Ok(Default::default()),
                hook_ticks: Ok(Default::default()),
                importance: Err("no value supplied for importance".to_string()),
                moment_id: Err("no value supplied for moment_id".to_string()),
                notes: Ok(Default::default()),
                payoff_ticks: Ok(Default::default()),
                required_context: Ok(Default::default()),
                span: Err("no value supplied for span".to_string()),
                topic: Ok(Default::default()),
            }
        }
    }
    impl Moment {
        pub fn alternatives<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Interval>>,
            T::Error: ::std::fmt::Display,
        {
            self.alternatives = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for alternatives: {e}"));
            self
        }
        pub fn hook_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.hook_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hook_ticks: {e}"));
            self
        }
        pub fn importance<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MomentImportance>,
            T::Error: ::std::fmt::Display,
        {
            self.importance = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for importance: {e}"));
            self
        }
        pub fn moment_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MomentMomentId>,
            T::Error: ::std::fmt::Display,
        {
            self.moment_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for moment_id: {e}"));
            self
        }
        pub fn notes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.notes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for notes: {e}"));
            self
        }
        pub fn payoff_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.payoff_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payoff_ticks: {e}"));
            self
        }
        pub fn required_context<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.required_context = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for required_context: {e}"));
            self
        }
        pub fn span<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Interval>,
            T::Error: ::std::fmt::Display,
        {
            self.span = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for span: {e}"));
            self
        }
        pub fn topic<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.topic = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for topic: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Moment> for super::Moment {
        type Error = super::error::ConversionError;
        fn try_from(value: Moment) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                alternatives: value.alternatives?,
                hook_ticks: value.hook_ticks?,
                importance: value.importance?,
                moment_id: value.moment_id?,
                notes: value.notes?,
                payoff_ticks: value.payoff_ticks?,
                required_context: value.required_context?,
                span: value.span?,
                topic: value.topic?,
            })
        }
    }
    impl ::std::convert::From<super::Moment> for Moment {
        fn from(value: super::Moment) -> Self {
            Self {
                alternatives: Ok(value.alternatives),
                hook_ticks: Ok(value.hook_ticks),
                importance: Ok(value.importance),
                moment_id: Ok(value.moment_id),
                notes: Ok(value.notes),
                payoff_ticks: Ok(value.payoff_ticks),
                required_context: Ok(value.required_context),
                span: Ok(value.span),
                topic: Ok(value.topic),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Timebase {
        den: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        num: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for Timebase {
        fn default() -> Self {
            Self {
                den: Err("no value supplied for den".to_string()),
                num: Err("no value supplied for num".to_string()),
            }
        }
    }
    impl Timebase {
        pub fn den<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.den = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for den: {e}"));
            self
        }
        pub fn num<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.num = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for num: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Timebase> for super::Timebase {
        type Error = super::error::ConversionError;
        fn try_from(value: Timebase) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                den: value.den?,
                num: value.num?,
            })
        }
    }
    impl ::std::convert::From<super::Timebase> for Timebase {
        fn from(value: super::Timebase) -> Self {
            Self {
                den: Ok(value.den),
                num: Ok(value.num),
            }
        }
    }
}
