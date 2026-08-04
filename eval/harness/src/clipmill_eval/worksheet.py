"""Turning a transcript into something a person can annotate.

Annotation is the one step nothing can automate, so the harness does everything
around it: it reads the transcript the pipeline already produced, prints it with
timecodes a person can scan, and writes a skeleton document with the fields
already named. What is left for the annotator is judgement, which is the only
part worth their time.

Two files per recording rather than one, because they are read differently. The
transcript is prose, scanned on a screen or on paper, and belongs in Markdown.
The annotation is a document with a published schema and `additionalProperties`
false — transcript text cannot live inside it, and stuffing it in a `notes`
field would be putting a page of prose where a sentence belongs.
"""

from __future__ import annotations

import json
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .annotation import SCHEMA_VERSION

#: Ticks per second everything in this project counts in (decision D06).
TICKS_PER_SECOND = 90_000


@dataclass(frozen=True, slots=True)
class Line:
    """One readable unit of the transcript, with where it is."""

    start_ticks: int
    end_ticks: int
    text: str


def sentences_from_index(index: dict[str, Any]) -> list[Line]:
    """The evidence index's sentences, with the ticks they cover.

    The index is read rather than the raw transcript because the index is what
    discovery reads: annotating against a different segmentation from the one
    the system proposes over would measure the segmentation as much as the
    ranking.

    The index carries each sentence's text beside its word range, so nothing is
    reassembled here — a worksheet that re-joined words could disagree with the
    document the proposers scored, over spacing if nothing else.
    """

    found = index.get("sentences")
    if not isinstance(found, list) or not found:
        raise ValueError("the index carries no sentences to annotate against")
    lines: list[Line] = []
    for unit in found:
        if not isinstance(unit, dict):
            raise ValueError("each sentence must be an object")
        start, end = int(unit["start_ticks"]), int(unit["end_ticks"])
        if end <= start:
            raise ValueError(f"sentence at {start} does not end after it starts")
        lines.append(Line(start_ticks=start, end_ticks=end, text=str(unit.get("text", "")).strip()))
    return lines


def timecode(ticks: int) -> str:
    """`H:MM:SS.mmm`, which is what a person types back into a player."""

    millis = round(ticks * 1000 / TICKS_PER_SECOND)
    seconds, millis = divmod(millis, 1000)
    minutes, seconds = divmod(seconds, 60)
    hours, minutes = divmod(minutes, 60)
    return f"{hours}:{minutes:02d}:{seconds:02d}.{millis:03d}"


def render_transcript(item_id: str, lines: Sequence[Line]) -> str:
    """The readable half of a worksheet."""

    out = [
        f"# {item_id}",
        "",
        "Every sentence the evidence index found, with the tick each one starts",
        "and ends at. Copy the ticks — not the timecode — into the annotation:",
        "the timecode is rounded for reading and the ticks are exact.",
        "",
        "| start | end | start_ticks | end_ticks | text |",
        "| --- | --- | --- | --- | --- |",
    ]
    for line in lines:
        text = line.text.replace("|", r"\|")
        out.append(
            f"| {timecode(line.start_ticks)} | {timecode(line.end_ticks)} "
            f"| {line.start_ticks} | {line.end_ticks} | {text} |"
        )
    out.append("")
    return "\n".join(out)


def skeleton(
    source_fingerprint: str,
    annotator_id: str,
    annotated_unix_millis: int,
    item_id: str | None = None,
) -> dict[str, Any]:
    """An annotation document with nothing decided yet.

    `moments` is an empty list rather than an omitted key: an empty list is a
    real answer — a recording with nothing worth clipping — and leaving the key
    out would make "not yet annotated" and "nothing here" the same document.
    """

    document: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "source_fingerprint": source_fingerprint,
        "annotator_id": annotator_id,
        "annotated_unix_millis": annotated_unix_millis,
        "timebase": {"num": 1, "den": TICKS_PER_SECOND},
        "moments": [],
        "exclusions": [],
    }
    if item_id:
        document["item_id"] = item_id
    return document


def write_worksheet(
    directory: Path,
    item_id: str,
    source_fingerprint: str,
    annotator_id: str,
    annotated_unix_millis: int,
    lines: Sequence[Line],
) -> tuple[Path, Path]:
    """Write both halves. Never overwrites an annotation somebody has filled in."""

    directory.mkdir(parents=True, exist_ok=True)
    transcript_path = directory / f"{item_id}.transcript.md"
    annotation_path = directory / f"{item_id}.annotation.json"
    transcript_path.write_text(render_transcript(item_id, lines), encoding="utf-8")
    if annotation_path.exists():
        # Regenerating a worksheet is normal — the transcript changes when the
        # recognizer is re-pinned — and losing an afternoon of annotation to it
        # would not be.
        return transcript_path, annotation_path
    document = skeleton(source_fingerprint, annotator_id, annotated_unix_millis, item_id)
    annotation_path.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    return transcript_path, annotation_path
