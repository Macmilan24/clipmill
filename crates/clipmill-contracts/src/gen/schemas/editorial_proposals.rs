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
#[doc = "`Decoding`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"constrained\","]
#[doc = "    \"max_output_tokens\","]
#[doc = "    \"seed\","]
#[doc = "    \"temperature_milli\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"constrained\": {"]
#[doc = "      \"description\": \"Whether the reply was decoded under the answer's JSON schema, so it is structurally valid or the window is marked malformed — never a parse of prose.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"max_output_tokens\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"seed\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"temperature_milli\": {"]
#[doc = "      \"description\": \"Temperature in thousandths, an integer so the key is exact. Zero is what the local route runs at.\","]
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
pub struct Decoding {
    #[doc = "Whether the reply was decoded under the answer's JSON schema, so it is structurally valid or the window is marked malformed — never a parse of prose."]
    pub constrained: bool,
    pub max_output_tokens: ::std::num::NonZeroU64,
    pub seed: u64,
    #[doc = "Temperature in thousandths, an integer so the key is exact. Zero is what the local route runs at."]
    pub temperature_milli: u64,
}
impl Decoding {
    pub fn builder() -> builder::Decoding {
        Default::default()
    }
}
#[doc = "What an editorial model proposed, window by window, before anything checked it (plan, Milestone 2). A proposal is a moment with a hook, the setup it needs, and a payoff, cited by sentence index — and by word index only at its edges — because a model that invents a timestamp invents a cut; timestamps come from the alignment, through the validator, never from here. A window may honestly hold no moment, and a window the model failed on is recorded as a failure rather than as an empty answer, so that 'nothing worthwhile here' and 'the model could not say' never look alike. This document is the model's word and is trusted for nothing: `editorial-validate` refuses every reference it cannot walk back to the transcript before a candidate exists."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.editorial.proposals.v1.json\","]
#[doc = "  \"title\": \"EditorialProposals\","]
#[doc = "  \"description\": \"What an editorial model proposed, window by window, before anything checked it (plan, Milestone 2). A proposal is a moment with a hook, the setup it needs, and a payoff, cited by sentence index — and by word index only at its edges — because a model that invents a timestamp invents a cut; timestamps come from the alignment, through the validator, never from here. A window may honestly hold no moment, and a window the model failed on is recorded as a failure rather than as an empty answer, so that 'nothing worthwhile here' and 'the model could not say' never look alike. This document is the model's word and is trusted for nothing: `editorial-validate` refuses every reference it cannot walk back to the transcript before a candidate exists.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"inputs\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"windows\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"windows_artifact_id\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"windows_artifact_id\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"$ref\": \"#/$defs/model_producer\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.editorial.proposals.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"windows\": {"]
#[doc = "      \"description\": \"One entry per window of the input, in order, whether or not the model proposed anything for it. A window absent from this list was not asked, which is a different document from one the model answered 'none' to.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/window_answer\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditorialProposals {
    pub inputs: EditorialProposalsInputs,
    pub producer: ModelProducer,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    #[doc = "One entry per window of the input, in order, whether or not the model proposed anything for it. A window absent from this list was not asked, which is a different document from one the model answered 'none' to."]
    pub windows: ::std::vec::Vec<WindowAnswer>,
}
impl EditorialProposals {
    pub fn builder() -> builder::EditorialProposals {
        Default::default()
    }
}
#[doc = "`EditorialProposalsInputs`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"windows_artifact_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"windows_artifact_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditorialProposalsInputs {
    pub windows_artifact_id: Sha256,
}
impl EditorialProposalsInputs {
    pub fn builder() -> builder::EditorialProposalsInputs {
        Default::default()
    }
}
#[doc = "`Model`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"name\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"digest\": {"]
#[doc = "      \"description\": \"The registry's digest of the local weights. Absent on the cloud route, where the weights are not ours to hash.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"description\": \"The registry name of a local model, or the provider's model identifier on the cloud route.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"provider\": {"]
#[doc = "      \"description\": \"The cloud provider, on that route.\","]
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
pub struct Model {
    #[doc = "The registry's digest of the local weights. Absent on the cloud route, where the weights are not ours to hash."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub digest: ::std::option::Option<Sha256>,
    #[doc = "The registry name of a local model, or the provider's model identifier on the cloud route."]
    pub name: ModelName,
    #[doc = "The cloud provider, on that route."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub provider: ::std::option::Option<ModelProvider>,
}
impl Model {
    pub fn builder() -> builder::Model {
        Default::default()
    }
}
#[doc = "The registry name of a local model, or the provider's model identifier on the cloud route."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The registry name of a local model, or the provider's model identifier on the cloud route.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ModelName(::std::string::String);
impl ::std::ops::Deref for ModelName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelName> for ::std::string::String {
    fn from(value: ModelName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelName {
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
#[doc = "Who answered, and under what: the implementation, the model by name and digest, the route the request took, the prompt version and the decoding configuration. Together with the inputs this is the cache identity of the answer; a changed prompt is a different result and an unchanged one is a hit."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Who answered, and under what: the implementation, the model by name and digest, the route the request took, the prompt version and the decoding configuration. Together with the inputs this is the cache identity of the answer; a changed prompt is a different result and an unchanged one is a hit.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"decoding\","]
#[doc = "    \"implementation\","]
#[doc = "    \"model\","]
#[doc = "    \"prompt_version\","]
#[doc = "    \"route\","]
#[doc = "    \"stage\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"decoding\": {"]
#[doc = "      \"$ref\": \"#/$defs/decoding\""]
#[doc = "    },"]
#[doc = "    \"implementation\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"model\": {"]
#[doc = "      \"$ref\": \"#/$defs/model\""]
#[doc = "    },"]
#[doc = "    \"prompt_version\": {"]
#[doc = "      \"description\": \"The versioned prompt and rubric the model was given, e.g. 'propose.v1', with its digest so a reworded prompt under the same name is a different answer.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"route\": {"]
#[doc = "      \"description\": \"'local' ran on this machine under the Local Lock; 'cloud' left it, with the provider named in the model.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"local\","]
#[doc = "        \"cloud\""]
#[doc = "      ]"]
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
pub struct ModelProducer {
    pub decoding: Decoding,
    pub implementation: ModelProducerImplementation,
    pub model: Model,
    #[doc = "The versioned prompt and rubric the model was given, e.g. 'propose.v1', with its digest so a reworded prompt under the same name is a different answer."]
    pub prompt_version: ModelProducerPromptVersion,
    #[doc = "'local' ran on this machine under the Local Lock; 'cloud' left it, with the provider named in the model."]
    pub route: ModelProducerRoute,
    pub stage: ModelProducerStage,
}
impl ModelProducer {
    pub fn builder() -> builder::ModelProducer {
        Default::default()
    }
}
#[doc = "`ModelProducerImplementation`"]
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
pub struct ModelProducerImplementation(::std::string::String);
impl ::std::ops::Deref for ModelProducerImplementation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelProducerImplementation> for ::std::string::String {
    fn from(value: ModelProducerImplementation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelProducerImplementation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelProducerImplementation {
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
#[doc = "The versioned prompt and rubric the model was given, e.g. 'propose.v1', with its digest so a reworded prompt under the same name is a different answer."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The versioned prompt and rubric the model was given, e.g. 'propose.v1', with its digest so a reworded prompt under the same name is a different answer.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ModelProducerPromptVersion(::std::string::String);
impl ::std::ops::Deref for ModelProducerPromptVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelProducerPromptVersion> for ::std::string::String {
    fn from(value: ModelProducerPromptVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelProducerPromptVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProducerPromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelProducerPromptVersion {
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
#[doc = "'local' ran on this machine under the Local Lock; 'cloud' left it, with the provider named in the model."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"'local' ran on this machine under the Local Lock; 'cloud' left it, with the provider named in the model.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"local\","]
#[doc = "    \"cloud\""]
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
pub enum ModelProducerRoute {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "cloud")]
    Cloud,
}
impl ::std::fmt::Display for ModelProducerRoute {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Local => f.write_str("local"),
            Self::Cloud => f.write_str("cloud"),
        }
    }
}
impl ::std::str::FromStr for ModelProducerRoute {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "local" => Ok(Self::Local),
            "cloud" => Ok(Self::Cloud),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ModelProducerRoute {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProducerRoute {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProducerRoute {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`ModelProducerStage`"]
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
pub struct ModelProducerStage(::std::string::String);
impl ::std::ops::Deref for ModelProducerStage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelProducerStage> for ::std::string::String {
    fn from(value: ModelProducerStage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelProducerStage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProducerStage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelProducerStage {
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
#[doc = "The cloud provider, on that route."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The cloud provider, on that route.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ModelProvider(::std::string::String);
impl ::std::ops::Deref for ModelProvider {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ModelProvider> for ::std::string::String {
    fn from(value: ModelProvider) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ModelProvider {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ModelProvider {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ModelProvider {
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
#[doc = "`Proposal`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"first_sentence_index\","]
#[doc = "    \"hook\","]
#[doc = "    \"id\","]
#[doc = "    \"payoff\","]
#[doc = "    \"reason\","]
#[doc = "    \"sentence_count\","]
#[doc = "    \"setup\","]
#[doc = "    \"splittable\","]
#[doc = "    \"title\","]
#[doc = "    \"uncertainties\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_word_index\": {"]
#[doc = "      \"description\": \"Optionally, the last word inside the last sentence the moment really ends on. Transcript word index; must lie inside the last sentence.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"first_sentence_index\": {"]
#[doc = "      \"description\": \"The moment, as whole sentences of the window's core. Sentences are the unit a model can be trusted to point at.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"hook\": {"]
#[doc = "      \"description\": \"In the model's words: what makes a viewer stay for the first seconds.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"description\": \"Unique within the document, e.g. 'prop_3_1' for the first proposal of window 3.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^prop_[0-9]+_[0-9]+$\""]
#[doc = "    },"]
#[doc = "    \"payoff\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"description\": \"One source-grounded sentence on why this is worth a clip. Grounded means it can be checked against the cited sentences, not that it was.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"sentence_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"setup\": {"]
#[doc = "      \"description\": \"What the viewer must have heard for the payoff to land, and whether the moment carries it.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"splittable\": {"]
#[doc = "      \"description\": \"The model's view that the moment holds shorter moments that stand alone; when true, `subspans` names them, so a proposal over the duration target is split into meaningful parts rather than discarded for length.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"start_word_index\": {"]
#[doc = "      \"description\": \"Optionally, a word inside the first sentence the moment really begins on, when the sentence's opening is not part of it. Transcript word index; must lie inside the first sentence.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"subspans\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"first_sentence_index\","]
#[doc = "          \"sentence_count\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"first_sentence_index\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"sentence_count\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 1.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 120,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"uncertainties\": {"]
#[doc = "      \"description\": \"What the model said it was unsure of — a reference it could not resolve, a claim it could not verify — each one line.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    #[doc = "Optionally, the last word inside the last sentence the moment really ends on. Transcript word index; must lie inside the last sentence."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub end_word_index: ::std::option::Option<u64>,
    #[doc = "The moment, as whole sentences of the window's core. Sentences are the unit a model can be trusted to point at."]
    pub first_sentence_index: u64,
    #[doc = "In the model's words: what makes a viewer stay for the first seconds."]
    pub hook: ProposalHook,
    #[doc = "Unique within the document, e.g. 'prop_3_1' for the first proposal of window 3."]
    pub id: ProposalId,
    pub payoff: ProposalPayoff,
    #[doc = "One source-grounded sentence on why this is worth a clip. Grounded means it can be checked against the cited sentences, not that it was."]
    pub reason: ProposalReason,
    pub sentence_count: ::std::num::NonZeroU64,
    #[doc = "What the viewer must have heard for the payoff to land, and whether the moment carries it."]
    pub setup: ProposalSetup,
    #[doc = "The model's view that the moment holds shorter moments that stand alone; when true, `subspans` names them, so a proposal over the duration target is split into meaningful parts rather than discarded for length."]
    pub splittable: bool,
    #[doc = "Optionally, a word inside the first sentence the moment really begins on, when the sentence's opening is not part of it. Transcript word index; must lie inside the first sentence."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub start_word_index: ::std::option::Option<u64>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub subspans: ::std::vec::Vec<ProposalSubspansItem>,
    pub title: ProposalTitle,
    #[doc = "What the model said it was unsure of — a reference it could not resolve, a claim it could not verify — each one line."]
    pub uncertainties: ::std::vec::Vec<ProposalUncertaintiesItem>,
}
impl Proposal {
    pub fn builder() -> builder::Proposal {
        Default::default()
    }
}
#[doc = "In the model's words: what makes a viewer stay for the first seconds."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"In the model's words: what makes a viewer stay for the first seconds.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProposalHook(::std::string::String);
impl ::std::ops::Deref for ProposalHook {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposalHook> for ::std::string::String {
    fn from(value: ProposalHook) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposalHook {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposalHook {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposalHook {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposalHook {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposalHook {
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
#[doc = "Unique within the document, e.g. 'prop_3_1' for the first proposal of window 3."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Unique within the document, e.g. 'prop_3_1' for the first proposal of window 3.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^prop_[0-9]+_[0-9]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProposalId(::std::string::String);
impl ::std::ops::Deref for ProposalId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposalId> for ::std::string::String {
    fn from(value: ProposalId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposalId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^prop_[0-9]+_[0-9]+$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^prop_[0-9]+_[0-9]+$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposalId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposalId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposalId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposalId {
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
#[doc = "`ProposalPayoff`"]
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
pub struct ProposalPayoff(::std::string::String);
impl ::std::ops::Deref for ProposalPayoff {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposalPayoff> for ::std::string::String {
    fn from(value: ProposalPayoff) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposalPayoff {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposalPayoff {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposalPayoff {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposalPayoff {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposalPayoff {
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
#[doc = "One source-grounded sentence on why this is worth a clip. Grounded means it can be checked against the cited sentences, not that it was."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"One source-grounded sentence on why this is worth a clip. Grounded means it can be checked against the cited sentences, not that it was.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProposalReason(::std::string::String);
impl ::std::ops::Deref for ProposalReason {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposalReason> for ::std::string::String {
    fn from(value: ProposalReason) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposalReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposalReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposalReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposalReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposalReason {
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
#[doc = "What the viewer must have heard for the payoff to land, and whether the moment carries it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What the viewer must have heard for the payoff to land, and whether the moment carries it.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProposalSetup(::std::string::String);
impl ::std::ops::Deref for ProposalSetup {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposalSetup> for ::std::string::String {
    fn from(value: ProposalSetup) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposalSetup {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposalSetup {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposalSetup {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposalSetup {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposalSetup {
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
#[doc = "`ProposalSubspansItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"first_sentence_index\","]
#[doc = "    \"sentence_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"first_sentence_index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"sentence_count\": {"]
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
pub struct ProposalSubspansItem {
    pub first_sentence_index: u64,
    pub sentence_count: ::std::num::NonZeroU64,
}
impl ProposalSubspansItem {
    pub fn builder() -> builder::ProposalSubspansItem {
        Default::default()
    }
}
#[doc = "`ProposalTitle`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 120,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProposalTitle(::std::string::String);
impl ::std::ops::Deref for ProposalTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposalTitle> for ::std::string::String {
    fn from(value: ProposalTitle) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposalTitle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 120usize {
            return Err("longer than 120 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposalTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposalTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposalTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposalTitle {
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
#[doc = "`ProposalUncertaintiesItem`"]
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
pub struct ProposalUncertaintiesItem(::std::string::String);
impl ::std::ops::Deref for ProposalUncertaintiesItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProposalUncertaintiesItem> for ::std::string::String {
    fn from(value: ProposalUncertaintiesItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProposalUncertaintiesItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProposalUncertaintiesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProposalUncertaintiesItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProposalUncertaintiesItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProposalUncertaintiesItem {
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
#[doc = "`WindowAnswer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"proposals\","]
#[doc = "    \"status\","]
#[doc = "    \"window_index\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"failure\": {"]
#[doc = "      \"description\": \"For 'malformed' and 'failed': what happened, in one sentence, and whether trying again could help.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"detail\","]
#[doc = "        \"failure_class\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"detail\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"failure_class\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"deterministic\","]
#[doc = "            \"transient\","]
#[doc = "            \"cancelled\","]
#[doc = "            \"budget\""]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"proposals\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/proposal\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"description\": \"'answered' with proposals; 'none' when the model said there was no complete moment in the window; 'malformed' when its reply did not fit the schema; 'failed' when the model could not be run — each a different thing to show a person.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"answered\","]
#[doc = "        \"none\","]
#[doc = "        \"malformed\","]
#[doc = "        \"failed\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"window_index\": {"]
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
pub struct WindowAnswer {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub failure: ::std::option::Option<WindowAnswerFailure>,
    pub proposals: ::std::vec::Vec<Proposal>,
    #[doc = "'answered' with proposals; 'none' when the model said there was no complete moment in the window; 'malformed' when its reply did not fit the schema; 'failed' when the model could not be run — each a different thing to show a person."]
    pub status: WindowAnswerStatus,
    pub window_index: u64,
}
impl WindowAnswer {
    pub fn builder() -> builder::WindowAnswer {
        Default::default()
    }
}
#[doc = "For 'malformed' and 'failed': what happened, in one sentence, and whether trying again could help."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"For 'malformed' and 'failed': what happened, in one sentence, and whether trying again could help.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"detail\","]
#[doc = "    \"failure_class\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"detail\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"failure_class\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"deterministic\","]
#[doc = "        \"transient\","]
#[doc = "        \"cancelled\","]
#[doc = "        \"budget\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WindowAnswerFailure {
    pub detail: WindowAnswerFailureDetail,
    pub failure_class: WindowAnswerFailureFailureClass,
}
impl WindowAnswerFailure {
    pub fn builder() -> builder::WindowAnswerFailure {
        Default::default()
    }
}
#[doc = "`WindowAnswerFailureDetail`"]
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
pub struct WindowAnswerFailureDetail(::std::string::String);
impl ::std::ops::Deref for WindowAnswerFailureDetail {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<WindowAnswerFailureDetail> for ::std::string::String {
    fn from(value: WindowAnswerFailureDetail) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for WindowAnswerFailureDetail {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for WindowAnswerFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for WindowAnswerFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for WindowAnswerFailureDetail {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WindowAnswerFailureDetail {
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
#[doc = "`WindowAnswerFailureFailureClass`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"deterministic\","]
#[doc = "    \"transient\","]
#[doc = "    \"cancelled\","]
#[doc = "    \"budget\""]
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
pub enum WindowAnswerFailureFailureClass {
    #[serde(rename = "deterministic")]
    Deterministic,
    #[serde(rename = "transient")]
    Transient,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "budget")]
    Budget,
}
impl ::std::fmt::Display for WindowAnswerFailureFailureClass {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Deterministic => f.write_str("deterministic"),
            Self::Transient => f.write_str("transient"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Budget => f.write_str("budget"),
        }
    }
}
impl ::std::str::FromStr for WindowAnswerFailureFailureClass {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "deterministic" => Ok(Self::Deterministic),
            "transient" => Ok(Self::Transient),
            "cancelled" => Ok(Self::Cancelled),
            "budget" => Ok(Self::Budget),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for WindowAnswerFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for WindowAnswerFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for WindowAnswerFailureFailureClass {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "'answered' with proposals; 'none' when the model said there was no complete moment in the window; 'malformed' when its reply did not fit the schema; 'failed' when the model could not be run — each a different thing to show a person."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"'answered' with proposals; 'none' when the model said there was no complete moment in the window; 'malformed' when its reply did not fit the schema; 'failed' when the model could not be run — each a different thing to show a person.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"answered\","]
#[doc = "    \"none\","]
#[doc = "    \"malformed\","]
#[doc = "    \"failed\""]
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
pub enum WindowAnswerStatus {
    #[serde(rename = "answered")]
    Answered,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "malformed")]
    Malformed,
    #[serde(rename = "failed")]
    Failed,
}
impl ::std::fmt::Display for WindowAnswerStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Answered => f.write_str("answered"),
            Self::None => f.write_str("none"),
            Self::Malformed => f.write_str("malformed"),
            Self::Failed => f.write_str("failed"),
        }
    }
}
impl ::std::str::FromStr for WindowAnswerStatus {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "answered" => Ok(Self::Answered),
            "none" => Ok(Self::None),
            "malformed" => Ok(Self::Malformed),
            "failed" => Ok(Self::Failed),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for WindowAnswerStatus {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for WindowAnswerStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for WindowAnswerStatus {
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
    pub struct Decoding {
        constrained: ::std::result::Result<bool, ::std::string::String>,
        max_output_tokens: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        seed: ::std::result::Result<u64, ::std::string::String>,
        temperature_milli: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Decoding {
        fn default() -> Self {
            Self {
                constrained: Err("no value supplied for constrained".to_string()),
                max_output_tokens: Err("no value supplied for max_output_tokens".to_string()),
                seed: Err("no value supplied for seed".to_string()),
                temperature_milli: Err("no value supplied for temperature_milli".to_string()),
            }
        }
    }
    impl Decoding {
        pub fn constrained<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.constrained = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for constrained: {e}"));
            self
        }
        pub fn max_output_tokens<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_output_tokens = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max_output_tokens: {e}"));
            self
        }
        pub fn seed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.seed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for seed: {e}"));
            self
        }
        pub fn temperature_milli<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.temperature_milli = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for temperature_milli: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Decoding> for super::Decoding {
        type Error = super::error::ConversionError;
        fn try_from(value: Decoding) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                constrained: value.constrained?,
                max_output_tokens: value.max_output_tokens?,
                seed: value.seed?,
                temperature_milli: value.temperature_milli?,
            })
        }
    }
    impl ::std::convert::From<super::Decoding> for Decoding {
        fn from(value: super::Decoding) -> Self {
            Self {
                constrained: Ok(value.constrained),
                max_output_tokens: Ok(value.max_output_tokens),
                seed: Ok(value.seed),
                temperature_milli: Ok(value.temperature_milli),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialProposals {
        inputs: ::std::result::Result<super::EditorialProposalsInputs, ::std::string::String>,
        producer: ::std::result::Result<super::ModelProducer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        windows: ::std::result::Result<::std::vec::Vec<super::WindowAnswer>, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialProposals {
        fn default() -> Self {
            Self {
                inputs: Err("no value supplied for inputs".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                windows: Err("no value supplied for windows".to_string()),
            }
        }
    }
    impl EditorialProposals {
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EditorialProposalsInputs>,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
            self
        }
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducer>,
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
        pub fn windows<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::WindowAnswer>>,
            T::Error: ::std::fmt::Display,
        {
            self.windows = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for windows: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<EditorialProposals> for super::EditorialProposals {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialProposals,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                inputs: value.inputs?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                windows: value.windows?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialProposals> for EditorialProposals {
        fn from(value: super::EditorialProposals) -> Self {
            Self {
                inputs: Ok(value.inputs),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                windows: Ok(value.windows),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorialProposalsInputs {
        windows_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for EditorialProposalsInputs {
        fn default() -> Self {
            Self {
                windows_artifact_id: Err("no value supplied for windows_artifact_id".to_string()),
            }
        }
    }
    impl EditorialProposalsInputs {
        pub fn windows_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.windows_artifact_id = value.try_into().map_err(|e| {
                format!("error converting supplied value for windows_artifact_id: {e}")
            });
            self
        }
    }
    impl ::std::convert::TryFrom<EditorialProposalsInputs> for super::EditorialProposalsInputs {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EditorialProposalsInputs,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                windows_artifact_id: value.windows_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::EditorialProposalsInputs> for EditorialProposalsInputs {
        fn from(value: super::EditorialProposalsInputs) -> Self {
            Self {
                windows_artifact_id: Ok(value.windows_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Model {
        digest: ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
        name: ::std::result::Result<super::ModelName, ::std::string::String>,
        provider: ::std::result::Result<
            ::std::option::Option<super::ModelProvider>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Model {
        fn default() -> Self {
            Self {
                digest: Ok(Default::default()),
                name: Err("no value supplied for name".to_string()),
                provider: Ok(Default::default()),
            }
        }
    }
    impl Model {
        pub fn digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for digest: {e}"));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn provider<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ModelProvider>>,
            T::Error: ::std::fmt::Display,
        {
            self.provider = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for provider: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Model> for super::Model {
        type Error = super::error::ConversionError;
        fn try_from(value: Model) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                digest: value.digest?,
                name: value.name?,
                provider: value.provider?,
            })
        }
    }
    impl ::std::convert::From<super::Model> for Model {
        fn from(value: super::Model) -> Self {
            Self {
                digest: Ok(value.digest),
                name: Ok(value.name),
                provider: Ok(value.provider),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ModelProducer {
        decoding: ::std::result::Result<super::Decoding, ::std::string::String>,
        implementation:
            ::std::result::Result<super::ModelProducerImplementation, ::std::string::String>,
        model: ::std::result::Result<super::Model, ::std::string::String>,
        prompt_version:
            ::std::result::Result<super::ModelProducerPromptVersion, ::std::string::String>,
        route: ::std::result::Result<super::ModelProducerRoute, ::std::string::String>,
        stage: ::std::result::Result<super::ModelProducerStage, ::std::string::String>,
    }
    impl ::std::default::Default for ModelProducer {
        fn default() -> Self {
            Self {
                decoding: Err("no value supplied for decoding".to_string()),
                implementation: Err("no value supplied for implementation".to_string()),
                model: Err("no value supplied for model".to_string()),
                prompt_version: Err("no value supplied for prompt_version".to_string()),
                route: Err("no value supplied for route".to_string()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl ModelProducer {
        pub fn decoding<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Decoding>,
            T::Error: ::std::fmt::Display,
        {
            self.decoding = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for decoding: {e}"));
            self
        }
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducerImplementation>,
            T::Error: ::std::fmt::Display,
        {
            self.implementation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for implementation: {e}"));
            self
        }
        pub fn model<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Model>,
            T::Error: ::std::fmt::Display,
        {
            self.model = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model: {e}"));
            self
        }
        pub fn prompt_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducerPromptVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt_version: {e}"));
            self
        }
        pub fn route<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducerRoute>,
            T::Error: ::std::fmt::Display,
        {
            self.route = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for route: {e}"));
            self
        }
        pub fn stage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ModelProducerStage>,
            T::Error: ::std::fmt::Display,
        {
            self.stage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stage: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ModelProducer> for super::ModelProducer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ModelProducer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                decoding: value.decoding?,
                implementation: value.implementation?,
                model: value.model?,
                prompt_version: value.prompt_version?,
                route: value.route?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::ModelProducer> for ModelProducer {
        fn from(value: super::ModelProducer) -> Self {
            Self {
                decoding: Ok(value.decoding),
                implementation: Ok(value.implementation),
                model: Ok(value.model),
                prompt_version: Ok(value.prompt_version),
                route: Ok(value.route),
                stage: Ok(value.stage),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Proposal {
        end_word_index: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        first_sentence_index: ::std::result::Result<u64, ::std::string::String>,
        hook: ::std::result::Result<super::ProposalHook, ::std::string::String>,
        id: ::std::result::Result<super::ProposalId, ::std::string::String>,
        payoff: ::std::result::Result<super::ProposalPayoff, ::std::string::String>,
        reason: ::std::result::Result<super::ProposalReason, ::std::string::String>,
        sentence_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        setup: ::std::result::Result<super::ProposalSetup, ::std::string::String>,
        splittable: ::std::result::Result<bool, ::std::string::String>,
        start_word_index: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        subspans: ::std::result::Result<
            ::std::vec::Vec<super::ProposalSubspansItem>,
            ::std::string::String,
        >,
        title: ::std::result::Result<super::ProposalTitle, ::std::string::String>,
        uncertainties: ::std::result::Result<
            ::std::vec::Vec<super::ProposalUncertaintiesItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Proposal {
        fn default() -> Self {
            Self {
                end_word_index: Ok(Default::default()),
                first_sentence_index: Err("no value supplied for first_sentence_index".to_string()),
                hook: Err("no value supplied for hook".to_string()),
                id: Err("no value supplied for id".to_string()),
                payoff: Err("no value supplied for payoff".to_string()),
                reason: Err("no value supplied for reason".to_string()),
                sentence_count: Err("no value supplied for sentence_count".to_string()),
                setup: Err("no value supplied for setup".to_string()),
                splittable: Err("no value supplied for splittable".to_string()),
                start_word_index: Ok(Default::default()),
                subspans: Ok(Default::default()),
                title: Err("no value supplied for title".to_string()),
                uncertainties: Err("no value supplied for uncertainties".to_string()),
            }
        }
    }
    impl Proposal {
        pub fn end_word_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.end_word_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_word_index: {e}"));
            self
        }
        pub fn first_sentence_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_sentence_index = value.try_into().map_err(|e| {
                format!("error converting supplied value for first_sentence_index: {e}")
            });
            self
        }
        pub fn hook<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposalHook>,
            T::Error: ::std::fmt::Display,
        {
            self.hook = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hook: {e}"));
            self
        }
        pub fn id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposalId>,
            T::Error: ::std::fmt::Display,
        {
            self.id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for id: {e}"));
            self
        }
        pub fn payoff<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposalPayoff>,
            T::Error: ::std::fmt::Display,
        {
            self.payoff = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payoff: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposalReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
        pub fn sentence_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.sentence_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sentence_count: {e}"));
            self
        }
        pub fn setup<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposalSetup>,
            T::Error: ::std::fmt::Display,
        {
            self.setup = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for setup: {e}"));
            self
        }
        pub fn splittable<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.splittable = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for splittable: {e}"));
            self
        }
        pub fn start_word_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.start_word_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for start_word_index: {e}"));
            self
        }
        pub fn subspans<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ProposalSubspansItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.subspans = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for subspans: {e}"));
            self
        }
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProposalTitle>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {e}"));
            self
        }
        pub fn uncertainties<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ProposalUncertaintiesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.uncertainties = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for uncertainties: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Proposal> for super::Proposal {
        type Error = super::error::ConversionError;
        fn try_from(value: Proposal) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_word_index: value.end_word_index?,
                first_sentence_index: value.first_sentence_index?,
                hook: value.hook?,
                id: value.id?,
                payoff: value.payoff?,
                reason: value.reason?,
                sentence_count: value.sentence_count?,
                setup: value.setup?,
                splittable: value.splittable?,
                start_word_index: value.start_word_index?,
                subspans: value.subspans?,
                title: value.title?,
                uncertainties: value.uncertainties?,
            })
        }
    }
    impl ::std::convert::From<super::Proposal> for Proposal {
        fn from(value: super::Proposal) -> Self {
            Self {
                end_word_index: Ok(value.end_word_index),
                first_sentence_index: Ok(value.first_sentence_index),
                hook: Ok(value.hook),
                id: Ok(value.id),
                payoff: Ok(value.payoff),
                reason: Ok(value.reason),
                sentence_count: Ok(value.sentence_count),
                setup: Ok(value.setup),
                splittable: Ok(value.splittable),
                start_word_index: Ok(value.start_word_index),
                subspans: Ok(value.subspans),
                title: Ok(value.title),
                uncertainties: Ok(value.uncertainties),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ProposalSubspansItem {
        first_sentence_index: ::std::result::Result<u64, ::std::string::String>,
        sentence_count: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for ProposalSubspansItem {
        fn default() -> Self {
            Self {
                first_sentence_index: Err("no value supplied for first_sentence_index".to_string()),
                sentence_count: Err("no value supplied for sentence_count".to_string()),
            }
        }
    }
    impl ProposalSubspansItem {
        pub fn first_sentence_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.first_sentence_index = value.try_into().map_err(|e| {
                format!("error converting supplied value for first_sentence_index: {e}")
            });
            self
        }
        pub fn sentence_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.sentence_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sentence_count: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ProposalSubspansItem> for super::ProposalSubspansItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ProposalSubspansItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                first_sentence_index: value.first_sentence_index?,
                sentence_count: value.sentence_count?,
            })
        }
    }
    impl ::std::convert::From<super::ProposalSubspansItem> for ProposalSubspansItem {
        fn from(value: super::ProposalSubspansItem) -> Self {
            Self {
                first_sentence_index: Ok(value.first_sentence_index),
                sentence_count: Ok(value.sentence_count),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct WindowAnswer {
        failure: ::std::result::Result<
            ::std::option::Option<super::WindowAnswerFailure>,
            ::std::string::String,
        >,
        proposals: ::std::result::Result<::std::vec::Vec<super::Proposal>, ::std::string::String>,
        status: ::std::result::Result<super::WindowAnswerStatus, ::std::string::String>,
        window_index: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for WindowAnswer {
        fn default() -> Self {
            Self {
                failure: Ok(Default::default()),
                proposals: Err("no value supplied for proposals".to_string()),
                status: Err("no value supplied for status".to_string()),
                window_index: Err("no value supplied for window_index".to_string()),
            }
        }
    }
    impl WindowAnswer {
        pub fn failure<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::WindowAnswerFailure>>,
            T::Error: ::std::fmt::Display,
        {
            self.failure = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for failure: {e}"));
            self
        }
        pub fn proposals<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Proposal>>,
            T::Error: ::std::fmt::Display,
        {
            self.proposals = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for proposals: {e}"));
            self
        }
        pub fn status<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::WindowAnswerStatus>,
            T::Error: ::std::fmt::Display,
        {
            self.status = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for status: {e}"));
            self
        }
        pub fn window_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.window_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for window_index: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<WindowAnswer> for super::WindowAnswer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: WindowAnswer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                failure: value.failure?,
                proposals: value.proposals?,
                status: value.status?,
                window_index: value.window_index?,
            })
        }
    }
    impl ::std::convert::From<super::WindowAnswer> for WindowAnswer {
        fn from(value: super::WindowAnswer) -> Self {
            Self {
                failure: Ok(value.failure),
                proposals: Ok(value.proposals),
                status: Ok(value.status),
                window_index: Ok(value.window_index),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct WindowAnswerFailure {
        detail: ::std::result::Result<super::WindowAnswerFailureDetail, ::std::string::String>,
        failure_class:
            ::std::result::Result<super::WindowAnswerFailureFailureClass, ::std::string::String>,
    }
    impl ::std::default::Default for WindowAnswerFailure {
        fn default() -> Self {
            Self {
                detail: Err("no value supplied for detail".to_string()),
                failure_class: Err("no value supplied for failure_class".to_string()),
            }
        }
    }
    impl WindowAnswerFailure {
        pub fn detail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::WindowAnswerFailureDetail>,
            T::Error: ::std::fmt::Display,
        {
            self.detail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detail: {e}"));
            self
        }
        pub fn failure_class<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::WindowAnswerFailureFailureClass>,
            T::Error: ::std::fmt::Display,
        {
            self.failure_class = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for failure_class: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<WindowAnswerFailure> for super::WindowAnswerFailure {
        type Error = super::error::ConversionError;
        fn try_from(
            value: WindowAnswerFailure,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                detail: value.detail?,
                failure_class: value.failure_class?,
            })
        }
    }
    impl ::std::convert::From<super::WindowAnswerFailure> for WindowAnswerFailure {
        fn from(value: super::WindowAnswerFailure) -> Self {
            Self {
                detail: Ok(value.detail),
                failure_class: Ok(value.failure_class),
            }
        }
    }
}
