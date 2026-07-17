"""The Phase 0 contracts exit gate, Python leg."""

import json
from pathlib import Path

import pytest
from clipmill.ipc.v1 import ping_pb2
from clipmill_worker_sdk.gen.schemas.artifact_manifest import ArtifactManifest
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
