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
#[doc = "`Chapter`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"chapter_id\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"first_unit\","]
#[doc = "    \"last_unit\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"summary\","]
#[doc = "    \"title\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"chapter_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/chapter_id\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"first_unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    },"]
#[doc = "    \"last_unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"summary\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 300,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 80,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub chapter_id: ChapterId,
    pub end_ticks: ::std::num::NonZeroU64,
    pub first_unit: Unit,
    pub last_unit: Unit,
    pub start_ticks: u64,
    pub summary: ChapterSummary,
    pub title: ChapterTitle,
}
impl Chapter {
    pub fn builder() -> builder::Chapter {
        Default::default()
    }
}
#[doc = "`ChapterId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^c[1-9][0-9]{0,4}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ChapterId(::std::string::String);
impl ::std::ops::Deref for ChapterId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ChapterId> for ::std::string::String {
    fn from(value: ChapterId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ChapterId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^c[1-9][0-9]{0,4}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^c[1-9][0-9]{0,4}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ChapterId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ChapterId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ChapterId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ChapterId {
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
#[doc = "`ChapterSummary`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 300,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ChapterSummary(::std::string::String);
impl ::std::ops::Deref for ChapterSummary {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ChapterSummary> for ::std::string::String {
    fn from(value: ChapterSummary) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ChapterSummary {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 300usize {
            return Err("longer than 300 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ChapterSummary {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ChapterSummary {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ChapterSummary {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ChapterSummary {
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
#[doc = "`ChapterTitle`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 80,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ChapterTitle(::std::string::String);
impl ::std::ops::Deref for ChapterTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ChapterTitle> for ::std::string::String {
    fn from(value: ChapterTitle) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ChapterTitle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 80usize {
            return Err("longer than 80 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ChapterTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ChapterTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ChapterTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ChapterTitle {
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
#[doc = "`Entity`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"aliases\","]
#[doc = "    \"entity_id\","]
#[doc = "    \"events\","]
#[doc = "    \"kind\","]
#[doc = "    \"name\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"aliases\": {"]
#[doc = "      \"description\": \"Other spellings the transcript uses, including recognition errors, so search finds them all.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"maxLength\": 80,"]
#[doc = "        \"minLength\": 1"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"entity_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/entity_id\""]
#[doc = "    },"]
#[doc = "    \"events\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/event_id\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"person\","]
#[doc = "        \"organization\","]
#[doc = "        \"product\","]
#[doc = "        \"place\","]
#[doc = "        \"work\","]
#[doc = "        \"concept\","]
#[doc = "        \"other\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 80,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    #[doc = "Other spellings the transcript uses, including recognition errors, so search finds them all."]
    pub aliases: ::std::vec::Vec<EntityAliasesItem>,
    pub entity_id: EntityId,
    pub events: ::std::vec::Vec<EventId>,
    pub kind: EntityKind,
    pub name: EntityName,
}
impl Entity {
    pub fn builder() -> builder::Entity {
        Default::default()
    }
}
#[doc = "`EntityAliasesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 80,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EntityAliasesItem(::std::string::String);
impl ::std::ops::Deref for EntityAliasesItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EntityAliasesItem> for ::std::string::String {
    fn from(value: EntityAliasesItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EntityAliasesItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 80usize {
            return Err("longer than 80 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EntityAliasesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EntityAliasesItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EntityAliasesItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EntityAliasesItem {
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
#[doc = "`EntityId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^e[1-9][0-9]{0,5}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EntityId(::std::string::String);
impl ::std::ops::Deref for EntityId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EntityId> for ::std::string::String {
    fn from(value: EntityId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EntityId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^e[1-9][0-9]{0,5}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^e[1-9][0-9]{0,5}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EntityId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EntityId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EntityId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EntityId {
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
#[doc = "`EntityKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"person\","]
#[doc = "    \"organization\","]
#[doc = "    \"product\","]
#[doc = "    \"place\","]
#[doc = "    \"work\","]
#[doc = "    \"concept\","]
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
pub enum EntityKind {
    #[serde(rename = "person")]
    Person,
    #[serde(rename = "organization")]
    Organization,
    #[serde(rename = "product")]
    Product,
    #[serde(rename = "place")]
    Place,
    #[serde(rename = "work")]
    Work,
    #[serde(rename = "concept")]
    Concept,
    #[serde(rename = "other")]
    Other,
}
impl ::std::fmt::Display for EntityKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Person => f.write_str("person"),
            Self::Organization => f.write_str("organization"),
            Self::Product => f.write_str("product"),
            Self::Place => f.write_str("place"),
            Self::Work => f.write_str("work"),
            Self::Concept => f.write_str("concept"),
            Self::Other => f.write_str("other"),
        }
    }
}
impl ::std::str::FromStr for EntityKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "person" => Ok(Self::Person),
            "organization" => Ok(Self::Organization),
            "product" => Ok(Self::Product),
            "place" => Ok(Self::Place),
            "work" => Ok(Self::Work),
            "concept" => Ok(Self::Concept),
            "other" => Ok(Self::Other),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EntityKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EntityKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EntityKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`EntityName`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 80,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EntityName(::std::string::String);
impl ::std::ops::Deref for EntityName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EntityName> for ::std::string::String {
    fn from(value: EntityName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EntityName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 80usize {
            return Err("longer than 80 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EntityName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EntityName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EntityName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EntityName {
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
#[doc = "`Event`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"entities\","]
#[doc = "    \"event_id\","]
#[doc = "    \"first_unit\","]
#[doc = "    \"kind\","]
#[doc = "    \"last_unit\","]
#[doc = "    \"mood\","]
#[doc = "    \"speakers\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"title\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"chapter_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/chapter_id\""]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"entities\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/entity_id\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"event_id\": {"]
#[doc = "      \"$ref\": \"#/$defs/event_id\""]
#[doc = "    },"]
#[doc = "    \"first_unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"story\","]
#[doc = "        \"claim\","]
#[doc = "        \"insight\","]
#[doc = "        \"joke\","]
#[doc = "        \"question\","]
#[doc = "        \"answer\","]
#[doc = "        \"explanation\","]
#[doc = "        \"demo_step\","]
#[doc = "        \"reveal\","]
#[doc = "        \"hot_take\","]
#[doc = "        \"reaction\","]
#[doc = "        \"promise\","]
#[doc = "        \"other\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"last_unit\": {"]
#[doc = "      \"$ref\": \"#/$defs/unit\""]
#[doc = "    },"]
#[doc = "    \"mood\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"funny\","]
#[doc = "        \"serious\","]
#[doc = "        \"tense\","]
#[doc = "        \"emotional\","]
#[doc = "        \"inspiring\","]
#[doc = "        \"surprising\","]
#[doc = "        \"neutral\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"speakers\": {"]
#[doc = "      \"description\": \"Who speaks in it, by script letter, from the index rather than the model.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/speaker\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 100,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Event {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub chapter_id: ::std::option::Option<ChapterId>,
    pub end_ticks: ::std::num::NonZeroU64,
    pub entities: ::std::vec::Vec<EntityId>,
    pub event_id: EventId,
    pub first_unit: Unit,
    pub kind: EventKind,
    pub last_unit: Unit,
    pub mood: EventMood,
    #[doc = "Who speaks in it, by script letter, from the index rather than the model."]
    pub speakers: ::std::vec::Vec<Speaker>,
    pub start_ticks: u64,
    pub title: EventTitle,
}
impl Event {
    pub fn builder() -> builder::Event {
        Default::default()
    }
}
#[doc = "`EventId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^m[1-9][0-9]{0,5}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EventId(::std::string::String);
impl ::std::ops::Deref for EventId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EventId> for ::std::string::String {
    fn from(value: EventId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EventId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^m[1-9][0-9]{0,5}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^m[1-9][0-9]{0,5}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EventId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EventId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EventId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EventId {
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
#[doc = "`EventKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"story\","]
#[doc = "    \"claim\","]
#[doc = "    \"insight\","]
#[doc = "    \"joke\","]
#[doc = "    \"question\","]
#[doc = "    \"answer\","]
#[doc = "    \"explanation\","]
#[doc = "    \"demo_step\","]
#[doc = "    \"reveal\","]
#[doc = "    \"hot_take\","]
#[doc = "    \"reaction\","]
#[doc = "    \"promise\","]
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
pub enum EventKind {
    #[serde(rename = "story")]
    Story,
    #[serde(rename = "claim")]
    Claim,
    #[serde(rename = "insight")]
    Insight,
    #[serde(rename = "joke")]
    Joke,
    #[serde(rename = "question")]
    Question,
    #[serde(rename = "answer")]
    Answer,
    #[serde(rename = "explanation")]
    Explanation,
    #[serde(rename = "demo_step")]
    DemoStep,
    #[serde(rename = "reveal")]
    Reveal,
    #[serde(rename = "hot_take")]
    HotTake,
    #[serde(rename = "reaction")]
    Reaction,
    #[serde(rename = "promise")]
    Promise,
    #[serde(rename = "other")]
    Other,
}
impl ::std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Story => f.write_str("story"),
            Self::Claim => f.write_str("claim"),
            Self::Insight => f.write_str("insight"),
            Self::Joke => f.write_str("joke"),
            Self::Question => f.write_str("question"),
            Self::Answer => f.write_str("answer"),
            Self::Explanation => f.write_str("explanation"),
            Self::DemoStep => f.write_str("demo_step"),
            Self::Reveal => f.write_str("reveal"),
            Self::HotTake => f.write_str("hot_take"),
            Self::Reaction => f.write_str("reaction"),
            Self::Promise => f.write_str("promise"),
            Self::Other => f.write_str("other"),
        }
    }
}
impl ::std::str::FromStr for EventKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "story" => Ok(Self::Story),
            "claim" => Ok(Self::Claim),
            "insight" => Ok(Self::Insight),
            "joke" => Ok(Self::Joke),
            "question" => Ok(Self::Question),
            "answer" => Ok(Self::Answer),
            "explanation" => Ok(Self::Explanation),
            "demo_step" => Ok(Self::DemoStep),
            "reveal" => Ok(Self::Reveal),
            "hot_take" => Ok(Self::HotTake),
            "reaction" => Ok(Self::Reaction),
            "promise" => Ok(Self::Promise),
            "other" => Ok(Self::Other),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EventKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EventKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EventKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`EventMood`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"funny\","]
#[doc = "    \"serious\","]
#[doc = "    \"tense\","]
#[doc = "    \"emotional\","]
#[doc = "    \"inspiring\","]
#[doc = "    \"surprising\","]
#[doc = "    \"neutral\""]
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
pub enum EventMood {
    #[serde(rename = "funny")]
    Funny,
    #[serde(rename = "serious")]
    Serious,
    #[serde(rename = "tense")]
    Tense,
    #[serde(rename = "emotional")]
    Emotional,
    #[serde(rename = "inspiring")]
    Inspiring,
    #[serde(rename = "surprising")]
    Surprising,
    #[serde(rename = "neutral")]
    Neutral,
}
impl ::std::fmt::Display for EventMood {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Funny => f.write_str("funny"),
            Self::Serious => f.write_str("serious"),
            Self::Tense => f.write_str("tense"),
            Self::Emotional => f.write_str("emotional"),
            Self::Inspiring => f.write_str("inspiring"),
            Self::Surprising => f.write_str("surprising"),
            Self::Neutral => f.write_str("neutral"),
        }
    }
}
impl ::std::str::FromStr for EventMood {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "funny" => Ok(Self::Funny),
            "serious" => Ok(Self::Serious),
            "tense" => Ok(Self::Tense),
            "emotional" => Ok(Self::Emotional),
            "inspiring" => Ok(Self::Inspiring),
            "surprising" => Ok(Self::Surprising),
            "neutral" => Ok(Self::Neutral),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for EventMood {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EventMood {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EventMood {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`EventTitle`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 100,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct EventTitle(::std::string::String);
impl ::std::ops::Deref for EventTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<EventTitle> for ::std::string::String {
    fn from(value: EventTitle) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for EventTitle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for EventTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for EventTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for EventTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for EventTitle {
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
#[doc = "`Link`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"from_event\","]
#[doc = "    \"kind\","]
#[doc = "    \"link_id\","]
#[doc = "    \"note\","]
#[doc = "    \"to_event\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"from_event\": {"]
#[doc = "      \"$ref\": \"#/$defs/event_id\""]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"description\": \"`question_answer`, `setup_payoff`, `callback`, `claim_evidence` and `promise_delivery` run forward in time; `contradiction` may run either way.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"question_answer\","]
#[doc = "        \"setup_payoff\","]
#[doc = "        \"callback\","]
#[doc = "        \"claim_evidence\","]
#[doc = "        \"promise_delivery\","]
#[doc = "        \"contradiction\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"link_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^l[1-9][0-9]{0,5}$\""]
#[doc = "    },"]
#[doc = "    \"note\": {"]
#[doc = "      \"description\": \"What connects them, in a sentence.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 200,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"to_event\": {"]
#[doc = "      \"$ref\": \"#/$defs/event_id\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub from_event: EventId,
    #[doc = "`question_answer`, `setup_payoff`, `callback`, `claim_evidence` and `promise_delivery` run forward in time; `contradiction` may run either way."]
    pub kind: LinkKind,
    pub link_id: LinkLinkId,
    #[doc = "What connects them, in a sentence."]
    pub note: LinkNote,
    pub to_event: EventId,
}
impl Link {
    pub fn builder() -> builder::Link {
        Default::default()
    }
}
#[doc = "`question_answer`, `setup_payoff`, `callback`, `claim_evidence` and `promise_delivery` run forward in time; `contradiction` may run either way."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"`question_answer`, `setup_payoff`, `callback`, `claim_evidence` and `promise_delivery` run forward in time; `contradiction` may run either way.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"question_answer\","]
#[doc = "    \"setup_payoff\","]
#[doc = "    \"callback\","]
#[doc = "    \"claim_evidence\","]
#[doc = "    \"promise_delivery\","]
#[doc = "    \"contradiction\""]
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
pub enum LinkKind {
    #[serde(rename = "question_answer")]
    QuestionAnswer,
    #[serde(rename = "setup_payoff")]
    SetupPayoff,
    #[serde(rename = "callback")]
    Callback,
    #[serde(rename = "claim_evidence")]
    ClaimEvidence,
    #[serde(rename = "promise_delivery")]
    PromiseDelivery,
    #[serde(rename = "contradiction")]
    Contradiction,
}
impl ::std::fmt::Display for LinkKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::QuestionAnswer => f.write_str("question_answer"),
            Self::SetupPayoff => f.write_str("setup_payoff"),
            Self::Callback => f.write_str("callback"),
            Self::ClaimEvidence => f.write_str("claim_evidence"),
            Self::PromiseDelivery => f.write_str("promise_delivery"),
            Self::Contradiction => f.write_str("contradiction"),
        }
    }
}
impl ::std::str::FromStr for LinkKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "question_answer" => Ok(Self::QuestionAnswer),
            "setup_payoff" => Ok(Self::SetupPayoff),
            "callback" => Ok(Self::Callback),
            "claim_evidence" => Ok(Self::ClaimEvidence),
            "promise_delivery" => Ok(Self::PromiseDelivery),
            "contradiction" => Ok(Self::Contradiction),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for LinkKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for LinkKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for LinkKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`LinkLinkId`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^l[1-9][0-9]{0,5}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct LinkLinkId(::std::string::String);
impl ::std::ops::Deref for LinkLinkId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<LinkLinkId> for ::std::string::String {
    fn from(value: LinkLinkId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for LinkLinkId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^l[1-9][0-9]{0,5}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^l[1-9][0-9]{0,5}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for LinkLinkId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for LinkLinkId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for LinkLinkId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for LinkLinkId {
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
#[doc = "What connects them, in a sentence."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What connects them, in a sentence.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct LinkNote(::std::string::String);
impl ::std::ops::Deref for LinkNote {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<LinkNote> for ::std::string::String {
    fn from(value: LinkNote) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for LinkNote {
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
impl ::std::convert::TryFrom<&str> for LinkNote {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for LinkNote {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for LinkNote {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for LinkNote {
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
#[doc = "What a recording is about, read once by the editorial model so every later decision can look it up (architecture §4.3, tier 2): chapters, the events in them (a story, a claim, a joke, a question, its answer), the people and things named, and the links between distant events that make a stitched clip possible (a question and its answer, a setup and its payoff, a callback). The model reads the annotated script a window at a time for events and entities, then reads the list of events for chapters and links; it cites sentences (`s12`) and events (`m3`) only from what it was shown, and code assigns every id and time. Items that fail a check are kept under `dropped` with the reason, so a thin memory can be told from a thin recording. Times are integer ticks at 1/90000 s of the source."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.mind.memory.v1.json\","]
#[doc = "  \"title\": \"MindMemory\","]
#[doc = "  \"description\": \"What a recording is about, read once by the editorial model so every later decision can look it up (architecture §4.3, tier 2): chapters, the events in them (a story, a claim, a joke, a question, its answer), the people and things named, and the links between distant events that make a stitched clip possible (a question and its answer, a setup and its payoff, a callback). The model reads the annotated script a window at a time for events and entities, then reads the list of events for chapters and links; it cites sentences (`s12`) and events (`m3`) only from what it was shown, and code assigns every id and time. Items that fail a check are kept under `dropped` with the reason, so a thin memory can be told from a thin recording. Times are integer ticks at 1/90000 s of the source.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"chapters\","]
#[doc = "    \"dropped\","]
#[doc = "    \"entities\","]
#[doc = "    \"events\","]
#[doc = "    \"links\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"script\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"chapters\": {"]
#[doc = "      \"description\": \"In time order, without overlaps.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/chapter\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"dropped\": {"]
#[doc = "      \"description\": \"What the model wrote that did not pass a check, and why.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"code\","]
#[doc = "          \"item\","]
#[doc = "          \"message\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"code\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"pattern\": \"^[a-z][a-z0-9_]{0,63}$\""]
#[doc = "          },"]
#[doc = "          \"item\": {"]
#[doc = "            \"description\": \"The id the item would have had, or the window it came from.\","]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"maxLength\": 64,"]
#[doc = "            \"minLength\": 1"]
#[doc = "          },"]
#[doc = "          \"message\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"maxLength\": 400,"]
#[doc = "            \"minLength\": 1"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"entities\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/entity\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"events\": {"]
#[doc = "      \"description\": \"In time order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/event\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"links\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/link\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"implementation\","]
#[doc = "        \"model\","]
#[doc = "        \"prompt_versions\","]
#[doc = "        \"stage\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"implementation\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"model\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"prompt_versions\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          },"]
#[doc = "          \"minItems\": 1"]
#[doc = "        },"]
#[doc = "        \"stage\": {"]
#[doc = "          \"const\": \"mind-memory\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.mind.memory.v1\""]
#[doc = "    },"]
#[doc = "    \"script\": {"]
#[doc = "      \"description\": \"The annotated script the model read, by level and digest, so a memory can be traced to the exact text behind it.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"digest\","]
#[doc = "        \"level\","]
#[doc = "        \"windows\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"digest\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"level\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"full\","]
#[doc = "            \"compact\","]
#[doc = "            \"outline\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"windows\": {"]
#[doc = "          \"description\": \"How many windows the read pass took.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
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
pub struct MindMemory {
    #[doc = "In time order, without overlaps."]
    pub chapters: ::std::vec::Vec<Chapter>,
    #[doc = "What the model wrote that did not pass a check, and why."]
    pub dropped: ::std::vec::Vec<MindMemoryDroppedItem>,
    pub entities: ::std::vec::Vec<Entity>,
    #[doc = "In time order."]
    pub events: ::std::vec::Vec<Event>,
    pub links: ::std::vec::Vec<Link>,
    pub producer: MindMemoryProducer,
    pub schema_version: ::serde_json::Value,
    pub script: MindMemoryScript,
    pub source_fingerprint: Sha256,
}
impl MindMemory {
    pub fn builder() -> builder::MindMemory {
        Default::default()
    }
}
#[doc = "`MindMemoryDroppedItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"item\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[a-z][a-z0-9_]{0,63}$\""]
#[doc = "    },"]
#[doc = "    \"item\": {"]
#[doc = "      \"description\": \"The id the item would have had, or the window it came from.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 64,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
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
pub struct MindMemoryDroppedItem {
    pub code: MindMemoryDroppedItemCode,
    #[doc = "The id the item would have had, or the window it came from."]
    pub item: MindMemoryDroppedItemItem,
    pub message: MindMemoryDroppedItemMessage,
}
impl MindMemoryDroppedItem {
    pub fn builder() -> builder::MindMemoryDroppedItem {
        Default::default()
    }
}
#[doc = "`MindMemoryDroppedItemCode`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[a-z][a-z0-9_]{0,63}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindMemoryDroppedItemCode(::std::string::String);
impl ::std::ops::Deref for MindMemoryDroppedItemCode {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindMemoryDroppedItemCode> for ::std::string::String {
    fn from(value: MindMemoryDroppedItemCode) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindMemoryDroppedItemCode {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[a-z][a-z0-9_]{0,63}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[a-z][a-z0-9_]{0,63}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindMemoryDroppedItemCode {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindMemoryDroppedItemCode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindMemoryDroppedItemCode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindMemoryDroppedItemCode {
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
#[doc = "The id the item would have had, or the window it came from."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The id the item would have had, or the window it came from.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 64,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindMemoryDroppedItemItem(::std::string::String);
impl ::std::ops::Deref for MindMemoryDroppedItemItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindMemoryDroppedItemItem> for ::std::string::String {
    fn from(value: MindMemoryDroppedItemItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindMemoryDroppedItemItem {
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
impl ::std::convert::TryFrom<&str> for MindMemoryDroppedItemItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindMemoryDroppedItemItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindMemoryDroppedItemItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindMemoryDroppedItemItem {
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
#[doc = "`MindMemoryDroppedItemMessage`"]
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
pub struct MindMemoryDroppedItemMessage(::std::string::String);
impl ::std::ops::Deref for MindMemoryDroppedItemMessage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindMemoryDroppedItemMessage> for ::std::string::String {
    fn from(value: MindMemoryDroppedItemMessage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindMemoryDroppedItemMessage {
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
impl ::std::convert::TryFrom<&str> for MindMemoryDroppedItemMessage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindMemoryDroppedItemMessage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindMemoryDroppedItemMessage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindMemoryDroppedItemMessage {
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
#[doc = "`MindMemoryProducer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"implementation\","]
#[doc = "    \"model\","]
#[doc = "    \"prompt_versions\","]
#[doc = "    \"stage\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"implementation\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"model\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"prompt_versions\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"minLength\": 1"]
#[doc = "      },"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"stage\": {"]
#[doc = "      \"const\": \"mind-memory\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindMemoryProducer {
    pub implementation: MindMemoryProducerImplementation,
    pub model: MindMemoryProducerModel,
    pub prompt_versions: ::std::vec::Vec<MindMemoryProducerPromptVersionsItem>,
    pub stage: ::serde_json::Value,
}
impl MindMemoryProducer {
    pub fn builder() -> builder::MindMemoryProducer {
        Default::default()
    }
}
#[doc = "`MindMemoryProducerImplementation`"]
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
pub struct MindMemoryProducerImplementation(::std::string::String);
impl ::std::ops::Deref for MindMemoryProducerImplementation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindMemoryProducerImplementation> for ::std::string::String {
    fn from(value: MindMemoryProducerImplementation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindMemoryProducerImplementation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindMemoryProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindMemoryProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindMemoryProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindMemoryProducerImplementation {
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
#[doc = "`MindMemoryProducerModel`"]
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
pub struct MindMemoryProducerModel(::std::string::String);
impl ::std::ops::Deref for MindMemoryProducerModel {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindMemoryProducerModel> for ::std::string::String {
    fn from(value: MindMemoryProducerModel) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindMemoryProducerModel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindMemoryProducerModel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindMemoryProducerModel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindMemoryProducerModel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindMemoryProducerModel {
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
#[doc = "`MindMemoryProducerPromptVersionsItem`"]
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
pub struct MindMemoryProducerPromptVersionsItem(::std::string::String);
impl ::std::ops::Deref for MindMemoryProducerPromptVersionsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindMemoryProducerPromptVersionsItem> for ::std::string::String {
    fn from(value: MindMemoryProducerPromptVersionsItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindMemoryProducerPromptVersionsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindMemoryProducerPromptVersionsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindMemoryProducerPromptVersionsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindMemoryProducerPromptVersionsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindMemoryProducerPromptVersionsItem {
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
#[doc = "The annotated script the model read, by level and digest, so a memory can be traced to the exact text behind it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The annotated script the model read, by level and digest, so a memory can be traced to the exact text behind it.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"digest\","]
#[doc = "    \"level\","]
#[doc = "    \"windows\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"digest\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"level\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"full\","]
#[doc = "        \"compact\","]
#[doc = "        \"outline\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"windows\": {"]
#[doc = "      \"description\": \"How many windows the read pass took.\","]
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
pub struct MindMemoryScript {
    pub digest: Sha256,
    pub level: MindMemoryScriptLevel,
    #[doc = "How many windows the read pass took."]
    pub windows: ::std::num::NonZeroU64,
}
impl MindMemoryScript {
    pub fn builder() -> builder::MindMemoryScript {
        Default::default()
    }
}
#[doc = "`MindMemoryScriptLevel`"]
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
pub enum MindMemoryScriptLevel {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "compact")]
    Compact,
    #[serde(rename = "outline")]
    Outline,
}
impl ::std::fmt::Display for MindMemoryScriptLevel {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Full => f.write_str("full"),
            Self::Compact => f.write_str("compact"),
            Self::Outline => f.write_str("outline"),
        }
    }
}
impl ::std::str::FromStr for MindMemoryScriptLevel {
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
impl ::std::convert::TryFrom<&str> for MindMemoryScriptLevel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindMemoryScriptLevel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindMemoryScriptLevel {
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
#[doc = "The script's letter for a voice."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The script's letter for a voice.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[A-Z][0-9]{0,3}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Speaker(::std::string::String);
impl ::std::ops::Deref for Speaker {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Speaker> for ::std::string::String {
    fn from(value: Speaker) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Speaker {
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
impl ::std::convert::TryFrom<&str> for Speaker {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Speaker {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Speaker {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Speaker {
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
#[doc = "A sentence of the Timeline Index, as the script cites it."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A sentence of the Timeline Index, as the script cites it.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^s(0|[1-9][0-9]{0,8})$\""]
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
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^s(0|[1-9][0-9]{0,8})$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^s(0|[1-9][0-9]{0,8})$\"".into());
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
    pub struct Chapter {
        chapter_id: ::std::result::Result<super::ChapterId, ::std::string::String>,
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        first_unit: ::std::result::Result<super::Unit, ::std::string::String>,
        last_unit: ::std::result::Result<super::Unit, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        summary: ::std::result::Result<super::ChapterSummary, ::std::string::String>,
        title: ::std::result::Result<super::ChapterTitle, ::std::string::String>,
    }
    impl ::std::default::Default for Chapter {
        fn default() -> Self {
            Self {
                chapter_id: Err("no value supplied for chapter_id".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                first_unit: Err("no value supplied for first_unit".to_string()),
                last_unit: Err("no value supplied for last_unit".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                summary: Err("no value supplied for summary".to_string()),
                title: Err("no value supplied for title".to_string()),
            }
        }
    }
    impl Chapter {
        pub fn chapter_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ChapterId>,
            T::Error: ::std::fmt::Display,
        {
            self.chapter_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for chapter_id: {e}"));
            self
        }
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
        pub fn first_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.first_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_unit: {e}"));
            self
        }
        pub fn last_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.last_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for last_unit: {e}"));
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
        pub fn summary<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ChapterSummary>,
            T::Error: ::std::fmt::Display,
        {
            self.summary = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for summary: {e}"));
            self
        }
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ChapterTitle>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Chapter> for super::Chapter {
        type Error = super::error::ConversionError;
        fn try_from(value: Chapter) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                chapter_id: value.chapter_id?,
                end_ticks: value.end_ticks?,
                first_unit: value.first_unit?,
                last_unit: value.last_unit?,
                start_ticks: value.start_ticks?,
                summary: value.summary?,
                title: value.title?,
            })
        }
    }
    impl ::std::convert::From<super::Chapter> for Chapter {
        fn from(value: super::Chapter) -> Self {
            Self {
                chapter_id: Ok(value.chapter_id),
                end_ticks: Ok(value.end_ticks),
                first_unit: Ok(value.first_unit),
                last_unit: Ok(value.last_unit),
                start_ticks: Ok(value.start_ticks),
                summary: Ok(value.summary),
                title: Ok(value.title),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Entity {
        aliases:
            ::std::result::Result<::std::vec::Vec<super::EntityAliasesItem>, ::std::string::String>,
        entity_id: ::std::result::Result<super::EntityId, ::std::string::String>,
        events: ::std::result::Result<::std::vec::Vec<super::EventId>, ::std::string::String>,
        kind: ::std::result::Result<super::EntityKind, ::std::string::String>,
        name: ::std::result::Result<super::EntityName, ::std::string::String>,
    }
    impl ::std::default::Default for Entity {
        fn default() -> Self {
            Self {
                aliases: Err("no value supplied for aliases".to_string()),
                entity_id: Err("no value supplied for entity_id".to_string()),
                events: Err("no value supplied for events".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                name: Err("no value supplied for name".to_string()),
            }
        }
    }
    impl Entity {
        pub fn aliases<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::EntityAliasesItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.aliases = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for aliases: {e}"));
            self
        }
        pub fn entity_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EntityId>,
            T::Error: ::std::fmt::Display,
        {
            self.entity_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for entity_id: {e}"));
            self
        }
        pub fn events<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::EventId>>,
            T::Error: ::std::fmt::Display,
        {
            self.events = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for events: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EntityKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EntityName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Entity> for super::Entity {
        type Error = super::error::ConversionError;
        fn try_from(value: Entity) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                aliases: value.aliases?,
                entity_id: value.entity_id?,
                events: value.events?,
                kind: value.kind?,
                name: value.name?,
            })
        }
    }
    impl ::std::convert::From<super::Entity> for Entity {
        fn from(value: super::Entity) -> Self {
            Self {
                aliases: Ok(value.aliases),
                entity_id: Ok(value.entity_id),
                events: Ok(value.events),
                kind: Ok(value.kind),
                name: Ok(value.name),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Event {
        chapter_id:
            ::std::result::Result<::std::option::Option<super::ChapterId>, ::std::string::String>,
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        entities: ::std::result::Result<::std::vec::Vec<super::EntityId>, ::std::string::String>,
        event_id: ::std::result::Result<super::EventId, ::std::string::String>,
        first_unit: ::std::result::Result<super::Unit, ::std::string::String>,
        kind: ::std::result::Result<super::EventKind, ::std::string::String>,
        last_unit: ::std::result::Result<super::Unit, ::std::string::String>,
        mood: ::std::result::Result<super::EventMood, ::std::string::String>,
        speakers: ::std::result::Result<::std::vec::Vec<super::Speaker>, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        title: ::std::result::Result<super::EventTitle, ::std::string::String>,
    }
    impl ::std::default::Default for Event {
        fn default() -> Self {
            Self {
                chapter_id: Ok(Default::default()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                entities: Err("no value supplied for entities".to_string()),
                event_id: Err("no value supplied for event_id".to_string()),
                first_unit: Err("no value supplied for first_unit".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                last_unit: Err("no value supplied for last_unit".to_string()),
                mood: Err("no value supplied for mood".to_string()),
                speakers: Err("no value supplied for speakers".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                title: Err("no value supplied for title".to_string()),
            }
        }
    }
    impl Event {
        pub fn chapter_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ChapterId>>,
            T::Error: ::std::fmt::Display,
        {
            self.chapter_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for chapter_id: {e}"));
            self
        }
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
        pub fn entities<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::EntityId>>,
            T::Error: ::std::fmt::Display,
        {
            self.entities = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for entities: {e}"));
            self
        }
        pub fn event_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EventId>,
            T::Error: ::std::fmt::Display,
        {
            self.event_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for event_id: {e}"));
            self
        }
        pub fn first_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.first_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for first_unit: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EventKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn last_unit<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Unit>,
            T::Error: ::std::fmt::Display,
        {
            self.last_unit = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for last_unit: {e}"));
            self
        }
        pub fn mood<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EventMood>,
            T::Error: ::std::fmt::Display,
        {
            self.mood = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mood: {e}"));
            self
        }
        pub fn speakers<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Speaker>>,
            T::Error: ::std::fmt::Display,
        {
            self.speakers = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for speakers: {e}"));
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
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EventTitle>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Event> for super::Event {
        type Error = super::error::ConversionError;
        fn try_from(value: Event) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                chapter_id: value.chapter_id?,
                end_ticks: value.end_ticks?,
                entities: value.entities?,
                event_id: value.event_id?,
                first_unit: value.first_unit?,
                kind: value.kind?,
                last_unit: value.last_unit?,
                mood: value.mood?,
                speakers: value.speakers?,
                start_ticks: value.start_ticks?,
                title: value.title?,
            })
        }
    }
    impl ::std::convert::From<super::Event> for Event {
        fn from(value: super::Event) -> Self {
            Self {
                chapter_id: Ok(value.chapter_id),
                end_ticks: Ok(value.end_ticks),
                entities: Ok(value.entities),
                event_id: Ok(value.event_id),
                first_unit: Ok(value.first_unit),
                kind: Ok(value.kind),
                last_unit: Ok(value.last_unit),
                mood: Ok(value.mood),
                speakers: Ok(value.speakers),
                start_ticks: Ok(value.start_ticks),
                title: Ok(value.title),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Link {
        from_event: ::std::result::Result<super::EventId, ::std::string::String>,
        kind: ::std::result::Result<super::LinkKind, ::std::string::String>,
        link_id: ::std::result::Result<super::LinkLinkId, ::std::string::String>,
        note: ::std::result::Result<super::LinkNote, ::std::string::String>,
        to_event: ::std::result::Result<super::EventId, ::std::string::String>,
    }
    impl ::std::default::Default for Link {
        fn default() -> Self {
            Self {
                from_event: Err("no value supplied for from_event".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                link_id: Err("no value supplied for link_id".to_string()),
                note: Err("no value supplied for note".to_string()),
                to_event: Err("no value supplied for to_event".to_string()),
            }
        }
    }
    impl Link {
        pub fn from_event<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EventId>,
            T::Error: ::std::fmt::Display,
        {
            self.from_event = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for from_event: {e}"));
            self
        }
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::LinkKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn link_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::LinkLinkId>,
            T::Error: ::std::fmt::Display,
        {
            self.link_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for link_id: {e}"));
            self
        }
        pub fn note<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::LinkNote>,
            T::Error: ::std::fmt::Display,
        {
            self.note = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for note: {e}"));
            self
        }
        pub fn to_event<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::EventId>,
            T::Error: ::std::fmt::Display,
        {
            self.to_event = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for to_event: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Link> for super::Link {
        type Error = super::error::ConversionError;
        fn try_from(value: Link) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                from_event: value.from_event?,
                kind: value.kind?,
                link_id: value.link_id?,
                note: value.note?,
                to_event: value.to_event?,
            })
        }
    }
    impl ::std::convert::From<super::Link> for Link {
        fn from(value: super::Link) -> Self {
            Self {
                from_event: Ok(value.from_event),
                kind: Ok(value.kind),
                link_id: Ok(value.link_id),
                note: Ok(value.note),
                to_event: Ok(value.to_event),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindMemory {
        chapters: ::std::result::Result<::std::vec::Vec<super::Chapter>, ::std::string::String>,
        dropped: ::std::result::Result<
            ::std::vec::Vec<super::MindMemoryDroppedItem>,
            ::std::string::String,
        >,
        entities: ::std::result::Result<::std::vec::Vec<super::Entity>, ::std::string::String>,
        events: ::std::result::Result<::std::vec::Vec<super::Event>, ::std::string::String>,
        links: ::std::result::Result<::std::vec::Vec<super::Link>, ::std::string::String>,
        producer: ::std::result::Result<super::MindMemoryProducer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        script: ::std::result::Result<super::MindMemoryScript, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
    }
    impl ::std::default::Default for MindMemory {
        fn default() -> Self {
            Self {
                chapters: Err("no value supplied for chapters".to_string()),
                dropped: Err("no value supplied for dropped".to_string()),
                entities: Err("no value supplied for entities".to_string()),
                events: Err("no value supplied for events".to_string()),
                links: Err("no value supplied for links".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                script: Err("no value supplied for script".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
            }
        }
    }
    impl MindMemory {
        pub fn chapters<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Chapter>>,
            T::Error: ::std::fmt::Display,
        {
            self.chapters = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for chapters: {e}"));
            self
        }
        pub fn dropped<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindMemoryDroppedItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.dropped = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for dropped: {e}"));
            self
        }
        pub fn entities<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Entity>>,
            T::Error: ::std::fmt::Display,
        {
            self.entities = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for entities: {e}"));
            self
        }
        pub fn events<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Event>>,
            T::Error: ::std::fmt::Display,
        {
            self.events = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for events: {e}"));
            self
        }
        pub fn links<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Link>>,
            T::Error: ::std::fmt::Display,
        {
            self.links = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for links: {e}"));
            self
        }
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindMemoryProducer>,
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
        pub fn script<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindMemoryScript>,
            T::Error: ::std::fmt::Display,
        {
            self.script = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for script: {e}"));
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
    impl ::std::convert::TryFrom<MindMemory> for super::MindMemory {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindMemory,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                chapters: value.chapters?,
                dropped: value.dropped?,
                entities: value.entities?,
                events: value.events?,
                links: value.links?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                script: value.script?,
                source_fingerprint: value.source_fingerprint?,
            })
        }
    }
    impl ::std::convert::From<super::MindMemory> for MindMemory {
        fn from(value: super::MindMemory) -> Self {
            Self {
                chapters: Ok(value.chapters),
                dropped: Ok(value.dropped),
                entities: Ok(value.entities),
                events: Ok(value.events),
                links: Ok(value.links),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                script: Ok(value.script),
                source_fingerprint: Ok(value.source_fingerprint),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindMemoryDroppedItem {
        code: ::std::result::Result<super::MindMemoryDroppedItemCode, ::std::string::String>,
        item: ::std::result::Result<super::MindMemoryDroppedItemItem, ::std::string::String>,
        message: ::std::result::Result<super::MindMemoryDroppedItemMessage, ::std::string::String>,
    }
    impl ::std::default::Default for MindMemoryDroppedItem {
        fn default() -> Self {
            Self {
                code: Err("no value supplied for code".to_string()),
                item: Err("no value supplied for item".to_string()),
                message: Err("no value supplied for message".to_string()),
            }
        }
    }
    impl MindMemoryDroppedItem {
        pub fn code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindMemoryDroppedItemCode>,
            T::Error: ::std::fmt::Display,
        {
            self.code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for code: {e}"));
            self
        }
        pub fn item<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindMemoryDroppedItemItem>,
            T::Error: ::std::fmt::Display,
        {
            self.item = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for item: {e}"));
            self
        }
        pub fn message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindMemoryDroppedItemMessage>,
            T::Error: ::std::fmt::Display,
        {
            self.message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for message: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindMemoryDroppedItem> for super::MindMemoryDroppedItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindMemoryDroppedItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                code: value.code?,
                item: value.item?,
                message: value.message?,
            })
        }
    }
    impl ::std::convert::From<super::MindMemoryDroppedItem> for MindMemoryDroppedItem {
        fn from(value: super::MindMemoryDroppedItem) -> Self {
            Self {
                code: Ok(value.code),
                item: Ok(value.item),
                message: Ok(value.message),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindMemoryProducer {
        implementation:
            ::std::result::Result<super::MindMemoryProducerImplementation, ::std::string::String>,
        model: ::std::result::Result<super::MindMemoryProducerModel, ::std::string::String>,
        prompt_versions: ::std::result::Result<
            ::std::vec::Vec<super::MindMemoryProducerPromptVersionsItem>,
            ::std::string::String,
        >,
        stage: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for MindMemoryProducer {
        fn default() -> Self {
            Self {
                implementation: Err("no value supplied for implementation".to_string()),
                model: Err("no value supplied for model".to_string()),
                prompt_versions: Err("no value supplied for prompt_versions".to_string()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl MindMemoryProducer {
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindMemoryProducerImplementation>,
            T::Error: ::std::fmt::Display,
        {
            self.implementation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for implementation: {e}"));
            self
        }
        pub fn model<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindMemoryProducerModel>,
            T::Error: ::std::fmt::Display,
        {
            self.model = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model: {e}"));
            self
        }
        pub fn prompt_versions<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<super::MindMemoryProducerPromptVersionsItem>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_versions = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt_versions: {e}"));
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
    impl ::std::convert::TryFrom<MindMemoryProducer> for super::MindMemoryProducer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindMemoryProducer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                implementation: value.implementation?,
                model: value.model?,
                prompt_versions: value.prompt_versions?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::MindMemoryProducer> for MindMemoryProducer {
        fn from(value: super::MindMemoryProducer) -> Self {
            Self {
                implementation: Ok(value.implementation),
                model: Ok(value.model),
                prompt_versions: Ok(value.prompt_versions),
                stage: Ok(value.stage),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindMemoryScript {
        digest: ::std::result::Result<super::Sha256, ::std::string::String>,
        level: ::std::result::Result<super::MindMemoryScriptLevel, ::std::string::String>,
        windows: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for MindMemoryScript {
        fn default() -> Self {
            Self {
                digest: Err("no value supplied for digest".to_string()),
                level: Err("no value supplied for level".to_string()),
                windows: Err("no value supplied for windows".to_string()),
            }
        }
    }
    impl MindMemoryScript {
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
        pub fn level<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindMemoryScriptLevel>,
            T::Error: ::std::fmt::Display,
        {
            self.level = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for level: {e}"));
            self
        }
        pub fn windows<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.windows = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for windows: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindMemoryScript> for super::MindMemoryScript {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindMemoryScript,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                digest: value.digest?,
                level: value.level?,
                windows: value.windows?,
            })
        }
    }
    impl ::std::convert::From<super::MindMemoryScript> for MindMemoryScript {
        fn from(value: super::MindMemoryScript) -> Self {
            Self {
                digest: Ok(value.digest),
                level: Ok(value.level),
                windows: Ok(value.windows),
            }
        }
    }
}
