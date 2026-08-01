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
#[doc = "Captions derived from aligned words, as their own track rather than a string baked into a filter (book ch. 19). One document, two rendering intents. The accessibility intent is the conservative profile every sidecar must meet; the burn-in intent may run hot, a few words at a time, because that is what a viewer scrolling with the sound off is reading. They are two groupings of the same tokens and never two sets of words: divergence between what a viewer reads and what a deaf viewer reads is the failure this shape exists to make impossible. Line breaks are decided here and stored here, because a break re-decided at render time is a break that can differ between the preview and the file."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.captions.cues.v1.json\","]
#[doc = "  \"title\": \"CaptionCues\","]
#[doc = "  \"description\": \"Captions derived from aligned words, as their own track rather than a string baked into a filter (book ch. 19). One document, two rendering intents. The accessibility intent is the conservative profile every sidecar must meet; the burn-in intent may run hot, a few words at a time, because that is what a viewer scrolling with the sound off is reading. They are two groupings of the same tokens and never two sets of words: divergence between what a viewer reads and what a deaf viewer reads is the failure this shape exists to make impossible. Line breaks are decided here and stored here, because a break re-decided at render time is a break that can differ between the preview and the file.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"corrections\","]
#[doc = "    \"coverage\","]
#[doc = "    \"direction\","]
#[doc = "    \"inputs\","]
#[doc = "    \"intents\","]
#[doc = "    \"invalid_regions\","]
#[doc = "    \"language\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"segmentation\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"tokens\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"corrections\": {"]
#[doc = "      \"description\": \"User edits, as an overlay keyed to the token they replace rather than as a rewrite of the token itself. This is what lets a better model re-transcribe a recording and propose updates without erasing a word somebody already fixed: the raw ASR stays underneath, and the overlay is applied on top of whatever it says next time.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/correction\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"direction\": {"]
#[doc = "      \"description\": \"Script direction, carried per document so a translated track is a sibling rather than a schema change.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"ltr\","]
#[doc = "        \"rtl\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"description\": \"What was read. The transcript is the authority for every word index below. The index supplies the sentence terminators a break is worth making at and the topic keywords emphasis is allowed to come from; shot cuts supply the moments a cue may not span. Both are optional, and their absence is a weaker document rather than a silently different one — which is why the segmentation block below records whether each was present.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"transcript_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"index_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"shots_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"transcript_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"intents\": {"]
#[doc = "      \"description\": \"Two groupings of the tokens above. Both are first-class: the accessibility intent is not a degraded burn-in and the burn-in is not an unvalidated accessibility track.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"accessibility\","]
#[doc = "        \"burn_in\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"accessibility\": {"]
#[doc = "          \"$ref\": \"#/$defs/intent\""]
#[doc = "        },"]
#[doc = "        \"burn_in\": {"]
#[doc = "          \"$ref\": \"#/$defs/intent\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"invalid_regions\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/invalid_region\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"description\": \"Echoed from the transcript. Cue rules are language-parameterized and the segmenter never applies English constants to a script they do not describe, so a consumer must be able to see which language's numbers produced these cues without opening the transcript.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 16,"]
#[doc = "      \"minLength\": 2"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.captions.cues.v1\""]
#[doc = "    },"]
#[doc = "    \"segmentation\": {"]
#[doc = "      \"description\": \"Every decision parameter the segmenter was given. Part of the artifact key: different numbers are a different reading of the same words, not a correction of this one.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"emphasis_source\","]
#[doc = "        \"filler_lexicon\","]
#[doc = "        \"had_index\","]
#[doc = "        \"had_shots\","]
#[doc = "        \"span_end_ticks\","]
#[doc = "        \"span_start_ticks\","]
#[doc = "        \"weights\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"emphasis_source\": {"]
#[doc = "          \"description\": \"Where emphasis was allowed to come from. `none` when no index was read, and there is no third option in this build — emphasis is selected from evidence or it is not selected. Decoration is not a source.\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"none\","]
#[doc = "            \"index_keywords\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"filler_lexicon\": {"]
#[doc = "          \"description\": \"Which filler list tagged the tokens. Versioned rather than inlined: the list changes with the language and with what creators actually say, and a cue set should name the one it was read with.\","]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"had_index\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"had_shots\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"span_end_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"span_start_ticks\": {"]
#[doc = "          \"description\": \"The window segmented. A cue may not span the edge of it, for the same reason it may not span a cut: the edge is where the picture changes.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"weights\": {"]
#[doc = "          \"description\": \"What the exact dynamic program was minimizing. Named individually because re-tuning any one of them invalidates exactly the cues it shaped.\","]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"required\": ["]
#[doc = "            \"break_quality\","]
#[doc = "            \"line_balance\","]
#[doc = "            \"orphan\","]
#[doc = "            \"reading_rate\","]
#[doc = "            \"short_cue\""]
#[doc = "          ],"]
#[doc = "          \"properties\": {"]
#[doc = "            \"break_quality\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"line_balance\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"orphan\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"reading_rate\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"short_cue\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            }"]
#[doc = "          },"]
#[doc = "          \"additionalProperties\": false"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"tokens\": {"]
#[doc = "      \"description\": \"Every word in the span, in time order, exactly once. Both intents group these and neither owns them, so a correction applied here is read by the burn-in and the sidecar alike.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/token\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CaptionCues {
    #[doc = "User edits, as an overlay keyed to the token they replace rather than as a rewrite of the token itself. This is what lets a better model re-transcribe a recording and propose updates without erasing a word somebody already fixed: the raw ASR stays underneath, and the overlay is applied on top of whatever it says next time."]
    pub corrections: ::std::vec::Vec<Correction>,
    pub coverage: Coverage,
    #[doc = "Script direction, carried per document so a translated track is a sibling rather than a schema change."]
    pub direction: CaptionCuesDirection,
    pub inputs: CaptionCuesInputs,
    pub intents: CaptionCuesIntents,
    pub invalid_regions: ::std::vec::Vec<InvalidRegion>,
    #[doc = "Echoed from the transcript. Cue rules are language-parameterized and the segmenter never applies English constants to a script they do not describe, so a consumer must be able to see which language's numbers produced these cues without opening the transcript."]
    pub language: CaptionCuesLanguage,
    pub producer: Producer,
    pub schema_version: ::serde_json::Value,
    pub segmentation: CaptionCuesSegmentation,
    pub source_fingerprint: Sha256,
    #[doc = "Every word in the span, in time order, exactly once. Both intents group these and neither owns them, so a correction applied here is read by the burn-in and the sidecar alike."]
    pub tokens: ::std::vec::Vec<Token>,
}
impl CaptionCues {
    pub fn builder() -> builder::CaptionCues {
        Default::default()
    }
}
#[doc = "Script direction, carried per document so a translated track is a sibling rather than a schema change."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Script direction, carried per document so a translated track is a sibling rather than a schema change.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"ltr\","]
#[doc = "    \"rtl\""]
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
pub enum CaptionCuesDirection {
    #[serde(rename = "ltr")]
    Ltr,
    #[serde(rename = "rtl")]
    Rtl,
}
impl ::std::fmt::Display for CaptionCuesDirection {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Ltr => f.write_str("ltr"),
            Self::Rtl => f.write_str("rtl"),
        }
    }
}
impl ::std::str::FromStr for CaptionCuesDirection {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "ltr" => Ok(Self::Ltr),
            "rtl" => Ok(Self::Rtl),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CaptionCuesDirection {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCuesDirection {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCuesDirection {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "What was read. The transcript is the authority for every word index below. The index supplies the sentence terminators a break is worth making at and the topic keywords emphasis is allowed to come from; shot cuts supply the moments a cue may not span. Both are optional, and their absence is a weaker document rather than a silently different one — which is why the segmentation block below records whether each was present."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was read. The transcript is the authority for every word index below. The index supplies the sentence terminators a break is worth making at and the topic keywords emphasis is allowed to come from; shot cuts supply the moments a cue may not span. Both are optional, and their absence is a weaker document rather than a silently different one — which is why the segmentation block below records whether each was present.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"transcript_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"index_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
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
pub struct CaptionCuesInputs {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub index_artifact_id: ::std::option::Option<Sha256>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub shots_artifact_id: ::std::option::Option<Sha256>,
    pub transcript_artifact_id: Sha256,
}
impl CaptionCuesInputs {
    pub fn builder() -> builder::CaptionCuesInputs {
        Default::default()
    }
}
#[doc = "Two groupings of the tokens above. Both are first-class: the accessibility intent is not a degraded burn-in and the burn-in is not an unvalidated accessibility track."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Two groupings of the tokens above. Both are first-class: the accessibility intent is not a degraded burn-in and the burn-in is not an unvalidated accessibility track.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"accessibility\","]
#[doc = "    \"burn_in\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"accessibility\": {"]
#[doc = "      \"$ref\": \"#/$defs/intent\""]
#[doc = "    },"]
#[doc = "    \"burn_in\": {"]
#[doc = "      \"$ref\": \"#/$defs/intent\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CaptionCuesIntents {
    pub accessibility: Intent,
    pub burn_in: Intent,
}
impl CaptionCuesIntents {
    pub fn builder() -> builder::CaptionCuesIntents {
        Default::default()
    }
}
#[doc = "Echoed from the transcript. Cue rules are language-parameterized and the segmenter never applies English constants to a script they do not describe, so a consumer must be able to see which language's numbers produced these cues without opening the transcript."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Echoed from the transcript. Cue rules are language-parameterized and the segmenter never applies English constants to a script they do not describe, so a consumer must be able to see which language's numbers produced these cues without opening the transcript.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 16,"]
#[doc = "  \"minLength\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CaptionCuesLanguage(::std::string::String);
impl ::std::ops::Deref for CaptionCuesLanguage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CaptionCuesLanguage> for ::std::string::String {
    fn from(value: CaptionCuesLanguage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CaptionCuesLanguage {
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
impl ::std::convert::TryFrom<&str> for CaptionCuesLanguage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCuesLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCuesLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CaptionCuesLanguage {
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
#[doc = "Every decision parameter the segmenter was given. Part of the artifact key: different numbers are a different reading of the same words, not a correction of this one."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Every decision parameter the segmenter was given. Part of the artifact key: different numbers are a different reading of the same words, not a correction of this one.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"emphasis_source\","]
#[doc = "    \"filler_lexicon\","]
#[doc = "    \"had_index\","]
#[doc = "    \"had_shots\","]
#[doc = "    \"span_end_ticks\","]
#[doc = "    \"span_start_ticks\","]
#[doc = "    \"weights\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"emphasis_source\": {"]
#[doc = "      \"description\": \"Where emphasis was allowed to come from. `none` when no index was read, and there is no third option in this build — emphasis is selected from evidence or it is not selected. Decoration is not a source.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"none\","]
#[doc = "        \"index_keywords\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"filler_lexicon\": {"]
#[doc = "      \"description\": \"Which filler list tagged the tokens. Versioned rather than inlined: the list changes with the language and with what creators actually say, and a cue set should name the one it was read with.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"had_index\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"had_shots\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"span_end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"span_start_ticks\": {"]
#[doc = "      \"description\": \"The window segmented. A cue may not span the edge of it, for the same reason it may not span a cut: the edge is where the picture changes.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"weights\": {"]
#[doc = "      \"description\": \"What the exact dynamic program was minimizing. Named individually because re-tuning any one of them invalidates exactly the cues it shaped.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"break_quality\","]
#[doc = "        \"line_balance\","]
#[doc = "        \"orphan\","]
#[doc = "        \"reading_rate\","]
#[doc = "        \"short_cue\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"break_quality\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"line_balance\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"orphan\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"reading_rate\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"short_cue\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
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
pub struct CaptionCuesSegmentation {
    #[doc = "Where emphasis was allowed to come from. `none` when no index was read, and there is no third option in this build — emphasis is selected from evidence or it is not selected. Decoration is not a source."]
    pub emphasis_source: CaptionCuesSegmentationEmphasisSource,
    #[doc = "Which filler list tagged the tokens. Versioned rather than inlined: the list changes with the language and with what creators actually say, and a cue set should name the one it was read with."]
    pub filler_lexicon: CaptionCuesSegmentationFillerLexicon,
    pub had_index: bool,
    pub had_shots: bool,
    pub span_end_ticks: ::std::num::NonZeroU64,
    #[doc = "The window segmented. A cue may not span the edge of it, for the same reason it may not span a cut: the edge is where the picture changes."]
    pub span_start_ticks: u64,
    pub weights: CaptionCuesSegmentationWeights,
}
impl CaptionCuesSegmentation {
    pub fn builder() -> builder::CaptionCuesSegmentation {
        Default::default()
    }
}
#[doc = "Where emphasis was allowed to come from. `none` when no index was read, and there is no third option in this build — emphasis is selected from evidence or it is not selected. Decoration is not a source."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Where emphasis was allowed to come from. `none` when no index was read, and there is no third option in this build — emphasis is selected from evidence or it is not selected. Decoration is not a source.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"none\","]
#[doc = "    \"index_keywords\""]
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
pub enum CaptionCuesSegmentationEmphasisSource {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "index_keywords")]
    IndexKeywords,
}
impl ::std::fmt::Display for CaptionCuesSegmentationEmphasisSource {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::None => f.write_str("none"),
            Self::IndexKeywords => f.write_str("index_keywords"),
        }
    }
}
impl ::std::str::FromStr for CaptionCuesSegmentationEmphasisSource {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "none" => Ok(Self::None),
            "index_keywords" => Ok(Self::IndexKeywords),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CaptionCuesSegmentationEmphasisSource {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCuesSegmentationEmphasisSource {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCuesSegmentationEmphasisSource {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Which filler list tagged the tokens. Versioned rather than inlined: the list changes with the language and with what creators actually say, and a cue set should name the one it was read with."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Which filler list tagged the tokens. Versioned rather than inlined: the list changes with the language and with what creators actually say, and a cue set should name the one it was read with.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CaptionCuesSegmentationFillerLexicon(::std::string::String);
impl ::std::ops::Deref for CaptionCuesSegmentationFillerLexicon {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CaptionCuesSegmentationFillerLexicon> for ::std::string::String {
    fn from(value: CaptionCuesSegmentationFillerLexicon) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CaptionCuesSegmentationFillerLexicon {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CaptionCuesSegmentationFillerLexicon {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaptionCuesSegmentationFillerLexicon {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaptionCuesSegmentationFillerLexicon {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CaptionCuesSegmentationFillerLexicon {
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
#[doc = "What the exact dynamic program was minimizing. Named individually because re-tuning any one of them invalidates exactly the cues it shaped."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What the exact dynamic program was minimizing. Named individually because re-tuning any one of them invalidates exactly the cues it shaped.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"break_quality\","]
#[doc = "    \"line_balance\","]
#[doc = "    \"orphan\","]
#[doc = "    \"reading_rate\","]
#[doc = "    \"short_cue\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"break_quality\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"line_balance\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"orphan\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"reading_rate\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"short_cue\": {"]
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
pub struct CaptionCuesSegmentationWeights {
    pub break_quality: f64,
    pub line_balance: f64,
    pub orphan: f64,
    pub reading_rate: f64,
    pub short_cue: f64,
}
impl CaptionCuesSegmentationWeights {
    pub fn builder() -> builder::CaptionCuesSegmentationWeights {
        Default::default()
    }
}
#[doc = "`Correction`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"origin\","]
#[doc = "    \"text\","]
#[doc = "    \"token_index\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"origin\": {"]
#[doc = "      \"description\": \"Who proposed it. A user correction outranks a re-transcription for the same token, which is the whole reason this is an overlay and not an edit.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"user\","]
#[doc = "        \"retranscription\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"token_index\": {"]
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
pub struct Correction {
    #[doc = "Who proposed it. A user correction outranks a re-transcription for the same token, which is the whole reason this is an overlay and not an edit."]
    pub origin: CorrectionOrigin,
    pub text: CorrectionText,
    pub token_index: u64,
}
impl Correction {
    pub fn builder() -> builder::Correction {
        Default::default()
    }
}
#[doc = "Who proposed it. A user correction outranks a re-transcription for the same token, which is the whole reason this is an overlay and not an edit."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Who proposed it. A user correction outranks a re-transcription for the same token, which is the whole reason this is an overlay and not an edit.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"user\","]
#[doc = "    \"retranscription\""]
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
pub enum CorrectionOrigin {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "retranscription")]
    Retranscription,
}
impl ::std::fmt::Display for CorrectionOrigin {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::User => f.write_str("user"),
            Self::Retranscription => f.write_str("retranscription"),
        }
    }
}
impl ::std::str::FromStr for CorrectionOrigin {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "user" => Ok(Self::User),
            "retranscription" => Ok(Self::Retranscription),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CorrectionOrigin {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CorrectionOrigin {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CorrectionOrigin {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CorrectionText`"]
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
pub struct CorrectionText(::std::string::String);
impl ::std::ops::Deref for CorrectionText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CorrectionText> for ::std::string::String {
    fn from(value: CorrectionText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CorrectionText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CorrectionText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CorrectionText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CorrectionText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CorrectionText {
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
#[doc = "`Coverage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
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
#[doc = "`Cue`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"characters\","]
#[doc = "    \"cue_id\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_token\","]
#[doc = "    \"lines\","]
#[doc = "    \"reading_rate_cps\","]
#[doc = "    \"region\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"token_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"characters\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"cue_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"first_token\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"lines\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/line\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"reading_rate_cps\": {"]
#[doc = "      \"description\": \"Characters divided by the seconds this cue is on screen, recorded rather than recomputed so the validator and the segmenter cannot disagree about what was measured.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"region\": {"]
#[doc = "      \"description\": \"Where the cue sits. One stable anchor in this phase: a caption that hops lanes every cue reads as broken even when every hop was locally right, so per-scene avoidance is deliberately not attempted until there is something measured to avoid.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"lower_safe\","]
#[doc = "        \"upper_safe\","]
#[doc = "        \"center\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"description\": \"When the cue appears, which is not always when its first word was spoken: a cue may be held longer than its speech to meet the reading rate, up to where the next one needs the screen.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"token_count\": {"]
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
pub struct Cue {
    pub characters: ::std::num::NonZeroU64,
    pub cue_id: CueCueId,
    pub end_ticks: ::std::num::NonZeroU64,
    pub first_token: u64,
    pub lines: ::std::vec::Vec<Line>,
    #[doc = "Characters divided by the seconds this cue is on screen, recorded rather than recomputed so the validator and the segmenter cannot disagree about what was measured."]
    pub reading_rate_cps: f64,
    #[doc = "Where the cue sits. One stable anchor in this phase: a caption that hops lanes every cue reads as broken even when every hop was locally right, so per-scene avoidance is deliberately not attempted until there is something measured to avoid."]
    pub region: CueRegion,
    #[doc = "When the cue appears, which is not always when its first word was spoken: a cue may be held longer than its speech to meet the reading rate, up to where the next one needs the screen."]
    pub start_ticks: u64,
    pub token_count: ::std::num::NonZeroU64,
}
impl Cue {
    pub fn builder() -> builder::Cue {
        Default::default()
    }
}
#[doc = "`CueCueId`"]
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
pub struct CueCueId(::std::string::String);
impl ::std::ops::Deref for CueCueId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CueCueId> for ::std::string::String {
    fn from(value: CueCueId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CueCueId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CueCueId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CueCueId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CueCueId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CueCueId {
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
#[doc = "Where the cue sits. One stable anchor in this phase: a caption that hops lanes every cue reads as broken even when every hop was locally right, so per-scene avoidance is deliberately not attempted until there is something measured to avoid."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Where the cue sits. One stable anchor in this phase: a caption that hops lanes every cue reads as broken even when every hop was locally right, so per-scene avoidance is deliberately not attempted until there is something measured to avoid.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"lower_safe\","]
#[doc = "    \"upper_safe\","]
#[doc = "    \"center\""]
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
pub enum CueRegion {
    #[serde(rename = "lower_safe")]
    LowerSafe,
    #[serde(rename = "upper_safe")]
    UpperSafe,
    #[serde(rename = "center")]
    Center,
}
impl ::std::fmt::Display for CueRegion {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::LowerSafe => f.write_str("lower_safe"),
            Self::UpperSafe => f.write_str("upper_safe"),
            Self::Center => f.write_str("center"),
        }
    }
}
impl ::std::str::FromStr for CueRegion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "lower_safe" => Ok(Self::LowerSafe),
            "upper_safe" => Ok(Self::UpperSafe),
            "center" => Ok(Self::Center),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CueRegion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CueRegion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CueRegion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Intent`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cues\","]
#[doc = "    \"profile\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cues\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/cue\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"profile\": {"]
#[doc = "      \"$ref\": \"#/$defs/profile\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    pub cues: ::std::vec::Vec<Cue>,
    pub profile: Profile,
}
impl Intent {
    pub fn builder() -> builder::Intent {
        Default::default()
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
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"description\": \"Why this stretch has no cues. `unreadable_at_any_grouping` is the honest one: words so dense that no legal grouping meets the reading rate, reported rather than shipped as a cue nobody can read.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"no_words\","]
#[doc = "        \"unreadable_at_any_grouping\","]
#[doc = "        \"not_analyzed\""]
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
    pub detail: ::std::option::Option<::std::string::String>,
    pub end_ticks: u64,
    #[doc = "Why this stretch has no cues. `unreadable_at_any_grouping` is the honest one: words so dense that no legal grouping meets the reading rate, reported rather than shipped as a cue nobody can read."]
    pub reason: InvalidRegionReason,
    pub start_ticks: u64,
}
impl InvalidRegion {
    pub fn builder() -> builder::InvalidRegion {
        Default::default()
    }
}
#[doc = "Why this stretch has no cues. `unreadable_at_any_grouping` is the honest one: words so dense that no legal grouping meets the reading rate, reported rather than shipped as a cue nobody can read."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Why this stretch has no cues. `unreadable_at_any_grouping` is the honest one: words so dense that no legal grouping meets the reading rate, reported rather than shipped as a cue nobody can read.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"no_words\","]
#[doc = "    \"unreadable_at_any_grouping\","]
#[doc = "    \"not_analyzed\""]
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
    #[serde(rename = "no_words")]
    NoWords,
    #[serde(rename = "unreadable_at_any_grouping")]
    UnreadableAtAnyGrouping,
    #[serde(rename = "not_analyzed")]
    NotAnalyzed,
}
impl ::std::fmt::Display for InvalidRegionReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::NoWords => f.write_str("no_words"),
            Self::UnreadableAtAnyGrouping => f.write_str("unreadable_at_any_grouping"),
            Self::NotAnalyzed => f.write_str("not_analyzed"),
        }
    }
}
impl ::std::str::FromStr for InvalidRegionReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "no_words" => Ok(Self::NoWords),
            "unreadable_at_any_grouping" => Ok(Self::UnreadableAtAnyGrouping),
            "not_analyzed" => Ok(Self::NotAnalyzed),
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
#[doc = "A run of tokens on one rendered line. Stored as a range into the token array rather than as text, so the words on a line cannot drift from the words in the cue."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A run of tokens on one rendered line. Stored as a range into the token array rather than as text, so the words on a line cannot drift from the words in the cue.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"characters\","]
#[doc = "    \"first_token\","]
#[doc = "    \"token_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"characters\": {"]
#[doc = "      \"description\": \"The rendered width this line was measured at, recorded so a validator does not have to re-join the words to check the ceiling.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"first_token\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"token_count\": {"]
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
pub struct Line {
    #[doc = "The rendered width this line was measured at, recorded so a validator does not have to re-join the words to check the ceiling."]
    pub characters: ::std::num::NonZeroU64,
    pub first_token: u64,
    pub token_count: ::std::num::NonZeroU64,
}
impl Line {
    pub fn builder() -> builder::Line {
        Default::default()
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
#[doc = "The numbers a grouping was held to. The accessibility intent carries the conservative public standard; the burn-in intent carries whatever the kinetic style asked for. Stored per intent rather than per document because the whole point of two intents is that they are held to different numbers."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The numbers a grouping was held to. The accessibility intent carries the conservative public standard; the burn-in intent carries whatever the kinetic style asked for. Stored per intent rather than per document because the whole point of two intents is that they are held to different numbers.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"max_duration_ticks\","]
#[doc = "    \"max_line_characters\","]
#[doc = "    \"max_lines\","]
#[doc = "    \"min_duration_ticks\","]
#[doc = "    \"min_gap_ticks\","]
#[doc = "    \"reading_rate_cps\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"max_duration_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"max_line_characters\": {"]
#[doc = "      \"description\": \"Characters per line, counted in Unicode scalar values of the rendered text. 42 for English, per the published timed-text standard; different for CJK, which is why it is a number here and not a constant in the segmenter.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"max_lines\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"min_duration_ticks\": {"]
#[doc = "      \"description\": \"The floor a cue is held on screen for, independent of how briefly its words were spoken. A cue that flashes is a cue nobody read.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"min_gap_ticks\": {"]
#[doc = "      \"description\": \"The blank between consecutive cues. Without it two cues read as one that changed under the reader.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"reading_rate_cps\": {"]
#[doc = "      \"description\": \"The ceiling a cue's characters-per-second may not exceed. A cue over it is legible only to a reader who already knows what it says.\","]
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
pub struct Profile {
    pub max_duration_ticks: ::std::num::NonZeroU64,
    #[doc = "Characters per line, counted in Unicode scalar values of the rendered text. 42 for English, per the published timed-text standard; different for CJK, which is why it is a number here and not a constant in the segmenter."]
    pub max_line_characters: ::std::num::NonZeroU64,
    pub max_lines: ::std::num::NonZeroU64,
    #[doc = "The floor a cue is held on screen for, independent of how briefly its words were spoken. A cue that flashes is a cue nobody read."]
    pub min_duration_ticks: ::std::num::NonZeroU64,
    #[doc = "The blank between consecutive cues. Without it two cues read as one that changed under the reader."]
    pub min_gap_ticks: u64,
    #[doc = "The ceiling a cue's characters-per-second may not exceed. A cue over it is legible only to a reader who already knows what it says."]
    pub reading_rate_cps: f64,
}
impl Profile {
    pub fn builder() -> builder::Profile {
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
#[doc = "One word, and everything decided about it. The normalized form is kept beside the text rather than instead of it, because matching against a lexicon and rendering to a screen want different strings and collapsing them loses the user's capitalization."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"One word, and everything decided about it. The normalized form is kept beside the text rather than instead of it, because matching against a lexicon and rendering to a screen want different strings and collapsing them loses the user's capitalization.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"emphasis\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"filler\","]
#[doc = "    \"index\","]
#[doc = "    \"normalized\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"text\","]
#[doc = "    \"word_index\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"emphasis\": {"]
#[doc = "      \"description\": \"Selected from evidence: a term the topic index found salient. Never from length, never from position, never from a model asked to be interesting.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"filler\": {"]
#[doc = "      \"description\": \"Tagged from the lexicon. A filler is still rendered — captions are a record of what was said, not an improvement on it — but it may never carry emphasis.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"normalized\": {"]
#[doc = "      \"description\": \"Lowercased and stripped of surrounding punctuation, for lexicon and keyword matching only. Never rendered.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"speaker\": {"]
#[doc = "      \"description\": \"Present only when the transcript named one. Absent is not the same as unknown-and-recorded.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"word_index\": {"]
#[doc = "      \"description\": \"The word this came from in the transcript, so any caption can be walked back to the observation behind it.\","]
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
pub struct Token {
    pub confidence: f64,
    #[doc = "Selected from evidence: a term the topic index found salient. Never from length, never from position, never from a model asked to be interesting."]
    pub emphasis: bool,
    pub end_ticks: ::std::num::NonZeroU64,
    #[doc = "Tagged from the lexicon. A filler is still rendered — captions are a record of what was said, not an improvement on it — but it may never carry emphasis."]
    pub filler: bool,
    pub index: u64,
    #[doc = "Lowercased and stripped of surrounding punctuation, for lexicon and keyword matching only. Never rendered."]
    pub normalized: ::std::string::String,
    #[doc = "Present only when the transcript named one. Absent is not the same as unknown-and-recorded."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub speaker: ::std::option::Option<TokenSpeaker>,
    pub start_ticks: u64,
    pub text: TokenText,
    #[doc = "The word this came from in the transcript, so any caption can be walked back to the observation behind it."]
    pub word_index: u64,
}
impl Token {
    pub fn builder() -> builder::Token {
        Default::default()
    }
}
#[doc = "Present only when the transcript named one. Absent is not the same as unknown-and-recorded."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Present only when the transcript named one. Absent is not the same as unknown-and-recorded.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TokenSpeaker(::std::string::String);
impl ::std::ops::Deref for TokenSpeaker {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TokenSpeaker> for ::std::string::String {
    fn from(value: TokenSpeaker) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TokenSpeaker {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TokenSpeaker {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TokenSpeaker {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TokenSpeaker {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TokenSpeaker {
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
#[doc = "`TokenText`"]
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
pub struct TokenText(::std::string::String);
impl ::std::ops::Deref for TokenText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TokenText> for ::std::string::String {
    fn from(value: TokenText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TokenText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TokenText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TokenText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TokenText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TokenText {
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
    pub struct CaptionCues {
        corrections:
            ::std::result::Result<::std::vec::Vec<super::Correction>, ::std::string::String>,
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        direction: ::std::result::Result<super::CaptionCuesDirection, ::std::string::String>,
        inputs: ::std::result::Result<super::CaptionCuesInputs, ::std::string::String>,
        intents: ::std::result::Result<super::CaptionCuesIntents, ::std::string::String>,
        invalid_regions:
            ::std::result::Result<::std::vec::Vec<super::InvalidRegion>, ::std::string::String>,
        language: ::std::result::Result<super::CaptionCuesLanguage, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        segmentation: ::std::result::Result<super::CaptionCuesSegmentation, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        tokens: ::std::result::Result<::std::vec::Vec<super::Token>, ::std::string::String>,
    }
    impl ::std::default::Default for CaptionCues {
        fn default() -> Self {
            Self {
                corrections: Err("no value supplied for corrections".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                direction: Err("no value supplied for direction".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                intents: Err("no value supplied for intents".to_string()),
                invalid_regions: Err("no value supplied for invalid_regions".to_string()),
                language: Err("no value supplied for language".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                segmentation: Err("no value supplied for segmentation".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                tokens: Err("no value supplied for tokens".to_string()),
            }
        }
    }
    impl CaptionCues {
        pub fn corrections<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Correction>>,
            T::Error: ::std::fmt::Display,
        {
            self.corrections = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for corrections: {e}"));
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
        pub fn direction<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCuesDirection>,
            T::Error: ::std::fmt::Display,
        {
            self.direction = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for direction: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCuesInputs>,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
            self
        }
        pub fn intents<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCuesIntents>,
            T::Error: ::std::fmt::Display,
        {
            self.intents = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for intents: {e}"));
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
            T: ::std::convert::TryInto<super::CaptionCuesLanguage>,
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
            T: ::std::convert::TryInto<super::CaptionCuesSegmentation>,
            T::Error: ::std::fmt::Display,
        {
            self.segmentation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segmentation: {e}"));
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
        pub fn tokens<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Token>>,
            T::Error: ::std::fmt::Display,
        {
            self.tokens = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tokens: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionCues> for super::CaptionCues {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCues,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                corrections: value.corrections?,
                coverage: value.coverage?,
                direction: value.direction?,
                inputs: value.inputs?,
                intents: value.intents?,
                invalid_regions: value.invalid_regions?,
                language: value.language?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                segmentation: value.segmentation?,
                source_fingerprint: value.source_fingerprint?,
                tokens: value.tokens?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCues> for CaptionCues {
        fn from(value: super::CaptionCues) -> Self {
            Self {
                corrections: Ok(value.corrections),
                coverage: Ok(value.coverage),
                direction: Ok(value.direction),
                inputs: Ok(value.inputs),
                intents: Ok(value.intents),
                invalid_regions: Ok(value.invalid_regions),
                language: Ok(value.language),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                segmentation: Ok(value.segmentation),
                source_fingerprint: Ok(value.source_fingerprint),
                tokens: Ok(value.tokens),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CaptionCuesInputs {
        index_artifact_id:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        shots_artifact_id:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        transcript_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for CaptionCuesInputs {
        fn default() -> Self {
            Self {
                index_artifact_id: Ok(Default::default()),
                shots_artifact_id: Ok(Default::default()),
                transcript_artifact_id: Err(
                    "no value supplied for transcript_artifact_id".to_string()
                ),
            }
        }
    }
    impl CaptionCuesInputs {
        pub fn index_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.index_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for index_artifact_id: {e}"));
            self
        }
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
    impl ::std::convert::TryFrom<CaptionCuesInputs> for super::CaptionCuesInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCuesInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                index_artifact_id: value.index_artifact_id?,
                shots_artifact_id: value.shots_artifact_id?,
                transcript_artifact_id: value.transcript_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCuesInputs> for CaptionCuesInputs {
        fn from(value: super::CaptionCuesInputs) -> Self {
            Self {
                index_artifact_id: Ok(value.index_artifact_id),
                shots_artifact_id: Ok(value.shots_artifact_id),
                transcript_artifact_id: Ok(value.transcript_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CaptionCuesIntents {
        accessibility: ::std::result::Result<super::Intent, ::std::string::String>,
        burn_in: ::std::result::Result<super::Intent, ::std::string::String>,
    }
    impl ::std::default::Default for CaptionCuesIntents {
        fn default() -> Self {
            Self {
                accessibility: Err("no value supplied for accessibility".to_string()),
                burn_in: Err("no value supplied for burn_in".to_string()),
            }
        }
    }
    impl CaptionCuesIntents {
        pub fn accessibility<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Intent>,
            T::Error: ::std::fmt::Display,
        {
            self.accessibility = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for accessibility: {e}"));
            self
        }
        pub fn burn_in<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Intent>,
            T::Error: ::std::fmt::Display,
        {
            self.burn_in = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for burn_in: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionCuesIntents> for super::CaptionCuesIntents {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCuesIntents,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                accessibility: value.accessibility?,
                burn_in: value.burn_in?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCuesIntents> for CaptionCuesIntents {
        fn from(value: super::CaptionCuesIntents) -> Self {
            Self {
                accessibility: Ok(value.accessibility),
                burn_in: Ok(value.burn_in),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CaptionCuesSegmentation {
        emphasis_source: ::std::result::Result<
            super::CaptionCuesSegmentationEmphasisSource,
            ::std::string::String,
        >,
        filler_lexicon: ::std::result::Result<
            super::CaptionCuesSegmentationFillerLexicon,
            ::std::string::String,
        >,
        had_index: ::std::result::Result<bool, ::std::string::String>,
        had_shots: ::std::result::Result<bool, ::std::string::String>,
        span_end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        span_start_ticks: ::std::result::Result<u64, ::std::string::String>,
        weights:
            ::std::result::Result<super::CaptionCuesSegmentationWeights, ::std::string::String>,
    }
    impl ::std::default::Default for CaptionCuesSegmentation {
        fn default() -> Self {
            Self {
                emphasis_source: Err("no value supplied for emphasis_source".to_string()),
                filler_lexicon: Err("no value supplied for filler_lexicon".to_string()),
                had_index: Err("no value supplied for had_index".to_string()),
                had_shots: Err("no value supplied for had_shots".to_string()),
                span_end_ticks: Err("no value supplied for span_end_ticks".to_string()),
                span_start_ticks: Err("no value supplied for span_start_ticks".to_string()),
                weights: Err("no value supplied for weights".to_string()),
            }
        }
    }
    impl CaptionCuesSegmentation {
        pub fn emphasis_source<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCuesSegmentationEmphasisSource>,
            T::Error: ::std::fmt::Display,
        {
            self.emphasis_source = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for emphasis_source: {e}"));
            self
        }
        pub fn filler_lexicon<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCuesSegmentationFillerLexicon>,
            T::Error: ::std::fmt::Display,
        {
            self.filler_lexicon = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for filler_lexicon: {e}"));
            self
        }
        pub fn had_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.had_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for had_index: {e}"));
            self
        }
        pub fn had_shots<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.had_shots = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for had_shots: {e}"));
            self
        }
        pub fn span_end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.span_end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for span_end_ticks: {e}"));
            self
        }
        pub fn span_start_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.span_start_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for span_start_ticks: {e}"));
            self
        }
        pub fn weights<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaptionCuesSegmentationWeights>,
            T::Error: ::std::fmt::Display,
        {
            self.weights = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for weights: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionCuesSegmentation> for super::CaptionCuesSegmentation {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCuesSegmentation,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                emphasis_source: value.emphasis_source?,
                filler_lexicon: value.filler_lexicon?,
                had_index: value.had_index?,
                had_shots: value.had_shots?,
                span_end_ticks: value.span_end_ticks?,
                span_start_ticks: value.span_start_ticks?,
                weights: value.weights?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCuesSegmentation> for CaptionCuesSegmentation {
        fn from(value: super::CaptionCuesSegmentation) -> Self {
            Self {
                emphasis_source: Ok(value.emphasis_source),
                filler_lexicon: Ok(value.filler_lexicon),
                had_index: Ok(value.had_index),
                had_shots: Ok(value.had_shots),
                span_end_ticks: Ok(value.span_end_ticks),
                span_start_ticks: Ok(value.span_start_ticks),
                weights: Ok(value.weights),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CaptionCuesSegmentationWeights {
        break_quality: ::std::result::Result<f64, ::std::string::String>,
        line_balance: ::std::result::Result<f64, ::std::string::String>,
        orphan: ::std::result::Result<f64, ::std::string::String>,
        reading_rate: ::std::result::Result<f64, ::std::string::String>,
        short_cue: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for CaptionCuesSegmentationWeights {
        fn default() -> Self {
            Self {
                break_quality: Err("no value supplied for break_quality".to_string()),
                line_balance: Err("no value supplied for line_balance".to_string()),
                orphan: Err("no value supplied for orphan".to_string()),
                reading_rate: Err("no value supplied for reading_rate".to_string()),
                short_cue: Err("no value supplied for short_cue".to_string()),
            }
        }
    }
    impl CaptionCuesSegmentationWeights {
        pub fn break_quality<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.break_quality = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for break_quality: {e}"));
            self
        }
        pub fn line_balance<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.line_balance = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for line_balance: {e}"));
            self
        }
        pub fn orphan<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.orphan = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for orphan: {e}"));
            self
        }
        pub fn reading_rate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.reading_rate = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reading_rate: {e}"));
            self
        }
        pub fn short_cue<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.short_cue = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for short_cue: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CaptionCuesSegmentationWeights>
        for super::CaptionCuesSegmentationWeights
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CaptionCuesSegmentationWeights,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                break_quality: value.break_quality?,
                line_balance: value.line_balance?,
                orphan: value.orphan?,
                reading_rate: value.reading_rate?,
                short_cue: value.short_cue?,
            })
        }
    }
    impl ::std::convert::From<super::CaptionCuesSegmentationWeights>
        for CaptionCuesSegmentationWeights
    {
        fn from(value: super::CaptionCuesSegmentationWeights) -> Self {
            Self {
                break_quality: Ok(value.break_quality),
                line_balance: Ok(value.line_balance),
                orphan: Ok(value.orphan),
                reading_rate: Ok(value.reading_rate),
                short_cue: Ok(value.short_cue),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Correction {
        origin: ::std::result::Result<super::CorrectionOrigin, ::std::string::String>,
        text: ::std::result::Result<super::CorrectionText, ::std::string::String>,
        token_index: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Correction {
        fn default() -> Self {
            Self {
                origin: Err("no value supplied for origin".to_string()),
                text: Err("no value supplied for text".to_string()),
                token_index: Err("no value supplied for token_index".to_string()),
            }
        }
    }
    impl Correction {
        pub fn origin<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CorrectionOrigin>,
            T::Error: ::std::fmt::Display,
        {
            self.origin = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for origin: {e}"));
            self
        }
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CorrectionText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
        pub fn token_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.token_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for token_index: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Correction> for super::Correction {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Correction,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                origin: value.origin?,
                text: value.text?,
                token_index: value.token_index?,
            })
        }
    }
    impl ::std::convert::From<super::Correction> for Correction {
        fn from(value: super::Correction) -> Self {
            Self {
                origin: Ok(value.origin),
                text: Ok(value.text),
                token_index: Ok(value.token_index),
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
    pub struct Cue {
        characters: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        cue_id: ::std::result::Result<super::CueCueId, ::std::string::String>,
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        first_token: ::std::result::Result<u64, ::std::string::String>,
        lines: ::std::result::Result<::std::vec::Vec<super::Line>, ::std::string::String>,
        reading_rate_cps: ::std::result::Result<f64, ::std::string::String>,
        region: ::std::result::Result<super::CueRegion, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        token_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for Cue {
        fn default() -> Self {
            Self {
                characters: Err("no value supplied for characters".to_string()),
                cue_id: Err("no value supplied for cue_id".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_token: Err("no value supplied for first_token".to_string()),
                lines: Err("no value supplied for lines".to_string()),
                reading_rate_cps: Err("no value supplied for reading_rate_cps".to_string()),
                region: Err("no value supplied for region".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                token_count: Err("no value supplied for token_count".to_string()),
            }
        }
    }
    impl Cue {
        pub fn characters<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.characters = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for characters: {e}"));
            self
        }
        pub fn cue_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CueCueId>,
            T::Error: ::std::fmt::Display,
        {
            self.cue_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cue_id: {e}"));
            self
        }
        pub fn end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_ticks: {e}"));
            self
        }
        pub fn first_token<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_token = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_token: {e}"));
            self
        }
        pub fn lines<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Line>>,
            T::Error: ::std::fmt::Display,
        {
            self.lines = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for lines: {e}"));
            self
        }
        pub fn reading_rate_cps<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.reading_rate_cps = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reading_rate_cps: {e}"));
            self
        }
        pub fn region<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CueRegion>,
            T::Error: ::std::fmt::Display,
        {
            self.region = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for region: {e}"));
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
        pub fn token_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.token_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for token_count: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Cue> for super::Cue {
        type Error = super::error::ConversionError;
        fn try_from(value: Cue) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                characters: value.characters?,
                cue_id: value.cue_id?,
                end_ticks: value.end_ticks?,
                first_token: value.first_token?,
                lines: value.lines?,
                reading_rate_cps: value.reading_rate_cps?,
                region: value.region?,
                start_ticks: value.start_ticks?,
                token_count: value.token_count?,
            })
        }
    }
    impl ::std::convert::From<super::Cue> for Cue {
        fn from(value: super::Cue) -> Self {
            Self {
                characters: Ok(value.characters),
                cue_id: Ok(value.cue_id),
                end_ticks: Ok(value.end_ticks),
                first_token: Ok(value.first_token),
                lines: Ok(value.lines),
                reading_rate_cps: Ok(value.reading_rate_cps),
                region: Ok(value.region),
                start_ticks: Ok(value.start_ticks),
                token_count: Ok(value.token_count),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Intent {
        cues: ::std::result::Result<::std::vec::Vec<super::Cue>, ::std::string::String>,
        profile: ::std::result::Result<super::Profile, ::std::string::String>,
    }
    impl ::std::default::Default for Intent {
        fn default() -> Self {
            Self {
                cues: Err("no value supplied for cues".to_string()),
                profile: Err("no value supplied for profile".to_string()),
            }
        }
    }
    impl Intent {
        pub fn cues<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Cue>>,
            T::Error: ::std::fmt::Display,
        {
            self.cues = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cues: {e}"));
            self
        }
        pub fn profile<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Profile>,
            T::Error: ::std::fmt::Display,
        {
            self.profile = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for profile: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Intent> for super::Intent {
        type Error = super::error::ConversionError;
        fn try_from(value: Intent) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cues: value.cues?,
                profile: value.profile?,
            })
        }
    }
    impl ::std::convert::From<super::Intent> for Intent {
        fn from(value: super::Intent) -> Self {
            Self {
                cues: Ok(value.cues),
                profile: Ok(value.profile),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct InvalidRegion {
        detail: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
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
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
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
    pub struct Line {
        characters: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        first_token: ::std::result::Result<u64, ::std::string::String>,
        token_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for Line {
        fn default() -> Self {
            Self {
                characters: Err("no value supplied for characters".to_string()),
                first_token: Err("no value supplied for first_token".to_string()),
                token_count: Err("no value supplied for token_count".to_string()),
            }
        }
    }
    impl Line {
        pub fn characters<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.characters = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for characters: {e}"));
            self
        }
        pub fn first_token<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_token = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_token: {e}"));
            self
        }
        pub fn token_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.token_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for token_count: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Line> for super::Line {
        type Error = super::error::ConversionError;
        fn try_from(value: Line) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                characters: value.characters?,
                first_token: value.first_token?,
                token_count: value.token_count?,
            })
        }
    }
    impl ::std::convert::From<super::Line> for Line {
        fn from(value: super::Line) -> Self {
            Self {
                characters: Ok(value.characters),
                first_token: Ok(value.first_token),
                token_count: Ok(value.token_count),
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
    pub struct Profile {
        max_duration_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        max_line_characters: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        max_lines: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        min_duration_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        min_gap_ticks: ::std::result::Result<u64, ::std::string::String>,
        reading_rate_cps: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Profile {
        fn default() -> Self {
            Self {
                max_duration_ticks: Err("no value supplied for max_duration_ticks".to_string()),
                max_line_characters: Err("no value supplied for max_line_characters".to_string()),
                max_lines: Err("no value supplied for max_lines".to_string()),
                min_duration_ticks: Err("no value supplied for min_duration_ticks".to_string()),
                min_gap_ticks: Err("no value supplied for min_gap_ticks".to_string()),
                reading_rate_cps: Err("no value supplied for reading_rate_cps".to_string()),
            }
        }
    }
    impl Profile {
        pub fn max_duration_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_duration_ticks = value.try_into().map_err(|e| {
                format!("error converting supplied value for max_duration_ticks: {e}")
            });
            self
        }
        pub fn max_line_characters<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_line_characters = value.try_into().map_err(|e| {
                format!("error converting supplied value for max_line_characters: {e}")
            });
            self
        }
        pub fn max_lines<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_lines = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max_lines: {e}"));
            self
        }
        pub fn min_duration_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_duration_ticks = value.try_into().map_err(|e| {
                format!("error converting supplied value for min_duration_ticks: {e}")
            });
            self
        }
        pub fn min_gap_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_gap_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for min_gap_ticks: {e}"));
            self
        }
        pub fn reading_rate_cps<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.reading_rate_cps = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reading_rate_cps: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Profile> for super::Profile {
        type Error = super::error::ConversionError;
        fn try_from(value: Profile) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                max_duration_ticks: value.max_duration_ticks?,
                max_line_characters: value.max_line_characters?,
                max_lines: value.max_lines?,
                min_duration_ticks: value.min_duration_ticks?,
                min_gap_ticks: value.min_gap_ticks?,
                reading_rate_cps: value.reading_rate_cps?,
            })
        }
    }
    impl ::std::convert::From<super::Profile> for Profile {
        fn from(value: super::Profile) -> Self {
            Self {
                max_duration_ticks: Ok(value.max_duration_ticks),
                max_line_characters: Ok(value.max_line_characters),
                max_lines: Ok(value.max_lines),
                min_duration_ticks: Ok(value.min_duration_ticks),
                min_gap_ticks: Ok(value.min_gap_ticks),
                reading_rate_cps: Ok(value.reading_rate_cps),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Token {
        confidence: ::std::result::Result<f64, ::std::string::String>,
        emphasis: ::std::result::Result<bool, ::std::string::String>,
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        filler: ::std::result::Result<bool, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        normalized: ::std::result::Result<::std::string::String, ::std::string::String>,
        speaker: ::std::result::Result<
            ::std::option::Option<super::TokenSpeaker>,
            ::std::string::String,
        >,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        text: ::std::result::Result<super::TokenText, ::std::string::String>,
        word_index: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Token {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                emphasis: Err("no value supplied for emphasis".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                filler: Err("no value supplied for filler".to_string()),
                index: Err("no value supplied for index".to_string()),
                normalized: Err("no value supplied for normalized".to_string()),
                speaker: Ok(Default::default()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                text: Err("no value supplied for text".to_string()),
                word_index: Err("no value supplied for word_index".to_string()),
            }
        }
    }
    impl Token {
        pub fn confidence<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.confidence = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for confidence: {e}"));
            self
        }
        pub fn emphasis<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.emphasis = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for emphasis: {e}"));
            self
        }
        pub fn end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_ticks: {e}"));
            self
        }
        pub fn filler<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.filler = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for filler: {e}"));
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
        pub fn normalized<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.normalized = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for normalized: {e}"));
            self
        }
        pub fn speaker<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::TokenSpeaker>>,
            T::Error: ::std::fmt::Display,
        {
            self.speaker = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for speaker: {e}"));
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
            T: ::std::convert::TryInto<super::TokenText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
        pub fn word_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.word_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for word_index: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Token> for super::Token {
        type Error = super::error::ConversionError;
        fn try_from(value: Token) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                emphasis: value.emphasis?,
                end_ticks: value.end_ticks?,
                filler: value.filler?,
                index: value.index?,
                normalized: value.normalized?,
                speaker: value.speaker?,
                start_ticks: value.start_ticks?,
                text: value.text?,
                word_index: value.word_index?,
            })
        }
    }
    impl ::std::convert::From<super::Token> for Token {
        fn from(value: super::Token) -> Self {
            Self {
                confidence: Ok(value.confidence),
                emphasis: Ok(value.emphasis),
                end_ticks: Ok(value.end_ticks),
                filler: Ok(value.filler),
                index: Ok(value.index),
                normalized: Ok(value.normalized),
                speaker: Ok(value.speaker),
                start_ticks: Ok(value.start_ticks),
                text: Ok(value.text),
                word_index: Ok(value.word_index),
            }
        }
    }
}
