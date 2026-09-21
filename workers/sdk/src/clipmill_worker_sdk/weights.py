"""Verify pinned model files immediately before loading them.

The model digest enters the artifact key. Rechecking against the manifest catches
corruption or substitution after download, before a parser consumes the files
or a result is attributed to the wrong weights."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

from clipmill.worker.v1 import worker_pb2

_CHUNK = 1024 * 1024


class ModelVerificationError(RuntimeError):
    """Weights are missing, unreadable, or no longer what was pinned."""


@dataclass(frozen=True, slots=True)
class VerifiedModel:
    name: str
    capability: str
    digest: str
    root: Path
    files: tuple[str, ...]

    def path(self, relative: str) -> Path:
        """Resolve one verified file. Anything undeclared is not reachable."""

        if relative not in self.files:
            raise ModelVerificationError(f"{relative} is not a pinned file of {self.name}")
        return self.root.joinpath(*PurePosixPath(relative).parts)


def verify_model(binding: worker_pb2.ModelBinding) -> VerifiedModel:
    """Hash every pinned file and refuse the model if any has moved."""

    if not binding.name or not binding.root:
        raise ModelVerificationError("model binding names no model")
    root = Path(binding.root)
    if not root.is_absolute() or not root.is_dir():
        raise ModelVerificationError(f"{binding.name} has no directory at {binding.root}")
    if not binding.files:
        raise ModelVerificationError(f"{binding.name} pins no files")

    names: list[str] = []
    for entry in binding.files:
        relative = PurePosixPath(entry.path)
        if entry.path.startswith("/") or any(part in {"", ".", ".."} for part in relative.parts):
            raise ModelVerificationError(f"pinned path {entry.path!r} escapes the model directory")
        path = root.joinpath(*relative.parts)
        if path.is_symlink() or not path.is_file():
            raise ModelVerificationError(f"{binding.name}/{entry.path} is not a regular file")
        size = path.stat().st_size
        if entry.bytes and size != entry.bytes:
            raise ModelVerificationError(
                f"{binding.name}/{entry.path} is {size} bytes, not the pinned {entry.bytes}"
            )
        digest = hashlib.sha256()
        with path.open("rb") as handle:
            while chunk := handle.read(_CHUNK):
                digest.update(chunk)
        if digest.hexdigest() != entry.sha256:
            raise ModelVerificationError(
                f"{binding.name}/{entry.path} does not match its pinned SHA-256"
            )
        names.append(entry.path)

    return VerifiedModel(
        name=binding.name,
        capability=binding.capability,
        digest=binding.digest,
        root=root,
        files=tuple(names),
    )


def require_model(lease: worker_pb2.TaskLease, capability: str) -> VerifiedModel:
    """The one model this lease provides for a capability, verified.

    Exactly one: a stage handed two candidates for the same capability has no
    basis for choosing, and a stage handed none cannot run. Both are refusals
    with a reason rather than a default that quietly changes what was produced.
    """

    candidates = [binding for binding in lease.models if binding.capability == capability]
    if not candidates:
        raise ModelVerificationError(f"this lease provides no {capability} model")
    if len(candidates) > 1:
        raise ModelVerificationError(
            f"this lease provides {len(candidates)} {capability} models and names no choice"
        )
    return verify_model(candidates[0])


__all__ = ["ModelVerificationError", "VerifiedModel", "require_model", "verify_model"]
