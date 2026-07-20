"""Lease-scoped helpers for daemon-owned CAS staging directories."""

from __future__ import annotations

import hashlib
import os
import stat
from contextlib import suppress
from pathlib import Path, PurePosixPath

from clipmill.worker.v1 import worker_pb2


def validate_artifact_path(value: str) -> PurePosixPath:
    if (
        value == ""
        or "\\" in value
        or "\0" in value
        or value == "manifest.json"
        or value.startswith("/")
    ):
        raise ValueError("invalid artifact path")
    path = PurePosixPath(value)
    if any(part in {"", ".", ".."} for part in path.parts):
        raise ValueError("invalid artifact path")
    if str(path) != value:
        raise ValueError("artifact path is not normalized")
    return path


class StagingArea:
    """Writes only validated files beneath the daemon-issued staging token."""

    def __init__(self, staging_id: str, directory: str) -> None:
        root = Path(directory)
        if not staging_id.startswith("stg_") or root.name != staging_id:
            raise ValueError("staging directory does not match its token")
        if not root.is_absolute() or root.is_symlink() or not root.is_dir():
            raise ValueError("staging directory is not a private directory")
        if stat.S_IMODE(root.stat().st_mode) & 0o077:
            raise ValueError("staging directory permissions are not private")
        self.staging_id = staging_id
        self.root = root.resolve(strict=True)
        self._created: set[PurePosixPath] = set()

    def write_bytes(self, relative_path: str, payload: bytes) -> None:
        path = validate_artifact_path(relative_path)
        disk_path = self.root.joinpath(*path.parts)
        parent = disk_path.parent
        parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        self._assert_private_parent(parent)
        flags = os.O_CREAT | os.O_TRUNC | os.O_WRONLY
        if hasattr(os, "O_NOFOLLOW"):
            flags |= os.O_NOFOLLOW
        descriptor = os.open(disk_path, flags, 0o600)
        try:
            with os.fdopen(descriptor, "wb", closefd=False) as output:
                output.write(payload)
                output.flush()
                os.fsync(output.fileno())
        finally:
            os.close(descriptor)
        os.chmod(disk_path, 0o600, follow_symlinks=False)
        self._created.add(path)

    def declare(self, relative_path: str) -> worker_pb2.StagedOutput:
        path = validate_artifact_path(relative_path)
        if path not in self._created:
            raise ValueError("worker did not create the declared staging path")
        disk_path = self.root.joinpath(*path.parts)
        flags = os.O_RDONLY
        if hasattr(os, "O_NOFOLLOW"):
            flags |= os.O_NOFOLLOW
        descriptor = os.open(disk_path, flags)
        try:
            metadata = os.fstat(descriptor)
            if not stat.S_ISREG(metadata.st_mode):
                raise ValueError("staged output is not a regular file")
            digest = hashlib.sha256()
            while chunk := os.read(descriptor, 64 * 1024):
                digest.update(chunk)
        finally:
            os.close(descriptor)
        return worker_pb2.StagedOutput(
            relative_path=str(path),
            byte_size=metadata.st_size,
            sha256=digest.hexdigest(),
        )

    def abandon(self) -> None:
        """Remove only files created by this worker; the daemon revokes the token."""

        for path in sorted(self._created, key=lambda item: len(item.parts), reverse=True):
            disk_path = self.root.joinpath(*path.parts)
            with suppress(OSError):
                disk_path.unlink(missing_ok=True)
            parent = disk_path.parent
            while parent != self.root:
                try:
                    parent.rmdir()
                except OSError:
                    break
                parent = parent.parent
        self._created.clear()

    def _assert_private_parent(self, parent: Path) -> None:
        current = parent
        while current != self.root:
            if current.is_symlink() or not current.is_dir():
                raise ValueError("staging parent is not a regular directory")
            if stat.S_IMODE(current.stat().st_mode) & 0o077:
                raise ValueError("staging parent permissions are not private")
            current = current.parent
