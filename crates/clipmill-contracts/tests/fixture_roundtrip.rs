//! The Phase 0 contracts exit gate, Rust leg: fixtures parse into the
//! generated types, invalid fixtures are rejected at the type level, and
//! canonical serialization round-trips byte-for-byte.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::proto::ipc::v1::PingRequest;
use clipmill_contracts::schemas::artifact_manifest::ArtifactManifest;
use prost::Message;

fn repo_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn read(rel: &str) -> String {
    let path = repo_path(rel);
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => panic!("cannot read {}: {err}", path.display()),
    }
}

/// Canonical JSON: sorted keys (`serde_json`'s default map is ordered),
/// two-space indent, trailing newline.
fn canonical(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_else(|err| panic!("{err}"));
    text.push('\n');
    text
}

#[test]
fn valid_manifest_parses_and_roundtrips_canonically() {
    for rel in [
        "contracts/fixtures/artifact.manifest/valid/minimal.json",
        "contracts/fixtures/artifact.manifest/valid/with-recipe.json",
    ] {
        let raw = read(rel);
        let manifest: ArtifactManifest = match serde_json::from_str(&raw) {
            Ok(manifest) => manifest,
            Err(err) => panic!("valid fixture {rel} rejected: {err}"),
        };
        let reserialized = match serde_json::to_value(&manifest) {
            Ok(value) => value,
            Err(err) => panic!("reserialize failed: {err}"),
        };
        assert_eq!(
            canonical(&reserialized),
            raw,
            "canonical round-trip must be byte-identical for {rel}"
        );
    }
}

#[test]
fn invalid_manifests_are_rejected() {
    for rel in [
        "contracts/fixtures/artifact.manifest/invalid/float-seconds.json",
        "contracts/fixtures/artifact.manifest/invalid/missing-policy.json",
        "contracts/fixtures/artifact.manifest/invalid/recipe-config-array.json",
        "contracts/fixtures/artifact.manifest/invalid/recipe-missing-semantic-version.json",
    ] {
        let raw = read(rel);
        let parsed = serde_json::from_str::<ArtifactManifest>(&raw);
        assert!(
            parsed.is_err(),
            "{rel} must fail to parse into the typed contract"
        );
    }
}

#[test]
fn ping_binpb_fixture_roundtrips() {
    let path = repo_path("contracts/fixtures/proto/ping/ping_request.binpb");
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) => panic!("cannot read {}: {err}", path.display()),
    };
    let decoded = match PingRequest::decode(bytes.as_slice()) {
        Ok(message) => message,
        Err(err) => panic!("decode failed: {err}"),
    };
    let twin: serde_json::Value =
        match serde_json::from_str(&read("contracts/fixtures/proto/ping/ping_request.json")) {
            Ok(value) => value,
            Err(err) => panic!("twin parse failed: {err}"),
        };
    assert_eq!(decoded.echo, twin["echo"].as_str().unwrap_or_default());
    assert_eq!(
        decoded.encode_to_vec(),
        bytes,
        "re-encode must be byte-identical"
    );
}
