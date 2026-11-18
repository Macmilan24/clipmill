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
#[doc = "A search query as a vector from the same model as a recording's embeddings, normalized to length one, so searching is a dot product against them."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.text.query_vector.v1.json\","]
#[doc = "  \"title\": \"TextQueryVector\","]
#[doc = "  \"description\": \"A search query as a vector from the same model as a recording's embeddings, normalized to length one, so searching is a dot product against them.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"model_digest\","]
#[doc = "    \"query\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"vector\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"model_digest\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^sha256:[0-9a-f]{64}$\""]
#[doc = "    },"]
#[doc = "    \"query\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 500,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.text.query_vector.v1\""]
#[doc = "    },"]
#[doc = "    \"vector\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 4096,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextQueryVector {
    pub model_digest: TextQueryVectorModelDigest,
    pub query: TextQueryVectorQuery,
    pub schema_version: ::serde_json::Value,
    pub vector: ::std::vec::Vec<f64>,
}
impl TextQueryVector {
    pub fn builder() -> builder::TextQueryVector {
        Default::default()
    }
}
#[doc = "`TextQueryVectorModelDigest`"]
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
pub struct TextQueryVectorModelDigest(::std::string::String);
impl ::std::ops::Deref for TextQueryVectorModelDigest {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TextQueryVectorModelDigest> for ::std::string::String {
    fn from(value: TextQueryVectorModelDigest) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TextQueryVectorModelDigest {
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
impl ::std::convert::TryFrom<&str> for TextQueryVectorModelDigest {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TextQueryVectorModelDigest {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TextQueryVectorModelDigest {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TextQueryVectorModelDigest {
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
#[doc = "`TextQueryVectorQuery`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 500,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TextQueryVectorQuery(::std::string::String);
impl ::std::ops::Deref for TextQueryVectorQuery {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TextQueryVectorQuery> for ::std::string::String {
    fn from(value: TextQueryVectorQuery) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TextQueryVectorQuery {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 500usize {
            return Err("longer than 500 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TextQueryVectorQuery {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TextQueryVectorQuery {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TextQueryVectorQuery {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TextQueryVectorQuery {
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
    pub struct TextQueryVector {
        model_digest:
            ::std::result::Result<super::TextQueryVectorModelDigest, ::std::string::String>,
        query: ::std::result::Result<super::TextQueryVectorQuery, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        vector: ::std::result::Result<::std::vec::Vec<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for TextQueryVector {
        fn default() -> Self {
            Self {
                model_digest: Err("no value supplied for model_digest".to_string()),
                query: Err("no value supplied for query".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                vector: Err("no value supplied for vector".to_string()),
            }
        }
    }
    impl TextQueryVector {
        pub fn model_digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextQueryVectorModelDigest>,
            T::Error: ::std::fmt::Display,
        {
            self.model_digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model_digest: {e}"));
            self
        }
        pub fn query<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TextQueryVectorQuery>,
            T::Error: ::std::fmt::Display,
        {
            self.query = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for query: {e}"));
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
        pub fn vector<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.vector = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for vector: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TextQueryVector> for super::TextQueryVector {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TextQueryVector,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                model_digest: value.model_digest?,
                query: value.query?,
                schema_version: value.schema_version?,
                vector: value.vector?,
            })
        }
    }
    impl ::std::convert::From<super::TextQueryVector> for TextQueryVector {
        fn from(value: super::TextQueryVector) -> Self {
            Self {
                model_digest: Ok(value.model_digest),
                query: Ok(value.query),
                schema_version: Ok(value.schema_version),
                vector: Ok(value.vector),
            }
        }
    }
}
