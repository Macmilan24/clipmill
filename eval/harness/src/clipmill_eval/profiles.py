"""Independent verification of daemon-produced device profile attestations."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from typing import Any

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

DEVICE_ATTESTATION_DOMAIN = b"clipmill.device.attestation.v1\0"
FINGERPRINT_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")


class DeviceProfileVerificationError(ValueError):
    """The profile schema, canonical envelope, or Ed25519 signature is invalid."""


@dataclass(frozen=True, slots=True)
class VerifiedDeviceProfile:
    hardware_fingerprint: str
    measurement_generation: int
    available_memory_bytes: int
    public_key: bytes


def verify_device_profile(profile_json: str) -> VerifiedDeviceProfile:
    try:
        profile: Any = json.loads(profile_json)
    except json.JSONDecodeError as error:
        raise DeviceProfileVerificationError("profile JSON is malformed") from error
    if not isinstance(profile, dict) or profile.get("schema_version") != (
        "clipmill.device_profile.v1"
    ):
        raise DeviceProfileVerificationError("profile schema version is invalid")
    phase0 = profile.get("phase0")
    if not isinstance(phase0, dict):
        raise DeviceProfileVerificationError("profile omitted the Phase 0 extension")
    fingerprint = phase0.get("hardware_fingerprint")
    generation = phase0.get("measurement_generation")
    available_memory = phase0.get("available_memory_bytes")
    envelope = phase0.get("attestation")
    if (
        not isinstance(fingerprint, str)
        or FINGERPRINT_PATTERN.fullmatch(fingerprint) is None
        or not isinstance(generation, int)
        or isinstance(generation, bool)
        or generation < 1
        or not isinstance(available_memory, int)
        or isinstance(available_memory, bool)
        or available_memory < 0
        or not isinstance(envelope, dict)
        or envelope.get("algorithm") != "ed25519"
    ):
        raise DeviceProfileVerificationError("profile Phase 0 fields are invalid")
    try:
        public_key = bytes.fromhex(str(envelope["public_key"]))
        signature = bytes.fromhex(str(envelope["signature"]))
    except (KeyError, ValueError) as error:
        raise DeviceProfileVerificationError("profile signature envelope is invalid") from error
    if len(public_key) != 32 or len(signature) != 64:
        raise DeviceProfileVerificationError("profile key or signature length is invalid")
    fragment = (
        '"attestation":{"algorithm":"ed25519","public_key":"'
        + public_key.hex()
        + '","signature":"'
        + signature.hex()
        + '"},'
    )
    if profile_json.count(fragment) != 1:
        raise DeviceProfileVerificationError("profile is not in the daemon's canonical signed form")
    unsigned = profile_json.replace(fragment, "", 1).encode("utf-8")
    try:
        Ed25519PublicKey.from_public_bytes(public_key).verify(
            signature,
            DEVICE_ATTESTATION_DOMAIN + unsigned,
        )
    except (InvalidSignature, ValueError) as error:
        raise DeviceProfileVerificationError("profile signature is invalid") from error
    return VerifiedDeviceProfile(
        fingerprint,
        generation,
        available_memory,
        public_key,
    )
