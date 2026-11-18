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
#[doc = "A recording's speech and remembered events as vectors, for search by meaning. Each passage is a run of consecutive sentences long enough to mean something on its own (a single recognised fragment is not), named by the sentences it spans; each event is a Recording Memory event's title. The vectors themselves are in `vectors_file`, little-endian float16, one row of `dims` values per unit in the order listed, each normalized to length one so a dot product is the cosine."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.text.embeddings.v1.json\","]
#[doc = "  \"title\": \"TextEmbeddings\","]
#[doc = "  \"description\": \"A recording's speech and remembered events as vectors, for search by meaning. Each passage is a run of consecutive sentences long enough to mean something on its own (a single recognised fragment is not), named by the sentences it spans; each event is a Recording Memory event's title. The vectors themselves are in `vectors_file`, little-endian float16, one row of `dims` values per unit in the order listed, each normalized to length one so a dot product is the cosine.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"dtype\","]
#[doc = "    \"model\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"units\","]
#[doc = "    \"vectors_file\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"dtype\": {"]
#[doc = "      \"const\": \"float16\""]
#[doc = "    },"]
#[doc = "    \"model\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"dims\","]
#[doc = "        \"name\","]
#[doc = "        \"pooling\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"dims\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 4096.0,"]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"name\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"pooling\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"mean\","]
#[doc = "            \"cls\","]
#[doc = "            \"last\""]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"implementation\","]
#[doc = "        \"model_digest\","]
#[doc = "        \"stage\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"implementation\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"model_digest\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"stage\": {"]
#[doc = "          \"const\": \"text-embed\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.text.embeddings.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"units\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"first_unit\","]
#[doc = "          \"id\","]
#[doc = "          \"kind\","]
#[doc = "          \"last_unit\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"first_unit\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"id\": {"]
#[doc = "            \"description\": \"`s12-s15` for a passage, the event id (`m3`) for an event.\","]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"maxLength\": 32,"]
#[doc = "            \"minLength\": 1"]
#[doc = "          },"]
#[doc = "          \"kind\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"passage\","]
#[doc = "              \"event\""]
#[doc = "            ]"]
#[doc = "          },"]
#[doc = "          \"last_unit\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"vectors_file\": {"]
#[doc = "      \"const\": \"vectors.f16\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextEmbeddings {
    pub dtype: ::serde_json::Value,
    pub model: TextEmbeddingsModel,
    pub producer: TextEmbeddingsProducer,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    pub units: ::std::vec::Vec<TextEmbeddingsUnitsItem>,
    pub vectors_file: ::serde_json::Value,
}
impl TextEmbeddings {
    pub fn builder() -> builder::TextEmbeddings {
        Default::default()
    }
}
#[doc = "`TextEmbeddingsModel`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"dims\","]
#[doc = "    \"name\","]
#[doc = "    \"pooling\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"dims\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 4096.0,"]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"pooling\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"mean\","]
#[doc = "        \"cls\","]
#[doc = "        \"last\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextEmbeddingsModel {
    pub dims: ::std::num::NonZeroU64,
    pub name: TextEmbeddingsModelName,
    pub pooling: TextEmbeddingsModelPooling,
}
impl TextEmbeddingsModel {
    pub fn builder() -> builder::TextEmbeddingsModel {
        Default::default()
    }
}
#[doc = "`TextEmbeddingsModelName`"]
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
pub struct TextEmbeddingsModelName(::std::string::String);
impl ::std::ops::Deref for TextEmbeddingsModelName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TextEmbeddingsModelName> for ::std::string::String {
    fn from(value: TextEmbeddingsModelName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TextEmbeddingsModelName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TextEmbeddingsModelName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TextEmbeddingsModelName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TextEmbeddingsModelName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TextEmbeddingsModelName {
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
#[doc = "`TextEmbeddingsModelPooling`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"mean\","]
#[doc = "    \"cls\","]
#[doc = "    \"last\""]
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
pub enum TextEmbeddingsModelPooling {
    #[serde(rename = "mean")]
    Mean,
    #[serde(rename = "cls")]
    Cls,
    #[serde(rename = "last")]
    Last,
}
impl ::std::fmt::Display for TextEmbeddingsModelPooling {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Mean => f.write_str("mean"),
            Self::Cls => f.write_str("cls"),
            Self::Last => f.write_str("last"),
        }
    }
}
impl ::std::str::FromStr for TextEmbeddingsModelPooling {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "mean" => Ok(Self::Mean),
            "cls" => Ok(Self::Cls),
            "last" => Ok(Self::Last),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TextEmbeddingsModelPooling {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TextEmbeddingsModelPooling {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TextEmbeddingsModelPooling {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`TextEmbeddingsProducer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"implementation\","]
#[doc = "    \"model_digest\","]
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
#[doc = "      \"const\": \"text-embed\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextEmbeddingsProducer {
    pub implementation: TextEmbeddingsProducerImplementation,
    pub model_digest: Sha256,
    pub stage: ::serde_json::Value,
}
impl TextEmbeddingsProducer {
    pub fn builder() -> builder::TextEmbeddingsProducer {
        Default::default()
    }
}
#[doc = "`TextEmbeddingsProducerImplementation`"]
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
pub struct TextEmbeddingsProducerImplementation(::std::string::String);
impl ::std::ops::Deref for TextEmbeddingsProducerImplementation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TextEmbeddingsProducerImplementation> for ::std::string::String {
    fn from(value: TextEmbeddingsProducerImplementation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TextEmbeddingsProducerImplementation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TextEmbeddingsProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TextEmbeddingsProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TextEmbeddingsProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TextEmbeddingsProducerImplementation {
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
#[doc = "`TextEmbeddingsUnitsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"first_unit\","]
#[doc = "    \"id\","]
#[doc = "    \"kind\","]
#[doc = "    \"last_unit\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"first_unit\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"description\": \"`s12-s15` for a passage, the event id (`m3`) for an event.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 32,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"passage\","]
#[doc = "        \"event\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"last_unit\": {"]
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
pub struct TextEmbeddingsUnitsItem {
    pub first_unit: u64,
    #[doc = "`s12-s15` for a passage, the event id (`m3`) for an event."]
    pub id: TextEmbeddingsUnitsItemId,
    pub kind: TextEmbeddingsUnitsItemKind,
    pub last_unit: u64,
}
impl TextEmbeddingsUnitsItem {
    pub fn builder() -> builder::TextEmbeddingsUnitsItem {
        Default::default()
    }
}
#[doc = "`s12-s15` for a passage, the event id (`m3`) for an event."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"`s12-s15` for a passage, the event id (`m3`) for an event.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 32,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TextEmbeddingsUnitsItemId(::std::string::String);
impl ::std::ops::Deref for TextEmbeddingsUnitsItemId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TextEmbeddingsUnitsItemId> for ::std::string::String {
    fn from(value: TextEmbeddingsUnitsItemId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TextEmbeddingsUnitsItemId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 32usize {
            return Err("longer than 32 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TextEmbeddingsUnitsItemId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TextEmbeddingsUnitsItemId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TextEmbeddingsUnitsItemId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TextEmbeddingsUnitsItemId {
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
#[doc = "`TextEmbeddingsUnitsItemKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"passage\","]
#[doc = "    \"event\""]
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
pub enum TextEmbeddingsUnitsItemKind {
    #[serde(rename = "passage")]
    Passage,
    #[serde(rename = "event")]
    Event,
}
impl ::std::fmt::Display for TextEmbeddingsUnitsItemKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Passage => f.write_str("passage"),
            Self::Event => f.write_str("event"),
        }
    }
}
impl ::std::str::FromStr for TextEmbeddingsUnitsItemKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "passage" => Ok(Self::Passage),
            "event" => Ok(Self::Event),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TextEmbeddingsUnitsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TextEmbeddingsUnitsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TextEmbeddingsUnitsItemKind {
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
    pub struct TextEmbeddings {
        dtype: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        model: ::std::result::Result<super::TextEmbeddingsModel, ::std::string::String>,
        producer: ::std::result::Result<super::TextEmbeddingsProducer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        units: ::std::result::Result<
            ::std::vec::Vec<super::TextEmbeddingsUnitsItem>,
            ::std::string::String,
        >,
        vectors_file: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for TextEmbeddings {
        fn default() -> Self {
            Self {
                dtype: Err("no value supplied for dtype".to_string()),
                model: Err("no value supplied for model".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                units: Err("no value supplied for units".to_string()),
                vectors_file: Err("no value supplied for vectors_file".to_string()),
            }
        }
    }
    impl TextEmbeddings {
        pub fn dtype<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.dtype = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for dtype: {e}"));
            self
        }
        pub fn model<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextEmbeddingsModel>,
            T::Error: ::std::fmt::Display,
        {
            self.model = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model: {e}"));
            self
        }
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextEmbeddingsProducer>,
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
        pub fn units<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::TextEmbeddingsUnitsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.units = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for units: {e}"));
            self
        }
        pub fn vectors_file<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.vectors_file = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for vectors_file: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TextEmbeddings> for super::TextEmbeddings {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TextEmbeddings,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                dtype: value.dtype?,
                model: value.model?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                units: value.units?,
                vectors_file: value.vectors_file?,
            })
        }
    }
    impl ::std::convert::From<super::TextEmbeddings> for TextEmbeddings {
        fn from(value: super::TextEmbeddings) -> Self {
            Self {
                dtype: Ok(value.dtype),
                model: Ok(value.model),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                units: Ok(value.units),
                vectors_file: Ok(value.vectors_file),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TextEmbeddingsModel {
        dims: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        name: ::std::result::Result<super::TextEmbeddingsModelName, ::std::string::String>,
        pooling: ::std::result::Result<super::TextEmbeddingsModelPooling, ::std::string::String>,
    }
    impl ::std::default::Default for TextEmbeddingsModel {
        fn default() -> Self {
            Self {
                dims: Err("no value supplied for dims".to_string()),
                name: Err("no value supplied for name".to_string()),
                pooling: Err("no value supplied for pooling".to_string()),
            }
        }
    }
    impl TextEmbeddingsModel {
        pub fn dims<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.dims = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for dims: {e}"));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextEmbeddingsModelName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn pooling<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextEmbeddingsModelPooling>,
            T::Error: ::std::fmt::Display,
        {
            self.pooling = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for pooling: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TextEmbeddingsModel> for super::TextEmbeddingsModel {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TextEmbeddingsModel,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                dims: value.dims?,
                name: value.name?,
                pooling: value.pooling?,
            })
        }
    }
    impl ::std::convert::From<super::TextEmbeddingsModel> for TextEmbeddingsModel {
        fn from(value: super::TextEmbeddingsModel) -> Self {
            Self {
                dims: Ok(value.dims),
                name: Ok(value.name),
                pooling: Ok(value.pooling),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TextEmbeddingsProducer {
        implementation: ::std::result::Result<
            super::TextEmbeddingsProducerImplementation,
            ::std::string::String,
        >,
        model_digest: ::std::result::Result<super::Sha256, ::std::string::String>,
        stage: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for TextEmbeddingsProducer {
        fn default() -> Self {
            Self {
                implementation: Err("no value supplied for implementation".to_string()),
                model_digest: Err("no value supplied for model_digest".to_string()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl TextEmbeddingsProducer {
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextEmbeddingsProducerImplementation>,
            T::Error: ::std::fmt::Display,
        {
            self.implementation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for implementation: {e}"));
            self
        }
        pub fn model_digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.model_digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model_digest: {e}"));
            self
        }
        pub fn stage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.stage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stage: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TextEmbeddingsProducer> for super::TextEmbeddingsProducer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TextEmbeddingsProducer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                implementation: value.implementation?,
                model_digest: value.model_digest?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::TextEmbeddingsProducer> for TextEmbeddingsProducer {
        fn from(value: super::TextEmbeddingsProducer) -> Self {
            Self {
                implementation: Ok(value.implementation),
                model_digest: Ok(value.model_digest),
                stage: Ok(value.stage),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TextEmbeddingsUnitsItem {
        first_unit: ::std::result::Result<u64, ::std::string::String>,
        id: ::std::result::Result<super::TextEmbeddingsUnitsItemId, ::std::string::String>,
        kind: ::std::result::Result<super::TextEmbeddingsUnitsItemKind, ::std::string::String>,
        last_unit: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for TextEmbeddingsUnitsItem {
        fn default() -> Self {
            Self {
                first_unit: Err("no value supplied for first_unit".to_string()),
                id: Err("no value supplied for id".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                last_unit: Err("no value supplied for last_unit".to_string()),
            }
        }
    }
    impl TextEmbeddingsUnitsItem {
        pub fn first_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_unit: {e}"));
            self
        }
        pub fn id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextEmbeddingsUnitsItemId>,
            T::Error: ::std::fmt::Display,
        {
            self.id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for id: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextEmbeddingsUnitsItemKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn last_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.last_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for last_unit: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TextEmbeddingsUnitsItem> for super::TextEmbeddingsUnitsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TextEmbeddingsUnitsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                first_unit: value.first_unit?,
                id: value.id?,
                kind: value.kind?,
                last_unit: value.last_unit?,
            })
        }
    }
    impl ::std::convert::From<super::TextEmbeddingsUnitsItem> for TextEmbeddingsUnitsItem {
        fn from(value: super::TextEmbeddingsUnitsItem) -> Self {
            Self {
                first_unit: Ok(value.first_unit),
                id: Ok(value.id),
                kind: Ok(value.kind),
                last_unit: Ok(value.last_unit),
            }
        }
    }
}
