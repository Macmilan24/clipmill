#!/usr/bin/env python3
"""Enforce the immutable, read-only GitHub Actions policy."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ACTION_PATTERN = re.compile(
    r"^\s*-\s+uses:\s+([^@\s]+)@([^\s#]+)(?:\s+#\s+(.+))?\s*$", re.MULTILINE
)
JOB_PATTERN = re.compile(r"^  ([a-zA-Z0-9_-]+):\s*$", re.MULTILINE)
PIN_PATTERN = re.compile(r"^[0-9a-f]{40}$")


def main() -> int:
    workflows = sorted(Path(".github/workflows").glob("*.y*ml"))
    if not workflows:
        return fail("no GitHub Actions workflows found")
    errors: list[str] = []
    for path in workflows:
        text = path.read_text(encoding="utf-8")
        if re.search(r"^permissions:\s*\n\s+contents:\s+read\s*$", text, re.MULTILINE) is None:
            errors.append(f"{path}: workflow permissions must be contents: read")
        if re.search(r"^\s+cancel-in-progress:\s+true\s*$", text, re.MULTILINE) is None:
            errors.append(f"{path}: pull-request concurrency cancellation is missing")
        if re.search(r"^\s+[a-zA-Z_-]+:\s+write\s*$", text, re.MULTILINE):
            errors.append(f"{path}: write workflow permission is forbidden")
        jobs_start = re.search(r"^jobs:\s*$", text, re.MULTILINE)
        if jobs_start is None:
            errors.append(f"{path}: jobs mapping is missing")
            continue
        jobs_text = text[jobs_start.end() :]
        matches = list(JOB_PATTERN.finditer(jobs_text))
        if not matches:
            errors.append(f"{path}: workflow has no jobs")
        for index, match in enumerate(matches):
            end = matches[index + 1].start() if index + 1 < len(matches) else len(jobs_text)
            block = jobs_text[match.end() : end]
            job = match.group(1)
            if re.search(r"^\s{4}timeout-minutes:\s+[1-9][0-9]*\s*$", block, re.MULTILINE) is None:
                errors.append(f"{path}:{job}: explicit timeout-minutes is missing")
            if "actions/checkout@" in block and not re.search(
                r"^\s+persist-credentials:\s+false\s*$", block, re.MULTILINE
            ):
                errors.append(f"{path}:{job}: checkout credentials must not persist")
        for action, revision, comment in ACTION_PATTERN.findall(text):
            if action.startswith("./"):
                continue
            if PIN_PATTERN.fullmatch(revision) is None:
                errors.append(f"{path}: {action} is not pinned to a full commit")
            if not comment or re.search(r"\bv?[0-9]+(?:\.[0-9]+){1,2}\b", comment) is None:
                errors.append(f"{path}: {action} pin needs a version comment")
    if errors:
        for error in errors:
            print(f"actions-policy: {error}", file=sys.stderr)
        return 1
    print(f"actions-policy: OK ({len(workflows)} workflow; immutable pins, read-only permissions)")
    return 0


def fail(message: str) -> int:
    print(f"actions-policy: {message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
