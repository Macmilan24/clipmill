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
#[doc = "`Case`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"expected\","]
#[doc = "    \"score\","]
#[doc = "    \"unit\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"expected\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"finished\","]
#[doc = "        \"continues\","]
#[doc = "        \"dangling\","]
#[doc = "        \"intact\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"$ref\": \"#/$defs/score\""]
#[doc = "    },"]
#[doc = "    \"unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub expected: CaseExpected,
    pub score: Score,
    pub unit: Unit,
}
impl Case {
    pub fn builder() -> builder::Case {
        Default::default()
    }
}
#[doc = "`CaseExpected`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"finished\","]
#[doc = "    \"continues\","]
#[doc = "    \"dangling\","]
#[doc = "    \"intact\""]
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
pub enum CaseExpected {
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "continues")]
    Continues,
    #[serde(rename = "dangling")]
    Dangling,
    #[serde(rename = "intact")]
    Intact,
}
impl ::std::fmt::Display for CaseExpected {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Finished => f.write_str("finished"),
            Self::Continues => f.write_str("continues"),
            Self::Dangling => f.write_str("dangling"),
            Self::Intact => f.write_str("intact"),
        }
    }
}
impl ::std::str::FromStr for CaseExpected {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "finished" => Ok(Self::Finished),
            "continues" => Ok(Self::Continues),
            "dangling" => Ok(Self::Dangling),
            "intact" => Ok(Self::Intact),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CaseExpected {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CaseExpected {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CaseExpected {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Check`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cases\","]
#[doc = "    \"passed\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cases\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/case\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"passed\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"separation\": {"]
#[doc = "      \"description\": \"The chance that a case of the first kind scores above a case of the second: 0.5 is chance, 1 is perfect.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub cases: ::std::vec::Vec<Case>,
    pub passed: bool,
    #[doc = "The chance that a case of the first kind scores above a case of the second: 0.5 is chance, 1 is perfect."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub separation: ::std::option::Option<f64>,
}
impl Check {
    pub fn builder() -> builder::Check {
        Default::default()
    }
}
#[doc = "`Ending`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"after_unit\","]
#[doc = "    \"score\","]
#[doc = "    \"verdict\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"after_unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    },"]
#[doc = "    \"score\": {"]
#[doc = "      \"$ref\": \"#/$defs/score\""]
#[doc = "    },"]
#[doc = "    \"verdict\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"finished\","]
#[doc = "        \"continues\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Ending {
    pub after_unit: Unit,
    pub score: Score,
    pub verdict: EndingVerdict,
}
impl Ending {
    pub fn builder() -> builder::Ending {
        Default::default()
    }
}
#[doc = "`EndingVerdict`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"finished\","]
#[doc = "    \"continues\""]
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
pub enum EndingVerdict {
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "continues")]
    Continues,
}
impl ::std::fmt::Display for EndingVerdict {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Finished => f.write_str("finished"),
            Self::Continues => f.write_str("continues"),
        }
    }
}
impl ::std::str::FromStr for EndingVerdict {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "finished" => Ok(Self::Finished),
            "continues" => Ok(Self::Continues),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EndingVerdict {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EndingVerdict {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EndingVerdict {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Narrow questions put to the editorial model before planning. A self-check first measures the model on cases whose answer is known from the recording itself; only a model that passes it decides whether a candidate clip's ending leaves the speaker mid-thought."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.mind.judgments.v1.json\","]
#[doc = "  \"title\": \"MindJudgments\","]
#[doc = "  \"description\": \"Narrow questions put to the editorial model before planning. A self-check first measures the model on cases whose answer is known from the recording itself; only a model that passes it decides whether a candidate clip's ending leaves the speaker mid-thought.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"endings\","]
#[doc = "    \"inputs\","]
#[doc = "    \"judge\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"self_check\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"endings\": {"]
#[doc = "      \"description\": \"Verdicts on candidate endings where the same speaker goes on. Empty when the ending self-check did not pass.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/ending\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"inputs\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"$ref\": \"#/$defs/sha256\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"judge\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"prompt_version\","]
#[doc = "        \"think_tokens\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"prompt_version\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"think_tokens\": {"]
#[doc = "          \"description\": \"How many tokens the model may reason for before it answers.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.mind.judgments.v1\""]
#[doc = "    },"]
#[doc = "    \"self_check\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"endings\","]
#[doc = "        \"removals\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"endings\": {"]
#[doc = "          \"$ref\": \"#/$defs/check\""]
#[doc = "        },"]
#[doc = "        \"removals\": {"]
#[doc = "          \"$ref\": \"#/$defs/check\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
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
pub struct MindJudgments {
    #[doc = "Verdicts on candidate endings where the same speaker goes on. Empty when the ending self-check did not pass."]
    pub endings: ::std::vec::Vec<Ending>,
    pub inputs: ::std::collections::HashMap<::std::string::String, Sha256>,
    pub judge: MindJudgmentsJudge,
    pub schema_version: ::serde_json::Value,
    pub self_check: MindJudgmentsSelfCheck,
    pub source_fingerprint: Sha256,
}
impl MindJudgments {
    pub fn builder() -> builder::MindJudgments {
        Default::default()
    }
}
#[doc = "`MindJudgmentsJudge`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"prompt_version\","]
#[doc = "    \"think_tokens\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"prompt_version\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"think_tokens\": {"]
#[doc = "      \"description\": \"How many tokens the model may reason for before it answers.\","]
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
pub struct MindJudgmentsJudge {
    pub prompt_version: MindJudgmentsJudgePromptVersion,
    #[doc = "How many tokens the model may reason for before it answers."]
    pub think_tokens: u64,
}
impl MindJudgmentsJudge {
    pub fn builder() -> builder::MindJudgmentsJudge {
        Default::default()
    }
}
#[doc = "`MindJudgmentsJudgePromptVersion`"]
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
pub struct MindJudgmentsJudgePromptVersion(::std::string::String);
impl ::std::ops::Deref for MindJudgmentsJudgePromptVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindJudgmentsJudgePromptVersion> for ::std::string::String {
    fn from(value: MindJudgmentsJudgePromptVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindJudgmentsJudgePromptVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindJudgmentsJudgePromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindJudgmentsJudgePromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindJudgmentsJudgePromptVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindJudgmentsJudgePromptVersion {
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
#[doc = "`MindJudgmentsSelfCheck`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"endings\","]
#[doc = "    \"removals\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"endings\": {"]
#[doc = "      \"$ref\": \"#/$defs/check\""]
#[doc = "    },"]
#[doc = "    \"removals\": {"]
#[doc = "      \"$ref\": \"#/$defs/check\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindJudgmentsSelfCheck {
    pub endings: Check,
    pub removals: Check,
}
impl MindJudgmentsSelfCheck {
    pub fn builder() -> builder::MindJudgmentsSelfCheck {
        Default::default()
    }
}
#[doc = "The model's probability for the first reading minus its probability for the opposite one, from -1 to 1."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The model's probability for the first reading minus its probability for the opposite one, from -1 to 1.\","]
#[doc = "  \"type\": \"number\","]
#[doc = "  \"maximum\": 1.0,"]
#[doc = "  \"minimum\": -1.0"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Score(pub f64);
impl ::std::ops::Deref for Score {
    type Target = f64;
    fn deref(&self) -> &f64 {
        &self.0
    }
}
impl ::std::convert::From<Score> for f64 {
    fn from(value: Score) -> Self {
        value.0
    }
}
impl ::std::convert::From<f64> for Score {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for Score {
    type Err = <f64 as ::std::str::FromStr>::Err;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl ::std::convert::TryFrom<&str> for Score {
    type Error = <f64 as ::std::str::FromStr>::Err;
    fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<String> for Score {
    type Error = <f64 as ::std::str::FromStr>::Err;
    fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::fmt::Display for Score {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
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
#[doc = "`Unit`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^s[0-9]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Unit(::std::string::String);
impl ::std::ops::Deref for Unit {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Unit> for ::std::string::String {
    fn from(value: Unit) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Unit {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^s[0-9]+$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^s[0-9]+$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for Unit {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Unit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Unit {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Unit {
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
    pub struct Case {
        expected: ::std::result::Result<super::CaseExpected, ::std::string::String>,
        score: ::std::result::Result<super::Score, ::std::string::String>,
        unit: ::std::result::Result<super::Unit, ::std::string::String>,
    }
    impl ::std::default::Default for Case {
        fn default() -> Self {
            Self {
                expected: Err("no value supplied for expected".to_string()),
                score: Err("no value supplied for score".to_string()),
                unit: Err("no value supplied for unit".to_string()),
            }
        }
    }
    impl Case {
        pub fn expected<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CaseExpected>,
            T::Error: ::std::fmt::Display,
        {
            self.expected = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for expected: {e}"));
            self
        }
        pub fn score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Score>,
            T::Error: ::std::fmt::Display,
        {
            self.score = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for score: {e}"));
            self
        }
        pub fn unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for unit: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Case> for super::Case {
        type Error = super::error::ConversionError;
        fn try_from(value: Case) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                expected: value.expected?,
                score: value.score?,
                unit: value.unit?,
            })
        }
    }
    impl ::std::convert::From<super::Case> for Case {
        fn from(value: super::Case) -> Self {
            Self {
                expected: Ok(value.expected),
                score: Ok(value.score),
                unit: Ok(value.unit),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Check {
        cases: ::std::result::Result<::std::vec::Vec<super::Case>, ::std::string::String>,
        passed: ::std::result::Result<bool, ::std::string::String>,
        separation: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for Check {
        fn default() -> Self {
            Self {
                cases: Err("no value supplied for cases".to_string()),
                passed: Err("no value supplied for passed".to_string()),
                separation: Ok(Default::default()),
            }
        }
    }
    impl Check {
        pub fn cases<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Case>>,
            T::Error: ::std::fmt::Display,
        {
            self.cases = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cases: {e}"));
            self
        }
        pub fn passed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.passed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for passed: {e}"));
            self
        }
        pub fn separation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.separation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for separation: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Check> for super::Check {
        type Error = super::error::ConversionError;
        fn try_from(value: Check) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cases: value.cases?,
                passed: value.passed?,
                separation: value.separation?,
            })
        }
    }
    impl ::std::convert::From<super::Check> for Check {
        fn from(value: super::Check) -> Self {
            Self {
                cases: Ok(value.cases),
                passed: Ok(value.passed),
                separation: Ok(value.separation),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Ending {
        after_unit: ::std::result::Result<super::Unit, ::std::string::String>,
        score: ::std::result::Result<super::Score, ::std::string::String>,
        verdict: ::std::result::Result<super::EndingVerdict, ::std::string::String>,
    }
    impl ::std::default::Default for Ending {
        fn default() -> Self {
            Self {
                after_unit: Err("no value supplied for after_unit".to_string()),
                score: Err("no value supplied for score".to_string()),
                verdict: Err("no value supplied for verdict".to_string()),
            }
        }
    }
    impl Ending {
        pub fn after_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.after_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for after_unit: {e}"));
            self
        }
        pub fn score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Score>,
            T::Error: ::std::fmt::Display,
        {
            self.score = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for score: {e}"));
            self
        }
        pub fn verdict<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EndingVerdict>,
            T::Error: ::std::fmt::Display,
        {
            self.verdict = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for verdict: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Ending> for super::Ending {
        type Error = super::error::ConversionError;
        fn try_from(value: Ending) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                after_unit: value.after_unit?,
                score: value.score?,
                verdict: value.verdict?,
            })
        }
    }
    impl ::std::convert::From<super::Ending> for Ending {
        fn from(value: super::Ending) -> Self {
            Self {
                after_unit: Ok(value.after_unit),
                score: Ok(value.score),
                verdict: Ok(value.verdict),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindJudgments {
        endings: ::std::result::Result<::std::vec::Vec<super::Ending>, ::std::string::String>,
        inputs: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, super::Sha256>,
            ::std::string::String,
        >,
        judge: ::std::result::Result<super::MindJudgmentsJudge, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        self_check: ::std::result::Result<super::MindJudgmentsSelfCheck, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for MindJudgments {
        fn default() -> Self {
            Self {
                endings: Err("no value supplied for endings".to_string()),
                inputs: Err("no value supplied for inputs".to_string()),
                judge: Err("no value supplied for judge".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                self_check: Err("no value supplied for self_check".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl MindJudgments {
        pub fn endings<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Ending>>,
            T::Error: ::std::fmt::Display,
        {
            self.endings = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for endings: {e}"));
            self
        }
        pub fn inputs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::collections::HashMap<::std::string::String, super::Sha256>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.inputs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inputs: {e}"));
            self
        }
        pub fn judge<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindJudgmentsJudge>,
            T::Error: ::std::fmt::Display,
        {
            self.judge = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for judge: {e}"));
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
        pub fn self_check<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindJudgmentsSelfCheck>,
            T::Error: ::std::fmt::Display,
        {
            self.self_check = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for self_check: {e}"));
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
    impl ::std::convert::TryFrom<MindJudgments> for super::MindJudgments {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindJudgments,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                endings: value.endings?,
                inputs: value.inputs?,
                judge: value.judge?,
                schema_version: value.schema_version?,
                self_check: value.self_check?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::MindJudgments> for MindJudgments {
        fn from(value: super::MindJudgments) -> Self {
            Self {
                endings: Ok(value.endings),
                inputs: Ok(value.inputs),
                judge: Ok(value.judge),
                schema_version: Ok(value.schema_version),
                self_check: Ok(value.self_check),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindJudgmentsJudge {
        prompt_version:
            ::std::result::Result<super::MindJudgmentsJudgePromptVersion, ::std::string::String>,
        think_tokens: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for MindJudgmentsJudge {
        fn default() -> Self {
            Self {
                prompt_version: Err("no value supplied for prompt_version".to_string()),
                think_tokens: Err("no value supplied for think_tokens".to_string()),
            }
        }
    }
    impl MindJudgmentsJudge {
        pub fn prompt_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindJudgmentsJudgePromptVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt_version: {e}"));
            self
        }
        pub fn think_tokens<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.think_tokens = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for think_tokens: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindJudgmentsJudge> for super::MindJudgmentsJudge {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindJudgmentsJudge,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                prompt_version: value.prompt_version?,
                think_tokens: value.think_tokens?,
            })
        }
    }
    impl ::std::convert::From<super::MindJudgmentsJudge> for MindJudgmentsJudge {
        fn from(value: super::MindJudgmentsJudge) -> Self {
            Self {
                prompt_version: Ok(value.prompt_version),
                think_tokens: Ok(value.think_tokens),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindJudgmentsSelfCheck {
        endings: ::std::result::Result<super::Check, ::std::string::String>,
        removals: ::std::result::Result<super::Check, ::std::string::String>,
    }
    impl ::std::default::Default for MindJudgmentsSelfCheck {
        fn default() -> Self {
            Self {
                endings: Err("no value supplied for endings".to_string()),
                removals: Err("no value supplied for removals".to_string()),
            }
        }
    }
    impl MindJudgmentsSelfCheck {
        pub fn endings<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Check>,
            T::Error: ::std::fmt::Display,
        {
            self.endings = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for endings: {e}"));
            self
        }
        pub fn removals<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Check>,
            T::Error: ::std::fmt::Display,
        {
            self.removals = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for removals: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindJudgmentsSelfCheck> for super::MindJudgmentsSelfCheck {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindJudgmentsSelfCheck,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                endings: value.endings?,
                removals: value.removals?,
            })
        }
    }
    impl ::std::convert::From<super::MindJudgmentsSelfCheck> for MindJudgmentsSelfCheck {
        fn from(value: super::MindJudgmentsSelfCheck) -> Self {
            Self {
                endings: Ok(value.endings),
                removals: Ok(value.removals),
            }
        }
    }
}
