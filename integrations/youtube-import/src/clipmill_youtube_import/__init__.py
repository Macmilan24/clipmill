"""One explicit public YouTube video -> a private, local source file.

The daemon owns consent, durable state and cancellation. This process owns no
database and emits only the small JSONL protocol, never yt-dlp's raw logs or
signed media URLs. The Python API does not read CLI configuration files.
"""

from __future__ import annotations

import argparse
import fcntl
import importlib.metadata
import json
import math
import os
import re
import shutil
import signal
import stat
import subprocess
import threading
import time
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

MAX_BYTES = 8 * 1024**3
RESERVE_BYTES = 1024**3
MAX_SECONDS = 6 * 60 * 60
VIDEO_ID = re.compile(r"[A-Za-z0-9_-]{11}\Z")


class ImportFailure(Exception):
    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code


def emit(event: str, **fields: object) -> None:
    print(json.dumps({"event": event, **fields}, ensure_ascii=True, allow_nan=False), flush=True)


def canonical_url(value: str) -> tuple[str, str]:
    """Reduce share/watch/shorts/embed links to one video; strip tracking/playlist data."""
    value = value.strip()
    if len(value) > 2048 or any(ord(c) < 32 for c in value):
        raise ImportFailure("invalid_url", "Enter a single YouTube video link.")
    try:
        parsed = urlsplit(value)
        if (
            parsed.scheme != "https"
            or parsed.username is not None
            or parsed.password is not None
            or parsed.port not in (None, 443)
        ):
            raise ValueError
        if parsed.hostname == "youtu.be":
            video_id = parsed.path.removeprefix("/")
        elif parsed.hostname in {"youtube.com", "www.youtube.com", "m.youtube.com"}:
            if parsed.path == "/watch":
                ids = parse_qs(parsed.query).get("v", [])
                video_id = ids[0] if len(ids) == 1 else ""
            else:
                parts = parsed.path.split("/")
                video_id = (
                    parts[2] if len(parts) == 3 and parts[1] in {"shorts", "embed", "live"} else ""
                )
        else:
            raise ValueError
        if not VIDEO_ID.fullmatch(video_id):
            raise ValueError
    except ValueError:
        raise ImportFailure("invalid_url", "Enter a single HTTPS YouTube video link.") from None
    return f"https://www.youtube.com/watch?v={video_id}", video_id


def safe_text(value: object, limit: int) -> str:
    return "".join(c for c in str(value or "") if c.isprintable())[:limit].strip()


def inspect_metadata(info: dict, expected_id: str) -> dict:
    duration = info.get("duration")
    if info.get("_type", "video") != "video" or info.get("id") != expected_id:
        raise ImportFailure("invalid_video", "The link did not resolve to the requested video.")
    if info.get("is_live") or info.get("live_status") in {"is_live", "is_upcoming", "post_live"}:
        raise ImportFailure("live_video", "Wait until this broadcast is finished and processed.")
    if not isinstance(duration, (float, int)) or not math.isfinite(duration) or duration <= 0:
        raise ImportFailure("invalid_duration", "YouTube did not provide a usable video duration.")
    if duration > MAX_SECONDS:
        raise ImportFailure("duration_limit", "This version imports videos up to six hours long.")
    if info.get("has_drm"):
        raise ImportFailure("protected_video", "This video is protected and cannot be imported.")
    formats = info.get("requested_formats") or [info]
    estimate = sum(f.get("filesize") or f.get("filesize_approx") or 0 for f in formats)
    if estimate > MAX_BYTES:
        raise ImportFailure("size_limit", "The selected video exceeds the 8 GB download limit.")
    return {
        "video_id": expected_id,
        "title": safe_text(info.get("title"), 300) or "YouTube video",
        "channel": safe_text(info.get("channel") or info.get("uploader"), 200),
        "duration_seconds": float(duration),
    }


def check_runtime(ffmpeg: str, node: str | None) -> str:
    for name, version in (("yt-dlp", "2026.8.19"), ("yt-dlp-ejs", "0.8.0")):
        if importlib.metadata.version(name) != version:
            raise ImportFailure("setup_required", "Run just setup-youtube to repair the importer.")
    ffmpeg_path = Path(ffmpeg)
    if not ffmpeg_path.is_absolute() or not ffmpeg_path.is_file() or not os.access(ffmpeg, os.X_OK):
        raise ImportFailure(
            "setup_required", "The configured FFmpeg is unavailable. Run just setup."
        )
    node = node or os.environ.get("CLIPMILL_NODE") or shutil.which("node")
    if not node:
        raise ImportFailure("setup_required", "YouTube import requires Node.js 22 or newer.")
    node = str(Path(node).resolve())
    try:
        result = subprocess.run(
            [node, "--version"], capture_output=True, timeout=5, check=True, text=True
        )
        major = int(result.stdout.strip().removeprefix("v").split(".")[0])
        if major < 22:
            raise ValueError
    except (OSError, ValueError, subprocess.SubprocessError):
        raise ImportFailure(
            "setup_required", "YouTube import requires Node.js 22 or newer."
        ) from None
    return node


def prepare_destination(value: str) -> Path:
    path = Path(value)
    if not path.is_absolute() or path.is_symlink():
        raise ImportFailure("storage_error", "The download destination is not a private directory.")
    path.mkdir(mode=0o700, parents=True, exist_ok=True)
    if not path.is_dir() or any(path.iterdir()):
        raise ImportFailure("storage_error", "A fresh download directory is required.")
    path.chmod(0o700)
    return path.resolve()


class Progress:
    def __init__(self, directory: Path):
        self.directory = directory
        self.bytes_by_file: dict[str, int] = {}
        self.totals_by_file: dict[str, int] = {}
        self.last_emit = 0.0
        self.expected_total: int | None = None

    def __call__(self, data: dict) -> None:
        name = data.get("filename", "media")
        downloaded = max(0, int(data.get("downloaded_bytes") or 0))
        self.bytes_by_file[name] = downloaded
        total = data.get("total_bytes")  # Estimates are not a measured total.
        if isinstance(total, int) and total > 0:
            self.totals_by_file[name] = total
        cumulative = sum(self.bytes_by_file.values())
        if cumulative > MAX_BYTES:
            raise ImportFailure("size_limit", "The video exceeded the 8 GB download limit.")
        if shutil.disk_usage(self.directory).free < RESERVE_BYTES:
            raise ImportFailure("disk_full", "Free some disk space, then retry this import.")
        now = time.monotonic()
        if now - self.last_emit >= 0.25 or data.get("status") == "finished":
            self.last_emit = now
            # A stream's total is not the whole import. Only advertise a total
            # when extraction provided exact sizes for every selected stream.
            total = self.expected_total
            emit(
                "progress",
                downloaded_bytes=cumulative,
                **({"total_bytes": total} if total and total >= cumulative else {}),
            )


class QuietLogger:
    def debug(self, _message):
        pass

    info = debug
    warning = debug
    error = debug


def options(
    directory: Path, ffmpeg: str, node: str, progress: Progress, max_height: int = 1080
) -> dict:
    if max_height not in (360, 720, 1080):
        raise ImportFailure("invalid_quality", "Choose 360p, 720p or 1080p for the download.")
    return {
        "quiet": True,
        "no_warnings": True,
        "logger": QuietLogger(),
        "noplaylist": True,
        "cachedir": False,
        "proxy": "",  # Do not inherit a user's proxy environment.
        "cookiefile": None,
        "cookiesfrombrowser": None,
        "usenetrc": False,
        "enable_file_urls": False,
        "allow_unplayable_formats": False,
        "remote_components": [],
        "js_runtimes": {"node": {"path": node}},
        "allowed_extractors": ["youtube"],
        "outtmpl": str(directory / "source.%(ext)s"),
        "paths": {"home": str(directory), "temp": str(directory)},
        # Direct HTTPS formats keep manifests from launching an unbounded live
        # capture. No upscaling or transcoding; the core makes its normal proxy.
        "format": (
            f"bv[height<={max_height}][protocol=https]+ba[protocol=https]/"
            f"b[height<={max_height}][protocol=https]"
        ),
        "merge_output_format": "mkv",
        "postprocessors": [{"key": "FFmpegVideoRemuxer", "preferedformat": "mkv"}],
        "ffmpeg_location": ffmpeg,
        "socket_timeout": 20,
        "retries": 2,
        "fragment_retries": 2,
        "skip_unavailable_fragments": False,
        "extractor_retries": 2,
        "concurrent_fragment_downloads": 1,
        "max_filesize": MAX_BYTES,
        "overwrites": False,
        "continuedl": False,
        "nopart": False,
        "writethumbnail": False,
        "writeinfojson": False,
        "writesubtitles": False,
        "writeautomaticsub": False,
        "progress_hooks": [progress],
        "postprocessor_hooks": [lambda _: emit("processing")],
    }


def download(url: str, destination: str, ffmpeg: str, node: str, max_height: int = 1080) -> None:
    # Disable plugins BEFORE constructing YoutubeDL. A normal Python API
    # invocation otherwise loads arbitrary user extractor/postprocessor plugins.
    from yt_dlp import YoutubeDL
    from yt_dlp.globals import plugin_dirs

    plugin_dirs.value = []
    url, video_id = canonical_url(url)
    directory = prepare_destination(destination)
    # Cleanup/recovery can test this advisory lock without guessing whether an
    # orphan is still using the directory. Keep the handle until process exit.
    lock = (directory / ".import.lock").open("xb")
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    try:
        download_into(
            ydl_class=YoutubeDL,
            directory=directory,
            url=url,
            video_id=video_id,
            ffmpeg=ffmpeg,
            node=node,
            max_height=max_height,
        )
    finally:
        lock.close()


def download_into(
    *,
    ydl_class,
    directory: Path,
    url: str,
    video_id: str,
    ffmpeg: str,
    node: str,
    max_height: int = 1080,
) -> None:
    if shutil.disk_usage(directory).free < 2 * RESERVE_BYTES:
        raise ImportFailure("disk_full", "Free at least 2 GB of disk space, then retry.")
    monitor_storage(directory)
    progress = Progress(directory)
    with ydl_class(options(directory, ffmpeg, node, progress, max_height)) as ydl:
        info = ydl.extract_info(url, download=False)
        if not isinstance(info, dict):
            raise ImportFailure("unavailable", "This YouTube video is unavailable.")
        metadata = inspect_metadata(info, video_id)
        formats = info.get("requested_formats") or [info]
        if all(isinstance(f.get("filesize"), int) and f["filesize"] > 0 for f in formats):
            progress.expected_total = sum(f["filesize"] for f in formats)
        estimate = sum(f.get("filesize") or f.get("filesize_approx") or 0 for f in formats)
        if shutil.disk_usage(directory).free < 2 * estimate + RESERVE_BYTES:
            raise ImportFailure(
                "disk_full", "There is not enough disk space to download and merge this video."
            )
        emit("metadata", **metadata)
        # Use the already selected/validated metadata; never perform a second
        # URL extraction which could change the chosen video or selected size.
        ydl.process_info(info)
    final = directory / "source.mkv"
    if final.is_symlink() or not final.is_file():
        raise ImportFailure("download_failed", "YouTube did not return a complete playable video.")
    size = final.stat().st_size
    if not 0 < size <= MAX_BYTES:
        raise ImportFailure(
            "size_limit", "The downloaded video is empty or exceeds the 8 GB limit."
        )
    if not stat.S_ISREG(final.stat().st_mode):
        raise ImportFailure("storage_error", "The downloaded source is not a regular file.")
    final.chmod(0o600)
    with final.open("rb") as media:
        os.fsync(media.fileno())
    directory_fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory_fd)
    finally:
        os.close(directory_fd)
    emit("complete", file_name="source.mkv", byte_size=size, **metadata)


def storage_failure(directory: Path) -> tuple[str, str] | None:
    # The only expected entries are two streams, a merged file and .part files.
    # Do not recurse/follow links or sum arbitrary external paths.
    entries = list(directory.iterdir())
    if len(entries) > 32 or any(p.is_symlink() or not p.is_file() for p in entries):
        return "storage_error", "The import directory contains unexpected files. Retry the import."
    if sum(p.stat().st_size for p in entries) > 2 * MAX_BYTES + RESERVE_BYTES:
        return "size_limit", "The download exceeded its temporary storage limit."
    if shutil.disk_usage(directory).free < RESERVE_BYTES:
        return "disk_full", "Free some disk space, then retry this import."
    return None


def monitor_storage(directory: Path) -> None:
    def watch():
        while True:
            time.sleep(0.25)
            failure = checked_storage_failure(directory)
            if failure:
                emit("error", code=failure[0], message=failure[1])
                if os.getpgrp() == os.getpid():
                    os.killpg(os.getpid(), signal.SIGKILL)
                os._exit(1)

    threading.Thread(target=watch, daemon=True).start()


def checked_storage_failure(directory: Path) -> tuple[str, str] | None:
    try:
        return storage_failure(directory)
    except FileNotFoundError:  # A stream was renamed by FFmpeg.
        return None
    except OSError:
        # An unreadable volume must not silently disable the watchdog thread.
        return "storage_error", "The download storage could not be checked. Retry the import."


def watch_parent() -> None:
    """A killed daemon must not leave a downloader/FFmpeg/Node group running."""
    parent = os.getppid()
    group_leader = os.getpgrp() == os.getpid()

    def watch():
        while True:
            time.sleep(0.5)
            if os.getppid() != parent or parent == 1:
                if group_leader:
                    os.killpg(os.getpid(), signal.SIGKILL)
                os._exit(1)

    threading.Thread(target=watch, daemon=True).start()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--url")
    parser.add_argument("--destination")
    parser.add_argument("--ffmpeg", required=True)
    parser.add_argument("--node")
    parser.add_argument("--max-height", type=int, choices=(360, 720, 1080), default=1080)
    args = parser.parse_args()
    os.umask(0o077)
    watch_parent()
    try:
        node = check_runtime(args.ffmpeg, args.node)
        if args.check:
            emit("ready")
            return 0
        if not args.url or not args.destination:
            raise ImportFailure(
                "invalid_request", "A video link and download destination are required."
            )
        download(args.url, args.destination, args.ffmpeg, node, args.max_height)
        return 0
    except ImportFailure as error:
        emit("error", code=error.code, message=str(error))
    except Exception as error:
        # The provider can include signed URLs, paths and response bodies in
        # exceptions. Only translate recognised failure categories, never echo.
        message = str(error).lower()
        if any(
            s in message for s in ("sign in", "private video", "members-only", "age-restricted")
        ):
            emit(
                "error",
                code="restricted_video",
                message=(
                    "This video needs sign-in or has access restrictions. "
                    "Ask the creator for a downloadable file."
                ),
            )
        elif any(s in message for s in ("not available", "unavailable", "removed")):
            emit(
                "error",
                code="unavailable",
                message=(
                    "This video is unavailable from your location. "
                    "Check the link or ask the creator for a file."
                ),
            )
        elif "no space" in message:
            emit("error", code="disk_full", message="Free some disk space, then retry this import.")
        else:
            emit(
                "error",
                code="download_failed",
                message=(
                    "YouTube could not complete the download. Check your connection and retry; "
                    "if it persists, update the importer or use a creator-provided file."
                ),
            )
    return 1
