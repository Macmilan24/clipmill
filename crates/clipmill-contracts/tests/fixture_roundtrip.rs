//! The Phase 0 contracts exit gate, Rust leg: fixtures parse into the
//! generated types, invalid fixtures are rejected at the type level, and
//! canonical serialization round-trips byte-for-byte.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::proto::ipc::v1::{
    DemoDagPayloadV1, DeviceProfilePayloadV1, IngestSourcePayloadV1, PingRequest,
    ProbeSourcePayloadV1,
};
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
fn valid_media_fixtures_parse_and_roundtrip_canonically() {
    fn roundtrip<T: serde::de::DeserializeOwned + serde::Serialize>(rel: &str) {
        let raw = read(rel);
        let parsed: T = match serde_json::from_str(&raw) {
            Ok(parsed) => parsed,
            Err(err) => panic!("valid fixture {rel} rejected: {err}"),
        };
        let reserialized = match serde_json::to_value(&parsed) {
            Ok(value) => value,
            Err(err) => panic!("reserialize failed for {rel}: {err}"),
        };
        assert_eq!(
            canonical(&reserialized),
            raw,
            "canonical round-trip must be byte-identical for {rel}"
        );
    }
    use clipmill_contracts::schemas::{
        media_audio::MediaAudio, media_audio_peaks::MediaAudioPeaks,
        media_filmstrip::MediaFilmstrip, media_frames::MediaFrames,
        media_ingest_manifest::MediaIngestManifest, media_loudness_envelope::MediaLoudnessEnvelope,
        media_proxy::MediaProxy, media_reference_index::MediaReferenceIndex,
    };
    roundtrip::<MediaProxy>("contracts/fixtures/media.proxy/valid/minimal.json");
    roundtrip::<MediaAudio>("contracts/fixtures/media.audio/valid/minimal.json");
    roundtrip::<MediaLoudnessEnvelope>(
        "contracts/fixtures/media.loudness_envelope/valid/minimal.json",
    );
    roundtrip::<MediaReferenceIndex>("contracts/fixtures/media.reference_index/valid/minimal.json");
    roundtrip::<MediaFilmstrip>("contracts/fixtures/media.filmstrip/valid/minimal.json");
    roundtrip::<MediaAudioPeaks>("contracts/fixtures/media.audio_peaks/valid/minimal.json");
    roundtrip::<MediaFrames>("contracts/fixtures/media.frames/valid/minimal.json");
    roundtrip::<MediaIngestManifest>("contracts/fixtures/media.ingest_manifest/valid/minimal.json");
}

#[test]
fn render_manifest_fixtures_state_what_was_produced() {
    use clipmill_contracts::schemas::render_clip::{
        RenderClipManifest, RenderClipManifestDeterminism as Determinism,
        RenderClipManifestProgramSegmentsItemLayout as Layout,
    };
    for rel in [
        "contracts/fixtures/render.clip/valid/first_slice.json",
        "contracts/fixtures/render.clip/valid/model_assisted.json",
    ] {
        let raw = read(rel);
        let parsed: RenderClipManifest = match serde_json::from_str(&raw) {
            Ok(parsed) => parsed,
            Err(err) => panic!("valid fixture {rel} rejected: {err}"),
        };
        let reserialized = match serde_json::to_value(&parsed) {
            Ok(value) => value,
            Err(err) => panic!("reserialize failed for {rel}: {err}"),
        };
        assert_eq!(
            canonical(&reserialized),
            raw,
            "canonical round-trip must be byte-identical for {rel}"
        );
        // Every published file carries a digest a recipient can verify, and
        // the loudness figures are measurements rather than restated targets.
        assert!(!parsed.outputs.is_empty());
        assert!(parsed.program.frame_count > 0);
        // The measurement is a reading, not the target echoed back.
        assert!(
            (parsed.loudness.measured_output.integrated_lufs - parsed.loudness.target_lufs).abs()
                > f64::EPSILON
        );
    }

    let first_slice: RenderClipManifest = serde_json::from_str(&read(
        "contracts/fixtures/render.clip/valid/first_slice.json",
    ))
    .unwrap_or_else(|err| panic!("{err}"));
    assert!(matches!(first_slice.determinism, Determinism::ByteStable));
    // A hand-authored document discloses nothing, because nothing was modelled.
    assert!(first_slice.ai_use_summary.assistance.is_empty());
    assert!(first_slice.ai_use_summary.generated.is_empty());
    assert!(!first_slice.ai_use_summary.requires_youtube_ai_disclosure);
    assert!(
        first_slice
            .program
            .segments
            .iter()
            .all(|segment| matches!(segment.layout, Layout::Fit))
    );

    let assisted: RenderClipManifest = serde_json::from_str(&read(
        "contracts/fixtures/render.clip/valid/model_assisted.json",
    ))
    .unwrap_or_else(|err| panic!("{err}"));
    let assistance = assisted
        .ai_use_summary
        .assistance
        .iter()
        .map(|item| item.as_str())
        .collect::<Vec<_>>();
    assert_eq!(assistance, ["asr_captions", "reframe"]);
    assert!(assisted.ai_use_summary.generated.is_empty());
}

#[test]
fn invalid_render_manifest_fixtures_are_rejected() {
    use clipmill_contracts::schemas::render_clip::RenderClipManifest;
    // `no-outputs` belongs to the schema and the Python leg: typify enforces
    // string patterns and enums through newtypes, but not array bounds, so
    // asserting it here would test a claim this type does not make.
    for (fixture, why) in [
        (
            "float-frame-count",
            "a fractional frame count must not parse",
        ),
        (
            "unknown-determinism",
            "an unlisted determinism class must not parse",
        ),
        (
            "unprefixed-output-digest",
            "an output digest without its algorithm must not parse",
        ),
    ] {
        let rejected = serde_json::from_str::<RenderClipManifest>(&read(&format!(
            "contracts/fixtures/render.clip/invalid/{fixture}.json"
        )));
        assert!(rejected.is_err(), "{why}");
    }
}

#[test]
fn invalid_media_fixtures_are_rejected() {
    use clipmill_contracts::schemas::{
        media_audio::MediaAudio, media_frames::MediaFrames,
        media_ingest_manifest::MediaIngestManifest, media_proxy::MediaProxy,
    };
    let rejected_proxy = serde_json::from_str::<MediaProxy>(&read(
        "contracts/fixtures/media.proxy/invalid/float-ticks.json",
    ));
    assert!(rejected_proxy.is_err(), "float ticks must not parse (D06)");
    let rejected_audio = serde_json::from_str::<MediaAudio>(&read(
        "contracts/fixtures/media.audio/invalid/wrong-codec.json",
    ));
    assert!(rejected_audio.is_err(), "non-PCM codec must not parse");
    let rejected_frames = serde_json::from_str::<MediaFrames>(&read(
        "contracts/fixtures/media.frames/invalid/missing-coverage.json",
    ));
    assert!(rejected_frames.is_err(), "missing coverage must not parse");
    let rejected_manifest = serde_json::from_str::<MediaIngestManifest>(&read(
        "contracts/fixtures/media.ingest_manifest/invalid/unknown-kind.json",
    ));
    assert!(rejected_manifest.is_err(), "unknown kind must not parse");
}

#[test]
fn ingest_source_payload_fixtures_enforce_the_w11_key_version() {
    let valid: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/ingest_source/valid/payload.json",
    ))
    .expect("valid ingest fixture JSON");
    let message = IngestSourcePayloadV1 {
        key_version: valid["keyVersion"].as_str().unwrap_or_default().to_owned(),
        source_id: valid["sourceId"].as_str().unwrap_or_default().to_owned(),
    };
    assert_eq!(message.key_version, "clipmill.ingest-source.v1");
    assert!(message.source_id.starts_with("src_"));
    assert_eq!(
        IngestSourcePayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip"),
        message
    );

    let invalid: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/ingest_source/invalid/wrong-version.json",
    ))
    .expect("invalid ingest fixture remains syntactically valid JSON");
    assert_ne!(
        invalid["keyVersion"].as_str().unwrap_or_default(),
        "clipmill.ingest-source.v1"
    );
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

#[test]
fn demo_dag_payload_fixtures_enforce_the_phase0_key_version() {
    let valid: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/demo_dag/valid/payload.json",
    ))
    .expect("valid demo fixture JSON");
    let message = DemoDagPayloadV1 {
        key_version: valid["keyVersion"].as_str().unwrap_or_default().to_owned(),
        seed: b"seed-40".to_vec(),
    };
    assert_eq!(message.key_version, "clipmill.demo-dag.v1");
    assert_eq!(
        DemoDagPayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip"),
        message
    );

    let invalid: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/demo_dag/invalid/wrong-version.json",
    ))
    .expect("invalid demo fixture remains syntactically valid JSON");
    assert_ne!(
        invalid["keyVersion"].as_str().unwrap_or_default(),
        "clipmill.demo-dag.v1"
    );
}

#[test]
fn probe_source_payload_fixtures_enforce_the_w5_key_version() {
    let valid: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/probe_source/valid/payload.json",
    ))
    .expect("valid probe fixture JSON");
    let message = ProbeSourcePayloadV1 {
        key_version: valid["keyVersion"].as_str().unwrap_or_default().to_owned(),
        source_id: valid["sourceId"].as_str().unwrap_or_default().to_owned(),
    };
    assert_eq!(message.key_version, "clipmill.probe-source.v1");
    assert!(message.source_id.starts_with("src_"));
    assert_eq!(
        ProbeSourcePayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip"),
        message
    );

    let invalid: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/probe_source/invalid/wrong-version.json",
    ))
    .expect("invalid probe fixture remains syntactically valid JSON");
    assert_ne!(
        invalid["keyVersion"].as_str().unwrap_or_default(),
        "clipmill.probe-source.v1"
    );
}

#[test]
fn device_profile_payload_fixtures_enforce_the_w7_key_version() {
    let valid: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/device_profile/valid/payload.json",
    ))
    .expect("valid device-profile fixture JSON");
    let message = DeviceProfilePayloadV1 {
        key_version: valid["keyVersion"].as_str().unwrap_or_default().to_owned(),
        hardware_fingerprint: valid["hardwareFingerprint"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        measurement_generation: valid["measurementGeneration"]
            .as_str()
            .unwrap_or_default()
            .parse()
            .expect("generation"),
    };
    assert_eq!(message.key_version, "clipmill.device-profile.v1");
    assert!(message.hardware_fingerprint.starts_with("sha256:"));
    assert_eq!(message.measurement_generation, 1);
    assert_eq!(
        DeviceProfilePayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip"),
        message
    );

    let invalid: serde_json::Value = serde_json::from_str(&read(
        "contracts/fixtures/proto/device_profile/invalid/wrong-version.json",
    ))
    .expect("invalid device-profile fixture remains syntactically valid JSON");
    assert_ne!(
        invalid["keyVersion"].as_str().unwrap_or_default(),
        "clipmill.device-profile.v1"
    );
}
