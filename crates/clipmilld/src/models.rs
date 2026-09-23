//! The pinned model registry, as the daemon reads it.
//!
//! A model is a *versioned input to an artifact*, not an ambient capability
//! (book ch. 11). Its identity therefore has to be a value the daemon can put
//! in a recipe: that is the manifest digest below, computed over the pinned
//! files rather than over the manifest's prose, so re-wording a comment does
//! not invalidate a transcript while re-pinning a weight does.
//!
//! Two directories feed it. The bundled registry ships with ClipMill and is
//! read once at startup. The person's own directory holds models they pinned
//! from a repository in the app; one may be added while the daemon runs, which
//! is why the map sits behind a lock. Entries are never *changed* once present:
//! a name is an identity, so a second pin under a name already registered is
//! refused rather than replacing what earlier tasks were keyed against.
//!
//! Nothing here downloads anything. Acquisition is the model library
//! (`library.rs`), an explicit operation a person starts; this module only
//! says what is pinned and whether the bytes on disk still match it.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use clipmill_contracts::proto::worker::v1::{ModelBinding, ModelFile as ModelFileBinding};
use clipmill_core::Sha256Digest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// The only host models are acquired from. A manifest naming another provider
/// can still be loaded — the weights may have been placed by hand — but the
/// library will not fetch it.
pub(crate) const HUGGING_FACE: &str = "https://huggingface.co";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ModelManifest {
    pub name: String,
    pub capability: String,
    pub family: String,
    pub runtime: String,
    pub backend: String,
    pub quantization: String,
    pub source: ModelSource,
    pub license: ModelLicense,
    pub memory: ModelMemory,
    pub files: Vec<ModelFile>,
    /// How the model is presented in the library. Prose, so it is outside the
    /// identity: rewording a summary must not re-key a transcript.
    #[serde(default)]
    pub catalog: ModelCatalog,
    /// Where this manifest came from. Never read from or written to the file:
    /// the directory it was loaded from is the only evidence that counts.
    #[serde(skip)]
    pub origin: ModelOrigin,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ModelSource {
    #[serde(default = "hugging_face")]
    pub provider: String,
    pub repo: String,
    pub revision: String,
}

fn hugging_face() -> String {
    HUGGING_FACE.to_owned()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ModelLicense {
    /// How the terms are known: a pinned licence file, the repository's own
    /// metadata, or a base model's. Stated so an operator can tell a verified
    /// term from an assumed one.
    #[serde(default)]
    pub source: String,
    pub spdx: String,
    pub class: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ModelFile {
    pub path: String,
    pub sha256: String,
    /// Carried through to the worker so a truncated file is refused before an
    /// ONNX parser is pointed at it, rather than after.
    pub bytes: u64,
}

/// What the library shows about a model. Optional throughout: a manifest that
/// predates the catalog still loads and is listed under its registry name.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ModelCatalog {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub summary: String,
    /// Part of the set a fresh installation is offered in one step, on the
    /// devices that can run it.
    #[serde(default)]
    pub recommended: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ModelOrigin {
    /// Shipped with ClipMill in the bundled registry.
    #[default]
    Bundled,
    /// Pinned by the person from a repository, in the app.
    Custom,
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

    /// Every pinned byte, which is what a download costs.
    pub fn download_bytes(&self) -> u64 {
        self.files
            .iter()
            .fold(0_u64, |total, file| total.saturating_add(file.bytes))
    }

    /// The name a person reads. The registry name when the catalog gives none.
    pub fn title(&self) -> &str {
        if self.catalog.title.is_empty() {
            &self.name
        } else {
            &self.catalog.title
        }
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

    /// Refuse a manifest whose names could reach outside its own directory.
    ///
    /// The bundled registry is checked by `check-models.py` before anything
    /// ships; a manifest the person pinned in the app is checked here, and the
    /// same rules apply to both so a hand-edited file cannot slip past either.
    pub fn validate(&self) -> Result<(), String> {
        if !valid_name(&self.name) {
            return Err(format!("{:?} is not a simple model name", self.name));
        }
        if self.files.is_empty() {
            return Err("a model must pin at least one file".to_owned());
        }
        if !is_commit(&self.source.revision) {
            return Err("the source revision must be a full commit hash".to_owned());
        }
        let mut seen = BTreeSet::new();
        for file in &self.files {
            if !valid_relative_path(&file.path) {
                return Err(format!("{:?} is not a safe relative path", file.path));
            }
            if !is_sha256(&file.sha256) {
                return Err(format!("{} pins a malformed digest", file.path));
            }
            if file.bytes == 0 {
                return Err(format!("{} pins no bytes", file.path));
            }
            if !seen.insert(file.path.as_str()) {
                return Err(format!("{} is pinned twice", file.path));
            }
        }
        Ok(())
    }
}

/// Every manifest the daemon knows, keyed by model name.
#[derive(Clone, Debug, Default)]
pub(crate) struct ModelRegistry {
    models: Arc<RwLock<BTreeMap<String, Arc<ModelManifest>>>>,
}

impl ModelRegistry {
    /// The bundled registry alone.
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
            models.insert(manifest.name.clone(), Arc::new(manifest));
        }
        Ok(Self {
            models: Arc::new(RwLock::new(models)),
        })
    }

    /// The bundled registry, then every model the person pinned.
    ///
    /// A bundled manifest that cannot be read stops the daemon: it is shipped
    /// configuration, and running without it would silently key artifacts
    /// against less than the release pinned. A custom one is the person's own
    /// file, so a bad one is skipped and named in the log instead of taking
    /// every other model down with it.
    pub fn load_with_custom(bundled: &Path, custom: &Path) -> Result<Self, ModelError> {
        let registry = Self::load(bundled)?;
        for (path, loaded) in read_custom_manifests(custom) {
            match loaded.and_then(|manifest| {
                registry
                    .register(manifest)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            }) {
                Ok(()) => {}
                Err(detail) => {
                    tracing::warn!(path = %path.display(), detail, "skipping a pinned model");
                }
            }
        }
        Ok(registry)
    }

    pub fn get(&self, name: &str) -> Option<Arc<ModelManifest>> {
        self.read().get(name).cloned()
    }

    pub fn len(&self) -> usize {
        self.read().len()
    }

    /// Every manifest, in name order, as it stands now.
    pub fn manifests(&self) -> Vec<Arc<ModelManifest>> {
        self.read().values().cloned().collect()
    }

    /// What this daemon has pinned, for the startup log. Operators asking
    /// "which model produced this?" should not have to read TOML to find out.
    pub fn summaries(&self) -> Vec<String> {
        self.read().values().map(|model| model.summary()).collect()
    }

    /// Add a model the person pinned. The name must be new: replacing an
    /// existing entry would let one name stand for two sets of weights, and a
    /// task planned against the first would be keyed against the second.
    pub fn register(&self, mut manifest: ModelManifest) -> Result<Arc<ModelManifest>, ModelError> {
        manifest.validate().map_err(|detail| ModelError::Refused {
            name: manifest.name.clone(),
            detail,
        })?;
        manifest.origin = ModelOrigin::Custom;
        let mut models = self
            .models
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if models.contains_key(&manifest.name) {
            return Err(ModelError::Refused {
                name: manifest.name.clone(),
                detail: "a model with this name is already registered".to_owned(),
            });
        }
        let manifest = Arc::new(manifest);
        models.insert(manifest.name.clone(), Arc::clone(&manifest));
        Ok(manifest)
    }

    /// Forget a model the person pinned. Bundled models cannot be forgotten —
    /// they are part of the release — only their weights removed.
    pub fn unregister(&self, name: &str) -> Option<Arc<ModelManifest>> {
        let mut models = self
            .models
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if models
            .get(name)
            .is_some_and(|manifest| manifest.origin == ModelOrigin::Custom)
        {
            models.remove(name)
        } else {
            None
        }
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
            // Prefixed, because that is what the lease contract documents
            // and what the worker echoes into `producer.model_digest` — a
            // field the published speech schemas require to match
            // `^sha256:[0-9a-f]{64}$`. `Sha256Digest` displays bare hex, so
            // sending it unadorned refused every document a model-running
            // worker produced.
            digest: format!("sha256:{}", manifest.digest()),
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
    pub fn license_classes(&self) -> BTreeSet<String> {
        self.read()
            .values()
            .map(|manifest| manifest.license.class.clone())
            .collect()
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, BTreeMap<String, Arc<ModelManifest>>> {
        self.models
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Every `<name>.json` under the person's directory, parsed but not yet
/// admitted. Missing directory, missing models: a fresh install has pinned
/// nothing, and that is an answer rather than an error.
fn read_custom_manifests(directory: &Path) -> Vec<(PathBuf, Result<ModelManifest, String>)> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    found.sort();
    found
        .into_iter()
        .map(|path| {
            let parsed = fs::read(&path)
                .map_err(|error| error.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<ModelManifest>(&bytes)
                        .map_err(|error| error.to_string())
                })
                .and_then(|manifest| {
                    // The file is named for the model, so a renamed file cannot
                    // quietly register under a different identity.
                    let stem = path.file_stem().and_then(|stem| stem.to_str());
                    if stem == Some(manifest.name.as_str()) {
                        Ok(manifest)
                    } else {
                        Err("the file is not named for the model it pins".to_owned())
                    }
                });
            (path, parsed)
        })
        .collect()
}

/// Lowercase words joined by hyphens or dots, as `check-models.py` requires.
pub(crate) fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
        })
        && name
            .as_bytes()
            .first()
            .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
        && !name.ends_with(['-', '.'])
        && !name.contains("..")
}

/// A relative path that stays inside the model's directory.
pub(crate) fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 512
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains('\0')
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        })
}

pub(crate) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

pub(crate) fn is_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

#[derive(Debug, Error)]
pub(crate) enum ModelError {
    #[error("model manifest {path} is unreadable: {detail}")]
    Unreadable { path: PathBuf, detail: String },
    #[error("model {name} was refused: {detail}")]
    Refused { name: String, detail: String },
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::{fs, path::Path};

    use tempfile::TempDir;

    use super::{ModelOrigin, ModelRegistry, valid_name, valid_relative_path};

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
            ["permissive".to_owned()].into_iter().collect()
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
                .manifests()
                .into_iter()
                .filter(|manifest| manifest.capability == capability)
                .filter(|manifest| matches!(manifest.backend.as_str(), "cpu" | "onnx-cpu"))
                .count();
            assert!(
                fallbacks > 0,
                "{capability} is pinned only on accelerated backends"
            );
        }
    }

    /// The library lists every bundled model under a name a person can read,
    /// says what it is for, and marks what a fresh install should download.
    #[test]
    fn every_bundled_model_is_presented_in_the_catalog() {
        let registry = published_registry();
        for manifest in registry.manifests() {
            assert!(
                !manifest.catalog.title.is_empty(),
                "{} has no catalog title",
                manifest.name
            );
            assert!(
                !manifest.catalog.summary.is_empty(),
                "{} has no catalog summary",
                manifest.name
            );
            manifest.validate().expect("a bundled manifest is valid");
            assert_eq!(manifest.origin, ModelOrigin::Bundled);
        }
        for capability in ["vad", "asr", "forced-align", "detect-faces"] {
            assert!(
                registry
                    .manifests()
                    .iter()
                    .any(|manifest| manifest.capability == capability
                        && manifest.catalog.recommended
                        && matches!(manifest.backend.as_str(), "cpu" | "onnx-cpu")),
                "no recommended model runs {capability} on every machine"
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
        // Prefixed. The worker echoes this straight into
        // `producer.model_digest`, and every published speech schema requires
        // that field to match `^sha256:[0-9a-f]{64}$` — so a bare digest here
        // is a document the contract refuses, which is what it did.
        assert_eq!(
            binding.digest,
            format!(
                "sha256:{}",
                registry.get("silero-vad").expect("pinned").digest()
            ),
            "the worker echoes this as the producing model's identity"
        );
        assert!(binding.digest.starts_with("sha256:"));
        assert_eq!(binding.digest.len(), "sha256:".len() + 64);
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

        let mut reordered = (*model).clone();
        reordered.files.reverse();
        assert_eq!(reordered.digest(), baseline, "file order is not identity");

        let mut relabelled = (*model).clone();
        relabelled.family = "something else entirely".to_owned();
        relabelled.catalog.title = "A different name".to_owned();
        relabelled.catalog.summary = "A different sentence.".to_owned();
        assert_eq!(relabelled.digest(), baseline, "prose is not identity");

        let mut repinned = (*model).clone();
        repinned.files[0].sha256 = "0".repeat(64);
        assert_ne!(
            repinned.digest(),
            baseline,
            "a re-pinned weight is a new model"
        );

        let mut moved = (*model).clone();
        moved.source.revision = "f".repeat(40);
        assert_ne!(moved.digest(), baseline, "a new revision is a new model");
    }

    fn custom_manifest(name: &str) -> super::ModelManifest {
        let registry = published_registry();
        let mut manifest = (*registry.get("whisper-base").expect("pinned")).clone();
        manifest.name = name.to_owned();
        manifest.catalog.title = "Whisper Small".to_owned();
        manifest
    }

    /// A person's own model joins the registry while the daemon runs, and it
    /// can never take over a name something else was keyed against.
    #[test]
    fn a_custom_model_is_added_once_and_never_replaces_a_name() {
        let registry = published_registry();
        let before = registry.len();
        let added = registry
            .register(custom_manifest("whisper-small-custom"))
            .expect("a new name registers");
        assert_eq!(added.origin, ModelOrigin::Custom);
        assert_eq!(registry.len(), before + 1);

        // Clones of the registry share the map: the worker plane and the
        // planner see the addition without being told.
        let shared = registry.clone();
        assert!(shared.get("whisper-small-custom").is_some());

        assert!(
            registry
                .register(custom_manifest("whisper-small-custom"))
                .is_err(),
            "the same name twice is refused"
        );
        assert!(
            registry.register(custom_manifest("whisper-base")).is_err(),
            "a bundled name cannot be taken over"
        );
        assert!(
            registry.unregister("whisper-base").is_none(),
            "bundled models are part of the release"
        );
        assert!(registry.unregister("whisper-small-custom").is_some());
        assert!(registry.get("whisper-small-custom").is_none());
    }

    #[test]
    fn a_custom_manifest_that_could_escape_its_directory_is_refused() {
        let registry = published_registry();
        let mut escaping = custom_manifest("escaping");
        escaping.files[0].path = "../../etc/passwd".to_owned();
        assert!(registry.register(escaping).is_err());

        let mut unpinned = custom_manifest("unpinned");
        unpinned.source.revision = "main".to_owned();
        assert!(registry.register(unpinned).is_err());

        let mut malformed = custom_manifest("malformed");
        malformed.files[0].sha256 = "not-a-digest".to_owned();
        assert!(registry.register(malformed).is_err());

        assert!(registry.register(custom_manifest("Not A Name")).is_err());
    }

    /// Custom manifests are read at startup; one broken file is skipped and
    /// the rest still load, and a file named for another model is refused.
    #[test]
    fn custom_manifests_load_beside_the_bundled_registry() {
        let directory = TempDir::new().expect("temp");
        let good = custom_manifest("whisper-small-custom");
        fs::write(
            directory.path().join("whisper-small-custom.json"),
            serde_json::to_vec(&good).unwrap(),
        )
        .unwrap();
        fs::write(directory.path().join("broken.json"), b"{not json").unwrap();
        fs::write(
            directory.path().join("renamed.json"),
            serde_json::to_vec(&custom_manifest("something-else")).unwrap(),
        )
        .unwrap();
        let bundled = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry");
        let registry =
            ModelRegistry::load_with_custom(&bundled, directory.path()).expect("loads anyway");
        let loaded = registry.get("whisper-small-custom").expect("loaded");
        assert_eq!(loaded.origin, ModelOrigin::Custom);
        assert_eq!(loaded.catalog.title, "Whisper Small");
        assert!(registry.get("something-else").is_none());
        assert!(registry.get("whisper-base").is_some());
    }

    #[test]
    fn names_and_paths_are_held_to_the_registry_rules() {
        for good in ["whisper-base", "qwen3.5-9b", "a", "x-1.2"] {
            assert!(valid_name(good), "{good}");
        }
        for bad in [
            "",
            "-leading",
            "trailing-",
            "Upper",
            "has space",
            "a/b",
            "a..b",
        ] {
            assert!(!valid_name(bad), "{bad}");
        }
        for good in ["model.safetensors", "onnx/model.onnx", "a_b-c.1/x.json"] {
            assert!(valid_relative_path(good), "{good}");
        }
        for bad in [
            "",
            "/abs",
            "../up",
            "a/../b",
            "a//b",
            "back\\slash",
            "sp ace",
            "./x",
        ] {
            assert!(!valid_relative_path(bad), "{bad}");
        }
    }
}
