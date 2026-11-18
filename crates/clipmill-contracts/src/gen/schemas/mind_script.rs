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
#[doc = "A recording's annotated script cut into the windows an editorial model reads one at a time (architecture §4.3, tier 1), with the sentences behind it. Written by the daemon from the Timeline Index: each window is the script text of its stretch of the recording and the sentences the model may cite from it, and every sentence belongs to exactly one window. Speakers are the script's letters. The digest is of the whole recording's script at this level, so what a model read can be traced to exact text."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.mind.script.v1.json\","]
#[doc = "  \"title\": \"MindScript\","]
#[doc = "  \"description\": \"A recording's annotated script cut into the windows an editorial model reads one at a time (architecture §4.3, tier 1), with the sentences behind it. Written by the daemon from the Timeline Index: each window is the script text of its stretch of the recording and the sentences the model may cite from it, and every sentence belongs to exactly one window. Speakers are the script's letters. The digest is of the whole recording's script at this level, so what a model read can be traced to exact text.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"digest\","]
#[doc = "    \"duration_ticks\","]
#[doc = "    \"level\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"units\","]
#[doc = "    \"windows\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"digest\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"duration_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"level\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"full\","]
#[doc = "        \"compact\","]
#[doc = "        \"outline\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.mind.script.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"units\": {"]
#[doc = "      \"description\": \"Every sentence, in recording order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"end_ticks\","]
#[doc = "          \"index\","]
#[doc = "          \"start_ticks\","]
#[doc = "          \"text\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"end_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"index\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"speaker\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"pattern\": \"^[A-Z][0-9]{0,3}$\""]
#[doc = "          },"]
#[doc = "          \"start_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"text\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"windows\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"text\","]
#[doc = "          \"tokens\","]
#[doc = "          \"units\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"text\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          },"]
#[doc = "          \"tokens\": {"]
#[doc = "            \"description\": \"The script module's pessimistic token estimate for the text.\","]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 1.0"]
#[doc = "          },"]
#[doc = "          \"units\": {"]
#[doc = "            \"type\": \"array\","]
#[doc = "            \"items\": {"]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"minimum\": 0.0"]
#[doc = "            },"]
#[doc = "            \"minItems\": 1"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindScript {
    pub digest: Sha256,
    pub duration_ticks: u64,
    pub level: MindScriptLevel,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    #[doc = "Every sentence, in recording order."]
    pub units: ::std::vec::Vec<MindScriptUnitsItem>,
    pub windows: ::std::vec::Vec<MindScriptWindowsItem>,
}
impl MindScript {
    pub fn builder() -> builder::MindScript {
        Default::default()
    }
}
#[doc = "`MindScriptLevel`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"full\","]
#[doc = "    \"compact\","]
#[doc = "    \"outline\""]
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
pub enum MindScriptLevel {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "compact")]
    Compact,
    #[serde(rename = "outline")]
    Outline,
}
impl ::std::fmt::Display for MindScriptLevel {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Full => f.write_str("full"),
            Self::Compact => f.write_str("compact"),
            Self::Outline => f.write_str("outline"),
        }
    }
}
impl ::std::str::FromStr for MindScriptLevel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "full" => Ok(Self::Full),
            "compact" => Ok(Self::Compact),
            "outline" => Ok(Self::Outline),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindScriptLevel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindScriptLevel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindScriptLevel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindScriptUnitsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"index\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"text\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"speaker\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[A-Z][0-9]{0,3}$\""]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindScriptUnitsItem {
    pub end_ticks: u64,
    pub index: u64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub speaker: ::std::option::Option<MindScriptUnitsItemSpeaker>,
    pub start_ticks: u64,
    pub text: ::std::string::String,
}
impl MindScriptUnitsItem {
    pub fn builder() -> builder::MindScriptUnitsItem {
        Default::default()
    }
}
#[doc = "`MindScriptUnitsItemSpeaker`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[A-Z][0-9]{0,3}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindScriptUnitsItemSpeaker(::std::string::String);
impl ::std::ops::Deref for MindScriptUnitsItemSpeaker {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindScriptUnitsItemSpeaker> for ::std::string::String {
    fn from(value: MindScriptUnitsItemSpeaker) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindScriptUnitsItemSpeaker {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[A-Z][0-9]{0,3}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[A-Z][0-9]{0,3}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindScriptUnitsItemSpeaker {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindScriptUnitsItemSpeaker {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindScriptUnitsItemSpeaker {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindScriptUnitsItemSpeaker {
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
#[doc = "`MindScriptWindowsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"text\","]
#[doc = "    \"tokens\","]
#[doc = "    \"units\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"tokens\": {"]
#[doc = "      \"description\": \"The script module's pessimistic token estimate for the text.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"units\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"integer\","]
#[doc = "        \"minimum\": 0.0"]
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
pub struct MindScriptWindowsItem {
    pub text: MindScriptWindowsItemText,
    #[doc = "The script module's pessimistic token estimate for the text."]
    pub tokens: ::std::num::NonZeroU64,
    pub units: ::std::vec::Vec<u64>,
}
impl MindScriptWindowsItem {
    pub fn builder() -> builder::MindScriptWindowsItem {
        Default::default()
    }
}
#[doc = "`MindScriptWindowsItemText`"]
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
pub struct MindScriptWindowsItemText(::std::string::String);
impl ::std::ops::Deref for MindScriptWindowsItemText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindScriptWindowsItemText> for ::std::string::String {
    fn from(value: MindScriptWindowsItemText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindScriptWindowsItemText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindScriptWindowsItemText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindScriptWindowsItemText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindScriptWindowsItemText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindScriptWindowsItemText {
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
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct MindScript {
        digest: ::std::result::Result<super::Sha256, ::std::string::String>,
        duration_ticks: ::std::result::Result<u64, ::std::string::String>,
        level: ::std::result::Result<super::MindScriptLevel, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        units: ::std::result::Result<
            ::std::vec::Vec<super::MindScriptUnitsItem>,
            ::std::string::String,
        >,
        windows: ::std::result::Result<
            ::std::vec::Vec<super::MindScriptWindowsItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for MindScript {
        fn default() -> Self {
            Self {
                digest: Err("no value supplied for digest".to_string()),
                duration_ticks: Err("no value supplied for duration_ticks".to_string()),
                level: Err("no value supplied for level".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                units: Err("no value supplied for units".to_string()),
                windows: Err("no value supplied for windows".to_string()),
            }
        }
    }
    impl MindScript {
        pub fn digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for digest: {e}"));
            self
        }
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
        pub fn level<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindScriptLevel>,
            T::Error: ::std::fmt::Display,
        {
            self.level = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for level: {e}"));
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
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindScriptUnitsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.units = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for units: {e}"));
            self
        }
        pub fn windows<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindScriptWindowsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.windows = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for windows: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindScript> for super::MindScript {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindScript,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                digest: value.digest?,
                duration_ticks: value.duration_ticks?,
                level: value.level?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                units: value.units?,
                windows: value.windows?,
            })
        }
    }
    impl ::std::convert::From<super::MindScript> for MindScript {
        fn from(value: super::MindScript) -> Self {
            Self {
                digest: Ok(value.digest),
                duration_ticks: Ok(value.duration_ticks),
                level: Ok(value.level),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                units: Ok(value.units),
                windows: Ok(value.windows),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindScriptUnitsItem {
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        speaker: ::std::result::Result<
            ::std::option::Option<super::MindScriptUnitsItemSpeaker>,
            ::std::string::String,
        >,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        text: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for MindScriptUnitsItem {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                index: Err("no value supplied for index".to_string()),
                speaker: Ok(Default::default()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                text: Err("no value supplied for text".to_string()),
            }
        }
    }
    impl MindScriptUnitsItem {
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
        pub fn speaker<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::MindScriptUnitsItemSpeaker>>,
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
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindScriptUnitsItem> for super::MindScriptUnitsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindScriptUnitsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                index: value.index?,
                speaker: value.speaker?,
                start_ticks: value.start_ticks?,
                text: value.text?,
            })
        }
    }
    impl ::std::convert::From<super::MindScriptUnitsItem> for MindScriptUnitsItem {
        fn from(value: super::MindScriptUnitsItem) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                index: Ok(value.index),
                speaker: Ok(value.speaker),
                start_ticks: Ok(value.start_ticks),
                text: Ok(value.text),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindScriptWindowsItem {
        text: ::std::result::Result<super::MindScriptWindowsItemText, ::std::string::String>,
        tokens: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        units: ::std::result::Result<::std::vec::Vec<u64>, ::std::string::String>,
    }
    impl ::std::default::Default for MindScriptWindowsItem {
        fn default() -> Self {
            Self {
                text: Err("no value supplied for text".to_string()),
                tokens: Err("no value supplied for tokens".to_string()),
                units: Err("no value supplied for units".to_string()),
            }
        }
    }
    impl MindScriptWindowsItem {
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindScriptWindowsItemText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {e}"));
            self
        }
        pub fn tokens<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.tokens = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tokens: {e}"));
            self
        }
        pub fn units<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.units = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for units: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindScriptWindowsItem> for super::MindScriptWindowsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindScriptWindowsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                text: value.text?,
                tokens: value.tokens?,
                units: value.units?,
            })
        }
    }
    impl ::std::convert::From<super::MindScriptWindowsItem> for MindScriptWindowsItem {
        fn from(value: super::MindScriptWindowsItem) -> Self {
            Self {
                text: Ok(value.text),
                tokens: Ok(value.tokens),
                units: Ok(value.units),
            }
        }
    }
}
