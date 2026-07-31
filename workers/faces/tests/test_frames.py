"""Reading frames: the header parse, the letterbox, and the decoder.

The header parse is exact arithmetic over bytes and is tested against a JPEG the
pinned encoder wrote, because a parser tested only against its own output is a
parser tested against nothing.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

import pytest
from clipmill_worker_faces.frames import (
    DecodeFailed,
    decode_frames,
    jpeg_size,
    letterbox_for,
)

REPOSITORY = Path(__file__).resolve().parents[3]
FFMPEG = REPOSITORY / ".cache" / "bin" / "ffmpeg"


def _write_jpeg(path: Path, width: int, height: int) -> None:
    if not FFMPEG.is_file():
        pytest.skip(f"pinned encoder absent at {FFMPEG}; run ./tools/fetch-ffmpeg.sh")
    subprocess.run(
        [
            str(FFMPEG),
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            f"testsrc2=size={width}x{height}:rate=1:duration=1",
            "-frames:v",
            "1",
            str(path),
        ],
        check=True,
    )


def test_the_header_parse_agrees_with_what_the_encoder_wrote(tmp_path: Path) -> None:
    path = tmp_path / "frame.jpg"
    _write_jpeg(path, 640, 360)

    assert jpeg_size(path) == (640, 360)


def test_something_that_is_not_a_jpeg_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "not.jpg"
    path.write_bytes(b"this is not an image")

    with pytest.raises(DecodeFailed):
        jpeg_size(path)


def test_a_wide_frame_letterboxes_to_the_top_left() -> None:
    box = letterbox_for(640, 360, 640)

    assert (box.width, box.height) == (640, 360)
    # A box filling the frame maps back to the whole of it, which only holds
    # because the padding is anchored rather than centred.
    left, top, width, height = box.to_normalized(0, 0, 640, 360)
    assert (left, top) == (0.0, 0.0)
    assert width == pytest.approx(1.0)
    assert height == pytest.approx(1.0)


def test_a_box_maps_back_to_a_share_of_the_original_frame() -> None:
    box = letterbox_for(1280, 720, 640)
    # A 64-pixel box a quarter of the way across the letterboxed image.
    left, top, width, height = box.to_normalized(160, 90, 64, 64)

    assert left == pytest.approx(0.25)
    assert top == pytest.approx(0.25)
    assert width == pytest.approx(64 / 640)
    assert height == pytest.approx(64 / 360)


def test_a_box_past_the_edge_is_clamped_rather_than_argued_with() -> None:
    box = letterbox_for(640, 360, 640)
    left, top, width, height = box.to_normalized(-10, -10, 700, 400)

    assert left == 0.0 and top == 0.0
    assert width <= 1.0 and height <= 1.0


def test_a_frame_with_no_extent_cannot_be_letterboxed() -> None:
    with pytest.raises(DecodeFailed):
        letterbox_for(0, 360, 640)


def test_frames_decode_through_the_pinned_decoder(tmp_path: Path) -> None:
    paths = []
    for index in range(3):
        path = tmp_path / f"frame_{index}.jpg"
        _write_jpeg(path, 640, 360)
        paths.append(path)

    box = letterbox_for(640, 360, 640)
    decoded = decode_frames(FFMPEG, paths, box)

    assert len(decoded) == 3
    for frame in decoded:
        assert frame.shape == (640, 640, 3)
    # The pad is black and sits below the image, which is what "anchored at the
    # top-left" has to mean for the mapping above to be a division.
    assert decoded[0][400:, :, :].max() == 0


def test_no_frames_decode_to_no_pixels() -> None:
    assert decode_frames(Path("/nonexistent"), [], letterbox_for(640, 360, 640)) == []


def test_a_decoder_that_is_not_there_is_a_refusal(tmp_path: Path) -> None:
    path = tmp_path / "frame.jpg"
    _write_jpeg(path, 320, 180)

    with pytest.raises(DecodeFailed):
        decode_frames(tmp_path / "no-such-ffmpeg", [path], letterbox_for(320, 180, 640))
