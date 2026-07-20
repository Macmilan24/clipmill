"""Generate the redistributable synthetic Phase 0 smoke corpus."""

from __future__ import annotations

import hashlib
import os
import subprocess
from pathlib import Path

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

from .corpus import CORPUS_DOMAIN, LICENSE_DOMAIN
from .signing import canonical_json, sign_document

SMOKE_ITEMS = (
    ("cfr", "cfr.mkv", "success", None),
    ("vfr", "vfr.mkv", "success", None),
    ("rotation", "rotation.mkv", "success", None),
    ("audio-offset", "audio-offset.mkv", "success", None),
    ("malformed", "malformed.mkv", "structured_failure", "probe"),
)


def build_smoke_corpus(output: Path, ffmpeg: Path) -> tuple[Path, Path, Path]:
    """Create tiny synthetic media plus ephemeral signed metadata."""

    output.mkdir(parents=True, exist_ok=True)
    _run_ffmpeg(
        ffmpeg,
        output,
        [
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=160x90:rate=30000/1001:duration=1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c:v",
            "ffv1",
            "-c:a",
            "pcm_s16le",
            "cfr.mkv",
        ],
    )
    _run_ffmpeg(
        ffmpeg,
        output,
        [
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=160x90:rate=24:duration=0.5",
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=160x90:rate=30:duration=0.5",
            "-filter_complex",
            "[0:v][1:v]concat=n=2:v=1:a=0[v]",
            "-map",
            "[v]",
            "-fps_mode",
            "vfr",
            "-c:v",
            "ffv1",
            "vfr.mkv",
        ],
    )
    _run_ffmpeg(
        ffmpeg,
        output,
        [
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=160x90:rate=30:duration=1",
            "-metadata:s:v:0",
            "rotate=90",
            "-c:v",
            "ffv1",
            "rotation.mkv",
        ],
    )
    _run_ffmpeg(
        ffmpeg,
        output,
        [
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=160x90:rate=30:duration=1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=0.75",
            "-itsoffset",
            "0.25",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=880:duration=0.75",
            "-map",
            "0:v",
            "-map",
            "1:a",
            "-map",
            "2:a",
            "-c:v",
            "ffv1",
            "-c:a",
            "pcm_s16le",
            "audio-offset.mkv",
        ],
    )
    (output / "malformed.mkv").write_bytes(b"ClipMill intentionally malformed smoke media\n")

    items = []
    licenses = []
    for item_id, relative_path, expected_result, expected_failure in SMOKE_ITEMS:
        path = output / relative_path
        record = {
            "item_id": item_id,
            "relative_path": relative_path,
            "sha256": _sha256(path),
            "bytes": path.stat().st_size,
            "expected_result": expected_result,
            "license_id": "CC0-1.0-synthetic",
        }
        if expected_failure is not None:
            record["expected_failure"] = expected_failure
        items.append(record)
        licenses.append(
            {
                "item_id": item_id,
                "license_id": "CC0-1.0-synthetic",
                "redistributable": True,
            }
        )
    signing_key = Ed25519PrivateKey.generate()
    manifest = sign_document(
        {
            "schema_version": "clipmill.corpus_manifest.v1",
            "corpus_id": "phase0-public-smoke-v1",
            "items": items,
        },
        signing_key,
        CORPUS_DOMAIN,
    )
    license_attestation = sign_document(
        {
            "schema_version": "clipmill.license_attestation.v1",
            "corpus_id": "phase0-public-smoke-v1",
            "licenses": licenses,
            "statement": "Synthetic FFmpeg lavfi fixtures are dedicated to CC0-1.0.",
        },
        signing_key,
        LICENSE_DOMAIN,
    )
    manifest_path = output / "corpus-manifest.json"
    license_path = output / "license-attestation.json"
    manifest_path.write_bytes(canonical_json(manifest) + b"\n")
    license_path.write_bytes(canonical_json(license_attestation) + b"\n")
    return output, manifest_path, license_path


def _run_ffmpeg(ffmpeg: Path, output: Path, arguments: list[str]) -> None:
    environment = {
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
    }
    try:
        subprocess.run(
            [
                str(ffmpeg),
                "-hide_banner",
                "-nostdin",
                "-loglevel",
                "error",
                "-y",
                *arguments,
            ],
            cwd=output,
            env=environment,
            check=True,
            timeout=15,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise RuntimeError(f"smoke corpus FFmpeg generation failed: {error}") from error


def _sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()
