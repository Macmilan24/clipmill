use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use clipmill_contracts::schemas::artifact_manifest::ArtifactManifest;
use clipmill_core::{ArtifactId, Sha256Digest};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::{
    ArtifactPath, ArtifactPathError, ArtifactRecipe, NetworkPolicy, Producer, RecipeError,
    RecipeSpec, Timebase,
    recipe::{KEY_VERSION, prefixed_digest},
};

pub(crate) const MANIFEST_NAME: &str = "manifest.json";
const SCHEMA_VERSION: &str = "clipmill.artifact.manifest.v1";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredManifest {
    artifact_id: String,
    files: Vec<StoredFile>,
    inputs: Vec<String>,
    kind: String,
    policy: NetworkPolicy,
    producer: StoredProducer,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    quality: BTreeMap<String, f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recipe: Option<StoredRecipe>,
    schema_version: String,
    source_fingerprint: String,
    timebase: Timebase,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredProducer {
    implementation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model_digest: Option<String>,
    stage: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredRecipe {
    config: Map<String, Value>,
    key_version: String,
    semantic_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredFile {
    pub bytes: u64,
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileRecord {
    pub path: ArtifactPath,
    pub digest: Sha256Digest,
    pub bytes: u64,
}

impl StoredManifest {
    pub(crate) fn from_parts(
        artifact_id: ArtifactId,
        recipe: &ArtifactRecipe,
        files: &[FileRecord],
        quality: BTreeMap<String, f64>,
    ) -> Self {
        Self {
            artifact_id: artifact_id.to_string(),
            files: files
                .iter()
                .map(|file| StoredFile {
                    bytes: file.bytes,
                    path: file.path.to_string(),
                    sha256: prefixed_digest(file.digest),
                })
                .collect(),
            inputs: recipe.inputs().iter().map(ToString::to_string).collect(),
            kind: recipe.kind().to_owned(),
            policy: recipe.policy(),
            producer: StoredProducer {
                implementation: recipe.producer().implementation.clone(),
                model_digest: recipe.producer().model_digest.map(prefixed_digest),
                stage: recipe.producer().stage.clone(),
            },
            quality,
            recipe: Some(StoredRecipe {
                config: recipe.config().clone(),
                key_version: KEY_VERSION.to_owned(),
                semantic_version: recipe.semantic_version().to_owned(),
            }),
            schema_version: SCHEMA_VERSION.to_owned(),
            source_fingerprint: prefixed_digest(recipe.source_fingerprint()),
            timebase: recipe.timebase(),
        }
    }

    pub(crate) fn from_bytes(bytes: &[u8], expected_id: ArtifactId) -> Result<Self, ManifestError> {
        let _contract: ArtifactManifest = serde_json::from_slice(bytes)?;
        let manifest: Self = serde_json::from_slice(bytes)?;
        manifest.validate(expected_id)?;
        Ok(manifest)
    }

    pub(crate) fn to_pretty_bytes(&self) -> Result<Vec<u8>, ManifestError> {
        let value = serde_json::to_value(self)?;
        let mut text = serde_json::to_string_pretty(&value)?;
        text.push('\n');
        let _contract: ArtifactManifest = serde_json::from_str(&text)?;
        Ok(text.into_bytes())
    }

    pub(crate) fn artifact_id(&self) -> Result<ArtifactId, ManifestError> {
        Ok(self.artifact_id.parse()?)
    }

    pub(crate) fn kind(&self) -> &str {
        &self.kind
    }

    pub(crate) fn stage(&self) -> &str {
        &self.producer.stage
    }

    pub(crate) fn recipe(&self) -> Result<Option<ArtifactRecipe>, ManifestError> {
        let Some(recipe) = &self.recipe else {
            return Ok(None);
        };
        if recipe.key_version != KEY_VERSION {
            return Err(ManifestError::WrongKeyVersion);
        }
        let source_fingerprint = parse_digest(&self.source_fingerprint)?;
        let model_digest = self
            .producer
            .model_digest
            .as_deref()
            .map(parse_digest)
            .transpose()?;
        let inputs = self
            .inputs
            .iter()
            .map(|value| value.parse::<ArtifactId>().map_err(ManifestError::from))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(ArtifactRecipe::try_from_spec(RecipeSpec {
            kind: self.kind.clone(),
            source_fingerprint,
            timebase: self.timebase,
            producer: Producer {
                stage: self.producer.stage.clone(),
                implementation: self.producer.implementation.clone(),
                model_digest,
            },
            inputs,
            policy: self.policy,
            config: recipe.config.clone(),
            semantic_version: recipe.semantic_version.clone(),
        })?))
    }

    pub(crate) fn input_ids(&self) -> Result<Vec<ArtifactId>, ManifestError> {
        self.inputs
            .iter()
            .map(|value| value.parse().map_err(ManifestError::from))
            .collect()
    }

    pub(crate) fn file_records(&self) -> Result<Vec<FileRecord>, ManifestError> {
        self.files
            .iter()
            .map(|file| {
                Ok(FileRecord {
                    path: ArtifactPath::from_str(&file.path)?,
                    digest: parse_digest(&file.sha256)?,
                    bytes: file.bytes,
                })
            })
            .collect()
    }

    pub(crate) fn validate(&self, expected_id: ArtifactId) -> Result<(), ManifestError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ManifestError::WrongSchemaVersion);
        }
        if self.artifact_id()? != expected_id {
            return Err(ManifestError::ArtifactIdMismatch);
        }
        let files = self.file_records()?;
        let unique = files
            .iter()
            .map(|file| file.path.clone())
            .collect::<BTreeSet<_>>();
        if unique.len() != files.len() {
            return Err(ManifestError::DuplicateFilePath);
        }
        let _source = parse_digest(&self.source_fingerprint)?;
        let _inputs = self.input_ids()?;
        if let Some(recipe) = self.recipe()? {
            let computed = recipe.artifact_id()?;
            if computed != expected_id {
                return Err(ManifestError::RecipeKeyMismatch);
            }
        }
        if self.quality.iter().any(|(key, value)| {
            key.is_empty() || key.chars().any(char::is_control) || !value.is_finite()
        }) {
            return Err(ManifestError::InvalidQuality);
        }
        Ok(())
    }
}

fn parse_digest(value: &str) -> Result<Sha256Digest, ManifestError> {
    Ok(value.parse::<ArtifactId>()?.digest())
}

#[derive(Debug, Error)]
pub(crate) enum ManifestError {
    #[error("manifest JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("manifest digest is invalid: {0}")]
    Digest(#[from] clipmill_core::DigestError),
    #[error("manifest artifact path is invalid: {0}")]
    Path(#[from] ArtifactPathError),
    #[error("manifest recipe is invalid: {0}")]
    Recipe(#[from] RecipeError),
    #[error("manifest schema version is unsupported")]
    WrongSchemaVersion,
    #[error("manifest key version is unsupported")]
    WrongKeyVersion,
    #[error("manifest artifact id does not match its object directory")]
    ArtifactIdMismatch,
    #[error("manifest recipe does not reproduce its artifact id")]
    RecipeKeyMismatch,
    #[error("manifest declares the same file path more than once")]
    DuplicateFilePath,
    #[error("manifest quality keys or values are invalid")]
    InvalidQuality,
}
