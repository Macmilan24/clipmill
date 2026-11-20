"""Narrow questions before planning, asked only of a model that shows it can answer them.

The judge decides one thing: whether a clip that ends while the same speaker
goes on has left them mid-thought. Text rules already refuse endings before
"and…", "but…", "that…" and the lower-case rest of a sentence; what is left
("Not yet.", "There's a yin and the yang…") needs judgement.

It is model-independent by construction. Before asking about any candidate,
the stage measures the model on endings whose answer the recording itself
gives: the same speaker carrying on with a connective (the thought goes on),
and another speaker taking over with a question or after a pause, or the
close of a chapter (the thought is over). Each question is answered after a short reasoning
budget, and the probability of "yes" is read from the model rather than
sampled. A model that cannot tell the known cases apart is never asked about
a candidate; one that can is given a threshold between its own scores for
the two kinds. A stronger model passes more easily and draws a sharper line,
with no change here.
"""

from __future__ import annotations

import time

from clipmill_worker_sdk import LeaseCancelled

from .inference import prompt_digest, prompt_for
from .planning import SENTENCE_END, Transcript, ending_unit, planning_pool, source_options

#: How long the model may reason before each answer.
THINK_TOKENS = 512
#: Known cases of each kind in the self-check. With four of each, a judge
#: that guesses passes about one time in ten.
CASES_PER_KIND = 4
#: The most candidate endings judged in one analysis, in the order the planner reads them.
MAX_ENDINGS = 10
#: How well the self-check must order the two kinds: the share of
#: (finished, continuing) pairs it scores the right way round.
MIN_SEPARATION = 0.8
#: Lines further apart than this are not one speaker carrying on.
ADJACENT_TICKS = 3 * 90_000
#: A turn without a question, or a chapter, counts as closed after this pause.
CLOSING_PAUSE_TICKS = 63_000

CONNECTIVES = {
    "and", "but", "because", "cause", "'cause", "or", "which", "although",
    "though", "whereas", "plus",
}  # fmt: skip


def first_word(text: str) -> str:
    words = text.split()
    return "".join(c for c in words[0] if c.isalnum() or c == "'").lower() if words else ""


class Recording:
    """The script's sentences in order, with what a question about one shows."""

    def __init__(self, script: dict):
        self.units = {unit["index"]: unit for unit in script["units"]}
        self.order = sorted(self.units, key=lambda i: (self.units[i]["start_ticks"], i))
        self.place = {index: place for place, index in enumerate(self.order)}

    def line(self, index: int) -> str:
        unit = self.units[index]
        return f"{unit.get('speaker') or '?'}: {unit['text']}"

    def next(self, index: int) -> int | None:
        place = self.place[index] + 1
        return self.order[place] if place < len(self.order) else None

    def pause_after(self, index: int) -> int:
        following = self.next(index)
        if following is None:
            return 0
        return self.units[following]["start_ticks"] - self.units[index]["end_ticks"]

    def same_voice_goes_on(self, index: int) -> bool:
        following = self.next(index)
        if following is None:
            return False
        speaker = self.units[index].get("speaker")
        return (
            bool(speaker)
            and speaker == self.units[following].get("speaker")
            and self.pause_after(index) <= ADJACENT_TICKS
        )

    def closes_sentence(self, index: int) -> bool:
        return bool(SENTENCE_END.search(self.units[index]["text"].rstrip()))

    def question(self, index: int) -> str:
        place = self.place[index]
        clip = "\n".join(self.line(i) for i in self.order[max(0, place - 2) : place + 1])
        return prompt_for("judge").format(clip=clip, next=self.line(self.next(index)))


def spread(items: list, count: int) -> list:
    """`count` items evenly spaced through the list, so the check covers the recording."""
    if len(items) <= count:
        return list(items)
    step = len(items) / count
    return [items[int(step * i + step / 2)] for i in range(count)]


def known_endings(recording: Recording, chapters=()) -> list[tuple[int, str]]:
    """Endings whose answer the recording gives, as (sentence, expected)."""
    goes_on, closed = [], []
    for index in recording.order:
        following = recording.next(index)
        if following is None or not recording.closes_sentence(index):
            continue
        unit, after = recording.units[index], recording.units[following]
        if recording.same_voice_goes_on(index):
            if first_word(after["text"]) in CONNECTIVES:
                goes_on.append((index, "continues"))
        elif (
            unit.get("speaker")
            and after.get("speaker")
            and unit["speaker"] != after["speaker"]
            and (
                after["text"].rstrip().endswith("?")
                or recording.pause_after(index) >= CLOSING_PAUSE_TICKS
            )
        ):
            # Another voice takes over after a closed sentence, with a
            # question of its own or after a pause: not an interruption.
            closed.append((index, "finished"))
    if len(closed) < CASES_PER_KIND:
        # A monologue has no turns: the close of a chapter, followed by a
        # pause and a fresh sentence, is the same kind of known ending.
        for chapter in chapters:
            index = int(chapter["last_unit"][1:])
            following = recording.next(index) if index in recording.units else None
            if (
                following is not None
                and recording.closes_sentence(index)
                and recording.pause_after(index) >= CLOSING_PAUSE_TICKS
                and first_word(recording.units[following]["text"]) not in CONNECTIVES
                and all(index != known for known, _ in closed)
            ):
                closed.append((index, "finished"))
    return spread(goes_on, CASES_PER_KIND) + spread(closed, CASES_PER_KIND)


def median(values: list[float]) -> float:
    ordered = sorted(values)
    middle = len(ordered) // 2
    return ordered[middle] if len(ordered) % 2 else (ordered[middle - 1] + ordered[middle]) / 2


def separation(high: list[float], low: list[float]) -> float:
    pairs = [(h > o) + 0.5 * (h == o) for h in high for o in low]
    return sum(pairs) / len(pairs) if pairs else 0.0


def endings_to_judge(recording, script, candidates, memory, brief, count) -> list[int]:
    """The last sentence of every option the planner will be offered where
    the same speaker goes on, in the order it reads them, each once."""
    transcript = Transcript(script)
    found: list[int] = []
    for source in planning_pool(candidates, memory, brief, count):
        for plan in source_options(source, transcript, candidates, brief).values():
            unit = ending_unit(plan, transcript)
            index = int(unit[1:]) if unit else None
            if (
                index in recording.units
                and index not in found
                and recording.same_voice_goes_on(index)
            ):
                found.append(index)
    return found[:MAX_ENDINGS]


def judge(runtime, script, candidates, memory, *, brief, count, identity, traces, progress):
    """The self-check, then verdicts on candidate endings when it passes."""
    recording = Recording(script)
    version = f"judge.v1/{prompt_digest('judge')}"

    def ask(index: int) -> float:
        started = time.monotonic()
        trace = {
            "index": len(traces),
            "job": "judge.ending",
            **identity,
            "prompt_version": version,
            "request_text": recording.question(index),
            "started_unix_millis": time.time_ns() // 1_000_000,
            "tokens": {"input": 0, "output": 0},
            "outcome": "failed",
        }
        try:
            probability, thinking = runtime.yes_probability(trace["request_text"], THINK_TOKENS)
        except LeaseCancelled:
            trace["outcome"] = "cancelled"
            raise
        except (RuntimeError, TimeoutError, ValueError) as error:
            trace["detail"] = str(error)[:1000] or type(error).__name__
            raise
        finally:
            trace["duration_millis"] = int((time.monotonic() - started) * 1000)
            traces.append(trace)
        trace.update(outcome="answered", response_text=f"{thinking}\n[yes: {probability:.4f}]")
        return probability

    candidates_asked = endings_to_judge(recording, script, candidates, memory, brief, count)
    # With nothing to decide, the model is not measured either.
    known = known_endings(recording, memory.get("chapters", [])) if candidates_asked else []
    total = len(known) + len(candidates_asked)
    scores: dict[int, float] = {}
    for done, (index, _) in enumerate(known):
        progress("judge_questions", done, total)
        scores[index] = ask(index)
    finished = [scores[i] for i, kind in known if kind == "finished"]
    goes_on = [scores[i] for i, kind in known if kind == "continues"]
    enough = len(finished) >= 3 and len(goes_on) >= 3
    check: dict = {
        "passed": False,
        "cases": [
            {"unit": f"s{i}", "expected": kind, "score": round(2 * scores[i] - 1, 4)}
            for i, kind in known
        ],
    }
    endings = []
    if enough:
        check["separation"] = round(separation(finished, goes_on), 4)
        check["passed"] = check["separation"] >= MIN_SEPARATION
        if check["passed"]:
            # Models differ in how sure they sound, so the line between the
            # two answers is drawn from this model's own scores; medians, so
            # one odd case does not move it.
            threshold = (median(finished) + median(goes_on)) / 2
            for done, index in enumerate(candidates_asked, start=len(known)):
                progress("judge_questions", done, total)
                probability = scores[index] if index in scores else ask(index)
                endings.append(
                    {
                        "after_unit": f"s{index}",
                        "score": round(2 * probability - 1, 4),
                        "verdict": "finished" if probability >= threshold else "continues",
                    }
                )
    progress("judge_questions", total, total)
    return {
        "judge": {"think_tokens": THINK_TOKENS, "prompt_version": version},
        "self_check": {
            "endings": check,
            # No stage proposes a removal that needs judging yet, so the
            # removal check is not asked and stays closed.
            "removals": {"passed": False, "cases": []},
        },
        "endings": endings,
    }


__all__ = ["Recording", "endings_to_judge", "judge", "known_endings", "separation"]
