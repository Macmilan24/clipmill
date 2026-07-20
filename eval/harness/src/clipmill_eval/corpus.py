"""Signed corpus and license-attestation verification."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

from .signing import AttestationError, verify_document

CORPUS_DOMAIN = b"clipmill.corpus-manifest.v1\0"
LICENSE_DOMAIN = b"clipmill.license-attestation.v1\0"
SHA256_PATTERN = re.compile(r"^[0-9a-f]{64}$")


class CorpusError(ValueError):
    """Corpus metadata or local media failed validation."""


@dataclass(frozen=True, slots=True)
class CorpusItem:
    item_id: str
    relative_path: str
    sha256: str
    byte_size: int
    expected_result: str
    expected_failure: str | None
    license_id: str


@dataclass(frozen=True, slots=True)
class VerifiedCorpus:
    corpus_id: str
    root: Path
    items: tuple[CorpusItem, ...]
    signing_public_key: bytes

    def path_for(self, item: CorpusItem) -> Path:
        return self.root.joinpath(*PurePosixPath(item.relative_path).parts)


def load_json(path: Path) -> dict[str, Any]:
    def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise CorpusError(f"duplicate JSON key: {key}")
            result[key] = value
        return result

    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise CorpusError(f"cannot read {path}: {error}") from error
    if not isinstance(value, dict):
        raise CorpusError(f"{path} must contain a JSON object")
    return value


def verify_corpus(
    corpus_root: Path,
    manifest_path: Path,
    license_path: Path,
    expected_public_key: bytes | None = None,
) -> VerifiedCorpus:
    """Verify signed metadata, license coverage, and every local media byte."""

    manifest = load_json(manifest_path)
    license_attestation = load_json(license_path)
    if manifest.get("schema_version") != "clipmill.corpus_manifest.v1":
        raise CorpusError("unsupported corpus manifest schema")
    if license_attestation.get("schema_version") != "clipmill.license_attestation.v1":
        raise CorpusError("unsupported license attestation schema")
    try:
        public_key = verify_document(manifest, CORPUS_DOMAIN, expected_public_key)
        verify_document(license_attestation, LICENSE_DOMAIN, public_key)
    except AttestationError as error:
        raise CorpusError(str(error)) from error
    corpus_id = manifest.get("corpus_id")
    if not isinstance(corpus_id, str) or not corpus_id or len(corpus_id) > 128:
        raise CorpusError("corpus_id is invalid")
    if license_attestation.get("corpus_id") != corpus_id:
        raise CorpusError("license attestation names a different corpus")
    raw_items = manifest.get("items")
    raw_licenses = license_attestation.get("licenses")
    if not isinstance(raw_items, list) or not raw_items:
        raise CorpusError("corpus manifest must contain at least one item")
    if not isinstance(raw_licenses, list):
        raise CorpusError("license attestation must contain a licenses list")
    licenses = _validated_licenses(raw_licenses)
    items = tuple(_parse_item(value) for value in raw_items)
    if len({item.item_id for item in items}) != len(items):
        raise CorpusError("corpus item IDs must be unique")
    if {item.item_id for item in items} != set(licenses):
        raise CorpusError("license attestation must cover exactly every corpus item")

    root = corpus_root.resolve(strict=True)
    if not root.is_dir() or root.is_symlink():
        raise CorpusError("corpus root must be a real directory")
    for item in items:
        if licenses[item.item_id] != item.license_id:
            raise CorpusError(f"license mismatch for {item.item_id}")
        _verify_media(root, item)
    return VerifiedCorpus(corpus_id, root, items, public_key)


def _parse_item(value: Any) -> CorpusItem:
    if not isinstance(value, dict):
        raise CorpusError("corpus items must be objects")
    item_id = value.get("item_id")
    relative_path = value.get("relative_path")
    digest = value.get("sha256")
    byte_size = value.get("bytes")
    expected_result = value.get("expected_result")
    expected_failure = value.get("expected_failure")
    license_id = value.get("license_id")
    if not isinstance(item_id, str) or not item_id or len(item_id) > 128:
        raise CorpusError("corpus item_id is invalid")
    if not isinstance(relative_path, str) or not _portable_relative_path(relative_path):
        raise CorpusError(f"item {item_id} has an unsafe relative path")
    if not isinstance(digest, str) or SHA256_PATTERN.fullmatch(digest) is None:
        raise CorpusError(f"item {item_id} has an invalid SHA-256")
    if not isinstance(byte_size, int) or isinstance(byte_size, bool) or byte_size < 0:
        raise CorpusError(f"item {item_id} has an invalid byte size")
    if expected_result not in {"success", "structured_failure"}:
        raise CorpusError(f"item {item_id} has an invalid expected result")
    if expected_result == "structured_failure":
        if not isinstance(expected_failure, str) or not expected_failure:
            raise CorpusError(f"hostile item {item_id} omitted expected_failure")
    elif expected_failure is not None:
        raise CorpusError(f"valid item {item_id} declared an expected failure")
    if not isinstance(license_id, str) or not license_id or len(license_id) > 128:
        raise CorpusError(f"item {item_id} has an invalid license ID")
    return CorpusItem(
        item_id,
        relative_path,
        digest,
        byte_size,
        expected_result,
        expected_failure,
        license_id,
    )


def _validated_licenses(values: list[Any]) -> dict[str, str]:
    result: dict[str, str] = {}
    for value in values:
        if not isinstance(value, dict):
            raise CorpusError("license records must be objects")
        item_id = value.get("item_id")
        license_id = value.get("license_id")
        redistributable = value.get("redistributable")
        if (
            not isinstance(item_id, str)
            or not item_id
            or not isinstance(license_id, str)
            or not license_id
            or redistributable is not True
            or item_id in result
        ):
            raise CorpusError("license record is invalid or duplicated")
        result[item_id] = license_id
    return result


def _portable_relative_path(value: str) -> bool:
    if not value or "\\" in value or "\x00" in value:
        return False
    path = PurePosixPath(value)
    return not path.is_absolute() and all(part not in {"", ".", ".."} for part in value.split("/"))


def _verify_media(root: Path, item: CorpusItem) -> None:
    path = root.joinpath(*PurePosixPath(item.relative_path).parts)
    try:
        relative = path.relative_to(root)
    except ValueError as error:
        raise CorpusError(f"item {item.item_id} escaped the corpus root") from error
    current = root
    for component in relative.parts:
        current = current / component
        if current.is_symlink():
            raise CorpusError(f"item {item.item_id} traverses a symlink")
    try:
        stat = path.stat()
    except OSError as error:
        raise CorpusError(f"item {item.item_id} is unavailable: {error}") from error
    if not path.is_file() or stat.st_size != item.byte_size:
        raise CorpusError(f"item {item.item_id} has the wrong file type or byte size")
    hasher = hashlib.sha256()
    try:
        with path.open("rb") as media:
            for chunk in iter(lambda: media.read(1024 * 1024), b""):
                hasher.update(chunk)
    except OSError as error:
        raise CorpusError(f"item {item.item_id} cannot be hashed: {error}") from error
    if hasher.hexdigest() != item.sha256:
        raise CorpusError(f"item {item.item_id} failed SHA-256 verification")
