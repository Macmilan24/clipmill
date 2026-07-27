"""Loading pinned model weights, after checking they are still what was pinned.

A model is a versioned input to an artifact, not an ambient capability (book
ch. 11): its digest joins the artifact key, so a transcript produced by other
weights is a different transcript and must not share an address with this one.
That guarantee is only worth what the check behind it is worth, and the daemon
having verified the download at some earlier point is not a statement about
the bytes on disk at load time.

So the worker verifies again, immediately before loading. The cost is a hash
of files that are about to be read from that same disk anyway; the failure it
prevents is a corrupt or swapped weight file being read by an ONNX parser and
its output published under an address asserting the pinned model produced it.
"""

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
