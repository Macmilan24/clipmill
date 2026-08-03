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
#[doc = "`AudioSummary`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"measured_lufs\","]
#[doc = "    \"measured_true_peak_dbtp\","]
#[doc = "    \"target_lufs\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"measured_lufs\": {"]
#[doc = "      \"description\": \"What the finished file measures, re-decoded from the output by the renderer rather than predicted from the filter's arguments.\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"measured_true_peak_dbtp\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"target_lufs\": {"]
#[doc = "      \"description\": \"What the render normalized to.\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AudioSummary {
    #[doc = "What the finished file measures, re-decoded from the output by the renderer rather than predicted from the filter's arguments."]
    pub measured_lufs: f64,
    pub measured_true_peak_dbtp: f64,
    #[doc = "What the render normalized to."]
    pub target_lufs: f64,
}
impl AudioSummary {
    pub fn builder() -> builder::AudioSummary {
        Default::default()
    }
}
#[doc = "`DeliveredFile`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"bytes\","]
#[doc = "    \"name\","]
#[doc = "    \"role\","]
#[doc = "    \"sha256\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bytes\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"description\": \"The file's name inside the export folder. A name, never a path.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1,"]
#[doc = "      \"pattern\": \"^[^/\\\\\\\\]+$\""]
#[doc = "    },"]
#[doc = "    \"role\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"clip\","]
#[doc = "        \"subtitles_srt\","]
#[doc = "        \"subtitles_vtt\","]
#[doc = "        \"thumbnail\","]
#[doc = "        \"render_manifest\","]
#[doc = "        \"metadata\","]
#[doc = "        \"checksums\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"sha256\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256_hex\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DeliveredFile {
    pub bytes: u64,
    #[doc = "The file's name inside the export folder. A name, never a path."]
    pub name: DeliveredFileName,
    pub role: DeliveredFileRole,
    pub sha256: Sha256Hex,
}
impl DeliveredFile {
    pub fn builder() -> builder::DeliveredFile {
        Default::default()
    }
}
#[doc = "The file's name inside the export folder. A name, never a path."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The file's name inside the export folder. A name, never a path.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1,"]
#[doc = "  \"pattern\": \"^[^/\\\\\\\\]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct DeliveredFileName(::std::string::String);
impl ::std::ops::Deref for DeliveredFileName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<DeliveredFileName> for ::std::string::String {
    fn from(value: DeliveredFileName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for DeliveredFileName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[^/\\\\]+$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[^/\\\\]+$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for DeliveredFileName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DeliveredFileName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DeliveredFileName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for DeliveredFileName {
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
#[doc = "`DeliveredFileRole`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"clip\","]
#[doc = "    \"subtitles_srt\","]
#[doc = "    \"subtitles_vtt\","]
#[doc = "    \"thumbnail\","]
#[doc = "    \"render_manifest\","]
#[doc = "    \"metadata\","]
#[doc = "    \"checksums\""]
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
pub enum DeliveredFileRole {
    #[serde(rename = "clip")]
    Clip,
    #[serde(rename = "subtitles_srt")]
    SubtitlesSrt,
    #[serde(rename = "subtitles_vtt")]
    SubtitlesVtt,
    #[serde(rename = "thumbnail")]
    Thumbnail,
    #[serde(rename = "render_manifest")]
    RenderManifest,
    #[serde(rename = "metadata")]
    Metadata,
    #[serde(rename = "checksums")]
    Checksums,
}
impl ::std::fmt::Display for DeliveredFileRole {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Clip => f.write_str("clip"),
            Self::SubtitlesSrt => f.write_str("subtitles_srt"),
            Self::SubtitlesVtt => f.write_str("subtitles_vtt"),
            Self::Thumbnail => f.write_str("thumbnail"),
            Self::RenderManifest => f.write_str("render_manifest"),
            Self::Metadata => f.write_str("metadata"),
            Self::Checksums => f.write_str("checksums"),
        }
    }
}
impl ::std::str::FromStr for DeliveredFileRole {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "clip" => Ok(Self::Clip),
            "subtitles_srt" => Ok(Self::SubtitlesSrt),
            "subtitles_vtt" => Ok(Self::SubtitlesVtt),
            "thumbnail" => Ok(Self::Thumbnail),
            "render_manifest" => Ok(Self::RenderManifest),
            "metadata" => Ok(Self::Metadata),
            "checksums" => Ok(Self::Checksums),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for DeliveredFileRole {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DeliveredFileRole {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DeliveredFileRole {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "The rights claim and the model-use disclosure, echoed from what the user attested at render time. Carried in the package rather than left to be recalled at upload time, because by then the person answering the platform's question may not be the person who made the clip."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The rights claim and the model-use disclosure, echoed from what the user attested at render time. Carried in the package rather than left to be recalled at upload time, because by then the person answering the platform's question may not be the person who made the clip.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"ai_assistance\","]
#[doc = "    \"gates_passed\","]
#[doc = "    \"requires_ai_disclosure\","]
#[doc = "    \"source_attestation\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"ai_assistance\": {"]
#[doc = "      \"description\": \"Model work that shaped the footage, e.g. asr_captions or reframe. Empty for a hand-authored document.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"gates_passed\": {"]
#[doc = "      \"description\": \"Confirmations the user gave, e.g. duration_60s for a clip past the minute mark.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"requires_ai_disclosure\": {"]
#[doc = "      \"description\": \"Whether a platform's synthetic-media question should be answered yes.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"source_attestation\": {"]
#[doc = "      \"description\": \"What the user said the footage is, e.g. own_content.\","]
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
pub struct Disclosure {
    #[doc = "Model work that shaped the footage, e.g. asr_captions or reframe. Empty for a hand-authored document."]
    pub ai_assistance: ::std::vec::Vec<DisclosureAiAssistanceItem>,
    #[doc = "Confirmations the user gave, e.g. duration_60s for a clip past the minute mark."]
    pub gates_passed: ::std::vec::Vec<DisclosureGatesPassedItem>,
    #[doc = "Whether a platform's synthetic-media question should be answered yes."]
    pub requires_ai_disclosure: bool,
    #[doc = "What the user said the footage is, e.g. own_content."]
    pub source_attestation: DisclosureSourceAttestation,
}
impl Disclosure {
    pub fn builder() -> builder::Disclosure {
        Default::default()
    }
}
#[doc = "`DisclosureAiAssistanceItem`"]
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
pub struct DisclosureAiAssistanceItem(::std::string::String);
impl ::std::ops::Deref for DisclosureAiAssistanceItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<DisclosureAiAssistanceItem> for ::std::string::String {
    fn from(value: DisclosureAiAssistanceItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for DisclosureAiAssistanceItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for DisclosureAiAssistanceItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DisclosureAiAssistanceItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DisclosureAiAssistanceItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for DisclosureAiAssistanceItem {
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
#[doc = "`DisclosureGatesPassedItem`"]
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
pub struct DisclosureGatesPassedItem(::std::string::String);
impl ::std::ops::Deref for DisclosureGatesPassedItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<DisclosureGatesPassedItem> for ::std::string::String {
    fn from(value: DisclosureGatesPassedItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for DisclosureGatesPassedItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for DisclosureGatesPassedItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DisclosureGatesPassedItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DisclosureGatesPassedItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for DisclosureGatesPassedItem {
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
#[doc = "What the user said the footage is, e.g. own_content."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What the user said the footage is, e.g. own_content.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct DisclosureSourceAttestation(::std::string::String);
impl ::std::ops::Deref for DisclosureSourceAttestation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<DisclosureSourceAttestation> for ::std::string::String {
    fn from(value: DisclosureSourceAttestation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for DisclosureSourceAttestation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for DisclosureSourceAttestation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DisclosureSourceAttestation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DisclosureSourceAttestation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for DisclosureSourceAttestation {
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
#[doc = "The metadata document that ships beside a delivered clip. An export is not one file — it is the clip, two sidecars, a thumbnail, a copy of the render manifest, and this, which answers the questions an upload form asks without the user opening anything else. It restates rather than reinterprets: every number here was measured by the renderer and recorded in the render manifest, and the disclosure fields are echoed verbatim from what the user attested. A second computation would be a second chance to disagree, and on the disclosure fields a disagreement is not a bug but a false statement about the work."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.export.package.v1.json\","]
#[doc = "  \"title\": \"ExportPackage\","]
#[doc = "  \"description\": \"The metadata document that ships beside a delivered clip. An export is not one file — it is the clip, two sidecars, a thumbnail, a copy of the render manifest, and this, which answers the questions an upload form asks without the user opening anything else. It restates rather than reinterprets: every number here was measured by the renderer and recorded in the render manifest, and the disclosure fields are echoed verbatim from what the user attested. A second computation would be a second chance to disagree, and on the disclosure fields a disagreement is not a bug but a false statement about the work.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"audio\","]
#[doc = "    \"disclosure\","]
#[doc = "    \"doc_id\","]
#[doc = "    \"files\","]
#[doc = "    \"render_artifact_id\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"title\","]
#[doc = "    \"video\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"audio\": {"]
#[doc = "      \"$ref\": \"#/$defs/audio_summary\""]
#[doc = "    },"]
#[doc = "    \"disclosure\": {"]
#[doc = "      \"$ref\": \"#/$defs/disclosure\""]
#[doc = "    },"]
#[doc = "    \"doc_id\": {"]
#[doc = "      \"description\": \"The edit document this clip was rendered from.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"files\": {"]
#[doc = "      \"description\": \"Every file in the delivery, sorted by name so two exports of the same clip agree on order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/delivered_file\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"render_artifact_id\": {"]
#[doc = "      \"description\": \"Content address of the render this was delivered from, so a package on a disk can be traced back to the artifact that produced it.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.export.package.v1\""]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"description\": \"The clip's own words, when it has a title. Empty when it does not — an untitled clip is a fact, not a blank to be filled with a filename.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"video\": {"]
#[doc = "      \"$ref\": \"#/$defs/video_summary\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExportPackage {
    pub audio: AudioSummary,
    pub disclosure: Disclosure,
    #[doc = "The edit document this clip was rendered from."]
    pub doc_id: ExportPackageDocId,
    #[doc = "Every file in the delivery, sorted by name so two exports of the same clip agree on order."]
    pub files: ::std::vec::Vec<DeliveredFile>,
    #[doc = "Content address of the render this was delivered from, so a package on a disk can be traced back to the artifact that produced it."]
    pub render_artifact_id: ExportPackageRenderArtifactId,
    pub schema_version: ::serde_json::Value,
    #[doc = "The clip's own words, when it has a title. Empty when it does not — an untitled clip is a fact, not a blank to be filled with a filename."]
    pub title: ::std::string::String,
    pub video: VideoSummary,
}
impl ExportPackage {
    pub fn builder() -> builder::ExportPackage {
        Default::default()
    }
}
#[doc = "The edit document this clip was rendered from."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The edit document this clip was rendered from.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ExportPackageDocId(::std::string::String);
impl ::std::ops::Deref for ExportPackageDocId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ExportPackageDocId> for ::std::string::String {
    fn from(value: ExportPackageDocId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ExportPackageDocId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ExportPackageDocId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ExportPackageDocId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExportPackageDocId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ExportPackageDocId {
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
#[doc = "Content address of the render this was delivered from, so a package on a disk can be traced back to the artifact that produced it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Content address of the render this was delivered from, so a package on a disk can be traced back to the artifact that produced it.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ExportPackageRenderArtifactId(::std::string::String);
impl ::std::ops::Deref for ExportPackageRenderArtifactId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ExportPackageRenderArtifactId> for ::std::string::String {
    fn from(value: ExportPackageRenderArtifactId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ExportPackageRenderArtifactId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ExportPackageRenderArtifactId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ExportPackageRenderArtifactId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExportPackageRenderArtifactId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ExportPackageRenderArtifactId {
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
#[doc = "Lower-case hex, unprefixed, matching the sha256sum-compatible checksum file delivered alongside."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Lower-case hex, unprefixed, matching the sha256sum-compatible checksum file delivered alongside.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[0-9a-f]{64}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Sha256Hex(::std::string::String);
impl ::std::ops::Deref for Sha256Hex {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Sha256Hex> for ::std::string::String {
    fn from(value: Sha256Hex) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Sha256Hex {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[0-9a-f]{64}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[0-9a-f]{64}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for Sha256Hex {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Sha256Hex {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Sha256Hex {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Sha256Hex {
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
#[doc = "`VideoSummary`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"duration_ticks\","]
#[doc = "    \"frame_count\","]
#[doc = "    \"frame_rate_den\","]
#[doc = "    \"frame_rate_num\","]
#[doc = "    \"height\","]
#[doc = "    \"width\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"duration_ticks\": {"]
#[doc = "      \"description\": \"Program length in ticks of the document's timebase (decision D06).\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"frame_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"frame_rate_den\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"frame_rate_num\": {"]
#[doc = "      \"description\": \"Frame rate as a rational, never a decimal: 30000/1001 is not 29.97 and the difference is a frame per half hour.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"height\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"width\": {"]
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
pub struct VideoSummary {
    #[doc = "Program length in ticks of the document's timebase (decision D06)."]
    pub duration_ticks: u64,
    pub frame_count: u64,
    pub frame_rate_den: ::std::num::NonZeroU64,
    #[doc = "Frame rate as a rational, never a decimal: 30000/1001 is not 29.97 and the difference is a frame per half hour."]
    pub frame_rate_num: ::std::num::NonZeroU64,
    pub height: ::std::num::NonZeroU64,
    pub width: ::std::num::NonZeroU64,
}
impl VideoSummary {
    pub fn builder() -> builder::VideoSummary {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct AudioSummary {
        measured_lufs: ::std::result::Result<f64, ::std::string::String>,
        measured_true_peak_dbtp: ::std::result::Result<f64, ::std::string::String>,
        target_lufs: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for AudioSummary {
        fn default() -> Self {
            Self {
                measured_lufs: Err("no value supplied for measured_lufs".to_string()),
                measured_true_peak_dbtp: Err(
                    "no value supplied for measured_true_peak_dbtp".to_string()
                ),
                target_lufs: Err("no value supplied for target_lufs".to_string()),
            }
        }
    }
    impl AudioSummary {
        pub fn measured_lufs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.measured_lufs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for measured_lufs: {e}"));
            self
        }
        pub fn measured_true_peak_dbtp<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.measured_true_peak_dbtp = value.try_into().map_err(|e| {
                format!("error converting supplied value for measured_true_peak_dbtp: {e}")
            });
            self
        }
        pub fn target_lufs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.target_lufs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for target_lufs: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AudioSummary> for super::AudioSummary {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AudioSummary,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                measured_lufs: value.measured_lufs?,
                measured_true_peak_dbtp: value.measured_true_peak_dbtp?,
                target_lufs: value.target_lufs?,
            })
        }
    }
    impl ::std::convert::From<super::AudioSummary> for AudioSummary {
        fn from(value: super::AudioSummary) -> Self {
            Self {
                measured_lufs: Ok(value.measured_lufs),
                measured_true_peak_dbtp: Ok(value.measured_true_peak_dbtp),
                target_lufs: Ok(value.target_lufs),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeliveredFile {
        bytes: ::std::result::Result<u64, ::std::string::String>,
        name: ::std::result::Result<super::DeliveredFileName, ::std::string::String>,
        role: ::std::result::Result<super::DeliveredFileRole, ::std::string::String>,
        sha256: ::std::result::Result<super::Sha256Hex, ::std::string::String>,
    }
    impl ::std::default::Default for DeliveredFile {
        fn default() -> Self {
            Self {
                bytes: Err("no value supplied for bytes".to_string()),
                name: Err("no value supplied for name".to_string()),
                role: Err("no value supplied for role".to_string()),
                sha256: Err("no value supplied for sha256".to_string()),
            }
        }
    }
    impl DeliveredFile {
        pub fn bytes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.bytes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for bytes: {e}"));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeliveredFileName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn role<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeliveredFileRole>,
            T::Error: ::std::fmt::Display,
        {
            self.role = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for role: {e}"));
            self
        }
        pub fn sha256<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256Hex>,
            T::Error: ::std::fmt::Display,
        {
            self.sha256 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sha256: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeliveredFile> for super::DeliveredFile {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeliveredFile,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                bytes: value.bytes?,
                name: value.name?,
                role: value.role?,
                sha256: value.sha256?,
            })
        }
    }
    impl ::std::convert::From<super::DeliveredFile> for DeliveredFile {
        fn from(value: super::DeliveredFile) -> Self {
            Self {
                bytes: Ok(value.bytes),
                name: Ok(value.name),
                role: Ok(value.role),
                sha256: Ok(value.sha256),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Disclosure {
        ai_assistance: ::std::result::Result<
            ::std::vec::Vec<super::DisclosureAiAssistanceItem>,
            ::std::string::String,
        >,
        gates_passed: ::std::result::Result<
            ::std::vec::Vec<super::DisclosureGatesPassedItem>,
            ::std::string::String,
        >,
        requires_ai_disclosure: ::std::result::Result<bool, ::std::string::String>,
        source_attestation:
            ::std::result::Result<super::DisclosureSourceAttestation, ::std::string::String>,
    }
    impl ::std::default::Default for Disclosure {
        fn default() -> Self {
            Self {
                ai_assistance: Err("no value supplied for ai_assistance".to_string()),
                gates_passed: Err("no value supplied for gates_passed".to_string()),
                requires_ai_disclosure: Err(
                    "no value supplied for requires_ai_disclosure".to_string()
                ),
                source_attestation: Err("no value supplied for source_attestation".to_string()),
            }
        }
    }
    impl Disclosure {
        pub fn ai_assistance<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::DisclosureAiAssistanceItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.ai_assistance = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for ai_assistance: {e}"));
            self
        }
        pub fn gates_passed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::DisclosureGatesPassedItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.gates_passed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for gates_passed: {e}"));
            self
        }
        pub fn requires_ai_disclosure<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.requires_ai_disclosure = value.try_into().map_err(|e| {
                format!("error converting supplied value for requires_ai_disclosure: {e}")
            });
            self
        }
        pub fn source_attestation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DisclosureSourceAttestation>,
            T::Error: ::std::fmt::Display,
        {
            self.source_attestation = value.try_into().map_err(|e| {
                format!("error converting supplied value for source_attestation: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<Disclosure> for super::Disclosure {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Disclosure,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                ai_assistance: value.ai_assistance?,
                gates_passed: value.gates_passed?,
                requires_ai_disclosure: value.requires_ai_disclosure?,
                source_attestation: value.source_attestation?,
            })
        }
    }
    impl ::std::convert::From<super::Disclosure> for Disclosure {
        fn from(value: super::Disclosure) -> Self {
            Self {
                ai_assistance: Ok(value.ai_assistance),
                gates_passed: Ok(value.gates_passed),
                requires_ai_disclosure: Ok(value.requires_ai_disclosure),
                source_attestation: Ok(value.source_attestation),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ExportPackage {
        audio: ::std::result::Result<super::AudioSummary, ::std::string::String>,
        disclosure: ::std::result::Result<super::Disclosure, ::std::string::String>,
        doc_id: ::std::result::Result<super::ExportPackageDocId, ::std::string::String>,
        files: ::std::result::Result<::std::vec::Vec<super::DeliveredFile>, ::std::string::String>,
        render_artifact_id:
            ::std::result::Result<super::ExportPackageRenderArtifactId, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        title: ::std::result::Result<::std::string::String, ::std::string::String>,
        video: ::std::result::Result<super::VideoSummary, ::std::string::String>,
    }
    impl ::std::default::Default for ExportPackage {
        fn default() -> Self {
            Self {
                audio: Err("no value supplied for audio".to_string()),
                disclosure: Err("no value supplied for disclosure".to_string()),
                doc_id: Err("no value supplied for doc_id".to_string()),
                files: Err("no value supplied for files".to_string()),
                render_artifact_id: Err("no value supplied for render_artifact_id".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                title: Err("no value supplied for title".to_string()),
                video: Err("no value supplied for video".to_string()),
            }
        }
    }
    impl ExportPackage {
        pub fn audio<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AudioSummary>,
            T::Error: ::std::fmt::Display,
        {
            self.audio = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio: {e}"));
            self
        }
        pub fn disclosure<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Disclosure>,
            T::Error: ::std::fmt::Display,
        {
            self.disclosure = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for disclosure: {e}"));
            self
        }
        pub fn doc_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ExportPackageDocId>,
            T::Error: ::std::fmt::Display,
        {
            self.doc_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for doc_id: {e}"));
            self
        }
        pub fn files<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::DeliveredFile>>,
            T::Error: ::std::fmt::Display,
        {
            self.files = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for files: {e}"));
            self
        }
        pub fn render_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ExportPackageRenderArtifactId>,
            T::Error: ::std::fmt::Display,
        {
            self.render_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for render_artifact_id: {e}")
            });
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
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {e}"));
            self
        }
        pub fn video<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::VideoSummary>,
            T::Error: ::std::fmt::Display,
        {
            self.video = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for video: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ExportPackage> for super::ExportPackage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ExportPackage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                audio: value.audio?,
                disclosure: value.disclosure?,
                doc_id: value.doc_id?,
                files: value.files?,
                render_artifact_id: value.render_artifact_id?,
                schema_version: value.schema_version?,
                title: value.title?,
                video: value.video?,
            })
        }
    }
    impl ::std::convert::From<super::ExportPackage> for ExportPackage {
        fn from(value: super::ExportPackage) -> Self {
            Self {
                audio: Ok(value.audio),
                disclosure: Ok(value.disclosure),
                doc_id: Ok(value.doc_id),
                files: Ok(value.files),
                render_artifact_id: Ok(value.render_artifact_id),
                schema_version: Ok(value.schema_version),
                title: Ok(value.title),
                video: Ok(value.video),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VideoSummary {
        duration_ticks: ::std::result::Result<u64, ::std::string::String>,
        frame_count: ::std::result::Result<u64, ::std::string::String>,
        frame_rate_den: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        frame_rate_num: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        height: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        width: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for VideoSummary {
        fn default() -> Self {
            Self {
                duration_ticks: Err("no value supplied for duration_ticks".to_string()),
                frame_count: Err("no value supplied for frame_count".to_string()),
                frame_rate_den: Err("no value supplied for frame_rate_den".to_string()),
                frame_rate_num: Err("no value supplied for frame_rate_num".to_string()),
                height: Err("no value supplied for height".to_string()),
                width: Err("no value supplied for width".to_string()),
            }
        }
    }
    impl VideoSummary {
        pub fn duration_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.duration_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for duration_ticks: {e}"));
            self
        }
        pub fn frame_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.frame_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frame_count: {e}"));
            self
        }
        pub fn frame_rate_den<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.frame_rate_den = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frame_rate_den: {e}"));
            self
        }
        pub fn frame_rate_num<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.frame_rate_num = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frame_rate_num: {e}"));
            self
        }
        pub fn height<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.height = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for height: {e}"));
            self
        }
        pub fn width<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.width = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for width: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<VideoSummary> for super::VideoSummary {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VideoSummary,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                duration_ticks: value.duration_ticks?,
                frame_count: value.frame_count?,
                frame_rate_den: value.frame_rate_den?,
                frame_rate_num: value.frame_rate_num?,
                height: value.height?,
                width: value.width?,
            })
        }
    }
    impl ::std::convert::From<super::VideoSummary> for VideoSummary {
        fn from(value: super::VideoSummary) -> Self {
            Self {
                duration_ticks: Ok(value.duration_ticks),
                frame_count: Ok(value.frame_count),
                frame_rate_den: Ok(value.frame_rate_den),
                frame_rate_num: Ok(value.frame_rate_num),
                height: Ok(value.height),
                width: Ok(value.width),
            }
        }
    }
}
