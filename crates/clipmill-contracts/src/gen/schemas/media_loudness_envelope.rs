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
#[doc = "EBU R128 loudness sampled at a fixed cadence over the 48 kHz rendition (book ch. 12/13). Timestamps are integer ticks at 1/90000 (D06); loudness values are measurements in LUFS/LU, which are legitimately real-valued. Downstream prosody features and the audio lane read this envelope instead of re-measuring."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.media.loudness_envelope.v1.json\","]
#[doc = "  \"title\": \"MediaLoudnessEnvelope\","]
#[doc = "  \"description\": \"EBU R128 loudness sampled at a fixed cadence over the 48 kHz rendition (book ch. 12/13). Timestamps are integer ticks at 1/90000 (D06); loudness values are measurements in LUFS/LU, which are legitimately real-valued. Downstream prosody features and the audio lane read this envelope instead of re-measuring.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cadence_hz\","]
#[doc = "    \"coverage\","]
#[doc = "    \"points\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"summary\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cadence_hz\": {"]
#[doc = "      \"description\": \"Envelope sampling cadence in points per second.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"$ref\": \"#/$defs/coverage\""]
#[doc = "    },"]
#[doc = "    \"points\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"momentary_lufs\","]
#[doc = "          \"t_ticks\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"momentary_lufs\": {"]
#[doc = "            \"type\": \"number\""]
#[doc = "          },"]
#[doc = "          \"short_term_lufs\": {"]
#[doc = "            \"type\": \"number\""]
#[doc = "          },"]
#[doc = "          \"t_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.media.loudness_envelope.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"summary\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"integrated_lufs\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"integrated_lufs\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"loudness_range_lu\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"sample_peak_dbfs\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MediaLoudnessEnvelope {
    #[doc = "Envelope sampling cadence in points per second."]
    pub cadence_hz: ::std::num::NonZeroU64,
    pub coverage: Coverage,
    pub points: ::std::vec::Vec<MediaLoudnessEnvelopePointsItem>,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    pub summary: MediaLoudnessEnvelopeSummary,
}
impl MediaLoudnessEnvelope {
    pub fn builder() -> builder::MediaLoudnessEnvelope {
        Default::default()
    }
}
#[doc = "`MediaLoudnessEnvelopePointsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"momentary_lufs\","]
#[doc = "    \"t_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"momentary_lufs\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"short_term_lufs\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"t_ticks\": {"]
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
pub struct MediaLoudnessEnvelopePointsItem {
    pub momentary_lufs: f64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub short_term_lufs: ::std::option::Option<f64>,
    pub t_ticks: u64,
}
impl MediaLoudnessEnvelopePointsItem {
    pub fn builder() -> builder::MediaLoudnessEnvelopePointsItem {
        Default::default()
    }
}
#[doc = "`MediaLoudnessEnvelopeSummary`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"integrated_lufs\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"integrated_lufs\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"loudness_range_lu\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"sample_peak_dbfs\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MediaLoudnessEnvelopeSummary {
    pub integrated_lufs: f64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub loudness_range_lu: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub sample_peak_dbfs: ::std::option::Option<f64>,
}
impl MediaLoudnessEnvelopeSummary {
    pub fn builder() -> builder::MediaLoudnessEnvelopeSummary {
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
    pub struct MediaLoudnessEnvelope {
        cadence_hz: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        coverage: ::std::result::Result<super::Coverage, ::std::string::String>,
        points: ::std::result::Result<
            ::std::vec::Vec<super::MediaLoudnessEnvelopePointsItem>,
            ::std::string::String,
        >,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        summary: ::std::result::Result<super::MediaLoudnessEnvelopeSummary, ::std::string::String>,
    }
    impl ::std::default::Default for MediaLoudnessEnvelope {
        fn default() -> Self {
            Self {
                cadence_hz: Err("no value supplied for cadence_hz".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                points: Err("no value supplied for points".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                summary: Err("no value supplied for summary".to_string()),
            }
        }
    }
    impl MediaLoudnessEnvelope {
        pub fn cadence_hz<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.cadence_hz = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cadence_hz: {e}"));
            self
        }
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
        pub fn points<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MediaLoudnessEnvelopePointsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.points = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for points: {e}"));
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
        pub fn summary<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MediaLoudnessEnvelopeSummary>,
            T::Error: ::std::fmt::Display,
        {
            self.summary = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for summary: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MediaLoudnessEnvelope> for super::MediaLoudnessEnvelope {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MediaLoudnessEnvelope,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cadence_hz: value.cadence_hz?,
                coverage: value.coverage?,
                points: value.points?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                summary: value.summary?,
            })
        }
    }
    impl ::std::convert::From<super::MediaLoudnessEnvelope> for MediaLoudnessEnvelope {
        fn from(value: super::MediaLoudnessEnvelope) -> Self {
            Self {
                cadence_hz: Ok(value.cadence_hz),
                coverage: Ok(value.coverage),
                points: Ok(value.points),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                summary: Ok(value.summary),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MediaLoudnessEnvelopePointsItem {
        momentary_lufs: ::std::result::Result<f64, ::std::string::String>,
        short_term_lufs: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        t_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for MediaLoudnessEnvelopePointsItem {
        fn default() -> Self {
            Self {
                momentary_lufs: Err("no value supplied for momentary_lufs".to_string()),
                short_term_lufs: Ok(Default::default()),
                t_ticks: Err("no value supplied for t_ticks".to_string()),
            }
        }
    }
    impl MediaLoudnessEnvelopePointsItem {
        pub fn momentary_lufs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.momentary_lufs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for momentary_lufs: {e}"));
            self
        }
        pub fn short_term_lufs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.short_term_lufs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for short_term_lufs: {e}"));
            self
        }
        pub fn t_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.t_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for t_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MediaLoudnessEnvelopePointsItem>
        for super::MediaLoudnessEnvelopePointsItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MediaLoudnessEnvelopePointsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                momentary_lufs: value.momentary_lufs?,
                short_term_lufs: value.short_term_lufs?,
                t_ticks: value.t_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::MediaLoudnessEnvelopePointsItem>
        for MediaLoudnessEnvelopePointsItem
    {
        fn from(value: super::MediaLoudnessEnvelopePointsItem) -> Self {
            Self {
                momentary_lufs: Ok(value.momentary_lufs),
                short_term_lufs: Ok(value.short_term_lufs),
                t_ticks: Ok(value.t_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MediaLoudnessEnvelopeSummary {
        integrated_lufs: ::std::result::Result<f64, ::std::string::String>,
        loudness_range_lu: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        sample_peak_dbfs: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for MediaLoudnessEnvelopeSummary {
        fn default() -> Self {
            Self {
                integrated_lufs: Err("no value supplied for integrated_lufs".to_string()),
                loudness_range_lu: Ok(Default::default()),
                sample_peak_dbfs: Ok(Default::default()),
            }
        }
    }
    impl MediaLoudnessEnvelopeSummary {
        pub fn integrated_lufs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.integrated_lufs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for integrated_lufs: {e}"));
            self
        }
        pub fn loudness_range_lu<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.loudness_range_lu = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for loudness_range_lu: {e}"));
            self
        }
        pub fn sample_peak_dbfs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.sample_peak_dbfs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sample_peak_dbfs: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MediaLoudnessEnvelopeSummary> for super::MediaLoudnessEnvelopeSummary {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MediaLoudnessEnvelopeSummary,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                integrated_lufs: value.integrated_lufs?,
                loudness_range_lu: value.loudness_range_lu?,
                sample_peak_dbfs: value.sample_peak_dbfs?,
            })
        }
    }
    impl ::std::convert::From<super::MediaLoudnessEnvelopeSummary> for MediaLoudnessEnvelopeSummary {
        fn from(value: super::MediaLoudnessEnvelopeSummary) -> Self {
            Self {
                integrated_lufs: Ok(value.integrated_lufs),
                loudness_range_lu: Ok(value.loudness_range_lu),
                sample_peak_dbfs: Ok(value.sample_peak_dbfs),
            }
        }
    }
}
