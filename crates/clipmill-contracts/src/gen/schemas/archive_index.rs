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
#[doc = "`ArchiveEntry`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"bytes\","]
#[doc = "    \"kind\","]
#[doc = "    \"path\","]
#[doc = "    \"sha256\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bytes\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"description\": \"What the file is. A closed vocabulary rather than a free string: a reader deciding what to do with an entry needs to know what it holds, and an unknown kind in a future archive is a reason for that reader to stop — which it can only do if the vocabulary was stated.\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"state\","]
#[doc = "        \"edit_doc\","]
#[doc = "        \"command_log\","]
#[doc = "        \"render_manifest\","]
#[doc = "        \"decisions\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"path\": {"]
#[doc = "      \"description\": \"Location inside the archive: forward-slashed and relative. It may not begin with a separator and may not contain a parent-directory segment, so extracting an archive cannot write outside the directory it was extracted into. Stated here and enforced by the writer; the pattern only checks the first half of it, because a lookahead is not a construct every language's regex engine in this project supports.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1,"]
#[doc = "      \"pattern\": \"^[^/\\\\\\\\].*$\""]
#[doc = "    },"]
#[doc = "    \"sha256\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256_hex\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ArchiveEntry {
    pub bytes: u64,
    #[doc = "What the file is. A closed vocabulary rather than a free string: a reader deciding what to do with an entry needs to know what it holds, and an unknown kind in a future archive is a reason for that reader to stop — which it can only do if the vocabulary was stated."]
    pub kind: ArchiveEntryKind,
    #[doc = "Location inside the archive: forward-slashed and relative. It may not begin with a separator and may not contain a parent-directory segment, so extracting an archive cannot write outside the directory it was extracted into. Stated here and enforced by the writer; the pattern only checks the first half of it, because a lookahead is not a construct every language's regex engine in this project supports."]
    pub path: ArchiveEntryPath,
    pub sha256: Sha256Hex,
}
impl ArchiveEntry {
    pub fn builder() -> builder::ArchiveEntry {
        Default::default()
    }
}
#[doc = "What the file is. A closed vocabulary rather than a free string: a reader deciding what to do with an entry needs to know what it holds, and an unknown kind in a future archive is a reason for that reader to stop — which it can only do if the vocabulary was stated."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What the file is. A closed vocabulary rather than a free string: a reader deciding what to do with an entry needs to know what it holds, and an unknown kind in a future archive is a reason for that reader to stop — which it can only do if the vocabulary was stated.\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"state\","]
#[doc = "    \"edit_doc\","]
#[doc = "    \"command_log\","]
#[doc = "    \"render_manifest\","]
#[doc = "    \"decisions\""]
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
pub enum ArchiveEntryKind {
    #[serde(rename = "state")]
    State,
    #[serde(rename = "edit_doc")]
    EditDoc,
    #[serde(rename = "command_log")]
    CommandLog,
    #[serde(rename = "render_manifest")]
    RenderManifest,
    #[serde(rename = "decisions")]
    Decisions,
}
impl ::std::fmt::Display for ArchiveEntryKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::State => f.write_str("state"),
            Self::EditDoc => f.write_str("edit_doc"),
            Self::CommandLog => f.write_str("command_log"),
            Self::RenderManifest => f.write_str("render_manifest"),
            Self::Decisions => f.write_str("decisions"),
        }
    }
}
impl ::std::str::FromStr for ArchiveEntryKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "state" => Ok(Self::State),
            "edit_doc" => Ok(Self::EditDoc),
            "command_log" => Ok(Self::CommandLog),
            "render_manifest" => Ok(Self::RenderManifest),
            "decisions" => Ok(Self::Decisions),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ArchiveEntryKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArchiveEntryKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArchiveEntryKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Location inside the archive: forward-slashed and relative. It may not begin with a separator and may not contain a parent-directory segment, so extracting an archive cannot write outside the directory it was extracted into. Stated here and enforced by the writer; the pattern only checks the first half of it, because a lookahead is not a construct every language's regex engine in this project supports."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Location inside the archive: forward-slashed and relative. It may not begin with a separator and may not contain a parent-directory segment, so extracting an archive cannot write outside the directory it was extracted into. Stated here and enforced by the writer; the pattern only checks the first half of it, because a lookahead is not a construct every language's regex engine in this project supports.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1,"]
#[doc = "  \"pattern\": \"^[^/\\\\\\\\].*$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ArchiveEntryPath(::std::string::String);
impl ::std::ops::Deref for ArchiveEntryPath {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ArchiveEntryPath> for ::std::string::String {
    fn from(value: ArchiveEntryPath) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ArchiveEntryPath {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[^/\\\\].*$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[^/\\\\].*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ArchiveEntryPath {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArchiveEntryPath {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArchiveEntryPath {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ArchiveEntryPath {
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
#[doc = "What a project archive contains, and what it deliberately leaves out (book ch. 10). The archive is the promise that a user's work is not held hostage to the application that made it, so its contents are described here rather than by ClipMill's own code. Everything that cannot be regenerated travels inside the zip: project state, edit documents, the command logs that explain how each document reached its shape, the clip decisions a user recorded, and the render manifests that say what was delivered. Media does not travel — a project's sources are three orders of magnitude larger than its documents and already exist on the user's disk — but every source is named with the fingerprint that identifies it, so a re-import can say which file it is looking for rather than failing on a path that stopped being true."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.archive_index.v1.json\","]
#[doc = "  \"title\": \"ArchiveIndex\","]
#[doc = "  \"description\": \"What a project archive contains, and what it deliberately leaves out (book ch. 10). The archive is the promise that a user's work is not held hostage to the application that made it, so its contents are described here rather than by ClipMill's own code. Everything that cannot be regenerated travels inside the zip: project state, edit documents, the command logs that explain how each document reached its shape, the clip decisions a user recorded, and the render manifests that say what was delivered. Media does not travel — a project's sources are three orders of magnitude larger than its documents and already exist on the user's disk — but every source is named with the fingerprint that identifies it, so a re-import can say which file it is looking for rather than failing on a path that stopped being true.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"created_unix_millis\","]
#[doc = "    \"entries\","]
#[doc = "    \"project_id\","]
#[doc = "    \"project_name\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"sources\","]
#[doc = "    \"writer_version\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"created_unix_millis\": {"]
#[doc = "      \"description\": \"When the archive was written. Wall time is recorded here because an archive is project state rather than a derived artifact, and a person opening one a year later needs to tell two of them apart. It is the one field that stops two archives of an unchanged project being byte-identical.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"entries\": {"]
#[doc = "      \"description\": \"Every file inside the archive besides this index, sorted by path so two archives of the same project agree on order.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/archive_entry\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"project_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"project_name\": {"]
#[doc = "      \"description\": \"What the user called the project, for a human reading a folder of archives.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.archive_index.v1\""]
#[doc = "    },"]
#[doc = "    \"sources\": {"]
#[doc = "      \"description\": \"The recordings this project was built from. Named, never carried.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/archived_source\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"writer_version\": {"]
#[doc = "      \"description\": \"The application version that wrote the archive, so a reader can tell whether it predates the format it is reading.\","]
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
pub struct ArchiveIndex {
    #[doc = "When the archive was written. Wall time is recorded here because an archive is project state rather than a derived artifact, and a person opening one a year later needs to tell two of them apart. It is the one field that stops two archives of an unchanged project being byte-identical."]
    pub created_unix_millis: u64,
    #[doc = "Every file inside the archive besides this index, sorted by path so two archives of the same project agree on order."]
    pub entries: ::std::vec::Vec<ArchiveEntry>,
    pub project_id: ArchiveIndexProjectId,
    #[doc = "What the user called the project, for a human reading a folder of archives."]
    pub project_name: ::std::string::String,
    pub schema_version: ::serde_json::Value,
    #[doc = "The recordings this project was built from. Named, never carried."]
    pub sources: ::std::vec::Vec<ArchivedSource>,
    #[doc = "The application version that wrote the archive, so a reader can tell whether it predates the format it is reading."]
    pub writer_version: ArchiveIndexWriterVersion,
}
impl ArchiveIndex {
    pub fn builder() -> builder::ArchiveIndex {
        Default::default()
    }
}
#[doc = "`ArchiveIndexProjectId`"]
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
pub struct ArchiveIndexProjectId(::std::string::String);
impl ::std::ops::Deref for ArchiveIndexProjectId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ArchiveIndexProjectId> for ::std::string::String {
    fn from(value: ArchiveIndexProjectId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ArchiveIndexProjectId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ArchiveIndexProjectId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArchiveIndexProjectId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArchiveIndexProjectId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ArchiveIndexProjectId {
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
#[doc = "The application version that wrote the archive, so a reader can tell whether it predates the format it is reading."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The application version that wrote the archive, so a reader can tell whether it predates the format it is reading.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ArchiveIndexWriterVersion(::std::string::String);
impl ::std::ops::Deref for ArchiveIndexWriterVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ArchiveIndexWriterVersion> for ::std::string::String {
    fn from(value: ArchiveIndexWriterVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ArchiveIndexWriterVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ArchiveIndexWriterVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArchiveIndexWriterVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArchiveIndexWriterVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ArchiveIndexWriterVersion {
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
#[doc = "`ArchivedSource`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"display_name\","]
#[doc = "    \"fingerprint\","]
#[doc = "    \"source_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"display_name\": {"]
#[doc = "      \"description\": \"What the user called it, for a human going to look for it.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"fingerprint\": {"]
#[doc = "      \"description\": \"The content fingerprint of the recording, which is what identifies it if the file has moved or been renamed.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"source_id\": {"]
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
pub struct ArchivedSource {
    #[doc = "What the user called it, for a human going to look for it."]
    pub display_name: ::std::string::String,
    #[doc = "The content fingerprint of the recording, which is what identifies it if the file has moved or been renamed."]
    pub fingerprint: ArchivedSourceFingerprint,
    pub source_id: ArchivedSourceSourceId,
}
impl ArchivedSource {
    pub fn builder() -> builder::ArchivedSource {
        Default::default()
    }
}
#[doc = "The content fingerprint of the recording, which is what identifies it if the file has moved or been renamed."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"The content fingerprint of the recording, which is what identifies it if the file has moved or been renamed.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ArchivedSourceFingerprint(::std::string::String);
impl ::std::ops::Deref for ArchivedSourceFingerprint {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ArchivedSourceFingerprint> for ::std::string::String {
    fn from(value: ArchivedSourceFingerprint) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ArchivedSourceFingerprint {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ArchivedSourceFingerprint {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArchivedSourceFingerprint {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArchivedSourceFingerprint {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ArchivedSourceFingerprint {
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
#[doc = "`ArchivedSourceSourceId`"]
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
pub struct ArchivedSourceSourceId(::std::string::String);
impl ::std::ops::Deref for ArchivedSourceSourceId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ArchivedSourceSourceId> for ::std::string::String {
    fn from(value: ArchivedSourceSourceId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ArchivedSourceSourceId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ArchivedSourceSourceId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ArchivedSourceSourceId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ArchivedSourceSourceId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ArchivedSourceSourceId {
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
#[doc = "Lower-case hex, unprefixed, as written into the checksum files an archive can be verified with."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Lower-case hex, unprefixed, as written into the checksum files an archive can be verified with.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[0-9a-f]{64}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Sha256Hex(::std::string::String);
impl ::std::ops::Deref for Sha256Hex {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Sha256Hex> for ::std::string::String {
    fn from(value: Sha256Hex) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Sha256Hex {
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
impl ::std::convert::TryFrom<&str> for Sha256Hex {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Sha256Hex {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Sha256Hex {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Sha256Hex {
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
    pub struct ArchiveEntry {
        bytes: ::std::result::Result<u64, ::std::string::String>,
        kind: ::std::result::Result<super::ArchiveEntryKind, ::std::string::String>,
        path: ::std::result::Result<super::ArchiveEntryPath, ::std::string::String>,
        sha256: ::std::result::Result<super::Sha256Hex, ::std::string::String>,
    }
    impl ::std::default::Default for ArchiveEntry {
        fn default() -> Self {
            Self {
                bytes: Err("no value supplied for bytes".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                path: Err("no value supplied for path".to_string()),
                sha256: Err("no value supplied for sha256".to_string()),
            }
        }
    }
    impl ArchiveEntry {
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
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArchiveEntryKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn path<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArchiveEntryPath>,
            T::Error: ::std::fmt::Display,
        {
            self.path = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for path: {e}"));
            self
        }
        pub fn sha256<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256Hex>,
            T::Error: ::std::fmt::Display,
        {
            self.sha256 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sha256: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ArchiveEntry> for super::ArchiveEntry {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ArchiveEntry,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                bytes: value.bytes?,
                kind: value.kind?,
                path: value.path?,
                sha256: value.sha256?,
            })
        }
    }
    impl ::std::convert::From<super::ArchiveEntry> for ArchiveEntry {
        fn from(value: super::ArchiveEntry) -> Self {
            Self {
                bytes: Ok(value.bytes),
                kind: Ok(value.kind),
                path: Ok(value.path),
                sha256: Ok(value.sha256),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ArchiveIndex {
        created_unix_millis: ::std::result::Result<u64, ::std::string::String>,
        entries: ::std::result::Result<::std::vec::Vec<super::ArchiveEntry>, ::std::string::String>,
        project_id: ::std::result::Result<super::ArchiveIndexProjectId, ::std::string::String>,
        project_name: ::std::result::Result<::std::string::String, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        sources:
            ::std::result::Result<::std::vec::Vec<super::ArchivedSource>, ::std::string::String>,
        writer_version:
            ::std::result::Result<super::ArchiveIndexWriterVersion, ::std::string::String>,
    }
    impl ::std::default::Default for ArchiveIndex {
        fn default() -> Self {
            Self {
                created_unix_millis: Err("no value supplied for created_unix_millis".to_string()),
                entries: Err("no value supplied for entries".to_string()),
                project_id: Err("no value supplied for project_id".to_string()),
                project_name: Err("no value supplied for project_name".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                sources: Err("no value supplied for sources".to_string()),
                writer_version: Err("no value supplied for writer_version".to_string()),
            }
        }
    }
    impl ArchiveIndex {
        pub fn created_unix_millis<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.created_unix_millis = value.try_into().map_err(|e| {
                format!("error converting supplied value for created_unix_millis: {e}")
            });
            self
        }
        pub fn entries<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ArchiveEntry>>,
            T::Error: ::std::fmt::Display,
        {
            self.entries = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for entries: {e}"));
            self
        }
        pub fn project_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArchiveIndexProjectId>,
            T::Error: ::std::fmt::Display,
        {
            self.project_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for project_id: {e}"));
            self
        }
        pub fn project_name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.project_name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for project_name: {e}"));
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
        pub fn sources<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ArchivedSource>>,
            T::Error: ::std::fmt::Display,
        {
            self.sources = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sources: {e}"));
            self
        }
        pub fn writer_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArchiveIndexWriterVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.writer_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for writer_version: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ArchiveIndex> for super::ArchiveIndex {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ArchiveIndex,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                created_unix_millis: value.created_unix_millis?,
                entries: value.entries?,
                project_id: value.project_id?,
                project_name: value.project_name?,
                schema_version: value.schema_version?,
                sources: value.sources?,
                writer_version: value.writer_version?,
            })
        }
    }
    impl ::std::convert::From<super::ArchiveIndex> for ArchiveIndex {
        fn from(value: super::ArchiveIndex) -> Self {
            Self {
                created_unix_millis: Ok(value.created_unix_millis),
                entries: Ok(value.entries),
                project_id: Ok(value.project_id),
                project_name: Ok(value.project_name),
                schema_version: Ok(value.schema_version),
                sources: Ok(value.sources),
                writer_version: Ok(value.writer_version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ArchivedSource {
        display_name: ::std::result::Result<::std::string::String, ::std::string::String>,
        fingerprint: ::std::result::Result<super::ArchivedSourceFingerprint, ::std::string::String>,
        source_id: ::std::result::Result<super::ArchivedSourceSourceId, ::std::string::String>,
    }
    impl ::std::default::Default for ArchivedSource {
        fn default() -> Self {
            Self {
                display_name: Err("no value supplied for display_name".to_string()),
                fingerprint: Err("no value supplied for fingerprint".to_string()),
                source_id: Err("no value supplied for source_id".to_string()),
            }
        }
    }
    impl ArchivedSource {
        pub fn display_name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.display_name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for display_name: {e}"));
            self
        }
        pub fn fingerprint<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArchivedSourceFingerprint>,
            T::Error: ::std::fmt::Display,
        {
            self.fingerprint = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for fingerprint: {e}"));
            self
        }
        pub fn source_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ArchivedSourceSourceId>,
            T::Error: ::std::fmt::Display,
        {
            self.source_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for source_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ArchivedSource> for super::ArchivedSource {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ArchivedSource,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                display_name: value.display_name?,
                fingerprint: value.fingerprint?,
                source_id: value.source_id?,
            })
        }
    }
    impl ::std::convert::From<super::ArchivedSource> for ArchivedSource {
        fn from(value: super::ArchivedSource) -> Self {
            Self {
                display_name: Ok(value.display_name),
                fingerprint: Ok(value.fingerprint),
                source_id: Ok(value.source_id),
            }
        }
    }
}
