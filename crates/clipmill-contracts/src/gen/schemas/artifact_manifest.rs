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
#[doc = "The manifest every content-addressed artifact carries. The manifest is the contract; the data files are opaque to everyone but the producing stage's schema. Deliberately excludes wall-clock time: artifact identity is a pure function of stage, inputs, config, model, and version — publish time is project state, recorded by the daemon's database."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.artifact.manifest.v1.json\","]
#[doc = "  \"title\": \"ArtifactManifest\","]
#[doc = "  \"description\": \"The manifest every content-addressed artifact carries. The manifest is the contract; the data files are opaque to everyone but the producing stage's schema. Deliberately excludes wall-clock time: artifact identity is a pure function of stage, inputs, config, model, and version — publish time is project state, recorded by the daemon's database.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"artifact_id\","]
#[doc = "    \"files\","]
#[doc = "    \"inputs\","]
#[doc = "    \"kind\","]
#[doc = "    \"policy\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"timebase\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"artifact_id\": {"]
#[doc = "      \"description\": \"Content address: hash over stage identity, ordered input hashes, canonical config, model manifest digest, and semantic version.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"files\": {"]
#[doc = "      \"description\": \"The artifact's data files, hash-verified on read.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"bytes\","]
#[doc = "          \"path\","]
#[doc = "          \"sha256\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"bytes\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"path\": {"]
#[doc = "            \"description\": \"Relative path within the artifact directory. The store additionally rejects traversal and absolute paths at write time.\","]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"pattern\": \"^[^/].*$\""]
#[doc = "          },"]
#[doc = "          \"sha256\": {"]
#[doc = "            \"$ref\": \"#/$defs/sha256\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"description\": \"Ordered artifact ids this artifact was derived from.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/sha256\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"description\": \"Versioned artifact kind, e.g. 'evidence.active_speaker.v2'.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[a-z][a-z0-9_.]*\\\\.v[0-9]+$\""]
#[doc = "    },"]
#[doc = "    \"policy\": {"]
#[doc = "      \"description\": \"Network policy under which this artifact was produced.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"local-lock\","]
#[doc = "        \"network-allowed\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"implementation\","]
#[doc = "        \"stage\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"implementation\": {"]
#[doc = "          \"description\": \"Implementation and version, e.g. 'loconet-adapter@1.4.0'.\","]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        \"model_digest\": {"]
#[doc = "          \"description\": \"Digest of the model manifest, when a model produced this artifact.\","]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"stage\": {"]
#[doc = "          \"description\": \"Stage identity, e.g. 'active-speaker'.\","]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"pattern\": \"^[a-z][a-z0-9-]*$\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"quality\": {"]
#[doc = "      \"description\": \"Producer-reported quality signals, e.g. {\\\"coverage\\\": 0.97}.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.artifact.manifest.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"description\": \"Fingerprint of the source media this artifact derives from.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"timebase\": {"]
#[doc = "      \"description\": \"The rational timebase all times in the artifact's data files are expressed against.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"den\","]
#[doc = "        \"num\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"den\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"num\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
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
pub struct ArtifactManifest {
    #[doc = "Content address: hash over stage identity, ordered input hashes, canonical config, model manifest digest, and semantic version."]
    pub artifact_id: Sha256,
    #[doc = "The artifact's data files, hash-verified on read."]
    pub files: ::std::vec::Vec<ArtifactManifestFilesItem>,
    #[doc = "Ordered artifact ids this artifact was derived from."]
    pub inputs: ::std::vec::Vec<Sha256>,
    #[doc = "Versioned artifact kind, e.g. 'evidence.active_speaker.v2'."]
    pub kind: ArtifactManifestKind,
    #[doc = "Network policy under which this artifact was produced."]
    pub policy: ArtifactManifestPolicy,
    pub producer: ArtifactManifestProducer,
    #[doc = "Producer-reported quality signals, e.g. {\"coverage\": 0.97}."]
    #[serde(
        default,
        skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
    )]
    pub quality: ::std::collections::HashMap<::std::string::String, f64>,
    pub schema_version: ::serde_json::Value,
    #[doc = "Fingerprint of the source media this artifact derives from."]
    pub source_fingerprint: Sha256,
    pub timebase: ArtifactManifestTimebase,
}
impl ArtifactManifest {
    pub fn builder() -> builder::ArtifactManifest {
        Default::default()
    }
}
#[doc = "`ArtifactManifestFilesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"bytes\","]
#[doc = "    \"path\","]
#[doc = "    \"sha256\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bytes\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"path\": {"]
#[doc = "      \"description\": \"Relative path within the artifact directory. The store additionally rejects traversal and absolute paths at write time.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[^/].*$\""]
#[doc = "    },"]
#[doc = "    \"sha256\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ArtifactManifestFilesItem {
    pub bytes: u64,
    #[doc = "Relative path within the artifact directory. The store additionally rejects traversal and absolute paths at write time."]
    pub path: ArtifactManifestFilesItemPath,
    pub sha256: Sha256,
}
impl ArtifactManifestFilesItem {
    pub fn builder() -> builder::ArtifactManifestFilesItem {
        Default::default()
    }
}
#[doc = "Relative path within the artifact directory. The store additionally rejects traversal and absolute paths at write time."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Relative path within the artifact directory. The store additionally rejects traversal and absolute paths at write time.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[^/].*$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ArtifactManifestFilesItemPath(::std::string::String);
impl ::std::ops::Deref for ArtifactManifestFilesItemPath {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ArtifactManifestFilesItemPath> for ::std::string::String {
    fn from(value: ArtifactManifestFilesItemPath) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ArtifactManifestFilesItemPath {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[^/].*$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[^/].*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ArtifactManifestFilesItemPath {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArtifactManifestFilesItemPath {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArtifactManifestFilesItemPath {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ArtifactManifestFilesItemPath {
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
#[doc = "Versioned artifact kind, e.g. 'evidence.active_speaker.v2'."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Versioned artifact kind, e.g. 'evidence.active_speaker.v2'.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[a-z][a-z0-9_.]*\\\\.v[0-9]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ArtifactManifestKind(::std::string::String);
impl ::std::ops::Deref for ArtifactManifestKind {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ArtifactManifestKind> for ::std::string::String {
    fn from(value: ArtifactManifestKind) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ArtifactManifestKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| {
                ::regress::Regex::new("^[a-z][a-z0-9_.]*\\.v[0-9]+$").unwrap()
            });
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[a-z][a-z0-9_.]*\\.v[0-9]+$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ArtifactManifestKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArtifactManifestKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArtifactManifestKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ArtifactManifestKind {
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
#[doc = "Network policy under which this artifact was produced."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Network policy under which this artifact was produced.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"local-lock\","]
#[doc = "    \"network-allowed\""]
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
pub enum ArtifactManifestPolicy {
    #[serde(rename = "local-lock")]
    LocalLock,
    #[serde(rename = "network-allowed")]
    NetworkAllowed,
}
impl ::std::fmt::Display for ArtifactManifestPolicy {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::LocalLock => f.write_str("local-lock"),
            Self::NetworkAllowed => f.write_str("network-allowed"),
        }
    }
}
impl ::std::str::FromStr for ArtifactManifestPolicy {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "local-lock" => Ok(Self::LocalLock),
            "network-allowed" => Ok(Self::NetworkAllowed),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ArtifactManifestPolicy {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArtifactManifestPolicy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArtifactManifestPolicy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`ArtifactManifestProducer`"]
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
#[doc = "      \"description\": \"Implementation and version, e.g. 'loconet-adapter@1.4.0'.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"model_digest\": {"]
#[doc = "      \"description\": \"Digest of the model manifest, when a model produced this artifact.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"stage\": {"]
#[doc = "      \"description\": \"Stage identity, e.g. 'active-speaker'.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[a-z][a-z0-9-]*$\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ArtifactManifestProducer {
    #[doc = "Implementation and version, e.g. 'loconet-adapter@1.4.0'."]
    pub implementation: ::std::string::String,
    #[doc = "Digest of the model manifest, when a model produced this artifact."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub model_digest: ::std::option::Option<Sha256>,
    #[doc = "Stage identity, e.g. 'active-speaker'."]
    pub stage: ArtifactManifestProducerStage,
}
impl ArtifactManifestProducer {
    pub fn builder() -> builder::ArtifactManifestProducer {
        Default::default()
    }
}
#[doc = "Stage identity, e.g. 'active-speaker'."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Stage identity, e.g. 'active-speaker'.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[a-z][a-z0-9-]*$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ArtifactManifestProducerStage(::std::string::String);
impl ::std::ops::Deref for ArtifactManifestProducerStage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ArtifactManifestProducerStage> for ::std::string::String {
    fn from(value: ArtifactManifestProducerStage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ArtifactManifestProducerStage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[a-z][a-z0-9-]*$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[a-z][a-z0-9-]*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ArtifactManifestProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArtifactManifestProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArtifactManifestProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ArtifactManifestProducerStage {
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
#[doc = "The rational timebase all times in the artifact's data files are expressed against."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The rational timebase all times in the artifact's data files are expressed against.\","]
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
pub struct ArtifactManifestTimebase {
    pub den: ::std::num::NonZeroU64,
    pub num: ::std::num::NonZeroU64,
}
impl ArtifactManifestTimebase {
    pub fn builder() -> builder::ArtifactManifestTimebase {
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
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct ArtifactManifest {
        artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        files: ::std::result::Result<
            ::std::vec::Vec<super::ArtifactManifestFilesItem>,
            ::std::string::String,
        >,
        inputs: ::std::result::Result<::std::vec::Vec<super::Sha256>, ::std::string::String>,
        kind: ::std::result::Result<super::ArtifactManifestKind, ::std::string::String>,
        policy: ::std::result::Result<super::ArtifactManifestPolicy, ::std::string::String>,
        producer: ::std::result::Result<super::ArtifactManifestProducer, ::std::string::String>,
        quality: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, f64>,
            ::std::string::String,
        >,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        timebase: ::std::result::Result<super::ArtifactManifestTimebase, ::std::string::String>,
    }
    impl ::std::default::Default for ArtifactManifest {
        fn default() -> Self {
            Self {
                artifact_id: Err("no value supplied for artifact_id".to_string()),
                files: Err("no value supplied for files".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                policy: Err("no value supplied for policy".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                quality: Ok(Default::default()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                timebase: Err("no value supplied for timebase".to_string()),
            }
        }
    }
    impl ArtifactManifest {
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
        pub fn files<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ArtifactManifestFilesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.files = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for files: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArtifactManifestKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn policy<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArtifactManifestPolicy>,
            T::Error: ::std::fmt::Display,
        {
            self.policy = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for policy: {e}"));
            self
        }
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArtifactManifestProducer>,
            T::Error: ::std::fmt::Display,
        {
            self.producer = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for producer: {e}"));
            self
        }
        pub fn quality<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::collections::HashMap<::std::string::String, f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.quality = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for quality: {e}"));
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
            T: ::std::convert::TryInto<super::ArtifactManifestTimebase>,
            T::Error: ::std::fmt::Display,
        {
            self.timebase = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timebase: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ArtifactManifest> for super::ArtifactManifest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ArtifactManifest,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                artifact_id: value.artifact_id?,
                files: value.files?,
                inputs: value.inputs?,
                kind: value.kind?,
                policy: value.policy?,
                producer: value.producer?,
                quality: value.quality?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                timebase: value.timebase?,
            })
        }
    }
    impl ::std::convert::From<super::ArtifactManifest> for ArtifactManifest {
        fn from(value: super::ArtifactManifest) -> Self {
            Self {
                artifact_id: Ok(value.artifact_id),
                files: Ok(value.files),
                inputs: Ok(value.inputs),
                kind: Ok(value.kind),
                policy: Ok(value.policy),
                producer: Ok(value.producer),
                quality: Ok(value.quality),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                timebase: Ok(value.timebase),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ArtifactManifestFilesItem {
        bytes: ::std::result::Result<u64, ::std::string::String>,
        path: ::std::result::Result<super::ArtifactManifestFilesItemPath, ::std::string::String>,
        sha256: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for ArtifactManifestFilesItem {
        fn default() -> Self {
            Self {
                bytes: Err("no value supplied for bytes".to_string()),
                path: Err("no value supplied for path".to_string()),
                sha256: Err("no value supplied for sha256".to_string()),
            }
        }
    }
    impl ArtifactManifestFilesItem {
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
        pub fn path<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArtifactManifestFilesItemPath>,
            T::Error: ::std::fmt::Display,
        {
            self.path = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for path: {e}"));
            self
        }
        pub fn sha256<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.sha256 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sha256: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ArtifactManifestFilesItem> for super::ArtifactManifestFilesItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ArtifactManifestFilesItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                bytes: value.bytes?,
                path: value.path?,
                sha256: value.sha256?,
            })
        }
    }
    impl ::std::convert::From<super::ArtifactManifestFilesItem> for ArtifactManifestFilesItem {
        fn from(value: super::ArtifactManifestFilesItem) -> Self {
            Self {
                bytes: Ok(value.bytes),
                path: Ok(value.path),
                sha256: Ok(value.sha256),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ArtifactManifestProducer {
        implementation: ::std::result::Result<::std::string::String, ::std::string::String>,
        model_digest:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        stage: ::std::result::Result<super::ArtifactManifestProducerStage, ::std::string::String>,
    }
    impl ::std::default::Default for ArtifactManifestProducer {
        fn default() -> Self {
            Self {
                implementation: Err("no value supplied for implementation".to_string()),
                model_digest: Ok(Default::default()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl ArtifactManifestProducer {
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
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
            T: ::std::convert::TryInto<super::ArtifactManifestProducerStage>,
            T::Error: ::std::fmt::Display,
        {
            self.stage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stage: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ArtifactManifestProducer> for super::ArtifactManifestProducer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ArtifactManifestProducer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                implementation: value.implementation?,
                model_digest: value.model_digest?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::ArtifactManifestProducer> for ArtifactManifestProducer {
        fn from(value: super::ArtifactManifestProducer) -> Self {
            Self {
                implementation: Ok(value.implementation),
                model_digest: Ok(value.model_digest),
                stage: Ok(value.stage),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ArtifactManifestTimebase {
        den: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        num: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for ArtifactManifestTimebase {
        fn default() -> Self {
            Self {
                den: Err("no value supplied for den".to_string()),
                num: Err("no value supplied for num".to_string()),
            }
        }
    }
    impl ArtifactManifestTimebase {
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
    impl ::std::convert::TryFrom<ArtifactManifestTimebase> for super::ArtifactManifestTimebase {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ArtifactManifestTimebase,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                den: value.den?,
                num: value.num?,
            })
        }
    }
    impl ::std::convert::From<super::ArtifactManifestTimebase> for ArtifactManifestTimebase {
        fn from(value: super::ArtifactManifestTimebase) -> Self {
            Self {
                den: Ok(value.den),
                num: Ok(value.num),
            }
        }
    }
}
