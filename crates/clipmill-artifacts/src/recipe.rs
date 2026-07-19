use std::fmt;

use clipmill_core::{ArtifactId, Sha256Digest};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub(crate) const KEY_VERSION: &str = "clipmill.artifact.key.v1";
const KEY_DOMAIN: &[u8] = b"clipmill.artifact.key.v1\0";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkPolicy {
    LocalLock,
    NetworkAllowed,
}

impl fmt::Display for NetworkPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalLock => formatter.write_str("local-lock"),
            Self::NetworkAllowed => formatter.write_str("network-allowed"),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Timebase {
    pub num: u64,
    pub den: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Producer {
    pub stage: String,
    pub implementation: String,
    pub model_digest: Option<Sha256Digest>,
}

#[derive(Clone, Debug)]
pub struct RecipeSpec {
    pub kind: String,
    pub source_fingerprint: Sha256Digest,
    pub timebase: Timebase,
    pub producer: Producer,
    pub inputs: Vec<ArtifactId>,
    pub policy: NetworkPolicy,
    pub config: Map<String, Value>,
    pub semantic_version: String,
}

/// Validated semantic inputs used to derive an artifact cache key.
#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactRecipe {
    kind: String,
    source_fingerprint: Sha256Digest,
    timebase: Timebase,
    producer: Producer,
    inputs: Vec<ArtifactId>,
    policy: NetworkPolicy,
    config: Map<String, Value>,
    semantic_version: String,
}

impl ArtifactRecipe {
    pub fn try_from_spec(spec: RecipeSpec) -> Result<Self, RecipeError> {
        validate_kind(&spec.kind)?;
        validate_stage(&spec.producer.stage)?;
        validate_text(
            &spec.producer.implementation,
            256,
            RecipeError::InvalidImplementation,
        )?;
        validate_text(
            &spec.semantic_version,
            128,
            RecipeError::InvalidSemanticVersion,
        )?;
        if spec.timebase.num == 0 || spec.timebase.den == 0 {
            return Err(RecipeError::InvalidTimebase);
        }
        Ok(Self {
            kind: spec.kind,
            source_fingerprint: spec.source_fingerprint,
            timebase: spec.timebase,
            producer: spec.producer,
            inputs: spec.inputs,
            policy: spec.policy,
            config: spec.config,
            semantic_version: spec.semantic_version,
        })
    }

    pub fn artifact_id(&self) -> Result<ArtifactId, RecipeError> {
        let preimage = KeyPreimage {
            key_version: KEY_VERSION,
            kind: &self.kind,
            source_fingerprint: prefixed_digest(self.source_fingerprint),
            timebase: self.timebase,
            stage: &self.producer.stage,
            implementation: &self.producer.implementation,
            model_digest: self.producer.model_digest.map(prefixed_digest),
            inputs: self.inputs.iter().map(ToString::to_string).collect(),
            policy: self.policy,
            config: &self.config,
            semantic_version: &self.semantic_version,
        };
        let canonical = serde_json_canonicalizer::to_vec(&preimage)?;
        let mut hasher = Sha256::new();
        hasher.update(KEY_DOMAIN);
        hasher.update(canonical);
        Ok(ArtifactId::from_digest(Sha256Digest::from_bytes(
            hasher.finalize().into(),
        )))
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    pub fn source_fingerprint(&self) -> Sha256Digest {
        self.source_fingerprint
    }

    #[must_use]
    pub fn timebase(&self) -> Timebase {
        self.timebase
    }

    #[must_use]
    pub fn producer(&self) -> &Producer {
        &self.producer
    }

    #[must_use]
    pub fn inputs(&self) -> &[ArtifactId] {
        &self.inputs
    }

    #[must_use]
    pub fn policy(&self) -> NetworkPolicy {
        self.policy
    }

    #[must_use]
    pub fn config(&self) -> &Map<String, Value> {
        &self.config
    }

    #[must_use]
    pub fn semantic_version(&self) -> &str {
        &self.semantic_version
    }
}

#[derive(Serialize)]
struct KeyPreimage<'a> {
    key_version: &'static str,
    kind: &'a str,
    source_fingerprint: String,
    timebase: Timebase,
    stage: &'a str,
    implementation: &'a str,
    model_digest: Option<String>,
    inputs: Vec<String>,
    policy: NetworkPolicy,
    config: &'a Map<String, Value>,
    semantic_version: &'a str,
}

pub(crate) fn prefixed_digest(digest: Sha256Digest) -> String {
    format!("sha256:{digest}")
}

fn validate_kind(value: &str) -> Result<(), RecipeError> {
    let Some((prefix, version)) = value.rsplit_once(".v") else {
        return Err(RecipeError::InvalidKind);
    };
    if prefix.is_empty()
        || !prefix.starts_with(|character: char| character.is_ascii_lowercase())
        || !prefix
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.".contains(&byte))
        || version.is_empty()
        || !version.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(RecipeError::InvalidKind);
    }
    Ok(())
}

fn validate_stage(value: &str) -> Result<(), RecipeError> {
    if value.is_empty()
        || !value.starts_with(|character: char| character.is_ascii_lowercase())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(RecipeError::InvalidStage);
    }
    Ok(())
}

fn validate_text(value: &str, maximum: usize, error: RecipeError) -> Result<(), RecipeError> {
    let count = value.chars().count();
    if count == 0 || count > maximum || value.chars().any(char::is_control) {
        return Err(error);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum RecipeError {
    #[error("artifact kind must match <lowercase-kind>.v<version>")]
    InvalidKind,
    #[error("producer stage must contain lower-case ASCII letters, digits, or hyphens")]
    InvalidStage,
    #[error("producer implementation must contain 1-256 non-control characters")]
    InvalidImplementation,
    #[error("semantic version must contain 1-128 non-control characters")]
    InvalidSemanticVersion,
    #[error("artifact timebase numerator and denominator must be positive")]
    InvalidTimebase,
    #[error("cannot canonicalize artifact recipe: {0}")]
    Canonical(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::collections::BTreeSet;

    use clipmill_core::{ArtifactId, Sha256Digest};
    use serde_json::{Map, json};

    use super::{ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, Timebase};

    fn recipe(config: Map<String, serde_json::Value>) -> ArtifactRecipe {
        ArtifactRecipe::try_from_spec(RecipeSpec {
            kind: "evidence.probe.v1".to_owned(),
            source_fingerprint: Sha256Digest::from_bytes([1; 32]),
            timebase: Timebase {
                num: 1,
                den: 90_000,
            },
            producer: Producer {
                stage: "probe".to_owned(),
                implementation: "probe@1.0.0".to_owned(),
                model_digest: None,
            },
            inputs: vec![ArtifactId::from_digest(Sha256Digest::from_bytes([2; 32]))],
            policy: NetworkPolicy::LocalLock,
            config,
            semantic_version: "1.0.0".to_owned(),
        })
        .expect("recipe")
    }

    #[test]
    fn canonical_object_order_does_not_change_identity() {
        let mut first = Map::new();
        first.insert("z".to_owned(), json!(1));
        first.insert("a".to_owned(), json!({"second": 2, "first": 1}));
        let mut second = Map::new();
        second.insert("a".to_owned(), json!({"first": 1, "second": 2}));
        second.insert("z".to_owned(), json!(1));
        assert_eq!(
            recipe(first).artifact_id().expect("id"),
            recipe(second).artifact_id().expect("id")
        );
    }

    #[test]
    fn changing_input_order_changes_identity() {
        let mut original = recipe(Map::new());
        original
            .inputs
            .push(ArtifactId::from_digest(Sha256Digest::from_bytes([3; 32])));
        let mut reversed = original.clone();
        reversed.inputs.reverse();
        assert_ne!(
            original.artifact_id().expect("original"),
            reversed.artifact_id().expect("reversed")
        );
    }

    #[test]
    fn matches_frozen_rfc8785_key_vector() {
        assert_eq!(
            recipe(Map::new()).artifact_id().expect("id").to_string(),
            "sha256:6f5cf59b95e23400afcef2b180f99e5827dc0ee3fd379dd8afbd6b8c2821802d"
        );
    }

    #[test]
    fn every_semantic_component_invalidates_the_key() {
        let original = recipe(Map::new());
        let original_id = original.artifact_id().expect("original");
        let mut variants = Vec::new();

        let mut changed = original.clone();
        changed.kind = "evidence.probe.v2".to_owned();
        variants.push(changed);
        let mut changed = original.clone();
        changed.source_fingerprint = Sha256Digest::from_bytes([9; 32]);
        variants.push(changed);
        let mut changed = original.clone();
        changed.timebase.den = 48_000;
        variants.push(changed);
        let mut changed = original.clone();
        changed.producer.stage = "probe-alt".to_owned();
        variants.push(changed);
        let mut changed = original.clone();
        changed.producer.implementation = "probe@2.0.0".to_owned();
        variants.push(changed);
        let mut changed = original.clone();
        changed.producer.model_digest = Some(Sha256Digest::from_bytes([8; 32]));
        variants.push(changed);
        let mut changed = original.clone();
        changed
            .inputs
            .push(ArtifactId::from_digest(Sha256Digest::from_bytes([7; 32])));
        variants.push(changed);
        let mut changed = original.clone();
        changed.policy = NetworkPolicy::NetworkAllowed;
        variants.push(changed);
        let mut changed = original.clone();
        changed.config.insert("mode".to_owned(), json!("accurate"));
        variants.push(changed);
        let mut changed = original.clone();
        changed.semantic_version = "2.0.0".to_owned();
        variants.push(changed);

        let ids = variants
            .iter()
            .map(|variant| variant.artifact_id().expect("variant"))
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), variants.len());
        assert!(!ids.contains(&original_id));
    }
}
