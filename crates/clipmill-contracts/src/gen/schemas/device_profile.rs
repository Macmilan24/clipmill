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
#[doc = "`CapabilityResult`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"available\","]
#[doc = "    \"backend\","]
#[doc = "    \"capability\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"available\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"backend\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 64,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"capability\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 64,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 512"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CapabilityResult {
    pub available: bool,
    pub backend: CapabilityResultBackend,
    pub capability: CapabilityResultCapability,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub detail: ::std::option::Option<CapabilityResultDetail>,
}
impl CapabilityResult {
    pub fn builder() -> builder::CapabilityResult {
        Default::default()
    }
}
#[doc = "`CapabilityResultBackend`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 64,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CapabilityResultBackend(::std::string::String);
impl ::std::ops::Deref for CapabilityResultBackend {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CapabilityResultBackend> for ::std::string::String {
    fn from(value: CapabilityResultBackend) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CapabilityResultBackend {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 64usize {
            return Err("longer than 64 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CapabilityResultBackend {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CapabilityResultBackend {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CapabilityResultBackend {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CapabilityResultBackend {
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
#[doc = "`CapabilityResultCapability`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 64,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CapabilityResultCapability(::std::string::String);
impl ::std::ops::Deref for CapabilityResultCapability {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CapabilityResultCapability> for ::std::string::String {
    fn from(value: CapabilityResultCapability) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CapabilityResultCapability {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 64usize {
            return Err("longer than 64 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CapabilityResultCapability {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CapabilityResultCapability {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CapabilityResultCapability {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CapabilityResultCapability {
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
#[doc = "`CapabilityResultDetail`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 512"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CapabilityResultDetail(::std::string::String);
impl ::std::ops::Deref for CapabilityResultDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CapabilityResultDetail> for ::std::string::String {
    fn from(value: CapabilityResultDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CapabilityResultDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 512usize {
            return Err("longer than 512 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CapabilityResultDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CapabilityResultDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CapabilityResultDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CapabilityResultDetail {
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
#[doc = "`CodecBenchList`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"type\": \"object\","]
#[doc = "    \"required\": ["]
#[doc = "      \"codec\","]
#[doc = "      \"fps_measured\","]
#[doc = "      \"height\""]
#[doc = "    ],"]
#[doc = "    \"properties\": {"]
#[doc = "      \"codec\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      },"]
#[doc = "      \"fps_measured\": {"]
#[doc = "        \"type\": \"number\","]
#[doc = "        \"exclusiveMinimum\": 0.0"]
#[doc = "      },"]
#[doc = "      \"hardware\": {"]
#[doc = "        \"description\": \"True when a hardware path (VideoToolbox, NVDEC, …) was used.\","]
#[doc = "        \"type\": \"boolean\""]
#[doc = "      },"]
#[doc = "      \"height\": {"]
#[doc = "        \"type\": \"integer\","]
#[doc = "        \"minimum\": 1.0"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"additionalProperties\": false"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct CodecBenchList(pub ::std::vec::Vec<CodecBenchListItem>);
impl ::std::ops::Deref for CodecBenchList {
    type Target = ::std::vec::Vec<CodecBenchListItem>;
    fn deref(&self) -> &::std::vec::Vec<CodecBenchListItem> {
        &self.0
    }
}
impl ::std::convert::From<CodecBenchList> for ::std::vec::Vec<CodecBenchListItem> {
    fn from(value: CodecBenchList) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::vec::Vec<CodecBenchListItem>> for CodecBenchList {
    fn from(value: ::std::vec::Vec<CodecBenchListItem>) -> Self {
        Self(value)
    }
}
#[doc = "`CodecBenchListItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"codec\","]
#[doc = "    \"fps_measured\","]
#[doc = "    \"height\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"codec\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"fps_measured\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"hardware\": {"]
#[doc = "      \"description\": \"True when a hardware path (VideoToolbox, NVDEC, …) was used.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"height\": {"]
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
pub struct CodecBenchListItem {
    pub codec: ::std::string::String,
    pub fps_measured: f64,
    #[doc = "True when a hardware path (VideoToolbox, NVDEC, …) was used."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub hardware: ::std::option::Option<bool>,
    pub height: ::std::num::NonZeroU64,
}
impl CodecBenchListItem {
    pub fn builder() -> builder::CodecBenchListItem {
        Default::default()
    }
}
#[doc = "Signature over RFC 8785 canonical profile JSON with this attestation object omitted."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Signature over RFC 8785 canonical profile JSON with this attestation object omitted.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"algorithm\","]
#[doc = "    \"public_key\","]
#[doc = "    \"signature\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"algorithm\": {"]
#[doc = "      \"const\": \"ed25519\""]
#[doc = "    },"]
#[doc = "    \"public_key\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[0-9a-f]{64}$\""]
#[doc = "    },"]
#[doc = "    \"signature\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[0-9a-f]{128}$\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DeviceAttestation {
    pub algorithm: ::serde_json::Value,
    pub public_key: DeviceAttestationPublicKey,
    pub signature: DeviceAttestationSignature,
}
impl DeviceAttestation {
    pub fn builder() -> builder::DeviceAttestation {
        Default::default()
    }
}
#[doc = "`DeviceAttestationPublicKey`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[0-9a-f]{64}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct DeviceAttestationPublicKey(::std::string::String);
impl ::std::ops::Deref for DeviceAttestationPublicKey {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<DeviceAttestationPublicKey> for ::std::string::String {
    fn from(value: DeviceAttestationPublicKey) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for DeviceAttestationPublicKey {
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
impl ::std::convert::TryFrom<&str> for DeviceAttestationPublicKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DeviceAttestationPublicKey {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DeviceAttestationPublicKey {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for DeviceAttestationPublicKey {
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
#[doc = "`DeviceAttestationSignature`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[0-9a-f]{128}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct DeviceAttestationSignature(::std::string::String);
impl ::std::ops::Deref for DeviceAttestationSignature {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<DeviceAttestationSignature> for ::std::string::String {
    fn from(value: DeviceAttestationSignature) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for DeviceAttestationSignature {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[0-9a-f]{128}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[0-9a-f]{128}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for DeviceAttestationSignature {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DeviceAttestationSignature {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DeviceAttestationSignature {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for DeviceAttestationSignature {
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
#[doc = "The measured device profile (book ch. 11): backend selection is driven by measurement, not static per-platform defaults (D19). Cached as an artifact; re-measured when hardware or runtimes change. Rates (fps, bytes/s) are measurements, not timeline positions, so they may be numbers."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.device_profile.v1.json\","]
#[doc = "  \"title\": \"DeviceProfile\","]
#[doc = "  \"description\": \"The measured device profile (book ch. 11): backend selection is driven by measurement, not static per-platform defaults (D19). Cached as an artifact; re-measured when hardware or runtimes change. Rates (fps, bytes/s) are measurements, not timeline positions, so they may be numbers.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"accelerators\","]
#[doc = "    \"cpu\","]
#[doc = "    \"measured\","]
#[doc = "    \"memory\","]
#[doc = "    \"platform\","]
#[doc = "    \"schema_version\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"accelerators\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"kind\","]
#[doc = "          \"name\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"kind\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"metal\","]
#[doc = "              \"cuda\","]
#[doc = "              \"vulkan\","]
#[doc = "              \"videotoolbox\","]
#[doc = "              \"vaapi\","]
#[doc = "              \"cpu\""]
#[doc = "            ]"]
#[doc = "          },"]
#[doc = "          \"name\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          },"]
#[doc = "          \"vram_bytes\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"cpu\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"logical_cores\","]
#[doc = "        \"model\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"efficiency_cores\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"logical_cores\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"model\": {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        \"performance_cores\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"physical_cores\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"measured\": {"]
#[doc = "      \"description\": \"Micro-benchmark results from the pinned FFmpeg build.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"ffmpeg_build\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"decode\": {"]
#[doc = "          \"$ref\": \"#/$defs/codec_bench_list\""]
#[doc = "        },"]
#[doc = "        \"encode\": {"]
#[doc = "          \"$ref\": \"#/$defs/codec_bench_list\""]
#[doc = "        },"]
#[doc = "        \"ffmpeg_build\": {"]
#[doc = "          \"description\": \"The bom.toml build identifier the measurements were taken with.\","]
#[doc = "          \"type\": \"string\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"memory\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"total_bytes\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"total_bytes\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"unified\": {"]
#[doc = "          \"description\": \"True on unified-memory architectures (Apple Silicon).\","]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"phase0\": {"]
#[doc = "      \"description\": \"Measured Phase 0 scheduler and attestation extension. Legacy v1 profiles omit it; every new Phase 0 profile includes it.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"attestation\","]
#[doc = "        \"available_memory_bytes\","]
#[doc = "        \"capability_results\","]
#[doc = "        \"hardware_fingerprint\","]
#[doc = "        \"hardware_roundtrip\","]
#[doc = "        \"measurement_generation\","]
#[doc = "        \"runtime_identities\","]
#[doc = "        \"shared_memory\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"attestation\": {"]
#[doc = "          \"$ref\": \"#/$defs/device_attestation\""]
#[doc = "        },"]
#[doc = "        \"available_memory_bytes\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"capability_results\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/capability_result\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"hardware_fingerprint\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"pattern\": \"^sha256:[0-9a-f]{64}$\""]
#[doc = "        },"]
#[doc = "        \"hardware_roundtrip\": {"]
#[doc = "          \"$ref\": \"#/$defs/hardware_roundtrip\""]
#[doc = "        },"]
#[doc = "        \"measurement_generation\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"runtime_identities\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/runtime_identity\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"shared_memory\": {"]
#[doc = "          \"$ref\": \"#/$defs/shared_memory_benchmark\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"platform\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"arch\","]
#[doc = "        \"os\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"arch\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"arm64\","]
#[doc = "            \"x86_64\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"os\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"macos\","]
#[doc = "            \"linux\","]
#[doc = "            \"windows\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"os_version\": {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.device_profile.v1\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DeviceProfile {
    pub accelerators: ::std::vec::Vec<DeviceProfileAcceleratorsItem>,
    pub cpu: DeviceProfileCpu,
    pub measured: DeviceProfileMeasured,
    pub memory: DeviceProfileMemory,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub phase0: ::std::option::Option<DeviceProfilePhase0>,
    pub platform: DeviceProfilePlatform,
    pub schema_version: ::serde_json::Value,
}
impl DeviceProfile {
    pub fn builder() -> builder::DeviceProfile {
        Default::default()
    }
}
#[doc = "`DeviceProfileAcceleratorsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"kind\","]
#[doc = "    \"name\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"kind\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"metal\","]
#[doc = "        \"cuda\","]
#[doc = "        \"vulkan\","]
#[doc = "        \"videotoolbox\","]
#[doc = "        \"vaapi\","]
#[doc = "        \"cpu\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"vram_bytes\": {"]
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
pub struct DeviceProfileAcceleratorsItem {
    pub kind: DeviceProfileAcceleratorsItemKind,
    pub name: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub vram_bytes: ::std::option::Option<u64>,
}
impl DeviceProfileAcceleratorsItem {
    pub fn builder() -> builder::DeviceProfileAcceleratorsItem {
        Default::default()
    }
}
#[doc = "`DeviceProfileAcceleratorsItemKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"metal\","]
#[doc = "    \"cuda\","]
#[doc = "    \"vulkan\","]
#[doc = "    \"videotoolbox\","]
#[doc = "    \"vaapi\","]
#[doc = "    \"cpu\""]
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
pub enum DeviceProfileAcceleratorsItemKind {
    #[serde(rename = "metal")]
    Metal,
    #[serde(rename = "cuda")]
    Cuda,
    #[serde(rename = "vulkan")]
    Vulkan,
    #[serde(rename = "videotoolbox")]
    Videotoolbox,
    #[serde(rename = "vaapi")]
    Vaapi,
    #[serde(rename = "cpu")]
    Cpu,
}
impl ::std::fmt::Display for DeviceProfileAcceleratorsItemKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Metal => f.write_str("metal"),
            Self::Cuda => f.write_str("cuda"),
            Self::Vulkan => f.write_str("vulkan"),
            Self::Videotoolbox => f.write_str("videotoolbox"),
            Self::Vaapi => f.write_str("vaapi"),
            Self::Cpu => f.write_str("cpu"),
        }
    }
}
impl ::std::str::FromStr for DeviceProfileAcceleratorsItemKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "metal" => Ok(Self::Metal),
            "cuda" => Ok(Self::Cuda),
            "vulkan" => Ok(Self::Vulkan),
            "videotoolbox" => Ok(Self::Videotoolbox),
            "vaapi" => Ok(Self::Vaapi),
            "cpu" => Ok(Self::Cpu),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for DeviceProfileAcceleratorsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DeviceProfileAcceleratorsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DeviceProfileAcceleratorsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`DeviceProfileCpu`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"logical_cores\","]
#[doc = "    \"model\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"efficiency_cores\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"logical_cores\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"model\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"performance_cores\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"physical_cores\": {"]
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
pub struct DeviceProfileCpu {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub efficiency_cores: ::std::option::Option<u64>,
    pub logical_cores: ::std::num::NonZeroU64,
    pub model: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub performance_cores: ::std::option::Option<u64>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub physical_cores: ::std::option::Option<::std::num::NonZeroU64>,
}
impl DeviceProfileCpu {
    pub fn builder() -> builder::DeviceProfileCpu {
        Default::default()
    }
}
#[doc = "Micro-benchmark results from the pinned FFmpeg build."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Micro-benchmark results from the pinned FFmpeg build.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"ffmpeg_build\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"decode\": {"]
#[doc = "      \"$ref\": \"#/$defs/codec_bench_list\""]
#[doc = "    },"]
#[doc = "    \"encode\": {"]
#[doc = "      \"$ref\": \"#/$defs/codec_bench_list\""]
#[doc = "    },"]
#[doc = "    \"ffmpeg_build\": {"]
#[doc = "      \"description\": \"The bom.toml build identifier the measurements were taken with.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DeviceProfileMeasured {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub decode: ::std::option::Option<CodecBenchList>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub encode: ::std::option::Option<CodecBenchList>,
    #[doc = "The bom.toml build identifier the measurements were taken with."]
    pub ffmpeg_build: ::std::string::String,
}
impl DeviceProfileMeasured {
    pub fn builder() -> builder::DeviceProfileMeasured {
        Default::default()
    }
}
#[doc = "`DeviceProfileMemory`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"total_bytes\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"total_bytes\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"unified\": {"]
#[doc = "      \"description\": \"True on unified-memory architectures (Apple Silicon).\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DeviceProfileMemory {
    pub total_bytes: u64,
    #[doc = "True on unified-memory architectures (Apple Silicon)."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub unified: ::std::option::Option<bool>,
}
impl DeviceProfileMemory {
    pub fn builder() -> builder::DeviceProfileMemory {
        Default::default()
    }
}
#[doc = "Measured Phase 0 scheduler and attestation extension. Legacy v1 profiles omit it; every new Phase 0 profile includes it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Measured Phase 0 scheduler and attestation extension. Legacy v1 profiles omit it; every new Phase 0 profile includes it.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"attestation\","]
#[doc = "    \"available_memory_bytes\","]
#[doc = "    \"capability_results\","]
#[doc = "    \"hardware_fingerprint\","]
#[doc = "    \"hardware_roundtrip\","]
#[doc = "    \"measurement_generation\","]
#[doc = "    \"runtime_identities\","]
#[doc = "    \"shared_memory\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"attestation\": {"]
#[doc = "      \"$ref\": \"#/$defs/device_attestation\""]
#[doc = "    },"]
#[doc = "    \"available_memory_bytes\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"capability_results\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/capability_result\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"hardware_fingerprint\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^sha256:[0-9a-f]{64}$\""]
#[doc = "    },"]
#[doc = "    \"hardware_roundtrip\": {"]
#[doc = "      \"$ref\": \"#/$defs/hardware_roundtrip\""]
#[doc = "    },"]
#[doc = "    \"measurement_generation\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"runtime_identities\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/runtime_identity\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"shared_memory\": {"]
#[doc = "      \"$ref\": \"#/$defs/shared_memory_benchmark\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DeviceProfilePhase0 {
    pub attestation: DeviceAttestation,
    pub available_memory_bytes: u64,
    pub capability_results: ::std::vec::Vec<CapabilityResult>,
    pub hardware_fingerprint: DeviceProfilePhase0HardwareFingerprint,
    pub hardware_roundtrip: HardwareRoundtrip,
    pub measurement_generation: ::std::num::NonZeroU64,
    pub runtime_identities: ::std::vec::Vec<RuntimeIdentity>,
    pub shared_memory: SharedMemoryBenchmark,
}
impl DeviceProfilePhase0 {
    pub fn builder() -> builder::DeviceProfilePhase0 {
        Default::default()
    }
}
#[doc = "`DeviceProfilePhase0HardwareFingerprint`"]
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
pub struct DeviceProfilePhase0HardwareFingerprint(::std::string::String);
impl ::std::ops::Deref for DeviceProfilePhase0HardwareFingerprint {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<DeviceProfilePhase0HardwareFingerprint> for ::std::string::String {
    fn from(value: DeviceProfilePhase0HardwareFingerprint) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for DeviceProfilePhase0HardwareFingerprint {
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
impl ::std::convert::TryFrom<&str> for DeviceProfilePhase0HardwareFingerprint {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DeviceProfilePhase0HardwareFingerprint {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DeviceProfilePhase0HardwareFingerprint {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for DeviceProfilePhase0HardwareFingerprint {
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
#[doc = "`DeviceProfilePlatform`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"arch\","]
#[doc = "    \"os\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"arch\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"arm64\","]
#[doc = "        \"x86_64\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"os\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"macos\","]
#[doc = "        \"linux\","]
#[doc = "        \"windows\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"os_version\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DeviceProfilePlatform {
    pub arch: DeviceProfilePlatformArch,
    pub os: DeviceProfilePlatformOs,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub os_version: ::std::option::Option<::std::string::String>,
}
impl DeviceProfilePlatform {
    pub fn builder() -> builder::DeviceProfilePlatform {
        Default::default()
    }
}
#[doc = "`DeviceProfilePlatformArch`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"arm64\","]
#[doc = "    \"x86_64\""]
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
pub enum DeviceProfilePlatformArch {
    #[serde(rename = "arm64")]
    Arm64,
    #[serde(rename = "x86_64")]
    X8664,
}
impl ::std::fmt::Display for DeviceProfilePlatformArch {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Arm64 => f.write_str("arm64"),
            Self::X8664 => f.write_str("x86_64"),
        }
    }
}
impl ::std::str::FromStr for DeviceProfilePlatformArch {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "arm64" => Ok(Self::Arm64),
            "x86_64" => Ok(Self::X8664),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for DeviceProfilePlatformArch {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DeviceProfilePlatformArch {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DeviceProfilePlatformArch {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`DeviceProfilePlatformOs`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"macos\","]
#[doc = "    \"linux\","]
#[doc = "    \"windows\""]
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
pub enum DeviceProfilePlatformOs {
    #[serde(rename = "macos")]
    Macos,
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "windows")]
    Windows,
}
impl ::std::fmt::Display for DeviceProfilePlatformOs {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Macos => f.write_str("macos"),
            Self::Linux => f.write_str("linux"),
            Self::Windows => f.write_str("windows"),
        }
    }
}
impl ::std::str::FromStr for DeviceProfilePlatformOs {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "macos" => Ok(Self::Macos),
            "linux" => Ok(Self::Linux),
            "windows" => Ok(Self::Windows),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for DeviceProfilePlatformOs {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DeviceProfilePlatformOs {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DeviceProfilePlatformOs {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`HardwareRoundtrip`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"not\": {"]
#[doc = "        \"required\": ["]
#[doc = "          \"unavailable_reason\""]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      \"required\": ["]
#[doc = "        \"milliseconds\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"available\": {"]
#[doc = "          \"const\": true"]
#[doc = "        }"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"not\": {"]
#[doc = "        \"required\": ["]
#[doc = "          \"milliseconds\""]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      \"required\": ["]
#[doc = "        \"unavailable_reason\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"available\": {"]
#[doc = "          \"const\": false"]
#[doc = "        }"]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  ],"]
#[doc = "  \"required\": ["]
#[doc = "    \"available\","]
#[doc = "    \"backend\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"available\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"backend\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 64,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"milliseconds\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"unavailable_reason\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 512,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged, deny_unknown_fields)]
pub enum HardwareRoundtrip {
    Variant0 {
        available: bool,
        backend: HardwareRoundtripVariant0Backend,
        milliseconds: f64,
    },
    Variant1 {
        available: bool,
        backend: HardwareRoundtripVariant1Backend,
        unavailable_reason: HardwareRoundtripVariant1UnavailableReason,
    },
}
#[doc = "`HardwareRoundtripVariant0Backend`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 64,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct HardwareRoundtripVariant0Backend(::std::string::String);
impl ::std::ops::Deref for HardwareRoundtripVariant0Backend {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<HardwareRoundtripVariant0Backend> for ::std::string::String {
    fn from(value: HardwareRoundtripVariant0Backend) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for HardwareRoundtripVariant0Backend {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 64usize {
            return Err("longer than 64 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for HardwareRoundtripVariant0Backend {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for HardwareRoundtripVariant0Backend {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for HardwareRoundtripVariant0Backend {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for HardwareRoundtripVariant0Backend {
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
#[doc = "`HardwareRoundtripVariant1Backend`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 64,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct HardwareRoundtripVariant1Backend(::std::string::String);
impl ::std::ops::Deref for HardwareRoundtripVariant1Backend {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<HardwareRoundtripVariant1Backend> for ::std::string::String {
    fn from(value: HardwareRoundtripVariant1Backend) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for HardwareRoundtripVariant1Backend {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 64usize {
            return Err("longer than 64 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for HardwareRoundtripVariant1Backend {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for HardwareRoundtripVariant1Backend {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for HardwareRoundtripVariant1Backend {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for HardwareRoundtripVariant1Backend {
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
#[doc = "`HardwareRoundtripVariant1UnavailableReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 512,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct HardwareRoundtripVariant1UnavailableReason(::std::string::String);
impl ::std::ops::Deref for HardwareRoundtripVariant1UnavailableReason {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<HardwareRoundtripVariant1UnavailableReason> for ::std::string::String {
    fn from(value: HardwareRoundtripVariant1UnavailableReason) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for HardwareRoundtripVariant1UnavailableReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 512usize {
            return Err("longer than 512 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for HardwareRoundtripVariant1UnavailableReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String>
    for HardwareRoundtripVariant1UnavailableReason
{
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for HardwareRoundtripVariant1UnavailableReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for HardwareRoundtripVariant1UnavailableReason {
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
#[doc = "`RuntimeIdentity`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"available\","]
#[doc = "    \"identity\","]
#[doc = "    \"kind\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"available\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"identity\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 512,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 64,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RuntimeIdentity {
    pub available: bool,
    pub identity: RuntimeIdentityIdentity,
    pub kind: RuntimeIdentityKind,
}
impl RuntimeIdentity {
    pub fn builder() -> builder::RuntimeIdentity {
        Default::default()
    }
}
#[doc = "`RuntimeIdentityIdentity`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 512,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RuntimeIdentityIdentity(::std::string::String);
impl ::std::ops::Deref for RuntimeIdentityIdentity {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RuntimeIdentityIdentity> for ::std::string::String {
    fn from(value: RuntimeIdentityIdentity) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RuntimeIdentityIdentity {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 512usize {
            return Err("longer than 512 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RuntimeIdentityIdentity {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RuntimeIdentityIdentity {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RuntimeIdentityIdentity {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RuntimeIdentityIdentity {
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
#[doc = "`RuntimeIdentityKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 64,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RuntimeIdentityKind(::std::string::String);
impl ::std::ops::Deref for RuntimeIdentityKind {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RuntimeIdentityKind> for ::std::string::String {
    fn from(value: RuntimeIdentityKind) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RuntimeIdentityKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 64usize {
            return Err("longer than 64 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RuntimeIdentityKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RuntimeIdentityKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RuntimeIdentityKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RuntimeIdentityKind {
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
#[doc = "`SharedMemoryBenchmark`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"bytes_per_second\","]
#[doc = "    \"sample_bytes\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bytes_per_second\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"sample_bytes\": {"]
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
pub struct SharedMemoryBenchmark {
    pub bytes_per_second: f64,
    pub sample_bytes: ::std::num::NonZeroU64,
}
impl SharedMemoryBenchmark {
    pub fn builder() -> builder::SharedMemoryBenchmark {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct CapabilityResult {
        available: ::std::result::Result<bool, ::std::string::String>,
        backend: ::std::result::Result<super::CapabilityResultBackend, ::std::string::String>,
        capability: ::std::result::Result<super::CapabilityResultCapability, ::std::string::String>,
        detail: ::std::result::Result<
            ::std::option::Option<super::CapabilityResultDetail>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for CapabilityResult {
        fn default() -> Self {
            Self {
                available: Err("no value supplied for available".to_string()),
                backend: Err("no value supplied for backend".to_string()),
                capability: Err("no value supplied for capability".to_string()),
                detail: Ok(Default::default()),
            }
        }
    }
    impl CapabilityResult {
        pub fn available<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.available = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for available: {e}"));
            self
        }
        pub fn backend<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CapabilityResultBackend>,
            T::Error: ::std::fmt::Display,
        {
            self.backend = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for backend: {e}"));
            self
        }
        pub fn capability<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CapabilityResultCapability>,
            T::Error: ::std::fmt::Display,
        {
            self.capability = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for capability: {e}"));
            self
        }
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CapabilityResultDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<CapabilityResult> for super::CapabilityResult {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CapabilityResult,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                available: value.available?,
                backend: value.backend?,
                capability: value.capability?,
                detail: value.detail?,
            })
        }
    }
    impl ::std::convert::From<super::CapabilityResult> for CapabilityResult {
        fn from(value: super::CapabilityResult) -> Self {
            Self {
                available: Ok(value.available),
                backend: Ok(value.backend),
                capability: Ok(value.capability),
                detail: Ok(value.detail),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CodecBenchListItem {
        codec: ::std::result::Result<::std::string::String, ::std::string::String>,
        fps_measured: ::std::result::Result<f64, ::std::string::String>,
        hardware: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        height: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for CodecBenchListItem {
        fn default() -> Self {
            Self {
                codec: Err("no value supplied for codec".to_string()),
                fps_measured: Err("no value supplied for fps_measured".to_string()),
                hardware: Ok(Default::default()),
                height: Err("no value supplied for height".to_string()),
            }
        }
    }
    impl CodecBenchListItem {
        pub fn codec<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.codec = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for codec: {e}"));
            self
        }
        pub fn fps_measured<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.fps_measured = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for fps_measured: {e}"));
            self
        }
        pub fn hardware<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.hardware = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hardware: {e}"));
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
    }
    impl ::std::convert::TryFrom<CodecBenchListItem> for super::CodecBenchListItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CodecBenchListItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                codec: value.codec?,
                fps_measured: value.fps_measured?,
                hardware: value.hardware?,
                height: value.height?,
            })
        }
    }
    impl ::std::convert::From<super::CodecBenchListItem> for CodecBenchListItem {
        fn from(value: super::CodecBenchListItem) -> Self {
            Self {
                codec: Ok(value.codec),
                fps_measured: Ok(value.fps_measured),
                hardware: Ok(value.hardware),
                height: Ok(value.height),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeviceAttestation {
        algorithm: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        public_key: ::std::result::Result<super::DeviceAttestationPublicKey, ::std::string::String>,
        signature: ::std::result::Result<super::DeviceAttestationSignature, ::std::string::String>,
    }
    impl ::std::default::Default for DeviceAttestation {
        fn default() -> Self {
            Self {
                algorithm: Err("no value supplied for algorithm".to_string()),
                public_key: Err("no value supplied for public_key".to_string()),
                signature: Err("no value supplied for signature".to_string()),
            }
        }
    }
    impl DeviceAttestation {
        pub fn algorithm<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.algorithm = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for algorithm: {e}"));
            self
        }
        pub fn public_key<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceAttestationPublicKey>,
            T::Error: ::std::fmt::Display,
        {
            self.public_key = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for public_key: {e}"));
            self
        }
        pub fn signature<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceAttestationSignature>,
            T::Error: ::std::fmt::Display,
        {
            self.signature = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for signature: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeviceAttestation> for super::DeviceAttestation {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeviceAttestation,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                algorithm: value.algorithm?,
                public_key: value.public_key?,
                signature: value.signature?,
            })
        }
    }
    impl ::std::convert::From<super::DeviceAttestation> for DeviceAttestation {
        fn from(value: super::DeviceAttestation) -> Self {
            Self {
                algorithm: Ok(value.algorithm),
                public_key: Ok(value.public_key),
                signature: Ok(value.signature),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeviceProfile {
        accelerators: ::std::result::Result<
            ::std::vec::Vec<super::DeviceProfileAcceleratorsItem>,
            ::std::string::String,
        >,
        cpu: ::std::result::Result<super::DeviceProfileCpu, ::std::string::String>,
        measured: ::std::result::Result<super::DeviceProfileMeasured, ::std::string::String>,
        memory: ::std::result::Result<super::DeviceProfileMemory, ::std::string::String>,
        phase0: ::std::result::Result<
            ::std::option::Option<super::DeviceProfilePhase0>,
            ::std::string::String,
        >,
        platform: ::std::result::Result<super::DeviceProfilePlatform, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for DeviceProfile {
        fn default() -> Self {
            Self {
                accelerators: Err("no value supplied for accelerators".to_string()),
                cpu: Err("no value supplied for cpu".to_string()),
                measured: Err("no value supplied for measured".to_string()),
                memory: Err("no value supplied for memory".to_string()),
                phase0: Ok(Default::default()),
                platform: Err("no value supplied for platform".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
            }
        }
    }
    impl DeviceProfile {
        pub fn accelerators<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::DeviceProfileAcceleratorsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.accelerators = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for accelerators: {e}"));
            self
        }
        pub fn cpu<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceProfileCpu>,
            T::Error: ::std::fmt::Display,
        {
            self.cpu = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cpu: {e}"));
            self
        }
        pub fn measured<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceProfileMeasured>,
            T::Error: ::std::fmt::Display,
        {
            self.measured = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for measured: {e}"));
            self
        }
        pub fn memory<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceProfileMemory>,
            T::Error: ::std::fmt::Display,
        {
            self.memory = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for memory: {e}"));
            self
        }
        pub fn phase0<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::DeviceProfilePhase0>>,
            T::Error: ::std::fmt::Display,
        {
            self.phase0 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for phase0: {e}"));
            self
        }
        pub fn platform<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceProfilePlatform>,
            T::Error: ::std::fmt::Display,
        {
            self.platform = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for platform: {e}"));
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
    }
    impl ::std::convert::TryFrom<DeviceProfile> for super::DeviceProfile {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeviceProfile,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                accelerators: value.accelerators?,
                cpu: value.cpu?,
                measured: value.measured?,
                memory: value.memory?,
                phase0: value.phase0?,
                platform: value.platform?,
                schema_version: value.schema_version?,
            })
        }
    }
    impl ::std::convert::From<super::DeviceProfile> for DeviceProfile {
        fn from(value: super::DeviceProfile) -> Self {
            Self {
                accelerators: Ok(value.accelerators),
                cpu: Ok(value.cpu),
                measured: Ok(value.measured),
                memory: Ok(value.memory),
                phase0: Ok(value.phase0),
                platform: Ok(value.platform),
                schema_version: Ok(value.schema_version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeviceProfileAcceleratorsItem {
        kind:
            ::std::result::Result<super::DeviceProfileAcceleratorsItemKind, ::std::string::String>,
        name: ::std::result::Result<::std::string::String, ::std::string::String>,
        vram_bytes: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
    }
    impl ::std::default::Default for DeviceProfileAcceleratorsItem {
        fn default() -> Self {
            Self {
                kind: Err("no value supplied for kind".to_string()),
                name: Err("no value supplied for name".to_string()),
                vram_bytes: Ok(Default::default()),
            }
        }
    }
    impl DeviceProfileAcceleratorsItem {
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceProfileAcceleratorsItemKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn vram_bytes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.vram_bytes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for vram_bytes: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeviceProfileAcceleratorsItem>
        for super::DeviceProfileAcceleratorsItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeviceProfileAcceleratorsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                kind: value.kind?,
                name: value.name?,
                vram_bytes: value.vram_bytes?,
            })
        }
    }
    impl ::std::convert::From<super::DeviceProfileAcceleratorsItem> for DeviceProfileAcceleratorsItem {
        fn from(value: super::DeviceProfileAcceleratorsItem) -> Self {
            Self {
                kind: Ok(value.kind),
                name: Ok(value.name),
                vram_bytes: Ok(value.vram_bytes),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeviceProfileCpu {
        efficiency_cores: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        logical_cores: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        model: ::std::result::Result<::std::string::String, ::std::string::String>,
        performance_cores: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        physical_cores: ::std::result::Result<
            ::std::option::Option<::std::num::NonZeroU64>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for DeviceProfileCpu {
        fn default() -> Self {
            Self {
                efficiency_cores: Ok(Default::default()),
                logical_cores: Err("no value supplied for logical_cores".to_string()),
                model: Err("no value supplied for model".to_string()),
                performance_cores: Ok(Default::default()),
                physical_cores: Ok(Default::default()),
            }
        }
    }
    impl DeviceProfileCpu {
        pub fn efficiency_cores<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.efficiency_cores = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for efficiency_cores: {e}"));
            self
        }
        pub fn logical_cores<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.logical_cores = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for logical_cores: {e}"));
            self
        }
        pub fn model<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.model = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model: {e}"));
            self
        }
        pub fn performance_cores<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.performance_cores = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for performance_cores: {e}"));
            self
        }
        pub fn physical_cores<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::num::NonZeroU64>>,
            T::Error: ::std::fmt::Display,
        {
            self.physical_cores = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for physical_cores: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeviceProfileCpu> for super::DeviceProfileCpu {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeviceProfileCpu,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                efficiency_cores: value.efficiency_cores?,
                logical_cores: value.logical_cores?,
                model: value.model?,
                performance_cores: value.performance_cores?,
                physical_cores: value.physical_cores?,
            })
        }
    }
    impl ::std::convert::From<super::DeviceProfileCpu> for DeviceProfileCpu {
        fn from(value: super::DeviceProfileCpu) -> Self {
            Self {
                efficiency_cores: Ok(value.efficiency_cores),
                logical_cores: Ok(value.logical_cores),
                model: Ok(value.model),
                performance_cores: Ok(value.performance_cores),
                physical_cores: Ok(value.physical_cores),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeviceProfileMeasured {
        decode: ::std::result::Result<
            ::std::option::Option<super::CodecBenchList>,
            ::std::string::String,
        >,
        encode: ::std::result::Result<
            ::std::option::Option<super::CodecBenchList>,
            ::std::string::String,
        >,
        ffmpeg_build: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for DeviceProfileMeasured {
        fn default() -> Self {
            Self {
                decode: Ok(Default::default()),
                encode: Ok(Default::default()),
                ffmpeg_build: Err("no value supplied for ffmpeg_build".to_string()),
            }
        }
    }
    impl DeviceProfileMeasured {
        pub fn decode<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CodecBenchList>>,
            T::Error: ::std::fmt::Display,
        {
            self.decode = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for decode: {e}"));
            self
        }
        pub fn encode<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CodecBenchList>>,
            T::Error: ::std::fmt::Display,
        {
            self.encode = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for encode: {e}"));
            self
        }
        pub fn ffmpeg_build<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.ffmpeg_build = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for ffmpeg_build: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeviceProfileMeasured> for super::DeviceProfileMeasured {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeviceProfileMeasured,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                decode: value.decode?,
                encode: value.encode?,
                ffmpeg_build: value.ffmpeg_build?,
            })
        }
    }
    impl ::std::convert::From<super::DeviceProfileMeasured> for DeviceProfileMeasured {
        fn from(value: super::DeviceProfileMeasured) -> Self {
            Self {
                decode: Ok(value.decode),
                encode: Ok(value.encode),
                ffmpeg_build: Ok(value.ffmpeg_build),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeviceProfileMemory {
        total_bytes: ::std::result::Result<u64, ::std::string::String>,
        unified: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
    }
    impl ::std::default::Default for DeviceProfileMemory {
        fn default() -> Self {
            Self {
                total_bytes: Err("no value supplied for total_bytes".to_string()),
                unified: Ok(Default::default()),
            }
        }
    }
    impl DeviceProfileMemory {
        pub fn total_bytes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.total_bytes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for total_bytes: {e}"));
            self
        }
        pub fn unified<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.unified = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for unified: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeviceProfileMemory> for super::DeviceProfileMemory {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeviceProfileMemory,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                total_bytes: value.total_bytes?,
                unified: value.unified?,
            })
        }
    }
    impl ::std::convert::From<super::DeviceProfileMemory> for DeviceProfileMemory {
        fn from(value: super::DeviceProfileMemory) -> Self {
            Self {
                total_bytes: Ok(value.total_bytes),
                unified: Ok(value.unified),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeviceProfilePhase0 {
        attestation: ::std::result::Result<super::DeviceAttestation, ::std::string::String>,
        available_memory_bytes: ::std::result::Result<u64, ::std::string::String>,
        capability_results:
            ::std::result::Result<::std::vec::Vec<super::CapabilityResult>, ::std::string::String>,
        hardware_fingerprint: ::std::result::Result<
            super::DeviceProfilePhase0HardwareFingerprint,
            ::std::string::String,
        >,
        hardware_roundtrip: ::std::result::Result<super::HardwareRoundtrip, ::std::string::String>,
        measurement_generation:
            ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        runtime_identities:
            ::std::result::Result<::std::vec::Vec<super::RuntimeIdentity>, ::std::string::String>,
        shared_memory: ::std::result::Result<super::SharedMemoryBenchmark, ::std::string::String>,
    }
    impl ::std::default::Default for DeviceProfilePhase0 {
        fn default() -> Self {
            Self {
                attestation: Err("no value supplied for attestation".to_string()),
                available_memory_bytes: Err(
                    "no value supplied for available_memory_bytes".to_string()
                ),
                capability_results: Err("no value supplied for capability_results".to_string()),
                hardware_fingerprint: Err("no value supplied for hardware_fingerprint".to_string()),
                hardware_roundtrip: Err("no value supplied for hardware_roundtrip".to_string()),
                measurement_generation: Err(
                    "no value supplied for measurement_generation".to_string()
                ),
                runtime_identities: Err("no value supplied for runtime_identities".to_string()),
                shared_memory: Err("no value supplied for shared_memory".to_string()),
            }
        }
    }
    impl DeviceProfilePhase0 {
        pub fn attestation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceAttestation>,
            T::Error: ::std::fmt::Display,
        {
            self.attestation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for attestation: {e}"));
            self
        }
        pub fn available_memory_bytes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.available_memory_bytes = value.try_into().map_err(|e| {
                format!("error converting supplied value for available_memory_bytes: {e}")
            });
            self
        }
        pub fn capability_results<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CapabilityResult>>,
            T::Error: ::std::fmt::Display,
        {
            self.capability_results = value.try_into().map_err(|e| {
                format!("error converting supplied value for capability_results: {e}")
            });
            self
        }
        pub fn hardware_fingerprint<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceProfilePhase0HardwareFingerprint>,
            T::Error: ::std::fmt::Display,
        {
            self.hardware_fingerprint = value.try_into().map_err(|e| {
                format!("error converting supplied value for hardware_fingerprint: {e}")
            });
            self
        }
        pub fn hardware_roundtrip<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::HardwareRoundtrip>,
            T::Error: ::std::fmt::Display,
        {
            self.hardware_roundtrip = value.try_into().map_err(|e| {
                format!("error converting supplied value for hardware_roundtrip: {e}")
            });
            self
        }
        pub fn measurement_generation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.measurement_generation = value.try_into().map_err(|e| {
                format!("error converting supplied value for measurement_generation: {e}")
            });
            self
        }
        pub fn runtime_identities<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::RuntimeIdentity>>,
            T::Error: ::std::fmt::Display,
        {
            self.runtime_identities = value.try_into().map_err(|e| {
                format!("error converting supplied value for runtime_identities: {e}")
            });
            self
        }
        pub fn shared_memory<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SharedMemoryBenchmark>,
            T::Error: ::std::fmt::Display,
        {
            self.shared_memory = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shared_memory: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeviceProfilePhase0> for super::DeviceProfilePhase0 {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeviceProfilePhase0,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                attestation: value.attestation?,
                available_memory_bytes: value.available_memory_bytes?,
                capability_results: value.capability_results?,
                hardware_fingerprint: value.hardware_fingerprint?,
                hardware_roundtrip: value.hardware_roundtrip?,
                measurement_generation: value.measurement_generation?,
                runtime_identities: value.runtime_identities?,
                shared_memory: value.shared_memory?,
            })
        }
    }
    impl ::std::convert::From<super::DeviceProfilePhase0> for DeviceProfilePhase0 {
        fn from(value: super::DeviceProfilePhase0) -> Self {
            Self {
                attestation: Ok(value.attestation),
                available_memory_bytes: Ok(value.available_memory_bytes),
                capability_results: Ok(value.capability_results),
                hardware_fingerprint: Ok(value.hardware_fingerprint),
                hardware_roundtrip: Ok(value.hardware_roundtrip),
                measurement_generation: Ok(value.measurement_generation),
                runtime_identities: Ok(value.runtime_identities),
                shared_memory: Ok(value.shared_memory),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeviceProfilePlatform {
        arch: ::std::result::Result<super::DeviceProfilePlatformArch, ::std::string::String>,
        os: ::std::result::Result<super::DeviceProfilePlatformOs, ::std::string::String>,
        os_version: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for DeviceProfilePlatform {
        fn default() -> Self {
            Self {
                arch: Err("no value supplied for arch".to_string()),
                os: Err("no value supplied for os".to_string()),
                os_version: Ok(Default::default()),
            }
        }
    }
    impl DeviceProfilePlatform {
        pub fn arch<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceProfilePlatformArch>,
            T::Error: ::std::fmt::Display,
        {
            self.arch = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for arch: {e}"));
            self
        }
        pub fn os<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DeviceProfilePlatformOs>,
            T::Error: ::std::fmt::Display,
        {
            self.os = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for os: {e}"));
            self
        }
        pub fn os_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.os_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for os_version: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeviceProfilePlatform> for super::DeviceProfilePlatform {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeviceProfilePlatform,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                arch: value.arch?,
                os: value.os?,
                os_version: value.os_version?,
            })
        }
    }
    impl ::std::convert::From<super::DeviceProfilePlatform> for DeviceProfilePlatform {
        fn from(value: super::DeviceProfilePlatform) -> Self {
            Self {
                arch: Ok(value.arch),
                os: Ok(value.os),
                os_version: Ok(value.os_version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RuntimeIdentity {
        available: ::std::result::Result<bool, ::std::string::String>,
        identity: ::std::result::Result<super::RuntimeIdentityIdentity, ::std::string::String>,
        kind: ::std::result::Result<super::RuntimeIdentityKind, ::std::string::String>,
    }
    impl ::std::default::Default for RuntimeIdentity {
        fn default() -> Self {
            Self {
                available: Err("no value supplied for available".to_string()),
                identity: Err("no value supplied for identity".to_string()),
                kind: Err("no value supplied for kind".to_string()),
            }
        }
    }
    impl RuntimeIdentity {
        pub fn available<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.available = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for available: {e}"));
            self
        }
        pub fn identity<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RuntimeIdentityIdentity>,
            T::Error: ::std::fmt::Display,
        {
            self.identity = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for identity: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RuntimeIdentityKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<RuntimeIdentity> for super::RuntimeIdentity {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RuntimeIdentity,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                available: value.available?,
                identity: value.identity?,
                kind: value.kind?,
            })
        }
    }
    impl ::std::convert::From<super::RuntimeIdentity> for RuntimeIdentity {
        fn from(value: super::RuntimeIdentity) -> Self {
            Self {
                available: Ok(value.available),
                identity: Ok(value.identity),
                kind: Ok(value.kind),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SharedMemoryBenchmark {
        bytes_per_second: ::std::result::Result<f64, ::std::string::String>,
        sample_bytes: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for SharedMemoryBenchmark {
        fn default() -> Self {
            Self {
                bytes_per_second: Err("no value supplied for bytes_per_second".to_string()),
                sample_bytes: Err("no value supplied for sample_bytes".to_string()),
            }
        }
    }
    impl SharedMemoryBenchmark {
        pub fn bytes_per_second<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.bytes_per_second = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for bytes_per_second: {e}"));
            self
        }
        pub fn sample_bytes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.sample_bytes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sample_bytes: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SharedMemoryBenchmark> for super::SharedMemoryBenchmark {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SharedMemoryBenchmark,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                bytes_per_second: value.bytes_per_second?,
                sample_bytes: value.sample_bytes?,
            })
        }
    }
    impl ::std::convert::From<super::SharedMemoryBenchmark> for SharedMemoryBenchmark {
        fn from(value: super::SharedMemoryBenchmark) -> Self {
            Self {
                bytes_per_second: Ok(value.bytes_per_second),
                sample_bytes: Ok(value.sample_bytes),
            }
        }
    }
}
