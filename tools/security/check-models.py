#!/usr/bin/env python3
"""Validate the pinned model registry and enforce the licence-class policy.

Weights are not a dependency like any other: they end up inside the frames and
the audio a user publishes. A model whose licence forbids commercial use would
quietly make every clip a creator sells a licence violation, and they would
find out from someone else. So the allowlist is narrow, the default is refusal,
and a manifest that cannot state its terms is treated as one that does not
permit them (book ch. 11).
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path
from urllib.parse import urlparse

SHA256_PATTERN = re.compile(r"^[0-9a-f]{64}$")
NAME_PATTERN = re.compile(r"^[a-z0-9]+(?:-[a-z0-9.]+)*$")

# Classes whose terms permit shipping the model's output inside work a user
# sells. Anything else is refused rather than reviewed case by case at 2am.
ALLOWED_CLASSES = {"permissive"}
REFUSED_CLASSES = {
    "noncommercial": "forbids the commercial use every creator tool implies",
    "research-only": "forbids production use",
    "copyleft": "would impose its terms on rendered output",
    "proprietary": "cannot be redistributed with the application",
    "unknown": "states no terms, which is not the same as permitting them",
}
# SPDX identifiers that may claim the permissive class.
PERMISSIVE_SPDX = {"MIT", "Apache-2.0", "BSD-3-Clause", "BSD-2-Clause", "CC0-1.0", "ISC"}
# Runtimes and backends a manifest may name. A typo here would otherwise
# become a model nothing can load, discovered at the first transcription.
RUNTIMES = {"onnxruntime", "whisper.cpp", "mlx", "ggml"}
BACKENDS = {"cpu", "onnx-cpu", "mlx", "coreml", "cuda"}
CAPABILITIES = {"vad", "asr", "forced-align", "detect-faces", "detect-shots"}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--registry", type=Path, default=Path("models/registry"))
    options = parser.parse_args()

    manifests = sorted(options.registry.glob("*.toml"))
    if not manifests:
        print(f"model-policy: no manifests in {options.registry}", file=sys.stderr)
        return 1
    errors: list[str] = []
    for path in manifests:
        errors.extend(f"{path.name}: {problem}" for problem in _check(path))
    if errors:
        for error in errors:
            print(f"model-policy: {error}", file=sys.stderr)
        return 1
    print(
        f"model-policy: OK ({len(manifests)} models; pinned revisions and digests, "
        "licence class permits publication)"
    )
    return 0


def _check(path: Path) -> list[str]:
    try:
        manifest = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [f"unreadable: {error}"]

    problems: list[str] = []
    name = manifest.get("name")
    if not isinstance(name, str) or NAME_PATTERN.fullmatch(name) is None:
        problems.append("name is missing or not a simple identifier")
    elif name != path.stem:
        problems.append(f"name {name!r} does not match its file")
    if manifest.get("capability") not in CAPABILITIES:
        problems.append(f"capability {manifest.get('capability')!r} is not one this phase defines")
    if manifest.get("runtime") not in RUNTIMES:
        problems.append(f"runtime {manifest.get('runtime')!r} is unknown")
    if manifest.get("backend") not in BACKENDS:
        problems.append(f"backend {manifest.get('backend')!r} is unknown")
    if not isinstance(manifest.get("quantization"), str) or not manifest["quantization"]:
        problems.append("quantization must be stated, even when it is 'none'")

    problems.extend(_check_source(manifest.get("source")))
    problems.extend(_check_license(manifest.get("license")))
    problems.extend(_check_memory(manifest.get("memory"), manifest.get("files")))
    problems.extend(_check_files(manifest.get("files")))
    return problems


def _check_source(source: object) -> list[str]:
    if not isinstance(source, dict):
        return ["source table is missing"]
    problems: list[str] = []
    provider = urlparse(str(source.get("provider", "")))
    if provider.scheme != "https" or not provider.hostname:
        problems.append("source provider must be an HTTPS host")
    repo = source.get("repo")
    if not isinstance(repo, str) or repo.count("/") != 1 or repo.startswith("/"):
        problems.append("source repo must be '<owner>/<name>'")
    revision = source.get("revision")
    # A tag or branch can move under a pin; a commit cannot.
    if not isinstance(revision, str) or re.fullmatch(r"[0-9a-f]{40}", revision) is None:
        problems.append("source revision must be a full commit hash, not a tag")
    return problems


def _check_license(license_table: object) -> list[str]:
    if not isinstance(license_table, dict):
        return ["license table is missing, which is not the same as permissive"]
    problems: list[str] = []
    class_ = license_table.get("class")
    if class_ in REFUSED_CLASSES:
        problems.append(f"licence class {class_!r} {REFUSED_CLASSES[class_]}")
    elif class_ not in ALLOWED_CLASSES:
        problems.append(f"licence class {class_!r} is not on the allowlist")
    spdx = license_table.get("spdx")
    if not isinstance(spdx, str) or not spdx:
        problems.append("licence must carry an SPDX identifier")
    elif class_ == "permissive" and spdx not in PERMISSIVE_SPDX:
        problems.append(f"{spdx} is claimed permissive but is not a permissive identifier")
    digest = license_table.get("sha256")
    if digest is not None and SHA256_PATTERN.fullmatch(str(digest)) is None:
        problems.append("licence text digest is malformed")
    if ("file" in license_table) != ("sha256" in license_table):
        problems.append("a licence file and its digest must appear together")
    return problems


def _check_memory(memory: object, files: object) -> list[str]:
    if not isinstance(memory, dict):
        return ["memory table is missing; admission needs a budget to check"]
    problems: list[str] = []
    weights = memory.get("weights_bytes")
    overhead = memory.get("runtime_overhead_bytes")
    for label, value in (("weights_bytes", weights), ("runtime_overhead_bytes", overhead)):
        if not isinstance(value, int) or value <= 0:
            problems.append(f"{label} must be a positive integer")
    if isinstance(weights, int) and isinstance(files, list):
        declared = sum(f.get("bytes", 0) for f in files if isinstance(f, dict))
        if declared and declared != weights:
            problems.append(f"weights_bytes {weights} does not match the pinned files' {declared}")
    return problems


def _check_files(files: object) -> list[str]:
    if not isinstance(files, list) or not files:
        return ["a model must pin at least one file"]
    problems: list[str] = []
    seen: set[str] = set()
    for entry in files:
        if not isinstance(entry, dict):
            problems.append("file entries must be tables")
            continue
        path = entry.get("path")
        if not isinstance(path, str) or not path:
            problems.append("a pinned file has no path")
            continue
        candidate = Path(path)
        if candidate.is_absolute() or ".." in candidate.parts:
            problems.append(f"pinned path {path!r} escapes the model directory")
        if path in seen:
            problems.append(f"pinned path {path!r} appears twice")
        seen.add(path)
        if SHA256_PATTERN.fullmatch(str(entry.get("sha256"))) is None:
            problems.append(f"{path} has no valid sha256")
        size = entry.get("bytes")
        if not isinstance(size, int) or size <= 0:
            problems.append(f"{path} has no positive byte size")
    return problems


if __name__ == "__main__":
    raise SystemExit(main())
