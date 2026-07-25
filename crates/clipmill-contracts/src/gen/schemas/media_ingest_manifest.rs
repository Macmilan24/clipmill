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
#[doc = "The fan-in artifact an ingest job roots (book ch. 12): one document naming every derivative produced from a source. The job store roots only a job's single final artifact, and garbage collection walks recipe inputs, so this manifest's recipe lists every child — reachability of the whole fan-out follows from this one root."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.media.ingest_manifest.v1.json\","]
#[doc = "  \"title\": \"MediaIngestManifest\","]
#[doc = "  \"description\": \"The fan-in artifact an ingest job roots (book ch. 12): one document naming every derivative produced from a source. The job store roots only a job's single final artifact, and garbage collection walks recipe inputs, so this manifest's recipe lists every child — reachability of the whole fan-out follows from this one root.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"children\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"children\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"artifact_id\","]
#[doc = "          \"kind\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"artifact_id\": {"]
#[doc = "            \"$ref\": \"#/$defs/sha256\""]
#[doc = "          },"]
#[doc = "          \"kind\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"media.proxy.v1\","]
#[doc = "              \"media.audio_16k.v1\","]
#[doc = "              \"media.audio_48k.v1\","]
#[doc = "              \"media.loudness_envelope.v1\","]
#[doc = "              \"media.reference_index.v1\","]
#[doc = "              \"media.filmstrip.v1\","]
#[doc = "              \"media.audio_peaks.v1\","]
#[doc = "              \"media.frames.v1\""]
#[doc = "            ]"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.media.ingest_manifest.v1\""]
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
pub struct MediaIngestManifest {
    pub children: ::std::vec::Vec<MediaIngestManifestChildrenItem>,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
}
impl MediaIngestManifest {
    pub fn builder() -> builder::MediaIngestManifest {
        Default::default()
    }
}
#[doc = "`MediaIngestManifestChildrenItem`"]
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
#[doc = "      \"enum\": ["]
#[doc = "        \"media.proxy.v1\","]
#[doc = "        \"media.audio_16k.v1\","]
#[doc = "        \"media.audio_48k.v1\","]
#[doc = "        \"media.loudness_envelope.v1\","]
#[doc = "        \"media.reference_index.v1\","]
#[doc = "        \"media.filmstrip.v1\","]
#[doc = "        \"media.audio_peaks.v1\","]
#[doc = "        \"media.frames.v1\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MediaIngestManifestChildrenItem {
    pub artifact_id: Sha256,
    pub kind: MediaIngestManifestChildrenItemKind,
}
impl MediaIngestManifestChildrenItem {
    pub fn builder() -> builder::MediaIngestManifestChildrenItem {
        Default::default()
    }
}
#[doc = "`MediaIngestManifestChildrenItemKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"media.proxy.v1\","]
#[doc = "    \"media.audio_16k.v1\","]
#[doc = "    \"media.audio_48k.v1\","]
#[doc = "    \"media.loudness_envelope.v1\","]
#[doc = "    \"media.reference_index.v1\","]
#[doc = "    \"media.filmstrip.v1\","]
#[doc = "    \"media.audio_peaks.v1\","]
#[doc = "    \"media.frames.v1\""]
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
pub enum MediaIngestManifestChildrenItemKind {
    #[serde(rename = "media.proxy.v1")]
    MediaProxyV1,
    #[serde(rename = "media.audio_16k.v1")]
    MediaAudio16kV1,
    #[serde(rename = "media.audio_48k.v1")]
    MediaAudio48kV1,
    #[serde(rename = "media.loudness_envelope.v1")]
    MediaLoudnessEnvelopeV1,
    #[serde(rename = "media.reference_index.v1")]
    MediaReferenceIndexV1,
    #[serde(rename = "media.filmstrip.v1")]
    MediaFilmstripV1,
    #[serde(rename = "media.audio_peaks.v1")]
    MediaAudioPeaksV1,
    #[serde(rename = "media.frames.v1")]
    MediaFramesV1,
}
impl ::std::fmt::Display for MediaIngestManifestChildrenItemKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::MediaProxyV1 => f.write_str("media.proxy.v1"),
            Self::MediaAudio16kV1 => f.write_str("media.audio_16k.v1"),
            Self::MediaAudio48kV1 => f.write_str("media.audio_48k.v1"),
            Self::MediaLoudnessEnvelopeV1 => f.write_str("media.loudness_envelope.v1"),
            Self::MediaReferenceIndexV1 => f.write_str("media.reference_index.v1"),
            Self::MediaFilmstripV1 => f.write_str("media.filmstrip.v1"),
            Self::MediaAudioPeaksV1 => f.write_str("media.audio_peaks.v1"),
            Self::MediaFramesV1 => f.write_str("media.frames.v1"),
        }
    }
}
impl ::std::str::FromStr for MediaIngestManifestChildrenItemKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "media.proxy.v1" => Ok(Self::MediaProxyV1),
            "media.audio_16k.v1" => Ok(Self::MediaAudio16kV1),
            "media.audio_48k.v1" => Ok(Self::MediaAudio48kV1),
            "media.loudness_envelope.v1" => Ok(Self::MediaLoudnessEnvelopeV1),
            "media.reference_index.v1" => Ok(Self::MediaReferenceIndexV1),
            "media.filmstrip.v1" => Ok(Self::MediaFilmstripV1),
            "media.audio_peaks.v1" => Ok(Self::MediaAudioPeaksV1),
            "media.frames.v1" => Ok(Self::MediaFramesV1),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MediaIngestManifestChildrenItemKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MediaIngestManifestChildrenItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MediaIngestManifestChildrenItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
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
    pub struct MediaIngestManifest {
        children: ::std::result::Result<
            ::std::vec::Vec<super::MediaIngestManifestChildrenItem>,
            ::std::string::String,
        >,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for MediaIngestManifest {
        fn default() -> Self {
            Self {
                children: Err("no value supplied for children".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl MediaIngestManifest {
        pub fn children<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MediaIngestManifestChildrenItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.children = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for children: {e}"));
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
    impl ::std::convert::TryFrom<MediaIngestManifest> for super::MediaIngestManifest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MediaIngestManifest,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                children: value.children?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::MediaIngestManifest> for MediaIngestManifest {
        fn from(value: super::MediaIngestManifest) -> Self {
            Self {
                children: Ok(value.children),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MediaIngestManifestChildrenItem {
        artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        kind: ::std::result::Result<
            super::MediaIngestManifestChildrenItemKind,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for MediaIngestManifestChildrenItem {
        fn default() -> Self {
            Self {
                artifact_id: Err("no value supplied for artifact_id".to_string()),
                kind: Err("no value supplied for kind".to_string()),
            }
        }
    }
    impl MediaIngestManifestChildrenItem {
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
            T: ::std::convert::TryInto<super::MediaIngestManifestChildrenItemKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MediaIngestManifestChildrenItem>
        for super::MediaIngestManifestChildrenItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MediaIngestManifestChildrenItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                artifact_id: value.artifact_id?,
                kind: value.kind?,
            })
        }
    }
    impl ::std::convert::From<super::MediaIngestManifestChildrenItem>
        for MediaIngestManifestChildrenItem
    {
        fn from(value: super::MediaIngestManifestChildrenItem) -> Self {
            Self {
                artifact_id: Ok(value.artifact_id),
                kind: Ok(value.kind),
            }
        }
    }
}
