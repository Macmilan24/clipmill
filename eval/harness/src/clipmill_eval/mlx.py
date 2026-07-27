"""The signed, committable evidence that the accelerated path was measured here.

`gate-speech` runs in CI on both supported platforms, over the portable
implementations, because those are the ones every machine has. The accelerated
path cannot work that way: no hosted runner has an Apple GPU, and a gate that
quietly skipped itself would be a gate that always passes.

So this follows the Seed-40 pattern instead (R18). The drill runs on hardware
that actually has the accelerator, and what reaches the repository is a small
signed document: which implementations ran, what they were measured at, what
the device bound each capability to, and the digests of the weights behind
those numbers. CI verifies the signature and the shape; it never pretends to
have taken the measurement.

What it does *not* require is that the accelerator win. The plan assumed the
dev Mac would bind everything to MLX; the first real run of this gate found
whisper.cpp-base recognizing faster than a 1.7B Qwen3 on the same machine,
which is the measurement doing its job. Demanding a particular winner would
reinstate the static per-platform default D19 exists to remove. What is
required is that every accelerated implementation was measured runnable here,
and that each contested capability was bound by measurement rather than by
falling back — whoever won.

Everything here is path-free by construction. A hardware fingerprint is a
digest, not a machine name, and nothing else in the document identifies where
it was produced.
"""

from __future__ import annotations

import os
import re
import stat
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

from .signing import AttestationError, canonical_json, sign_document, verify_document

MLX_ATTESTATION_DOMAIN = b"clipmill.mlx-selection-attestation.v1\0"
MLX_ATTESTATION_SCHEMA = "clipmill.mlx_selection_attestation.v1"
DIGEST_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")
PUBLIC_FILENAMES = frozenset({"verification-key.hex", "mlx-attestation.json"})
#: The capabilities with more than one implementation behind them — the ones a
#: measurement can actually change the answer for.
REQUIRED_CAPABILITIES = ("asr", "forced-align")
#: The implementations this gate exists to prove can run here. Every one of
#: them must be measured runnable; none of them is required to *win*. Demanding
#: that the accelerator win would be the static per-platform default D19
#: replaces, wearing a gate's clothing — and it would be wrong on this
#: project's own dev machine, where a tiny CPU model beats a 1.7B one.
REQUIRED_IMPLEMENTATIONS = (
    "clipmill-worker-speech-mlx@0.1.0/asr",
    "clipmill-worker-speech-mlx@0.1.0/align",
)
ACCELERATED_BACKEND = "mlx"


class MlxAttestationError(ValueError):
    """The MLX evidence is incomplete, unsafe to publish, or invalid."""


@dataclass(frozen=True, slots=True)
class MlxAttestationBundle:
    verification_key: bytes
    attestation: dict[str, Any]


def build_mlx_attestation(
    profile: dict[str, Any],
    timing: dict[str, Any],
    signing_key: Ed25519PrivateKey,
) -> MlxAttestationBundle:
    """Reduce a measured device profile and a timing run to public evidence."""

    selection = profile.get("selection")
    if not isinstance(selection, dict):
        raise MlxAttestationError("the device profile carries no selection block")
    bindings = {
        binding["capability"]: binding
        for binding in selection.get("bindings", [])
        if isinstance(binding, dict) and "capability" in binding
    }
    for capability in REQUIRED_CAPABILITIES:
        binding = bindings.get(capability)
        if binding is None:
            raise MlxAttestationError(f"the profile binds nothing for {capability}")
        if binding.get("selected_by") != "measured":
            raise MlxAttestationError(
                f"{capability} was bound by {binding.get('selected_by')!r} rather than measurement"
            )

    candidates = [
        candidate
        for candidate in selection.get("candidates", [])
        if isinstance(candidate, dict) and candidate.get("runnable")
    ]
    if not candidates:
        raise MlxAttestationError("no candidate was measured runnable on this device")
    for candidate in candidates:
        digest = str(candidate.get("model_digest", ""))
        if not DIGEST_PATTERN.match(digest):
            raise MlxAttestationError(f"candidate {candidate.get('implementation')} has no digest")
    ran = {candidate["implementation"] for candidate in candidates}
    for implementation in REQUIRED_IMPLEMENTATIONS:
        if implementation not in ran:
            raise MlxAttestationError(
                f"{implementation} was not measured runnable here, so this run proves "
                "nothing about the accelerated path"
            )
    if not any(candidate.get("backend") == ACCELERATED_BACKEND for candidate in candidates):
        raise MlxAttestationError("no accelerated candidate ran on this device")

    unsigned = {
        "schema_version": MLX_ATTESTATION_SCHEMA,
        "hardware_fingerprint": _fingerprint(profile),
        "platform": _platform(profile),
        "bindings": sorted(
            (
                {
                    "backend": binding["backend"],
                    "capability": binding["capability"],
                    "implementation": binding["implementation"],
                    "model": binding["model"],
                    "selected_by": binding["selected_by"],
                    "stage": binding["stage"],
                }
                for binding in bindings.values()
            ),
            key=lambda binding: binding["capability"],
        ),
        "measurements": sorted(
            (
                {
                    "backend": candidate["backend"],
                    "capability": candidate["capability"],
                    "implementation": candidate["implementation"],
                    "model": candidate["model"],
                    "model_digest": candidate["model_digest"],
                    "peak_resident_bytes": int(candidate["peak_resident_bytes"]),
                    "real_time_factor": float(candidate["real_time_factor"]),
                }
                for candidate in candidates
            ),
            key=lambda candidate: candidate["implementation"],
        ),
        "timing": _timing(timing),
    }
    _assert_path_free(unsigned)
    signed = sign_document(unsigned, signing_key, MLX_ATTESTATION_DOMAIN)
    bundle = MlxAttestationBundle(signing_key.public_key().public_bytes_raw(), signed)
    _verify_bundle(bundle)
    return bundle


def write_mlx_attestation(output_directory: Path, bundle: MlxAttestationBundle) -> None:
    """Publish only the two files that are safe to commit."""

    output_directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    if output_directory.is_symlink() or not output_directory.is_dir():
        raise MlxAttestationError("attestation output must be a real directory")
    unexpected = {entry.name for entry in output_directory.iterdir()} - PUBLIC_FILENAMES
    if unexpected:
        raise MlxAttestationError(
            "attestation output directory contains unexpected files: "
            + ", ".join(sorted(unexpected))
        )
    _atomic_write(
        output_directory / "verification-key.hex",
        bundle.verification_key.hex().encode("ascii") + b"\n",
    )
    _atomic_write(
        output_directory / "mlx-attestation.json",
        canonical_json(bundle.attestation) + b"\n",
    )
    descriptor = os.open(output_directory, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def verify_mlx_attestation(output_directory: Path) -> MlxAttestationBundle:
    """Verify the committed evidence exactly as protected `main` must."""

    if output_directory.is_symlink() or not output_directory.is_dir():
        raise MlxAttestationError("attestation directory is missing or unsafe")
    present = {entry.name for entry in output_directory.iterdir()}
    if present != PUBLIC_FILENAMES:
        raise MlxAttestationError(
            f"attestation file set is wrong (found {sorted(present)}, "
            f"expected {sorted(PUBLIC_FILENAMES)})"
        )
    for name in sorted(PUBLIC_FILENAMES):
        path = output_directory / name
        if path.is_symlink() or not path.is_file():
            raise MlxAttestationError(f"{name} must be a regular, non-symlink file")

    key_path = output_directory / "verification-key.hex"
    try:
        verification_key = bytes.fromhex(key_path.read_text(encoding="ascii").strip())
    except (OSError, UnicodeError, ValueError) as error:
        raise MlxAttestationError("verification key is not hexadecimal") from error
    if len(verification_key) != 32:
        raise MlxAttestationError("verification key must contain exactly 32 bytes")

    attestation_path = output_directory / "mlx-attestation.json"
    import json

    try:
        attestation = json.loads(attestation_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise MlxAttestationError(f"attestation JSON is invalid: {error}") from error
    if canonical_json(attestation) + b"\n" != attestation_path.read_bytes():
        raise MlxAttestationError("attestation file is not canonical JSON")

    bundle = MlxAttestationBundle(verification_key, attestation)
    _verify_bundle(bundle)
    return bundle


def _verify_bundle(bundle: MlxAttestationBundle) -> None:
    attestation = bundle.attestation
    if attestation.get("schema_version") != MLX_ATTESTATION_SCHEMA:
        raise MlxAttestationError("attestation declares an unexpected schema version")
    try:
        verify_document(attestation, MLX_ATTESTATION_DOMAIN, bundle.verification_key)
    except AttestationError as error:
        raise MlxAttestationError(str(error)) from error
    _assert_path_free({key: value for key, value in attestation.items() if key != "signature"})
    bound = {binding["capability"]: binding for binding in attestation.get("bindings", [])}
    for capability in REQUIRED_CAPABILITIES:
        binding = bound.get(capability)
        if binding is None:
            raise MlxAttestationError(f"the attestation binds nothing for {capability}")
        if binding.get("selected_by") != "measured":
            raise MlxAttestationError(f"{capability} was not bound by measurement")
    measured = {entry["implementation"] for entry in attestation.get("measurements", [])}
    if not measured:
        raise MlxAttestationError("the attestation carries no measurements")
    for implementation in REQUIRED_IMPLEMENTATIONS:
        if implementation not in measured:
            raise MlxAttestationError(f"the attestation does not cover {implementation}")


def _fingerprint(profile: dict[str, Any]) -> str:
    fingerprint = str(profile.get("phase0", {}).get("hardware_fingerprint", ""))
    if not DIGEST_PATTERN.match(fingerprint):
        raise MlxAttestationError("the profile carries no hardware fingerprint")
    return fingerprint


def _platform(profile: dict[str, Any]) -> dict[str, str]:
    """OS and architecture only. The OS *version* is deliberately dropped: it
    identifies a machine more precisely than this evidence needs to."""

    platform = profile.get("platform", {})
    return {
        "arch": str(platform.get("arch", "")),
        "os": str(platform.get("os", "")),
    }


def _timing(timing: dict[str, Any]) -> dict[str, Any]:
    """The accelerated path's word timing, held to the same bar as CI's."""

    try:
        words = int(timing["words"])
        median_error_ms = int(timing["median_error_ms"])
        bar_ms = int(timing["bar_ms"])
    except (KeyError, TypeError, ValueError) as error:
        raise MlxAttestationError(f"timing evidence is incomplete: {error}") from error
    if words <= 0:
        raise MlxAttestationError("timing evidence covers no words")
    if median_error_ms > bar_ms:
        raise MlxAttestationError(
            f"median word-boundary error {median_error_ms} ms exceeds the {bar_ms} ms bar"
        )
    return {"bar_ms": bar_ms, "median_error_ms": median_error_ms, "words": words}


def _assert_path_free(value: Any) -> None:
    """No filesystem path may reach the repository through this document."""

    if isinstance(value, dict):
        for key, entry in value.items():
            _assert_path_free(key)
            _assert_path_free(entry)
        return
    if isinstance(value, (list, tuple)):
        for entry in value:
            _assert_path_free(entry)
        return
    # Implementation names carry a slash on purpose ("…/asr"), and a digest
    # carries a colon; neither is a path, and neither starts with one or
    # contains a traversal. What is refused is anything that could be read as a
    # location on the machine this ran on.
    if isinstance(value, str) and (value.startswith(("/", "\\", "~", ".")) or ".." in value):
        raise MlxAttestationError(f"attestation contains a filesystem path: {value!r}")


def _atomic_write(path: Path, payload: bytes) -> None:
    directory = path.parent
    descriptor, temporary = tempfile.mkstemp(dir=directory, prefix=".partial-")
    try:
        os.write(descriptor, payload)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    os.chmod(temporary, stat.S_IRUSR | stat.S_IWUSR | stat.S_IRGRP | stat.S_IROTH)
    os.replace(temporary, path)


__all__ = [
    "MLX_ATTESTATION_DOMAIN",
    "MLX_ATTESTATION_SCHEMA",
    "MlxAttestationBundle",
    "MlxAttestationError",
    "build_mlx_attestation",
    "verify_mlx_attestation",
    "write_mlx_attestation",
]
