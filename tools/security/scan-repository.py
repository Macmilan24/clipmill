#!/usr/bin/env python3
"""Keep private keys, restricted media, and oversized binary blobs out of Git."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

MEDIA_SUFFIXES = {
    ".avi",
    ".flac",
    ".m4a",
    ".mkv",
    ".mov",
    ".mp3",
    ".mp4",
    ".mpeg",
    ".mpg",
    ".wav",
    ".webm",
}
KEY_SUFFIXES = {".key", ".p12", ".pfx", ".pem"}
SEED40_PUBLIC_FILES = {
    "eval/seed40/corpus-metadata.json",
    "eval/seed40/license-summary.json",
    "eval/seed40/run-attestation.json",
    "eval/seed40/verification-key.hex",
}
MAX_TRACKED_BYTES = 5 * 1024 * 1024


def main() -> int:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        check=True,
        stdout=subprocess.PIPE,
    )
    paths = [Path(value.decode("utf-8")) for value in result.stdout.split(b"\0") if value]
    errors: list[str] = []
    private_marker = b"-----BEGIN " + b"PRIVATE KEY-----"
    openssh_marker = b"-----BEGIN OPENSSH " + b"PRIVATE KEY-----"
    github_prefixes = (b"gh" + b"p_", b"github_pat" + b"_")
    for path in paths:
        portable = path.as_posix()
        suffix = path.suffix.casefold()
        if suffix in MEDIA_SUFFIXES:
            errors.append(f"restricted media extension is tracked: {portable}")
        if suffix in KEY_SUFFIXES or path.name in {"id_ed25519", "id_rsa"}:
            errors.append(f"private-key-shaped file is tracked: {portable}")
        if portable.startswith("eval/seed40/") and portable not in SEED40_PUBLIC_FILES:
            errors.append(f"private Seed-40 material is tracked: {portable}")
        try:
            size = path.stat().st_size
            data = path.read_bytes()
        except OSError as error:
            errors.append(f"cannot inspect tracked file {portable}: {error}")
            continue
        if size > MAX_TRACKED_BYTES:
            errors.append(
                f"oversized tracked file requires explicit review: {portable} ({size} bytes)"
            )
        if private_marker in data or openssh_marker in data:
            errors.append(f"private key material appears in {portable}")
        if any(prefix in data for prefix in github_prefixes):
            errors.append(f"GitHub credential pattern appears in {portable}")
    if errors:
        for error in errors:
            print(f"repository-scan: {error}", file=sys.stderr)
        return 1
    print(
        f"repository-scan: OK ({len(paths)} versioned/visible files; "
        "no keys, private media, or large blobs)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
