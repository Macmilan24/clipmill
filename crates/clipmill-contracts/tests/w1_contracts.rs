//! W1 contract fixtures, Rust leg: source map, device profile, and the
//! shared-memory descriptor (which exercises the cross-package proto
//! import, clipmill.shm.v1 -> clipmill.time.v1).
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::proto::{
    shm::v1::{BufferDescriptor, DataType, TransportType},
    worker::v1::{CapabilityDescriptor, WorkerRequest, worker_request},
};
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
    for fixture in ["minimal.json", "with-mapping.json"] {
        let raw = read(&format!("contracts/fixtures/source_map/valid/{fixture}"));
        let map: SourceMap = serde_json::from_str(&raw).expect("valid fixture rejected");
        assert_eq!(canonical(&serde_json::to_value(&map).unwrap()), raw);
    }
}

#[test]
fn source_map_invalid_rejected() {
    for fixture in ["float-ticks.json", "bad-mapping-timebase.json"] {
        let raw = read(&format!("contracts/fixtures/source_map/invalid/{fixture}"));
        assert!(serde_json::from_str::<SourceMap>(&raw).is_err());
    }
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

#[test]
fn w6_additive_worker_and_shared_memory_contracts_roundtrip() {
    let descriptor_fixture: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/shm/buffer_descriptor_w6.json",
    ))
    .unwrap();
    let descriptor = BufferDescriptor {
        shm_name: descriptor_fixture["shm_name"].as_str().unwrap().to_owned(),
        shape: vec![4],
        dtype: DataType::U8 as i32,
        colorspace: String::new(),
        timebase: Some(clipmill_contracts::proto::time::v1::Timebase {
            num: 1,
            den: 90_000,
        }),
        byte_len: 4,
        sha256: descriptor_fixture["sha256"].as_str().unwrap().to_owned(),
        lease_id: descriptor_fixture["lease_id"].as_str().unwrap().to_owned(),
        transport_type: TransportType::ScmRightsMemfd as i32,
        handle_token: descriptor_fixture["handle_token"]
            .as_str()
            .unwrap()
            .to_owned(),
    };
    let decoded = BufferDescriptor::decode(descriptor.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded.transport_type(), TransportType::ScmRightsMemfd);
    assert!(!decoded.handle_token.is_empty());

    let capability = CapabilityDescriptor {
        worker_id: "wrk_01J00000000000000000000000".to_owned(),
        family: "echo".to_owned(),
        capabilities: vec!["demo-seed".to_owned()],
        protocol_version: "1.1".to_owned(),
        backend: "cpu".to_owned(),
        max_memory_bytes: 268_435_456,
        public_key: vec![0; 32],
        signature: vec![0; 64],
    };
    let request = WorkerRequest {
        body: Some(worker_request::Body::Register(
            clipmill_contracts::proto::worker::v1::RegisterWorker {
                descriptor: Some(capability),
            },
        )),
    };
    assert_eq!(
        WorkerRequest::decode(request.encode_to_vec().as_slice()).unwrap(),
        request
    );
}
