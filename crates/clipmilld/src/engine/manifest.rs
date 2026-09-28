//! `engine/engine.json`: what a packaged app ships for its engine.
//!
//! The release build writes it beside the packages it names. Every part is one
//! worker: its own packages as wheels (pinned by SHA-256 here), the third-party
//! wheels it needs as a requirements file whose every line carries hashes, and
//! the platforms it runs on. Nothing in it is taken on trust that a check here
//! can refuse: names and commands are plain words, paths stay inside the
//! engine directory, and digests are hex.

use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: &str = "clipmill.engine.v1";
/// The largest manifest read: a few dozen parts, generously.
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    schema_version: String,
    /// The Python version every part runs on, as `uv python install` names it.
    pub python: String,
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Part {
    /// Its name, e.g. `vad`.
    pub name: String,
    /// `worker`, kept running and leased tasks, or `tool`, run by the
    /// daemon when a task needs it (`YouTube` import).
    #[serde(default = "worker_kind")]
    pub kind: String,
    /// What it does, for a person.
    pub title: String,
    /// The worker family its tasks are leased to.
    pub family: String,
    /// The console script that starts it.
    pub command: String,
    /// The module imported to check an install before it is used.
    pub module: String,
    /// Where it runs, as [`platform`] names them.
    pub platforms: Vec<String>,
    /// Third-party wheels, relative to the engine directory, every line
    /// pinned by hash.
    pub requirements: String,
    pub requirements_sha256: String,
    /// ClipMill's own packages for it, installed from the app itself.
    pub wheels: Vec<Wheel>,
    /// What installing it downloads, per platform, as measured when the app
    /// was built.
    #[serde(default)]
    pub download_bytes: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Wheel {
    pub file: String,
    pub sha256: String,
}

fn worker_kind() -> String {
    WORKER.to_owned()
}

/// A part kept running as a worker.
pub(crate) const WORKER: &str = "worker";
/// A part the daemon runs itself when a task needs it.
pub(crate) const TOOL: &str = "tool";

impl Manifest {
    /// Read and check the manifest in `engine_dir`.
    pub(crate) fn load(engine_dir: &Path) -> Result<Self, String> {
        let path = engine_dir.join("engine.json");
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("the app's list of components cannot be read: {error}"))?;
        if metadata.len() > MAX_MANIFEST_BYTES {
            return Err("the app's list of components is too large to be one".to_owned());
        }
        let bytes = fs::read(&path)
            .map_err(|error| format!("the app's list of components cannot be read: {error}"))?;
        let manifest: Self = serde_json::from_slice(&bytes)
            .map_err(|error| format!("the app's list of components is not valid: {error}"))?;
        manifest.check()?;
        Ok(manifest)
    }

    fn check(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(format!(
                "the app's list of components is {}, and this daemon reads {SCHEMA_VERSION}",
                self.schema_version
            ));
        }
        if !is_version(&self.python) {
            return Err("the engine's Python version is not a version".to_owned());
        }
        let mut names = std::collections::BTreeSet::new();
        for part in &self.parts {
            if !is_word(&part.name) || !names.insert(part.name.as_str()) {
                return Err(format!(
                    "the engine part {:?} is not a usable name",
                    part.name
                ));
            }
            if part.kind != WORKER && part.kind != TOOL {
                return Err(format!(
                    "the engine part {} is neither a worker nor a tool",
                    part.name
                ));
            }
            if !is_word(&part.family) || !is_word(&part.command) || !is_module(&part.module) {
                return Err(format!(
                    "the engine part {} names no usable worker",
                    part.name
                ));
            }
            if part.title.trim().is_empty() || part.title.chars().count() > 80 {
                return Err(format!("the engine part {} has no usable title", part.name));
            }
            if part.platforms.is_empty() || part.platforms.iter().any(|value| !is_platform(value)) {
                return Err(format!("the engine part {} names no platform", part.name));
            }
            if !is_inside(&part.requirements) || !is_digest(&part.requirements_sha256) {
                return Err(format!(
                    "the engine part {} has no pinned requirements",
                    part.name
                ));
            }
            if part.wheels.is_empty()
                || part
                    .wheels
                    .iter()
                    .any(|wheel| !is_inside(&wheel.file) || !is_digest(&wheel.sha256))
            {
                return Err(format!(
                    "the engine part {} has no pinned wheels",
                    part.name
                ));
            }
        }
        Ok(())
    }

    /// The parts that run on this computer, in the manifest's order.
    pub(crate) fn parts_here(&self) -> impl Iterator<Item = &Part> {
        self.parts
            .iter()
            .filter(|part| part.platforms.iter().any(|value| value == platform()))
    }
}

impl Part {
    /// Whether the daemon keeps it running as a worker.
    pub(crate) fn is_worker(&self) -> bool {
        self.kind == WORKER
    }

    /// What an install of this part is: the Python it runs on and the bytes
    /// of everything it installs. Two installs with one digest are the same
    /// install, so an app update that changed nothing here reinstalls nothing.
    pub(crate) fn digest(&self, python: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"clipmill.engine.part.v1\0");
        hasher.update(python.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.requirements_sha256.as_bytes());
        for wheel in &self.wheels {
            hasher.update(b"\0");
            hasher.update(wheel.sha256.as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    /// What installing it downloads here, when the build measured it.
    pub(crate) fn download_bytes_here(&self) -> u64 {
        self.download_bytes.get(platform()).copied().unwrap_or(0)
    }
}

/// This computer, as the manifest names platforms.
pub(crate) const fn platform() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-arm64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "macos-x86_64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x86_64"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "linux-arm64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x86_64"
    } else {
        "unsupported"
    }
}

/// A file the manifest names, resolved inside the engine directory.
pub(crate) fn resolve(engine_dir: &Path, relative: &str) -> PathBuf {
    engine_dir.join(relative)
}

/// The SHA-256 of a file, as lowercase hex.
pub(crate) fn file_sha256(path: &Path) -> std::io::Result<String> {
    use std::io::Read as _;
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Lowercase letters, digits and hyphens, as names, families and commands are.
fn is_word(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
}

/// A platform as [`platform`] names one: `linux-x86_64` has an underscore.
fn is_platform(value: &str) -> bool {
    (1..=32).contains(&value.len())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
}

fn is_module(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
}

fn is_version(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    (2..=3).contains(&parts.len())
        && parts.iter().all(|part| {
            !part.is_empty() && part.len() <= 4 && part.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A relative path with only normal components: nothing absolute, nothing
/// that climbs out.
fn is_inside(relative: &str) -> bool {
    let path = Path::new(relative);
    !relative.is_empty()
        && relative.len() <= 256
        && !relative.contains('\\')
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn manifest(parts: &str) -> String {
        format!(r#"{{"schema_version":"clipmill.engine.v1","python":"3.12.13","parts":[{parts}]}}"#)
    }

    fn part(name: &str, requirements: &str) -> String {
        let digest = "a".repeat(64);
        format!(
            r#"{{"name":"{name}","title":"Speech detection","family":"speech-vad","command":"clipmill-worker-vad","module":"clipmill_worker_vad","platforms":["{}"],"requirements":"{requirements}","requirements_sha256":"{digest}","wheels":[{{"file":"wheels/x.whl","sha256":"{digest}"}}]}}"#,
            platform()
        )
    }

    fn load(text: &str) -> Result<Manifest, String> {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("engine.json"), text).unwrap();
        Manifest::load(dir.path())
    }

    #[test]
    fn a_manifest_names_the_parts_that_run_here() {
        let loaded = load(&manifest(&part("vad", "requirements/vad.txt"))).unwrap();
        assert_eq!(loaded.python, "3.12.13");
        assert_eq!(loaded.parts_here().count(), 1);
    }

    #[test]
    fn a_path_that_climbs_out_of_the_engine_is_refused() {
        for bad in ["../vad.txt", "/etc/passwd", "requirements\\\\vad.txt", ""] {
            let refused = load(&manifest(&part("vad", bad)));
            assert!(refused.is_err(), "{bad:?} should be refused");
        }
    }

    #[test]
    fn every_platform_the_daemon_names_is_one_a_manifest_may_list() {
        let digest = "a".repeat(64);
        let part = format!(
            r#"{{"name":"vad","title":"Speech detection","family":"speech-vad","command":"clipmill-worker-vad","module":"clipmill_worker_vad","platforms":["macos-arm64","macos-x86_64","linux-x86_64","linux-arm64","windows-x86_64"],"requirements":"r.txt","requirements_sha256":"{digest}","wheels":[{{"file":"w.whl","sha256":"{digest}"}}]}}"#
        );
        let loaded = load(&manifest(&part)).unwrap();
        assert_eq!(
            loaded.parts_here().count(),
            1,
            "this computer is one of them"
        );
        assert!(load(&manifest(&part.replace("linux-arm64", "Linux ARM"))).is_err());
    }

    #[test]
    fn names_are_plain_words_and_unique() {
        assert!(load(&manifest(&part("Vad", "r.txt"))).is_err());
        assert!(load(&manifest(&part("vad; rm", "r.txt"))).is_err());
        let twice = format!("{},{}", part("vad", "r.txt"), part("vad", "r.txt"));
        assert!(load(&manifest(&twice)).is_err());
    }

    #[test]
    fn another_schema_is_refused_by_name() {
        let other = manifest(&part("vad", "r.txt")).replace("engine.v1", "engine.v9");
        let error = load(&other).unwrap_err();
        assert!(error.contains("clipmill.engine.v9"), "{error}");
    }

    #[test]
    fn a_part_digest_changes_with_python_and_with_every_package() {
        let loaded = load(&manifest(&part("vad", "r.txt"))).unwrap();
        let vad = &loaded.parts[0];
        let base = vad.digest("3.12.13");
        assert_eq!(base, vad.digest("3.12.13"));
        assert_ne!(base, vad.digest("3.12.14"));
        let mut changed = vad.clone();
        changed.wheels[0].sha256 = "b".repeat(64);
        assert_ne!(base, changed.digest("3.12.13"));
        changed = vad.clone();
        changed.requirements_sha256 = "c".repeat(64);
        assert_ne!(base, changed.digest("3.12.13"));
    }
}
