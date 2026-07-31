"""Reading the sampled frames ingest already published, back into pixels.

Ingest decoded the source once and wrote a frame every quarter second (book
ch. 12). This stage reads those rather than decoding the proxy again, which is
what stops two visual surfaces disagreeing about what was on screen at a moment
— and it is why this worker never sees a user's file.

The JPEGs are turned back into arrays by **the decoder the daemon named on the
lease**, not by an imaging library this package chose. Two JPEG decoders differ
in the last bit of a chroma-upsampled pixel, and a face score sits on the far
side of a threshold often enough for that to matter to a document addressed by
content.

The frame is letterboxed rather than stretched to the model's square input:
YuNet is trained on faces with faces' proportions, and a 16:9 frame squeezed
into a square is a room full of tall thin people. The padding is anchored at the
top-left rather than centred so that mapping a box back is a division and not a
division plus an offset that has to agree with a filter's rounding.

The pixels come out **blue first**, which is not a preference. YuNet was trained
through OpenCV, whose images are BGR, so that is the channel order the weights
learned faces in. Handing it the other order is not a crash and not obviously
wrong — most faces are still found — but it costs score on the marginal ones,
and the marginal ones are exactly what the focus gate downstream is deciding
about.
"""

from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from pathlib import Path

import numpy as np
from clipmill_worker_sdk.artifacts import ArtifactVerificationError, VerifiedArtifact

FRAMES_DESCRIPTOR = "index.json"
#: Named rather than defaulted. Bicubic is FFmpeg's current default and a
#: reasonable one; writing it down means a future default cannot silently change
#: what this stage observed.
SCALER = "bicubic"
#: Blue first, because that is the order YuNet was trained in. See the module
#: docstring: this is a property of the weights, not a taste in formats.
PIXEL_FORMAT = "bgr24"
DECODE_TIMEOUT_SECONDS = 120.0


@dataclass(frozen=True, slots=True)
class SampledFrame:
    file: str
    t_ticks: int


@dataclass(frozen=True, slots=True)
class Frames:
    """The frames artifact's descriptor, read for what it states about itself."""

    artifact_id: str
    frames: tuple[SampledFrame, ...]
    frame_height: int
    rate_num: int
    rate_den: int
    source_fingerprint: str
    coverage_start_ticks: int
    coverage_end_ticks: int


class DecodeFailed(RuntimeError):
    """The frames would not decode, which they will not next time either."""


def read_frames(artifact: VerifiedArtifact, descriptor_path: Path) -> Frames:
    """Read a verified frames descriptor, refusing anything that is not one."""

    descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
    if descriptor.get("schema_version") != "clipmill.media.frames.v1":
        raise ArtifactVerificationError("input artifact is not a sampled frame set")
    rate = descriptor.get("frame_rate")
    if not isinstance(rate, dict) or not rate.get("num") or not rate.get("den"):
        raise ArtifactVerificationError("frames descriptor states no frame rate")
    listed = descriptor.get("frames")
    if not isinstance(listed, list):
        raise ArtifactVerificationError("frames descriptor names no frames")
    coverage = descriptor.get("coverage") or {}
    return Frames(
        artifact_id=artifact.artifact_id,
        frames=tuple(
            SampledFrame(file=str(entry["file"]), t_ticks=int(entry["t_ticks"])) for entry in listed
        ),
        frame_height=int(descriptor.get("frame_height", 0)),
        rate_num=int(rate["num"]),
        rate_den=int(rate["den"]),
        source_fingerprint=str(descriptor.get("source_fingerprint", "")),
        coverage_start_ticks=int(coverage.get("start_ticks", 0)),
        coverage_end_ticks=int(coverage.get("end_ticks", 0)),
    )


def jpeg_size(path: Path) -> tuple[int, int]:
    """Width and height from a JPEG's start-of-frame marker.

    Read here rather than asked of the decoder because the answer is needed
    *before* decoding, to compute the letterbox — and because a JPEG header is
    an exact, twenty-line parse, whereas asking a subprocess is a second process
    and a second thing that can fail.
    """

    data = path.read_bytes()
    if len(data) < 4 or data[0] != 0xFF or data[1] != 0xD8:
        raise DecodeFailed(f"{path.name} is not a JPEG")
    offset = 2
    while offset + 9 < len(data):
        if data[offset] != 0xFF:
            offset += 1
            continue
        marker = data[offset + 1]
        # Every SOFn except the four that are not frame headers.
        if 0xC0 <= marker <= 0xCF and marker not in (0xC4, 0xC8, 0xCC):
            height = int.from_bytes(data[offset + 5 : offset + 7], "big")
            width = int.from_bytes(data[offset + 7 : offset + 9], "big")
            if width == 0 or height == 0:
                raise DecodeFailed(f"{path.name} declares a zero dimension")
            return width, height
        if marker in (0xD8, 0x01) or 0xD0 <= marker <= 0xD7:
            offset += 2
            continue
        segment = int.from_bytes(data[offset + 2 : offset + 4], "big")
        if segment < 2:
            raise DecodeFailed(f"{path.name} has a malformed segment")
        offset += 2 + segment
    raise DecodeFailed(f"{path.name} has no start-of-frame marker")


@dataclass(frozen=True, slots=True)
class Letterbox:
    """How a frame was placed inside the model's square input."""

    width: int
    height: int
    size: int

    def to_normalized(
        self, x: float, y: float, w: float, h: float
    ) -> tuple[float, float, float, float]:
        """A box in input pixels, as a share of the original frame.

        Clamped, because a detector may place a box a pixel off the edge and a
        contract that says `[0,1]` should not be argued with over a pixel.
        """

        left = min(max(x / self.width, 0.0), 1.0)
        top = min(max(y / self.height, 0.0), 1.0)
        right = min(max((x + w) / self.width, 0.0), 1.0)
        bottom = min(max((y + h) / self.height, 0.0), 1.0)
        return left, top, max(right - left, 1e-6), max(bottom - top, 1e-6)


def letterbox_for(width: int, height: int, size: int) -> Letterbox:
    """Where a frame of this shape lands inside a square of this size."""

    if width <= 0 or height <= 0:
        raise DecodeFailed("a frame with no extent cannot be letterboxed")
    scale = min(size / width, size / height)
    return Letterbox(width=round(width * scale), height=round(height * scale), size=size)


def decode_frames(
    decoder: Path,
    paths: list[Path],
    box: Letterbox,
) -> list[np.ndarray]:
    """Every frame, letterboxed and blue-first, in one decoder process.

    One process rather than one per frame: a ten-minute recording is two and a
    half thousand JPEGs, and two and a half thousand process spawns is most of
    the stage's wall clock. The images go in back to back on a pipe, which is
    what `image2pipe` is for.
    """

    if not paths:
        return []
    payload = b"".join(path.read_bytes() for path in paths)
    filters = (
        f"scale={box.width}:{box.height}:flags={SCALER},pad={box.size}:{box.size}:0:0:color=black"
    )
    command = [
        str(decoder),
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-f",
        "image2pipe",
        "-i",
        "-",
        "-vf",
        filters,
        "-pix_fmt",
        PIXEL_FORMAT,
        "-f",
        "rawvideo",
        "-",
    ]
    try:
        finished = subprocess.run(
            command,
            input=payload,
            capture_output=True,
            timeout=DECODE_TIMEOUT_SECONDS,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise DecodeFailed(f"the pinned decoder did not run: {error}") from error
    if finished.returncode != 0:
        detail = finished.stderr.decode("utf-8", "replace").strip().splitlines()
        reason = detail[-1] if detail else "no reason given"
        raise DecodeFailed(f"the pinned decoder refused these frames: {reason}")

    stride = box.size * box.size * 3
    if stride == 0 or len(finished.stdout) % stride != 0:
        raise DecodeFailed("the decoder produced a partial frame")
    count = len(finished.stdout) // stride
    if count != len(paths):
        # A short read here would silently shift every timestamp after it, which
        # is the one failure a face track cannot recover from.
        raise DecodeFailed(f"asked for {len(paths)} frames and got {count}")
    buffer = np.frombuffer(finished.stdout, dtype=np.uint8)
    return [
        buffer[index * stride : (index + 1) * stride].reshape(box.size, box.size, 3)
        for index in range(count)
    ]
