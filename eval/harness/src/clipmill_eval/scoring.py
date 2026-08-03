"""Joining what the system offered to what an editor said was there.

Three documents meet here and each is read for exactly one thing: the ranking
set says which clips were selected and where they were cut, the candidate set
says which cluster each belonged to, and the annotation says what was worth
finding. The arithmetic itself lives in `recall`; this module is the part that
knows the contracts.

The selection is read in `selected` order, not by rank over the whole cohort.
`selected` is what a user is shown — the diversity-aware subset — and measuring
recall over the full cohort instead would report how well a hundred candidates
cover the recording, which is a number nobody sees the consequences of.
"""

from __future__ import annotations

from collections.abc import Iterable, Sequence
from typing import Any

from .annotation import Annotation, Interval
from .recall import Clip, CorpusScore, ItemScore, score_corpus, score_item


class ScoringError(ValueError):
    """A run's documents could not be scored against an annotation."""


def clips_from_ranking(
    ranking: dict[str, Any],
    candidates: dict[str, Any] | None = None,
) -> list[Clip]:
    """The clips a user would have been shown, in the order they were shown.

    The interval taken is `boundary.chosen` — where the optimizer decided the
    clip should actually be cut — rather than the candidate's proposed interval.
    That is what would be rendered, so it is what an editor's cut is compared
    against; scoring the proposal instead would credit the system for a span it
    would not have delivered.
    """

    ranked = {entry["candidate_id"]: entry for entry in _list(ranking, "cohort")}
    clusters = _clusters(candidates) if candidates else {}
    clips: list[Clip] = []
    for candidate_id in _list(ranking, "selected"):
        entry = ranked.get(candidate_id)
        if entry is None:
            raise ScoringError(f"{candidate_id} was selected but is not in the cohort")
        chosen = entry.get("boundary", {}).get("chosen")
        if not isinstance(chosen, dict):
            raise ScoringError(f"{candidate_id} carries no chosen boundary")
        clips.append(
            Clip(
                candidate_id=candidate_id,
                span=Interval(
                    start_ticks=int(chosen["start_ticks"]),
                    end_ticks=int(chosen["end_ticks"]),
                ),
                cluster_id=clusters.get(candidate_id),
            )
        )
    return clips


def _clusters(candidates: dict[str, Any]) -> dict[str, str]:
    return {
        entry["id"]: entry.get("cluster_id")
        for entry in _list(candidates, "candidates")
        if isinstance(entry, dict) and "id" in entry
    }


def _list(document: dict[str, Any], key: str) -> list[Any]:
    value = document.get(key)
    if not isinstance(value, list):
        raise ScoringError(f"{key} must be an array")
    return value


def score_run(
    pairs: Iterable[tuple[Annotation, Sequence[Clip]]],
    **options: Any,
) -> CorpusScore:
    """Score every annotated recording in a run.

    Annotations are the population, not the run: a recording the run skipped is
    a recording whose moments were all missed, and dropping it would turn a
    crash into a perfect score.
    """

    scored: list[ItemScore] = [
        score_item(annotation, list(clips), **options) for annotation, clips in pairs
    ]
    return score_corpus(
        scored,
        recall_k=options.get("recall_k", 10),
        duplicate_k=options.get("duplicate_k", 5),
        threshold=options.get("threshold", 0.5),
    )


def meets_bar(report: dict[str, Any], bar: dict[str, Any]) -> list[str]:
    """Every way a report falls short of a bar, in the words a gate prints.

    Returns the failures rather than a boolean, because "recall gate failed" is
    not something anybody can act on and the whole point of the metric stack is
    that the parts are separable.

    A bar names only what it constrains. An absent key is not a silent pass of
    something set to zero — it is a metric this bar does not yet make a claim
    about, which is the honest state before the first baseline is measured.
    """

    failures: list[str] = []
    if "min_recall" in bar and report["recall"] < bar["min_recall"]:
        failures.append(
            f"recall {report['recall']:.3f} is below the bar {bar['min_recall']:.3f} "
            f"({report['moments_recalled']}/{report['moments_total']} moments)"
        )
    if "min_multi_moment_recall" in bar:
        measured = report["multi_moment_recall"]
        if measured < bar["min_multi_moment_recall"]:
            failures.append(
                f"multi-moment recall {measured:.3f} is below the bar "
                f"{bar['min_multi_moment_recall']:.3f}"
            )
    if "max_duplicate_rate" in bar and report["duplicate_rate"] > bar["max_duplicate_rate"]:
        failures.append(
            f"duplicate rate {report['duplicate_rate']:.3f} is above the bar "
            f"{bar['max_duplicate_rate']:.3f}"
        )
    if "max_boundary_edge_error_millis" in bar:
        measured = report["boundary_edge_error_millis"]["median"]
        if measured > bar["max_boundary_edge_error_millis"]:
            failures.append(
                f"median boundary error {measured} ms is above the bar "
                f"{bar['max_boundary_edge_error_millis']} ms"
            )
    for grade, floor in (bar.get("min_recall_by_importance") or {}).items():
        measured = report["recall_by_importance"].get(grade)
        if measured is None:
            # A grade nobody annotated cannot be scored, and treating it as a
            # pass would let a corpus with no essential moments satisfy a bar
            # about essential moments.
            failures.append(f"the bar names {grade} moments and the corpus holds none")
        elif measured < floor:
            failures.append(f"{grade} recall {measured:.3f} is below the bar {floor:.3f}")
    # Offering an excluded span is never acceptable, so it is checked whether or
    # not a bar mentions it.
    if report["excluded_offered"]:
        failures.append(f"{report['excluded_offered']} excluded spans were offered")
    return failures
