#!/usr/bin/env python3
"""Inspect every uv-locked environment against the Python license policy."""

from __future__ import annotations

import importlib.metadata
import subprocess
import sys
from pathlib import Path

ALLOWED = {
    "AGPL-3.0-only",
    "Apache-2.0",
    "Apache-2.0 OR BSD-2-Clause",
    "Apache-2.0 OR BSD-3-Clause",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "MIT",
    "MIT-0",
    "PSF-2.0",
}
ALIASES = {
    "3-Clause BSD License": "BSD-3-Clause",
    "BSD License": "BSD-3-Clause",
}
PACKAGE_OVERRIDES = {"annotated-types": "MIT"}
CLASSIFIER_LICENSES = {
    "License :: OSI Approved :: Apache Software License": "Apache-2.0",
    "License :: OSI Approved :: BSD License": "BSD-3-Clause",
    "License :: OSI Approved :: MIT License": "MIT",
    "License :: OSI Approved :: Python Software Foundation License": "PSF-2.0",
}


def main() -> int:
    if len(sys.argv) == 2 and sys.argv[1] == "--child":
        return check_current_environment()
    lockfiles = sorted(Path(".").glob("**/uv.lock"))
    if not lockfiles:
        return fail("no uv.lock files found")
    script = Path(__file__).resolve()
    for lockfile in lockfiles:
        project = lockfile.parent
        result = subprocess.run(
            [
                "uv",
                "run",
                "--offline",
                "--frozen",
                "--project",
                str(project),
                "python",
                str(script),
                "--child",
            ],
            text=True,
        )
        if result.returncode != 0:
            return fail(f"license validation failed for {project}")
    print(f"python-licenses: OK ({len(lockfiles)} uv lockfiles)")
    return 0


def check_current_environment() -> int:
    rejected: list[str] = []
    checked = 0
    for distribution in importlib.metadata.distributions():
        name = canonical_name(distribution.metadata.get("Name", ""))
        if not name:
            rejected.append("unnamed distribution")
            continue
        expression = distribution.metadata.get("License-Expression")
        raw_license = expression or distribution.metadata.get("License")
        license_name = PACKAGE_OVERRIDES.get(name)
        if license_name is None and raw_license and raw_license != "UNKNOWN":
            license_name = ALIASES.get(raw_license, raw_license)
        if license_name is None:
            classifiers = distribution.metadata.get_all("Classifier", [])
            matches = {
                CLASSIFIER_LICENSES[value] for value in classifiers if value in CLASSIFIER_LICENSES
            }
            if len(matches) == 1:
                license_name = matches.pop()
        checked += 1
        if license_name not in ALLOWED:
            rejected.append(f"{name}=={distribution.version}: {license_name or 'UNKNOWN'}")
    if rejected:
        for rejection in sorted(rejected):
            print(f"python-licenses: rejected {rejection}", file=sys.stderr)
        return 1
    print(f"python-licenses: checked {checked} installed distributions")
    return 0


def canonical_name(value: str) -> str:
    return value.casefold().replace("_", "-").replace(".", "-")


def fail(message: str) -> int:
    print(f"python-licenses: {message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
