"""The exit gate's reading of a bar, held to the failure that motivated it.

`check-recall` exists because the Phase 1 verifier used to restate the
comparison itself, in a gate, against key names the bars never used. Every
lookup found nothing, every check skipped itself, and a report of total failure
passed. The tests here are written against that shape rather than against the
happy path: a gate whose only proof is that it accepts good evidence cannot
tell you whether it would have rejected bad.
"""

from __future__ import annotations

import json
from pathlib import Path

from clipmill_eval.cli import main

BAR = {
    "bar_id": "test-bar",
    "min_recall": 0.7,
    "min_multi_moment_recall": 0.5,
    "max_duplicate_rate": 0.2,
    "max_boundary_edge_error_millis": 500,
}


def report(**overrides: object) -> dict:
    """A passing report, so each test changes exactly the thing it is about."""

    base = {
        "schema_version": "clipmill.eval.recall.v1",
        "moments_total": 10,
        "moments_recalled": 8,
        "recall": 0.8,
        "recall_by_importance": {"essential": 1.0},
        "multi_moment_recall": 0.75,
        "duplicate_rate": 0.0,
        "excluded_offered": 0,
        "boundary_edge_error_millis": {"median": 120, "p90": 300, "measured": 8},
    }
    base.update(overrides)
    return base


def write(directory: Path, report_body: dict, bar_body: dict) -> tuple[Path, Path]:
    report_path = directory / "recall.json"
    bar_path = directory / "bar.json"
    report_path.write_text(json.dumps(report_body), encoding="utf-8")
    bar_path.write_text(json.dumps(bar_body), encoding="utf-8")
    return report_path, bar_path


def check(directory: Path, report_body: dict, bar_body: dict) -> int:
    report_path, bar_path = write(directory, report_body, bar_body)
    return main(["check-recall", "--report", str(report_path), "--bar", str(bar_path)])


def test_a_report_that_clears_every_bar_passes(tmp_path: Path) -> None:
    assert check(tmp_path, report(), BAR) == 0


def test_recall_below_the_bar_fails(tmp_path: Path) -> None:
    assert check(tmp_path, report(recall=0.5, moments_recalled=5), BAR) == 1


def test_a_duplicate_rate_above_the_bar_fails(tmp_path: Path) -> None:
    assert check(tmp_path, report(duplicate_rate=0.5), BAR) == 1


def test_a_boundary_error_above_the_bar_fails(tmp_path: Path) -> None:
    edges = {"median": 900, "p90": 1_200, "measured": 8}
    assert check(tmp_path, report(boundary_edge_error_millis=edges), BAR) == 1


def test_an_offered_exclusion_fails_whatever_the_bar_says(tmp_path: Path) -> None:
    """Never conditional on a bar mentioning it, so a bar cannot permit it."""

    assert check(tmp_path, report(excluded_offered=2), BAR) == 1


def test_a_bar_naming_no_minimum_recall_is_refused(tmp_path: Path) -> None:
    """The regression this command was written for.

    A bar using the wrong key names — or none — must not read as a bar every
    report satisfies. `meets_bar` is right to treat an absent key as a claim
    not yet made; the exit gate is the one caller for which that cannot pass.
    """

    disarmed = {"bar_id": "no-claims", "comment": "nothing constrained"}
    assert check(tmp_path, report(recall=0.0, duplicate_rate=1.0), disarmed) == 1


def test_the_key_names_the_bars_are_written_with_are_the_ones_read(
    tmp_path: Path,
) -> None:
    """The exact drift that disarmed the verifier, pinned by name.

    The old gate read `recall`/`multi_moment_recall`/`duplicate_rate` where
    every committed bar writes `min_recall`/`min_multi_moment_recall`/
    `max_duplicate_rate`. Spelling them here means a future rename has to touch
    a test that says why.
    """

    misspelt = {
        "bar_id": "the-old-gates-spelling",
        "recall": 1.0,
        "multi_moment_recall": 1.0,
        "duplicate_rate": 0.0,
    }
    assert check(tmp_path, report(recall=0.0, duplicate_rate=1.0), misspelt) == 1


def test_something_that_is_not_a_recall_report_is_refused(tmp_path: Path) -> None:
    assert check(tmp_path, {"schema_version": "clipmill.eval.render_slo.v1"}, BAR) == 1


def test_the_committed_planted_bar_still_names_a_minimum_recall() -> None:
    """The bars in the tree are readable by the code that reads bars.

    Cheap, and it is the half of the drift the unit tests above cannot see:
    they build their own bars, so they would keep passing if every committed
    bar were renamed out from under them.
    """

    root = Path(__file__).resolve().parents[3]
    planted = json.loads((root / "eval/recall/planted-bar.json").read_text(encoding="utf-8"))
    assert "min_recall" in planted
    assert planted["min_recall"] == 1.0
