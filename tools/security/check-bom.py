#!/usr/bin/env python3
"""Validate pinned substrate metadata and the installed FFmpeg license build."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from urllib.parse import urlparse

SHA256_PATTERN = re.compile(r"^[0-9a-f]{64}$")
BUILD_PATTERN = re.compile(r"^[0-9]+_[0-9]+\.[0-9]+\.[0-9]+$")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bom", type=Path, default=Path("bom.toml"))
    parser.add_argument("--ffmpeg", type=Path, default=Path(".cache/bin/ffmpeg"))
    parser.add_argument("--ffprobe", type=Path, default=Path(".cache/bin/ffprobe"))
    options = parser.parse_args()
    try:
        bom = tomllib.loads(options.bom.read_text(encoding="utf-8"))
        ffmpeg = bom["ffmpeg"]
        version = ffmpeg["version"]
        if not isinstance(version, str) or re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version) is None:
            raise ValueError("FFmpeg version is invalid")
        provider_host = urlparse(str(ffmpeg["provider"]).split(" ", 1)[0]).hostname
        if not provider_host:
            raise ValueError("FFmpeg provider must be HTTPS")
        platforms = {key for key, value in ffmpeg.items() if isinstance(value, dict)}
        if platforms != {"macos-arm64", "linux-amd64"}:
            raise ValueError("FFmpeg BOM must pin exactly macOS arm64 and Linux amd64")
        for platform in sorted(platforms):
            entry = ffmpeg[platform]
            build = entry.get("build")
            if (
                not isinstance(build, str)
                or BUILD_PATTERN.fullmatch(build) is None
                or not build.endswith(version)
            ):
                raise ValueError(f"{platform} build identity is invalid")
            for binary in ("ffmpeg", "ffprobe"):
                url = entry.get(f"{binary}_url")
                digest = entry.get(f"{binary}_sha256")
                parsed = urlparse(str(url))
                if parsed.scheme != "https" or parsed.hostname != provider_host:
                    raise ValueError(f"{platform} {binary} URL has an untrusted provider")
                if build not in parsed.path:
                    raise ValueError(f"{platform} {binary} URL omits its build identity")
                if not isinstance(digest, str) or SHA256_PATTERN.fullmatch(digest) is None:
                    raise ValueError(f"{platform} {binary} digest is invalid")
        sqlite = bom["sqlite"]
        if sqlite.get("min_version") != "3.51.3" or sqlite.get("min_version_number") != 3051003:
            raise ValueError("SQLite corruption-fix floor changed without a BOM decision")
        for name, path in (("ffmpeg", options.ffmpeg), ("ffprobe", options.ffprobe)):
            _verify_binary(name, path, version)
    except (
        KeyError,
        OSError,
        subprocess.SubprocessError,
        tomllib.TOMLDecodeError,
        ValueError,
    ) as error:
        print(f"bom-policy: {error}", file=sys.stderr)
        return 1
    print("bom-policy: OK (pinned hashes; GPL/version3 FFmpeg; nonfree disabled; SQLite floor)")
    return 0


def _verify_binary(name: str, path: Path, version: str) -> None:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"installed {name} is missing or unsafe: {path}")
    result = subprocess.run(
        [str(path), "-hide_banner", "-version"],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=10,
    )
    output = result.stdout
    first_line = output.splitlines()[0] if output else ""
    if not first_line.startswith(f"{name} version {version}"):
        raise ValueError(f"installed {name} does not match BOM version {version}")
    if "--enable-gpl" not in output or "--enable-version3" not in output:
        raise ValueError(f"installed {name} omitted its declared GPL/version3 license flags")
    if "--enable-nonfree" in output:
        raise ValueError(f"installed {name} enables non-redistributable components")


if __name__ == "__main__":
    raise SystemExit(main())
