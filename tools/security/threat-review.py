#!/usr/bin/env python3
"""Require explicit PR threat review for sensitive boundary changes."""

from __future__ import annotations

import argparse
import fnmatch
import os
import re
import subprocess
import sys

CATEGORIES = {
    "Hostile input and parsers": (
        "contracts/**",
        "crates/clipmilld/src/sources.rs",
        "crates/clipmilld/src/probe.rs",
        "eval/harness/**",
    ),
    "IPC and worker authentication": (
        "contracts/proto/**",
        "crates/clipmilld/src/ipc.rs",
        "crates/clipmilld/src/service.rs",
        "crates/clipmilld/src/worker.rs",
        "workers/**",
    ),
    "Filesystem publication and paths": (
        "crates/clipmill-artifacts/**",
        "crates/clipmilld/src/db/**",
        "crates/clipmilld/src/db.rs",
        "tools/drills/**",
    ),
    "Subprocess and sandbox": (
        "crates/clipmilld/src/probe.rs",
        "crates/clipmilld/src/device.rs",
        "workers/**",
        "tools/fetch-ffmpeg.sh",
    ),
    "Secrets, logs, and credentials": (
        ".github/**",
        "crates/clipmilld/src/device.rs",
        "eval/harness/src/clipmill_eval/attestation.py",
        "tools/security/**",
    ),
    "Network policy and egress": (
        ".github/workflows/**",
        "docs/local-lock.md",
        "tools/drills/network-denial.sh",
    ),
    "Dependencies, licenses, and generated code": (
        "Cargo.toml",
        "**/Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "**/pyproject.toml",
        "uv.lock",
        "**/uv.lock",
        "package.json",
        "pnpm-lock.yaml",
        "bom.toml",
        "deny.toml",
        "tools/codegen/**",
    ),
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", default="origin/main")
    parser.add_argument("--body", default=os.environ.get("PR_BODY", ""))
    options = parser.parse_args()
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", f"{options.base}...HEAD"],
            check=True,
            stdout=subprocess.PIPE,
            text=True,
        )
    except (OSError, subprocess.SubprocessError) as error:
        print(f"threat-review: cannot resolve PR diff: {error}", file=sys.stderr)
        return 1
    changed = tuple(line for line in result.stdout.splitlines() if line)
    required = {
        category
        for category, patterns in CATEGORIES.items()
        if any(_matches(path, patterns) for path in changed)
    }
    if not required:
        print("threat-review: OK (no sensitive boundary changed)")
        return 0
    missing = [category for category in sorted(required) if not _is_checked(options.body, category)]
    if missing:
        print("threat-review: sensitive changes require checked PR declarations:", file=sys.stderr)
        for category in missing:
            print(f"  - [ ] {category}", file=sys.stderr)
        return 1
    print(f"threat-review: OK ({len(required)} sensitive categories explicitly reviewed)")
    return 0


def _matches(path: str, patterns: tuple[str, ...]) -> bool:
    return any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns)


def _is_checked(body: str, category: str) -> bool:
    pattern = rf"^\s*-\s*\[[xX]\]\s*{re.escape(category)}\s*$"
    return re.search(pattern, body, re.MULTILINE) is not None


if __name__ == "__main__":
    raise SystemExit(main())
