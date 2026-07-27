"""The evidence CI accepts about hardware CI does not have.

Everything here runs on any machine, MLX or not, because the point is the
document rather than the measurement: what may be published, what must be
refused, and what a tampered file does. A verifier that only ever saw valid
input is a verifier nobody has tested.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from clipmill_eval.mlx import (
    MlxAttestationError,
    build_mlx_attestation,
    verify_mlx_attestation,
    write_mlx_attestation,
)
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

FINGERPRINT = "sha256:" + "a1" * 32
ASR_DIGEST = "sha256:" + "b2" * 32
ALIGN_DIGEST = "sha256:" + "c3" * 32


def binding(capability: str, backend: str = "mlx", selected_by: str = "measured") -> dict:
    accelerated = backend == "mlx"
    stage = "speech-asr" if capability == "asr" else "speech-align"
    suffix = "asr" if capability == "asr" else "align"
    return {
        "backend": backend,
        "capability": capability,
        "implementation": (
            f"clipmill-worker-speech-mlx@0.1.0/{suffix}"
            if accelerated
            else f"clipmill-worker-{suffix}@0.1.0"
        ),
        "model": (
            f"qwen3-{'asr' if capability == 'asr' else 'aligner'}-mlx"
            if accelerated
            else "whisper-base"
        ),
        "selected_by": selected_by,
        "stage": stage,
    }


def candidate(capability: str, digest: str) -> dict:
    suffix = "asr" if capability == "asr" else "align"
    return {
        "backend": "mlx",
        "capability": capability,
        "implementation": f"clipmill-worker-speech-mlx@0.1.0/{suffix}",
        "model": f"qwen3-{'asr' if capability == 'asr' else 'aligner'}-mlx",
        "model_digest": digest,
        "peak_resident_bytes": 3_400_000_000,
        "real_time_factor": 22.5,
        "runnable": True,
    }


def profile(**overrides) -> dict:
    document = {
        "phase0": {"hardware_fingerprint": FINGERPRINT},
        "platform": {"arch": "arm64", "os": "macos", "os_version": "26.5.2"},
        "selection": {
            "bindings": [binding("asr"), binding("forced-align")],
            "candidates": [
                candidate("asr", ASR_DIGEST),
                candidate("forced-align", ALIGN_DIGEST),
            ],
        },
    }
    document.update(overrides)
    return document


TIMING = {"bar_ms": 120, "median_error_ms": 27, "words": 70}


def key() -> Ed25519PrivateKey:
    return Ed25519PrivateKey.from_private_bytes(bytes(range(32)))


def test_a_measured_accelerated_binding_is_publishable_evidence(tmp_path: Path):
    bundle = build_mlx_attestation(profile(), TIMING, key())
    write_mlx_attestation(tmp_path, bundle)

    verified = verify_mlx_attestation(tmp_path)
    assert verified.attestation["hardware_fingerprint"] == FINGERPRINT
    assert [entry["capability"] for entry in verified.attestation["bindings"]] == [
        "asr",
        "forced-align",
    ]
    assert verified.attestation["timing"] == TIMING


def test_the_published_document_names_no_machine(tmp_path: Path):
    """A fingerprint is a digest; an OS version is a machine.

    The evidence has to be committable to a public repository, so it carries
    what makes the measurement checkable and nothing that identifies where it
    was taken.
    """

    write_mlx_attestation(tmp_path, build_mlx_attestation(profile(), TIMING, key()))
    published = json.loads((tmp_path / "mlx-attestation.json").read_text(encoding="utf-8"))
    assert published["platform"] == {"arch": "arm64", "os": "macos"}
    assert "os_version" not in json.dumps(published)


def test_the_portable_path_winning_a_capability_is_still_valid_evidence():
    """The measurement decides, and sometimes it decides against MLX.

    On this project's own dev machine whisper.cpp-base recognizes faster than
    a 1.7B Qwen3 while the Qwen3 aligner beats the CTC one five times over.
    Both are real answers. A gate that refused the first would be asserting
    the static per-platform default D19 removes.
    """

    fell_back = profile()
    fell_back["selection"]["bindings"] = [
        binding("asr", backend="cpu", selected_by="measured"),
        binding("forced-align"),
    ]
    bundle = build_mlx_attestation(fell_back, TIMING, key())
    asr = next(entry for entry in bundle.attestation["bindings"] if entry["capability"] == "asr")
    assert asr["backend"] == "cpu"
    assert asr["selected_by"] == "measured"


def test_an_accelerated_implementation_that_never_ran_proves_nothing():
    """What the gate is actually for: the accelerated path *works* here.

    A run where MLX was absent, or crashed, or fell out of the candidate list
    is a run that says nothing about the code this attestation exists to cover.
    """

    absent = profile()
    absent["selection"]["candidates"] = [candidate("asr", ASR_DIGEST)]
    with pytest.raises(MlxAttestationError, match="was not measured runnable here"):
        build_mlx_attestation(absent, TIMING, key())


def test_a_binding_nobody_measured_is_not_evidence_of_a_measurement():
    unmeasured = profile()
    unmeasured["selection"]["bindings"] = [
        binding("asr", selected_by="unmeasured_fallback"),
        binding("forced-align"),
    ]
    with pytest.raises(MlxAttestationError, match="rather than measurement"):
        build_mlx_attestation(unmeasured, TIMING, key())


def test_timing_worse_than_the_bar_refuses_to_be_signed():
    """The accelerated path is held to the bar CI holds the portable one to.
    An attestation that recorded a miss would be evidence of a failure."""

    with pytest.raises(MlxAttestationError, match="exceeds the 120 ms bar"):
        missed = {"bar_ms": 120, "median_error_ms": 340, "words": 70}
        build_mlx_attestation(profile(), missed, key())


def test_a_profile_without_a_selection_block_has_nothing_to_attest():
    with pytest.raises(MlxAttestationError, match="no selection block"):
        build_mlx_attestation({"phase0": {"hardware_fingerprint": FINGERPRINT}}, TIMING, key())


def test_an_edited_measurement_fails_verification(tmp_path: Path):
    write_mlx_attestation(tmp_path, build_mlx_attestation(profile(), TIMING, key()))
    path = tmp_path / "mlx-attestation.json"
    tampered = path.read_text(encoding="utf-8").replace(
        '"real_time_factor":22.5', '"real_time_factor":99.5'
    )
    assert tampered != path.read_text(encoding="utf-8"), "the substitution must have happened"
    path.write_text(tampered, encoding="utf-8")

    with pytest.raises(MlxAttestationError, match="signature is invalid"):
        verify_mlx_attestation(tmp_path)


def test_a_document_signed_by_another_key_is_refused(tmp_path: Path):
    write_mlx_attestation(tmp_path, build_mlx_attestation(profile(), TIMING, key()))
    other = Ed25519PrivateKey.from_private_bytes(bytes(range(32, 64)))
    (tmp_path / "verification-key.hex").write_text(
        other.public_key().public_bytes_raw().hex() + "\n", encoding="ascii"
    )

    with pytest.raises(MlxAttestationError, match="unexpected key"):
        verify_mlx_attestation(tmp_path)


def test_an_incomplete_file_set_is_refused(tmp_path: Path):
    write_mlx_attestation(tmp_path, build_mlx_attestation(profile(), TIMING, key()))
    (tmp_path / "verification-key.hex").unlink()

    with pytest.raises(MlxAttestationError, match="file set is wrong"):
        verify_mlx_attestation(tmp_path)


def test_the_committed_evidence_verifies():
    """What protected `main` actually checks, checked here too."""

    committed = Path(__file__).resolve().parents[3] / "models/attestations/mlx-selection"
    if not committed.is_dir():
        pytest.skip("no MLX attestation is committed yet")
    verify_mlx_attestation(committed)
