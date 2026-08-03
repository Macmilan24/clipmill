"""What a ranked set got right, measured against plural ground truth.

This is the only implementation of these numbers. Everything that reports a
recall figure — the private corpus gate, the synthetic smoke, the attestation —
computes it here, because a metric implemented twice is a metric that will one
day disagree with itself and no reader will know which number was the claim.

Four things are measured and they answer different questions (book ch. 22):

- **Recall@K at IoU** asks whether the acceptable moments reached the pool at
  all. It is the retrieval question and it is the one a missing proposer breaks.
- **Multi-moment recall** asks whether a recording holding several moments gave
  up all of them, not just the easiest one. Reported apart from Recall@K
  because a system that always finds exactly one moment per recording scores
  well on the first and badly on the second, and those are different failures.
- **Duplicate rate** asks whether the *set* is useful. Five cuts of one moment
  is a high recall score and a bad results board.
- **Boundary-edge error** asks whether the cut landed where an editor would
  have put it, and is measured against the annotated alternatives rather than
  against one preferred span — a cut that agrees with the editor's second
  choice is not an error, and scoring it as one would punish agreement.

Nothing here reads a file, a clock, or a socket. Every function takes the
numbers it needs so the same call can be made against a real run and against a
planted fixture, which is what lets the smoke prove the metric rather than
prove itself.
"""

from __future__ import annotations

from collections.abc import Iterable, Sequence
from dataclasses import dataclass

from .annotation import Annotation, Interval, Moment

#: The overlap at which a clip is held to have found a moment. Half is the
#: figure the retrieval literature reports at and the figure the plan names; it
#: is stated as a parameter everywhere below so a stricter run is a call, not a
#: fork.
DEFAULT_IOU = 0.5

#: Two clips overlapping by this much are the same moment cut twice. Set equal
#: to the recall threshold on purpose: a pair that would both be credited with
#: finding one moment is, by that same rule, a duplicate.
DEFAULT_DUPLICATE_IOU = 0.5

#: What the board shows without scrolling, and what the plan measures at.
DEFAULT_RECALL_K = 10
DEFAULT_DUPLICATE_K = 5


def intersection_ticks(left: Interval, right: Interval) -> int:
    """Ticks two spans share. Never negative."""

    return max(0, min(left.end_ticks, right.end_ticks) - max(left.start_ticks, right.start_ticks))


def iou(left: Interval, right: Interval) -> float:
    """Intersection over union, on the tick line.

    Two empty spans at the same point are not "perfectly overlapping" — they are
    two things with no extent — so the answer is zero rather than a division
    nobody can defend.
    """

    overlap = intersection_ticks(left, right)
    union = left.duration_ticks + right.duration_ticks - overlap
    if union <= 0:
        return 0.0
    return overlap / union


@dataclass(frozen=True, slots=True)
class Clip:
    """One clip a run offered, reduced to what scoring reads."""

    candidate_id: str
    span: Interval
    #: The cluster discovery put it in. Used to report how much of the
    #: duplication the system already knew about, which is a different fact
    #: from how much it shipped.
    cluster_id: str | None = None


@dataclass(frozen=True, slots=True)
class MomentMatch:
    """One moment, and the best a run did against it."""

    moment_id: str
    importance: str
    matched: bool
    best_iou: float
    #: The clip that matched, when one did.
    candidate_id: str | None
    #: Ticks the chosen cut differs from the nearest acceptable start and end,
    #: summed. `None` when nothing matched, because an edge error against a
    #: moment nobody found is not a measurement.
    edge_error_ticks: int | None


def best_overlap(clip: Interval, moment: Moment) -> float:
    """The best a clip does against a moment, over every span the annotator
    would have accepted.

    The preferred span and the alternatives are scored together rather than the
    preferred one alone: the alternatives exist precisely because the annotator
    said those cuts were also right.
    """

    return max(iou(clip, candidate) for candidate in moment.acceptable_spans())


def edge_error(clip: Interval, moment: Moment) -> int:
    """Ticks between a cut and the nearest acceptable cut.

    The start and end are matched to the *same* acceptable span rather than to
    the nearest start and the nearest end independently: taking one span's start
    and another's end would report an error against a cut no annotator offered.
    """

    return min(
        abs(clip.start_ticks - candidate.start_ticks) + abs(clip.end_ticks - candidate.end_ticks)
        for candidate in moment.acceptable_spans()
    )


def match_moments(
    clips: Sequence[Clip],
    moments: Sequence[Moment],
    threshold: float = DEFAULT_IOU,
) -> list[MomentMatch]:
    """Score every moment against the clips on offer.

    Every moment is scored against every clip — a clip may satisfy two moments
    and a moment may be found by two clips, and forcing a one-to-one assignment
    would report a lower number than the truth for exactly the recordings the
    multi-moment metric exists to describe.
    """

    matches: list[MomentMatch] = []
    for moment in moments:
        best_value = 0.0
        best_clip: Clip | None = None
        for clip in clips:
            overlap = best_overlap(clip.span, moment)
            if overlap > best_value:
                best_value, best_clip = overlap, clip
        matched = best_clip is not None and best_value >= threshold
        matches.append(
            MomentMatch(
                moment_id=moment.moment_id,
                importance=moment.importance,
                matched=matched,
                best_iou=best_value,
                candidate_id=best_clip.candidate_id if matched and best_clip else None,
                edge_error_ticks=(
                    edge_error(best_clip.span, moment) if matched and best_clip else None
                ),
            )
        )
    return matches


def duplicate_rate(
    clips: Sequence[Clip],
    threshold: float = DEFAULT_DUPLICATE_IOU,
) -> tuple[float, int]:
    """How much of a set is the same moment more than once.

    A clip counts as a duplicate when it overlaps an *earlier, better-ranked*
    clip past the threshold. Ranked order matters: the first cut of a moment is
    the one the set wanted, and the second is the one it should not have shown.
    """

    duplicated = 0
    for index, clip in enumerate(clips):
        if any(iou(clip.span, earlier.span) >= threshold for earlier in clips[:index]):
            duplicated += 1
    if not clips:
        return 0.0, 0
    return duplicated / len(clips), duplicated


def _percentile(values: Sequence[int], fraction: float) -> int:
    """Nearest-rank percentile. Integers in, integers out: these are ticks, and
    interpolating between two measured errors would report an error nobody
    measured."""

    if not values:
        return 0
    ordered = sorted(values)
    index = min(len(ordered) - 1, max(0, round(fraction * (len(ordered) - 1))))
    return ordered[index]


@dataclass(frozen=True, slots=True)
class ItemScore:
    """One recording, scored."""

    item_id: str
    source_fingerprint: str
    annotator_id: str
    moments_total: int
    moments_recalled: int
    matches: tuple[MomentMatch, ...]
    duplicate_rate_at_k: float
    duplicates_at_k: int
    clips_offered: int
    #: True when this recording holds more than one moment, which is the
    #: population the multi-moment number is about.
    multi_moment: bool
    #: True when every moment in a multi-moment recording was recalled.
    all_moments_recalled: bool
    excluded_offered: int

    @property
    def recall(self) -> float:
        if self.moments_total == 0:
            return 1.0
        return self.moments_recalled / self.moments_total


def score_item(
    annotation: Annotation,
    clips: Sequence[Clip],
    *,
    recall_k: int = DEFAULT_RECALL_K,
    duplicate_k: int = DEFAULT_DUPLICATE_K,
    threshold: float = DEFAULT_IOU,
    duplicate_threshold: float = DEFAULT_DUPLICATE_IOU,
) -> ItemScore:
    """Score one recording's ranked set against one annotator's document."""

    top = list(clips[:recall_k])
    matches = match_moments(top, annotation.moments, threshold)
    recalled = sum(1 for match in matches if match.matched)
    rate, duplicated = duplicate_rate(list(clips[:duplicate_k]), duplicate_threshold)
    # An excluded span that was offered is not a recall miss — it is a separate
    # and worse failure, so it is counted rather than folded into the rate.
    excluded_offered = sum(
        1
        for clip in top
        if any(iou(clip.span, exclusion.span) >= threshold for exclusion in annotation.exclusions)
    )
    multi = len(annotation.moments) > 1
    return ItemScore(
        item_id=annotation.item_id or annotation.source_fingerprint,
        source_fingerprint=annotation.source_fingerprint,
        annotator_id=annotation.annotator_id,
        moments_total=len(annotation.moments),
        moments_recalled=recalled,
        matches=tuple(matches),
        duplicate_rate_at_k=rate,
        duplicates_at_k=duplicated,
        clips_offered=len(clips),
        multi_moment=multi,
        all_moments_recalled=multi and recalled == len(annotation.moments),
        excluded_offered=excluded_offered,
    )


@dataclass(frozen=True, slots=True)
class CorpusScore:
    """Every recording, aggregated — with the parts kept separable."""

    items: tuple[ItemScore, ...]
    recall_k: int
    duplicate_k: int
    iou_threshold: float

    @property
    def moments_total(self) -> int:
        return sum(item.moments_total for item in self.items)

    @property
    def moments_recalled(self) -> int:
        return sum(item.moments_recalled for item in self.items)

    @property
    def recall(self) -> float:
        """Recall over every annotated moment in the corpus.

        Pooled over moments rather than averaged over recordings: a recording
        with one moment and a recording with nine are different amounts of
        evidence, and averaging their rates would weight them the same.
        """

        if self.moments_total == 0:
            return 1.0
        return self.moments_recalled / self.moments_total

    def recall_by_importance(self) -> dict[str, float]:
        """Recall per grade.

        Reported because the aggregate hides the failure that matters: finding
        every acceptable moment and missing every essential one is a good
        overall number and a useless product.
        """

        totals: dict[str, list[int]] = {}
        for item in self.items:
            for match in item.matches:
                found, total = totals.setdefault(match.importance, [0, 0])
                totals[match.importance] = [found + int(match.matched), total + 1]
        return {grade: found / total for grade, (found, total) in totals.items() if total}

    @property
    def multi_moment_recall(self) -> float:
        """Fraction of multi-moment recordings that gave up *every* moment.

        One means a recording where several things were worth clipping produced
        all of them. Recordings with a single moment are excluded rather than
        counted as successes, because they cannot fail this test and including
        them would inflate it.
        """

        population = [item for item in self.items if item.multi_moment]
        if not population:
            return 1.0
        return sum(1 for item in population if item.all_moments_recalled) / len(population)

    @property
    def duplicate_rate(self) -> float:
        """Mean duplicate rate over recordings that offered any clips."""

        offering = [item for item in self.items if item.clips_offered]
        if not offering:
            return 0.0
        return sum(item.duplicate_rate_at_k for item in offering) / len(offering)

    @property
    def excluded_offered(self) -> int:
        return sum(item.excluded_offered for item in self.items)

    def edge_errors(self) -> list[int]:
        return [
            match.edge_error_ticks
            for item in self.items
            for match in item.matches
            if match.edge_error_ticks is not None
        ]

    def report(self, timebase_den: int = 90_000) -> dict[str, object]:
        """The document a gate reads and an attestation signs.

        Ticks are converted to milliseconds only here, at the edge, because a
        report is read by people and every other number in this module is exact.
        """

        errors = self.edge_errors()
        return {
            "schema_version": "clipmill.eval.recall.v1",
            "protocol": {
                "recall_k": self.recall_k,
                "duplicate_k": self.duplicate_k,
                "iou_threshold": self.iou_threshold,
            },
            "moments_total": self.moments_total,
            "moments_recalled": self.moments_recalled,
            "recall": self.recall,
            "recall_by_importance": self.recall_by_importance(),
            "multi_moment_recall": self.multi_moment_recall,
            "multi_moment_items": sum(1 for item in self.items if item.multi_moment),
            "duplicate_rate": self.duplicate_rate,
            "excluded_offered": self.excluded_offered,
            "boundary_edge_error_millis": {
                "median": _ticks_to_millis(_percentile(errors, 0.5), timebase_den),
                "p90": _ticks_to_millis(_percentile(errors, 0.9), timebase_den),
                "measured": len(errors),
            },
            "items": [
                {
                    "item_id": item.item_id,
                    "annotator_id": item.annotator_id,
                    "moments_total": item.moments_total,
                    "moments_recalled": item.moments_recalled,
                    "recall": item.recall,
                    "duplicate_rate": item.duplicate_rate_at_k,
                    "clips_offered": item.clips_offered,
                    "excluded_offered": item.excluded_offered,
                    "misses": [
                        {"moment_id": match.moment_id, "best_iou": match.best_iou}
                        for match in item.matches
                        if not match.matched
                    ],
                }
                for item in self.items
            ],
        }


def _ticks_to_millis(ticks: int, den: int) -> int:
    if den <= 0:
        return 0
    return round(ticks * 1000 / den)


def score_corpus(
    scored: Iterable[ItemScore],
    *,
    recall_k: int = DEFAULT_RECALL_K,
    duplicate_k: int = DEFAULT_DUPLICATE_K,
    threshold: float = DEFAULT_IOU,
) -> CorpusScore:
    return CorpusScore(
        items=tuple(scored),
        recall_k=recall_k,
        duplicate_k=duplicate_k,
        iou_threshold=threshold,
    )
