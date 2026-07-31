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
#[doc = "One face in one frame, normalized against the display frame: x and y are the top-left corner, w and h the extent, all in [0,1]. A box may touch an edge and may not leave it — the detector saw pixels that exist."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"One face in one frame, normalized against the display frame: x and y are the top-left corner, w and h the extent, all in [0,1]. A box may touch an edge and may not leave it — the detector saw pixels that exist.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"h\","]
#[doc = "    \"score\","]
#[doc = "    \"t_ticks\","]
#[doc = "    \"w\","]
#[doc = "    \"x\","]
#[doc = "    \"y\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"h\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"interpolated\": {"]
#[doc = "      \"description\": \"True when the box was carried across a frame the detector missed rather than measured in it. Present so a solver can weigh a bridged frame differently from a seen one, and so nobody reads a gap-filled track as continuous evidence.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"description\": \"The detector's own confidence in this box, kept raw so a re-tuned threshold can be applied without decoding anything again.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"t_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"w\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"x\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"y\": {"]
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
pub struct Box {
    pub h: f64,
    #[doc = "True when the box was carried across a frame the detector missed rather than measured in it. Present so a solver can weigh a bridged frame differently from a seen one, and so nobody reads a gap-filled track as continuous evidence."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub interpolated: ::std::option::Option<bool>,
    #[doc = "The detector's own confidence in this box, kept raw so a re-tuned threshold can be applied without decoding anything again."]
    pub score: f64,
    pub t_ticks: u64,
    pub w: f64,
    pub x: f64,
    pub y: f64,
}
impl Box {
    pub fn builder() -> builder::Box {
        Default::default()
    }
}
#[doc = "What was actually examined. A pass that ran and found nobody, a recording with no video, and frames nobody read are three different facts, and no downstream stage may read an empty track list as 'there was no one on screen'."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was actually examined. A pass that ran and found nobody, a recording with no video, and frames nobody read are three different facts, and no downstream stage may read an empty track list as 'there was no one on screen'.\","]
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
#[doc = "    \"frames_examined\": {"]
#[doc = "      \"description\": \"How many sampled frames the detector actually ran on.\","]
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
    #[doc = "How many sampled frames the detector actually ran on."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub frames_examined: ::std::option::Option<u64>,
    pub start_ticks: u64,
}
impl Coverage {
    pub fn builder() -> builder::Coverage {
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
#[doc = "`Track`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"boxes\","]
#[doc = "    \"first_ticks\","]
#[doc = "    \"frames_present\","]
#[doc = "    \"last_ticks\","]
#[doc = "    \"mean_score\","]
#[doc = "    \"track_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"boxes\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/box\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"first_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"frames_present\": {"]
#[doc = "      \"description\": \"Frames in which this face was actually detected, excluding any that were bridged. Published beside the span rather than folded into one confidence, so a stage that gates on continuity can see the two facts apart: a track present in 9 of 10 frames and one present in 3 of 10 can share a span and should not share a decision.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"last_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"mean_score\": {"]
#[doc = "      \"description\": \"Mean detector score over the frames it was seen in. Bridged boxes do not contribute — averaging in a number nobody measured is how a weak track comes to look strong.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"track_id\": {"]
#[doc = "      \"description\": \"Stable within this artifact and meaningless outside it. Two runs over the same frames assign the same ids because association is deterministic, but an id says nothing about which person it is.\","]
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
pub struct Track {
    pub boxes: ::std::vec::Vec<Box>,
    pub first_ticks: u64,
    #[doc = "Frames in which this face was actually detected, excluding any that were bridged. Published beside the span rather than folded into one confidence, so a stage that gates on continuity can see the two facts apart: a track present in 9 of 10 frames and one present in 3 of 10 can share a span and should not share a decision."]
    pub frames_present: ::std::num::NonZeroU64,
    pub last_ticks: u64,
    #[doc = "Mean detector score over the frames it was seen in. Bridged boxes do not contribute — averaging in a number nobody measured is how a weak track comes to look strong."]
    pub mean_score: f64,
    #[doc = "Stable within this artifact and meaningless outside it. Two runs over the same frames assign the same ids because association is deterministic, but an id says nothing about which person it is."]
    pub track_id: u64,
}
impl Track {
    pub fn builder() -> builder::Track {
        Default::default()
    }
}
#[doc = "Where faces were, and which of them are the same face over time. This is the only evidence the reframe solver has about who deserves the frame, so it records what was detected rather than who matters: a track carries its own scores and its own span, and the decision about which track the camera follows is made later, by something that can be argued with. Positions are normalized against the display frame so the same observation drives a crop at any resolution, which is the same reason camera speed is expressed in frame-widths per second."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.vision.face_track.v1.json\","]
#[doc = "  \"title\": \"VisionFaceTrack\","]
#[doc = "  \"description\": \"Where faces were, and which of them are the same face over time. This is the only evidence the reframe solver has about who deserves the frame, so it records what was detected rather than who matters: a track carries its own scores and its own span, and the decision about which track the camera follows is made later, by something that can be argued with. Positions are normalized against the display frame so the same observation drives a crop at any resolution, which is the same reason camera speed is expressed in frame-widths per second.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coverage\","]
#[doc = "    \"detection\","]
#[doc = "    \"frames_artifact_id\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"tracks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"detection\": {"]
#[doc = "      \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"frame_rate\","]
#[doc = "        \"input_height\","]
#[doc = "        \"input_width\","]
#[doc = "        \"match_iou\","]
#[doc = "        \"max_gap_frames\","]
#[doc = "        \"min_track_frames\","]
#[doc = "        \"nms_iou\","]
#[doc = "        \"recover_iou\","]
#[doc = "        \"score_threshold\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"frame_rate\": {"]
#[doc = "          \"description\": \"Rate the frames were sampled at, as a rational. This is the resolution of every position below: a face cannot be located in time more precisely than the frame that showed it.\","]
#[doc = "          \"$ref\": \"#/$defs/timebase\""]
#[doc = "        },"]
#[doc = "        \"input_height\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 32.0"]
#[doc = "        },"]
#[doc = "        \"input_width\": {"]
#[doc = "          \"description\": \"Width every frame was resized to for the detector. Pinned because the model's anchors are defined against it, and because a different resize produces different boxes for the same pixels.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 32.0"]
#[doc = "        },"]
#[doc = "        \"match_iou\": {"]
#[doc = "          \"description\": \"Overlap at which a detection continues an existing track. The first of two passes: confident detections claim their track before anything else is allowed to.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"exclusiveMinimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"max_gap_frames\": {"]
#[doc = "          \"description\": \"How many consecutive frames a track may go unmatched before it is closed. Longer bridges a turn of the head; too long welds two people into one track.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"min_track_frames\": {"]
#[doc = "          \"description\": \"Tracks shorter than this are discarded rather than published. A face that appeared for two frames is a detector artefact as often as it is a person, and a camera that followed one would be chasing.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"nms_iou\": {"]
#[doc = "          \"description\": \"Overlap at which two detections of the same frame are treated as one face.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"exclusiveMinimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"recover_iou\": {"]
#[doc = "          \"description\": \"The second pass, over detections too weak to start a track of their own. A face that dips below the threshold for a few frames — turned away, half-lit — keeps its identity instead of ending and starting again under a new id, which is what would make the camera cut to itself.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"exclusiveMinimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"score_threshold\": {"]
#[doc = "          \"description\": \"Detections below this are dropped before association. Lower finds more faces and more of them are not faces.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"exclusiveMinimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"frames_artifact_id\": {"]
#[doc = "      \"description\": \"The sampled frames this pass read, verified before use. Faces are detected on the frames every visual surface shares rather than on a decode of this stage's own, so two stages that disagree about what was on screen is a bug rather than a sampling difference.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.vision.face_track.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"tracks\": {"]
#[doc = "      \"description\": \"One entry per face followed, in the order they first appeared. Ordered by appearance rather than by any notion of importance, because importance is the next stage's decision and an ordering that implied it here would be that decision made by whoever had least reason to make it.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/track\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VisionFaceTrack {
    pub coverage: Coverage,
    pub detection: VisionFaceTrackDetection,
    #[doc = "The sampled frames this pass read, verified before use. Faces are detected on the frames every visual surface shares rather than on a decode of this stage's own, so two stages that disagree about what was on screen is a bug rather than a sampling difference."]
    pub frames_artifact_id: Sha256,
    pub producer: Producer,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    #[doc = "One entry per face followed, in the order they first appeared. Ordered by appearance rather than by any notion of importance, because importance is the next stage's decision and an ordering that implied it here would be that decision made by whoever had least reason to make it."]
    pub tracks: ::std::vec::Vec<Track>,
}
impl VisionFaceTrack {
    pub fn builder() -> builder::VisionFaceTrack {
        Default::default()
    }
}
#[doc = "The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The decision parameters, recorded so a later pass knows what to change rather than guessing. They are part of the artifact key: a different threshold is a different observation, not a correction of this one.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"frame_rate\","]
#[doc = "    \"input_height\","]
#[doc = "    \"input_width\","]
#[doc = "    \"match_iou\","]
#[doc = "    \"max_gap_frames\","]
#[doc = "    \"min_track_frames\","]
#[doc = "    \"nms_iou\","]
#[doc = "    \"recover_iou\","]
#[doc = "    \"score_threshold\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"frame_rate\": {"]
#[doc = "      \"description\": \"Rate the frames were sampled at, as a rational. This is the resolution of every position below: a face cannot be located in time more precisely than the frame that showed it.\","]
#[doc = "      \"$ref\": \"#/$defs/timebase\""]
#[doc = "    },"]
#[doc = "    \"input_height\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 32.0"]
#[doc = "    },"]
#[doc = "    \"input_width\": {"]
#[doc = "      \"description\": \"Width every frame was resized to for the detector. Pinned because the model's anchors are defined against it, and because a different resize produces different boxes for the same pixels.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 32.0"]
#[doc = "    },"]
#[doc = "    \"match_iou\": {"]
#[doc = "      \"description\": \"Overlap at which a detection continues an existing track. The first of two passes: confident detections claim their track before anything else is allowed to.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"max_gap_frames\": {"]
#[doc = "      \"description\": \"How many consecutive frames a track may go unmatched before it is closed. Longer bridges a turn of the head; too long welds two people into one track.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"min_track_frames\": {"]
#[doc = "      \"description\": \"Tracks shorter than this are discarded rather than published. A face that appeared for two frames is a detector artefact as often as it is a person, and a camera that followed one would be chasing.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"nms_iou\": {"]
#[doc = "      \"description\": \"Overlap at which two detections of the same frame are treated as one face.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"recover_iou\": {"]
#[doc = "      \"description\": \"The second pass, over detections too weak to start a track of their own. A face that dips below the threshold for a few frames — turned away, half-lit — keeps its identity instead of ending and starting again under a new id, which is what would make the camera cut to itself.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"score_threshold\": {"]
#[doc = "      \"description\": \"Detections below this are dropped before association. Lower finds more faces and more of them are not faces.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VisionFaceTrackDetection {
    #[doc = "Rate the frames were sampled at, as a rational. This is the resolution of every position below: a face cannot be located in time more precisely than the frame that showed it."]
    pub frame_rate: Timebase,
    pub input_height: i64,
    #[doc = "Width every frame was resized to for the detector. Pinned because the model's anchors are defined against it, and because a different resize produces different boxes for the same pixels."]
    pub input_width: i64,
    #[doc = "Overlap at which a detection continues an existing track. The first of two passes: confident detections claim their track before anything else is allowed to."]
    pub match_iou: f64,
    #[doc = "How many consecutive frames a track may go unmatched before it is closed. Longer bridges a turn of the head; too long welds two people into one track."]
    pub max_gap_frames: u64,
    #[doc = "Tracks shorter than this are discarded rather than published. A face that appeared for two frames is a detector artefact as often as it is a person, and a camera that followed one would be chasing."]
    pub min_track_frames: ::std::num::NonZeroU64,
    #[doc = "Overlap at which two detections of the same frame are treated as one face."]
    pub nms_iou: f64,
    #[doc = "The second pass, over detections too weak to start a track of their own. A face that dips below the threshold for a few frames — turned away, half-lit — keeps its identity instead of ending and starting again under a new id, which is what would make the camera cut to itself."]
    pub recover_iou: f64,
    #[doc = "Detections below this are dropped before association. Lower finds more faces and more of them are not faces."]
    pub score_threshold: f64,
}
impl VisionFaceTrackDetection {
    pub fn builder() -> builder::VisionFaceTrackDetection {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct Box {
        h: ::std::result::Result<f64, ::std::string::String>,
        interpolated: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        score: ::std::result::Result<f64, ::std::string::String>,
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
        w: ::std::result::Result<f64, ::std::string::String>,
        x: ::std::result::Result<f64, ::std::string::String>,
        y: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for Box {
        fn default() -> Self {
            Self {
                h: Err("no value supplied for h".to_string()),
                interpolated: Ok(Default::default()),
                score: Err("no value supplied for score".to_string()),
                t_ticks: Err("no value supplied for t_ticks".to_string()),
                w: Err("no value supplied for w".to_string()),
                x: Err("no value supplied for x".to_string()),
                y: Err("no value supplied for y".to_string()),
            }
        }
    }
    impl Box {
        pub fn h<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.h = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for h: {e}"));
            self
        }
        pub fn interpolated<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.interpolated = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for interpolated: {e}"));
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
        pub fn w<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.w = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for w: {e}"));
            self
        }
        pub fn x<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.x = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for x: {e}"));
            self
        }
        pub fn y<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.y = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for y: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Box> for super::Box {
        type Error = super::error::ConversionError;
        fn try_from(value: Box) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                h: value.h?,
                interpolated: value.interpolated?,
                score: value.score?,
                t_ticks: value.t_ticks?,
                w: value.w?,
                x: value.x?,
                y: value.y?,
            })
        }
    }
    impl ::std::convert::From<super::Box> for Box {
        fn from(value: super::Box) -> Self {
            Self {
                h: Ok(value.h),
                interpolated: Ok(value.interpolated),
                score: Ok(value.score),
                t_ticks: Ok(value.t_ticks),
                w: Ok(value.w),
                x: Ok(value.x),
                y: Ok(value.y),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Coverage {
        analyzed: ::std::result::Result<bool, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        frames_examined: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Coverage {
        fn default() -> Self {
            Self {
                analyzed: Err("no value supplied for analyzed".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                frames_examined: Ok(Default::default()),
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
        pub fn frames_examined<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.frames_examined = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frames_examined: {e}"));
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
                frames_examined: value.frames_examined?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Coverage> for Coverage {
        fn from(value: super::Coverage) -> Self {
            Self {
                analyzed: Ok(value.analyzed),
                end_ticks: Ok(value.end_ticks),
                frames_examined: Ok(value.frames_examined),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Producer {
        implementation: ::std::result::Result<super::ProducerImplementation, ::std::string::String>,
        model_digest:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        stage: ::std::result::Result<super::ProducerStage, ::std::string::String>,
    }
    impl ::std::default::Default for Producer {
        fn default() -> Self {
            Self {
                implementation: Err("no value supplied for implementation".to_string()),
                model_digest: Ok(Default::default()),
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
                implementation: value.implementation?,
                model_digest: value.model_digest?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::Producer> for Producer {
        fn from(value: super::Producer) -> Self {
            Self {
                implementation: Ok(value.implementation),
                model_digest: Ok(value.model_digest),
                stage: Ok(value.stage),
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
    #[derive(Clone, Debug)]
    pub struct Track {
        boxes: ::std::result::Result<::std::vec::Vec<super::Box>, ::std::string::String>,
        first_ticks: ::std::result::Result<u64, ::std::string::String>,
        frames_present: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        last_ticks: ::std::result::Result<u64, ::std::string::String>,
        mean_score: ::std::result::Result<f64, ::std::string::String>,
        track_id: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Track {
        fn default() -> Self {
            Self {
                boxes: Err("no value supplied for boxes".to_string()),
                first_ticks: Err("no value supplied for first_ticks".to_string()),
                frames_present: Err("no value supplied for frames_present".to_string()),
                last_ticks: Err("no value supplied for last_ticks".to_string()),
                mean_score: Err("no value supplied for mean_score".to_string()),
                track_id: Err("no value supplied for track_id".to_string()),
            }
        }
    }
    impl Track {
        pub fn boxes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Box>>,
            T::Error: ::std::fmt::Display,
        {
            self.boxes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for boxes: {e}"));
            self
        }
        pub fn first_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_ticks: {e}"));
            self
        }
        pub fn frames_present<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.frames_present = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frames_present: {e}"));
            self
        }
        pub fn last_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.last_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for last_ticks: {e}"));
            self
        }
        pub fn mean_score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.mean_score = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mean_score: {e}"));
            self
        }
        pub fn track_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.track_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for track_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Track> for super::Track {
        type Error = super::error::ConversionError;
        fn try_from(value: Track) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                boxes: value.boxes?,
                first_ticks: value.first_ticks?,
                frames_present: value.frames_present?,
                last_ticks: value.last_ticks?,
                mean_score: value.mean_score?,
                track_id: value.track_id?,
            })
        }
    }
    impl ::std::convert::From<super::Track> for Track {
        fn from(value: super::Track) -> Self {
            Self {
                boxes: Ok(value.boxes),
                first_ticks: Ok(value.first_ticks),
                frames_present: Ok(value.frames_present),
                last_ticks: Ok(value.last_ticks),
                mean_score: Ok(value.mean_score),
                track_id: Ok(value.track_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VisionFaceTrack {
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        detection: ::std::result::Result<super::VisionFaceTrackDetection, ::std::string::String>,
        frames_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        tracks: ::std::result::Result<::std::vec::Vec<super::Track>, ::std::string::String>,
    }
    impl ::std::default::Default for VisionFaceTrack {
        fn default() -> Self {
            Self {
                coverage: Err("no value supplied for coverage".to_string()),
                detection: Err("no value supplied for detection".to_string()),
                frames_artifact_id: Err("no value supplied for frames_artifact_id".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                tracks: Err("no value supplied for tracks".to_string()),
            }
        }
    }
    impl VisionFaceTrack {
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
        pub fn detection<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::VisionFaceTrackDetection>,
            T::Error: ::std::fmt::Display,
        {
            self.detection = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detection: {e}"));
            self
        }
        pub fn frames_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.frames_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for frames_artifact_id: {e}")
            });
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
        pub fn tracks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Track>>,
            T::Error: ::std::fmt::Display,
        {
            self.tracks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tracks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<VisionFaceTrack> for super::VisionFaceTrack {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VisionFaceTrack,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                coverage: value.coverage?,
                detection: value.detection?,
                frames_artifact_id: value.frames_artifact_id?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                tracks: value.tracks?,
            })
        }
    }
    impl ::std::convert::From<super::VisionFaceTrack> for VisionFaceTrack {
        fn from(value: super::VisionFaceTrack) -> Self {
            Self {
                coverage: Ok(value.coverage),
                detection: Ok(value.detection),
                frames_artifact_id: Ok(value.frames_artifact_id),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                tracks: Ok(value.tracks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VisionFaceTrackDetection {
        frame_rate: ::std::result::Result<super::Timebase, ::std::string::String>,
        input_height: ::std::result::Result<i64, ::std::string::String>,
        input_width: ::std::result::Result<i64, ::std::string::String>,
        match_iou: ::std::result::Result<f64, ::std::string::String>,
        max_gap_frames: ::std::result::Result<u64, ::std::string::String>,
        min_track_frames: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        nms_iou: ::std::result::Result<f64, ::std::string::String>,
        recover_iou: ::std::result::Result<f64, ::std::string::String>,
        score_threshold: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for VisionFaceTrackDetection {
        fn default() -> Self {
            Self {
                frame_rate: Err("no value supplied for frame_rate".to_string()),
                input_height: Err("no value supplied for input_height".to_string()),
                input_width: Err("no value supplied for input_width".to_string()),
                match_iou: Err("no value supplied for match_iou".to_string()),
                max_gap_frames: Err("no value supplied for max_gap_frames".to_string()),
                min_track_frames: Err("no value supplied for min_track_frames".to_string()),
                nms_iou: Err("no value supplied for nms_iou".to_string()),
                recover_iou: Err("no value supplied for recover_iou".to_string()),
                score_threshold: Err("no value supplied for score_threshold".to_string()),
            }
        }
    }
    impl VisionFaceTrackDetection {
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
        pub fn input_height<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.input_height = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for input_height: {e}"));
            self
        }
        pub fn input_width<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.input_width = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for input_width: {e}"));
            self
        }
        pub fn match_iou<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.match_iou = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for match_iou: {e}"));
            self
        }
        pub fn max_gap_frames<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_gap_frames = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max_gap_frames: {e}"));
            self
        }
        pub fn min_track_frames<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_track_frames = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for min_track_frames: {e}"));
            self
        }
        pub fn nms_iou<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.nms_iou = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for nms_iou: {e}"));
            self
        }
        pub fn recover_iou<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.recover_iou = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for recover_iou: {e}"));
            self
        }
        pub fn score_threshold<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.score_threshold = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for score_threshold: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<VisionFaceTrackDetection> for super::VisionFaceTrackDetection {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VisionFaceTrackDetection,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                frame_rate: value.frame_rate?,
                input_height: value.input_height?,
                input_width: value.input_width?,
                match_iou: value.match_iou?,
                max_gap_frames: value.max_gap_frames?,
                min_track_frames: value.min_track_frames?,
                nms_iou: value.nms_iou?,
                recover_iou: value.recover_iou?,
                score_threshold: value.score_threshold?,
            })
        }
    }
    impl ::std::convert::From<super::VisionFaceTrackDetection> for VisionFaceTrackDetection {
        fn from(value: super::VisionFaceTrackDetection) -> Self {
            Self {
                frame_rate: Ok(value.frame_rate),
                input_height: Ok(value.input_height),
                input_width: Ok(value.input_width),
                match_iou: Ok(value.match_iou),
                max_gap_frames: Ok(value.max_gap_frames),
                min_track_frames: Ok(value.min_track_frames),
                nms_iou: Ok(value.nms_iou),
                recover_iou: Ok(value.recover_iou),
                score_threshold: Ok(value.score_threshold),
            }
        }
    }
}
