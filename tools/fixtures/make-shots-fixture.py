#!/usr/bin/env python3
"""Build a shots fixture whose cuts are known exactly.

A cut gate needs ground truth, and ground truth normally means somebody scrubbed
a timeline and wrote down frame numbers. This sidesteps that the same way the
speech fixture does: the frames are generated here, so the truth is not measured
at all — it is the arithmetic that laid the scenes down.

Two properties are deliberate rather than incidental.

    Every scene moves. A fixture of perfectly static shots would pass a
    detector that reported any change at all, which is not the thing being
    tested. Each scene drifts a few pixels per frame, so the gate proves the
    threshold discriminates between a camera moving and a camera changing.

    The encode matches the proxy's. Same codec, rate, GOP, and pixel format the
    ingest fan-out pins, because the stage under test reads a proxy and a
    fixture encoded some other way would be measuring a different decode.

    tools/fixtures/make-shots-fixture.py <output-dir>

Writes shots.mp4 and truth.json.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

TICKS_PER_SECOND = 90_000
RATE_NUM = 30_000
RATE_DEN = 1_001
WIDTH = 640
HEIGHT = 360

# Four scenes, alternating saturated colour with near-neutral grey so that every
# adjacent pair differs in hue, saturation, and brightness at once. Two
# saturated scenes side by side differ in hue alone, which is a third of what
# the content distance measures, and a fixture built that way sits close enough
# to the threshold that a different encoder build could move it across.
#
# Measured on the pinned FFmpeg at the default 27.0 threshold: the smallest cut
# scores about 97 and the largest within-scene change about 8. The gate is
# therefore centred rather than merely passing — it would survive a threshold
# anywhere between roughly 10 and 90.
SCENES = (
    # (base BGR, bar BGR, frames)
    ((32, 30, 30), (205, 195, 150), 45),
    ((195, 50, 25), (35, 235, 245), 45),
    ((216, 214, 212), (28, 26, 30), 45),
    ((40, 155, 45), (210, 60, 200), 45),
)
# How far the bar travels per frame. 24 pixels at 30 fps is faster than a screen
# width per second, so the fixture is not merely four still images: the gate's
# claim is that a fast pan is not a cut, which needs a pan in it.
DRIFT_PIXELS = 24
BAR_WIDTH = 96


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--ffmpeg",
        default="ffmpeg",
        help="path to the pinned ffmpeg (defaults to whatever is on PATH)",
    )
    parser.add_argument(
        "--repeat",
        type=int,
        default=1,
        help="how many times to lay the four scenes down, for a longer fixture",
    )
    options = parser.parse_args()
    if options.repeat < 1:
        print("make-shots-fixture: --repeat must be at least 1", file=sys.stderr)
        return 2

    options.output.mkdir(parents=True, exist_ok=True)
    video = options.output / "shots.mp4"
    scenes = [scene for _ in range(options.repeat) for scene in SCENES]

    cuts: list[int] = []
    frame_index = 0
    encoder = subprocess.Popen(
        _encode_command(options.ffmpeg, video),
        stdin=subprocess.PIPE,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    if encoder.stdin is None:
        print("make-shots-fixture: the encoder took no input", file=sys.stderr)
        return 1
    try:
        for position, (base, bar, length) in enumerate(scenes):
            if position > 0:
                # The first frame of a new scene is the cut. Recorded here
                # rather than derived from the scene lengths afterwards, so the
                # truth and the frames come from the same loop.
                cuts.append(frame_index)
            for offset in range(length):
                encoder.stdin.write(_frame(base, bar, offset))
                frame_index += 1
        encoder.stdin.close()
    except BrokenPipeError:
        pass
    status = encoder.wait()
    if status != 0:
        detail = b"" if encoder.stderr is None else encoder.stderr.read()
        print(
            f"make-shots-fixture: the encoder refused the frames: "
            f"{detail.decode('utf-8', 'replace').strip() or status}",
            file=sys.stderr,
        )
        return 1

    truth = {
        "frame_count": frame_index,
        "frame_rate": {"num": RATE_NUM, "den": RATE_DEN},
        "width": WIDTH,
        "height": HEIGHT,
        "duration_ticks": _ticks(frame_index),
        "cuts": [{"frame": frame, "t_ticks": _ticks(frame)} for frame in cuts],
        "shots": [
            {
                "start_frame": start,
                "end_frame": end,
                "start_ticks": _ticks(start),
                "end_ticks": _ticks(end),
            }
            for start, end in zip([0, *cuts], [*cuts, frame_index], strict=True)
        ],
    }
    (options.output / "truth.json").write_text(
        json.dumps(truth, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(
        f"make-shots-fixture: {video} "
        f"({frame_index} frames, {frame_index * RATE_DEN / RATE_NUM:.2f}s, "
        f"{len(cuts)} cuts in {len(scenes)} scenes)"
    )
    return 0


def _frame(base: tuple[int, int, int], bar: tuple[int, int, int], offset: int) -> bytes:
    """One BGR24 frame: a flat scene colour with a bar drifting across it.

    No array library. Every row is identical, so one row is built and repeated —
    which keeps this script runnable by whatever `python3` a machine has, the
    same property the speech fixture generator relies on.
    """

    row = bytearray(bytes(base) * WIDTH)
    start = (offset * DRIFT_PIXELS) % WIDTH
    for column in range(start, start + BAR_WIDTH):
        at = (column % WIDTH) * 3
        row[at : at + 3] = bytes(bar)
    return bytes(row) * HEIGHT


def _encode_command(ffmpeg: str, output: Path) -> list[str]:
    """The ingest proxy's own encode, minus the audio it has no source for."""

    return [
        ffmpeg,
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "bgr24",
        "-s",
        f"{WIDTH}x{HEIGHT}",
        "-r",
        f"{RATE_NUM}/{RATE_DEN}",
        "-i",
        "-",
        "-fps_mode",
        "cfr",
        "-r",
        f"{RATE_NUM}/{RATE_DEN}",
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "23",
        "-pix_fmt",
        "yuv420p",
        "-g",
        "30",
        "-keyint_min",
        "30",
        # A fixed GOP, as the proxy pins: an encoder that inserted its own
        # keyframes at the scene changes would make the container agree with
        # the truth for reasons that have nothing to do with the detector.
        "-sc_threshold",
        "0",
        "-movflags",
        "+faststart",
        "-f",
        "mp4",
        str(output),
    ]


def _ticks(frame: int) -> int:
    return frame * RATE_DEN * TICKS_PER_SECOND // RATE_NUM


if __name__ == "__main__":
    raise SystemExit(main())
