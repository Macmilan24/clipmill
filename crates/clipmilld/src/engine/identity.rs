//! Keys for the workers the daemon starts itself.
//!
//! A worker proves who it is by signing the daemon's challenge, and the daemon
//! reads the public keys it trusts once, when it starts. So the keys of every
//! worker the engine may start are made before that read: one private identity
//! per part, readable only by this user, and its public half in the trust
//! directory. A key that no longer matches its trusted half is replaced rather
//! than repaired, since nothing else can have been signed with it.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clipmill_core::WorkerId;
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

use crate::library::write_private;

const KEY_VERSION: &str = "clipmill.worker.identity.v1";

#[derive(Debug, Deserialize, Serialize)]
struct IdentityFile {
    key_version: String,
    private_key: String,
    worker_id: String,
}

/// Where a part's private identity lives.
pub(crate) fn identity_path(identity_dir: &Path, part: &str) -> PathBuf {
    identity_dir.join(format!("{part}.json"))
}

/// Make sure `part` has a private identity whose public half is trusted,
/// and return where the identity is.
pub(crate) fn enrol(identity_dir: &Path, trust_dir: &Path, part: &str) -> Result<PathBuf, String> {
    create_private_dir(identity_dir)?;
    create_private_dir(trust_dir)?;
    let path = identity_path(identity_dir, part);
    if let Some(existing) = read_identity(&path)
        && trusted(trust_dir, &existing)
    {
        return Ok(path);
    }
    if let Some(stale) = read_identity(&path) {
        // Its public half is missing or different: nothing trusts this key,
        // so it is replaced along with whatever the trust store held for it.
        let _ = fs::remove_file(trust_dir.join(format!("{}.pub", stale.worker_id)));
    }

    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed)
        .map_err(|error| format!("no randomness for a worker key: {error}"))?;
    let key = SigningKey::from_bytes(&seed);
    let worker_id = WorkerId::new().to_string();
    let identity = IdentityFile {
        key_version: KEY_VERSION.to_owned(),
        private_key: hex::encode(seed),
        worker_id: worker_id.clone(),
    };
    let mut bytes = serde_json::to_vec_pretty(&identity).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    write_private(&path, &bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    let public = format!("{}\n", hex::encode(key.verifying_key().to_bytes()));
    let trust = trust_dir.join(format!("{worker_id}.pub"));
    write_private(&trust, public.as_bytes())
        .map_err(|error| format!("{}: {error}", trust.display()))?;
    Ok(path)
}

fn read_identity(path: &Path) -> Option<IdentityFile> {
    let bytes = fs::read(path).ok()?;
    let identity: IdentityFile = serde_json::from_slice(&bytes).ok()?;
    (identity.key_version == KEY_VERSION && identity.worker_id.parse::<WorkerId>().is_ok())
        .then_some(identity)
}

/// Whether the trust store holds exactly this identity's public half.
fn trusted(trust_dir: &Path, identity: &IdentityFile) -> bool {
    let Ok(seed) = hex::decode(&identity.private_key) else {
        return false;
    };
    let Ok(seed) = <[u8; 32]>::try_from(seed.as_slice()) else {
        return false;
    };
    let expected = hex::encode(SigningKey::from_bytes(&seed).verifying_key().to_bytes());
    fs::read_to_string(trust_dir.join(format!("{}.pub", identity.worker_id)))
        .is_ok_and(|text| text.trim() == expected)
}

fn create_private_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| format!("{}: {error}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn a_part_is_enrolled_once_and_kept() {
        let dir = tempfile::tempdir().unwrap();
        let (identities, trust) = (dir.path().join("id"), dir.path().join("trust"));
        let first = enrol(&identities, &trust, "vad").unwrap();
        let before = fs::read(&first).unwrap();
        let again = enrol(&identities, &trust, "vad").unwrap();
        assert_eq!(first, again);
        assert_eq!(before, fs::read(&again).unwrap(), "a trusted key is kept");
        assert_eq!(fs::read_dir(&trust).unwrap().count(), 1);
    }

    #[test]
    fn a_key_whose_public_half_is_gone_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let (identities, trust) = (dir.path().join("id"), dir.path().join("trust"));
        let path = enrol(&identities, &trust, "vad").unwrap();
        let before = read_identity(&path).unwrap().worker_id;
        for entry in fs::read_dir(&trust).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
        enrol(&identities, &trust, "vad").unwrap();
        let after = read_identity(&path).unwrap();
        assert_ne!(before, after.worker_id);
        assert!(trusted(&trust, &after));
    }

    #[test]
    fn the_trusted_half_is_one_the_daemon_accepts() {
        let dir = tempfile::tempdir().unwrap();
        let (identities, trust) = (dir.path().join("id"), dir.path().join("trust"));
        enrol(&identities, &trust, "vad").unwrap();
        enrol(&identities, &trust, "faces").unwrap();
        // The worker service's own reader: private, single-line, hex.
        let keys = crate::worker::load_trust_for_tests(&trust).unwrap();
        assert_eq!(keys.len(), 2);
    }
}
