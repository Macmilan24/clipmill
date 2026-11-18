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
#[doc = "Timed sounds and vocal delivery in the 16 kHz rendition. A tagger detects laughter, applause, cheering, music, shouts and gasps. Emphasis records a sustained rise in pitch or level relative to recent voiced speech. Intervals use 1/90000-second ticks."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.audio.events.v1.json\","]
#[doc = "  \"title\": \"AudioEvents\","]
#[doc = "  \"description\": \"Timed sounds and vocal delivery in the 16 kHz rendition. A tagger detects laughter, applause, cheering, music, shouts and gasps. Emphasis records a sustained rise in pitch or level relative to recent voiced speech. Intervals use 1/90000-second ticks.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"audio_artifact_id\","]
#[doc = "    \"coverage\","]
#[doc = "    \"detection\","]
#[doc = "    \"events\","]
#[doc = "    \"producer\","]
#[doc = "    \"schema_version\","]
#[doc = "    \"source_fingerprint\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"audio_artifact_id\": {"]
#[doc = "      \"description\": \"The 16 kHz mono rendition this pass read, verified before use.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"coverage\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"analyzed\","]
#[doc = "        \"end_ticks\","]
#[doc = "        \"start_ticks\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"analyzed\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"end_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"start_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"detection\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"hop_ticks\","]
#[doc = "        \"thresholds\","]
#[doc = "        \"window_ticks\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"hop_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"thresholds\": {"]
#[doc = "          \"description\": \"Start and continuation scores for the audio tagger's classes.\","]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"kind\","]
#[doc = "              \"off\","]
#[doc = "              \"on\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"kind\": {"]
#[doc = "                \"$ref\": \"#/$defs/kind\""]
#[doc = "              },"]
#[doc = "              \"off\": {"]
#[doc = "                \"type\": \"number\","]
#[doc = "                \"exclusiveMaximum\": 1.0,"]
#[doc = "                \"exclusiveMinimum\": 0.0"]
#[doc = "              },"]
#[doc = "              \"on\": {"]
#[doc = "                \"type\": \"number\","]
#[doc = "                \"exclusiveMaximum\": 1.0,"]
#[doc = "                \"exclusiveMinimum\": 0.0"]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"window_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"events\": {"]
#[doc = "      \"description\": \"In time order, then by kind.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"end_ticks\","]
#[doc = "          \"kind\","]
#[doc = "          \"mean\","]
#[doc = "          \"peak\","]
#[doc = "          \"start_ticks\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"delivery\": {"]
#[doc = "            \"description\": \"Measurements on the voiced frames of an emphasis event. Changes are relative to the preceding local median.\","]
#[doc = "            \"type\": \"object\","]
#[doc = "            \"required\": ["]
#[doc = "              \"level_change_db\","]
#[doc = "              \"level_dbfs\","]
#[doc = "              \"pitch_change_semitones\","]
#[doc = "              \"pitch_hz\""]
#[doc = "            ],"]
#[doc = "            \"properties\": {"]
#[doc = "              \"level_change_db\": {"]
#[doc = "                \"type\": \"number\""]
#[doc = "              },"]
#[doc = "              \"level_dbfs\": {"]
#[doc = "                \"type\": \"number\","]
#[doc = "                \"maximum\": 0.0"]
#[doc = "              },"]
#[doc = "              \"pitch_change_semitones\": {"]
#[doc = "                \"type\": \"number\""]
#[doc = "              },"]
#[doc = "              \"pitch_hz\": {"]
#[doc = "                \"type\": \"number\","]
#[doc = "                \"exclusiveMinimum\": 0.0"]
#[doc = "              }"]
#[doc = "            },"]
#[doc = "            \"additionalProperties\": false"]
#[doc = "          },"]
#[doc = "          \"end_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 1.0"]
#[doc = "          },"]
#[doc = "          \"kind\": {"]
#[doc = "            \"$ref\": \"#/$defs/kind\""]
#[doc = "          },"]
#[doc = "          \"mean\": {"]
#[doc = "            \"description\": \"Mean tagger score, or mean normalized periodicity for an emphasis event.\","]
#[doc = "            \"type\": \"number\","]
#[doc = "            \"maximum\": 1.0,"]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"peak\": {"]
#[doc = "            \"description\": \"Highest tagger score, or highest normalized periodicity for an emphasis event.\","]
#[doc = "            \"type\": \"number\","]
#[doc = "            \"maximum\": 1.0,"]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"start_ticks\": {"]
#[doc = "            \"type\": \"integer\","]
#[doc = "            \"minimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"producer\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"implementation\","]
#[doc = "        \"model_digest\","]
#[doc = "        \"stage\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"implementation\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"model_digest\": {"]
#[doc = "          \"$ref\": \"#/$defs/sha256\""]
#[doc = "        },"]
#[doc = "        \"stage\": {"]
#[doc = "          \"const\": \"audio-events\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.audio.events.v1\""]
#[doc = "    },"]
#[doc = "    \"source_fingerprint\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"vad_artifact_id\": {"]
#[doc = "      \"description\": \"Voice-activity evidence used to measure emphasis. Absent on earlier recordings whose producer only tagged sounds.\","]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AudioEvents {
    #[doc = "The 16 kHz mono rendition this pass read, verified before use."]
    pub audio_artifact_id: Sha256,
    pub coverage: AudioEventsCoverage,
    pub detection: AudioEventsDetection,
    #[doc = "In time order, then by kind."]
    pub events: ::std::vec::Vec<AudioEventsEventsItem>,
    pub producer: AudioEventsProducer,
    pub schema_version: ::serde_json::Value,
    pub source_fingerprint: Sha256,
    #[doc = "Voice-activity evidence used to measure emphasis. Absent on earlier recordings whose producer only tagged sounds."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub vad_artifact_id: ::std::option::Option<Sha256>,
}
impl AudioEvents {
    pub fn builder() -> builder::AudioEvents {
        Default::default()
    }
}
#[doc = "`AudioEventsCoverage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"analyzed\","]
#[doc = "    \"end_ticks\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"analyzed\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
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
pub struct AudioEventsCoverage {
    pub analyzed: bool,
    pub end_ticks: u64,
    pub start_ticks: u64,
}
impl AudioEventsCoverage {
    pub fn builder() -> builder::AudioEventsCoverage {
        Default::default()
    }
}
#[doc = "`AudioEventsDetection`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"hop_ticks\","]
#[doc = "    \"thresholds\","]
#[doc = "    \"window_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"hop_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"thresholds\": {"]
#[doc = "      \"description\": \"Start and continuation scores for the audio tagger's classes.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\","]
#[doc = "        \"required\": ["]
#[doc = "          \"kind\","]
#[doc = "          \"off\","]
#[doc = "          \"on\""]
#[doc = "        ],"]
#[doc = "        \"properties\": {"]
#[doc = "          \"kind\": {"]
#[doc = "            \"$ref\": \"#/$defs/kind\""]
#[doc = "          },"]
#[doc = "          \"off\": {"]
#[doc = "            \"type\": \"number\","]
#[doc = "            \"exclusiveMaximum\": 1.0,"]
#[doc = "            \"exclusiveMinimum\": 0.0"]
#[doc = "          },"]
#[doc = "          \"on\": {"]
#[doc = "            \"type\": \"number\","]
#[doc = "            \"exclusiveMaximum\": 1.0,"]
#[doc = "            \"exclusiveMinimum\": 0.0"]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        \"additionalProperties\": false"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"window_ticks\": {"]
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
pub struct AudioEventsDetection {
    pub hop_ticks: ::std::num::NonZeroU64,
    #[doc = "Start and continuation scores for the audio tagger's classes."]
    pub thresholds: ::std::vec::Vec<AudioEventsDetectionThresholdsItem>,
    pub window_ticks: ::std::num::NonZeroU64,
}
impl AudioEventsDetection {
    pub fn builder() -> builder::AudioEventsDetection {
        Default::default()
    }
}
#[doc = "`AudioEventsDetectionThresholdsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"kind\","]
#[doc = "    \"off\","]
#[doc = "    \"on\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"kind\": {"]
#[doc = "      \"$ref\": \"#/$defs/kind\""]
#[doc = "    },"]
#[doc = "    \"off\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"exclusiveMaximum\": 1.0,"]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"on\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"exclusiveMaximum\": 1.0,"]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AudioEventsDetectionThresholdsItem {
    pub kind: Kind,
    pub off: f64,
    pub on: f64,
}
impl AudioEventsDetectionThresholdsItem {
    pub fn builder() -> builder::AudioEventsDetectionThresholdsItem {
        Default::default()
    }
}
#[doc = "`AudioEventsEventsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"end_ticks\","]
#[doc = "    \"kind\","]
#[doc = "    \"mean\","]
#[doc = "    \"peak\","]
#[doc = "    \"start_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"delivery\": {"]
#[doc = "      \"description\": \"Measurements on the voiced frames of an emphasis event. Changes are relative to the preceding local median.\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"level_change_db\","]
#[doc = "        \"level_dbfs\","]
#[doc = "        \"pitch_change_semitones\","]
#[doc = "        \"pitch_hz\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"level_change_db\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"level_dbfs\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"maximum\": 0.0"]
#[doc = "        },"]
#[doc = "        \"pitch_change_semitones\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"pitch_hz\": {"]
#[doc = "          \"type\": \"number\","]
#[doc = "          \"exclusiveMinimum\": 0.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"end_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"kind\": {"]
#[doc = "      \"$ref\": \"#/$defs/kind\""]
#[doc = "    },"]
#[doc = "    \"mean\": {"]
#[doc = "      \"description\": \"Mean tagger score, or mean normalized periodicity for an emphasis event.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"peak\": {"]
#[doc = "      \"description\": \"Highest tagger score, or highest normalized periodicity for an emphasis event.\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
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
pub struct AudioEventsEventsItem {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub delivery: ::std::option::Option<AudioEventsEventsItemDelivery>,
    pub end_ticks: ::std::num::NonZeroU64,
    pub kind: Kind,
    #[doc = "Mean tagger score, or mean normalized periodicity for an emphasis event."]
    pub mean: f64,
    #[doc = "Highest tagger score, or highest normalized periodicity for an emphasis event."]
    pub peak: f64,
    pub start_ticks: u64,
}
impl AudioEventsEventsItem {
    pub fn builder() -> builder::AudioEventsEventsItem {
        Default::default()
    }
}
#[doc = "Measurements on the voiced frames of an emphasis event. Changes are relative to the preceding local median."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Measurements on the voiced frames of an emphasis event. Changes are relative to the preceding local median.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"level_change_db\","]
#[doc = "    \"level_dbfs\","]
#[doc = "    \"pitch_change_semitones\","]
#[doc = "    \"pitch_hz\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"level_change_db\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"level_dbfs\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"pitch_change_semitones\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"pitch_hz\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"exclusiveMinimum\": 0.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AudioEventsEventsItemDelivery {
    pub level_change_db: f64,
    pub level_dbfs: f64,
    pub pitch_change_semitones: f64,
    pub pitch_hz: f64,
}
impl AudioEventsEventsItemDelivery {
    pub fn builder() -> builder::AudioEventsEventsItemDelivery {
        Default::default()
    }
}
#[doc = "`AudioEventsProducer`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"implementation\","]
#[doc = "    \"model_digest\","]
#[doc = "    \"stage\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"implementation\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"model_digest\": {"]
#[doc = "      \"$ref\": \"#/$defs/sha256\""]
#[doc = "    },"]
#[doc = "    \"stage\": {"]
#[doc = "      \"const\": \"audio-events\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AudioEventsProducer {
    pub implementation: AudioEventsProducerImplementation,
    pub model_digest: Sha256,
    pub stage: ::serde_json::Value,
}
impl AudioEventsProducer {
    pub fn builder() -> builder::AudioEventsProducer {
        Default::default()
    }
}
#[doc = "`AudioEventsProducerImplementation`"]
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
pub struct AudioEventsProducerImplementation(::std::string::String);
impl ::std::ops::Deref for AudioEventsProducerImplementation {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<AudioEventsProducerImplementation> for ::std::string::String {
    fn from(value: AudioEventsProducerImplementation) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for AudioEventsProducerImplementation {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for AudioEventsProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for AudioEventsProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for AudioEventsProducerImplementation {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for AudioEventsProducerImplementation {
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
#[doc = "`Kind`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"laughter\","]
#[doc = "    \"applause\","]
#[doc = "    \"cheering\","]
#[doc = "    \"music\","]
#[doc = "    \"shout\","]
#[doc = "    \"gasp\","]
#[doc = "    \"emphasis\""]
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
pub enum Kind {
    #[serde(rename = "laughter")]
    Laughter,
    #[serde(rename = "applause")]
    Applause,
    #[serde(rename = "cheering")]
    Cheering,
    #[serde(rename = "music")]
    Music,
    #[serde(rename = "shout")]
    Shout,
    #[serde(rename = "gasp")]
    Gasp,
    #[serde(rename = "emphasis")]
    Emphasis,
}
impl ::std::fmt::Display for Kind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Laughter => f.write_str("laughter"),
            Self::Applause => f.write_str("applause"),
            Self::Cheering => f.write_str("cheering"),
            Self::Music => f.write_str("music"),
            Self::Shout => f.write_str("shout"),
            Self::Gasp => f.write_str("gasp"),
            Self::Emphasis => f.write_str("emphasis"),
        }
    }
}
impl ::std::str::FromStr for Kind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "laughter" => Ok(Self::Laughter),
            "applause" => Ok(Self::Applause),
            "cheering" => Ok(Self::Cheering),
            "music" => Ok(Self::Music),
            "shout" => Ok(Self::Shout),
            "gasp" => Ok(Self::Gasp),
            "emphasis" => Ok(Self::Emphasis),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Kind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Kind {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Kind {
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
    pub struct AudioEvents {
        audio_artifact_id: ::std::result::Result<super::Sha256, ::std::string::String>,
        coverage: ::std::result::Result<super::AudioEventsCoverage, ::std::string::String>,
        detection: ::std::result::Result<super::AudioEventsDetection, ::std::string::String>,
        events: ::std::result::Result<
            ::std::vec::Vec<super::AudioEventsEventsItem>,
            ::std::string::String,
        >,
        producer: ::std::result::Result<super::AudioEventsProducer, ::std::string::String>,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
        source_fingerprint: ::std::result::Result<super::Sha256, ::std::string::String>,
        vad_artifact_id:
            ::std::result::Result<::std::option::Option<super::Sha256>, ::std::string::String>,
    }
    impl ::std::default::Default for AudioEvents {
        fn default() -> Self {
            Self {
                audio_artifact_id: Err("no value supplied for audio_artifact_id".to_string()),
                coverage: Err("no value supplied for coverage".to_string()),
                detection: Err("no value supplied for detection".to_string()),
                events: Err("no value supplied for events".to_string()),
                producer: Err("no value supplied for producer".to_string()),
                schema_version: Err("no value supplied for schema_version".to_string()),
                source_fingerprint: Err("no value supplied for source_fingerprint".to_string()),
                vad_artifact_id: Ok(Default::default()),
            }
        }
    }
    impl AudioEvents {
        pub fn audio_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.audio_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for audio_artifact_id: {e}"));
            self
        }
        pub fn coverage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AudioEventsCoverage>,
            T::Error: ::std::fmt::Display,
        {
            self.coverage = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for coverage: {e}"));
            self
        }
        pub fn detection<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AudioEventsDetection>,
            T::Error: ::std::fmt::Display,
        {
            self.detection = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for detection: {e}"));
            self
        }
        pub fn events<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::AudioEventsEventsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.events = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for events: {e}"));
            self
        }
        pub fn producer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AudioEventsProducer>,
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
        pub fn vad_artifact_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Sha256>>,
            T::Error: ::std::fmt::Display,
        {
            self.vad_artifact_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for vad_artifact_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AudioEvents> for super::AudioEvents {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AudioEvents,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                audio_artifact_id: value.audio_artifact_id?,
                coverage: value.coverage?,
                detection: value.detection?,
                events: value.events?,
                producer: value.producer?,
                schema_version: value.schema_version?,
                source_fingerprint: value.source_fingerprint?,
                vad_artifact_id: value.vad_artifact_id?,
            })
        }
    }
    impl ::std::convert::From<super::AudioEvents> for AudioEvents {
        fn from(value: super::AudioEvents) -> Self {
            Self {
                audio_artifact_id: Ok(value.audio_artifact_id),
                coverage: Ok(value.coverage),
                detection: Ok(value.detection),
                events: Ok(value.events),
                producer: Ok(value.producer),
                schema_version: Ok(value.schema_version),
                source_fingerprint: Ok(value.source_fingerprint),
                vad_artifact_id: Ok(value.vad_artifact_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AudioEventsCoverage {
        analyzed: ::std::result::Result<bool, ::std::string::String>,
        end_ticks: ::std::result::Result<u64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for AudioEventsCoverage {
        fn default() -> Self {
            Self {
                analyzed: Err("no value supplied for analyzed".to_string()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl AudioEventsCoverage {
        pub fn analyzed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.analyzed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for analyzed: {e}"));
            self
        }
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
    impl ::std::convert::TryFrom<AudioEventsCoverage> for super::AudioEventsCoverage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AudioEventsCoverage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                analyzed: value.analyzed?,
                end_ticks: value.end_ticks?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::AudioEventsCoverage> for AudioEventsCoverage {
        fn from(value: super::AudioEventsCoverage) -> Self {
            Self {
                analyzed: Ok(value.analyzed),
                end_ticks: Ok(value.end_ticks),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AudioEventsDetection {
        hop_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        thresholds: ::std::result::Result<
            ::std::vec::Vec<super::AudioEventsDetectionThresholdsItem>,
            ::std::string::String,
        >,
        window_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for AudioEventsDetection {
        fn default() -> Self {
            Self {
                hop_ticks: Err("no value supplied for hop_ticks".to_string()),
                thresholds: Err("no value supplied for thresholds".to_string()),
                window_ticks: Err("no value supplied for window_ticks".to_string()),
            }
        }
    }
    impl AudioEventsDetection {
        pub fn hop_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.hop_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hop_ticks: {e}"));
            self
        }
        pub fn thresholds<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::AudioEventsDetectionThresholdsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.thresholds = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for thresholds: {e}"));
            self
        }
        pub fn window_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.window_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for window_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AudioEventsDetection> for super::AudioEventsDetection {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AudioEventsDetection,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                hop_ticks: value.hop_ticks?,
                thresholds: value.thresholds?,
                window_ticks: value.window_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::AudioEventsDetection> for AudioEventsDetection {
        fn from(value: super::AudioEventsDetection) -> Self {
            Self {
                hop_ticks: Ok(value.hop_ticks),
                thresholds: Ok(value.thresholds),
                window_ticks: Ok(value.window_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AudioEventsDetectionThresholdsItem {
        kind: ::std::result::Result<super::Kind, ::std::string::String>,
        off: ::std::result::Result<f64, ::std::string::String>,
        on: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for AudioEventsDetectionThresholdsItem {
        fn default() -> Self {
            Self {
                kind: Err("no value supplied for kind".to_string()),
                off: Err("no value supplied for off".to_string()),
                on: Err("no value supplied for on".to_string()),
            }
        }
    }
    impl AudioEventsDetectionThresholdsItem {
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Kind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn off<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.off = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for off: {e}"));
            self
        }
        pub fn on<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.on = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for on: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AudioEventsDetectionThresholdsItem>
        for super::AudioEventsDetectionThresholdsItem
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AudioEventsDetectionThresholdsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                kind: value.kind?,
                off: value.off?,
                on: value.on?,
            })
        }
    }
    impl ::std::convert::From<super::AudioEventsDetectionThresholdsItem>
        for AudioEventsDetectionThresholdsItem
    {
        fn from(value: super::AudioEventsDetectionThresholdsItem) -> Self {
            Self {
                kind: Ok(value.kind),
                off: Ok(value.off),
                on: Ok(value.on),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AudioEventsEventsItem {
        delivery: ::std::result::Result<
            ::std::option::Option<super::AudioEventsEventsItemDelivery>,
            ::std::string::String,
        >,
        end_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        kind: ::std::result::Result<super::Kind, ::std::string::String>,
        mean: ::std::result::Result<f64, ::std::string::String>,
        peak: ::std::result::Result<f64, ::std::string::String>,
        start_ticks: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for AudioEventsEventsItem {
        fn default() -> Self {
            Self {
                delivery: Ok(Default::default()),
                end_ticks: Err("no value supplied for end_ticks".to_string()),
                kind: Err("no value supplied for kind".to_string()),
                mean: Err("no value supplied for mean".to_string()),
                peak: Err("no value supplied for peak".to_string()),
                start_ticks: Err("no value supplied for start_ticks".to_string()),
            }
        }
    }
    impl AudioEventsEventsItem {
        pub fn delivery<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::AudioEventsEventsItemDelivery>>,
            T::Error: ::std::fmt::Display,
        {
            self.delivery = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for delivery: {e}"));
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
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Kind>,
            T::Error: ::std::fmt::Display,
        {
            self.kind = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for kind: {e}"));
            self
        }
        pub fn mean<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.mean = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mean: {e}"));
            self
        }
        pub fn peak<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.peak = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for peak: {e}"));
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
    impl ::std::convert::TryFrom<AudioEventsEventsItem> for super::AudioEventsEventsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AudioEventsEventsItem,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                delivery: value.delivery?,
                end_ticks: value.end_ticks?,
                kind: value.kind?,
                mean: value.mean?,
                peak: value.peak?,
                start_ticks: value.start_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::AudioEventsEventsItem> for AudioEventsEventsItem {
        fn from(value: super::AudioEventsEventsItem) -> Self {
            Self {
                delivery: Ok(value.delivery),
                end_ticks: Ok(value.end_ticks),
                kind: Ok(value.kind),
                mean: Ok(value.mean),
                peak: Ok(value.peak),
                start_ticks: Ok(value.start_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AudioEventsEventsItemDelivery {
        level_change_db: ::std::result::Result<f64, ::std::string::String>,
        level_dbfs: ::std::result::Result<f64, ::std::string::String>,
        pitch_change_semitones: ::std::result::Result<f64, ::std::string::String>,
        pitch_hz: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for AudioEventsEventsItemDelivery {
        fn default() -> Self {
            Self {
                level_change_db: Err("no value supplied for level_change_db".to_string()),
                level_dbfs: Err("no value supplied for level_dbfs".to_string()),
                pitch_change_semitones: Err(
                    "no value supplied for pitch_change_semitones".to_string()
                ),
                pitch_hz: Err("no value supplied for pitch_hz".to_string()),
            }
        }
    }
    impl AudioEventsEventsItemDelivery {
        pub fn level_change_db<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.level_change_db = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for level_change_db: {e}"));
            self
        }
        pub fn level_dbfs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.level_dbfs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for level_dbfs: {e}"));
            self
        }
        pub fn pitch_change_semitones<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.pitch_change_semitones = value.try_into().map_err(|e| {
                format!("error converting supplied value for pitch_change_semitones: {e}")
            });
            self
        }
        pub fn pitch_hz<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.pitch_hz = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for pitch_hz: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AudioEventsEventsItemDelivery>
        for super::AudioEventsEventsItemDelivery
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AudioEventsEventsItemDelivery,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                level_change_db: value.level_change_db?,
                level_dbfs: value.level_dbfs?,
                pitch_change_semitones: value.pitch_change_semitones?,
                pitch_hz: value.pitch_hz?,
            })
        }
    }
    impl ::std::convert::From<super::AudioEventsEventsItemDelivery> for AudioEventsEventsItemDelivery {
        fn from(value: super::AudioEventsEventsItemDelivery) -> Self {
            Self {
                level_change_db: Ok(value.level_change_db),
                level_dbfs: Ok(value.level_dbfs),
                pitch_change_semitones: Ok(value.pitch_change_semitones),
                pitch_hz: Ok(value.pitch_hz),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AudioEventsProducer {
        implementation:
            ::std::result::Result<super::AudioEventsProducerImplementation, ::std::string::String>,
        model_digest: ::std::result::Result<super::Sha256, ::std::string::String>,
        stage: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for AudioEventsProducer {
        fn default() -> Self {
            Self {
                implementation: Err("no value supplied for implementation".to_string()),
                model_digest: Err("no value supplied for model_digest".to_string()),
                stage: Err("no value supplied for stage".to_string()),
            }
        }
    }
    impl AudioEventsProducer {
        pub fn implementation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AudioEventsProducerImplementation>,
            T::Error: ::std::fmt::Display,
        {
            self.implementation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for implementation: {e}"));
            self
        }
        pub fn model_digest<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Sha256>,
            T::Error: ::std::fmt::Display,
        {
            self.model_digest = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for model_digest: {e}"));
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
    impl ::std::convert::TryFrom<AudioEventsProducer> for super::AudioEventsProducer {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AudioEventsProducer,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                implementation: value.implementation?,
                model_digest: value.model_digest?,
                stage: value.stage?,
            })
        }
    }
    impl ::std::convert::From<super::AudioEventsProducer> for AudioEventsProducer {
        fn from(value: super::AudioEventsProducer) -> Self {
            Self {
                implementation: Ok(value.implementation),
                model_digest: Ok(value.model_digest),
                stage: Ok(value.stage),
            }
        }
    }
}
