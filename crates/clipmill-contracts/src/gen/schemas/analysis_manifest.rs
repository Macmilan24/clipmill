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
#[doc = "The fan-in artifact an analyze job roots: one document naming every observation produced from a source, from the probe through the ranked set. The job store roots only a job's single final artifact and garbage collection walks recipe inputs, so this manifest's recipe lists every stage — reachability of the whole analysis follows from this one root. It is also what a shell asks for when it wants to know what a project actually has: one read instead of nine."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.analysis.manifest.v1.json\","]
#[doc = "  \"title\": \"AnalysisManifest\","]
#[doc = "  \"description\": \"The fan-in artifact an analyze job roots: one document naming every observation produced from a source, from the probe through the ranked set. The job store roots only a job's single final artifact and garbage collection walks recipe inputs, so this manifest's recipe lists every stage — reachability of the whole analysis follows from this one root. It is also what a shell asks for when it wants to know what a project actually has: one read instead of nine.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coverage\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"stages\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"description\": \"The span the analysis speaks for, which is the narrowest of what its stages examined. A consumer reading a candidate outside this range is reading a claim nobody made.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"analyzed\","]
#[doc = "        \"end_ticks\","]
#[doc = "        \"start_ticks\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"analyzed\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"end_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"start_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.analysis.manifest.v1\""]
#[doc = "    },"]
#[doc = "    \"skipped\": {"]
#[doc = "      \"description\": \"Stages the plan did not run, and why. Present so that 'this recording has no shot cuts' and 'nobody looked for shot cuts' remain distinguishable at the top level, without opening any of the artifacts below.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/skipped_stage\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"stages\": {"]
#[doc = "      \"description\": \"Every stage that produced an artifact, in the order the DAG ran them. A stage that was skipped because the source lacked the input it needs is absent rather than present with a null: a recording with no video has no shot detection, and that is a different document from one whose shot detection failed.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/stage\""]
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
pub struct AnalysisManifest {
    pub coverage: AnalysisManifestCoverage,
    pub schema_version: ::serde_json::Value,
    #[doc = "Stages the plan did not run, and why. Present so that 'this recording has no shot cuts' and 'nobody looked for shot cuts' remain distinguishable at the top level, without opening any of the artifacts below."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub skipped: ::std::vec::Vec<SkippedStage>,
    pub source_fingerprint: Sha256,
    #[doc = "Every stage that produced an artifact, in the order the DAG ran them. A stage that was skipped because the source lacked the input it needs is absent rather than present with a null: a recording with no video has no shot detection, and that is a different document from one whose shot detection failed."]
    pub stages: ::std::vec::Vec<Stage>,
}
impl AnalysisManifest {
    pub fn builder() -> builder::AnalysisManifest {
        Default::default()
    }
}
#[doc = "The span the analysis speaks for, which is the narrowest of what its stages examined. A consumer reading a candidate outside this range is reading a claim nobody made."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The span the analysis speaks for, which is the narrowest of what its stages examined. A consumer reading a candidate outside this range is reading a claim nobody made.\","]
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
pub struct AnalysisManifestCoverage {
    pub analyzed: bool,
    pub end_ticks: u64,
    pub start_ticks: u64,
}
impl AnalysisManifestCoverage {
    pub fn builder() -> builder::AnalysisManifestCoverage {
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
#[doc = "`SkippedStage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"kind\","]
#[doc = "    \"reason\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"kind\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"no_video\","]
#[doc = "        \"no_audio\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SkippedStage {
    pub kind: SkippedStageKind,
    pub reason: SkippedStageReason,
}
impl SkippedStage {
    pub fn builder() -> builder::SkippedStage {
        Default::default()
    }
}
#[doc = "`SkippedStageKind`"]
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
pub struct SkippedStageKind(::std::string::String);
impl ::std::ops::Deref for SkippedStageKind {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<SkippedStageKind> for ::std::string::String {
    fn from(value: SkippedStageKind) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for SkippedStageKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for SkippedStageKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SkippedStageKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SkippedStageKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for SkippedStageKind {
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
#[doc = "`SkippedStageReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"no_video\","]
#[doc = "    \"no_audio\""]
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
pub enum SkippedStageReason {
    #[serde(rename = "no_video")]
    NoVideo,
    #[serde(rename = "no_audio")]
    NoAudio,
}
impl ::std::fmt::Display for SkippedStageReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::NoVideo => f.write_str("no_video"),
            Self::NoAudio => f.write_str("no_audio"),
        }
    }
}
impl ::std::str::FromStr for SkippedStageReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "no_video" => Ok(Self::NoVideo),
            "no_audio" => Ok(Self::NoAudio),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for SkippedStageReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SkippedStageReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SkippedStageReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Stage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"artifact_id\","]
#[doc = "    \"kind\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"evidence.source_map.v1\","]
#[doc = "        \"media.ingest_manifest.v1\","]
#[doc = "        \"speech.vad.v1\","]
#[doc = "        \"speech.asr.v1\","]
#[doc = "        \"speech.alignment.v1\","]
#[doc = "        \"speech.transcript.v1\","]
#[doc = "        \"evidence.shots.v1\","]
#[doc = "        \"index.transcript.v1\","]
#[doc = "        \"discovery.candidates.v1\","]
#[doc = "        \"ranking.set.v1\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub artifact_id: Sha256,
    pub kind: StageKind,
}
impl Stage {
    pub fn builder() -> builder::Stage {
        Default::default()
    }
}
#[doc = "`StageKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"evidence.source_map.v1\","]
#[doc = "    \"media.ingest_manifest.v1\","]
#[doc = "    \"speech.vad.v1\","]
#[doc = "    \"speech.asr.v1\","]
#[doc = "    \"speech.alignment.v1\","]
#[doc = "    \"speech.transcript.v1\","]
#[doc = "    \"evidence.shots.v1\","]
#[doc = "    \"index.transcript.v1\","]
#[doc = "    \"discovery.candidates.v1\","]
#[doc = "    \"ranking.set.v1\""]
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
pub enum StageKind {
    #[serde(rename = "evidence.source_map.v1")]
    EvidenceSourceMapV1,
    #[serde(rename = "media.ingest_manifest.v1")]
    MediaIngestManifestV1,
    #[serde(rename = "speech.vad.v1")]
    SpeechVadV1,
    #[serde(rename = "speech.asr.v1")]
    SpeechAsrV1,
    #[serde(rename = "speech.alignment.v1")]
    SpeechAlignmentV1,
    #[serde(rename = "speech.transcript.v1")]
    SpeechTranscriptV1,
    #[serde(rename = "evidence.shots.v1")]
    EvidenceShotsV1,
    #[serde(rename = "index.transcript.v1")]
    IndexTranscriptV1,
    #[serde(rename = "discovery.candidates.v1")]
    DiscoveryCandidatesV1,
    #[serde(rename = "ranking.set.v1")]
    RankingSetV1,
}
impl ::std::fmt::Display for StageKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::EvidenceSourceMapV1 => f.write_str("evidence.source_map.v1"),
            Self::MediaIngestManifestV1 => f.write_str("media.ingest_manifest.v1"),
            Self::SpeechVadV1 => f.write_str("speech.vad.v1"),
            Self::SpeechAsrV1 => f.write_str("speech.asr.v1"),
            Self::SpeechAlignmentV1 => f.write_str("speech.alignment.v1"),
            Self::SpeechTranscriptV1 => f.write_str("speech.transcript.v1"),
            Self::EvidenceShotsV1 => f.write_str("evidence.shots.v1"),
            Self::IndexTranscriptV1 => f.write_str("index.transcript.v1"),
            Self::DiscoveryCandidatesV1 => f.write_str("discovery.candidates.v1"),
            Self::RankingSetV1 => f.write_str("ranking.set.v1"),
        }
    }
}
impl ::std::str::FromStr for StageKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "evidence.source_map.v1" => Ok(Self::EvidenceSourceMapV1),
            "media.ingest_manifest.v1" => Ok(Self::MediaIngestManifestV1),
            "speech.vad.v1" => Ok(Self::SpeechVadV1),
            "speech.asr.v1" => Ok(Self::SpeechAsrV1),
            "speech.alignment.v1" => Ok(Self::SpeechAlignmentV1),
            "speech.transcript.v1" => Ok(Self::SpeechTranscriptV1),
            "evidence.shots.v1" => Ok(Self::EvidenceShotsV1),
            "index.transcript.v1" => Ok(Self::IndexTranscriptV1),
            "discovery.candidates.v1" => Ok(Self::DiscoveryCandidatesV1),
            "ranking.set.v1" => Ok(Self::RankingSetV1),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for StageKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for StageKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StageKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct AnalysisManifest {
        coverage: ::std::result::Result<super::AnalysisManifestCoverage, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        skipped: ::std::result::Result<::std::vec::Vec<super::SkippedStage>, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        stages: ::std::result::Result<::std::vec::Vec<super::Stage>, ::std::string::String>,
    }
    impl ::std::default::Default for AnalysisManifest {
        fn default() -> Self {
            Self {
                coverage: Err("no value supplied for coverage".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                skipped: Ok(Default::default()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                stages: Err("no value supplied for stages".to_string()),
            }
        }
    }
    impl AnalysisManifest {
        pub fn coverage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AnalysisManifestCoverage>,
            T::Error: ::std::fmt::Display,
        {
            self.coverage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for coverage: {e}"));
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
        pub fn skipped<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::SkippedStage>>,
            T::Error: ::std::fmt::Display,
        {
            self.skipped = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for skipped: {e}"));
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
        pub fn stages<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Stage>>,
            T::Error: ::std::fmt::Display,
        {
            self.stages = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stages: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AnalysisManifest> for super::AnalysisManifest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AnalysisManifest,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                coverage: value.coverage?,
                schema_version: value.schema_version?,
                skipped: value.skipped?,
                source_fingerprint: value.source_fingerprint?,
                stages: value.stages?,
            })
        }
    }
    impl ::std::convert::From<super::AnalysisManifest> for AnalysisManifest {
        fn from(value: super::AnalysisManifest) -> Self {
            Self {
                coverage: Ok(value.coverage),
                schema_version: Ok(value.schema_version),
                skipped: Ok(value.skipped),
                source_fingerprint: Ok(value.source_fingerprint),
                stages: Ok(value.stages),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AnalysisManifestCoverage {
        analyzed: ::std::result::Result<bool, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for AnalysisManifestCoverage {
        fn default() -> Self {
            Self {
                analyzed: Err("no value supplied for analyzed".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl AnalysisManifestCoverage {
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
    impl ::std::convert::TryFrom<AnalysisManifestCoverage> for super::AnalysisManifestCoverage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AnalysisManifestCoverage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                analyzed: value.analyzed?,
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::AnalysisManifestCoverage> for AnalysisManifestCoverage {
        fn from(value: super::AnalysisManifestCoverage) -> Self {
            Self {
                analyzed: Ok(value.analyzed),
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SkippedStage {
        kind: ::std::result::Result<super::SkippedStageKind, ::std::string::String>,
        reason: ::std::result::Result<super::SkippedStageReason, ::std::string::String>,
    }
    impl ::std::default::Default for SkippedStage {
        fn default() -> Self {
            Self {
                kind: Err("no value supplied for kind".to_string()),
                reason: Err("no value supplied for reason".to_string()),
            }
        }
    }
    impl SkippedStage {
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SkippedStageKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SkippedStageReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SkippedStage> for super::SkippedStage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SkippedStage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                kind: value.kind?,
                reason: value.reason?,
            })
        }
    }
    impl ::std::convert::From<super::SkippedStage> for SkippedStage {
        fn from(value: super::SkippedStage) -> Self {
            Self {
                kind: Ok(value.kind),
                reason: Ok(value.reason),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Stage {
        artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        kind: ::std::result::Result<super::StageKind, ::std::string::String>,
    }
    impl ::std::default::Default for Stage {
        fn default() -> Self {
            Self {
                artifact_id: Err("no value supplied for artifact_id".to_string()),
                kind: Err("no value supplied for kind".to_string()),
            }
        }
    }
    impl Stage {
        pub fn artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for artifact_id: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::StageKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Stage> for super::Stage {
        type Error = super::error::ConversionError;
        fn try_from(value: Stage) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                artifact_id: value.artifact_id?,
                kind: value.kind?,
            })
        }
    }
    impl ::std::convert::From<super::Stage> for Stage {
        fn from(value: super::Stage) -> Self {
            Self {
                artifact_id: Ok(value.artifact_id),
                kind: Ok(value.kind),
            }
        }
    }
}
