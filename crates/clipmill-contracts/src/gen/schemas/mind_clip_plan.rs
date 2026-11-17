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
#[doc = "`Join`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"after\","]
#[doc = "    \"kind\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"after\": {"]
#[doc = "      \"description\": \"The part this join follows.\","]
#[doc = "      \"$ref\": \"#/$defs/part_id\""]
#[doc = "    },"]
#[doc = "    \"crossfade_ms\": {"]
#[doc = "      \"description\": \"A short audio blend that hides a click or breath at the join.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 500.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"framing\": {"]
#[doc = "      \"description\": \"Change of framing at the join, which makes a jump cut on one camera look intended.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"keep\","]
#[doc = "        \"punch_in\","]
#[doc = "        \"punch_out\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"description\": \"`adjacent`: the next part continues in the source. `reorder`: the order differs from the source, as in a cold open. `distant`: the next part comes from elsewhere in the recording, which needs a link.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"adjacent\","]
#[doc = "        \"reorder\","]
#[doc = "        \"distant\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"link\": {"]
#[doc = "      \"description\": \"The Recording Memory link that justifies a distant join, such as `question_answer:s0102>s0981`.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 200,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"room_tone\": {"]
#[doc = "      \"description\": \"Fill a gap at the join with the recording's own background sound.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Join {
    #[doc = "The part this join follows."]
    pub after: PartId,
    #[doc = "A short audio blend that hides a click or breath at the join."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub crossfade_ms: ::std::option::Option<i64>,
    #[doc = "Change of framing at the join, which makes a jump cut on one camera look intended."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub framing: ::std::option::Option<JoinFraming>,
    #[doc = "`adjacent`: the next part continues in the source. `reorder`: the order differs from the source, as in a cold open. `distant`: the next part comes from elsewhere in the recording, which needs a link."]
    pub kind: JoinKind,
    #[doc = "The Recording Memory link that justifies a distant join, such as `question_answer:s0102>s0981`."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub link: ::std::option::Option<JoinLink>,
    #[doc = "Fill a gap at the join with the recording's own background sound."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub room_tone: ::std::option::Option<bool>,
}
impl Join {
    pub fn builder() -> builder::Join {
        Default::default()
    }
}
#[doc = "Change of framing at the join, which makes a jump cut on one camera look intended."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Change of framing at the join, which makes a jump cut on one camera look intended.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"keep\","]
#[doc = "    \"punch_in\","]
#[doc = "    \"punch_out\""]
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
pub enum JoinFraming {
    #[serde(rename = "keep")]
    Keep,
    #[serde(rename = "punch_in")]
    PunchIn,
    #[serde(rename = "punch_out")]
    PunchOut,
}
impl ::std::fmt::Display for JoinFraming {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Keep => f.write_str("keep"),
            Self::PunchIn => f.write_str("punch_in"),
            Self::PunchOut => f.write_str("punch_out"),
        }
    }
}
impl ::std::str::FromStr for JoinFraming {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "keep" => Ok(Self::Keep),
            "punch_in" => Ok(Self::PunchIn),
            "punch_out" => Ok(Self::PunchOut),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for JoinFraming {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JoinFraming {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JoinFraming {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`adjacent`: the next part continues in the source. `reorder`: the order differs from the source, as in a cold open. `distant`: the next part comes from elsewhere in the recording, which needs a link."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"`adjacent`: the next part continues in the source. `reorder`: the order differs from the source, as in a cold open. `distant`: the next part comes from elsewhere in the recording, which needs a link.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"adjacent\","]
#[doc = "    \"reorder\","]
#[doc = "    \"distant\""]
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
pub enum JoinKind {
    #[serde(rename = "adjacent")]
    Adjacent,
    #[serde(rename = "reorder")]
    Reorder,
    #[serde(rename = "distant")]
    Distant,
}
impl ::std::fmt::Display for JoinKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Adjacent => f.write_str("adjacent"),
            Self::Reorder => f.write_str("reorder"),
            Self::Distant => f.write_str("distant"),
        }
    }
}
impl ::std::str::FromStr for JoinKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "adjacent" => Ok(Self::Adjacent),
            "reorder" => Ok(Self::Reorder),
            "distant" => Ok(Self::Distant),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for JoinKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JoinKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JoinKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "The Recording Memory link that justifies a distant join, such as `question_answer:s0102>s0981`."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The Recording Memory link that justifies a distant join, such as `question_answer:s0102>s0981`.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct JoinLink(::std::string::String);
impl ::std::ops::Deref for JoinLink {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<JoinLink> for ::std::string::String {
    fn from(value: JoinLink) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for JoinLink {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for JoinLink {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for JoinLink {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for JoinLink {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for JoinLink {
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
#[doc = "One clip as an editor plans it: an ordered list of parts taken from anywhere in one recording, each with the role it plays, the ranges cut out of it, and the joins between them (decision R76). A continuous span is a plan of one part. The plan is a proposal: code checks every time, join and form rule before it becomes an edit document, and a critic checks that no join changes what was meant. Times are integer ticks at 1/90000 s of the source."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.mind.clip_plan.v1.json\","]
#[doc = "  \"title\": \"MindClipPlan\","]
#[doc = "  \"description\": \"One clip as an editor plans it: an ordered list of parts taken from anywhere in one recording, each with the role it plays, the ranges cut out of it, and the joins between them (decision R76). A continuous span is a plan of one part. The plan is a proposal: code checks every time, join and form rule before it becomes an edit document, and a critic checks that no join changes what was meant. Times are integer ticks at 1/90000 s of the source.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"duration_ticks\","]
#[doc = "    \"form\","]
#[doc = "    \"joins\","]
#[doc = "    \"parts\","]
#[doc = "    \"plan_id\","]
#[doc = "    \"provenance\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"doubts\": {"]
#[doc = "      \"description\": \"What the planner is unsure of, in plain words, shown with the clip.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"maxLength\": 400,"]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"duration_ticks\": {"]
#[doc = "      \"description\": \"Programme length: the parts' lengths less what was removed from them.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"evidence\": {"]
#[doc = "      \"description\": \"What justifies the plan: Recording Memory event and link ids, unit ids.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"maxLength\": 200,"]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"form\": {"]
#[doc = "      \"description\": \"The shape of the edit. `straight`: one part, nothing removed. `tightened`: one moment with ranges cut out of it, parts in source order. `stitch`: parts from distant places joined by evidence. `cold_open`: a line lifted from later opens the clip. `theme_montage`: several short moments on one topic or feeling.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"straight\","]
#[doc = "        \"tightened\","]
#[doc = "        \"stitch\","]
#[doc = "        \"cold_open\","]
#[doc = "        \"theme_montage\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"hook_text\": {"]
#[doc = "      \"description\": \"The opening line or on-screen text the plan is built around, as spoken or written; never invented speech.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 160,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"joins\": {"]
#[doc = "      \"description\": \"One per pair of consecutive parts, in programme order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/join\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 11"]
#[doc = "    },"]
#[doc = "    \"parts\": {"]
#[doc = "      \"description\": \"In programme order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/part\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 12,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"plan_id\": {"]
#[doc = "      \"description\": \"Derived from the plan's content, so the same plan always has the same id.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^plan_[0-9a-f]{16,64}$\""]
#[doc = "    },"]
#[doc = "    \"provenance\": {"]
#[doc = "      \"$ref\": \"#/$defs/provenance\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.mind.clip_plan.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 120,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindClipPlan {
    #[doc = "What the planner is unsure of, in plain words, shown with the clip."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub doubts: ::std::vec::Vec<MindClipPlanDoubtsItem>,
    #[doc = "Programme length: the parts' lengths less what was removed from them."]
    pub duration_ticks: ::std::num::NonZeroU64,
    #[doc = "What justifies the plan: Recording Memory event and link ids, unit ids."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub evidence: ::std::vec::Vec<MindClipPlanEvidenceItem>,
    #[doc = "The shape of the edit. `straight`: one part, nothing removed. `tightened`: one moment with ranges cut out of it, parts in source order. `stitch`: parts from distant places joined by evidence. `cold_open`: a line lifted from later opens the clip. `theme_montage`: several short moments on one topic or feeling."]
    pub form: MindClipPlanForm,
    #[doc = "The opening line or on-screen text the plan is built around, as spoken or written; never invented speech."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub hook_text: ::std::option::Option<MindClipPlanHookText>,
    #[doc = "One per pair of consecutive parts, in programme order."]
    pub joins: ::std::vec::Vec<Join>,
    #[doc = "In programme order."]
    pub parts: ::std::vec::Vec<Part>,
    #[doc = "Derived from the plan's content, so the same plan always has the same id."]
    pub plan_id: MindClipPlanPlanId,
    pub provenance: Provenance,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub title: ::std::option::Option<MindClipPlanTitle>,
}
impl MindClipPlan {
    pub fn builder() -> builder::MindClipPlan {
        Default::default()
    }
}
#[doc = "`MindClipPlanDoubtsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 400,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindClipPlanDoubtsItem(::std::string::String);
impl ::std::ops::Deref for MindClipPlanDoubtsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindClipPlanDoubtsItem> for ::std::string::String {
    fn from(value: MindClipPlanDoubtsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindClipPlanDoubtsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 400usize {
            return Err("longer than 400 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindClipPlanDoubtsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindClipPlanDoubtsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindClipPlanDoubtsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindClipPlanDoubtsItem {
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
#[doc = "`MindClipPlanEvidenceItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindClipPlanEvidenceItem(::std::string::String);
impl ::std::ops::Deref for MindClipPlanEvidenceItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindClipPlanEvidenceItem> for ::std::string::String {
    fn from(value: MindClipPlanEvidenceItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindClipPlanEvidenceItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindClipPlanEvidenceItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindClipPlanEvidenceItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindClipPlanEvidenceItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindClipPlanEvidenceItem {
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
#[doc = "The shape of the edit. `straight`: one part, nothing removed. `tightened`: one moment with ranges cut out of it, parts in source order. `stitch`: parts from distant places joined by evidence. `cold_open`: a line lifted from later opens the clip. `theme_montage`: several short moments on one topic or feeling."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The shape of the edit. `straight`: one part, nothing removed. `tightened`: one moment with ranges cut out of it, parts in source order. `stitch`: parts from distant places joined by evidence. `cold_open`: a line lifted from later opens the clip. `theme_montage`: several short moments on one topic or feeling.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"straight\","]
#[doc = "    \"tightened\","]
#[doc = "    \"stitch\","]
#[doc = "    \"cold_open\","]
#[doc = "    \"theme_montage\""]
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
pub enum MindClipPlanForm {
    #[serde(rename = "straight")]
    Straight,
    #[serde(rename = "tightened")]
    Tightened,
    #[serde(rename = "stitch")]
    Stitch,
    #[serde(rename = "cold_open")]
    ColdOpen,
    #[serde(rename = "theme_montage")]
    ThemeMontage,
}
impl ::std::fmt::Display for MindClipPlanForm {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Straight => f.write_str("straight"),
            Self::Tightened => f.write_str("tightened"),
            Self::Stitch => f.write_str("stitch"),
            Self::ColdOpen => f.write_str("cold_open"),
            Self::ThemeMontage => f.write_str("theme_montage"),
        }
    }
}
impl ::std::str::FromStr for MindClipPlanForm {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "straight" => Ok(Self::Straight),
            "tightened" => Ok(Self::Tightened),
            "stitch" => Ok(Self::Stitch),
            "cold_open" => Ok(Self::ColdOpen),
            "theme_montage" => Ok(Self::ThemeMontage),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindClipPlanForm {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindClipPlanForm {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindClipPlanForm {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "The opening line or on-screen text the plan is built around, as spoken or written; never invented speech."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The opening line or on-screen text the plan is built around, as spoken or written; never invented speech.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 160,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindClipPlanHookText(::std::string::String);
impl ::std::ops::Deref for MindClipPlanHookText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindClipPlanHookText> for ::std::string::String {
    fn from(value: MindClipPlanHookText) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindClipPlanHookText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 160usize {
            return Err("longer than 160 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindClipPlanHookText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindClipPlanHookText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindClipPlanHookText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindClipPlanHookText {
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
#[doc = "Derived from the plan's content, so the same plan always has the same id."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Derived from the plan's content, so the same plan always has the same id.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^plan_[0-9a-f]{16,64}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindClipPlanPlanId(::std::string::String);
impl ::std::ops::Deref for MindClipPlanPlanId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindClipPlanPlanId> for ::std::string::String {
    fn from(value: MindClipPlanPlanId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindClipPlanPlanId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^plan_[0-9a-f]{16,64}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^plan_[0-9a-f]{16,64}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindClipPlanPlanId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindClipPlanPlanId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindClipPlanPlanId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindClipPlanPlanId {
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
#[doc = "`MindClipPlanTitle`"]
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
pub struct MindClipPlanTitle(::std::string::String);
impl ::std::ops::Deref for MindClipPlanTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindClipPlanTitle> for ::std::string::String {
    fn from(value: MindClipPlanTitle) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindClipPlanTitle {
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
impl ::std::convert::TryFrom<&str> for MindClipPlanTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindClipPlanTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindClipPlanTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindClipPlanTitle {
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
#[doc = "`Part`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"part_id\","]
#[doc = "    \"role\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"part_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/part_id\""]
#[doc = "    },"]
#[doc = "    \"removed\": {"]
#[doc = "      \"description\": \"Ranges inside this part that the clip leaves out, so a person can see and restore each one.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/removed_range\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"role\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"hook\","]
#[doc = "        \"setup\","]
#[doc = "        \"question\","]
#[doc = "        \"answer\","]
#[doc = "        \"body\","]
#[doc = "        \"payoff\","]
#[doc = "        \"proof\","]
#[doc = "        \"reaction\","]
#[doc = "        \"item\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"units\": {"]
#[doc = "      \"description\": \"The Timeline Index units (sentences or shots) the part covers.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"maxLength\": 64,"]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"why\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 400,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub end_ticks: ::std::num::NonZeroU64,
    pub part_id: PartId,
    #[doc = "Ranges inside this part that the clip leaves out, so a person can see and restore each one."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub removed: ::std::vec::Vec<RemovedRange>,
    pub role: PartRole,
    pub start_ticks: u64,
    #[doc = "The Timeline Index units (sentences or shots) the part covers."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub units: ::std::vec::Vec<PartUnitsItem>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub why: ::std::option::Option<PartWhy>,
}
impl Part {
    pub fn builder() -> builder::Part {
        Default::default()
    }
}
#[doc = "`PartId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^p[0-9]{1,2}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PartId(::std::string::String);
impl ::std::ops::Deref for PartId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PartId> for ::std::string::String {
    fn from(value: PartId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for PartId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^p[0-9]{1,2}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^p[0-9]{1,2}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PartId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PartId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PartId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PartId {
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
#[doc = "`PartRole`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"hook\","]
#[doc = "    \"setup\","]
#[doc = "    \"question\","]
#[doc = "    \"answer\","]
#[doc = "    \"body\","]
#[doc = "    \"payoff\","]
#[doc = "    \"proof\","]
#[doc = "    \"reaction\","]
#[doc = "    \"item\""]
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
pub enum PartRole {
    #[serde(rename = "hook")]
    Hook,
    #[serde(rename = "setup")]
    Setup,
    #[serde(rename = "question")]
    Question,
    #[serde(rename = "answer")]
    Answer,
    #[serde(rename = "body")]
    Body,
    #[serde(rename = "payoff")]
    Payoff,
    #[serde(rename = "proof")]
    Proof,
    #[serde(rename = "reaction")]
    Reaction,
    #[serde(rename = "item")]
    Item,
}
impl ::std::fmt::Display for PartRole {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Hook => f.write_str("hook"),
            Self::Setup => f.write_str("setup"),
            Self::Question => f.write_str("question"),
            Self::Answer => f.write_str("answer"),
            Self::Body => f.write_str("body"),
            Self::Payoff => f.write_str("payoff"),
            Self::Proof => f.write_str("proof"),
            Self::Reaction => f.write_str("reaction"),
            Self::Item => f.write_str("item"),
        }
    }
}
impl ::std::str::FromStr for PartRole {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "hook" => Ok(Self::Hook),
            "setup" => Ok(Self::Setup),
            "question" => Ok(Self::Question),
            "answer" => Ok(Self::Answer),
            "body" => Ok(Self::Body),
            "payoff" => Ok(Self::Payoff),
            "proof" => Ok(Self::Proof),
            "reaction" => Ok(Self::Reaction),
            "item" => Ok(Self::Item),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for PartRole {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PartRole {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PartRole {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`PartUnitsItem`"]
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
pub struct PartUnitsItem(::std::string::String);
impl ::std::ops::Deref for PartUnitsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PartUnitsItem> for ::std::string::String {
    fn from(value: PartUnitsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for PartUnitsItem {
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
impl ::std::convert::TryFrom<&str> for PartUnitsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PartUnitsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PartUnitsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PartUnitsItem {
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
#[doc = "`PartWhy`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 400,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PartWhy(::std::string::String);
impl ::std::ops::Deref for PartWhy {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PartWhy> for ::std::string::String {
    fn from(value: PartWhy) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for PartWhy {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 400usize {
            return Err("longer than 400 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PartWhy {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PartWhy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PartWhy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PartWhy {
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
#[doc = "`Provenance`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"by\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"by\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"mind\","]
#[doc = "        \"person\","]
#[doc = "        \"agent\","]
#[doc = "        \"recipe\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"client_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 64,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"models\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"maxLength\": 200,"]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"prompts\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"maxLength\": 200,"]
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
pub struct Provenance {
    pub by: ProvenanceBy,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub client_id: ::std::option::Option<ProvenanceClientId>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub models: ::std::vec::Vec<ProvenanceModelsItem>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub prompts: ::std::vec::Vec<ProvenancePromptsItem>,
}
impl Provenance {
    pub fn builder() -> builder::Provenance {
        Default::default()
    }
}
#[doc = "`ProvenanceBy`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"mind\","]
#[doc = "    \"person\","]
#[doc = "    \"agent\","]
#[doc = "    \"recipe\""]
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
pub enum ProvenanceBy {
    #[serde(rename = "mind")]
    Mind,
    #[serde(rename = "person")]
    Person,
    #[serde(rename = "agent")]
    Agent,
    #[serde(rename = "recipe")]
    Recipe,
}
impl ::std::fmt::Display for ProvenanceBy {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Mind => f.write_str("mind"),
            Self::Person => f.write_str("person"),
            Self::Agent => f.write_str("agent"),
            Self::Recipe => f.write_str("recipe"),
        }
    }
}
impl ::std::str::FromStr for ProvenanceBy {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "mind" => Ok(Self::Mind),
            "person" => Ok(Self::Person),
            "agent" => Ok(Self::Agent),
            "recipe" => Ok(Self::Recipe),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ProvenanceBy {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProvenanceBy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProvenanceBy {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`ProvenanceClientId`"]
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
pub struct ProvenanceClientId(::std::string::String);
impl ::std::ops::Deref for ProvenanceClientId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProvenanceClientId> for ::std::string::String {
    fn from(value: ProvenanceClientId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProvenanceClientId {
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
impl ::std::convert::TryFrom<&str> for ProvenanceClientId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProvenanceClientId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProvenanceClientId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProvenanceClientId {
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
#[doc = "`ProvenanceModelsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProvenanceModelsItem(::std::string::String);
impl ::std::ops::Deref for ProvenanceModelsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProvenanceModelsItem> for ::std::string::String {
    fn from(value: ProvenanceModelsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProvenanceModelsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProvenanceModelsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProvenanceModelsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProvenanceModelsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProvenanceModelsItem {
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
#[doc = "`ProvenancePromptsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ProvenancePromptsItem(::std::string::String);
impl ::std::ops::Deref for ProvenancePromptsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ProvenancePromptsItem> for ::std::string::String {
    fn from(value: ProvenancePromptsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ProvenancePromptsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ProvenancePromptsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ProvenancePromptsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ProvenancePromptsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ProvenancePromptsItem {
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
#[doc = "`RemovedRange`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"reason\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"note\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 200,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"tangent\","]
#[doc = "        \"false_start\","]
#[doc = "        \"repetition\","]
#[doc = "        \"filler\","]
#[doc = "        \"slow\","]
#[doc = "        \"sponsor\","]
#[doc = "        \"other\""]
#[doc = "      ]"]
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
pub struct RemovedRange {
    pub end_ticks: ::std::num::NonZeroU64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub note: ::std::option::Option<RemovedRangeNote>,
    pub reason: RemovedRangeReason,
    pub start_ticks: u64,
}
impl RemovedRange {
    pub fn builder() -> builder::RemovedRange {
        Default::default()
    }
}
#[doc = "`RemovedRangeNote`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RemovedRangeNote(::std::string::String);
impl ::std::ops::Deref for RemovedRangeNote {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RemovedRangeNote> for ::std::string::String {
    fn from(value: RemovedRangeNote) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RemovedRangeNote {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RemovedRangeNote {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RemovedRangeNote {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RemovedRangeNote {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RemovedRangeNote {
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
#[doc = "`RemovedRangeReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"tangent\","]
#[doc = "    \"false_start\","]
#[doc = "    \"repetition\","]
#[doc = "    \"filler\","]
#[doc = "    \"slow\","]
#[doc = "    \"sponsor\","]
#[doc = "    \"other\""]
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
pub enum RemovedRangeReason {
    #[serde(rename = "tangent")]
    Tangent,
    #[serde(rename = "false_start")]
    FalseStart,
    #[serde(rename = "repetition")]
    Repetition,
    #[serde(rename = "filler")]
    Filler,
    #[serde(rename = "slow")]
    Slow,
    #[serde(rename = "sponsor")]
    Sponsor,
    #[serde(rename = "other")]
    Other,
}
impl ::std::fmt::Display for RemovedRangeReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Tangent => f.write_str("tangent"),
            Self::FalseStart => f.write_str("false_start"),
            Self::Repetition => f.write_str("repetition"),
            Self::Filler => f.write_str("filler"),
            Self::Slow => f.write_str("slow"),
            Self::Sponsor => f.write_str("sponsor"),
            Self::Other => f.write_str("other"),
        }
    }
}
impl ::std::str::FromStr for RemovedRangeReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "tangent" => Ok(Self::Tangent),
            "false_start" => Ok(Self::FalseStart),
            "repetition" => Ok(Self::Repetition),
            "filler" => Ok(Self::Filler),
            "slow" => Ok(Self::Slow),
            "sponsor" => Ok(Self::Sponsor),
            "other" => Ok(Self::Other),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for RemovedRangeReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RemovedRangeReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RemovedRangeReason {
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
    pub struct Join {
        after: ::std::result::Result<super::PartId, ::std::string::String>,
        crossfade_ms: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        framing:
            ::std::result::Result<::std::option::Option<super::JoinFraming>, ::std::string::String>,
        kind: ::std::result::Result<super::JoinKind, ::std::string::String>,
        link: ::std::result::Result<::std::option::Option<super::JoinLink>, ::std::string::String>,
        room_tone: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
    }
    impl ::std::default::Default for Join {
        fn default() -> Self {
            Self {
                after: Err("no value supplied for after".to_string()),
                crossfade_ms: Ok(Default::default()),
                framing: Ok(Default::default()),
                kind: Err("no value supplied for kind".to_string()),
                link: Ok(Default::default()),
                room_tone: Ok(Default::default()),
            }
        }
    }
    impl Join {
        pub fn after<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PartId>,
            T::Error: ::std::fmt::Display,
        {
            self.after = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for after: {e}"));
            self
        }
        pub fn crossfade_ms<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.crossfade_ms = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for crossfade_ms: {e}"));
            self
        }
        pub fn framing<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::JoinFraming>>,
            T::Error: ::std::fmt::Display,
        {
            self.framing = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for framing: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::JoinKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn link<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::JoinLink>>,
            T::Error: ::std::fmt::Display,
        {
            self.link = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for link: {e}"));
            self
        }
        pub fn room_tone<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.room_tone = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for room_tone: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Join> for super::Join {
        type Error = super::error::ConversionError;
        fn try_from(value: Join) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                after: value.after?,
                crossfade_ms: value.crossfade_ms?,
                framing: value.framing?,
                kind: value.kind?,
                link: value.link?,
                room_tone: value.room_tone?,
            })
        }
    }
    impl ::std::convert::From<super::Join> for Join {
        fn from(value: super::Join) -> Self {
            Self {
                after: Ok(value.after),
                crossfade_ms: Ok(value.crossfade_ms),
                framing: Ok(value.framing),
                kind: Ok(value.kind),
                link: Ok(value.link),
                room_tone: Ok(value.room_tone),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindClipPlan {
        doubts: ::std::result::Result<
            ::std::vec::Vec<super::MindClipPlanDoubtsItem>,
            ::std::string::String,
        >,
        duration_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        evidence: ::std::result::Result<
            ::std::vec::Vec<super::MindClipPlanEvidenceItem>,
            ::std::string::String,
        >,
        form: ::std::result::Result<super::MindClipPlanForm, ::std::string::String>,
        hook_text: ::std::result::Result<
            ::std::option::Option<super::MindClipPlanHookText>,
            ::std::string::String,
        >,
        joins: ::std::result::Result<::std::vec::Vec<super::Join>, ::std::string::String>,
        parts: ::std::result::Result<::std::vec::Vec<super::Part>, ::std::string::String>,
        plan_id: ::std::result::Result<super::MindClipPlanPlanId, ::std::string::String>,
        provenance: ::std::result::Result<super::Provenance, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        title: ::std::result::Result<
            ::std::option::Option<super::MindClipPlanTitle>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for MindClipPlan {
        fn default() -> Self {
            Self {
                doubts: Ok(Default::default()),
                duration_ticks: Err("no value supplied for duration_ticks".to_string()),
                evidence: Ok(Default::default()),
                form: Err("no value supplied for form".to_string()),
                hook_text: Ok(Default::default()),
                joins: Err("no value supplied for joins".to_string()),
                parts: Err("no value supplied for parts".to_string()),
                plan_id: Err("no value supplied for plan_id".to_string()),
                provenance: Err("no value supplied for provenance".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                title: Ok(Default::default()),
            }
        }
    }
    impl MindClipPlan {
        pub fn doubts<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindClipPlanDoubtsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.doubts = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for doubts: {e}"));
            self
        }
        pub fn duration_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.duration_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for duration_ticks: {e}"));
            self
        }
        pub fn evidence<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindClipPlanEvidenceItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.evidence = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for evidence: {e}"));
            self
        }
        pub fn form<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindClipPlanForm>,
            T::Error: ::std::fmt::Display,
        {
            self.form = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for form: {e}"));
            self
        }
        pub fn hook_text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::MindClipPlanHookText>>,
            T::Error: ::std::fmt::Display,
        {
            self.hook_text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hook_text: {e}"));
            self
        }
        pub fn joins<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Join>>,
            T::Error: ::std::fmt::Display,
        {
            self.joins = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for joins: {e}"));
            self
        }
        pub fn parts<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Part>>,
            T::Error: ::std::fmt::Display,
        {
            self.parts = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for parts: {e}"));
            self
        }
        pub fn plan_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindClipPlanPlanId>,
            T::Error: ::std::fmt::Display,
        {
            self.plan_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for plan_id: {e}"));
            self
        }
        pub fn provenance<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Provenance>,
            T::Error: ::std::fmt::Display,
        {
            self.provenance = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for provenance: {e}"));
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
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::MindClipPlanTitle>>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindClipPlan> for super::MindClipPlan {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindClipPlan,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                doubts: value.doubts?,
                duration_ticks: value.duration_ticks?,
                evidence: value.evidence?,
                form: value.form?,
                hook_text: value.hook_text?,
                joins: value.joins?,
                parts: value.parts?,
                plan_id: value.plan_id?,
                provenance: value.provenance?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                title: value.title?,
            })
        }
    }
    impl ::std::convert::From<super::MindClipPlan> for MindClipPlan {
        fn from(value: super::MindClipPlan) -> Self {
            Self {
                doubts: Ok(value.doubts),
                duration_ticks: Ok(value.duration_ticks),
                evidence: Ok(value.evidence),
                form: Ok(value.form),
                hook_text: Ok(value.hook_text),
                joins: Ok(value.joins),
                parts: Ok(value.parts),
                plan_id: Ok(value.plan_id),
                provenance: Ok(value.provenance),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                title: Ok(value.title),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Part {
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        part_id: ::std::result::Result<super::PartId, ::std::string::String>,
        removed: ::std::result::Result<::std::vec::Vec<super::RemovedRange>, ::std::string::String>,
        role: ::std::result::Result<super::PartRole, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        units: ::std::result::Result<::std::vec::Vec<super::PartUnitsItem>, ::std::string::String>,
        why: ::std::result::Result<::std::option::Option<super::PartWhy>, ::std::string::String>,
    }
    impl ::std::default::Default for Part {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                part_id: Err("no value supplied for part_id".to_string()),
                removed: Ok(Default::default()),
                role: Err("no value supplied for role".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                units: Ok(Default::default()),
                why: Ok(Default::default()),
            }
        }
    }
    impl Part {
        pub fn end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_ticks: {e}"));
            self
        }
        pub fn part_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PartId>,
            T::Error: ::std::fmt::Display,
        {
            self.part_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for part_id: {e}"));
            self
        }
        pub fn removed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::RemovedRange>>,
            T::Error: ::std::fmt::Display,
        {
            self.removed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for removed: {e}"));
            self
        }
        pub fn role<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PartRole>,
            T::Error: ::std::fmt::Display,
        {
            self.role = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for role: {e}"));
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
        pub fn units<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::PartUnitsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.units = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for units: {e}"));
            self
        }
        pub fn why<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::PartWhy>>,
            T::Error: ::std::fmt::Display,
        {
            self.why = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for why: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Part> for super::Part {
        type Error = super::error::ConversionError;
        fn try_from(value: Part) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                part_id: value.part_id?,
                removed: value.removed?,
                role: value.role?,
                start_ticks: value.start_ticks?,
                units: value.units?,
                why: value.why?,
            })
        }
    }
    impl ::std::convert::From<super::Part> for Part {
        fn from(value: super::Part) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                part_id: Ok(value.part_id),
                removed: Ok(value.removed),
                role: Ok(value.role),
                start_ticks: Ok(value.start_ticks),
                units: Ok(value.units),
                why: Ok(value.why),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Provenance {
        by: ::std::result::Result<super::ProvenanceBy, ::std::string::String>,
        client_id: ::std::result::Result<
            ::std::option::Option<super::ProvenanceClientId>,
            ::std::string::String,
        >,
        models: ::std::result::Result<
            ::std::vec::Vec<super::ProvenanceModelsItem>,
            ::std::string::String,
        >,
        prompts: ::std::result::Result<
            ::std::vec::Vec<super::ProvenancePromptsItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Provenance {
        fn default() -> Self {
            Self {
                by: Err("no value supplied for by".to_string()),
                client_id: Ok(Default::default()),
                models: Ok(Default::default()),
                prompts: Ok(Default::default()),
            }
        }
    }
    impl Provenance {
        pub fn by<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ProvenanceBy>,
            T::Error: ::std::fmt::Display,
        {
            self.by = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for by: {e}"));
            self
        }
        pub fn client_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ProvenanceClientId>>,
            T::Error: ::std::fmt::Display,
        {
            self.client_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for client_id: {e}"));
            self
        }
        pub fn models<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ProvenanceModelsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.models = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for models: {e}"));
            self
        }
        pub fn prompts<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ProvenancePromptsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.prompts = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompts: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Provenance> for super::Provenance {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Provenance,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                by: value.by?,
                client_id: value.client_id?,
                models: value.models?,
                prompts: value.prompts?,
            })
        }
    }
    impl ::std::convert::From<super::Provenance> for Provenance {
        fn from(value: super::Provenance) -> Self {
            Self {
                by: Ok(value.by),
                client_id: Ok(value.client_id),
                models: Ok(value.models),
                prompts: Ok(value.prompts),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct RemovedRange {
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        note: ::std::result::Result<
            ::std::option::Option<super::RemovedRangeNote>,
            ::std::string::String,
        >,
        reason: ::std::result::Result<super::RemovedRangeReason, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for RemovedRange {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                note: Ok(Default::default()),
                reason: Err("no value supplied for reason".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl RemovedRange {
        pub fn end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for end_ticks: {e}"));
            self
        }
        pub fn note<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::RemovedRangeNote>>,
            T::Error: ::std::fmt::Display,
        {
            self.note = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for note: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RemovedRangeReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
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
    impl ::std::convert::TryFrom<RemovedRange> for super::RemovedRange {
        type Error = super::error::ConversionError;
        fn try_from(
            value: RemovedRange,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                note: value.note?,
                reason: value.reason?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::RemovedRange> for RemovedRange {
        fn from(value: super::RemovedRange) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                note: Ok(value.note),
                reason: Ok(value.reason),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
}
