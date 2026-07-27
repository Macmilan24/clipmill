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
#[doc = "A distribution, not a scalar: p10 is what the boundary is worth if the detector is having a bad time, and downstream stages that widen uncertainty need both numbers. A shot is only as certain as the weaker of the two cuts that bound it; an edge that is the start or end of coverage is a fact rather than a detection and claims nothing."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A distribution, not a scalar: p10 is what the boundary is worth if the detector is having a bad time, and downstream stages that widen uncertainty need both numbers. A shot is only as certain as the weaker of the two cuts that bound it; an edge that is the start or end of coverage is a fact rather than a detection and claims nothing.\","]
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
#[doc = "What was actually examined. A skipped pass, a single-shot result, and a recording nobody decoded are three different facts, and no downstream stage may read an empty cut list as 'the camera never changed'."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was actually examined. A skipped pass, a single-shot result, and a recording nobody decoded are three different facts, and no downstream stage may read an empty cut list as 'the camera never changed'.\","]
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
#[doc = "    \"sampling_plan\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
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
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub sampling_plan: ::std::option::Option<CoverageSamplingPlan>,
    pub start_ticks: u64,
}
impl Coverage {
    pub fn builder() -> builder::Coverage {
        Default::default()
    }
}
#[doc = "`CoverageSamplingPlan`"]
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
pub struct CoverageSamplingPlan(::std::string::String);
impl ::std::ops::Deref for CoverageSamplingPlan {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CoverageSamplingPlan> for ::std::string::String {
    fn from(value: CoverageSamplingPlan) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CoverageSamplingPlan {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CoverageSamplingPlan {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CoverageSamplingPlan {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CoverageSamplingPlan {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CoverageSamplingPlan {
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
#[doc = "`Cut`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"score\","]
#[doc = "    \"t_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"description\": \"The detector's raw content distance at this boundary, before any mapping onto a confidence. Kept because it is the only number here that can be compared against a re-tuned threshold without decoding the video again.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"t_ticks\": {"]
#[doc = "      \"description\": \"First tick of the incoming shot.\","]
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
pub struct Cut {
    pub confidence: Confidence,
    #[doc = "The detector's raw content distance at this boundary, before any mapping onto a confidence. Kept because it is the only number here that can be compared against a re-tuned threshold without decoding the video again."]
    pub score: f64,
    #[doc = "First tick of the incoming shot."]
    pub t_ticks: u64,
}
impl Cut {
    pub fn builder() -> builder::Cut {
        Default::default()
    }
}
#[doc = "Where the camera changed, decided from the proxy's pixels (book ch. 13). A cut is a boundary the editor did not have to justify, which is why the boundary lattice is later allowed to start and end a clip on one. Every position is integer ticks at 1/90000, and absence is explicit: a recording nobody analyzed is not a recording with no cuts in it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.evidence.shots.v1.json\","]
#[doc = "  \"title\": \"EvidenceShots\","]
#[doc = "  \"description\": \"Where the camera changed, decided from the proxy's pixels (book ch. 13). A cut is a boundary the editor did not have to justify, which is why the boundary lattice is later allowed to start and end a clip on one. Every position is integer ticks at 1/90000, and absence is explicit: a recording nobody analyzed is not a recording with no cuts in it.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coverage\","]
#[doc = "    \"cuts\","]
#[doc = "    \"detection\","]
#[doc = "    \"invalid_regions\","]
#[doc = "    \"producer\","]
#[doc = "    \"proxy_artifact_id\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"shots\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"cuts\": {"]
#[doc = "      \"description\": \"Shot boundaries in source time, ascending and distinct. Each names the first tick of the incoming shot, so a cut and the shot it starts share a position rather than differing by a frame nobody agrees on.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/cut\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"detection\": {"]
#[doc = "      \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one. The decoder identity belongs here for the same reason — a different build can hand the detector different pixels.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"analysis_height\","]
#[doc = "        \"decoder\","]
#[doc = "        \"frame_rate\","]
#[doc = "        \"min_shot_ticks\","]
#[doc = "        \"threshold\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"analysis_height\": {"]
#[doc = "          \"description\": \"Pixel height every frame was scaled to before comparison; width follows the display aspect. Downscaling is what makes the score about the shot rather than about sensor noise, and it is pinned because the score depends on it.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 16.0"]
#[doc = "        },"]
#[doc = "        \"decoder\": {"]
#[doc = "          \"description\": \"Identity of the pinned decoder build, e.g. 'ffmpeg-8.1.2-btb-n8.1.2'. A name and version, never a path — a machine-specific directory in the key would give the same recording two content addresses on two machines.\","]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"frame_rate\": {"]
#[doc = "          \"description\": \"Rate the proxy was decoded at, frames per second as a rational. This is the resolution of every position below: a cut cannot be located more precisely than the frame that revealed it.\","]
#[doc = "          \"$ref\": \"#/$defs/timebase\""]
#[doc = "        },"]
#[doc = "        \"min_shot_ticks\": {"]
#[doc = "          \"description\": \"A cut closer than this to the previous one is suppressed, because two cuts inside a few frames are one cut plus a flash.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"threshold\": {"]
#[doc = "          \"description\": \"Mean inter-frame content distance at or above which a frame boundary counts as a cut. Higher detects fewer, and misses the soft ones first.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"exclusiveMinimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"weights\": {"]
#[doc = "          \"description\": \"How the content distance was composed from its hue, saturation, luminance, and edge components. Omitted where the detector's own defaults were used.\","]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"required\": ["]
#[doc = "            \"edges\","]
#[doc = "            \"hue\","]
#[doc = "            \"luma\","]
#[doc = "            \"saturation\""]
#[doc = "          ],"]
#[doc = "          \"properties\": {"]
#[doc = "            \"edges\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"hue\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"luma\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"saturation\": {"]
#[doc = "              \"type\": \"number\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            }"]
#[doc = "          },"]
#[doc = "          \"additionalProperties\": false"]
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
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"proxy_artifact_id\": {"]
#[doc = "      \"description\": \"The mezzanine proxy this pass decoded, verified before use. Not the original: the proxy is the one decode every analysis surface shares, so two stages that disagree about a frame are a bug rather than a licensing accident.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.evidence.shots.v1\""]
#[doc = "    },"]
#[doc = "    \"shots\": {"]
#[doc = "      \"description\": \"The spans between cuts, including the ones bounded by the start and end of coverage. Carried explicitly rather than left to be derived, because these are the intervals the boundary lattice and the layout stage consume, and a consumer should not have to reconstruct them correctly from cuts plus coverage.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/shot\""]
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
pub struct EvidenceShots {
    pub coverage: Coverage,
    #[doc = "Shot boundaries in source time, ascending and distinct. Each names the first tick of the incoming shot, so a cut and the shot it starts share a position rather than differing by a frame nobody agrees on."]
    pub cuts: ::std::vec::Vec<Cut>,
    pub detection: EvidenceShotsDetection,
    pub invalid_regions: ::std::vec::Vec<InvalidRegion>,
    pub producer: Producer,
    #[doc = "The mezzanine proxy this pass decoded, verified before use. Not the original: the proxy is the one decode every analysis surface shares, so two stages that disagree about a frame are a bug rather than a licensing accident."]
    pub proxy_artifact_id: Sha256,
    pub schema_version: ::serde_json::Value,
    #[doc = "The spans between cuts, including the ones bounded by the start and end of coverage. Carried explicitly rather than left to be derived, because these are the intervals the boundary lattice and the layout stage consume, and a consumer should not have to reconstruct them correctly from cuts plus coverage."]
    pub shots: ::std::vec::Vec<Shot>,
    pub source_fingerprint: Sha256,
}
impl EvidenceShots {
    pub fn builder() -> builder::EvidenceShots {
        Default::default()
    }
}
#[doc = "The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one. The decoder identity belongs here for the same reason — a different build can hand the detector different pixels."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one. The decoder identity belongs here for the same reason — a different build can hand the detector different pixels.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"analysis_height\","]
#[doc = "    \"decoder\","]
#[doc = "    \"frame_rate\","]
#[doc = "    \"min_shot_ticks\","]
#[doc = "    \"threshold\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"analysis_height\": {"]
#[doc = "      \"description\": \"Pixel height every frame was scaled to before comparison; width follows the display aspect. Downscaling is what makes the score about the shot rather than about sensor noise, and it is pinned because the score depends on it.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 16.0"]
#[doc = "    },"]
#[doc = "    \"decoder\": {"]
#[doc = "      \"description\": \"Identity of the pinned decoder build, e.g. 'ffmpeg-8.1.2-btb-n8.1.2'. A name and version, never a path — a machine-specific directory in the key would give the same recording two content addresses on two machines.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"frame_rate\": {"]
#[doc = "      \"description\": \"Rate the proxy was decoded at, frames per second as a rational. This is the resolution of every position below: a cut cannot be located more precisely than the frame that revealed it.\","]
#[doc = "      \"$ref\": \"#/$defs/timebase\""]
#[doc = "    },"]
#[doc = "    \"min_shot_ticks\": {"]
#[doc = "      \"description\": \"A cut closer than this to the previous one is suppressed, because two cuts inside a few frames are one cut plus a flash.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"threshold\": {"]
#[doc = "      \"description\": \"Mean inter-frame content distance at or above which a frame boundary counts as a cut. Higher detects fewer, and misses the soft ones first.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"weights\": {"]
#[doc = "      \"description\": \"How the content distance was composed from its hue, saturation, luminance, and edge components. Omitted where the detector's own defaults were used.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"edges\","]
#[doc = "        \"hue\","]
#[doc = "        \"luma\","]
#[doc = "        \"saturation\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"edges\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"hue\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"luma\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"saturation\": {"]
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
pub struct EvidenceShotsDetection {
    #[doc = "Pixel height every frame was scaled to before comparison; width follows the display aspect. Downscaling is what makes the score about the shot rather than about sensor noise, and it is pinned because the score depends on it."]
    pub analysis_height: i64,
    #[doc = "Identity of the pinned decoder build, e.g. 'ffmpeg-8.1.2-btb-n8.1.2'. A name and version, never a path — a machine-specific directory in the key would give the same recording two content addresses on two machines."]
    pub decoder: EvidenceShotsDetectionDecoder,
    #[doc = "Rate the proxy was decoded at, frames per second as a rational. This is the resolution of every position below: a cut cannot be located more precisely than the frame that revealed it."]
    pub frame_rate: Timebase,
    #[doc = "A cut closer than this to the previous one is suppressed, because two cuts inside a few frames are one cut plus a flash."]
    pub min_shot_ticks: u64,
    #[doc = "Mean inter-frame content distance at or above which a frame boundary counts as a cut. Higher detects fewer, and misses the soft ones first."]
    pub threshold: f64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub weights: ::std::option::Option<EvidenceShotsDetectionWeights>,
}
impl EvidenceShotsDetection {
    pub fn builder() -> builder::EvidenceShotsDetection {
        Default::default()
    }
}
#[doc = "Identity of the pinned decoder build, e.g. 'ffmpeg-8.1.2-btb-n8.1.2'. A name and version, never a path — a machine-specific directory in the key would give the same recording two content addresses on two machines."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Identity of the pinned decoder build, e.g. 'ffmpeg-8.1.2-btb-n8.1.2'. A name and version, never a path — a machine-specific directory in the key would give the same recording two content addresses on two machines.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EvidenceShotsDetectionDecoder(::std::string::String);
impl ::std::ops::Deref for EvidenceShotsDetectionDecoder {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EvidenceShotsDetectionDecoder> for ::std::string::String {
    fn from(value: EvidenceShotsDetectionDecoder) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EvidenceShotsDetectionDecoder {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EvidenceShotsDetectionDecoder {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EvidenceShotsDetectionDecoder {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EvidenceShotsDetectionDecoder {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EvidenceShotsDetectionDecoder {
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
#[doc = "How the content distance was composed from its hue, saturation, luminance, and edge components. Omitted where the detector's own defaults were used."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How the content distance was composed from its hue, saturation, luminance, and edge components. Omitted where the detector's own defaults were used.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"edges\","]
#[doc = "    \"hue\","]
#[doc = "    \"luma\","]
#[doc = "    \"saturation\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"edges\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"hue\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"luma\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"saturation\": {"]
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
pub struct EvidenceShotsDetectionWeights {
    pub edges: f64,
    pub hue: f64,
    pub luma: f64,
    pub saturation: f64,
}
impl EvidenceShotsDetectionWeights {
    pub fn builder() -> builder::EvidenceShotsDetectionWeights {
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
#[doc = "        \"no_video\","]
#[doc = "        \"not_analyzed\","]
#[doc = "        \"decode_failed\""]
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
#[doc = "    \"no_video\","]
#[doc = "    \"not_analyzed\","]
#[doc = "    \"decode_failed\""]
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
    #[serde(rename = "no_video")]
    NoVideo,
    #[serde(rename = "not_analyzed")]
    NotAnalyzed,
    #[serde(rename = "decode_failed")]
    DecodeFailed,
}
impl ::std::fmt::Display for InvalidRegionReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::NoVideo => f.write_str("no_video"),
            Self::NotAnalyzed => f.write_str("not_analyzed"),
            Self::DecodeFailed => f.write_str("decode_failed"),
        }
    }
}
impl ::std::str::FromStr for InvalidRegionReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "no_video" => Ok(Self::NoVideo),
            "not_analyzed" => Ok(Self::NotAnalyzed),
            "decode_failed" => Ok(Self::DecodeFailed),
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
#[doc = "    \"calibration\": {"]
#[doc = "      \"description\": \"Which calibration mapped raw content distances onto the confidences above.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"implementation\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"model_digest\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
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
    #[doc = "Which calibration mapped raw content distances onto the confidences above."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub calibration: ::std::option::Option<ProducerCalibration>,
    pub implementation: ProducerImplementation,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub model_digest: ::std::option::Option<Sha256>,
    pub stage: ProducerStage,
}
impl Producer {
    pub fn builder() -> builder::Producer {
        Default::default()
    }
}
#[doc = "Which calibration mapped raw content distances onto the confidences above."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Which calibration mapped raw content distances onto the confidences above.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProducerCalibration(::std::string::String);
impl ::std::ops::Deref for ProducerCalibration {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProducerCalibration> for ::std::string::String {
    fn from(value: ProducerCalibration) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProducerCalibration {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProducerCalibration {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProducerCalibration {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProducerCalibration {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProducerCalibration {
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
#[doc = "`Shot`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"confidence\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"confidence\": {"]
#[doc = "      \"$ref\": \"#/$defs/confidence\""]
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
pub struct Shot {
    pub confidence: Confidence,
    pub end_ticks: u64,
    pub start_ticks: u64,
}
impl Shot {
    pub fn builder() -> builder::Shot {
        Default::default()
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
        sampling_plan: ::std::result::Result<
            ::std::option::Option<super::CoverageSamplingPlan>,
            ::std::string::String,
        >,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Coverage {
        fn default() -> Self {
            Self {
                analyzed: Err("no value supplied for analyzed".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                sampling_plan: Ok(Default::default()),
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
        pub fn sampling_plan<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CoverageSamplingPlan>>,
            T::Error: ::std::fmt::Display,
        {
            self.sampling_plan = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sampling_plan: {e}"));
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
                sampling_plan: value.sampling_plan?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Coverage> for Coverage {
        fn from(value: super::Coverage) -> Self {
            Self {
                analyzed: Ok(value.analyzed),
                end_ticks: Ok(value.end_ticks),
                sampling_plan: Ok(value.sampling_plan),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Cut {
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        score: ::std::result::Result<f64, ::std::string::String>,
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Cut {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                score: Err("no value supplied for score".to_string()),
                t_ticks: Err("no value supplied for t_ticks".to_string()),
            }
        }
    }
    impl Cut {
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
    impl ::std::convert::TryFrom<Cut> for super::Cut {
        type Error = super::error::ConversionError;
        fn try_from(value: Cut) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                score: value.score?,
                t_ticks: value.t_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Cut> for Cut {
        fn from(value: super::Cut) -> Self {
            Self {
                confidence: Ok(value.confidence),
                score: Ok(value.score),
                t_ticks: Ok(value.t_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EvidenceShots {
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        cuts: ::std::result::Result<::std::vec::Vec<super::Cut>, ::std::string::String>,
        detection: ::std::result::Result<super::EvidenceShotsDetection, ::std::string::String>,
        invalid_regions:
            ::std::result::Result<::std::vec::Vec<super::InvalidRegion>, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        proxy_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        shots: ::std::result::Result<::std::vec::Vec<super::Shot>, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for EvidenceShots {
        fn default() -> Self {
            Self {
                coverage: Err("no value supplied for coverage".to_string()),
                cuts: Err("no value supplied for cuts".to_string()),
                detection: Err("no value supplied for detection".to_string()),
                invalid_regions: Err("no value supplied for invalid_regions".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                proxy_artifact_id: Err("no value supplied for proxy_artifact_id".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                shots: Err("no value supplied for shots".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl EvidenceShots {
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
        pub fn cuts<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Cut>>,
            T::Error: ::std::fmt::Display,
        {
            self.cuts = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cuts: {e}"));
            self
        }
        pub fn detection<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EvidenceShotsDetection>,
            T::Error: ::std::fmt::Display,
        {
            self.detection = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detection: {e}"));
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
        pub fn proxy_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.proxy_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for proxy_artifact_id: {e}"));
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
        pub fn shots<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Shot>>,
            T::Error: ::std::fmt::Display,
        {
            self.shots = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shots: {e}"));
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
    impl ::std::convert::TryFrom<EvidenceShots> for super::EvidenceShots {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EvidenceShots,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                coverage: value.coverage?,
                cuts: value.cuts?,
                detection: value.detection?,
                invalid_regions: value.invalid_regions?,
                producer: value.producer?,
                proxy_artifact_id: value.proxy_artifact_id?,
                schema_version: value.schema_version?,
                shots: value.shots?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::EvidenceShots> for EvidenceShots {
        fn from(value: super::EvidenceShots) -> Self {
            Self {
                coverage: Ok(value.coverage),
                cuts: Ok(value.cuts),
                detection: Ok(value.detection),
                invalid_regions: Ok(value.invalid_regions),
                producer: Ok(value.producer),
                proxy_artifact_id: Ok(value.proxy_artifact_id),
                schema_version: Ok(value.schema_version),
                shots: Ok(value.shots),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EvidenceShotsDetection {
        analysis_height: ::std::result::Result<i64, ::std::string::String>,
        decoder: ::std::result::Result<super::EvidenceShotsDetectionDecoder, ::std::string::String>,
        frame_rate: ::std::result::Result<super::Timebase, ::std::string::String>,
        min_shot_ticks: ::std::result::Result<u64, ::std::string::String>,
        threshold: ::std::result::Result<f64, ::std::string::String>,
        weights: ::std::result::Result<
            ::std::option::Option<super::EvidenceShotsDetectionWeights>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for EvidenceShotsDetection {
        fn default() -> Self {
            Self {
                analysis_height: Err("no value supplied for analysis_height".to_string()),
                decoder: Err("no value supplied for decoder".to_string()),
                frame_rate: Err("no value supplied for frame_rate".to_string()),
                min_shot_ticks: Err("no value supplied for min_shot_ticks".to_string()),
                threshold: Err("no value supplied for threshold".to_string()),
                weights: Ok(Default::default()),
            }
        }
    }
    impl EvidenceShotsDetection {
        pub fn analysis_height<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.analysis_height = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for analysis_height: {e}"));
            self
        }
        pub fn decoder<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EvidenceShotsDetectionDecoder>,
            T::Error: ::std::fmt::Display,
        {
            self.decoder = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for decoder: {e}"));
            self
        }
        pub fn frame_rate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Timebase>,
            T::Error: ::std::fmt::Display,
        {
            self.frame_rate = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frame_rate: {e}"));
            self
        }
        pub fn min_shot_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_shot_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for min_shot_ticks: {e}"));
            self
        }
        pub fn threshold<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.threshold = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for threshold: {e}"));
            self
        }
        pub fn weights<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EvidenceShotsDetectionWeights>>,
            T::Error: ::std::fmt::Display,
        {
            self.weights = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for weights: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EvidenceShotsDetection> for super::EvidenceShotsDetection {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EvidenceShotsDetection,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                analysis_height: value.analysis_height?,
                decoder: value.decoder?,
                frame_rate: value.frame_rate?,
                min_shot_ticks: value.min_shot_ticks?,
                threshold: value.threshold?,
                weights: value.weights?,
            })
        }
    }
    impl ::std::convert::From<super::EvidenceShotsDetection> for EvidenceShotsDetection {
        fn from(value: super::EvidenceShotsDetection) -> Self {
            Self {
                analysis_height: Ok(value.analysis_height),
                decoder: Ok(value.decoder),
                frame_rate: Ok(value.frame_rate),
                min_shot_ticks: Ok(value.min_shot_ticks),
                threshold: Ok(value.threshold),
                weights: Ok(value.weights),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EvidenceShotsDetectionWeights {
        edges: ::std::result::Result<f64, ::std::string::String>,
        hue: ::std::result::Result<f64, ::std::string::String>,
        luma: ::std::result::Result<f64, ::std::string::String>,
        saturation: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for EvidenceShotsDetectionWeights {
        fn default() -> Self {
            Self {
                edges: Err("no value supplied for edges".to_string()),
                hue: Err("no value supplied for hue".to_string()),
                luma: Err("no value supplied for luma".to_string()),
                saturation: Err("no value supplied for saturation".to_string()),
            }
        }
    }
    impl EvidenceShotsDetectionWeights {
        pub fn edges<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.edges = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for edges: {e}"));
            self
        }
        pub fn hue<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.hue = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hue: {e}"));
            self
        }
        pub fn luma<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.luma = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for luma: {e}"));
            self
        }
        pub fn saturation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.saturation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for saturation: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EvidenceShotsDetectionWeights>
        for super::EvidenceShotsDetectionWeights
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EvidenceShotsDetectionWeights,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                edges: value.edges?,
                hue: value.hue?,
                luma: value.luma?,
                saturation: value.saturation?,
            })
        }
    }
    impl ::std::convert::From<super::EvidenceShotsDetectionWeights> for EvidenceShotsDetectionWeights {
        fn from(value: super::EvidenceShotsDetectionWeights) -> Self {
            Self {
                edges: Ok(value.edges),
                hue: Ok(value.hue),
                luma: Ok(value.luma),
                saturation: Ok(value.saturation),
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
    pub struct Producer {
        calibration: ::std::result::Result<
            ::std::option::Option<super::ProducerCalibration>,
            ::std::string::String,
        >,
        implementation: ::std::result::Result<super::ProducerImplementation, ::std::string::String>,
        model_digest:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        stage: ::std::result::Result<super::ProducerStage, ::std::string::String>,
    }
    impl ::std::default::Default for Producer {
        fn default() -> Self {
            Self {
                calibration: Ok(Default::default()),
                implementation: Err("no value supplied for implementation".to_string()),
                model_digest: Ok(Default::default()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl Producer {
        pub fn calibration<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ProducerCalibration>>,
            T::Error: ::std::fmt::Display,
        {
            self.calibration = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for calibration: {e}"));
            self
        }
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
        pub fn model_digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.model_digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model_digest: {e}"));
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
                calibration: value.calibration?,
                implementation: value.implementation?,
                model_digest: value.model_digest?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::Producer> for Producer {
        fn from(value: super::Producer) -> Self {
            Self {
                calibration: Ok(value.calibration),
                implementation: Ok(value.implementation),
                model_digest: Ok(value.model_digest),
                stage: Ok(value.stage),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Shot {
        confidence: ::std::result::Result<super::Confidence, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Shot {
        fn default() -> Self {
            Self {
                confidence: Err("no value supplied for confidence".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Shot {
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
    impl ::std::convert::TryFrom<Shot> for super::Shot {
        type Error = super::error::ConversionError;
        fn try_from(value: Shot) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                confidence: value.confidence?,
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Shot> for Shot {
        fn from(value: super::Shot) -> Self {
            Self {
                confidence: Ok(value.confidence),
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
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
