"""Local Phase 0 worker identities and challenge-bound registration signing."""

from __future__ import annotations

import json
import re
import stat
from dataclasses import dataclass
from pathlib import Path

from clipmill.worker.v1 import worker_pb2
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

_WORKER_ID = re.compile(r"^wrk_[0-9A-HJKMNP-TV-Z]{26}$")
_WORD = re.compile(r"^[a-z0-9][a-z0-9_-]{0,63}$")
_DOMAIN = b"clipmill.worker.registration.v1\0"
SUPPORTED_PROTOCOLS = ("1.2", "1.1")
# The runtime a worker executes on. Names are runtimes rather than chips,
# because that is what a worker knows about itself; the daemon maps them to
# accelerator classes. "metal" is the 1.1 spelling, kept so those workers
# still register.
BACKENDS = frozenset({"cpu", "onnx-cpu", "mlx", "coreml", "metal", "cuda"})
# Declared resources joined the signature in 1.2, so they cannot be widened
# after signing.
_RESOURCES_PROTOCOL = "1.2"


@dataclass(frozen=True, slots=True)
class WorkerIdentity:
    """A private development identity loaded from a mode-0600 JSON file."""

    worker_id: str
    private_key: Ed25519PrivateKey

    @classmethod
    def load(cls, path: Path) -> WorkerIdentity:
        metadata = path.lstat()
        if (
            path.is_symlink()
            or not stat.S_ISREG(metadata.st_mode)
            or stat.S_IMODE(metadata.st_mode) & 0o077
            or metadata.st_size > 1024
        ):
            raise ValueError("worker identity is not a private regular file")
        raw = json.loads(path.read_text(encoding="utf-8"))
        if raw.get("key_version") != "clipmill.worker.identity.v1":
            raise ValueError("unsupported worker identity key version")
        worker_id = raw.get("worker_id")
        private_hex = raw.get("private_key")
        if not isinstance(worker_id, str) or _WORKER_ID.fullmatch(worker_id) is None:
            raise ValueError("invalid worker ID")
        if not isinstance(private_hex, str) or len(private_hex) != 64:
            raise ValueError("invalid Ed25519 private key")
        try:
            private_bytes = bytes.fromhex(private_hex)
        except ValueError as error:
            raise ValueError("invalid Ed25519 private key") from error
        return cls(worker_id, Ed25519PrivateKey.from_private_bytes(private_bytes))

    @property
    def public_key_bytes(self) -> bytes:
        return self.private_key.public_key().public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw,
        )

    def signed_descriptor(
        self,
        challenge: worker_pb2.RegistrationChallenge,
        *,
        family: str,
        capabilities: tuple[str, ...],
        protocol_version: str,
        backend: str,
        max_memory_bytes: int,
        cpu_threads: int = 1,
        vram_bytes: int = 0,
    ) -> worker_pb2.CapabilityDescriptor:
        if family == "" or _WORD.fullmatch(family) is None:
            raise ValueError("invalid worker family")
        ordered = tuple(sorted(set(capabilities)))
        if not ordered or ordered != capabilities:
            raise ValueError("capabilities must be nonempty, sorted, and unique")
        if any(_WORD.fullmatch(value) is None for value in ordered):
            raise ValueError("invalid worker capability")
        if protocol_version not in SUPPORTED_PROTOCOLS:
            raise ValueError("unsupported worker protocol")
        if (
            len(challenge.nonce) != 32
            or protocol_version not in challenge.supported_protocol_versions
        ):
            raise ValueError("worker registration challenge is invalid")
        if backend not in BACKENDS or max_memory_bytes <= 0:
            raise ValueError("invalid worker resources")
        if protocol_version == _RESOURCES_PROTOCOL:
            if cpu_threads < 1 or cpu_threads > 1024 or vram_bytes < 0:
                raise ValueError("invalid declared worker resources")
        elif cpu_threads != 1 or vram_bytes != 0:
            # A 1.1 signature does not cover these fields, so a 1.1 worker may
            # not declare them; the daemon refuses descriptors that try.
            raise ValueError("declared resources require protocol 1.2")
        descriptor = worker_pb2.CapabilityDescriptor(
            worker_id=self.worker_id,
            family=family,
            capabilities=ordered,
            protocol_version=protocol_version,
            backend=backend,
            max_memory_bytes=max_memory_bytes,
            public_key=self.public_key_bytes,
            cpu_threads=cpu_threads if protocol_version == _RESOURCES_PROTOCOL else 0,
            vram_bytes=vram_bytes if protocol_version == _RESOURCES_PROTOCOL else 0,
        )
        descriptor.signature = self.private_key.sign(registration_preimage(challenge, descriptor))
        return descriptor


def registration_preimage(
    challenge: worker_pb2.RegistrationChallenge,
    descriptor: worker_pb2.CapabilityDescriptor,
) -> bytes:
    """Return the byte-identical registration preimage used by the daemon."""

    parts = [
        _DOMAIN,
        challenge.nonce,
        b"\0",
        descriptor.worker_id.encode(),
        b"\0",
        descriptor.family.encode(),
        b"\0",
        "\x1f".join(descriptor.capabilities).encode(),
        b"\0",
        descriptor.protocol_version.encode(),
        b"\0",
        descriptor.backend.encode(),
        b"\0",
        descriptor.max_memory_bytes.to_bytes(8, "big"),
    ]
    if descriptor.protocol_version == _RESOURCES_PROTOCOL:
        parts.append(descriptor.cpu_threads.to_bytes(4, "big"))
        parts.append(descriptor.vram_bytes.to_bytes(8, "big"))
    parts.append(descriptor.public_key)
    return b"".join(parts)
