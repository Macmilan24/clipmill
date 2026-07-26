//! The pinned model registry, as the daemon reads it.
//!
//! A model is a *versioned input to an artifact*, not an ambient capability
//! (book ch. 11). Its identity therefore has to be a value the daemon can put
//! in a recipe: that is the manifest digest below, computed over the pinned
//! files rather than over the manifest's prose, so re-wording a comment does
//! not invalidate a transcript while re-pinning a weight does.
//!
//! Nothing here downloads anything. Acquisition is `tools/fetch-models.sh`,
//! outside the Local Lock; the daemon only ever reads what is already on disk,
//! verifies it, and refuses when it does not match.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clipmill_contracts::proto::worker::v1::{ModelBinding, ModelFile as ModelFileBinding};
use clipmill_core::Sha256Digest;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModelManifest {
    pub name: String,
    pub capability: String,
    #[allow(
        dead_code,
        reason = "stated in the manifest for operators and the fetcher"
    )]
    pub family: String,
    pub runtime: String,
    pub backend: String,
    pub quantization: String,
    pub source: ModelSource,
    pub license: ModelLicense,
    pub memory: ModelMemory,
    pub files: Vec<ModelFile>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModelSource {
    pub repo: String,
    pub revision: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModelLicense {
    pub spdx: String,
    pub class: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModelMemory {
    pub weights_bytes: u64,
    pub runtime_overhead_bytes: u64,
}

impl ModelMemory {
    /// What a worker must actually have free to run this model. Admission
    /// checks the sum, because a machine that fits the weights and not the
    /// runtime cannot run the model either.
    pub fn resident_bytes(&self) -> u64 {
        self.weights_bytes
            .saturating_add(self.runtime_overhead_bytes)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModelFile {
    pub path: String,
    pub sha256: String,
    /// Carried through to the worker so a truncated file is refused before an
    /// ONNX parser is pointed at it, rather than after.
    pub bytes: u64,
}

impl ModelManifest {
    /// The model's identity for an artifact recipe.
    ///
    /// Computed over the pinned name, revision, and file digests — never over
    /// the manifest text. A comment change must not invalidate a cached
    /// transcript, and a re-pinned weight must.
    pub fn digest(&self) -> Sha256Digest {
        let mut hasher = Sha256::new();
        hasher.update(b"clipmill.model.identity.v1\0");
        hasher.update(self.name.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.source.repo.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.source.revision.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.quantization.as_bytes());
        hasher.update(b"\0");
        // Sorted, so the manifest's file order cannot change the identity.
        let mut files = self
            .files
            .iter()
            .map(|file| (file.path.as_str(), file.sha256.as_str()))
            .collect::<Vec<_>>();
        files.sort_unstable();
        for (path, sha256) in files {
            hasher.update(path.as_bytes());
            hasher.update(b"\0");
            hasher.update(sha256.as_bytes());
            hasher.update(b"\0");
        }
        Sha256Digest::from_bytes(hasher.finalize().into())
    }

    /// One line an operator can read: what is pinned, and what it costs.
    pub fn summary(&self) -> String {
        format!(
            "{} ({} via {} on {}, {}, {} MiB resident, {})",
            self.name,
            self.capability,
            self.runtime,
            self.backend,
            self.quantization,
            self.memory.resident_bytes() / (1024 * 1024),
            self.license.spdx,
        )
    }
}

/// Every manifest under a registry directory, keyed by model name.
#[derive(Clone, Debug, Default)]
pub(crate) struct ModelRegistry {
    models: BTreeMap<String, ModelManifest>,
}

impl ModelRegistry {
    pub fn load(directory: &Path) -> Result<Self, ModelError> {
        let mut models = BTreeMap::new();
        // A daemon with no registry can still run every model-free stage, so
        // its absence is not a startup failure.
        let Ok(entries) = fs::read_dir(directory) else {
            return Ok(Self::default());
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "toml") {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|error| ModelError::Unreadable {
                path: path.clone(),
                detail: error.to_string(),
            })?;
            let manifest: ModelManifest =
                toml::from_str(&text).map_err(|error| ModelError::Unreadable {
                    path: path.clone(),
                    detail: error.to_string(),
                })?;
            if manifest.files.is_empty() {
                return Err(ModelError::Unreadable {
                    path,
                    detail: "a model must pin at least one file".to_owned(),
                });
            }
            models.insert(manifest.name.clone(), manifest);
        }
        Ok(Self { models })
    }

    pub fn get(&self, name: &str) -> Option<&ModelManifest> {
        self.models.get(name)
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// What this daemon has pinned, for the startup log. Operators asking
    /// "which model produced this?" should not have to read TOML to find out.
    pub fn summaries(&self) -> Vec<String> {
        self.models.values().map(ModelManifest::summary).collect()
    }

    /// What a worker needs to load one pinned model: where the files are, and
    /// what each must hash to.
    ///
    /// The path travels on the lease rather than in the task payload because
    /// the payload is hashed into the artifact key, and a machine-specific
    /// directory in the key would give one transcript two content addresses
    /// on two machines. Identity reaches the key by a different road: the
    /// manifest digest, through the recipe.
    ///
    /// Nothing is verified here. The daemon hands over the digests it pinned
    /// and the worker checks the bytes immediately before loading them, which
    /// is the only check close enough to the load to mean anything.
    pub fn binding(&self, name: &str, weights_root: &Path) -> Option<ModelBinding> {
        let manifest = self.get(name)?;
        Some(ModelBinding {
            name: manifest.name.clone(),
            root: weights_root
                .join(&manifest.name)
                .to_string_lossy()
                .into_owned(),
            digest: manifest.digest().to_string(),
            capability: manifest.capability.clone(),
            files: manifest
                .files
                .iter()
                .map(|file| ModelFileBinding {
                    path: file.path.clone(),
                    sha256: file.sha256.clone(),
                    bytes: file.bytes,
                })
                .collect(),
        })
    }

    /// The licence classes present, so the daemon can state its rights
    /// position rather than implying one.
    pub fn license_classes(&self) -> std::collections::BTreeSet<&str> {
        self.models
            .values()
            .map(|manifest| manifest.license.class.as_str())
            .collect()
    }
}

#[derive(Debug, Error)]
pub(crate) enum ModelError {
    #[error("model manifest {path} is unreadable: {detail}")]
    Unreadable { path: PathBuf, detail: String },
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::path::Path;

    use super::ModelRegistry;

    fn published_registry() -> ModelRegistry {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry");
        ModelRegistry::load(&path).expect("the published registry loads")
    }

    #[test]
    fn the_published_registry_pins_every_phase_one_capability() {
        let registry = published_registry();
        assert!(registry.len() >= 6, "expected the phase's pinned models");
        let capabilities = registry.summaries().join(" ");
        for capability in ["vad", "asr", "forced-align", "detect-faces"] {
            assert!(
                capabilities.contains(capability),
                "no model offers {capability}"
            );
        }
        // Every pinned licence must be one that permits publication.
        assert_eq!(
            registry.license_classes(),
            ["permissive"].into_iter().collect()
        );
    }

    /// The speech chain has to run on a machine with no accelerator at all,
    /// or the phase's "end-to-end offline on reference machines" gate is a
    /// claim about one laptop. Every capability the chain needs must therefore
    /// have at least one pinned model that a plain CPU can load.
    #[test]
    fn every_speech_capability_has_a_cpu_only_fallback() {
        let registry = published_registry();
        for capability in ["vad", "asr", "forced-align"] {
            let fallbacks = registry
                .models
                .values()
                .filter(|manifest| manifest.capability == capability)
                .filter(|manifest| matches!(manifest.backend.as_str(), "cpu" | "onnx-cpu"))
                .count();
            assert!(
                fallbacks > 0,
                "{capability} is pinned only on accelerated backends"
            );
        }
    }

    /// The binding is what a worker actually loads from, so it has to carry
    /// enough to refuse a bad file — and it must not carry the model's path
    /// anywhere the artifact key can see it.
    #[test]
    fn a_binding_hands_over_a_path_and_the_digests_to_check_it_against() {
        let registry = published_registry();
        let binding = registry
            .binding("silero-vad", Path::new("/opt/clipmill/models"))
            .expect("silero-vad is pinned");

        assert_eq!(binding.root, "/opt/clipmill/models/silero-vad");
        assert_eq!(binding.capability, "vad");
        assert_eq!(
            binding.digest,
            registry
                .get("silero-vad")
                .expect("pinned")
                .digest()
                .to_string(),
            "the worker echoes this as the producing model's identity"
        );
        assert!(!binding.files.is_empty());
        for file in &binding.files {
            assert_eq!(file.sha256.len(), 64, "a bare hex digest, as pinned");
            assert!(
                file.bytes > 0,
                "so a truncated file is refused before it is parsed"
            );
        }

        // The directory is the only machine-specific value here, and it lives
        // on the lease rather than in the payload the recipe hashes.
        let elsewhere = registry
            .binding("silero-vad", Path::new("/somewhere/else"))
            .expect("pinned");
        assert_ne!(elsewhere.root, binding.root);
        assert_eq!(elsewhere.digest, binding.digest);
    }

    #[test]
    fn a_model_nobody_pinned_has_no_binding() {
        let registry = published_registry();
        assert!(registry.binding("not-a-model", Path::new("/opt")).is_none());
    }

    #[test]
    fn a_missing_registry_is_not_a_startup_failure() {
        let registry = ModelRegistry::load(Path::new("/nonexistent/registry")).expect("loads");
        assert_eq!(registry.len(), 0);
    }

    /// The identity must follow the pinned bytes, not the prose around them.
    #[test]
    fn the_digest_tracks_the_pins_rather_than_the_manifest_text() {
        let registry = published_registry();
        let model = registry.get("silero-vad").expect("silero-vad is pinned");
        let baseline = model.digest();

        let mut reordered = model.clone();
        reordered.files.reverse();
        assert_eq!(reordered.digest(), baseline, "file order is not identity");

        let mut relabelled = model.clone();
        relabelled.family = "something else entirely".to_owned();
        assert_eq!(relabelled.digest(), baseline, "prose is not identity");

        let mut repinned = model.clone();
        repinned.files[0].sha256 = "0".repeat(64);
        assert_ne!(
            repinned.digest(),
            baseline,
            "a re-pinned weight is a new model"
        );

        let mut moved = model.clone();
        moved.source.revision = "f".repeat(40);
        assert_ne!(moved.digest(), baseline, "a new revision is a new model");
    }
}
