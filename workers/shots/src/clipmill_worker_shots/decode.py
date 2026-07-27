"""Turning the proxy back into pixels, through the decoder the daemon pinned.

This is the only impure half of the stage, and it is kept apart from the
detection for that reason: everything in `content` is arithmetic over arrays
and can be tested against frames written by hand, while everything here is a
subprocess and a pipe.

Three things are pinned rather than defaulted, because all three change the
numbers the detector produces:

    the binary      the daemon names it on the lease; two FFmpeg builds decode
                    and scale differently, and the build identity is part of
                    the artifact key
    the size        stated as an exact width and height rather than letting the
                    scaler infer one from `-2`, so the frame the detector sees
                    does not depend on how a filter rounds
    the scaler      named explicitly; the default is a good choice, but a
                    default is not a decision and a later FFmpeg may change it

Frames arrive as raw BGR24 on a pipe, which is what OpenCV's colour conversion
expects and therefore what the detector expects. Nothing is buffered beyond one
frame: an hour of 320x180 video is fifteen gigabytes of raw pixels, and the
detector only ever compares a frame with the one before it.
"""

from __future__ import annotations

import subprocess
import tempfile
from collections.abc import Callable, Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np

# Named rather than defaulted. Bicubic is FFmpeg's current default and a
# reasonable one; writing it down means a future default cannot silently change
# what this stage observed.
SCALER = "bicubic"
# How long the decoder gets to exit after closing its output. It has already
# finished writing by then; anything longer than this is a process that is not
# going to leave on its own.
DRAIN_SECONDS = 30.0


class DecodeFailed(RuntimeError):
    """The proxy could not be decoded into frames."""


@dataclass(frozen=True, slots=True)
class AnalysisSize:
    width: int
    height: int


def analysis_size(source_width: int, source_height: int, target_height: int) -> AnalysisSize:
    """The exact frame size the detector will see.

    Computed here rather than delegated to the scaler's `-2` so that the number
    is ours and is stated. Width follows the display aspect and is rounded down
    to an even number, which every 4:2:0 pixel format requires.
    """

    if source_width < 2 or source_height < 2:
        raise DecodeFailed("the proxy declares a frame smaller than two pixels")
    if target_height < 16:
        raise DecodeFailed("an analysis height below sixteen pixels measures noise")
    height = target_height - (target_height % 2)
    width = round(source_width * height / source_height)
    width -= width % 2
    return AnalysisSize(width=max(2, width), height=max(2, height))


def decode_command(ffmpeg: Path, proxy: Path, size: AnalysisSize) -> list[str]:
    """The exact invocation, as a fixed argument list with no shell."""

    return [
        str(ffmpeg),
        "-nostdin",
        "-hide_banner",
        "-loglevel",
        "error",
        "-i",
        str(proxy),
        # Video only. A proxy carries audio, and decoding it would be work
        # whose result is thrown away.
        "-an",
        "-sn",
        "-dn",
        "-vf",
        f"scale={size.width}:{size.height}:flags={SCALER}",
        "-pix_fmt",
        "bgr24",
        "-f",
        "rawvideo",
        "-",
    ]


def decode_frames(
    ffmpeg: Path,
    proxy: Path,
    size: AnalysisSize,
    on_frame: Callable[[int], None] | None = None,
) -> Iterator[np.ndarray]:
    """Yield every frame of the proxy as a BGR24 array of exactly `size`.

    The child gets no stdin, so a decoder that decides to ask a question cannot
    wait forever for an answer nobody is going to type. Its diagnostics go to a
    file rather than a second pipe, because a pipe nobody is draining while the
    frames are being read is a pipe that can fill and stop the decoder.
    """

    frame_bytes = size.width * size.height * 3
    with tempfile.TemporaryFile() as diagnostics:
        child = subprocess.Popen(
            decode_command(ffmpeg, proxy, size),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=diagnostics,
        )
        index = 0
        try:
            stdout = child.stdout
            if stdout is None:
                raise DecodeFailed("the decoder was started without an output pipe")
            while True:
                raw = stdout.read(frame_bytes)
                if not raw:
                    break
                if len(raw) != frame_bytes:
                    # A short final read is a truncated frame, not a frame. A
                    # detector handed a half-decoded image would report the
                    # tear as a cut.
                    raise DecodeFailed(
                        f"the decoder produced {len(raw)} bytes for frame {index}, "
                        f"not the {frame_bytes} a {size.width}x{size.height} frame needs"
                    )
                yield np.frombuffer(raw, dtype=np.uint8).reshape(size.height, size.width, 3)
                index += 1
                if on_frame is not None:
                    on_frame(index)
        except BaseException:
            # Includes the GeneratorExit a cancelled task raises here: whoever
            # stops reading also stops the decoder.
            child.kill()
            child.wait()
            raise
        finally:
            if child.stdout is not None:
                child.stdout.close()

        try:
            status = child.wait(timeout=DRAIN_SECONDS)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()
            raise DecodeFailed("the decoder did not exit after closing its output") from None
        if status != 0:
            diagnostics.seek(0)
            detail = diagnostics.read().decode("utf-8", "replace").strip()
            raise DecodeFailed(f"the decoder refused the proxy: {detail or f'exit {status}'}")
        if index == 0:
            raise DecodeFailed("the proxy decoded to no frames at all")


__all__ = [
    "DRAIN_SECONDS",
    "SCALER",
    "AnalysisSize",
    "DecodeFailed",
    "analysis_size",
    "decode_command",
    "decode_frames",
]
