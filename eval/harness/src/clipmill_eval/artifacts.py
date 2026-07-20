"""Read-only verification of daemon-published CAS artifacts."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

ARTIFACT_PATTERN = re.compile(r"^sha256:([0-9a-f]{64})$")


class ArtifactVerificationError(ValueError):
    """A CAS object is missing, malformed, or has been modified."""


@dataclass(frozen=True, slots=True)
class VerifiedArtifact:
    artifact_id: str
    kind: str
    stage: str
    manifest: dict[str, Any]
    object_directory: Path


def verify_artifact(data_dir: Path, artifact_id: str) -> VerifiedArtifact:
    match = ARTIFACT_PATTERN.fullmatch(artifact_id)
    if match is None:
        raise ArtifactVerificationError("artifact ID is invalid")
    digest = match.group(1)
    object_directory = data_dir / "artifacts" / "objects" / "sha256" / digest[:2] / digest
    _require_real_directory(object_directory)
    manifest_path = object_directory / "manifest.json"
    if manifest_path.is_symlink() or not manifest_path.is_file():
        raise ArtifactVerificationError("artifact manifest is not a regular file")
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ArtifactVerificationError(f"artifact manifest is unreadable: {error}") from error
    if not isinstance(manifest, dict):
        raise ArtifactVerificationError("artifact manifest must be an object")
    if (
        manifest.get("schema_version") != "clipmill.artifact.manifest.v1"
        or manifest.get("artifact_id") != artifact_id
    ):
        raise ArtifactVerificationError("artifact manifest identity is inconsistent")
    files = manifest.get("files")
    if not isinstance(files, list) or not files:
        raise ArtifactVerificationError("artifact manifest omitted payload files")
    declared: set[str] = set()
    for record in files:
        if not isinstance(record, dict):
            raise ArtifactVerificationError("artifact file record must be an object")
        relative = record.get("path")
        expected_bytes = record.get("bytes")
        expected_sha256 = record.get("sha256")
        if (
            not isinstance(relative, str)
            or not _portable_path(relative)
            or relative == "manifest.json"
            or relative in declared
        ):
            raise ArtifactVerificationError("artifact file path is unsafe or duplicated")
        if (
            not isinstance(expected_bytes, int)
            or isinstance(expected_bytes, bool)
            or expected_bytes < 0
            or not isinstance(expected_sha256, str)
            or ARTIFACT_PATTERN.fullmatch(expected_sha256) is None
        ):
            raise ArtifactVerificationError("artifact file metadata is invalid")
        declared.add(relative)
        _verify_payload(
            object_directory,
            relative,
            expected_bytes,
            expected_sha256.removeprefix("sha256:"),
        )
    actual = {
        path.relative_to(object_directory).as_posix()
        for path in object_directory.rglob("*")
        if path.is_file() and path.name != "manifest.json"
    }
    if actual != declared:
        raise ArtifactVerificationError("artifact contains undeclared or missing payload files")
    producer = manifest.get("producer")
    kind = manifest.get("kind")
    if not isinstance(producer, dict) or not isinstance(kind, str):
        raise ArtifactVerificationError("artifact kind or producer is invalid")
    stage = producer.get("stage")
    if not isinstance(stage, str):
        raise ArtifactVerificationError("artifact producer stage is invalid")
    return VerifiedArtifact(artifact_id, kind, stage, manifest, object_directory)


def _verify_payload(root: Path, relative: str, expected_bytes: int, expected_sha256: str) -> None:
    path = root.joinpath(*PurePosixPath(relative).parts)
    current = root
    for component in PurePosixPath(relative).parts:
        current = current / component
        if current.is_symlink():
            raise ArtifactVerificationError("artifact payload traverses a symlink")
    try:
        stat = path.stat()
    except OSError as error:
        raise ArtifactVerificationError(f"artifact payload is unavailable: {error}") from error
    if not path.is_file() or stat.st_size != expected_bytes:
        raise ArtifactVerificationError("artifact payload type or size is invalid")
    hasher = hashlib.sha256()
    with path.open("rb") as payload:
        for chunk in iter(lambda: payload.read(1024 * 1024), b""):
            hasher.update(chunk)
    if hasher.hexdigest() != expected_sha256:
        raise ArtifactVerificationError("artifact payload SHA-256 does not match")


def _require_real_directory(path: Path) -> None:
    if path.is_symlink() or not path.is_dir():
        raise ArtifactVerificationError("artifact object directory is unavailable")


def _portable_path(value: str) -> bool:
    components = value.split("/")
    return (
        bool(value)
        and "\\" not in value
        and "\x00" not in value
        and not PurePosixPath(value).is_absolute()
        and all(component not in {"", ".", ".."} for component in components)
    )
