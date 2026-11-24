"""An edited clip set against its plain version, by a judge that first shows
it can tell a whole passage from a broken one.

An edit (a cold open, a stitch, a tightened moment) is only worth shipping
when it is a better short than the same moment played whole. The judge sees
both versions exactly as a viewer hears them and is asked whether version A
is the better short, once with the edit as A and once with the plain version
as A. The edit ships only when it wins both ways; a judge's habit of
preferring whichever version comes first cancels out.

Before any edit is compared, the judge is measured on pairs whose answer the
recording itself gives: a passage that ends on a finished thought against
the same passage stopped while its speaker goes on with "and…" or "but…",
and a passage against the same passage opened on a line that points back at
the one before it ("That's why…"). A judge that cannot tell those apart is
never asked about an edit, and an edit whose plain version passed review then
ships as the plain version: an unmeasured edit is not shipped on trust. A
stronger judge, the local model or a connected agent, passes more easily and
lets more edits through, with no change here.
"""

from __future__ import annotations

import copy

from .inference import prompt_digest, prompt_for
from .judging import (
    CONNECTIVES,
    Asker,
    Book,
    Recording,
    Unanswered,
    best_separation,
    first_word,
    known_endings,
    median,
    separation,
)
from .planning import Transcript, geometry_id

#: Known pairs measured, at most and at least. Each is asked in both orders.
MAX_PAIRS = 4
MIN_PAIRS = 3
#: The share of (whole first, broken first) question pairs a judge must
#: score the right way round.
MIN_SEPARATION = 0.8
#: The most edits the local model compares in one analysis. A connected
#: agent answers every one.
MAX_COMPARISONS = 12
#: How long a known passage may run, from its first line to its last.
MIN_PASSAGE_TICKS = 6 * 90_000
MAX_PASSAGE_TICKS = 60 * 90_000
#: How far back from a cut-off ending its passage may open.
MAX_LEAD_TICKS = 25 * 90_000
#: Openings that point back at the line before them.
POINTS_BACK = (
    "that's why",
    "that is why",
    "this is why",
    "which is why",
    "that's what",
    "as i said",
    "like i said",
)


def edited(plan: dict) -> bool:
    """Whether a plan plays anything other than one stretch of the recording."""
    parts = plan["parts"]
    return len(parts) > 1 or any(part.get("removed") for part in parts)


def plain_version(plan: dict, transcript: Transcript) -> dict | None:
    """The same moment played whole: the edit's body, or its longest part,
    with nothing left out. `None` for a plan that is not an edit, and for a
    theme montage, which a person asked for as a montage."""
    if not edited(plan) or plan["form"] == "theme_montage":
        return None
    parts = plan["parts"]
    body = next((part for part in parts if part["role"] == "body"), None) or max(
        parts, key=lambda part: part["end_ticks"] - part["start_ticks"]
    )
    units = [line["unit"] for line in transcript.segment(body["start_ticks"], body["end_ticks"])]
    if not units:
        return None
    whole = {
        "part_id": "p1",
        "role": "body",
        "start_ticks": body["start_ticks"],
        "end_ticks": body["end_ticks"],
        "units": units,
    }
    plain = {
        **copy.deepcopy(plan),
        "form": "straight",
        "parts": [whole],
        "joins": [],
        "duration_ticks": body["end_ticks"] - body["start_ticks"],
        "evidence": units,
    }
    plain["plan_id"] = geometry_id(plain)
    return plain


def heard(lines: list[dict]) -> str:
    """Lines as a viewer hears them, one turn per speaker."""
    turns: list[list[str]] = []
    for line in lines:
        speaker = line.get("speaker") or "?"
        if turns and turns[-1][0] == speaker:
            turns[-1][1] += " " + line["text"].strip()
        else:
            turns.append([speaker, line["text"].strip()])
    return "\n".join(f"{speaker}: {text}" for speaker, text in turns)


def question(a: str, b: str) -> str:
    return prompt_for("judge.compare").format(a=a, b=b)


class Passages:
    """Known pairs of a whole and a broken passage, from the recording."""

    def __init__(self, script: dict, chapters=()):
        self.recording = Recording(script)
        self.chapters = chapters

    def lines(self, first: int, last: int) -> list[dict]:
        order = self.recording.order
        span = order[self.recording.place[first] : self.recording.place[last] + 1]
        return [
            {
                "speaker": self.recording.units[i].get("speaker"),
                "text": self.recording.units[i]["text"],
            }
            for i in span
        ]

    def length(self, first: int, last: int) -> int:
        units = self.recording.units
        return units[last]["end_ticks"] - units[first]["start_ticks"]

    def opens_cleanly(self, index: int) -> bool:
        """A line a short may open on: the line before closed a sentence and
        this one does not carry on or point back at it."""
        place = self.recording.place[index]
        text = self.recording.units[index]["text"].strip()
        before = self.recording.order[place - 1] if place else None
        return (
            (before is None or self.recording.closes_sentence(before))
            and not text[:1].islower()
            and first_word(text) not in CONNECTIVES
            and not text.lower().startswith(POINTS_BACK)
        )

    def opening_before(self, end: int) -> int | None:
        """The earliest clean opening in the same speaker's turn, within reach."""
        units, order = self.recording.units, self.recording.order
        speaker = units[end].get("speaker")
        best = None
        for place in range(self.recording.place[end], -1, -1):
            index = order[place]
            if units[index].get("speaker") != speaker:
                break
            if units[end]["start_ticks"] - units[index]["start_ticks"] > MAX_LEAD_TICKS:
                break
            if self.opens_cleanly(index):
                best = index
        return best

    def cut_off(self) -> list[dict]:
        """A passage ending on a finished thought, against the same passage
        stopped where its speaker carries on."""
        known = known_endings(self.recording, self.chapters, per_kind=None)
        finished = sorted(i for i, kind in known if kind == "finished")
        pairs = []
        for index, kind in known:
            if kind != "continues":
                continue
            first = self.opening_before(index)
            end = next(
                (f for f in finished if self.recording.place[f] > self.recording.place[index]), None
            )
            if first is None or end is None:
                continue
            if (
                self.length(first, index) < MIN_PASSAGE_TICKS
                or self.length(first, end) > MAX_PASSAGE_TICKS
            ):
                continue
            pairs.append(
                {"kind": "cut_off_ending", "whole": (first, end), "broken": (first, index)}
            )
        return pairs

    def dangling(self) -> list[dict]:
        """A passage, against the same passage opened on a line that points
        back at the one before it."""
        known = known_endings(self.recording, self.chapters, per_kind=None)
        finished = sorted(i for i, kind in known if kind == "finished")
        units, order = self.recording.units, self.recording.order
        pairs = []
        for place in range(1, len(order)):
            index, before = order[place], order[place - 1]
            text = units[index]["text"].strip().lower()
            if (
                not text.startswith(POINTS_BACK)
                or units[index].get("speaker") != units[before].get("speaker")
                or not self.opens_cleanly(before)
            ):
                continue
            end = next((f for f in finished if self.recording.place[f] >= place), None)
            if end is None or self.length(before, end) > MAX_PASSAGE_TICKS:
                continue
            if self.length(index, end) < MIN_PASSAGE_TICKS:
                continue
            pairs.append({"kind": "dangling_start", "whole": (before, end), "broken": (index, end)})
        return pairs

    def pairs(self) -> list[dict]:
        """Up to MAX_PAIRS known pairs, both kinds where the recording has them,
        spread through the recording."""
        cut_off, dangling = spread(self.cut_off(), MAX_PAIRS), spread(self.dangling(), MAX_PAIRS)
        chosen = []
        while len(chosen) < MAX_PAIRS and (cut_off or dangling):
            for kind in (cut_off, dangling):
                if kind and len(chosen) < MAX_PAIRS:
                    chosen.append(kind.pop(0))
        return chosen


def spread(items: list, count: int) -> list:
    if len(items) <= count:
        return list(items)
    step = len(items) / count
    return [items[int(step * i + step / 2)] for i in range(count)]


class Versus:
    """The comparison judge for one review stage, measured the first time an
    edit needs it."""

    def __init__(self, runtime, script, chapters, identity, traces, book=None):
        self.runtime = runtime
        self.passages = Passages(script, chapters)
        self.book = book if book is not None else Book()
        self.ask = Asker(
            runtime,
            identity,
            traces,
            "judge.compare",
            f"judge.compare.v1/{prompt_digest('judge.compare')}",
            self.book,
        )
        self.check: dict | None = None
        self.threshold: float | None = None
        self.compared = 0

    def texts(self, pair: dict, whole_first: bool) -> str:
        whole = heard(self.passages.lines(*pair["whole"]))
        broken = heard(self.passages.lines(*pair["broken"]))
        return question(whole, broken) if whole_first else question(broken, whole)

    def ready(self) -> bool:
        if self.check is None:
            self.check = self._measure()
        return self.threshold is not None

    def _measure(self) -> dict:
        pairs = self.passages.pairs() if self.runtime is not None else []
        questions = [self.texts(pair, first) for pair in pairs for first in (True, False)]
        self.ask.agent = self.book.covers("judge.compare", questions)
        check: dict = {
            "passed": False,
            "answered_by": "agent" if self.ask.agent else "model",
            "cases": [],
        }
        whole_first, broken_first = [], []
        for pair in pairs:
            high = self.ask(self.texts(pair, True))
            low = self.ask(self.texts(pair, False))
            whole_first.append(high)
            broken_first.append(low)
            check["cases"].append(
                {
                    "kind": pair["kind"],
                    "whole": span(pair["whole"]),
                    "broken": span(pair["broken"]),
                    "whole_first": round(high, 4),
                    "broken_first": round(low, 4),
                }
            )
            if (
                best_separation(whole_first, broken_first, (len(pairs), len(pairs)))
                < MIN_SEPARATION
            ):
                break
        if len(whole_first) >= MIN_PAIRS and len(whole_first) == len(pairs):
            check["separation"] = round(separation(whole_first, broken_first), 4)
            check["passed"] = check["separation"] >= MIN_SEPARATION
            if check["passed"]:
                self.threshold = (median(whole_first) + median(broken_first)) / 2
        return check

    def note(self, edit: dict, plain: dict, transcript: Transcript) -> None:
        """List both questions about an edit for a connected agent."""
        a, b = played(edit, transcript), played(plain, transcript)
        self.book.note("judge.compare", question(a, b))
        self.book.note("judge.compare", question(b, a))

    def prefers_edit(self, edit: dict, plain: dict, transcript: Transcript) -> dict:
        """The comparison's verdict, as the quality report records it."""
        self.note(edit, plain, transcript)
        verdict = {"plain_plan_id": plain["plan_id"]}
        if not self.ready():
            return {**verdict, "kept": "plain", "reason": "not_measured"}
        if self.compared >= MAX_COMPARISONS and not self.ask.agent:
            return {**verdict, "kept": "plain", "reason": "not_measured"}
        self.compared += 1
        a, b = played(edit, transcript), played(plain, transcript)
        try:
            edit_first = self.ask(question(a, b))
            plain_first = self.ask(question(b, a))
        except Unanswered:
            return {**verdict, "kept": "plain", "reason": "awaiting_agent"}
        wins = edit_first >= self.threshold and plain_first < self.threshold
        return {
            **verdict,
            "kept": "edit" if wins else "plain",
            "reason": "edit_preferred" if wins else "plain_preferred",
            "edit_first": round(edit_first, 4),
            "plain_first": round(plain_first, 4),
        }


def span(pair: tuple[int, int]) -> dict:
    return {"first_unit": f"s{pair[0]}", "last_unit": f"s{pair[1]}"}


def played(plan: dict, transcript: Transcript) -> str:
    return heard(transcript.evidence(plan)["playback"])


__all__ = ["Passages", "Versus", "edited", "heard", "plain_version"]
