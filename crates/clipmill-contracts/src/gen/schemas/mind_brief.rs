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
#[doc = "What the person wants made from a recording: the goal, where it will be posted, how long, what it should be about and avoid, which story forms are allowed, and how much production. The scorer and the planner read it; every field the person did not choose is a default named in `inferred`, so the Studio can show it as a suggestion rather than a decision. Times are integer ticks at 1/90000 s."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://clipmill.dev/schemas/clipmill.mind.brief.v1.json\","]
#[doc = "  \"title\": \"MindBrief\","]
#[doc = "  \"description\": \"What the person wants made from a recording: the goal, where it will be posted, how long, what it should be about and avoid, which story forms are allowed, and how much production. The scorer and the planner read it; every field the person did not choose is a default named in `inferred`, so the Studio can show it as a suggestion rather than a decision. Times are integer ticks at 1/90000 s.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"aspect\","]
#[doc = "    \"content\","]
#[doc = "    \"count\","]
#[doc = "    \"forms\","]
#[doc = "    \"generation\","]
#[doc = "    \"genre\","]
#[doc = "    \"goal\","]
#[doc = "    \"inferred\","]
#[doc = "    \"length\","]
#[doc = "    \"platforms\","]
#[doc = "    \"production\","]
#[doc = "    \"schema_version\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"aspect\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"vertical\","]
#[doc = "        \"portrait\","]
#[doc = "        \"square\","]
#[doc = "        \"landscape\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"content\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"properties\": {"]
#[doc = "        \"avoid\": {"]
#[doc = "          \"description\": \"Subjects, words or passages the clips must not contain, such as sponsor reads.\","]
#[doc = "          \"$ref\": \"#/$defs/phrases\""]
#[doc = "        },"]
#[doc = "        \"include\": {"]
#[doc = "          \"$ref\": \"#/$defs/phrases\""]
#[doc = "        },"]
#[doc = "        \"moments\": {"]
#[doc = "          \"description\": \"Moments asked for in the person's words, such as `every time they laugh`.\","]
#[doc = "          \"$ref\": \"#/$defs/phrases\""]
#[doc = "        },"]
#[doc = "        \"mood\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"enum\": ["]
#[doc = "              \"funny\","]
#[doc = "              \"emotional\","]
#[doc = "              \"inspiring\","]
#[doc = "              \"controversial\","]
#[doc = "              \"informative\","]
#[doc = "              \"dramatic\","]
#[doc = "              \"calm\""]
#[doc = "            ]"]
#[doc = "          },"]
#[doc = "          \"maxItems\": 7"]
#[doc = "        },"]
#[doc = "        \"people\": {"]
#[doc = "          \"$ref\": \"#/$defs/phrases\""]
#[doc = "        },"]
#[doc = "        \"topics\": {"]
#[doc = "          \"$ref\": \"#/$defs/phrases\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"count\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"target\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"all_worthwhile\": {"]
#[doc = "          \"description\": \"Return every clip worth posting, more or fewer than the target.\","]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"target\": {"]
#[doc = "          \"description\": \"How many clips to aim for.\","]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 100.0,"]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"forms\": {"]
#[doc = "      \"description\": \"The story forms the planner may use.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"enum\": ["]
#[doc = "          \"straight\","]
#[doc = "          \"tightened\","]
#[doc = "          \"stitch\","]
#[doc = "          \"cold_open\","]
#[doc = "          \"theme_montage\""]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      \"maxItems\": 5,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"generation\": {"]
#[doc = "      \"description\": \"Which generated media the person allows; none is ever required (decision R79).\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"images\","]
#[doc = "        \"video\","]
#[doc = "        \"voice\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"images\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"video\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"voice\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"genre\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"podcast\","]
#[doc = "        \"interview\","]
#[doc = "        \"talk\","]
#[doc = "        \"tutorial\","]
#[doc = "        \"vlog\","]
#[doc = "        \"sports\","]
#[doc = "        \"gaming\","]
#[doc = "        \"scripted\","]
#[doc = "        \"news\","]
#[doc = "        \"music\","]
#[doc = "        \"other\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"goal\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"highlights\","]
#[doc = "        \"teaser\","]
#[doc = "        \"compilation\","]
#[doc = "        \"extended_cut\","]
#[doc = "        \"tutorial\","]
#[doc = "        \"announcement\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"inferred\": {"]
#[doc = "      \"description\": \"The top-level fields filled with a default rather than chosen.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"enum\": ["]
#[doc = "          \"goal\","]
#[doc = "          \"genre\","]
#[doc = "          \"platforms\","]
#[doc = "          \"aspect\","]
#[doc = "          \"language\","]
#[doc = "          \"length\","]
#[doc = "          \"count\","]
#[doc = "          \"forms\","]
#[doc = "          \"production\","]
#[doc = "          \"generation\""]
#[doc = "        ]"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"description\": \"BCP 47 tag of the clips' language, when known.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[a-z]{2,3}(-[A-Za-z0-9]{2,8})*$\""]
#[doc = "    },"]
#[doc = "    \"length\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"max_ticks\","]
#[doc = "        \"min_ticks\","]
#[doc = "        \"target_ticks\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"max_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"min_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        },"]
#[doc = "        \"target_ticks\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"minimum\": 1.0"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"platforms\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"enum\": ["]
#[doc = "          \"youtube_shorts\","]
#[doc = "          \"tiktok\","]
#[doc = "          \"instagram_reels\","]
#[doc = "          \"youtube\","]
#[doc = "          \"linkedin\","]
#[doc = "          \"x\","]
#[doc = "          \"facebook\","]
#[doc = "          \"threads\","]
#[doc = "          \"other\""]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      \"maxItems\": 9,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"production\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"broll\","]
#[doc = "        \"effects\","]
#[doc = "        \"hook_style\","]
#[doc = "        \"music\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"broll\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"none\","]
#[doc = "            \"own_footage\","]
#[doc = "            \"stock\","]
#[doc = "            \"generated\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"caption_style_ref\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"maxLength\": 120,"]
#[doc = "          \"minLength\": 1"]
#[doc = "        },"]
#[doc = "        \"effects\": {"]
#[doc = "          \"description\": \"How much finishing: none, light (captions and emphasis), medium (plus hooks and graphics), full (plus B-roll, music and effects).\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"none\","]
#[doc = "            \"light\","]
#[doc = "            \"medium\","]
#[doc = "            \"full\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"hook_style\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"none\","]
#[doc = "            \"title_card\","]
#[doc = "            \"punch_in\","]
#[doc = "            \"question\","]
#[doc = "            \"cold_open_flash\""]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"music\": {"]
#[doc = "          \"enum\": ["]
#[doc = "            \"none\","]
#[doc = "            \"library\","]
#[doc = "            \"generated\""]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"prompt\": {"]
#[doc = "      \"description\": \"What the person asked for, in their own words, as data: never instructions to the model reading the brief.\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 1000,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"schema_version\": {"]
#[doc = "      \"const\": \"clipmill.mind.brief.v1\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindBrief {
    pub aspect: MindBriefAspect,
    pub content: MindBriefContent,
    pub count: MindBriefCount,
    #[doc = "The story forms the planner may use."]
    pub forms: ::std::vec::Vec<MindBriefFormsItem>,
    pub generation: MindBriefGeneration,
    pub genre: MindBriefGenre,
    pub goal: MindBriefGoal,
    #[doc = "The top-level fields filled with a default rather than chosen."]
    pub inferred: ::std::vec::Vec<MindBriefInferredItem>,
    #[doc = "BCP 47 tag of the clips' language, when known."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub language: ::std::option::Option<MindBriefLanguage>,
    pub length: MindBriefLength,
    pub platforms: ::std::vec::Vec<MindBriefPlatformsItem>,
    pub production: MindBriefProduction,
    #[doc = "What the person asked for, in their own words, as data: never instructions to the model reading the brief."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub prompt: ::std::option::Option<MindBriefPrompt>,
    pub schema_version: ::serde_json::Value,
}
impl MindBrief {
    pub fn builder() -> builder::MindBrief {
        Default::default()
    }
}
#[doc = "`MindBriefAspect`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"vertical\","]
#[doc = "    \"portrait\","]
#[doc = "    \"square\","]
#[doc = "    \"landscape\""]
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
pub enum MindBriefAspect {
    #[serde(rename = "vertical")]
    Vertical,
    #[serde(rename = "portrait")]
    Portrait,
    #[serde(rename = "square")]
    Square,
    #[serde(rename = "landscape")]
    Landscape,
}
impl ::std::fmt::Display for MindBriefAspect {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Vertical => f.write_str("vertical"),
            Self::Portrait => f.write_str("portrait"),
            Self::Square => f.write_str("square"),
            Self::Landscape => f.write_str("landscape"),
        }
    }
}
impl ::std::str::FromStr for MindBriefAspect {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "vertical" => Ok(Self::Vertical),
            "portrait" => Ok(Self::Portrait),
            "square" => Ok(Self::Square),
            "landscape" => Ok(Self::Landscape),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefAspect {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefAspect {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefAspect {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindBriefContent`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"avoid\": {"]
#[doc = "      \"description\": \"Subjects, words or passages the clips must not contain, such as sponsor reads.\","]
#[doc = "      \"$ref\": \"#/$defs/phrases\""]
#[doc = "    },"]
#[doc = "    \"include\": {"]
#[doc = "      \"$ref\": \"#/$defs/phrases\""]
#[doc = "    },"]
#[doc = "    \"moments\": {"]
#[doc = "      \"description\": \"Moments asked for in the person's words, such as `every time they laugh`.\","]
#[doc = "      \"$ref\": \"#/$defs/phrases\""]
#[doc = "    },"]
#[doc = "    \"mood\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"enum\": ["]
#[doc = "          \"funny\","]
#[doc = "          \"emotional\","]
#[doc = "          \"inspiring\","]
#[doc = "          \"controversial\","]
#[doc = "          \"informative\","]
#[doc = "          \"dramatic\","]
#[doc = "          \"calm\""]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      \"maxItems\": 7"]
#[doc = "    },"]
#[doc = "    \"people\": {"]
#[doc = "      \"$ref\": \"#/$defs/phrases\""]
#[doc = "    },"]
#[doc = "    \"topics\": {"]
#[doc = "      \"$ref\": \"#/$defs/phrases\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindBriefContent {
    #[doc = "Subjects, words or passages the clips must not contain, such as sponsor reads."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub avoid: ::std::option::Option<Phrases>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub include: ::std::option::Option<Phrases>,
    #[doc = "Moments asked for in the person's words, such as `every time they laugh`."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub moments: ::std::option::Option<Phrases>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub mood: ::std::vec::Vec<MindBriefContentMoodItem>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub people: ::std::option::Option<Phrases>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub topics: ::std::option::Option<Phrases>,
}
impl ::std::default::Default for MindBriefContent {
    fn default() -> Self {
        Self {
            avoid: Default::default(),
            include: Default::default(),
            moments: Default::default(),
            mood: Default::default(),
            people: Default::default(),
            topics: Default::default(),
        }
    }
}
impl MindBriefContent {
    pub fn builder() -> builder::MindBriefContent {
        Default::default()
    }
}
#[doc = "`MindBriefContentMoodItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"funny\","]
#[doc = "    \"emotional\","]
#[doc = "    \"inspiring\","]
#[doc = "    \"controversial\","]
#[doc = "    \"informative\","]
#[doc = "    \"dramatic\","]
#[doc = "    \"calm\""]
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
pub enum MindBriefContentMoodItem {
    #[serde(rename = "funny")]
    Funny,
    #[serde(rename = "emotional")]
    Emotional,
    #[serde(rename = "inspiring")]
    Inspiring,
    #[serde(rename = "controversial")]
    Controversial,
    #[serde(rename = "informative")]
    Informative,
    #[serde(rename = "dramatic")]
    Dramatic,
    #[serde(rename = "calm")]
    Calm,
}
impl ::std::fmt::Display for MindBriefContentMoodItem {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Funny => f.write_str("funny"),
            Self::Emotional => f.write_str("emotional"),
            Self::Inspiring => f.write_str("inspiring"),
            Self::Controversial => f.write_str("controversial"),
            Self::Informative => f.write_str("informative"),
            Self::Dramatic => f.write_str("dramatic"),
            Self::Calm => f.write_str("calm"),
        }
    }
}
impl ::std::str::FromStr for MindBriefContentMoodItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "funny" => Ok(Self::Funny),
            "emotional" => Ok(Self::Emotional),
            "inspiring" => Ok(Self::Inspiring),
            "controversial" => Ok(Self::Controversial),
            "informative" => Ok(Self::Informative),
            "dramatic" => Ok(Self::Dramatic),
            "calm" => Ok(Self::Calm),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefContentMoodItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefContentMoodItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefContentMoodItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindBriefCount`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"target\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"all_worthwhile\": {"]
#[doc = "      \"description\": \"Return every clip worth posting, more or fewer than the target.\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"target\": {"]
#[doc = "      \"description\": \"How many clips to aim for.\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 100.0,"]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindBriefCount {
    #[doc = "Return every clip worth posting, more or fewer than the target."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub all_worthwhile: ::std::option::Option<bool>,
    #[doc = "How many clips to aim for."]
    pub target: ::std::num::NonZeroU64,
}
impl MindBriefCount {
    pub fn builder() -> builder::MindBriefCount {
        Default::default()
    }
}
#[doc = "`MindBriefFormsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
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
pub enum MindBriefFormsItem {
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
impl ::std::fmt::Display for MindBriefFormsItem {
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
impl ::std::str::FromStr for MindBriefFormsItem {
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
impl ::std::convert::TryFrom<&str> for MindBriefFormsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefFormsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefFormsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Which generated media the person allows; none is ever required (decision R79)."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Which generated media the person allows; none is ever required (decision R79).\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"images\","]
#[doc = "    \"video\","]
#[doc = "    \"voice\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"images\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"video\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"voice\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindBriefGeneration {
    pub images: bool,
    pub video: bool,
    pub voice: bool,
}
impl MindBriefGeneration {
    pub fn builder() -> builder::MindBriefGeneration {
        Default::default()
    }
}
#[doc = "`MindBriefGenre`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"podcast\","]
#[doc = "    \"interview\","]
#[doc = "    \"talk\","]
#[doc = "    \"tutorial\","]
#[doc = "    \"vlog\","]
#[doc = "    \"sports\","]
#[doc = "    \"gaming\","]
#[doc = "    \"scripted\","]
#[doc = "    \"news\","]
#[doc = "    \"music\","]
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
pub enum MindBriefGenre {
    #[serde(rename = "podcast")]
    Podcast,
    #[serde(rename = "interview")]
    Interview,
    #[serde(rename = "talk")]
    Talk,
    #[serde(rename = "tutorial")]
    Tutorial,
    #[serde(rename = "vlog")]
    Vlog,
    #[serde(rename = "sports")]
    Sports,
    #[serde(rename = "gaming")]
    Gaming,
    #[serde(rename = "scripted")]
    Scripted,
    #[serde(rename = "news")]
    News,
    #[serde(rename = "music")]
    Music,
    #[serde(rename = "other")]
    Other,
}
impl ::std::fmt::Display for MindBriefGenre {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Podcast => f.write_str("podcast"),
            Self::Interview => f.write_str("interview"),
            Self::Talk => f.write_str("talk"),
            Self::Tutorial => f.write_str("tutorial"),
            Self::Vlog => f.write_str("vlog"),
            Self::Sports => f.write_str("sports"),
            Self::Gaming => f.write_str("gaming"),
            Self::Scripted => f.write_str("scripted"),
            Self::News => f.write_str("news"),
            Self::Music => f.write_str("music"),
            Self::Other => f.write_str("other"),
        }
    }
}
impl ::std::str::FromStr for MindBriefGenre {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "podcast" => Ok(Self::Podcast),
            "interview" => Ok(Self::Interview),
            "talk" => Ok(Self::Talk),
            "tutorial" => Ok(Self::Tutorial),
            "vlog" => Ok(Self::Vlog),
            "sports" => Ok(Self::Sports),
            "gaming" => Ok(Self::Gaming),
            "scripted" => Ok(Self::Scripted),
            "news" => Ok(Self::News),
            "music" => Ok(Self::Music),
            "other" => Ok(Self::Other),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefGenre {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefGenre {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefGenre {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindBriefGoal`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"highlights\","]
#[doc = "    \"teaser\","]
#[doc = "    \"compilation\","]
#[doc = "    \"extended_cut\","]
#[doc = "    \"tutorial\","]
#[doc = "    \"announcement\""]
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
pub enum MindBriefGoal {
    #[serde(rename = "highlights")]
    Highlights,
    #[serde(rename = "teaser")]
    Teaser,
    #[serde(rename = "compilation")]
    Compilation,
    #[serde(rename = "extended_cut")]
    ExtendedCut,
    #[serde(rename = "tutorial")]
    Tutorial,
    #[serde(rename = "announcement")]
    Announcement,
}
impl ::std::fmt::Display for MindBriefGoal {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Highlights => f.write_str("highlights"),
            Self::Teaser => f.write_str("teaser"),
            Self::Compilation => f.write_str("compilation"),
            Self::ExtendedCut => f.write_str("extended_cut"),
            Self::Tutorial => f.write_str("tutorial"),
            Self::Announcement => f.write_str("announcement"),
        }
    }
}
impl ::std::str::FromStr for MindBriefGoal {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "highlights" => Ok(Self::Highlights),
            "teaser" => Ok(Self::Teaser),
            "compilation" => Ok(Self::Compilation),
            "extended_cut" => Ok(Self::ExtendedCut),
            "tutorial" => Ok(Self::Tutorial),
            "announcement" => Ok(Self::Announcement),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefGoal {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefGoal {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefGoal {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindBriefInferredItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"goal\","]
#[doc = "    \"genre\","]
#[doc = "    \"platforms\","]
#[doc = "    \"aspect\","]
#[doc = "    \"language\","]
#[doc = "    \"length\","]
#[doc = "    \"count\","]
#[doc = "    \"forms\","]
#[doc = "    \"production\","]
#[doc = "    \"generation\""]
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
pub enum MindBriefInferredItem {
    #[serde(rename = "goal")]
    Goal,
    #[serde(rename = "genre")]
    Genre,
    #[serde(rename = "platforms")]
    Platforms,
    #[serde(rename = "aspect")]
    Aspect,
    #[serde(rename = "language")]
    Language,
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "count")]
    Count,
    #[serde(rename = "forms")]
    Forms,
    #[serde(rename = "production")]
    Production,
    #[serde(rename = "generation")]
    Generation,
}
impl ::std::fmt::Display for MindBriefInferredItem {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Goal => f.write_str("goal"),
            Self::Genre => f.write_str("genre"),
            Self::Platforms => f.write_str("platforms"),
            Self::Aspect => f.write_str("aspect"),
            Self::Language => f.write_str("language"),
            Self::Length => f.write_str("length"),
            Self::Count => f.write_str("count"),
            Self::Forms => f.write_str("forms"),
            Self::Production => f.write_str("production"),
            Self::Generation => f.write_str("generation"),
        }
    }
}
impl ::std::str::FromStr for MindBriefInferredItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "goal" => Ok(Self::Goal),
            "genre" => Ok(Self::Genre),
            "platforms" => Ok(Self::Platforms),
            "aspect" => Ok(Self::Aspect),
            "language" => Ok(Self::Language),
            "length" => Ok(Self::Length),
            "count" => Ok(Self::Count),
            "forms" => Ok(Self::Forms),
            "production" => Ok(Self::Production),
            "generation" => Ok(Self::Generation),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefInferredItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefInferredItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefInferredItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "BCP 47 tag of the clips' language, when known."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"BCP 47 tag of the clips' language, when known.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[a-z]{2,3}(-[A-Za-z0-9]{2,8})*$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindBriefLanguage(::std::string::String);
impl ::std::ops::Deref for MindBriefLanguage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindBriefLanguage> for ::std::string::String {
    fn from(value: MindBriefLanguage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindBriefLanguage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| {
                ::regress::Regex::new("^[a-z]{2,3}(-[A-Za-z0-9]{2,8})*$").unwrap()
            });
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[a-z]{2,3}(-[A-Za-z0-9]{2,8})*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefLanguage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefLanguage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindBriefLanguage {
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
#[doc = "`MindBriefLength`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"max_ticks\","]
#[doc = "    \"min_ticks\","]
#[doc = "    \"target_ticks\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"max_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"min_ticks\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"target_ticks\": {"]
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
pub struct MindBriefLength {
    pub max_ticks: ::std::num::NonZeroU64,
    pub min_ticks: ::std::num::NonZeroU64,
    pub target_ticks: ::std::num::NonZeroU64,
}
impl MindBriefLength {
    pub fn builder() -> builder::MindBriefLength {
        Default::default()
    }
}
#[doc = "`MindBriefPlatformsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"youtube_shorts\","]
#[doc = "    \"tiktok\","]
#[doc = "    \"instagram_reels\","]
#[doc = "    \"youtube\","]
#[doc = "    \"linkedin\","]
#[doc = "    \"x\","]
#[doc = "    \"facebook\","]
#[doc = "    \"threads\","]
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
pub enum MindBriefPlatformsItem {
    #[serde(rename = "youtube_shorts")]
    YoutubeShorts,
    #[serde(rename = "tiktok")]
    Tiktok,
    #[serde(rename = "instagram_reels")]
    InstagramReels,
    #[serde(rename = "youtube")]
    Youtube,
    #[serde(rename = "linkedin")]
    Linkedin,
    #[serde(rename = "x")]
    X,
    #[serde(rename = "facebook")]
    Facebook,
    #[serde(rename = "threads")]
    Threads,
    #[serde(rename = "other")]
    Other,
}
impl ::std::fmt::Display for MindBriefPlatformsItem {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::YoutubeShorts => f.write_str("youtube_shorts"),
            Self::Tiktok => f.write_str("tiktok"),
            Self::InstagramReels => f.write_str("instagram_reels"),
            Self::Youtube => f.write_str("youtube"),
            Self::Linkedin => f.write_str("linkedin"),
            Self::X => f.write_str("x"),
            Self::Facebook => f.write_str("facebook"),
            Self::Threads => f.write_str("threads"),
            Self::Other => f.write_str("other"),
        }
    }
}
impl ::std::str::FromStr for MindBriefPlatformsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "youtube_shorts" => Ok(Self::YoutubeShorts),
            "tiktok" => Ok(Self::Tiktok),
            "instagram_reels" => Ok(Self::InstagramReels),
            "youtube" => Ok(Self::Youtube),
            "linkedin" => Ok(Self::Linkedin),
            "x" => Ok(Self::X),
            "facebook" => Ok(Self::Facebook),
            "threads" => Ok(Self::Threads),
            "other" => Ok(Self::Other),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefPlatformsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefPlatformsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefPlatformsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindBriefProduction`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"broll\","]
#[doc = "    \"effects\","]
#[doc = "    \"hook_style\","]
#[doc = "    \"music\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"broll\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"none\","]
#[doc = "        \"own_footage\","]
#[doc = "        \"stock\","]
#[doc = "        \"generated\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"caption_style_ref\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 120,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"effects\": {"]
#[doc = "      \"description\": \"How much finishing: none, light (captions and emphasis), medium (plus hooks and graphics), full (plus B-roll, music and effects).\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"none\","]
#[doc = "        \"light\","]
#[doc = "        \"medium\","]
#[doc = "        \"full\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"hook_style\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"none\","]
#[doc = "        \"title_card\","]
#[doc = "        \"punch_in\","]
#[doc = "        \"question\","]
#[doc = "        \"cold_open_flash\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"music\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"none\","]
#[doc = "        \"library\","]
#[doc = "        \"generated\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MindBriefProduction {
    pub broll: MindBriefProductionBroll,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub caption_style_ref: ::std::option::Option<MindBriefProductionCaptionStyleRef>,
    #[doc = "How much finishing: none, light (captions and emphasis), medium (plus hooks and graphics), full (plus B-roll, music and effects)."]
    pub effects: MindBriefProductionEffects,
    pub hook_style: MindBriefProductionHookStyle,
    pub music: MindBriefProductionMusic,
}
impl MindBriefProduction {
    pub fn builder() -> builder::MindBriefProduction {
        Default::default()
    }
}
#[doc = "`MindBriefProductionBroll`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"none\","]
#[doc = "    \"own_footage\","]
#[doc = "    \"stock\","]
#[doc = "    \"generated\""]
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
pub enum MindBriefProductionBroll {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "own_footage")]
    OwnFootage,
    #[serde(rename = "stock")]
    Stock,
    #[serde(rename = "generated")]
    Generated,
}
impl ::std::fmt::Display for MindBriefProductionBroll {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::None => f.write_str("none"),
            Self::OwnFootage => f.write_str("own_footage"),
            Self::Stock => f.write_str("stock"),
            Self::Generated => f.write_str("generated"),
        }
    }
}
impl ::std::str::FromStr for MindBriefProductionBroll {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "none" => Ok(Self::None),
            "own_footage" => Ok(Self::OwnFootage),
            "stock" => Ok(Self::Stock),
            "generated" => Ok(Self::Generated),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefProductionBroll {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefProductionBroll {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefProductionBroll {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindBriefProductionCaptionStyleRef`"]
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
pub struct MindBriefProductionCaptionStyleRef(::std::string::String);
impl ::std::ops::Deref for MindBriefProductionCaptionStyleRef {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindBriefProductionCaptionStyleRef> for ::std::string::String {
    fn from(value: MindBriefProductionCaptionStyleRef) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindBriefProductionCaptionStyleRef {
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
impl ::std::convert::TryFrom<&str> for MindBriefProductionCaptionStyleRef {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefProductionCaptionStyleRef {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefProductionCaptionStyleRef {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindBriefProductionCaptionStyleRef {
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
#[doc = "How much finishing: none, light (captions and emphasis), medium (plus hooks and graphics), full (plus B-roll, music and effects)."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How much finishing: none, light (captions and emphasis), medium (plus hooks and graphics), full (plus B-roll, music and effects).\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"none\","]
#[doc = "    \"light\","]
#[doc = "    \"medium\","]
#[doc = "    \"full\""]
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
pub enum MindBriefProductionEffects {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "full")]
    Full,
}
impl ::std::fmt::Display for MindBriefProductionEffects {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::None => f.write_str("none"),
            Self::Light => f.write_str("light"),
            Self::Medium => f.write_str("medium"),
            Self::Full => f.write_str("full"),
        }
    }
}
impl ::std::str::FromStr for MindBriefProductionEffects {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "none" => Ok(Self::None),
            "light" => Ok(Self::Light),
            "medium" => Ok(Self::Medium),
            "full" => Ok(Self::Full),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefProductionEffects {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefProductionEffects {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefProductionEffects {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindBriefProductionHookStyle`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"none\","]
#[doc = "    \"title_card\","]
#[doc = "    \"punch_in\","]
#[doc = "    \"question\","]
#[doc = "    \"cold_open_flash\""]
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
pub enum MindBriefProductionHookStyle {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "title_card")]
    TitleCard,
    #[serde(rename = "punch_in")]
    PunchIn,
    #[serde(rename = "question")]
    Question,
    #[serde(rename = "cold_open_flash")]
    ColdOpenFlash,
}
impl ::std::fmt::Display for MindBriefProductionHookStyle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::None => f.write_str("none"),
            Self::TitleCard => f.write_str("title_card"),
            Self::PunchIn => f.write_str("punch_in"),
            Self::Question => f.write_str("question"),
            Self::ColdOpenFlash => f.write_str("cold_open_flash"),
        }
    }
}
impl ::std::str::FromStr for MindBriefProductionHookStyle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "none" => Ok(Self::None),
            "title_card" => Ok(Self::TitleCard),
            "punch_in" => Ok(Self::PunchIn),
            "question" => Ok(Self::Question),
            "cold_open_flash" => Ok(Self::ColdOpenFlash),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefProductionHookStyle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefProductionHookStyle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefProductionHookStyle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MindBriefProductionMusic`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"none\","]
#[doc = "    \"library\","]
#[doc = "    \"generated\""]
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
pub enum MindBriefProductionMusic {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "library")]
    Library,
    #[serde(rename = "generated")]
    Generated,
}
impl ::std::fmt::Display for MindBriefProductionMusic {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::None => f.write_str("none"),
            Self::Library => f.write_str("library"),
            Self::Generated => f.write_str("generated"),
        }
    }
}
impl ::std::str::FromStr for MindBriefProductionMusic {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "none" => Ok(Self::None),
            "library" => Ok(Self::Library),
            "generated" => Ok(Self::Generated),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefProductionMusic {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefProductionMusic {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefProductionMusic {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "What the person asked for, in their own words, as data: never instructions to the model reading the brief."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"What the person asked for, in their own words, as data: never instructions to the model reading the brief.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 1000,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MindBriefPrompt(::std::string::String);
impl ::std::ops::Deref for MindBriefPrompt {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MindBriefPrompt> for ::std::string::String {
    fn from(value: MindBriefPrompt) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MindBriefPrompt {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 1000usize {
            return Err("longer than 1000 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MindBriefPrompt {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MindBriefPrompt {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MindBriefPrompt {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MindBriefPrompt {
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
#[doc = "`Phrases`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"type\": \"string\","]
#[doc = "    \"maxLength\": 200,"]
#[doc = "    \"minLength\": 1"]
#[doc = "  },"]
#[doc = "  \"maxItems\": 20"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Phrases(pub ::std::vec::Vec<PhrasesItem>);
impl ::std::ops::Deref for Phrases {
    type Target = ::std::vec::Vec<PhrasesItem>;
    fn deref(&self) -> &::std::vec::Vec<PhrasesItem> {
        &self.0
    }
}
impl ::std::convert::From<Phrases> for ::std::vec::Vec<PhrasesItem> {
    fn from(value: Phrases) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::vec::Vec<PhrasesItem>> for Phrases {
    fn from(value: ::std::vec::Vec<PhrasesItem>) -> Self {
        Self(value)
    }
}
#[doc = "`PhrasesItem`"]
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
pub struct PhrasesItem(::std::string::String);
impl ::std::ops::Deref for PhrasesItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PhrasesItem> for ::std::string::String {
    fn from(value: PhrasesItem) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for PhrasesItem {
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
impl ::std::convert::TryFrom<&str> for PhrasesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PhrasesItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PhrasesItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PhrasesItem {
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
    pub struct MindBrief {
        aspect: ::std::result::Result<super::MindBriefAspect, ::std::string::String>,
        content: ::std::result::Result<super::MindBriefContent, ::std::string::String>,
        count: ::std::result::Result<super::MindBriefCount, ::std::string::String>,
        forms: ::std::result::Result<
            ::std::vec::Vec<super::MindBriefFormsItem>,
            ::std::string::String,
        >,
        generation: ::std::result::Result<super::MindBriefGeneration, ::std::string::String>,
        genre: ::std::result::Result<super::MindBriefGenre, ::std::string::String>,
        goal: ::std::result::Result<super::MindBriefGoal, ::std::string::String>,
        inferred: ::std::result::Result<
            ::std::vec::Vec<super::MindBriefInferredItem>,
            ::std::string::String,
        >,
        language: ::std::result::Result<
            ::std::option::Option<super::MindBriefLanguage>,
            ::std::string::String,
        >,
        length: ::std::result::Result<super::MindBriefLength, ::std::string::String>,
        platforms: ::std::result::Result<
            ::std::vec::Vec<super::MindBriefPlatformsItem>,
            ::std::string::String,
        >,
        production: ::std::result::Result<super::MindBriefProduction, ::std::string::String>,
        prompt: ::std::result::Result<
            ::std::option::Option<super::MindBriefPrompt>,
            ::std::string::String,
        >,
        schema_version: ::std::result::Result<::serde_json::Value, ::std::string::String>,
    }
    impl ::std::default::Default for MindBrief {
        fn default() -> Self {
            Self {
                aspect: Err("no value supplied for aspect".to_string()),
                content: Err("no value supplied for content".to_string()),
                count: Err("no value supplied for count".to_string()),
                forms: Err("no value supplied for forms".to_string()),
                generation: Err("no value supplied for generation".to_string()),
                genre: Err("no value supplied for genre".to_string()),
                goal: Err("no value supplied for goal".to_string()),
                inferred: Err("no value supplied for inferred".to_string()),
                language: Ok(Default::default()),
                length: Err("no value supplied for length".to_string()),
                platforms: Err("no value supplied for platforms".to_string()),
                production: Err("no value supplied for production".to_string()),
                prompt: Ok(Default::default()),
                schema_version: Err("no value supplied for schema_version".to_string()),
            }
        }
    }
    impl MindBrief {
        pub fn aspect<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefAspect>,
            T::Error: ::std::fmt::Display,
        {
            self.aspect = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for aspect: {e}"));
            self
        }
        pub fn content<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefContent>,
            T::Error: ::std::fmt::Display,
        {
            self.content = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for content: {e}"));
            self
        }
        pub fn count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefCount>,
            T::Error: ::std::fmt::Display,
        {
            self.count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for count: {e}"));
            self
        }
        pub fn forms<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindBriefFormsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.forms = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for forms: {e}"));
            self
        }
        pub fn generation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefGeneration>,
            T::Error: ::std::fmt::Display,
        {
            self.generation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for generation: {e}"));
            self
        }
        pub fn genre<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefGenre>,
            T::Error: ::std::fmt::Display,
        {
            self.genre = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for genre: {e}"));
            self
        }
        pub fn goal<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefGoal>,
            T::Error: ::std::fmt::Display,
        {
            self.goal = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for goal: {e}"));
            self
        }
        pub fn inferred<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindBriefInferredItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.inferred = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for inferred: {e}"));
            self
        }
        pub fn language<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::MindBriefLanguage>>,
            T::Error: ::std::fmt::Display,
        {
            self.language = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for language: {e}"));
            self
        }
        pub fn length<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefLength>,
            T::Error: ::std::fmt::Display,
        {
            self.length = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for length: {e}"));
            self
        }
        pub fn platforms<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindBriefPlatformsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.platforms = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for platforms: {e}"));
            self
        }
        pub fn production<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefProduction>,
            T::Error: ::std::fmt::Display,
        {
            self.production = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for production: {e}"));
            self
        }
        pub fn prompt<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::MindBriefPrompt>>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt: {e}"));
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
    impl ::std::convert::TryFrom<MindBrief> for super::MindBrief {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindBrief,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                aspect: value.aspect?,
                content: value.content?,
                count: value.count?,
                forms: value.forms?,
                generation: value.generation?,
                genre: value.genre?,
                goal: value.goal?,
                inferred: value.inferred?,
                language: value.language?,
                length: value.length?,
                platforms: value.platforms?,
                production: value.production?,
                prompt: value.prompt?,
                schema_version: value.schema_version?,
            })
        }
    }
    impl ::std::convert::From<super::MindBrief> for MindBrief {
        fn from(value: super::MindBrief) -> Self {
            Self {
                aspect: Ok(value.aspect),
                content: Ok(value.content),
                count: Ok(value.count),
                forms: Ok(value.forms),
                generation: Ok(value.generation),
                genre: Ok(value.genre),
                goal: Ok(value.goal),
                inferred: Ok(value.inferred),
                language: Ok(value.language),
                length: Ok(value.length),
                platforms: Ok(value.platforms),
                production: Ok(value.production),
                prompt: Ok(value.prompt),
                schema_version: Ok(value.schema_version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindBriefContent {
        avoid: ::std::result::Result<::std::option::Option<super::Phrases>, ::std::string::String>,
        include:
            ::std::result::Result<::std::option::Option<super::Phrases>, ::std::string::String>,
        moments:
            ::std::result::Result<::std::option::Option<super::Phrases>, ::std::string::String>,
        mood: ::std::result::Result<
            ::std::vec::Vec<super::MindBriefContentMoodItem>,
            ::std::string::String,
        >,
        people: ::std::result::Result<::std::option::Option<super::Phrases>, ::std::string::String>,
        topics: ::std::result::Result<::std::option::Option<super::Phrases>, ::std::string::String>,
    }
    impl ::std::default::Default for MindBriefContent {
        fn default() -> Self {
            Self {
                avoid: Ok(Default::default()),
                include: Ok(Default::default()),
                moments: Ok(Default::default()),
                mood: Ok(Default::default()),
                people: Ok(Default::default()),
                topics: Ok(Default::default()),
            }
        }
    }
    impl MindBriefContent {
        pub fn avoid<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Phrases>>,
            T::Error: ::std::fmt::Display,
        {
            self.avoid = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for avoid: {e}"));
            self
        }
        pub fn include<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Phrases>>,
            T::Error: ::std::fmt::Display,
        {
            self.include = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for include: {e}"));
            self
        }
        pub fn moments<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Phrases>>,
            T::Error: ::std::fmt::Display,
        {
            self.moments = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for moments: {e}"));
            self
        }
        pub fn mood<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::MindBriefContentMoodItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.mood = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mood: {e}"));
            self
        }
        pub fn people<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Phrases>>,
            T::Error: ::std::fmt::Display,
        {
            self.people = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for people: {e}"));
            self
        }
        pub fn topics<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Phrases>>,
            T::Error: ::std::fmt::Display,
        {
            self.topics = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for topics: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindBriefContent> for super::MindBriefContent {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindBriefContent,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                avoid: value.avoid?,
                include: value.include?,
                moments: value.moments?,
                mood: value.mood?,
                people: value.people?,
                topics: value.topics?,
            })
        }
    }
    impl ::std::convert::From<super::MindBriefContent> for MindBriefContent {
        fn from(value: super::MindBriefContent) -> Self {
            Self {
                avoid: Ok(value.avoid),
                include: Ok(value.include),
                moments: Ok(value.moments),
                mood: Ok(value.mood),
                people: Ok(value.people),
                topics: Ok(value.topics),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindBriefCount {
        all_worthwhile: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        target: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for MindBriefCount {
        fn default() -> Self {
            Self {
                all_worthwhile: Ok(Default::default()),
                target: Err("no value supplied for target".to_string()),
            }
        }
    }
    impl MindBriefCount {
        pub fn all_worthwhile<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.all_worthwhile = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for all_worthwhile: {e}"));
            self
        }
        pub fn target<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.target = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for target: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindBriefCount> for super::MindBriefCount {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindBriefCount,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                all_worthwhile: value.all_worthwhile?,
                target: value.target?,
            })
        }
    }
    impl ::std::convert::From<super::MindBriefCount> for MindBriefCount {
        fn from(value: super::MindBriefCount) -> Self {
            Self {
                all_worthwhile: Ok(value.all_worthwhile),
                target: Ok(value.target),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindBriefGeneration {
        images: ::std::result::Result<bool, ::std::string::String>,
        video: ::std::result::Result<bool, ::std::string::String>,
        voice: ::std::result::Result<bool, ::std::string::String>,
    }
    impl ::std::default::Default for MindBriefGeneration {
        fn default() -> Self {
            Self {
                images: Err("no value supplied for images".to_string()),
                video: Err("no value supplied for video".to_string()),
                voice: Err("no value supplied for voice".to_string()),
            }
        }
    }
    impl MindBriefGeneration {
        pub fn images<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.images = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for images: {e}"));
            self
        }
        pub fn video<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.video = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for video: {e}"));
            self
        }
        pub fn voice<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.voice = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for voice: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindBriefGeneration> for super::MindBriefGeneration {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindBriefGeneration,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                images: value.images?,
                video: value.video?,
                voice: value.voice?,
            })
        }
    }
    impl ::std::convert::From<super::MindBriefGeneration> for MindBriefGeneration {
        fn from(value: super::MindBriefGeneration) -> Self {
            Self {
                images: Ok(value.images),
                video: Ok(value.video),
                voice: Ok(value.voice),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindBriefLength {
        max_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        min_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        target_ticks: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for MindBriefLength {
        fn default() -> Self {
            Self {
                max_ticks: Err("no value supplied for max_ticks".to_string()),
                min_ticks: Err("no value supplied for min_ticks".to_string()),
                target_ticks: Err("no value supplied for target_ticks".to_string()),
            }
        }
    }
    impl MindBriefLength {
        pub fn max_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max_ticks: {e}"));
            self
        }
        pub fn min_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for min_ticks: {e}"));
            self
        }
        pub fn target_ticks<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.target_ticks = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for target_ticks: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindBriefLength> for super::MindBriefLength {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindBriefLength,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                max_ticks: value.max_ticks?,
                min_ticks: value.min_ticks?,
                target_ticks: value.target_ticks?,
            })
        }
    }
    impl ::std::convert::From<super::MindBriefLength> for MindBriefLength {
        fn from(value: super::MindBriefLength) -> Self {
            Self {
                max_ticks: Ok(value.max_ticks),
                min_ticks: Ok(value.min_ticks),
                target_ticks: Ok(value.target_ticks),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MindBriefProduction {
        broll: ::std::result::Result<super::MindBriefProductionBroll, ::std::string::String>,
        caption_style_ref: ::std::result::Result<
            ::std::option::Option<super::MindBriefProductionCaptionStyleRef>,
            ::std::string::String,
        >,
        effects: ::std::result::Result<super::MindBriefProductionEffects, ::std::string::String>,
        hook_style:
            ::std::result::Result<super::MindBriefProductionHookStyle, ::std::string::String>,
        music: ::std::result::Result<super::MindBriefProductionMusic, ::std::string::String>,
    }
    impl ::std::default::Default for MindBriefProduction {
        fn default() -> Self {
            Self {
                broll: Err("no value supplied for broll".to_string()),
                caption_style_ref: Ok(Default::default()),
                effects: Err("no value supplied for effects".to_string()),
                hook_style: Err("no value supplied for hook_style".to_string()),
                music: Err("no value supplied for music".to_string()),
            }
        }
    }
    impl MindBriefProduction {
        pub fn broll<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefProductionBroll>,
            T::Error: ::std::fmt::Display,
        {
            self.broll = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for broll: {e}"));
            self
        }
        pub fn caption_style_ref<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                    ::std::option::Option<super::MindBriefProductionCaptionStyleRef>,
                >,
            T::Error: ::std::fmt::Display,
        {
            self.caption_style_ref = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for caption_style_ref: {e}"));
            self
        }
        pub fn effects<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefProductionEffects>,
            T::Error: ::std::fmt::Display,
        {
            self.effects = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for effects: {e}"));
            self
        }
        pub fn hook_style<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefProductionHookStyle>,
            T::Error: ::std::fmt::Display,
        {
            self.hook_style = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for hook_style: {e}"));
            self
        }
        pub fn music<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MindBriefProductionMusic>,
            T::Error: ::std::fmt::Display,
        {
            self.music = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for music: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MindBriefProduction> for super::MindBriefProduction {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MindBriefProduction,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                broll: value.broll?,
                caption_style_ref: value.caption_style_ref?,
                effects: value.effects?,
                hook_style: value.hook_style?,
                music: value.music?,
            })
        }
    }
    impl ::std::convert::From<super::MindBriefProduction> for MindBriefProduction {
        fn from(value: super::MindBriefProduction) -> Self {
            Self {
                broll: Ok(value.broll),
                caption_style_ref: Ok(value.caption_style_ref),
                effects: Ok(value.effects),
                hook_style: Ok(value.hook_style),
                music: Ok(value.music),
            }
        }
    }
}
