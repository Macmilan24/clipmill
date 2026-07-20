"""Build and verify the public, path-free Phase 0 Seed-40 attestation."""

from __future__ import annotations

import os
import re
import stat
import tempfile
import unicodedata
from collections import Counter
from contextlib import suppress
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

from .corpus import SHA256_PATTERN, VerifiedCorpus, load_json
from .signing import AttestationError, canonical_json, sign_document, verify_document

PHASE0_ATTESTATION_DOMAIN = b"clipmill.phase0-run-attestation.v1\0"
PHASE0_ATTESTATION_SCHEMA = "clipmill.phase0_run_attestation.v1"
CORPUS_METADATA_SCHEMA = "clipmill.seed40.corpus_metadata.v1"
LICENSE_SUMMARY_SCHEMA = "clipmill.seed40.license_summary.v1"
PUBLIC_KEY_PATTERN = re.compile(r"^[0-9a-f]{64}$")
ARTIFACT_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")
SOURCE_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")
PUBLIC_FILENAMES = frozenset(
    {
        "verification-key.hex",
        "corpus-metadata.json",
        "license-summary.json",
        "run-attestation.json",
    }
)


class Phase0AttestationError(ValueError):
    """The Seed-40 evidence is incomplete, unsafe to publish, or invalid."""


@dataclass(frozen=True, slots=True)
class Phase0AttestationBundle:
    verification_key: bytes
    corpus_metadata: dict[str, Any]
    license_summary: dict[str, Any]
    run_attestation: dict[str, Any]


def load_private_signing_key(path: Path) -> Ed25519PrivateKey:
    """Load a raw hexadecimal Ed25519 seed from a private regular file."""

    try:
        file_stat = path.lstat()
    except OSError as error:
        raise Phase0AttestationError(f"cannot inspect signing key: {error}") from error
    if not stat.S_ISREG(file_stat.st_mode) or path.is_symlink():
        raise Phase0AttestationError("signing key must be a regular, non-symlink file")
    if stat.S_IMODE(file_stat.st_mode) & 0o077:
        raise Phase0AttestationError("signing key permissions must be 0600 or stricter")
    try:
        private_bytes = bytes.fromhex(path.read_text(encoding="ascii").strip())
    except (OSError, UnicodeError, ValueError) as error:
        raise Phase0AttestationError("signing key must contain hexadecimal bytes") from error
    if len(private_bytes) != 32:
        raise Phase0AttestationError("signing key must contain exactly 32 bytes")
    return Ed25519PrivateKey.from_private_bytes(private_bytes)


def build_phase0_attestation(
    corpus: VerifiedCorpus,
    run_manifest: dict[str, Any],
    signing_key: Ed25519PrivateKey,
) -> Phase0AttestationBundle:
    """Validate the full baseline and reduce it to safe, signed public evidence."""

    if len(corpus.items) != 40:
        raise Phase0AttestationError("the Phase 0 baseline must contain exactly 40 items")
    _validate_run(corpus, run_manifest)
    corpus_metadata = _corpus_metadata(corpus)
    license_summary = _license_summary(corpus)
    unsigned = {
        "schema_version": PHASE0_ATTESTATION_SCHEMA,
        "phase": "phase0",
        "corpus_metadata": corpus_metadata,
        "license_summary": license_summary,
        "run": run_manifest,
    }
    _assert_path_free(unsigned)
    signed = sign_document(unsigned, signing_key, PHASE0_ATTESTATION_DOMAIN)
    verification_key = signing_key.public_key().public_bytes_raw()
    bundle = Phase0AttestationBundle(
        verification_key,
        corpus_metadata,
        license_summary,
        signed,
    )
    _verify_bundle(bundle)
    return bundle


def write_phase0_attestation(output_directory: Path, bundle: Phase0AttestationBundle) -> None:
    """Atomically publish only the four artifacts safe to commit to Git."""

    output_directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    if output_directory.is_symlink() or not output_directory.is_dir():
        raise Phase0AttestationError("attestation output must be a real directory")
    unexpected = {entry.name for entry in output_directory.iterdir()} - PUBLIC_FILENAMES
    if unexpected:
        raise Phase0AttestationError(
            "attestation output directory contains unexpected files: "
            + ", ".join(sorted(unexpected))
        )
    _atomic_write(
        output_directory / "verification-key.hex",
        bundle.verification_key.hex().encode("ascii") + b"\n",
    )
    _atomic_write(
        output_directory / "corpus-metadata.json",
        canonical_json(bundle.corpus_metadata) + b"\n",
    )
    _atomic_write(
        output_directory / "license-summary.json",
        canonical_json(bundle.license_summary) + b"\n",
    )
    _atomic_write(
        output_directory / "run-attestation.json",
        canonical_json(bundle.run_attestation) + b"\n",
    )
    directory_fd = os.open(output_directory, os.O_RDONLY)
    try:
        os.fsync(directory_fd)
    finally:
        os.close(directory_fd)


def verify_phase0_attestation(output_directory: Path) -> Phase0AttestationBundle:
    """Verify the exact public Seed-40 evidence set intended for protected main."""

    if output_directory.is_symlink() or not output_directory.is_dir():
        raise Phase0AttestationError("attestation directory is missing or unsafe")
    entries = tuple(output_directory.iterdir())
    actual = {entry.name for entry in entries}
    if actual != PUBLIC_FILENAMES:
        missing = sorted(PUBLIC_FILENAMES - actual)
        extra = sorted(actual - PUBLIC_FILENAMES)
        raise Phase0AttestationError(
            f"attestation file set is incomplete (missing={missing}, extra={extra})"
        )
    if any(entry.is_symlink() or not entry.is_file() for entry in entries):
        raise Phase0AttestationError("attestation entries must be regular, non-symlink files")
    try:
        verification_key = bytes.fromhex(
            (output_directory / "verification-key.hex").read_text(encoding="ascii").strip()
        )
    except (OSError, UnicodeError, ValueError) as error:
        raise Phase0AttestationError("verification key is invalid") from error
    if len(verification_key) != 32:
        raise Phase0AttestationError("verification key must contain exactly 32 bytes")
    metadata_path = output_directory / "corpus-metadata.json"
    summary_path = output_directory / "license-summary.json"
    attestation_path = output_directory / "run-attestation.json"
    try:
        corpus_metadata = load_json(metadata_path)
        license_summary = load_json(summary_path)
        run_attestation = load_json(attestation_path)
    except ValueError as error:
        raise Phase0AttestationError(f"public attestation JSON is invalid: {error}") from error
    _require_canonical_file(metadata_path, corpus_metadata)
    _require_canonical_file(summary_path, license_summary)
    _require_canonical_file(attestation_path, run_attestation)
    bundle = Phase0AttestationBundle(
        verification_key,
        corpus_metadata,
        license_summary,
        run_attestation,
    )
    _verify_bundle(bundle)
    return bundle


def _verify_bundle(bundle: Phase0AttestationBundle) -> None:
    try:
        verify_document(
            bundle.run_attestation,
            PHASE0_ATTESTATION_DOMAIN,
            bundle.verification_key,
        )
    except AttestationError as error:
        raise Phase0AttestationError(str(error)) from error
    document = bundle.run_attestation
    if document.get("schema_version") != PHASE0_ATTESTATION_SCHEMA:
        raise Phase0AttestationError("unsupported Phase 0 attestation schema")
    if document.get("phase") != "phase0":
        raise Phase0AttestationError("run attestation names the wrong phase")
    if document.get("corpus_metadata") != bundle.corpus_metadata:
        raise Phase0AttestationError("corpus metadata differs from its signed copy")
    if document.get("license_summary") != bundle.license_summary:
        raise Phase0AttestationError("license summary differs from its signed copy")
    _validate_public_evidence(bundle.corpus_metadata, bundle.license_summary, document.get("run"))
    _assert_path_free(document)


def _validate_run(corpus: VerifiedCorpus, run: dict[str, Any]) -> None:
    if run.get("schema_version") != "clipmill.eval.run.v1":
        raise Phase0AttestationError("run manifest schema is invalid")
    if run.get("corpus_id") != corpus.corpus_id:
        raise Phase0AttestationError("run manifest names a different corpus")
    if run.get("corpus_signing_key") != corpus.signing_public_key.hex():
        raise Phase0AttestationError("run manifest names a different corpus signing key")
    _validate_run_header(run)
    raw_results = run.get("items")
    if not isinstance(raw_results, list) or len(raw_results) != 40:
        raise Phase0AttestationError("run manifest must contain exactly 40 results")
    results = _indexed_results(raw_results)
    expected = {item.item_id: item for item in corpus.items}
    if set(results) != set(expected):
        raise Phase0AttestationError("run results do not cover the exact signed corpus")
    for item_id, item in expected.items():
        result = results[item_id]
        if result.get("expected_result") != item.expected_result:
            raise Phase0AttestationError(f"result expectation changed for {item_id}")
        if item.expected_result == "success":
            _validate_success_result(item_id, result)
        else:
            _validate_failure_result(item_id, item.expected_failure, result)
    _assert_path_free(run)


def _validate_public_evidence(
    metadata: dict[str, Any],
    license_summary: dict[str, Any],
    run: Any,
) -> None:
    if metadata.get("schema_version") != CORPUS_METADATA_SCHEMA:
        raise Phase0AttestationError("corpus metadata schema is invalid")
    if license_summary.get("schema_version") != LICENSE_SUMMARY_SCHEMA:
        raise Phase0AttestationError("license summary schema is invalid")
    if metadata.get("items_total") != 40 or license_summary.get("items_total") != 40:
        raise Phase0AttestationError("public evidence does not describe exactly 40 items")
    if metadata.get("corpus_id") != license_summary.get("corpus_id"):
        raise Phase0AttestationError("public evidence corpus IDs disagree")
    if not isinstance(run, dict) or run.get("corpus_id") != metadata.get("corpus_id"):
        raise Phase0AttestationError("signed run names a different corpus")
    if run.get("corpus_signing_key") != metadata.get("corpus_signing_public_key"):
        raise Phase0AttestationError("signed run uses a different corpus key")
    if (
        not isinstance(metadata.get("corpus_signing_public_key"), str)
        or PUBLIC_KEY_PATTERN.fullmatch(metadata["corpus_signing_public_key"]) is None
    ):
        raise Phase0AttestationError("corpus signing public key is invalid")
    for field in ("corpus_manifest_sha256",):
        if not _is_digest(metadata.get(field)):
            raise Phase0AttestationError(f"{field} is invalid")
    if not _is_digest(license_summary.get("license_attestation_sha256")):
        raise Phase0AttestationError("license_attestation_sha256 is invalid")
    results = run.get("items")
    if not isinstance(results, list) or len(results) != 40:
        raise Phase0AttestationError("signed run does not contain exactly 40 results")
    _validate_run_header(run)
    indexed = _indexed_results(results)
    valid_count = 0
    hostile_count = 0
    for item_id, result in indexed.items():
        if result.get("expected_result") == "success":
            _validate_success_result(item_id, result)
            valid_count += 1
        elif result.get("expected_result") == "structured_failure":
            expected_failure = result.get("failure")
            if not isinstance(expected_failure, str) or not expected_failure:
                raise Phase0AttestationError(f"hostile result {item_id} omitted its failure")
            _validate_failure_result(item_id, expected_failure, result)
            hostile_count += 1
        else:
            raise Phase0AttestationError(f"result {item_id} has an invalid expectation")
    if metadata.get("valid_items") != valid_count or metadata.get("hostile_items") != hostile_count:
        raise Phase0AttestationError("public outcome counts differ from the signed run")
    redistributable = license_summary.get("redistributable_items")
    evaluation_only = license_summary.get("evaluation_only_items")
    if (
        not isinstance(redistributable, int)
        or isinstance(redistributable, bool)
        or not isinstance(evaluation_only, int)
        or isinstance(evaluation_only, bool)
        or redistributable + evaluation_only != 40
    ):
        raise Phase0AttestationError("license grant totals are invalid")
    license_counts = license_summary.get("license_counts")
    if (
        not isinstance(license_counts, dict)
        or any(not isinstance(key, str) or not key for key in license_counts)
        or any(
            not isinstance(value, int) or isinstance(value, bool) or value <= 0
            for value in license_counts.values()
        )
        or sum(license_counts.values()) != 40
    ):
        raise Phase0AttestationError("aggregate license counts are invalid")


def _validate_run_header(run: dict[str, Any]) -> None:
    daemon_version = run.get("daemon_version")
    if not isinstance(daemon_version, str) or not daemon_version or len(daemon_version) > 128:
        raise Phase0AttestationError("run daemon version is invalid")
    if run.get("policy") != "local-lock":
        raise Phase0AttestationError("Phase 0 evaluation did not run under Local Lock")
    if run.get("versions") != {
        "artifact_manifest": "clipmill.artifact.manifest.v1",
        "evaluation": "clipmill.eval.run.v1",
        "source_map": "clipmill.source_map.v1",
    }:
        raise Phase0AttestationError("run contract versions are incomplete or unsupported")
    started = run.get("started_unix_millis")
    completed = run.get("completed_unix_millis")
    if (
        not isinstance(started, int)
        or isinstance(started, bool)
        or started < 0
        or not isinstance(completed, int)
        or isinstance(completed, bool)
        or completed < started
    ):
        raise Phase0AttestationError("run timestamps are invalid")
    profile = run.get("hardware_profile")
    if not isinstance(profile, dict):
        raise Phase0AttestationError("run omitted its verified hardware profile")
    artifact_id = profile.get("artifact_id")
    fingerprint = profile.get("hardware_fingerprint")
    generation = profile.get("measurement_generation")
    if (
        not isinstance(artifact_id, str)
        or ARTIFACT_PATTERN.fullmatch(artifact_id) is None
        or not isinstance(fingerprint, str)
        or SOURCE_PATTERN.fullmatch(fingerprint) is None
        or not isinstance(generation, int)
        or isinstance(generation, bool)
        or generation < 1
    ):
        raise Phase0AttestationError("run hardware profile identity is invalid")


def _validate_success_result(item_id: str, result: dict[str, Any]) -> None:
    if result.get("observed_result") != "success" or result.get("warm_cache_hit") is not True:
        raise Phase0AttestationError(f"valid item {item_id} did not pass cold and warm")
    artifact_id = result.get("source_map_artifact_id")
    cold_id = result.get("cold_source_map_artifact_id")
    warm_id = result.get("warm_source_map_artifact_id")
    if (
        not isinstance(artifact_id, str)
        or ARTIFACT_PATTERN.fullmatch(artifact_id) is None
        or cold_id != artifact_id
        or warm_id != artifact_id
    ):
        raise Phase0AttestationError(f"valid item {item_id} changed artifact identity")
    if (
        not isinstance(result.get("source_fingerprint"), str)
        or SOURCE_PATTERN.fullmatch(result["source_fingerprint"]) is None
    ):
        raise Phase0AttestationError(f"valid item {item_id} has an invalid source fingerprint")
    if (
        result.get("artifact_key_version") != "clipmill.artifact.key.v1"
        or result.get("ffmpeg_bom") != "ffmpeg-8.1.2-btb-n8.1.2"
        or result.get("mapping_algorithm") != "clipmill.source-map.mapping.v1"
        or result.get("probe_algorithm") != "clipmill.ffprobe.normalize.v1"
        or result.get("source_map_schema_version") != "clipmill.source_map.v1"
        or not isinstance(result.get("producer_implementation"), str)
        or not result["producer_implementation"]
    ):
        raise Phase0AttestationError(f"valid item {item_id} omitted production versions")
    metrics = result.get("source_map_metrics")
    if (
        not isinstance(metrics, dict)
        or set(metrics) != {"chapters", "mapping_segments", "streams"}
        or any(
            not isinstance(value, int) or isinstance(value, bool) or value < 0
            for value in metrics.values()
        )
        or metrics["streams"] < 1
        or metrics["mapping_segments"] < 1
    ):
        raise Phase0AttestationError(f"valid item {item_id} has invalid source-map metrics")
    _validate_timings(item_id, result)


def _validate_failure_result(
    item_id: str,
    expected_failure: str | None,
    result: dict[str, Any],
) -> None:
    if (
        result.get("observed_result") != "structured_failure"
        or result.get("warm_observed_result") != "structured_failure"
        or result.get("failure") != expected_failure
        or "failure_code" not in result
    ):
        raise Phase0AttestationError(f"hostile item {item_id} did not repeat its expected failure")
    _validate_timings(item_id, result)


def _validate_timings(item_id: str, result: dict[str, Any]) -> None:
    for field in ("cold_millis", "warm_millis"):
        value = result.get(field)
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            raise Phase0AttestationError(f"result {item_id} has invalid {field}")


def _indexed_results(values: list[Any]) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for value in values:
        if not isinstance(value, dict):
            raise Phase0AttestationError("run results must be objects")
        item_id = value.get("item_id")
        if not isinstance(item_id, str) or not item_id or item_id in result:
            raise Phase0AttestationError("run result IDs are invalid or duplicated")
        result[item_id] = value
    return result


def _corpus_metadata(corpus: VerifiedCorpus) -> dict[str, Any]:
    valid = sum(item.expected_result == "success" for item in corpus.items)
    return {
        "schema_version": CORPUS_METADATA_SCHEMA,
        "corpus_id": corpus.corpus_id,
        "corpus_manifest_sha256": corpus.manifest_sha256,
        "corpus_signing_public_key": corpus.signing_public_key.hex(),
        "items_total": len(corpus.items),
        "valid_items": valid,
        "hostile_items": len(corpus.items) - valid,
    }


def _license_summary(corpus: VerifiedCorpus) -> dict[str, Any]:
    counts = Counter(grant.license_id for grant in corpus.licenses)
    redistributable = sum(grant.redistributable for grant in corpus.licenses)
    return {
        "schema_version": LICENSE_SUMMARY_SCHEMA,
        "corpus_id": corpus.corpus_id,
        "license_attestation_sha256": corpus.license_attestation_sha256,
        "items_total": len(corpus.licenses),
        "redistributable_items": redistributable,
        "evaluation_only_items": len(corpus.licenses) - redistributable,
        "license_counts": dict(sorted(counts.items())),
    }


def _assert_path_free(value: Any, location: str = "$") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if not isinstance(key, str) or _unsafe_public_string(key):
                raise Phase0AttestationError(f"unsafe public field name at {location}")
            normalized = key.casefold()
            if normalized in {
                "path",
                "absolute_path",
                "relative_path",
                "corpus_dir",
                "data_dir",
                "working_directory",
                "socket",
            } or normalized.endswith(("_path", "_dir", "_directory")):
                raise Phase0AttestationError(f"private path field is forbidden at {location}.{key}")
            _assert_path_free(child, f"{location}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _assert_path_free(child, f"{location}[{index}]")
    elif isinstance(value, str) and _unsafe_public_string(value):
        raise Phase0AttestationError(f"unsafe or path-shaped string leaked at {location}")


def _is_digest(value: Any) -> bool:
    return isinstance(value, str) and SHA256_PATTERN.fullmatch(value) is not None


def _unsafe_public_string(value: str) -> bool:
    return (
        "/" in value
        or "\\" in value
        or any(unicodedata.category(character).startswith("C") for character in value)
    )


def _require_canonical_file(path: Path, value: dict[str, Any]) -> None:
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise Phase0AttestationError(f"cannot read {path.name}: {error}") from error
    if raw != canonical_json(value) + b"\n":
        raise Phase0AttestationError(f"{path.name} is not canonical JSON")


def _atomic_write(path: Path, contents: bytes) -> None:
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        os.fchmod(descriptor, 0o644)
        with os.fdopen(descriptor, "wb") as output:
            output.write(contents)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    except BaseException:
        with suppress(OSError):
            os.close(descriptor)
        temporary.unlink(missing_ok=True)
        raise
