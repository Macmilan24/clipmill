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
#[doc = "What was examined. A recording nobody listened to has no speakers for a different reason than one where nobody spoke."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What was examined. A recording nobody listened to has no speakers for a different reason than one where nobody spoke.\","]
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
#[doc = "`Speaker`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"first_ticks\","]
#[doc = "    \"speaker_id\","]
#[doc = "    \"speech_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"first_ticks\": {"]
#[doc = "      \"description\": \"Where it is first heard.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"speaker_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/speaker_id\""]
#[doc = "    },"]
#[doc = "    \"speech_ticks\": {"]
#[doc = "      \"description\": \"How long this voice is heard in all.\","]
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
pub struct Speaker {
    #[doc = "Where it is first heard."]
    pub first_ticks: u64,
    pub speaker_id: SpeakerId,
    #[doc = "How long this voice is heard in all."]
    pub speech_ticks: u64,
}
impl Speaker {
    pub fn builder() -> builder::Speaker {
        Default::default()
    }
}
#[doc = "`SpeakerId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^spk_[1-9][0-9]*$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct SpeakerId(::std::string::String);
impl ::std::ops::Deref for SpeakerId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<SpeakerId> for ::std::string::String {
    fn from(value: SpeakerId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for SpeakerId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^spk_[1-9][0-9]*$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^spk_[1-9][0-9]*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for SpeakerId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SpeakerId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SpeakerId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for SpeakerId {
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
#[doc = "Who speaks when: the recording's speech told apart by voice (diarization). Voices are named in the order they are first heard — spk_1 is whoever speaks first — so the same recording read twice names them the same. A voice is a cluster of how people sound, not an identity: two voices can be one person heard through two microphones, and naming them is left to the person. Every interval is integer ticks at 1/90000, in source time."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.speech.speakers.v1.json\","]
#[doc = "  \"title\": \"SpeechSpeakers\","]
#[doc = "  \"description\": \"Who speaks when: the recording's speech told apart by voice (diarization). Voices are named in the order they are first heard — spk_1 is whoever speaks first — so the same recording read twice names them the same. A voice is a cluster of how people sound, not an identity: two voices can be one person heard through two microphones, and naming them is left to the person. Every interval is integer ticks at 1/90000, in source time.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"audio_artifact_id\","]
#[doc = "    \"clustering\","]
#[doc = "    \"coverage\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"speakers\","]
#[doc = "    \"turns\","]
#[doc = "    \"vad_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"audio_artifact_id\": {"]
#[doc = "      \"description\": \"The 16 kHz mono rendition this pass read.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"clustering\": {"]
#[doc = "      \"description\": \"How voices were told apart, recorded because a different setting is a different observation.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"hop_ticks\","]
#[doc = "        \"link\","]
#[doc = "        \"merge\","]
#[doc = "        \"smallest\","]
#[doc = "        \"window_ticks\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"hop_ticks\": {"]
#[doc = "          \"description\": \"How far apart consecutive prints start.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"link\": {"]
#[doc = "          \"description\": \"Average similarity at or above which two groups of prints are first joined.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": -1.0"]
#[doc = "        },"]
#[doc = "        \"merge\": {"]
#[doc = "          \"description\": \"Similarity of two voices' centres at or above which they are one voice.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": -1.0"]
#[doc = "        },"]
#[doc = "        \"smallest\": {"]
#[doc = "          \"description\": \"The least share of the prints a voice must hold to be kept; a smaller one is folded into the voice nearest it.\","]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 1.0,"]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"window_ticks\": {"]
#[doc = "          \"description\": \"The span of speech each voice print is taken over.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.speech.speakers.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"speakers\": {"]
#[doc = "      \"description\": \"Every voice heard, in the order first heard.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/speaker\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"turns\": {"]
#[doc = "      \"description\": \"Speech by voice, ordered and non-overlapping. A turn may span a pause the same voice resumes after; silence between two voices belongs to neither.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/turn\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"vad_artifact_id\": {"]
#[doc = "      \"description\": \"The voice activity whose speech segments were told apart. Silence has no speaker.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpeechSpeakers {
    #[doc = "The 16 kHz mono rendition this pass read."]
    pub audio_artifact_id: Sha256,
    pub clustering: SpeechSpeakersClustering,
    pub coverage: Coverage,
    pub producer: Producer,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    #[doc = "Every voice heard, in the order first heard."]
    pub speakers: ::std::vec::Vec<Speaker>,
    #[doc = "Speech by voice, ordered and non-overlapping. A turn may span a pause the same voice resumes after; silence between two voices belongs to neither."]
    pub turns: ::std::vec::Vec<Turn>,
    #[doc = "The voice activity whose speech segments were told apart. Silence has no speaker."]
    pub vad_artifact_id: Sha256,
}
impl SpeechSpeakers {
    pub fn builder() -> builder::SpeechSpeakers {
        Default::default()
    }
}
#[doc = "How voices were told apart, recorded because a different setting is a different observation."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How voices were told apart, recorded because a different setting is a different observation.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"hop_ticks\","]
#[doc = "    \"link\","]
#[doc = "    \"merge\","]
#[doc = "    \"smallest\","]
#[doc = "    \"window_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"hop_ticks\": {"]
#[doc = "      \"description\": \"How far apart consecutive prints start.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"link\": {"]
#[doc = "      \"description\": \"Average similarity at or above which two groups of prints are first joined.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": -1.0"]
#[doc = "    },"]
#[doc = "    \"merge\": {"]
#[doc = "      \"description\": \"Similarity of two voices' centres at or above which they are one voice.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": -1.0"]
#[doc = "    },"]
#[doc = "    \"smallest\": {"]
#[doc = "      \"description\": \"The least share of the prints a voice must hold to be kept; a smaller one is folded into the voice nearest it.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"window_ticks\": {"]
#[doc = "      \"description\": \"The span of speech each voice print is taken over.\","]
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
pub struct SpeechSpeakersClustering {
    #[doc = "How far apart consecutive prints start."]
    pub hop_ticks: ::std::num::NonZeroU64,
    #[doc = "Average similarity at or above which two groups of prints are first joined."]
    pub link: f64,
    #[doc = "Similarity of two voices' centres at or above which they are one voice."]
    pub merge: f64,
    #[doc = "The least share of the prints a voice must hold to be kept; a smaller one is folded into the voice nearest it."]
    pub smallest: f64,
    #[doc = "The span of speech each voice print is taken over."]
    pub window_ticks: ::std::num::NonZeroU64,
}
impl SpeechSpeakersClustering {
    pub fn builder() -> builder::SpeechSpeakersClustering {
        Default::default()
    }
}
#[doc = "`Turn`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"speaker_id\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"speaker_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/speaker_id\""]
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
pub struct Turn {
    pub end_ticks: ::std::num::NonZeroU64,
    pub speaker_id: SpeakerId,
    pub start_ticks: u64,
}
impl Turn {
    pub fn builder() -> builder::Turn {
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
    pub struct Speaker {
        first_ticks: ::std::result::Result<u64, ::std::string::String>,
        speaker_id: ::std::result::Result<super::SpeakerId, ::std::string::String>,
        speech_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Speaker {
        fn default() -> Self {
            Self {
                first_ticks: Err("no value supplied for first_ticks".to_string()),
                speaker_id: Err("no value supplied for speaker_id".to_string()),
                speech_ticks: Err("no value supplied for speech_ticks".to_string()),
            }
        }
    }
    impl Speaker {
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
        pub fn speaker_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SpeakerId>,
            T::Error: ::std::fmt::Display,
        {
            self.speaker_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for speaker_id: {e}"));
            self
        }
        pub fn speech_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.speech_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for speech_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Speaker> for super::Speaker {
        type Error = super::error::ConversionError;
        fn try_from(value: Speaker) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                first_ticks: value.first_ticks?,
                speaker_id: value.speaker_id?,
                speech_ticks: value.speech_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Speaker> for Speaker {
        fn from(value: super::Speaker) -> Self {
            Self {
                first_ticks: Ok(value.first_ticks),
                speaker_id: Ok(value.speaker_id),
                speech_ticks: Ok(value.speech_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SpeechSpeakers {
        audio_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        clustering: ::std::result::Result<super::SpeechSpeakersClustering, ::std::string::String>,
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        producer: ::std::result::Result<super::Producer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        speakers: ::std::result::Result<::std::vec::Vec<super::Speaker>, ::std::string::String>,
        turns: ::std::result::Result<::std::vec::Vec<super::Turn>, ::std::string::String>,
        vad_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechSpeakers {
        fn default() -> Self {
            Self {
                audio_artifact_id: Err("no value supplied for audio_artifact_id".to_string()),
                clustering: Err("no value supplied for clustering".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                speakers: Err("no value supplied for speakers".to_string()),
                turns: Err("no value supplied for turns".to_string()),
                vad_artifact_id: Err("no value supplied for vad_artifact_id".to_string()),
            }
        }
    }
    impl SpeechSpeakers {
        pub fn audio_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.audio_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio_artifact_id: {e}"));
            self
        }
        pub fn clustering<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SpeechSpeakersClustering>,
            T::Error: ::std::fmt::Display,
        {
            self.clustering = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for clustering: {e}"));
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
        pub fn speakers<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Speaker>>,
            T::Error: ::std::fmt::Display,
        {
            self.speakers = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for speakers: {e}"));
            self
        }
        pub fn turns<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Turn>>,
            T::Error: ::std::fmt::Display,
        {
            self.turns = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for turns: {e}"));
            self
        }
        pub fn vad_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.vad_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for vad_artifact_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SpeechSpeakers> for super::SpeechSpeakers {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechSpeakers,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                audio_artifact_id: value.audio_artifact_id?,
                clustering: value.clustering?,
                coverage: value.coverage?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                speakers: value.speakers?,
                turns: value.turns?,
                vad_artifact_id: value.vad_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechSpeakers> for SpeechSpeakers {
        fn from(value: super::SpeechSpeakers) -> Self {
            Self {
                audio_artifact_id: Ok(value.audio_artifact_id),
                clustering: Ok(value.clustering),
                coverage: Ok(value.coverage),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                speakers: Ok(value.speakers),
                turns: Ok(value.turns),
                vad_artifact_id: Ok(value.vad_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SpeechSpeakersClustering {
        hop_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        link: ::std::result::Result<f64, ::std::string::String>,
        merge: ::std::result::Result<f64, ::std::string::String>,
        smallest: ::std::result::Result<f64, ::std::string::String>,
        window_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for SpeechSpeakersClustering {
        fn default() -> Self {
            Self {
                hop_ticks: Err("no value supplied for hop_ticks".to_string()),
                link: Err("no value supplied for link".to_string()),
                merge: Err("no value supplied for merge".to_string()),
                smallest: Err("no value supplied for smallest".to_string()),
                window_ticks: Err("no value supplied for window_ticks".to_string()),
            }
        }
    }
    impl SpeechSpeakersClustering {
        pub fn hop_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.hop_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hop_ticks: {e}"));
            self
        }
        pub fn link<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.link = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for link: {e}"));
            self
        }
        pub fn merge<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.merge = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for merge: {e}"));
            self
        }
        pub fn smallest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.smallest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for smallest: {e}"));
            self
        }
        pub fn window_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.window_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for window_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SpeechSpeakersClustering> for super::SpeechSpeakersClustering {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SpeechSpeakersClustering,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                hop_ticks: value.hop_ticks?,
                link: value.link?,
                merge: value.merge?,
                smallest: value.smallest?,
                window_ticks: value.window_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::SpeechSpeakersClustering> for SpeechSpeakersClustering {
        fn from(value: super::SpeechSpeakersClustering) -> Self {
            Self {
                hop_ticks: Ok(value.hop_ticks),
                link: Ok(value.link),
                merge: Ok(value.merge),
                smallest: Ok(value.smallest),
                window_ticks: Ok(value.window_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Turn {
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        speaker_id: ::std::result::Result<super::SpeakerId, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Turn {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                speaker_id: Err("no value supplied for speaker_id".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Turn {
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
        pub fn speaker_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SpeakerId>,
            T::Error: ::std::fmt::Display,
        {
            self.speaker_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for speaker_id: {e}"));
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
    impl ::std::convert::TryFrom<Turn> for super::Turn {
        type Error = super::error::ConversionError;
        fn try_from(value: Turn) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                speaker_id: value.speaker_id?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Turn> for Turn {
        fn from(value: super::Turn) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                speaker_id: Ok(value.speaker_id),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
}
