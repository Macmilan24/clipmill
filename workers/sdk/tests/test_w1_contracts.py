"""W1 contract fixtures, Python leg."""

import json
from pathlib import Path

import pytest
from clipmill.shm.v1 import shm_pb2
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk.gen.schemas.device_profile import DeviceProfile
from clipmill_worker_sdk.gen.schemas.source_map import SourceMap
from pydantic import ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures"


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


@pytest.mark.parametrize(
    ("model", "kind"),
    [(SourceMap, "source_map"), (DeviceProfile, "device_profile")],
)
def test_valid_fixture_roundtrips_canonically(model: type, kind: str) -> None:
    raw = (FIXTURES / kind / "valid" / "minimal.json").read_text()
    parsed = model.model_validate_json(raw)
    assert canonical(parsed.model_dump(mode="json", exclude_none=True)) == raw


def test_phase0_device_profile_fixture_roundtrips_canonically() -> None:
    raw = (FIXTURES / "device_profile" / "valid" / "phase0.json").read_text()
    parsed = DeviceProfile.model_validate_json(raw)
    assert canonical(parsed.model_dump(mode="json", exclude_none=True)) == raw


@pytest.mark.parametrize(
    ("model", "kind", "name"),
    [
        (SourceMap, "source_map", "float-ticks.json"),
        (DeviceProfile, "device_profile", "missing-measured.json"),
        (DeviceProfile, "device_profile", "bad-attestation.json"),
    ],
)
def test_invalid_fixture_rejected(model: type, kind: str, name: str) -> None:
    raw = (FIXTURES / kind / "invalid" / name).read_text()
    with pytest.raises(ValidationError):
        model.model_validate_json(raw)


def test_shm_descriptor_binpb_roundtrips() -> None:
    binpb = (FIXTURES / "proto" / "shm" / "buffer_descriptor.binpb").read_bytes()
    twin = json.loads((FIXTURES / "proto" / "shm" / "buffer_descriptor.json").read_text())
    descriptor = shm_pb2.BufferDescriptor.FromString(binpb)
    assert descriptor.shm_name == twin["shm_name"]
    assert descriptor.byte_len == twin["byte_len"]
    assert descriptor.dtype == shm_pb2.DATA_TYPE_U8
    assert descriptor.timebase.den == twin["timebase"]["den"]
    assert list(descriptor.shape) == twin["shape"]
    assert descriptor.SerializeToString(deterministic=True) == binpb


def test_w6_worker_and_shared_memory_extensions_are_available() -> None:
    descriptor = shm_pb2.BufferDescriptor(
        shape=[4],
        dtype=shm_pb2.DATA_TYPE_U8,
        byte_len=4,
        lease_id="lse_01J00000000000000000000000",
        transport_type=shm_pb2.TRANSPORT_TYPE_SCM_RIGHTS_MEMFD,
        handle_token="shm_01J00000000000000000000000",
    )
    lease = worker_pb2.TaskLease(
        task_id="tsk_01J00000000000000000000000",
        lease_id=descriptor.lease_id,
        kind="demo-seed",
        staging_id="stg_01J00000000000000000000000",
        staging_dir="/private/staging",
        shared_buffer=descriptor,
    )
    decoded = worker_pb2.TaskLease.FromString(lease.SerializeToString(deterministic=True))
    assert decoded.shared_buffer.handle_token == descriptor.handle_token
    assert decoded.staging_id.startswith("stg_")
