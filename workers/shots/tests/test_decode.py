"""What the decoder is asked for, and what happens when it does not deliver.

The invocation is asserted rather than trusted because every part of it is in
the artifact key by implication: a stage that quietly stopped pinning the
scaler, or let the scaler pick the width, would keep producing valid documents
whose numbers no longer mean what earlier ones meant.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest
from clipmill_worker_shots.decode import (
    SCALER,
    AnalysisSize,
    DecodeFailed,
    analysis_size,
    decode_command,
    decode_frames,
)


def test_the_analysis_frame_keeps_the_aspect_and_stays_even() -> None:
    assert analysis_size(1920, 1080, 180) == AnalysisSize(width=320, height=180)
    assert analysis_size(1080, 1920, 180) == AnalysisSize(width=100, height=180)
    # 4:3 at 180 rounds to 240, which is already even.
    assert analysis_size(640, 480, 180) == AnalysisSize(width=240, height=180)
    # An odd requested height is brought down, not up: a taller frame than was
    # asked for is more work than was asked for.
    assert analysis_size(1920, 1080, 181) == AnalysisSize(width=320, height=180)


def test_an_odd_aspect_still_produces_an_even_width() -> None:
    for width in range(100, 140):
        size = analysis_size(width, 99, 64)
        assert size.width % 2 == 0
        assert size.height % 2 == 0
        assert size.width >= 2


def test_a_degenerate_size_is_refused() -> None:
    with pytest.raises(DecodeFailed):
        analysis_size(1, 1080, 180)
    with pytest.raises(DecodeFailed):
        analysis_size(1920, 1080, 8)


def test_the_invocation_pins_the_size_the_scaler_and_the_pixel_format() -> None:
    command = decode_command(Path("/pinned/ffmpeg"), Path("/cas/proxy.mp4"), AnalysisSize(320, 180))
    assert command[0] == "/pinned/ffmpeg"
    assert "-nostdin" in command
    # An explicit width and height, never `-2`: how a filter rounds must not be
    # able to change what the detector saw.
    assert f"scale=320:180:flags={SCALER}" in command
    assert command[command.index("-pix_fmt") + 1] == "bgr24"
    assert command[command.index("-f") + 1] == "rawvideo"
    assert command[-1] == "-"
    # Audio and subtitles are not decoded; that work would be discarded.
    for stream in ("-an", "-sn", "-dn"):
        assert stream in command


def fake_decoder(root: Path, body: str) -> Path:
    """A stand-in for FFmpeg, so the failure paths run without one."""

    path = root / "fake-ffmpeg"
    path.write_text(f"#!{sys.executable}\nimport sys\n{body}\n", encoding="utf-8")
    path.chmod(0o755)
    return path


def test_a_decoder_that_produces_nothing_is_a_refusal(tmp_path: Path) -> None:
    decoder = fake_decoder(tmp_path, "sys.exit(0)")
    with pytest.raises(DecodeFailed, match="no frames at all"):
        list(decode_frames(decoder, tmp_path / "proxy.mp4", AnalysisSize(4, 4)))


def test_a_decoder_that_fails_reports_what_it_said(tmp_path: Path) -> None:
    decoder = fake_decoder(
        tmp_path,
        "sys.stderr.write('moov atom not found\\n')\nsys.exit(1)",
    )
    with pytest.raises(DecodeFailed, match="moov atom not found"):
        list(decode_frames(decoder, tmp_path / "proxy.mp4", AnalysisSize(4, 4)))


def test_a_truncated_final_frame_is_not_a_frame(tmp_path: Path) -> None:
    """Half a frame is a tear, and a detector handed one would report it as a
    cut. 48 bytes is one 4x4 BGR frame; this writes one and a half."""

    decoder = fake_decoder(
        tmp_path,
        "sys.stdout.buffer.write(bytes(48 + 24))\nsys.exit(0)",
    )
    with pytest.raises(DecodeFailed, match="not the 48"):
        list(decode_frames(decoder, tmp_path / "proxy.mp4", AnalysisSize(4, 4)))


def test_whole_frames_arrive_with_the_shape_that_was_asked_for(tmp_path: Path) -> None:
    decoder = fake_decoder(
        tmp_path,
        "sys.stdout.buffer.write(bytes(range(48)) * 3)\nsys.exit(0)",
    )
    seen: list[int] = []
    frames = list(
        decode_frames(decoder, tmp_path / "proxy.mp4", AnalysisSize(4, 4), on_frame=seen.append)
    )
    assert len(frames) == 3
    assert all(frame.shape == (4, 4, 3) for frame in frames)
    assert seen == [1, 2, 3]


def test_abandoning_the_stream_does_not_leave_the_decoder_running(tmp_path: Path) -> None:
    """What a cancelled task does. The generator is closed part-way through,
    and the child must not outlive it."""

    decoder = fake_decoder(
        tmp_path,
        "import time\n"
        "sys.stdout.buffer.write(bytes(48))\n"
        "sys.stdout.buffer.flush()\n"
        "time.sleep(120)\n",
    )
    stream = decode_frames(decoder, tmp_path / "proxy.mp4", AnalysisSize(4, 4))
    assert next(stream).shape == (4, 4, 3)
    stream.close()
    # Nothing is left behind that would keep the worker's process group alive.
    remaining = subprocess.run(
        ["pgrep", "-f", "fake-ffmpeg"],
        capture_output=True,
        check=False,
    )
    assert remaining.returncode != 0 or not remaining.stdout.strip()
