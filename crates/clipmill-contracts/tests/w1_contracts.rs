//! W1 contract fixtures, Rust leg: source map, device profile, and the
//! shared-memory descriptor (which exercises the cross-package proto
//! import, clipmill.shm.v1 -> clipmill.time.v1).
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::proto::shm::v1::{BufferDescriptor, DataType};
use clipmill_contracts::schemas::device_profile::DeviceProfile;
use clipmill_contracts::schemas::source_map::SourceMap;
use prost::Message;

fn repo_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel)).unwrap()
}

fn canonical(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap();
    text.push('\n');
    text
}

#[test]
fn source_map_valid_roundtrips_canonically() {
    let raw = read("contracts/fixtures/source_map/valid/minimal.json");
    let map: SourceMap = serde_json::from_str(&raw).expect("valid fixture rejected");
    assert_eq!(canonical(&serde_json::to_value(&map).unwrap()), raw);
}

#[test]
fn source_map_invalid_rejected() {
    let raw = read("contracts/fixtures/source_map/invalid/float-ticks.json");
    assert!(serde_json::from_str::<SourceMap>(&raw).is_err());
}

#[test]
fn device_profile_valid_roundtrips_canonically() {
    let raw = read("contracts/fixtures/device_profile/valid/minimal.json");
    let profile: DeviceProfile = serde_json::from_str(&raw).expect("valid fixture rejected");
    assert_eq!(canonical(&serde_json::to_value(&profile).unwrap()), raw);
}

#[test]
fn device_profile_invalid_rejected() {
    let raw = read("contracts/fixtures/device_profile/invalid/missing-measured.json");
    assert!(serde_json::from_str::<DeviceProfile>(&raw).is_err());
}

#[test]
fn shm_descriptor_binpb_roundtrips_with_cross_package_timebase() {
    let bytes = std::fs::read(repo_path(
        "contracts/fixtures/proto/shm/buffer_descriptor.binpb",
    ))
    .unwrap();
    let twin: serde_json::Value =
        serde_json::from_str(&read("contracts/fixtures/proto/shm/buffer_descriptor.json")).unwrap();

    let decoded = BufferDescriptor::decode(bytes.as_slice()).expect("decode failed");
    assert_eq!(decoded.shm_name, twin["shm_name"].as_str().unwrap());
    assert_eq!(decoded.byte_len, twin["byte_len"].as_u64().unwrap());
    assert_eq!(decoded.dtype(), DataType::U8);
    let timebase = decoded.timebase.as_ref().expect("timebase missing");
    assert_eq!(timebase.num, twin["timebase"]["num"].as_i64().unwrap());
    assert_eq!(timebase.den, twin["timebase"]["den"].as_i64().unwrap());
    assert_eq!(
        decoded.encode_to_vec(),
        bytes,
        "re-encode must be byte-identical"
    );
}
