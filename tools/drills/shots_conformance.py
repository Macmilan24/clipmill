#!/usr/bin/env python3
"""Run shot detection over a fixture whose cuts are known.

The arithmetic is unit-tested against frames written by hand, which is where the
algorithm is checked. This is the other half: a real encode, a real decode
through the pinned FFmpeg, and the four properties the gate is written in terms
of.

    cuts            found where the fixture was cut, to the frame
    motion          a fast pan inside a shot is not reported as a cut
    determinism     a second pass over the same file produces the same document
    a broken proxy  fails with a reason rather than publishing "no cuts"

The third is what makes a cached detection worth caching, and the one most
likely to break quietly: a run that is merely *usually* deterministic passes
every other check here.

    tools/drills/shots_conformance.py <fixture-dir> --ffmpeg <path>
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from itertools import pairwise
from pathlib import Path

from clipmill_worker_sdk.ticks import frames_to_ticks
from clipmill_worker_shots import content
from clipmill_worker_shots.content import Parameters, detect, spans
from clipmill_worker_shots.decode import DecodeFailed, analysis_size, decode_frames

# A cut may land one frame from where the fixture put it and still be right: the
# encode is lossy, and the frame that first *differs* is what a content detector
# can possibly report. Anything looser than this would let a detector be a whole
# shot wrong at the edges and still pass.
FRAME_TOLERANCE = 1


@dataclass
class Failure:
    check: str
    detail: str


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path)
    parser.add_argument(
        "--ffmpeg",
        type=Path,
        default=Path(".cache/bin/ffmpeg"),
        help="the pinned decoder, as the daemon would hand it to the worker",
    )
    options = parser.parse_args()

    truth = json.loads((options.fixture / "truth.json").read_text(encoding="utf-8"))
    video = options.fixture / "shots.mp4"
    ffmpeg = options.ffmpeg.resolve()
    if not ffmpeg.is_file():
        print(f"shots-conformance: no decoder at {ffmpeg}", file=sys.stderr)
        return 2

    failures: list[Failure] = []
    first = run_detection(ffmpeg, video, truth)
    # A second pass over the same file through the same decoder. Anything that
    # differs is a stage whose output a cache cannot serve.
    second = run_detection(ffmpeg, video, truth)
    if first != second:
        failures.append(
            Failure("determinism", "a second pass over the same file produced different output")
        )

    failures.extend(check_cuts(first, truth))
    failures.extend(check_shots(first, truth))
    failures.extend(check_broken_proxy(ffmpeg, options.fixture, truth))

    for failure in failures:
        print(f"shots-conformance: {failure.check}: {failure.detail}", file=sys.stderr)
    if failures:
        return 1
    print(
        f"shots-conformance: OK ({len(first['cuts'])} cuts found at the frames the fixture "
        f"was cut at, {first['frames']} frames decoded, largest change inside a shot "
        f"{first['loudest_within_shot']:.1f} against a {first['threshold']:.0f} threshold, "
        f"identical on a second pass, damaged proxies refused or honestly short)"
    )
    return 0


def run_detection(ffmpeg: Path, video: Path, truth: dict) -> dict:
    """Decode and detect exactly as the worker does, at the shipped defaults."""

    rate_num = truth["frame_rate"]["num"]
    rate_den = truth["frame_rate"]["den"]
    size = analysis_size(truth["width"], truth["height"], content.DEFAULT_ANALYSIS_HEIGHT)
    per_frame = frames_to_ticks(1, rate_num, rate_den)
    parameters = Parameters(
        threshold=content.DEFAULT_THRESHOLD,
        min_shot_frames=max(1, -(-content.DEFAULT_MIN_SHOT_TICKS // per_frame)),
    )
    cuts, frames = detect(
        decode_frames(ffmpeg, video, size),
        parameters,
        frame_rate=rate_num / rate_den,
    )
    return {
        "frames": frames,
        "threshold": parameters.threshold,
        "cuts": [
            {
                "frame": cut.frame,
                "t_ticks": frames_to_ticks(cut.frame, rate_num, rate_den),
                "score": round(cut.score, 4),
                "p50": cut.confidence.p50,
                "p10": cut.confidence.p10,
            }
            for cut in cuts
        ],
        "shots": [
            {
                "start_ticks": frames_to_ticks(span.start_frame, rate_num, rate_den),
                "end_ticks": frames_to_ticks(span.end_frame, rate_num, rate_den),
                "p50": span.confidence.p50,
            }
            for span in spans(cuts, frames)
        ],
        # The largest change the detector saw that it did *not* call a cut. This
        # is the number that says whether the threshold is discriminating or
        # merely lucky, and it is reported rather than only asserted.
        "loudest_within_shot": _loudest_within_shot(ffmpeg, video, truth),
    }


def _loudest_within_shot(ffmpeg: Path, video: Path, truth: dict) -> float:
    """Re-run at a threshold nothing clears, to read every frame's distance.

    Cheap enough on a six-second fixture, and the alternative is exporting the
    per-frame scores from the worker for a number only a gate wants.
    """

    rate_num = truth["frame_rate"]["num"]
    rate_den = truth["frame_rate"]["den"]
    size = analysis_size(truth["width"], truth["height"], content.DEFAULT_ANALYSIS_HEIGHT)
    everything, _ = detect(
        decode_frames(ffmpeg, video, size),
        Parameters(threshold=0.001, min_shot_frames=1),
        frame_rate=rate_num / rate_den,
    )
    cut_frames = {entry["frame"] for entry in truth["cuts"]}
    return max(
        (cut.score for cut in everything if cut.frame not in cut_frames),
        default=0.0,
    )


def check_cuts(result: dict, truth: dict) -> list[Failure]:
    """Every cut the fixture made, and no cut it did not."""

    expected = [entry["frame"] for entry in truth["cuts"]]
    found = [entry["frame"] for entry in result["cuts"]]
    if len(found) != len(expected):
        return [
            Failure(
                "cuts",
                f"the fixture was cut at {expected} and the detector reported {found}",
            )
        ]
    failures = []
    for want, got in zip(expected, found, strict=True):
        if abs(want - got) > FRAME_TOLERANCE:
            failures.append(
                Failure(
                    "cuts",
                    f"a cut at frame {want} was reported at {got}, "
                    f"more than {FRAME_TOLERANCE} frame away",
                )
            )
    # A pan is not a cut. Stated as a threshold comparison rather than as "no
    # extra cuts were found", because the count above already says that and
    # this says how much room there was.
    if result["loudest_within_shot"] >= result["threshold"]:
        failures.append(
            Failure(
                "motion",
                f"the largest change inside a shot scored "
                f"{result['loudest_within_shot']:.1f}, at or above the "
                f"{result['threshold']:.0f} threshold: the fixture's pan is "
                f"indistinguishable from its cuts",
            )
        )
    return failures


def check_shots(result: dict, truth: dict) -> list[Failure]:
    """The spans tile the decoded range, and every cut starts one."""

    failures = []
    if result["frames"] != truth["frame_count"]:
        failures.append(
            Failure(
                "coverage",
                f"the fixture holds {truth['frame_count']} frames and "
                f"{result['frames']} were decoded",
            )
        )
    shots = result["shots"]
    if not shots:
        return [*failures, Failure("shots", "no shots were published at all")]
    if shots[0]["start_ticks"] != 0:
        failures.append(Failure("shots", "the first shot does not start at the recording's start"))
    for earlier, later in pairwise(shots):
        if earlier["end_ticks"] != later["start_ticks"]:
            failures.append(
                Failure(
                    "shots",
                    f"a gap or overlap between {earlier['end_ticks']} and {later['start_ticks']}",
                )
            )
    for cut, shot in zip(result["cuts"], shots[1:], strict=True):
        if cut["t_ticks"] != shot["start_ticks"]:
            failures.append(
                Failure(
                    "shots",
                    f"a cut at {cut['t_ticks']} does not start the shot at {shot['start_ticks']}",
                )
            )
    return failures


def check_broken_proxy(ffmpeg: Path, fixture: Path, truth: dict) -> list[Failure]:
    """A proxy that is not video refuses; a short one says how short it was.

    The dangerous failure here is not a crash. It is a document saying "this
    recording has no cuts" under a content address, produced from a file nobody
    could read — indistinguishable, downstream, from a genuinely unbroken take.

    The two damaged files below fail differently on purpose. Bytes that are not
    a video are refused outright with the decoder's own reason. A file that is
    merely cut short still decodes, because that is what a decoder does with
    the frames it has, and the obligation is then honesty rather than refusal:
    coverage must describe what was examined, never what the container
    promised. (Neither can reach the worker through the real path — the CAS
    verifies a proxy's digest before the worker opens it — which is exactly why
    the decode layer's behaviour is worth pinning down here.)
    """

    size = analysis_size(truth["width"], truth["height"], content.DEFAULT_ANALYSIS_HEIGHT)
    original = (fixture / "shots.mp4").read_bytes()
    failures = []

    not_video = fixture / "not-video.mp4"
    not_video.write_bytes(b"\x00\xff" * 4096)
    try:
        list(decode_frames(ffmpeg, not_video, size))
        failures.append(
            Failure(
                "broken proxy",
                "bytes that are not a video decoded without complaint",
            )
        )
    except DecodeFailed:
        pass
    finally:
        not_video.unlink(missing_ok=True)

    truncated = fixture / "truncated.mp4"
    truncated.write_bytes(original[: len(original) // 3])
    try:
        decoded = len(list(decode_frames(ffmpeg, truncated, size)))
        if decoded >= truth["frame_count"]:
            failures.append(
                Failure(
                    "broken proxy",
                    f"a third of a file decoded to {decoded} frames, at or beyond "
                    f"the {truth['frame_count']} the whole one holds: coverage "
                    f"would claim footage that is not there",
                )
            )
        if decoded == 0:
            failures.append(
                Failure(
                    "broken proxy",
                    "a truncated file decoded to nothing, so this check is not "
                    "exercising the partial-decode path it was written for",
                )
            )
    except DecodeFailed:
        # Also acceptable, and on some builds this is what happens. The failure
        # being guarded against is a full-length claim, not a refusal.
        pass
    finally:
        truncated.unlink(missing_ok=True)
    return failures


if __name__ == "__main__":
    raise SystemExit(main())
