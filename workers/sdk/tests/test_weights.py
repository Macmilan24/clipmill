"""A worker verifies its weights immediately before loading them.

The daemon verified the same files when it fetched them, which is exactly why
this is worth testing: the argument for skipping the second check is always
that the first one happened, and the gap between them is where a truncated
download, a half-restored backup, or an edited file lives. What the artifact
key asserts — that these weights produced this transcript — is only as good as
the check nearest the load.
"""

from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk.weights import (
    ModelVerificationError,
    require_model,
    verify_model,
)


def pinned(
    root: Path,
    name: str = "silero-vad",
    capability: str = "vad",
) -> worker_pb2.ModelBinding:
    weights = root / name / "onnx"
    weights.mkdir(parents=True, exist_ok=True)
    payload = b"not really a neural network"
    (weights / "model.onnx").write_bytes(payload)
    return worker_pb2.ModelBinding(
        name=name,
        root=str(root / name),
        digest="sha256:" + "a" * 64,
        capability=capability,
        files=[
            worker_pb2.ModelFile(
                path="onnx/model.onnx",
                sha256=hashlib.sha256(payload).hexdigest(),
                bytes=len(payload),
            )
        ],
    )


def test_pinned_weights_verify_and_resolve(tmp_path: Path) -> None:
    model = verify_model(pinned(tmp_path))
    assert model.name == "silero-vad"
    assert model.capability == "vad"
    assert model.path("onnx/model.onnx").is_file()


def test_weights_that_changed_since_they_were_pinned_are_refused(tmp_path: Path) -> None:
    """The same size, different bytes — so only the digest can catch it."""

    binding = pinned(tmp_path)
    replacement = b"NOT really a neural network"
    assert len(replacement) == binding.files[0].bytes
    (tmp_path / "silero-vad" / "onnx" / "model.onnx").write_bytes(replacement)
    with pytest.raises(ModelVerificationError, match="SHA-256"):
        verify_model(binding)


def test_a_truncated_download_is_refused_before_it_is_parsed(tmp_path: Path) -> None:
    binding = pinned(tmp_path)
    (tmp_path / "silero-vad" / "onnx" / "model.onnx").write_bytes(b"not really")
    with pytest.raises(ModelVerificationError, match="bytes"):
        verify_model(binding)


def test_missing_weights_are_refused_rather_than_loaded_empty(tmp_path: Path) -> None:
    binding = pinned(tmp_path)
    (tmp_path / "silero-vad" / "onnx" / "model.onnx").unlink()
    with pytest.raises(ModelVerificationError, match="regular file"):
        verify_model(binding)


def test_a_pinned_path_may_not_escape_the_model_directory(tmp_path: Path) -> None:
    binding = pinned(tmp_path)
    binding.files[0].path = "../../etc/passwd"
    with pytest.raises(ModelVerificationError, match="escapes"):
        verify_model(binding)


def test_only_pinned_files_are_reachable_by_name(tmp_path: Path) -> None:
    """A file nobody hashed is not part of the model.

    Model directories accumulate: a README, a cached tokenizer, a leftover
    from an older revision. Reaching one by name would let it into a load
    without ever appearing in the digest the artifact key rests on.
    """

    model = verify_model(pinned(tmp_path))
    (tmp_path / "silero-vad" / "extra.bin").write_bytes(b"unpinned")
    with pytest.raises(ModelVerificationError, match="not a pinned file"):
        model.path("extra.bin")


def test_a_lease_naming_no_model_for_the_capability_is_refused(tmp_path: Path) -> None:
    lease = worker_pb2.TaskLease(models=[pinned(tmp_path, capability="vad")])
    with pytest.raises(ModelVerificationError, match="no asr model"):
        require_model(lease, "asr")


def test_a_lease_naming_two_models_for_one_capability_is_refused(tmp_path: Path) -> None:
    """Two candidates and no stated choice is not a default; it is a bug.

    Picking one silently would make the artifact key say a model produced the
    output while a coin flip decided which.
    """

    lease = worker_pb2.TaskLease(
        models=[
            pinned(tmp_path, name="whisper-base", capability="asr"),
            pinned(tmp_path, name="qwen3-asr-mlx", capability="asr"),
        ]
    )
    with pytest.raises(ModelVerificationError, match="names no choice"):
        require_model(lease, "asr")


def test_the_right_model_is_selected_from_a_lease_carrying_several(tmp_path: Path) -> None:
    lease = worker_pb2.TaskLease(
        models=[
            pinned(tmp_path, name="silero-vad", capability="vad"),
            pinned(tmp_path, name="whisper-base", capability="asr"),
        ]
    )
    assert require_model(lease, "asr").name == "whisper-base"
    assert require_model(lease, "vad").name == "silero-vad"
