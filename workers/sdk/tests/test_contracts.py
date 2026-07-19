"""The Phase 0 contracts exit gate, Python leg."""

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
