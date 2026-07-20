import json

import pytest
from clipmill_eval.profiles import (
    DEVICE_ATTESTATION_DOMAIN,
    DeviceProfileVerificationError,
    verify_device_profile,
)
from clipmill_eval.signing import canonical_json
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey


def signed_profile() -> str:
    key = Ed25519PrivateKey.generate()
    profile = {
        "schema_version": "clipmill.device_profile.v1",
        "cpu": {"logical_cores": 8, "model": "fixture"},
        "phase0": {
            "available_memory_bytes": 1024,
            "hardware_fingerprint": "sha256:" + "0" * 64,
            "measurement_generation": 3,
        },
    }
    unsigned = canonical_json(profile)
    signature = key.sign(DEVICE_ATTESTATION_DOMAIN + unsigned)
    profile["phase0"]["attestation"] = {
        "algorithm": "ed25519",
        "public_key": key.public_key().public_bytes_raw().hex(),
        "signature": signature.hex(),
    }
    return canonical_json(profile).decode("utf-8")


def test_device_profile_signature_is_verified_without_float_reserialization() -> None:
    profile = signed_profile()
    verified = verify_device_profile(profile)
    assert verified.measurement_generation == 3
    assert verified.available_memory_bytes == 1024


def test_device_profile_tampering_is_rejected() -> None:
    value = json.loads(signed_profile())
    value["phase0"]["measurement_generation"] = 4
    with pytest.raises(DeviceProfileVerificationError, match="signature"):
        verify_device_profile(canonical_json(value).decode("utf-8"))
