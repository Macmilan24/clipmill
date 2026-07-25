"""The Phase 0 contracts exit gate, Python leg."""

import importlib
import json
from pathlib import Path

import pytest
from clipmill.ipc.v1 import daemon_pb2, ping_pb2
from clipmill_worker_sdk.gen.schemas.artifact_manifest import ArtifactManifest
from clipmill_worker_sdk.gen.schemas.source_map import SourceMap
from google.protobuf import json_format
from pydantic import ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures"


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


def test_valid_manifest_parses_and_roundtrips_canonically() -> None:
    raw = (FIXTURES / "artifact.manifest" / "valid" / "minimal.json").read_text()
    manifest = ArtifactManifest.model_validate_json(raw)
    reserialized = manifest.model_dump(mode="json", exclude_none=True)
    assert canonical(reserialized) == raw, "canonical round-trip must be byte-identical"


@pytest.mark.parametrize("name", ["float-seconds.json", "missing-policy.json"])
def test_invalid_manifests_are_rejected(name: str) -> None:
    raw = (FIXTURES / "artifact.manifest" / "invalid" / name).read_text()
    with pytest.raises(ValidationError):
        ArtifactManifest.model_validate_json(raw)


def test_ping_binpb_fixture_roundtrips() -> None:
    binpb = (FIXTURES / "proto" / "ping" / "ping_request.binpb").read_bytes()
    twin = json.loads((FIXTURES / "proto" / "ping" / "ping_request.json").read_text())
    message = ping_pb2.PingRequest.FromString(binpb)
    assert message.echo == twin["echo"]
    assert message.SerializeToString(deterministic=True) == binpb


def test_demo_dag_payload_fixtures_enforce_the_phase0_key_version() -> None:
    valid = json.loads((FIXTURES / "proto" / "demo_dag" / "valid" / "payload.json").read_text())
    message = json_format.ParseDict(valid, daemon_pb2.DemoDagPayloadV1())
    assert message.key_version == "clipmill.demo-dag.v1"
    assert message.seed == b"seed-40"

    invalid = json.loads(
        (FIXTURES / "proto" / "demo_dag" / "invalid" / "wrong-version.json").read_text()
    )
    parsed = json_format.ParseDict(invalid, daemon_pb2.DemoDagPayloadV1())
    assert parsed.key_version != "clipmill.demo-dag.v1"


def test_probe_source_payload_fixtures_enforce_the_w5_key_version() -> None:
    valid = json.loads((FIXTURES / "proto" / "probe_source" / "valid" / "payload.json").read_text())
    message = json_format.ParseDict(valid, daemon_pb2.ProbeSourcePayloadV1())
    assert message.key_version == "clipmill.probe-source.v1"
    assert message.source_id.startswith("src_")

    invalid = json.loads(
        (FIXTURES / "proto" / "probe_source" / "invalid" / "wrong-version.json").read_text()
    )
    parsed = json_format.ParseDict(invalid, daemon_pb2.ProbeSourcePayloadV1())
    assert parsed.key_version != "clipmill.probe-source.v1"


@pytest.mark.parametrize("name", ["minimal.json", "with-mapping.json"])
def test_source_map_v1_legacy_and_mapping_fixtures_are_readable(name: str) -> None:
    raw = (FIXTURES / "source_map" / "valid" / name).read_text()
    SourceMap.model_validate_json(raw)


@pytest.mark.parametrize("name", ["float-ticks.json", "bad-mapping-timebase.json"])
def test_invalid_source_map_fixtures_are_rejected(name: str) -> None:
    raw = (FIXTURES / "source_map" / "invalid" / name).read_text()
    with pytest.raises(ValidationError):
        SourceMap.model_validate_json(raw)


def test_ingest_source_payload_fixtures_enforce_the_w11_key_version() -> None:
    valid = json.loads(
        (FIXTURES / "proto" / "ingest_source" / "valid" / "payload.json").read_text()
    )
    message = json_format.ParseDict(valid, daemon_pb2.IngestSourcePayloadV1())
    assert message.key_version == "clipmill.ingest-source.v1"
    assert message.source_id.startswith("src_")

    invalid = json.loads(
        (FIXTURES / "proto" / "ingest_source" / "invalid" / "wrong-version.json").read_text()
    )
    parsed = json_format.ParseDict(invalid, daemon_pb2.IngestSourcePayloadV1())
    assert parsed.key_version != "clipmill.ingest-source.v1"


EDIT_IR_INVALID = ["wrong-timebase.json", "float-ticks.json", "empty-caption-line.json"]


@pytest.mark.parametrize("name", ["clip.json", "minimal.json"])
def test_valid_edit_ir_fixtures_roundtrip_canonically(name: str) -> None:
    schemas = importlib.import_module("clipmill_worker_sdk.gen.schemas.edit_ir")
    raw = (FIXTURES / "edit_ir" / "valid" / name).read_text()
    parsed = schemas.EditIr.model_validate_json(raw)
    reserialized = parsed.model_dump(mode="json", exclude_none=True)
    assert canonical(reserialized) == raw, f"edit_ir/{name} round-trip must be byte-identical"


@pytest.mark.parametrize("name", EDIT_IR_INVALID)
def test_invalid_edit_ir_fixtures_are_rejected(name: str) -> None:
    schemas = importlib.import_module("clipmill_worker_sdk.gen.schemas.edit_ir")
    raw = (FIXTURES / "edit_ir" / "invalid" / name).read_text()
    with pytest.raises(ValidationError):
        schemas.EditIr.model_validate_json(raw)


MEDIA_FIXTURES = [
    ("media.proxy", "media_proxy", "MediaProxy", "float-ticks.json"),
    ("media.audio", "media_audio", "MediaAudio", "wrong-codec.json"),
    (
        "media.loudness_envelope",
        "media_loudness_envelope",
        "MediaLoudnessEnvelope",
        "float-ticks.json",
    ),
    (
        "media.reference_index",
        "media_reference_index",
        "MediaReferenceIndex",
        "missing-keyframes.json",
    ),
    ("media.filmstrip", "media_filmstrip", "MediaFilmstrip", "float-ticks.json"),
    ("media.audio_peaks", "media_audio_peaks", "MediaAudioPeaks", "out-of-range.json"),
    ("media.frames", "media_frames", "MediaFrames", "missing-coverage.json"),
    ("media.ingest_manifest", "media_ingest_manifest", "MediaIngestManifest", "unknown-kind.json"),
]


@pytest.mark.parametrize(("fixture_dir", "module", "model", "_invalid"), MEDIA_FIXTURES)
def test_valid_media_fixtures_roundtrip_canonically(
    fixture_dir: str, module: str, model: str, _invalid: str
) -> None:
    schemas = importlib.import_module(f"clipmill_worker_sdk.gen.schemas.{module}")
    model_type = getattr(schemas, model)
    raw = (FIXTURES / fixture_dir / "valid" / "minimal.json").read_text()
    parsed = model_type.model_validate_json(raw)
    reserialized = parsed.model_dump(mode="json", exclude_none=True)
    assert canonical(reserialized) == raw, f"{fixture_dir} round-trip must be byte-identical"


@pytest.mark.parametrize(("fixture_dir", "module", "model", "invalid"), MEDIA_FIXTURES)
def test_invalid_media_fixtures_are_rejected(
    fixture_dir: str, module: str, model: str, invalid: str
) -> None:
    schemas = importlib.import_module(f"clipmill_worker_sdk.gen.schemas.{module}")
    model_type = getattr(schemas, model)
    raw = (FIXTURES / fixture_dir / "invalid" / invalid).read_text()
    with pytest.raises(ValidationError):
        model_type.model_validate_json(raw)
