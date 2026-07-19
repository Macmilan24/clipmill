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
#[doc = "`Color`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"primaries\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"range\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"space\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"transfer\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Color {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub primaries: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub range: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub space: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub transfer: ::std::option::Option<::std::string::String>,
}
impl ::std::default::Default for Color {
    fn default() -> Self {
        Self {
            primaries: Default::default(),
            range: Default::default(),
            space: Default::default(),
            transfer: Default::default(),
        }
    }
}
impl Color {
    pub fn builder() -> builder::Color {
        Default::default()
    }
}
#[doc = "`Hdr`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"content_light_level\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"mastering_display\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Hdr {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub content_light_level: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub mastering_display: ::std::option::Option<::std::string::String>,
}
impl ::std::default::Default for Hdr {
    fn default() -> Self {
        Self {
            content_light_level: Default::default(),
            mastering_display: Default::default(),
        }
    }
}
impl Hdr {
    pub fn builder() -> builder::Hdr {
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
#[doc = "The source map built at ingest: the ONLY authority for converting between edit time and source presentation time (book ch. 10/12). All durations and offsets are integer ticks at 1/90000 against the stated timebase — variable frame rate and 29.97 drop-frame material are exactly where float-seconds pipelines drift by a frame."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.source_map.v1.json\","]
#[doc = "  \"title\": \"SourceMap\","]
#[doc = "  \"description\": \"The source map built at ingest: the ONLY authority for converting between edit time and source presentation time (book ch. 10/12). All durations and offsets are integer ticks at 1/90000 against the stated timebase — variable frame rate and 29.97 drop-frame material are exactly where float-seconds pipelines drift by a frame.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"container\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\","]
#[doc = "    \"streams\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"chapters\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"end_ticks\","]
#[doc = "          \"id\","]
#[doc = "          \"start_ticks\","]
#[doc = "          \"title\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"end_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"id\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"start_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"title\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"container\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"duration_ticks\","]
#[doc = "        \"format\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"color\": {"]
#[doc = "          \"$ref\": \"#/$defs/color\""]
#[doc = "        },"]
#[doc = "        \"duration_ticks\": {"]
#[doc = "          \"description\": \"Container duration in ticks at 1/90000.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"format\": {"]
#[doc = "          \"description\": \"Container format, e.g. 'mov,mp4,m4a,3gp,3g2,mj2'.\","]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        \"hdr\": {"]
#[doc = "          \"$ref\": \"#/$defs/hdr\""]
#[doc = "        },"]
#[doc = "        \"rotation_deg\": {"]
#[doc = "          \"description\": \"Display rotation from side data / display matrix.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"enum\": ["]
#[doc = "            0,"]
#[doc = "            90,"]
#[doc = "            180,"]
#[doc = "            270"]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"mapping\": {"]
#[doc = "      \"description\": \"Exact source-presentation to 90 kHz edit-time mapping. Required on new Phase 0 output; omitted only by legacy v1 fixtures.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"edit_timebase\","]
#[doc = "        \"segments\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"edit_timebase\": {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"required\": ["]
#[doc = "            \"den\","]
#[doc = "            \"num\""]
#[doc = "          ],"]
#[doc = "          \"properties\": {"]
#[doc = "            \"den\": {"]
#[doc = "              \"const\": 90000"]
#[doc = "            },"]
#[doc = "            \"num\": {"]
#[doc = "              \"const\": 1"]
#[doc = "            }"]
#[doc = "          },"]
#[doc = "          \"additionalProperties\": false"]
#[doc = "        },"]
#[doc = "        \"segments\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"edit_end_ticks\","]
#[doc = "              \"edit_start_ticks\","]
#[doc = "              \"segment_id\","]
#[doc = "              \"source_end\","]
#[doc = "              \"source_start\","]
#[doc = "              \"stream_index\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"edit_end_ticks\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              },"]
#[doc = "              \"edit_start_ticks\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              },"]
#[doc = "              \"segment_id\": {"]
#[doc = "                \"type\": \"string\","]
#[doc = "                \"minLength\": 1"]
#[doc = "              },"]
#[doc = "              \"source_end\": {"]
#[doc = "                \"type\": \"integer\""]
#[doc = "              },"]
#[doc = "              \"source_start\": {"]
#[doc = "                \"type\": \"integer\""]
#[doc = "              },"]
#[doc = "              \"stream_index\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 0.0"]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          },"]
#[doc = "          \"minItems\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.source_map.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"streams\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"codec\","]
#[doc = "          \"index\","]
#[doc = "          \"kind\","]
#[doc = "          \"timebase\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"audio\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"channels\","]
#[doc = "              \"sample_rate\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"channels\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 1.0"]
#[doc = "              },"]
#[doc = "              \"layout\": {"]
#[doc = "                \"type\": \"string\""]
#[doc = "              },"]
#[doc = "              \"sample_rate\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 1.0"]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          },"]
#[doc = "          \"codec\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          },"]
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
#[doc = "          \"start_offset_ticks\": {"]
#[doc = "            \"description\": \"Stream start offset in ticks at 1/90000.\","]
#[doc = "            \"type\": \"integer\""]
#[doc = "          },"]
#[doc = "          \"timebase\": {"]
#[doc = "            \"$ref\": \"#/$defs/timebase\""]
#[doc = "          },"]
#[doc = "          \"video\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"coded_height\","]
#[doc = "              \"coded_width\","]
#[doc = "              \"display_height\","]
#[doc = "              \"display_width\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"coded_height\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 1.0"]
#[doc = "              },"]
#[doc = "              \"coded_width\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 1.0"]
#[doc = "              },"]
#[doc = "              \"color\": {"]
#[doc = "                \"$ref\": \"#/$defs/color\""]
#[doc = "              },"]
#[doc = "              \"display_height\": {"]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 1.0"]
#[doc = "              },"]
#[doc = "              \"display_width\": {"]
#[doc = "                \"description\": \"Width after rotation/aspect are applied — what the viewer sees.\","]
#[doc = "                \"type\": \"integer\","]
#[doc = "                \"minimum\": 1.0"]
#[doc = "              },"]
#[doc = "              \"frame_rate\": {"]
#[doc = "                \"$ref\": \"#/$defs/timebase\""]
#[doc = "              },"]
#[doc = "              \"hdr\": {"]
#[doc = "                \"$ref\": \"#/$defs/hdr\""]
#[doc = "              },"]
#[doc = "              \"vfr\": {"]
#[doc = "                \"description\": \"True when frame intervals vary (variable frame rate).\","]
#[doc = "                \"type\": \"boolean\""]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
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
pub struct SourceMap {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub chapters: ::std::vec::Vec<SourceMapChaptersItem>,
    pub container: SourceMapContainer,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub mapping: ::std::option::Option<SourceMapMapping>,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    pub streams: ::std::vec::Vec<SourceMapStreamsItem>,
}
impl SourceMap {
    pub fn builder() -> builder::SourceMap {
        Default::default()
    }
}
#[doc = "`SourceMapChaptersItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"id\","]
#[doc = "    \"start_ticks\","]
#[doc = "    \"title\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SourceMapChaptersItem {
    pub end_ticks: u64,
    pub id: u64,
    pub start_ticks: u64,
    pub title: ::std::string::String,
}
impl SourceMapChaptersItem {
    pub fn builder() -> builder::SourceMapChaptersItem {
        Default::default()
    }
}
#[doc = "`SourceMapContainer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"duration_ticks\","]
#[doc = "    \"format\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"color\": {"]
#[doc = "      \"$ref\": \"#/$defs/color\""]
#[doc = "    },"]
#[doc = "    \"duration_ticks\": {"]
#[doc = "      \"description\": \"Container duration in ticks at 1/90000.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"format\": {"]
#[doc = "      \"description\": \"Container format, e.g. 'mov,mp4,m4a,3gp,3g2,mj2'.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"hdr\": {"]
#[doc = "      \"$ref\": \"#/$defs/hdr\""]
#[doc = "    },"]
#[doc = "    \"rotation_deg\": {"]
#[doc = "      \"description\": \"Display rotation from side data / display matrix.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        0,"]
#[doc = "        90,"]
#[doc = "        180,"]
#[doc = "        270"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SourceMapContainer {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub color: ::std::option::Option<Color>,
    #[doc = "Container duration in ticks at 1/90000."]
    pub duration_ticks: u64,
    #[doc = "Container format, e.g. 'mov,mp4,m4a,3gp,3g2,mj2'."]
    pub format: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub hdr: ::std::option::Option<Hdr>,
    #[doc = "Display rotation from side data / display matrix."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub rotation_deg: ::std::option::Option<SourceMapContainerRotationDeg>,
}
impl SourceMapContainer {
    pub fn builder() -> builder::SourceMapContainer {
        Default::default()
    }
}
#[doc = "Display rotation from side data / display matrix."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Display rotation from side data / display matrix.\","]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    0,"]
#[doc = "    90,"]
#[doc = "    180,"]
#[doc = "    270"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct SourceMapContainerRotationDeg(i64);
impl ::std::ops::Deref for SourceMapContainerRotationDeg {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<SourceMapContainerRotationDeg> for i64 {
    fn from(value: SourceMapContainerRotationDeg) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for SourceMapContainerRotationDeg {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![0_i64, 90_i64, 180_i64, 270_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for SourceMapContainerRotationDeg {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "Exact source-presentation to 90 kHz edit-time mapping. Required on new Phase 0 output; omitted only by legacy v1 fixtures."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Exact source-presentation to 90 kHz edit-time mapping. Required on new Phase 0 output; omitted only by legacy v1 fixtures.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"edit_timebase\","]
#[doc = "    \"segments\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"edit_timebase\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"den\","]
#[doc = "        \"num\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"den\": {"]
#[doc = "          \"const\": 90000"]
#[doc = "        },"]
#[doc = "        \"num\": {"]
#[doc = "          \"const\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"segments\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"edit_end_ticks\","]
#[doc = "          \"edit_start_ticks\","]
#[doc = "          \"segment_id\","]
#[doc = "          \"source_end\","]
#[doc = "          \"source_start\","]
#[doc = "          \"stream_index\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"edit_end_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"edit_start_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"segment_id\": {"]
#[doc = "            \"type\": \"string\","]
#[doc = "            \"minLength\": 1"]
#[doc = "          },"]
#[doc = "          \"source_end\": {"]
#[doc = "            \"type\": \"integer\""]
#[doc = "          },"]
#[doc = "          \"source_start\": {"]
#[doc = "            \"type\": \"integer\""]
#[doc = "          },"]
#[doc = "          \"stream_index\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
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
pub struct SourceMapMapping {
    pub edit_timebase: SourceMapMappingEditTimebase,
    pub segments: ::std::vec::Vec<SourceMapMappingSegmentsItem>,
}
impl SourceMapMapping {
    pub fn builder() -> builder::SourceMapMapping {
        Default::default()
    }
}
#[doc = "`SourceMapMappingEditTimebase`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"den\","]
#[doc = "    \"num\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"den\": {"]
#[doc = "      \"const\": 90000"]
#[doc = "    },"]
#[doc = "    \"num\": {"]
#[doc = "      \"const\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SourceMapMappingEditTimebase {
    pub den: ::serde_json::Value,
    pub num: ::serde_json::Value,
}
impl SourceMapMappingEditTimebase {
    pub fn builder() -> builder::SourceMapMappingEditTimebase {
        Default::default()
    }
}
#[doc = "`SourceMapMappingSegmentsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"edit_end_ticks\","]
#[doc = "    \"edit_start_ticks\","]
#[doc = "    \"segment_id\","]
#[doc = "    \"source_end\","]
#[doc = "    \"source_start\","]
#[doc = "    \"stream_index\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"edit_end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"edit_start_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"segment_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"source_end\": {"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"source_start\": {"]
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
pub struct SourceMapMappingSegmentsItem {
    pub edit_end_ticks: u64,
    pub edit_start_ticks: u64,
    pub segment_id: SourceMapMappingSegmentsItemSegmentId,
    pub source_end: i64,
    pub source_start: i64,
    pub stream_index: u64,
}
impl SourceMapMappingSegmentsItem {
    pub fn builder() -> builder::SourceMapMappingSegmentsItem {
        Default::default()
    }
}
#[doc = "`SourceMapMappingSegmentsItemSegmentId`"]
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
pub struct SourceMapMappingSegmentsItemSegmentId(::std::string::String);
impl ::std::ops::Deref for SourceMapMappingSegmentsItemSegmentId {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<SourceMapMappingSegmentsItemSegmentId> for ::std::string::String {
    fn from(value: SourceMapMappingSegmentsItemSegmentId) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for SourceMapMappingSegmentsItemSegmentId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for SourceMapMappingSegmentsItemSegmentId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SourceMapMappingSegmentsItemSegmentId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SourceMapMappingSegmentsItemSegmentId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for SourceMapMappingSegmentsItemSegmentId {
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
#[doc = "`SourceMapStreamsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"codec\","]
#[doc = "    \"index\","]
#[doc = "    \"kind\","]
#[doc = "    \"timebase\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"audio\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"channels\","]
#[doc = "        \"sample_rate\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"channels\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"layout\": {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        \"sample_rate\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"codec\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
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
#[doc = "    \"start_offset_ticks\": {"]
#[doc = "      \"description\": \"Stream start offset in ticks at 1/90000.\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"timebase\": {"]
#[doc = "      \"$ref\": \"#/$defs/timebase\""]
#[doc = "    },"]
#[doc = "    \"video\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"coded_height\","]
#[doc = "        \"coded_width\","]
#[doc = "        \"display_height\","]
#[doc = "        \"display_width\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"coded_height\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"coded_width\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"color\": {"]
#[doc = "          \"$ref\": \"#/$defs/color\""]
#[doc = "        },"]
#[doc = "        \"display_height\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"display_width\": {"]
#[doc = "          \"description\": \"Width after rotation/aspect are applied — what the viewer sees.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"frame_rate\": {"]
#[doc = "          \"$ref\": \"#/$defs/timebase\""]
#[doc = "        },"]
#[doc = "        \"hdr\": {"]
#[doc = "          \"$ref\": \"#/$defs/hdr\""]
#[doc = "        },"]
#[doc = "        \"vfr\": {"]
#[doc = "          \"description\": \"True when frame intervals vary (variable frame rate).\","]
#[doc = "          \"type\": \"boolean\""]
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
pub struct SourceMapStreamsItem {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub audio: ::std::option::Option<SourceMapStreamsItemAudio>,
    pub codec: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub duration_ticks: ::std::option::Option<u64>,
    pub index: u64,
    pub kind: SourceMapStreamsItemKind,
    #[doc = "Stream start offset in ticks at 1/90000."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub start_offset_ticks: ::std::option::Option<i64>,
    pub timebase: Timebase,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub video: ::std::option::Option<SourceMapStreamsItemVideo>,
}
impl SourceMapStreamsItem {
    pub fn builder() -> builder::SourceMapStreamsItem {
        Default::default()
    }
}
#[doc = "`SourceMapStreamsItemAudio`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"channels\","]
#[doc = "    \"sample_rate\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"channels\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"layout\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"sample_rate\": {"]
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
pub struct SourceMapStreamsItemAudio {
    pub channels: ::std::num::NonZeroU64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub layout: ::std::option::Option<::std::string::String>,
    pub sample_rate: ::std::num::NonZeroU64,
}
impl SourceMapStreamsItemAudio {
    pub fn builder() -> builder::SourceMapStreamsItemAudio {
        Default::default()
    }
}
#[doc = "`SourceMapStreamsItemKind`"]
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
pub enum SourceMapStreamsItemKind {
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "subtitle")]
    Subtitle,
    #[serde(rename = "data")]
    Data,
}
impl ::std::fmt::Display for SourceMapStreamsItemKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Video => f.write_str("video"),
            Self::Audio => f.write_str("audio"),
            Self::Subtitle => f.write_str("subtitle"),
            Self::Data => f.write_str("data"),
        }
    }
}
impl ::std::str::FromStr for SourceMapStreamsItemKind {
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
impl ::std::convert::TryFrom<&str> for SourceMapStreamsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SourceMapStreamsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SourceMapStreamsItemKind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`SourceMapStreamsItemVideo`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coded_height\","]
#[doc = "    \"coded_width\","]
#[doc = "    \"display_height\","]
#[doc = "    \"display_width\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"coded_height\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"coded_width\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"color\": {"]
#[doc = "      \"$ref\": \"#/$defs/color\""]
#[doc = "    },"]
#[doc = "    \"display_height\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"display_width\": {"]
#[doc = "      \"description\": \"Width after rotation/aspect are applied — what the viewer sees.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"frame_rate\": {"]
#[doc = "      \"$ref\": \"#/$defs/timebase\""]
#[doc = "    },"]
#[doc = "    \"hdr\": {"]
#[doc = "      \"$ref\": \"#/$defs/hdr\""]
#[doc = "    },"]
#[doc = "    \"vfr\": {"]
#[doc = "      \"description\": \"True when frame intervals vary (variable frame rate).\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SourceMapStreamsItemVideo {
    pub coded_height: ::std::num::NonZeroU64,
    pub coded_width: ::std::num::NonZeroU64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub color: ::std::option::Option<Color>,
    pub display_height: ::std::num::NonZeroU64,
    #[doc = "Width after rotation/aspect are applied — what the viewer sees."]
    pub display_width: ::std::num::NonZeroU64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub frame_rate: ::std::option::Option<Timebase>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub hdr: ::std::option::Option<Hdr>,
    #[doc = "True when frame intervals vary (variable frame rate)."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub vfr: ::std::option::Option<bool>,
}
impl SourceMapStreamsItemVideo {
    pub fn builder() -> builder::SourceMapStreamsItemVideo {
        Default::default()
    }
}
#[doc = "`Timebase`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"den\","]
#[doc = "    \"num\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"den\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"num\": {"]
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
pub struct Timebase {
    pub den: ::std::num::NonZeroU64,
    pub num: ::std::num::NonZeroU64,
}
impl Timebase {
    pub fn builder() -> builder::Timebase {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct Color {
        primaries: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        range: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        space: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        transfer: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Color {
        fn default() -> Self {
            Self {
                primaries: Ok(Default::default()),
                range: Ok(Default::default()),
                space: Ok(Default::default()),
                transfer: Ok(Default::default()),
            }
        }
    }
    impl Color {
        pub fn primaries<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.primaries = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for primaries: {e}"));
            self
        }
        pub fn range<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.range = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for range: {e}"));
            self
        }
        pub fn space<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.space = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for space: {e}"));
            self
        }
        pub fn transfer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.transfer = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transfer: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Color> for super::Color {
        type Error = super::error::ConversionError;
        fn try_from(value: Color) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                primaries: value.primaries?,
                range: value.range?,
                space: value.space?,
                transfer: value.transfer?,
            })
        }
    }
    impl ::std::convert::From<super::Color> for Color {
        fn from(value: super::Color) -> Self {
            Self {
                primaries: Ok(value.primaries),
                range: Ok(value.range),
                space: Ok(value.space),
                transfer: Ok(value.transfer),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Hdr {
        content_light_level: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        mastering_display: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Hdr {
        fn default() -> Self {
            Self {
                content_light_level: Ok(Default::default()),
                mastering_display: Ok(Default::default()),
            }
        }
    }
    impl Hdr {
        pub fn content_light_level<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.content_light_level = value.try_into().map_err(|e| {
                format!("error converting supplied value for content_light_level: {e}")
            });
            self
        }
        pub fn mastering_display<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.mastering_display = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mastering_display: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Hdr> for super::Hdr {
        type Error = super::error::ConversionError;
        fn try_from(value: Hdr) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                content_light_level: value.content_light_level?,
                mastering_display: value.mastering_display?,
            })
        }
    }
    impl ::std::convert::From<super::Hdr> for Hdr {
        fn from(value: super::Hdr) -> Self {
            Self {
                content_light_level: Ok(value.content_light_level),
                mastering_display: Ok(value.mastering_display),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMap {
        chapters: ::std::result::Result<
            ::std::vec::Vec<super::SourceMapChaptersItem>,
            ::std::string::String,
        >,
        container: ::std::result::Result<super::SourceMapContainer, ::std::string::String>,
        mapping: ::std::result::Result<
            ::std::option::Option<super::SourceMapMapping>,
            ::std::string::String,
        >,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        streams: ::std::result::Result<
            ::std::vec::Vec<super::SourceMapStreamsItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for SourceMap {
        fn default() -> Self {
            Self {
                chapters: Ok(Default::default()),
                container: Err("no value supplied for container".to_string()),
                mapping: Ok(Default::default()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                streams: Err("no value supplied for streams".to_string()),
            }
        }
    }
    impl SourceMap {
        pub fn chapters<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::SourceMapChaptersItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.chapters = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for chapters: {e}"));
            self
        }
        pub fn container<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SourceMapContainer>,
            T::Error: ::std::fmt::Display,
        {
            self.container = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for container: {e}"));
            self
        }
        pub fn mapping<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::SourceMapMapping>>,
            T::Error: ::std::fmt::Display,
        {
            self.mapping = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mapping: {e}"));
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
            T: ::std::convert::TryInto<::std::vec::Vec<super::SourceMapStreamsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.streams = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for streams: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SourceMap> for super::SourceMap {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMap,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                chapters: value.chapters?,
                container: value.container?,
                mapping: value.mapping?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                streams: value.streams?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMap> for SourceMap {
        fn from(value: super::SourceMap) -> Self {
            Self {
                chapters: Ok(value.chapters),
                container: Ok(value.container),
                mapping: Ok(value.mapping),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                streams: Ok(value.streams),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMapChaptersItem {
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        id: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
        title: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for SourceMapChaptersItem {
        fn default() -> Self {
            Self {
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                id: Err("no value supplied for id".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
                title: Err("no value supplied for title".to_string()),
            }
        }
    }
    impl SourceMapChaptersItem {
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
        pub fn id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for id: {e}"));
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
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SourceMapChaptersItem> for super::SourceMapChaptersItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMapChaptersItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                end_ticks: value.end_ticks?,
                id: value.id?,
                start_ticks: value.start_ticks?,
                title: value.title?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMapChaptersItem> for SourceMapChaptersItem {
        fn from(value: super::SourceMapChaptersItem) -> Self {
            Self {
                end_ticks: Ok(value.end_ticks),
                id: Ok(value.id),
                start_ticks: Ok(value.start_ticks),
                title: Ok(value.title),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMapContainer {
        color: ::std::result::Result<::std::option::Option<super::Color>, ::std::string::String>,
        duration_ticks: ::std::result::Result<u64, ::std::string::String>,
        format: ::std::result::Result<::std::string::String, ::std::string::String>,
        hdr: ::std::result::Result<::std::option::Option<super::Hdr>, ::std::string::String>,
        rotation_deg: ::std::result::Result<
            ::std::option::Option<super::SourceMapContainerRotationDeg>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for SourceMapContainer {
        fn default() -> Self {
            Self {
                color: Ok(Default::default()),
                duration_ticks: Err("no value supplied for duration_ticks".to_string()),
                format: Err("no value supplied for format".to_string()),
                hdr: Ok(Default::default()),
                rotation_deg: Ok(Default::default()),
            }
        }
    }
    impl SourceMapContainer {
        pub fn color<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Color>>,
            T::Error: ::std::fmt::Display,
        {
            self.color = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for color: {e}"));
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
        pub fn format<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.format = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for format: {e}"));
            self
        }
        pub fn hdr<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Hdr>>,
            T::Error: ::std::fmt::Display,
        {
            self.hdr = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hdr: {e}"));
            self
        }
        pub fn rotation_deg<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::SourceMapContainerRotationDeg>>,
            T::Error: ::std::fmt::Display,
        {
            self.rotation_deg = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rotation_deg: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SourceMapContainer> for super::SourceMapContainer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMapContainer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                color: value.color?,
                duration_ticks: value.duration_ticks?,
                format: value.format?,
                hdr: value.hdr?,
                rotation_deg: value.rotation_deg?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMapContainer> for SourceMapContainer {
        fn from(value: super::SourceMapContainer) -> Self {
            Self {
                color: Ok(value.color),
                duration_ticks: Ok(value.duration_ticks),
                format: Ok(value.format),
                hdr: Ok(value.hdr),
                rotation_deg: Ok(value.rotation_deg),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMapMapping {
        edit_timebase:
            ::std::result::Result<super::SourceMapMappingEditTimebase, ::std::string::String>,
        segments: ::std::result::Result<
            ::std::vec::Vec<super::SourceMapMappingSegmentsItem>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for SourceMapMapping {
        fn default() -> Self {
            Self {
                edit_timebase: Err("no value supplied for edit_timebase".to_string()),
                segments: Err("no value supplied for segments".to_string()),
            }
        }
    }
    impl SourceMapMapping {
        pub fn edit_timebase<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SourceMapMappingEditTimebase>,
            T::Error: ::std::fmt::Display,
        {
            self.edit_timebase = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for edit_timebase: {e}"));
            self
        }
        pub fn segments<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::SourceMapMappingSegmentsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.segments = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segments: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SourceMapMapping> for super::SourceMapMapping {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMapMapping,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                edit_timebase: value.edit_timebase?,
                segments: value.segments?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMapMapping> for SourceMapMapping {
        fn from(value: super::SourceMapMapping) -> Self {
            Self {
                edit_timebase: Ok(value.edit_timebase),
                segments: Ok(value.segments),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMapMappingEditTimebase {
        den: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        num: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for SourceMapMappingEditTimebase {
        fn default() -> Self {
            Self {
                den: Err("no value supplied for den".to_string()),
                num: Err("no value supplied for num".to_string()),
            }
        }
    }
    impl SourceMapMappingEditTimebase {
        pub fn den<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.den = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for den: {e}"));
            self
        }
        pub fn num<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::serde_json::Value>,
            T::Error: ::std::fmt::Display,
        {
            self.num = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for num: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SourceMapMappingEditTimebase> for super::SourceMapMappingEditTimebase {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMapMappingEditTimebase,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                den: value.den?,
                num: value.num?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMapMappingEditTimebase> for SourceMapMappingEditTimebase {
        fn from(value: super::SourceMapMappingEditTimebase) -> Self {
            Self {
                den: Ok(value.den),
                num: Ok(value.num),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMapMappingSegmentsItem {
        edit_end_ticks: ::std::result::Result<u64, ::std::string::String>,
        edit_start_ticks: ::std::result::Result<u64, ::std::string::String>,
        segment_id: ::std::result::Result<
            super::SourceMapMappingSegmentsItemSegmentId,
            ::std::string::String,
        >,
        source_end: ::std::result::Result<i64, ::std::string::String>,
        source_start: ::std::result::Result<i64, ::std::string::String>,
        stream_index: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for SourceMapMappingSegmentsItem {
        fn default() -> Self {
            Self {
                edit_end_ticks: Err("no value supplied for edit_end_ticks".to_string()),
                edit_start_ticks: Err("no value supplied for edit_start_ticks".to_string()),
                segment_id: Err("no value supplied for segment_id".to_string()),
                source_end: Err("no value supplied for source_end".to_string()),
                source_start: Err("no value supplied for source_start".to_string()),
                stream_index: Err("no value supplied for stream_index".to_string()),
            }
        }
    }
    impl SourceMapMappingSegmentsItem {
        pub fn edit_end_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.edit_end_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for edit_end_ticks: {e}"));
            self
        }
        pub fn edit_start_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.edit_start_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for edit_start_ticks: {e}"));
            self
        }
        pub fn segment_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SourceMapMappingSegmentsItemSegmentId>,
            T::Error: ::std::fmt::Display,
        {
            self.segment_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for segment_id: {e}"));
            self
        }
        pub fn source_end<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.source_end = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for source_end: {e}"));
            self
        }
        pub fn source_start<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.source_start = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for source_start: {e}"));
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
    impl ::std::convert::TryFrom<SourceMapMappingSegmentsItem> for super::SourceMapMappingSegmentsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMapMappingSegmentsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                edit_end_ticks: value.edit_end_ticks?,
                edit_start_ticks: value.edit_start_ticks?,
                segment_id: value.segment_id?,
                source_end: value.source_end?,
                source_start: value.source_start?,
                stream_index: value.stream_index?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMapMappingSegmentsItem> for SourceMapMappingSegmentsItem {
        fn from(value: super::SourceMapMappingSegmentsItem) -> Self {
            Self {
                edit_end_ticks: Ok(value.edit_end_ticks),
                edit_start_ticks: Ok(value.edit_start_ticks),
                segment_id: Ok(value.segment_id),
                source_end: Ok(value.source_end),
                source_start: Ok(value.source_start),
                stream_index: Ok(value.stream_index),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMapStreamsItem {
        audio: ::std::result::Result<
            ::std::option::Option<super::SourceMapStreamsItemAudio>,
            ::std::string::String,
        >,
        codec: ::std::result::Result<::std::string::String, ::std::string::String>,
        duration_ticks: ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        index: ::std::result::Result<u64, ::std::string::String>,
        kind: ::std::result::Result<super::SourceMapStreamsItemKind, ::std::string::String>,
        start_offset_ticks:
            ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        timebase: ::std::result::Result<super::Timebase, ::std::string::String>,
        video: ::std::result::Result<
            ::std::option::Option<super::SourceMapStreamsItemVideo>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for SourceMapStreamsItem {
        fn default() -> Self {
            Self {
                audio: Ok(Default::default()),
                codec: Err("no value supplied for codec".to_string()),
                duration_ticks: Ok(Default::default()),
                index: Err("no value supplied for index".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                start_offset_ticks: Ok(Default::default()),
                timebase: Err("no value supplied for timebase".to_string()),
                video: Ok(Default::default()),
            }
        }
    }
    impl SourceMapStreamsItem {
        pub fn audio<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::SourceMapStreamsItemAudio>>,
            T::Error: ::std::fmt::Display,
        {
            self.audio = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio: {e}"));
            self
        }
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
            T: ::std::convert::TryInto<super::SourceMapStreamsItemKind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn start_offset_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.start_offset_ticks = value.try_into().map_err(|e| {
                format!("error converting supplied value for start_offset_ticks: {e}")
            });
            self
        }
        pub fn timebase<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Timebase>,
            T::Error: ::std::fmt::Display,
        {
            self.timebase = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timebase: {e}"));
            self
        }
        pub fn video<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::SourceMapStreamsItemVideo>>,
            T::Error: ::std::fmt::Display,
        {
            self.video = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for video: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SourceMapStreamsItem> for super::SourceMapStreamsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMapStreamsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                audio: value.audio?,
                codec: value.codec?,
                duration_ticks: value.duration_ticks?,
                index: value.index?,
                kind: value.kind?,
                start_offset_ticks: value.start_offset_ticks?,
                timebase: value.timebase?,
                video: value.video?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMapStreamsItem> for SourceMapStreamsItem {
        fn from(value: super::SourceMapStreamsItem) -> Self {
            Self {
                audio: Ok(value.audio),
                codec: Ok(value.codec),
                duration_ticks: Ok(value.duration_ticks),
                index: Ok(value.index),
                kind: Ok(value.kind),
                start_offset_ticks: Ok(value.start_offset_ticks),
                timebase: Ok(value.timebase),
                video: Ok(value.video),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMapStreamsItemAudio {
        channels: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        layout: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        sample_rate: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for SourceMapStreamsItemAudio {
        fn default() -> Self {
            Self {
                channels: Err("no value supplied for channels".to_string()),
                layout: Ok(Default::default()),
                sample_rate: Err("no value supplied for sample_rate".to_string()),
            }
        }
    }
    impl SourceMapStreamsItemAudio {
        pub fn channels<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.channels = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for channels: {e}"));
            self
        }
        pub fn layout<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.layout = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for layout: {e}"));
            self
        }
        pub fn sample_rate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.sample_rate = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sample_rate: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SourceMapStreamsItemAudio> for super::SourceMapStreamsItemAudio {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMapStreamsItemAudio,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                channels: value.channels?,
                layout: value.layout?,
                sample_rate: value.sample_rate?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMapStreamsItemAudio> for SourceMapStreamsItemAudio {
        fn from(value: super::SourceMapStreamsItemAudio) -> Self {
            Self {
                channels: Ok(value.channels),
                layout: Ok(value.layout),
                sample_rate: Ok(value.sample_rate),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct SourceMapStreamsItemVideo {
        coded_height: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        coded_width: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        color: ::std::result::Result<::std::option::Option<super::Color>, ::std::string::String>,
        display_height: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        display_width: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        frame_rate:
            ::std::result::Result<::std::option::Option<super::Timebase>, ::std::string::String>,
        hdr: ::std::result::Result<::std::option::Option<super::Hdr>, ::std::string::String>,
        vfr: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
    }
    impl ::std::default::Default for SourceMapStreamsItemVideo {
        fn default() -> Self {
            Self {
                coded_height: Err("no value supplied for coded_height".to_string()),
                coded_width: Err("no value supplied for coded_width".to_string()),
                color: Ok(Default::default()),
                display_height: Err("no value supplied for display_height".to_string()),
                display_width: Err("no value supplied for display_width".to_string()),
                frame_rate: Ok(Default::default()),
                hdr: Ok(Default::default()),
                vfr: Ok(Default::default()),
            }
        }
    }
    impl SourceMapStreamsItemVideo {
        pub fn coded_height<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.coded_height = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for coded_height: {e}"));
            self
        }
        pub fn coded_width<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.coded_width = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for coded_width: {e}"));
            self
        }
        pub fn color<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Color>>,
            T::Error: ::std::fmt::Display,
        {
            self.color = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for color: {e}"));
            self
        }
        pub fn display_height<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.display_height = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for display_height: {e}"));
            self
        }
        pub fn display_width<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.display_width = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for display_width: {e}"));
            self
        }
        pub fn frame_rate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Timebase>>,
            T::Error: ::std::fmt::Display,
        {
            self.frame_rate = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for frame_rate: {e}"));
            self
        }
        pub fn hdr<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Hdr>>,
            T::Error: ::std::fmt::Display,
        {
            self.hdr = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hdr: {e}"));
            self
        }
        pub fn vfr<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.vfr = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for vfr: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<SourceMapStreamsItemVideo> for super::SourceMapStreamsItemVideo {
        type Error = super::error::ConversionError;
        fn try_from(
            value: SourceMapStreamsItemVideo,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                coded_height: value.coded_height?,
                coded_width: value.coded_width?,
                color: value.color?,
                display_height: value.display_height?,
                display_width: value.display_width?,
                frame_rate: value.frame_rate?,
                hdr: value.hdr?,
                vfr: value.vfr?,
            })
        }
    }
    impl ::std::convert::From<super::SourceMapStreamsItemVideo> for SourceMapStreamsItemVideo {
        fn from(value: super::SourceMapStreamsItemVideo) -> Self {
            Self {
                coded_height: Ok(value.coded_height),
                coded_width: Ok(value.coded_width),
                color: Ok(value.color),
                display_height: Ok(value.display_height),
                display_width: Ok(value.display_width),
                frame_rate: Ok(value.frame_rate),
                hdr: Ok(value.hdr),
                vfr: Ok(value.vfr),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Timebase {
        den: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        num: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for Timebase {
        fn default() -> Self {
            Self {
                den: Err("no value supplied for den".to_string()),
                num: Err("no value supplied for num".to_string()),
            }
        }
    }
    impl Timebase {
        pub fn den<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.den = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for den: {e}"));
            self
        }
        pub fn num<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.num = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for num: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Timebase> for super::Timebase {
        type Error = super::error::ConversionError;
        fn try_from(value: Timebase) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                den: value.den?,
                num: value.num?,
            })
        }
    }
    impl ::std::convert::From<super::Timebase> for Timebase {
        fn from(value: super::Timebase) -> Self {
            Self {
                den: Ok(value.den),
                num: Ok(value.num),
            }
        }
    }
}
