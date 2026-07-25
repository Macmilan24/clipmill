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
#[doc = "`Coverage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
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
pub struct Coverage {
    pub end_ticks: u64,
    pub start_ticks: u64,
}
impl Coverage {
    pub fn builder() -> builder::Coverage {
        Default::default()
    }
}
#[doc = "The exact-seek substrate for the original source (book ch. 12/17): every video keyframe with its edit-time tick and byte position, demuxed without decoding. The render compiler plans '-ss keyframe + accurate trim' from this index instead of trusting container duration math."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.media.reference_index.v1.json\","]
#[doc = "  \"title\": \"MediaReferenceIndex\","]
#[doc = "  \"description\": \"The exact-seek substrate for the original source (book ch. 12/17): every video keyframe with its edit-time tick and byte position, demuxed without decoding. The render compiler plans '-ss keyframe + accurate trim' from this index instead of trusting container duration math.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coverage\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"streams\","]
#[doc = "    \"video_keyframes\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.media.reference_index.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"streams\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"index\","]
#[doc = "          \"kind\","]
#[doc = "          \"packet_count\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"duration_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"index\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"kind\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"video\","]
#[doc = "              \"audio\","]
#[doc = "              \"subtitle\","]
#[doc = "              \"data\""]
#[doc = "            ]"]
#[doc = "          },"]
#[doc = "          \"packet_count\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"video_keyframes\": {"]
#[doc = "      \"description\": \"Keyframes across all video streams, ordered by (stream_index, pts_ticks).\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"pts_ticks\","]
#[doc = "          \"stream_index\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"byte_pos\": {"]
#[doc = "            \"description\": \"Byte position of the packet in the container when the demuxer reports one.\","]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"pts_ticks\": {"]
#[doc = "            \"description\": \"Presentation tick at 1/90000 in source edit time.\","]
#[doc = "            \"type\": \"integer\""]
#[doc = "          },"]
#[doc = "          \"stream_index\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
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
pub struct MediaReferenceIndex {
    pub coverage: Coverage,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    pub streams: ::std::vec::Vec<MediaReferenceIndexStreamsItem>,
    #[doc = "Keyframes across all video streams, ordered by (stream_index, pts_ticks)."]
    pub video_keyframes: ::std::vec::Vec<MediaReferenceIndexVideoKeyframesItem>,
}
impl MediaReferenceIndex {
    pub fn builder() -> builder::MediaReferenceIndex {
        Default::default()
    }
}
#[doc = "`MediaReferenceIndexStreamsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"index\","]
#[doc = "    \"kind\","]
#[doc = "    \"packet_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"duration_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"video\","]
#[doc = "        \"audio\","]
#[doc = "        \"subtitle\","]
#[doc = "        \"data\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"packet_count\": {"]
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
pub struct MediaReferenceIndexStreamsItem {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub duration_ticks: ::std::option::Option<u64>,
    pub index: u64,
    pub kind: MediaReferenceIndexStreamsItemKind,
    pub packet_count: u64,
}
impl MediaReferenceIndexStreamsItem {
    pub fn builder() -> builder::MediaReferenceIndexStreamsItem {
        Default::default()
    }
}
#[doc = "`MediaReferenceIndexStreamsItemKind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"video\","]
#[doc = "    \"audio\","]
#[doc = "    \"subtitle\","]
#[doc = "    \"data\""]
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
pub enum MediaReferenceIndexStreamsItemKind {
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "subtitle")]
    Subtitle,
    #[serde(rename = "data")]
    Data,
}
impl ::std::fmt::Display for MediaReferenceIndexStreamsItemKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Video => f.write_str("video"),
            Self::Audio => f.write_str("audio"),
            Self::Subtitle => f.write_str("subtitle"),
            Self::Data => f.write_str("data"),
        }
    }
}
impl ::std::str::FromStr for MediaReferenceIndexStreamsItemKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "video" => Ok(Self::Video),
            "audio" => Ok(Self::Audio),
            "subtitle" => Ok(Self::Subtitle),
            "data" => Ok(Self::Data),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MediaReferenceIndexStreamsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MediaReferenceIndexStreamsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MediaReferenceIndexStreamsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MediaReferenceIndexVideoKeyframesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"pts_ticks\","]
#[doc = "    \"stream_index\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"byte_pos\": {"]
#[doc = "      \"description\": \"Byte position of the packet in the container when the demuxer reports one.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"pts_ticks\": {"]
#[doc = "      \"description\": \"Presentation tick at 1/90000 in source edit time.\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"stream_index\": {"]
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
pub struct MediaReferenceIndexVideoKeyframesItem {
    #[doc = "Byte position of the packet in the container when the demuxer reports one."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub byte_pos: ::std::option::Option<u64>,
    #[doc = "Presentation tick at 1/90000 in source edit time."]
    pub pts_ticks: i64,
    pub stream_index: u64,
}
impl MediaReferenceIndexVideoKeyframesItem {
    pub fn builder() -> builder::MediaReferenceIndexVideoKeyframesItem {
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
    pub struct Coverage {
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for Coverage {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl Coverage {
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
    impl ::std::convert::TryFrom<Coverage> for super::Coverage {
        type Error = super::error::ConversionError;
        fn try_from(value: Coverage) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::Coverage> for Coverage {
        fn from(value: super::Coverage) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MediaReferenceIndex {
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        streams: ::std::result::Result<
            ::std::vec::Vec<super::MediaReferenceIndexStreamsItem>,
            ::std::string::String,
        >,
        video_keyframes: ::std::result::Result<
            ::std::vec::Vec<super::MediaReferenceIndexVideoKeyframesItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for MediaReferenceIndex {
        fn default() -> Self {
            Self {
                coverage: Err("no value supplied for coverage".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                streams: Err("no value supplied for streams".to_string()),
                video_keyframes: Err("no value supplied for video_keyframes".to_string()),
            }
        }
    }
    impl MediaReferenceIndex {
        pub fn coverage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Coverage>,
            T::Error: ::std::fmt::Display,
        {
            self.coverage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for coverage: {e}"));
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
        pub fn streams<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MediaReferenceIndexStreamsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.streams = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for streams: {e}"));
            self
        }
        pub fn video_keyframes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::vec::Vec<super::MediaReferenceIndexVideoKeyframesItem>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.video_keyframes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for video_keyframes: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MediaReferenceIndex> for super::MediaReferenceIndex {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MediaReferenceIndex,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                coverage: value.coverage?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                streams: value.streams?,
                video_keyframes: value.video_keyframes?,
            })
        }
    }
    impl ::std::convert::From<super::MediaReferenceIndex> for MediaReferenceIndex {
        fn from(value: super::MediaReferenceIndex) -> Self {
            Self {
                coverage: Ok(value.coverage),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                streams: Ok(value.streams),
                video_keyframes: Ok(value.video_keyframes),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MediaReferenceIndexStreamsItem {
        duration_ticks: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        kind:
            ::std::result::Result<super::MediaReferenceIndexStreamsItemKind, ::std::string::String>,
        packet_count: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for MediaReferenceIndexStreamsItem {
        fn default() -> Self {
            Self {
                duration_ticks: Ok(Default::default()),
                index: Err("no value supplied for index".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                packet_count: Err("no value supplied for packet_count".to_string()),
            }
        }
    }
    impl MediaReferenceIndexStreamsItem {
        pub fn duration_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.duration_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for duration_ticks: {e}"));
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
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MediaReferenceIndexStreamsItemKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn packet_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.packet_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for packet_count: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MediaReferenceIndexStreamsItem>
        for super::MediaReferenceIndexStreamsItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MediaReferenceIndexStreamsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                duration_ticks: value.duration_ticks?,
                index: value.index?,
                kind: value.kind?,
                packet_count: value.packet_count?,
            })
        }
    }
    impl ::std::convert::From<super::MediaReferenceIndexStreamsItem>
        for MediaReferenceIndexStreamsItem
    {
        fn from(value: super::MediaReferenceIndexStreamsItem) -> Self {
            Self {
                duration_ticks: Ok(value.duration_ticks),
                index: Ok(value.index),
                kind: Ok(value.kind),
                packet_count: Ok(value.packet_count),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MediaReferenceIndexVideoKeyframesItem {
        byte_pos: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        pts_ticks: ::std::result::Result<i64, ::std::string::String>,
        stream_index: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for MediaReferenceIndexVideoKeyframesItem {
        fn default() -> Self {
            Self {
                byte_pos: Ok(Default::default()),
                pts_ticks: Err("no value supplied for pts_ticks".to_string()),
                stream_index: Err("no value supplied for stream_index".to_string()),
            }
        }
    }
    impl MediaReferenceIndexVideoKeyframesItem {
        pub fn byte_pos<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.byte_pos = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for byte_pos: {e}"));
            self
        }
        pub fn pts_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.pts_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for pts_ticks: {e}"));
            self
        }
        pub fn stream_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.stream_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stream_index: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MediaReferenceIndexVideoKeyframesItem>
        for super::MediaReferenceIndexVideoKeyframesItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MediaReferenceIndexVideoKeyframesItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                byte_pos: value.byte_pos?,
                pts_ticks: value.pts_ticks?,
                stream_index: value.stream_index?,
            })
        }
    }
    impl ::std::convert::From<super::MediaReferenceIndexVideoKeyframesItem>
        for MediaReferenceIndexVideoKeyframesItem
    {
        fn from(value: super::MediaReferenceIndexVideoKeyframesItem) -> Self {
            Self {
                byte_pos: Ok(value.byte_pos),
                pts_ticks: Ok(value.pts_ticks),
                stream_index: Ok(value.stream_index),
            }
        }
    }
}
