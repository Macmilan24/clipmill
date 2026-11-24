"""Leaving a passage out of an accepted clip by meaning, only when measured checks allow it.

Code lists the passages that may go: whole sentences inside one speaker's
turn, between two cut points, with nothing kept around them pointing at
them. The planner may choose one per clip that wanders from the clip's
point. That choice counts only if every instrument that passed its own
self-check on this recording agrees that nothing is left dangling:

* the likelihood meter: how much more predictable the line after the
  passage is with it than without it. A line that relies on what was cut
  becomes surprising.
* the judge: whether the edited lines leave a viewer wondering what
  something refers to.

Each instrument is first measured on removals whose effect the recording
shows: taking out a question before its answer, or a line the speaker then
points back at, leaves something dangling; taking out filler or an abandoned
start does not. An instrument is used only if it orders those right. The
meter is cheap and was the stronger of the two in testing, so the judge is
measured only when there is no meter or the meter fails. With no instrument
passing, nothing is left out. The critic then reviews the shorter clip like
any other.
"""

from __future__ import annotations

import copy
import re
from typing import Literal

from pydantic import create_model

from .inference import prompt_digest, prompt_for
from .judging import Asker, Book, Unanswered, best_separation, median, separation
from .planning import SENTENCE_END, Reply, ShortText, geometry_id

#: Known removals of each kind measured, at most and at least.
MAX_CASES = 8
MIN_CASES = 3
#: Known removals of each kind the judge is asked about.
JUDGE_CASES = 5
#: The share of (dangling, intact) pairs an instrument must order right.
MIN_SEPARATION = 0.8
#: Lines of context before a passage that the instruments see.
CONTEXT_LINES = 3
#: As in the cutter: the most of a moment a clip may leave out, and the
#: least distance between two removals.
MAX_CUT_SHARE = 0.35
REMOVAL_SPACING_TICKS = 15 * 90_000

FILLER = {
    "um", "uh", "er", "erm", "ah", "hmm", "mm", "like", "you", "know", "i", "mean", "yeah",
    "yes", "right", "okay", "ok", "sure",
}  # fmt: skip
#: Openings that can only point back at what came just before them. A bare
#: "that" or "this" so often points further back, past the passage, that a
#: removal before one is no known case either way.
POINTERS = ("that's why", "that is why", "this is why", "which is why", "that's what", "as i said")


def tokens(text: str) -> list[str]:
    return [w for w in (re.sub(r"[^\w']", "", word).lower() for word in text.split()) if w]


def fragment(text: str) -> bool:
    """An abandoned start: no closing punctuation, or only a few words."""
    return not SENTENCE_END.search(text.strip()) or len(tokens(text)) <= 4


def spread(items: list, count: int) -> list:
    if len(items) <= count:
        return list(items)
    step = len(items) / count
    return [items[int(step * i + step / 2)] for i in range(count)]


class Lines:
    """The script's sentences in source order, as the instruments read them."""

    def __init__(self, script: dict):
        self.units = sorted(script["units"], key=lambda u: (u["start_ticks"], u["index"]))
        self.place = {unit["index"]: place for place, unit in enumerate(self.units)}

    def text(self, place: int) -> str:
        unit = self.units[place]
        return f"{unit.get('speaker') or '?'}: {unit['text'].strip()}"

    def case(self, first: int, last: int, expected: str) -> dict | None:
        if first < 1 or last + 1 >= len(self.units):
            return None
        return {
            "first_unit": f"s{self.units[first]['index']}",
            "last_unit": f"s{self.units[last]['index']}",
            "expected": expected,
            "before": [self.text(p) for p in range(max(0, first - CONTEXT_LINES), first)],
            "passage": [self.text(p) for p in range(first, last + 1)],
            "after": self.text(last + 1),
        }


def removal_cases(lines: Lines) -> list[dict]:
    """Removals whose effect the recording shows, as instrument cases."""
    intact, dangling = [], []
    units = lines.units
    for place in range(1, len(units) - 1):
        unit, after = units[place], units[place + 1]
        words, text = tokens(unit["text"]), unit["text"].strip()
        same_voice = bool(unit.get("speaker")) and unit.get("speaker") == after.get("speaker")
        # Filler that stands alone between finished sentences ("Um, yeah."),
        # not a word a recogniser split off a sentence ("you / know /
        # what.").
        only_filler = (
            bool(words)
            and all(word in FILLER for word in words)
            and bool(SENTENCE_END.search(text))
            and bool(SENTENCE_END.search(units[place - 1]["text"].strip()))
            and not after["text"].strip()[:1].islower()
        )
        restart = (
            same_voice
            and fragment(text)
            and len(words) >= 2
            and tokens(after["text"])[:2] == words[:2]
        )
        answered = text.endswith("?") and not same_voice and bool(after.get("speaker"))
        pointed_at = (
            same_voice
            and bool(SENTENCE_END.search(text))
            and after["text"].strip().lower().startswith(POINTERS)
        )
        if only_filler or restart:
            intact.append((place, "intact"))
        elif answered or pointed_at:
            dangling.append((place, "dangling"))
    cases = []
    for place, kind in spread(dangling, MAX_CASES) + spread(intact, MAX_CASES):
        if case := lines.case(place, place, kind):
            cases.append(case)
    return cases


def check_of(cases: list[dict], scores: list[float]) -> tuple[dict, float | None]:
    """The check document for one instrument, and its threshold if it passed:
    midway between its median scores for the two kinds."""
    high = [s for case, s in zip(cases, scores, strict=True) if case["expected"] == "dangling"]
    low = [s for case, s in zip(cases, scores, strict=True) if case["expected"] == "intact"]
    check: dict = {
        "passed": False,
        "cases": [
            {
                "first_unit": case["first_unit"],
                "last_unit": case["last_unit"],
                "expected": case["expected"],
                "score": round(score, 4),
            }
            for case, score in zip(cases, scores, strict=True)
        ],
    }
    if len(high) < MIN_CASES or len(low) < MIN_CASES:
        return check, None
    check["separation"] = round(separation(high, low), 4)
    check["passed"] = check["separation"] >= MIN_SEPARATION
    return check, (median(high) + median(low)) / 2 if check["passed"] else None


def mean(values: list[float]) -> float:
    return sum(values) / len(values)


def spoken(lines: list[str]) -> list[str]:
    """Lines as "Speaker: words", with the speaker taken off."""
    return [line.split(": ", 1)[1] if ": " in line else line for line in lines]


class Gates:
    """The instruments that may confirm a removal, measured once per stage."""

    def __init__(self, runtime, script, identity, traces, book=None):
        self.runtime = runtime
        self.lines = Lines(script)
        self.book = book if book is not None else Book()
        self.ask = Asker(
            runtime,
            identity,
            traces,
            "judge.removal",
            f"judge.removal.v1/{prompt_digest('judge.removal')}",
            self.book,
        )
        self.checks: dict[str, dict] = {}
        self.thresholds: dict[str, float] = {}
        self.measured = False

    def meter(self, before: list[str], passage: list[str], after: str) -> float:
        """How much more predictable the next line is with the passage than
        without it, in log probability per token. The meter reads the words
        alone: with speaker names, leaving out another voice's "Yeah." would
        change who seems to speak next, and that would swamp what it measures."""
        before, passage, after = spoken(before), spoken(passage), spoken([after])[0]
        kept = self.runtime.continuation_logprobs("\n".join([*before, *passage]), after)
        cut = self.runtime.continuation_logprobs("\n".join(before), after)
        return mean(kept) - mean(cut)

    @staticmethod
    def question(before: list[str], passage: list[str], after: str) -> str:
        """The removal as a reader sees it: the kept lines around the passage,
        and the passage itself, so the question is about what the removal
        takes away rather than about everything the lines leave unsaid."""
        return prompt_for("judge.removal").format(
            before="\n".join(before[-2:]), passage="\n".join(passage), after=after
        )

    def judge(self, before: list[str], passage: list[str], after: str) -> float:
        return self.ask(self.question(before, passage, after))

    def note(self, before: list[str], passage: list[str], after: str) -> None:
        """List a removal question for a connected agent without asking it."""
        self.book.note("judge.removal", self.question(before, passage, after))

    def ready(self) -> bool:
        """Whether any instrument passed. Measured the first time it is asked."""
        if not self.measured:
            self.measured = True
            self._measure()
        return bool(self.thresholds)

    def _measure(self):
        cases = [case for case in removal_cases(self.lines) if case["before"]]
        if not cases or self.runtime is None:
            return
        # The judge costs a reasoning budget per question: a few cases of
        # each kind, alternately, stopping once the bar is out of reach.
        kinds = {
            kind: [c for c in cases if c["expected"] == kind][:JUDGE_CASES]
            for kind in ("dangling", "intact")
        }
        order = [
            case for pair in zip(kinds["dangling"], kinds["intact"], strict=False) for case in pair
        ]
        # A connected agent that answered every known case is the judge;
        # otherwise the local model is, measured the same way.
        self.ask.agent = self.book.covers(
            "judge.removal",
            [self.question(c["before"], c["passage"], c["after"]) for c in order],
        )
        if getattr(self.runtime, "has_meter", False):
            scores = [self.meter(c["before"], c["passage"], c["after"]) for c in cases]
            self.checks["meter"], threshold = check_of(cases, scores)
            if threshold is not None:
                self.thresholds["meter"] = threshold
                if not self.ask.agent:
                    return
        planned = (len(kinds["dangling"]), len(kinds["intact"]))
        asked, scores = [], []
        for case in order:
            asked.append(case)
            scores.append(self.judge(case["before"], case["passage"], case["after"]))
            high = [s for c, s in zip(asked, scores, strict=True) if c["expected"] == "dangling"]
            low = [s for c, s in zip(asked, scores, strict=True) if c["expected"] == "intact"]
            if best_separation(high, low, planned) < MIN_SEPARATION:
                break
        self.checks["judge"], threshold = check_of(asked, scores)
        if threshold is not None:
            self.thresholds["judge"] = threshold

    def allows(self, before: list[str], passage: list[str], after: str) -> bool:
        """Whether every instrument that passed says nothing is left dangling."""
        if not self.ready():
            return False
        if "meter" in self.thresholds and (
            self.meter(before, passage, after) >= self.thresholds["meter"]
        ):
            return False
        if "judge" not in self.thresholds:
            return True
        try:
            return self.judge(before, passage, after) < self.thresholds["judge"]
        except Unanswered:
            # Left open for the agent's next answers; nothing goes meanwhile.
            return False


def options(plan, transcript, passages, length):
    """The listed passages this accepted single-part clip could do without."""
    if len(plan["parts"]) != 1 or plan["form"] not in ("straight", "tightened"):
        return [], []
    part = plan["parts"][0]
    played = transcript.evidence(plan)["playback"]
    heard = [line["unit"] for line in played]
    cited = {item for item in plan.get("evidence", []) if re.fullmatch(r"s\d+", item)}
    removed = part.get("removed", [])
    span = part["end_ticks"] - part["start_ticks"]
    already = sum(r["end_ticks"] - r["start_ticks"] for r in removed)
    found = []
    for passage in passages:
        first, last = passage["first_unit"], passage["last_unit"]
        if first not in heard or last not in heard:
            continue
        a, b = heard.index(first), heard.index(last)
        units = heard[a : b + 1]
        start, end = passage["start_ticks"], passage["end_ticks"]
        if (
            a == 0
            or b + 1 >= len(heard)
            or start < part["start_ticks"]
            or end > part["end_ticks"]
            or cited & set(units)
            or any(
                start < r["end_ticks"] + REMOVAL_SPACING_TICKS
                and r["start_ticks"] < end + REMOVAL_SPACING_TICKS
                for r in removed
            )
            or plan["duration_ticks"] - (end - start) < length["min_ticks"]
            or already + end - start > MAX_CUT_SHARE * span
        ):
            continue
        found.append({**passage, "units": units, "before": a, "after": b + 1})
    return found, played


def shortened(plan, passage, transcript):
    """The same clip with the passage left out as a tangent."""
    tightened = copy.deepcopy(plan)
    part = tightened["parts"][0]
    cut = {
        "start_ticks": passage["start_ticks"],
        "end_ticks": passage["end_ticks"],
        "reason": "tangent",
    }
    part["removed"] = sorted([*part.get("removed", []), cut], key=lambda r: r["start_ticks"])
    part["units"] = [
        line["unit"] for line in transcript.evidence(tightened)["playback"]
    ] or part.get("units", [])
    tightened["form"] = "tightened"
    tightened["duration_ticks"] -= passage["end_ticks"] - passage["start_ticks"]
    tightened["plan_id"] = geometry_id(tightened)
    return tightened


def tighten(ctx, plan, transcript, passages, gates, length):
    """At most one passage the planner chose and the instruments allow,
    left out of an accepted clip; `None` when nothing should go."""
    found, played = options(plan, transcript, passages, length)
    if not found:
        return None
    text = [f"{line['speaker'] or '?'}: {line['text'].strip()}" for line in played]
    for passage in found:
        # Every passage is listed, so a connected agent can answer for any
        # the planner might choose once its answers make the judge usable.
        gates.note(
            text[max(0, passage["before"] - CONTEXT_LINES) : passage["before"]],
            text[passage["before"] : passage["after"]],
            text[passage["after"]],
        )
    if not gates.ready():
        return None
    names = {f"p{i}": passage for i, passage in enumerate(found, start=1)}
    reply = create_model(
        "TightenChoice",
        __base__=Reply,
        choice=(Literal[tuple([*names, "none"])], ...),
        reason=(ShortText, ...),
    )
    answer, failure = ctx.ask(
        "tighten",
        {
            "title": plan["title"],
            "spoken_lines": played,
            "passages": [{"id": name, "units": p["units"]} for name, p in names.items()],
        },
        reply,
    )
    if failure or not answer or answer["choice"] not in names:
        return None
    passage = names[answer["choice"]]
    before = text[max(0, passage["before"] - CONTEXT_LINES) : passage["before"]]
    if not gates.allows(before, text[passage["before"] : passage["after"]], text[passage["after"]]):
        return None
    return shortened(plan, passage, transcript)


__all__ = ["Gates", "Lines", "check_of", "options", "removal_cases", "shortened", "tighten"]
