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
# Every pinned build is one a released app may carry (R4): GPL, never
# nonfree. A build that is not redistributable cannot be pinned at all.
LICENSE_POLICY = {
    "macos-arm64": ("gpl-v3", True),
    "linux-amd64": ("gpl-v3", True),
    "windows-amd64": ("gpl-v3", True),
}
UV_PLATFORMS = {"macos-arm64", "linux-amd64", "windows-amd64"}
# Fonts ship inside the rendered pixels of every clip a user publishes, so the
# licence has to permit that without a per-user grant.
FONT_LICENSE_ALLOWLIST = {"OFL-1.1", "Apache-2.0", "CC0-1.0"}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bom", type=Path, default=Path("bom.toml"))
    parser.add_argument("--ffmpeg", type=Path, default=Path(".cache/bin/ffmpeg"))
    parser.add_argument("--ffprobe", type=Path, default=Path(".cache/bin/ffprobe"))
    parser.add_argument("--fonts", type=Path, default=Path(".cache/fonts"))
    parser.add_argument("--emoji", type=Path, default=Path(".cache/emoji"))
    options = parser.parse_args()
    try:
        bom = tomllib.loads(options.bom.read_text(encoding="utf-8"))
        ffmpeg = bom["ffmpeg"]
        platforms = {key for key, value in ffmpeg.items() if isinstance(value, dict)}
        if platforms != set(LICENSE_POLICY):
            raise ValueError(
                "FFmpeg BOM must pin exactly macOS arm64, Linux amd64 and Windows amd64"
            )
        for platform in sorted(platforms):
            _check_ffmpeg_pin(platform, ffmpeg, ffmpeg[platform])
        _check_uv(bom)
        sqlite = bom["sqlite"]
        if sqlite.get("min_version") != "3.51.3" or sqlite.get("min_version_number") != 3051003:
            raise ValueError("SQLite corruption-fix floor changed without a BOM decision")
        current_platform = _current_platform()
        entry = ffmpeg[current_platform]
        reports = str(entry.get("reports", entry.get("version", ffmpeg["version"])))
        for name, path in (("ffmpeg", options.ffmpeg), ("ffprobe", options.ffprobe)):
            _verify_binary(name, path, reports)
        _verify_font(bom, options.fonts)
        _verify_emoji(bom, options.emoji)
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
        "platform distribution policy; SQLite floor; caption font and libass; emoji)"
    )
    return 0


def _check_ffmpeg_pin(platform: str, ffmpeg: dict, entry: dict) -> None:
    """One platform's build: its provider, identity, licence and every digest.

    A platform either pins ffmpeg and ffprobe as two files, or pins one
    archive and names the two members taken out of it.
    """
    version = str(entry.get("version", ffmpeg["version"]))
    if re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version) is None:
        raise ValueError(f"{platform} FFmpeg version is invalid")
    provider_host = urlparse(
        str(entry.get("provider", ffmpeg["provider"])).split(" ", 1)[0]
    ).hostname
    if not provider_host:
        raise ValueError(f"{platform} FFmpeg provider must be HTTPS")
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
    # The identity the URL must carry: the provider's own build name when it
    # prints one, otherwise the pinned build.
    identity = str(entry.get("reports", build))
    if "archive_url" in entry:
        pins = [("archive", entry.get("archive_url"), entry.get("archive_sha256"))]
        for binary in ("ffmpeg", "ffprobe"):
            member = Path(str(entry.get(f"{binary}_member", "")))
            if (
                not member.parts
                or member.is_absolute()
                or ".." in member.parts
                or not member.name.startswith(binary)
            ):
                raise ValueError(f"{platform} {binary} member escapes its archive")
    else:
        pins = [
            (binary, entry.get(f"{binary}_url"), entry.get(f"{binary}_sha256"))
            for binary in ("ffmpeg", "ffprobe")
        ]
    for label, url, digest in pins:
        parsed = urlparse(str(url))
        if parsed.scheme != "https" or parsed.hostname != provider_host:
            raise ValueError(f"{platform} {label} URL has an untrusted provider")
        if identity not in parsed.path:
            raise ValueError(f"{platform} {label} URL omits its build identity")
        if not isinstance(digest, str) or SHA256_PATTERN.fullmatch(digest) is None:
            raise ValueError(f"{platform} {label} digest is invalid")
    # A GPL build is shipped with a pointer to its complete source, which the
    # app's FFmpeg notice prints: the provider's scripts and library versions.
    source = entry.get("source")
    if (
        not isinstance(source, list)
        or not source
        or any(urlparse(str(url)).scheme != "https" for url in source)
    ):
        raise ValueError(f"{platform} FFmpeg names no HTTPS source for its build")


def _check_uv(bom: dict) -> None:
    """The installer a packaged app ships: one pinned release, every archive hashed."""
    uv = bom["uv"]
    version = str(uv.get("version"))
    if re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version) is None:
        raise ValueError("uv version is invalid")
    platforms = {key for key, value in uv.items() if isinstance(value, dict)}
    if platforms != UV_PLATFORMS:
        raise ValueError("uv must be pinned for macOS arm64, Linux amd64 and Windows amd64")
    for platform in sorted(platforms):
        entry = uv[platform]
        url = urlparse(str(entry.get("url")))
        if (
            url.scheme != "https"
            or url.hostname != "github.com"
            or not url.path.startswith(f"/astral-sh/uv/releases/download/{version}/")
        ):
            raise ValueError(f"{platform} uv is not a pinned upstream release")
        if SHA256_PATTERN.fullmatch(str(entry.get("sha256"))) is None:
            raise ValueError(f"{platform} uv digest is invalid")
        member = Path(str(entry.get("member", "")))
        if not member.parts or member.is_absolute() or ".." in member.parts:
            raise ValueError(f"{platform} uv member escapes its archive")
    python = str(bom["python"].get("version"))
    if re.fullmatch(r"3\.[0-9]+\.[0-9]+", python) is None:
        raise ValueError("the engine's Python version is invalid")


EMOJI_CODE_PATTERN = re.compile(r"^[0-9a-f]{4,5}(_[0-9a-f]{4,5})*$")


def _verify_emoji(bom: dict, emoji_dir: Path) -> None:
    """The emoji drawn over clips: a permitted licence, one pinned commit, digests."""
    emoji = bom["emoji"]
    if emoji.get("license") not in FONT_LICENSE_ALLOWLIST:
        raise ValueError(f"emoji license {emoji.get('license')!r} is not permitted")
    commit = str(emoji.get("commit"))
    if re.fullmatch(r"[0-9a-f]{40}", commit) is None:
        raise ValueError("emoji must be pinned to a commit")
    for key in ("base_url", "license_url", "notice_url"):
        url = urlparse(str(emoji[key]))
        if (
            url.scheme != "https"
            or url.hostname != "raw.githubusercontent.com"
            or not url.path.startswith(f"/googlefonts/noto-emoji/{commit}/")
        ):
            raise ValueError(f"emoji {key} is not a pinned upstream file")
    for key in ("license_sha256", "notice_sha256"):
        if SHA256_PATTERN.fullmatch(str(emoji.get(key))) is None:
            raise ValueError(f"emoji {key} is invalid")
    codes = [code for code in str(emoji["ids"]).split(",") if code]
    digests = emoji["sha256"]
    if sorted(codes) != sorted(digests) or len(codes) != len(set(codes)):
        raise ValueError("the emoji list and its digests disagree")
    for code in codes:
        if EMOJI_CODE_PATTERN.fullmatch(code) is None:
            raise ValueError(f"emoji {code!r} is not a code point name")
        if SHA256_PATTERN.fullmatch(str(digests[code])) is None:
            raise ValueError(f"emoji {code} digest is invalid")
        installed = emoji_dir / f"emoji_u{code}.png"
        if installed.is_symlink() or not installed.is_file():
            raise ValueError(f"pinned emoji is missing or unsafe: {installed}")
        if hashlib.sha256(installed.read_bytes()).hexdigest() != digests[code]:
            raise ValueError(f"installed emoji {code} does not match its pinned digest")
    if not (emoji_dir / "LICENSE.txt").is_file():
        raise ValueError("the emoji licence text was not installed beside them")


def _current_platform() -> str:
    system = host_platform.system()
    machine = host_platform.machine().casefold()
    if system == "Darwin" and machine == "arm64":
        return "macos-arm64"
    if system == "Linux" and machine in {"amd64", "x86_64"}:
        return "linux-amd64"
    if system == "Windows" and machine in {"amd64", "x86_64"}:
        return "windows-amd64"
    raise ValueError(f"unsupported BOM verification platform: {system}-{machine}")


def _verify_binary(name: str, path: Path, reports: str) -> None:
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
    if not first_line.startswith(f"{name} version {reports}"):
        raise ValueError(f"installed {name} does not match its pinned build {reports}")
    if "--enable-gpl" not in output or "--enable-version3" not in output:
        raise ValueError(f"installed {name} omitted its declared GPL/version3 license flags")
    # Nonfree builds cannot be redistributed, and every pinned build is one a
    # released app carries.
    if "--enable-nonfree" in output:
        raise ValueError(f"installed {name} is a nonfree build, which cannot be redistributed")
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
    _verify_faces(bom, fonts_dir)


# The upstream repositories the other caption faces are taken from, each at a
# pinned commit rather than a branch.
FACE_PROVIDERS = {
    "/google/fonts/",
    "/JulietaUla/Montserrat/",
}
COMMIT_PATTERN = re.compile(r"/[0-9a-f]{40}/")
FILE_PATTERN = re.compile(r"^[A-Za-z0-9-]+\.(ttf|otf|txt)$")


def _verify_faces(bom: dict, fonts_dir: Path) -> None:
    """The other caption faces: permitted licences, pinned commits and digests."""
    ids = [face for face in str(bom["fonts"]["faces"]["ids"]).split(",") if face]
    faces = bom["fonts"]["face"]
    if sorted(ids) != sorted(faces):
        raise ValueError("the caption face list and its entries disagree")
    for face_id in ids:
        face = faces[face_id]
        if face.get("license") not in FONT_LICENSE_ALLOWLIST:
            raise ValueError(
                f"caption face {face_id} license {face.get('license')!r} is not permitted"
            )
        for url_key in ("url", "license_url"):
            url = urlparse(str(face[url_key]))
            if (
                url.scheme != "https"
                or url.hostname != "raw.githubusercontent.com"
                or not any(url.path.startswith(provider) for provider in FACE_PROVIDERS)
                or COMMIT_PATTERN.search(url.path) is None
            ):
                raise ValueError(f"caption face {face_id} {url_key} is not a pinned upstream file")
        for key in ("sha256", "license_sha256"):
            if SHA256_PATTERN.fullmatch(str(face.get(key))) is None:
                raise ValueError(f"caption face {face_id} {key} is invalid")
        for key in ("file", "license_file"):
            if FILE_PATTERN.fullmatch(str(face.get(key))) is None:
                raise ValueError(f"caption face {face_id} {key} is not a plain file name")
        installed = fonts_dir / str(face["file"])
        if installed.is_symlink() or not installed.is_file():
            raise ValueError(f"pinned caption face is missing or unsafe: {installed}")
        if hashlib.sha256(installed.read_bytes()).hexdigest() != face["sha256"]:
            raise ValueError(f"installed caption face {installed.name} does not match its pin")
        if not (fonts_dir / str(face["license_file"])).is_file():
            raise ValueError(f"caption face {face_id} licence text was not installed beside it")


if __name__ == "__main__":
    raise SystemExit(main())
