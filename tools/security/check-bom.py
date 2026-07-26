#!/usr/bin/env python3
"""Validate pinned substrate metadata and the installed FFmpeg license build."""

from __future__ import annotations

import argparse
import hashlib
import platform as host_platform
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from urllib.parse import urlparse

SHA256_PATTERN = re.compile(r"^[0-9a-f]{64}$")
BUILD_PATTERN = re.compile(r"^[0-9]+_[0-9]+\.[0-9]+\.[0-9]+$")
LICENSE_POLICY = {
    "macos-arm64": ("gpl-v3", True),
    "linux-amd64": ("gpl-v3-nonfree", False),
}
# Fonts ship inside the rendered pixels of every clip a user publishes, so the
# licence has to permit that without a per-user grant.
FONT_LICENSE_ALLOWLIST = {"OFL-1.1", "Apache-2.0", "CC0-1.0"}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bom", type=Path, default=Path("bom.toml"))
    parser.add_argument("--ffmpeg", type=Path, default=Path(".cache/bin/ffmpeg"))
    parser.add_argument("--ffprobe", type=Path, default=Path(".cache/bin/ffprobe"))
    parser.add_argument("--fonts", type=Path, default=Path(".cache/fonts"))
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
            expected_license, expected_redistributable = LICENSE_POLICY[platform]
            if (
                entry.get("license_mode") != expected_license
                or entry.get("redistributable") is not expected_redistributable
            ):
                raise ValueError(f"{platform} license/distribution policy is invalid")
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
        current_platform = _current_platform()
        allow_nonfree = ffmpeg[current_platform]["license_mode"] == "gpl-v3-nonfree"
        for name, path in (("ffmpeg", options.ffmpeg), ("ffprobe", options.ffprobe)):
            _verify_binary(name, path, version, allow_nonfree)
        _verify_font(bom, options.fonts)
    except (
        KeyError,
        OSError,
        subprocess.SubprocessError,
        tomllib.TOMLDecodeError,
        ValueError,
    ) as error:
        print(f"bom-policy: {error}", file=sys.stderr)
        return 1
    print(
        "bom-policy: OK (pinned hashes; runtime license flags match the "
        "platform distribution policy; SQLite floor; caption font and libass)"
    )
    return 0


def _current_platform() -> str:
    system = host_platform.system()
    machine = host_platform.machine().casefold()
    if system == "Darwin" and machine == "arm64":
        return "macos-arm64"
    if system == "Linux" and machine in {"amd64", "x86_64"}:
        return "linux-amd64"
    raise ValueError(f"unsupported BOM verification platform: {system}-{machine}")


def _verify_binary(name: str, path: Path, version: str, allow_nonfree: bool) -> None:
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
    has_nonfree = "--enable-nonfree" in output
    if has_nonfree != allow_nonfree:
        raise ValueError(f"installed {name} nonfree mode differs from its BOM policy")
    # Captions burn in through libass. A build without it cannot produce a
    # compliant clip, so its absence is a policy failure rather than a
    # surprise discovered mid-render.
    if name == "ffmpeg" and "--enable-libass" not in output:
        raise ValueError("installed ffmpeg has no libass; captions cannot be burned in")


def _verify_font(bom: dict, fonts_dir: Path) -> None:
    """The one face libass may see, pinned by archive *and* member digest."""
    captions = bom["fonts"]["captions"]
    family = str(captions["family"])
    style = str(captions["style"])
    if not family.isalnum() or not style.isalnum():
        raise ValueError("caption font family and style must be simple names")
    if captions.get("license") not in FONT_LICENSE_ALLOWLIST:
        raise ValueError(f"caption font license {captions.get('license')!r} is not permitted")
    provider_host = urlparse(str(captions["provider"]).split(" ", 1)[0]).hostname
    archive = urlparse(str(captions["archive_url"]))
    if archive.scheme != "https" or archive.hostname != provider_host:
        raise ValueError("caption font archive has an untrusted provider")
    if str(captions["version"]) not in archive.path:
        raise ValueError("caption font URL omits its version identity")
    for key in ("archive_sha256", "member_sha256", "license_sha256"):
        if SHA256_PATTERN.fullmatch(str(captions.get(key))) is None:
            raise ValueError(f"caption font {key} is invalid")
    member = Path(str(captions["member"]))
    if member.is_absolute() or ".." in member.parts:
        raise ValueError("caption font member escapes its archive")

    installed = fonts_dir / f"{family}-{style}.ttf"
    if installed.is_symlink() or not installed.is_file():
        raise ValueError(f"pinned caption font is missing or unsafe: {installed}")
    digest = hashlib.sha256(installed.read_bytes()).hexdigest()
    if digest != captions["member_sha256"]:
        raise ValueError("installed caption font does not match its pinned digest")
    licence = fonts_dir / f"{family}-LICENSE.txt"
    if not licence.is_file():
        raise ValueError(f"caption font licence text was not installed beside it: {licence}")


if __name__ == "__main__":
    raise SystemExit(main())
