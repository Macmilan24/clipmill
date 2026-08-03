"""Reading what an editor said, and refusing what nobody could act on.

An annotation is the only thing in this project that is *not* derived from the
recording — it is a person's opinion, and it is the thing every recall number is
measured against. So it is validated hard on the way in: a span that runs
backwards, a moment graded with a word nobody defined, an exclusion with no
reason, a hook outside the moment it belongs to. Each of these would produce a
number rather than an error, and a number computed from a broken annotation is
worse than no number because somebody would quote it.

Worksheets go the other way: the harness knows the times and the words, and the
annotator does not want to type either. `worksheet` emits a document already
carrying the transcript so what is left is judgement.
"""

from __future__ import annotations

import json
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "clipmill.eval_annotation.v1"
IMPORTANCE = ("essential", "strong", "acceptable")
EXCLUSION_REASONS = (
    "rights",
    "personal_information",
    "misleading_out_of_context",
    "poor_audio",
    "poor_picture",
    "off_topic",
    "duplicate_of_a_moment",
)


class AnnotationError(ValueError):
    """An annotation document could not be trusted."""


@dataclass(frozen=True, slots=True)
class Interval:
    start_ticks: int
    end_ticks: int

    @property
    def duration_ticks(self) -> int:
        return self.end_ticks - self.start_ticks


@dataclass(frozen=True, slots=True)
class Moment:
    moment_id: str
    span: Interval
    importance: str
    alternatives: tuple[Interval, ...] = ()
    hook_ticks: int | None = None
    payoff_ticks: int | None = None
    required_context: str = ""
    topic: str = ""

    def acceptable_spans(self) -> tuple[Interval, ...]:
        """Every cut this annotator would have accepted for this moment.

        The preferred span is first and the alternatives follow. Scoring reads
        this rather than `span` alone, which is the whole reason alternatives
        are collected.
        """

        return (self.span, *self.alternatives)


@dataclass(frozen=True, slots=True)
class Exclusion:
    span: Interval
    reason: str


@dataclass(frozen=True, slots=True)
class Annotation:
    source_fingerprint: str
    annotator_id: str
    annotated_unix_millis: int
    timebase_num: int
    timebase_den: int
    moments: tuple[Moment, ...]
    exclusions: tuple[Exclusion, ...]
    item_id: str | None = None
    notes: str = ""


def load_annotation(path: Path) -> Annotation:
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise AnnotationError(f"cannot read {path}: {error}") from error
    if not isinstance(raw, dict):
        raise AnnotationError(f"{path} must contain a JSON object")
    try:
        return parse_annotation(raw)
    except AnnotationError as error:
        raise AnnotationError(f"{path}: {error}") from error


def load_annotations(directory: Path) -> tuple[Annotation, ...]:
    """Every annotation in a directory, in a stable order.

    Sorted by filename so two runs over the same directory produce the same
    report — a recall figure whose item order depended on the filesystem would
    not be comparable with the one before it.
    """

    if not directory.is_dir():
        raise AnnotationError(f"{directory} is not a directory")
    found = sorted(directory.glob("*.json"))
    if not found:
        raise AnnotationError(f"{directory} holds no annotations")
    return tuple(load_annotation(path) for path in found)


def parse_annotation(raw: dict[str, Any]) -> Annotation:
    if raw.get("schema_version") != SCHEMA_VERSION:
        raise AnnotationError(f"schema_version must be {SCHEMA_VERSION}")
    fingerprint = _text(raw, "source_fingerprint")
    if not fingerprint.startswith("sha256:") or len(fingerprint) != len("sha256:") + 64:
        raise AnnotationError("source_fingerprint must be a prefixed sha256 digest")
    timebase = raw.get("timebase")
    if not isinstance(timebase, dict):
        raise AnnotationError("timebase is required")
    num, den = _positive(timebase, "num"), _positive(timebase, "den")

    moments = tuple(_moment(entry) for entry in _sequence(raw, "moments"))
    seen: set[str] = set()
    for moment in moments:
        if moment.moment_id in seen:
            raise AnnotationError(f"duplicate moment_id: {moment.moment_id}")
        seen.add(moment.moment_id)

    exclusions = tuple(_exclusion(entry) for entry in raw.get("exclusions", []) or [])
    item_id = raw.get("item_id")
    if item_id is not None and (not isinstance(item_id, str) or not item_id):
        raise AnnotationError("item_id must be a non-empty string when present")
    return Annotation(
        source_fingerprint=fingerprint,
        annotator_id=_text(raw, "annotator_id"),
        annotated_unix_millis=_non_negative(raw, "annotated_unix_millis"),
        timebase_num=num,
        timebase_den=den,
        moments=moments,
        exclusions=exclusions,
        item_id=item_id,
        notes=raw.get("notes", "") or "",
    )


def _moment(entry: Any) -> Moment:
    if not isinstance(entry, dict):
        raise AnnotationError("each moment must be an object")
    span = _interval(entry.get("span"), "span")
    importance = _text(entry, "importance")
    if importance not in IMPORTANCE:
        raise AnnotationError(f"importance must be one of {IMPORTANCE}, not {importance!r}")
    alternatives = tuple(
        _interval(value, "alternative") for value in entry.get("alternatives", []) or []
    )
    hook = _optional_tick(entry, "hook_ticks")
    payoff = _optional_tick(entry, "payoff_ticks")
    # A hook outside the span it belongs to is a transcription slip, not an
    # opinion, and it would move a future hook metric without anybody noticing.
    for name, value in (("hook_ticks", hook), ("payoff_ticks", payoff)):
        if value is not None and not span.start_ticks <= value <= span.end_ticks:
            raise AnnotationError(f"{name} lies outside the moment it belongs to")
    return Moment(
        moment_id=_text(entry, "moment_id"),
        span=span,
        importance=importance,
        alternatives=alternatives,
        hook_ticks=hook,
        payoff_ticks=payoff,
        required_context=entry.get("required_context", "") or "",
        topic=entry.get("topic", "") or "",
    )


def _exclusion(entry: Any) -> Exclusion:
    if not isinstance(entry, dict):
        raise AnnotationError("each exclusion must be an object")
    reason = _text(entry, "reason")
    if reason not in EXCLUSION_REASONS:
        raise AnnotationError(f"exclusion reason must be one of {EXCLUSION_REASONS}")
    return Exclusion(span=_interval(entry.get("span"), "exclusion span"), reason=reason)


def _interval(value: Any, label: str) -> Interval:
    if not isinstance(value, dict):
        raise AnnotationError(f"{label} must be an object")
    start, end = _non_negative(value, "start_ticks"), _non_negative(value, "end_ticks")
    if end <= start:
        raise AnnotationError(f"{label} must end after it starts")
    return Interval(start_ticks=start, end_ticks=end)


def _text(raw: dict[str, Any], key: str) -> str:
    value = raw.get(key)
    if not isinstance(value, str) or not value:
        raise AnnotationError(f"{key} must be a non-empty string")
    return value


def _non_negative(raw: dict[str, Any], key: str) -> int:
    value = raw.get(key)
    # `bool` is an `int` in Python and would sail through as 0 or 1.
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise AnnotationError(f"{key} must be a non-negative integer")
    return value


def _positive(raw: dict[str, Any], key: str) -> int:
    value = _non_negative(raw, key)
    if value == 0:
        raise AnnotationError(f"{key} must be positive")
    return value


def _optional_tick(raw: dict[str, Any], key: str) -> int | None:
    if key not in raw or raw[key] is None:
        return None
    return _non_negative(raw, key)


def _sequence(raw: dict[str, Any], key: str) -> Iterator[Any]:
    value = raw.get(key)
    if not isinstance(value, list):
        raise AnnotationError(f"{key} must be an array")
    return iter(value)
