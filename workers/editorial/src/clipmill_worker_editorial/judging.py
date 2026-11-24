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

import hashlib
import time

from clipmill_worker_sdk import LeaseCancelled

from .inference import prompt_digest, prompt_for
from .planning import SENTENCE_END, Transcript, ending_unit, planning_pool, source_options

#: How long the model may reason before each answer.
THINK_TOKENS = 512
#: Known cases of each kind in the self-check. With five of each, a judge
#: that guesses passes about one time in thirteen.
CASES_PER_KIND = 5
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


def best_separation(high: list[float], low: list[float], planned: tuple[int, int]) -> float:
    """The most the check can still reach: every pair that involves a case
    not yet asked counted as ordered right."""
    total = planned[0] * planned[1]
    if not total:
        return 0.0
    ordered = sum((h > o) + 0.5 * (h == o) for h in high for o in low)
    return (ordered + total - len(high) * len(low)) / total


def interleaved(known: list[tuple[int, str]]) -> list[tuple[int, str]]:
    """The known cases alternating between the two kinds, so an early stop
    has seen both."""
    goes_on = [case for case in known if case[1] == "continues"]
    closed = [case for case in known if case[1] == "finished"]
    order = []
    for place in range(max(len(goes_on), len(closed))):
        order.extend(kind[place] for kind in (goes_on, closed) if place < len(kind))
    return order


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


def question_id(kind: str, text: str) -> str:
    """A stable name for one question: the same kind and words, in any run
    of any recording, have the same id."""
    digest = hashlib.sha256(f"{kind}\n{text}".encode()).hexdigest()
    return f"q_{digest[:24]}"


class Unanswered(LookupError):
    """A question the connected agent has not answered yet."""


class Book:
    """Answers a connected agent gave to this recording's narrow questions,
    and every question this run asked or would ask, so an agent can answer
    them for the next run.

    A kind of question is put to the agent only when the agent has answered
    every known case of that kind's self-check: then the agent is measured
    on exactly what the local model would be measured on, and a judge is
    never half agent, half model. The list of questions never says which
    ones are known cases."""

    def __init__(self, answers: dict[str, float] | None = None):
        self.answers = dict(answers or {})
        self.questions: dict[str, dict] = {}

    def note(self, kind: str, text: str) -> str:
        identifier = question_id(kind, text)
        self.questions.setdefault(
            identifier, {"question_id": identifier, "kind": kind, "text": text}
        )
        return identifier

    def answer(self, kind: str, text: str) -> float | None:
        return self.answers.get(self.note(kind, text))

    def covers(self, kind: str, texts: list[str]) -> bool:
        """Whether the agent answered every one of these questions. Each is
        listed either way."""
        answers = [self.answer(kind, text) for text in texts]
        return bool(answers) and all(answer is not None for answer in answers)

    def open(self) -> list[dict]:
        """The questions noted in this run that have no answer, in the order asked."""
        return [q for q in self.questions.values() if q["question_id"] not in self.answers]


class Asker:
    """One narrow question at a time, each recorded in the stage's trace.

    With `agent` set, the answers come from the connected agent's book
    instead of the local model; a question it has not answered raises
    `Unanswered` and stays open for the next run."""

    def __init__(self, runtime, identity, traces, job, version, book=None, agent=False):
        self.runtime = runtime
        self.identity = identity
        self.traces = traces
        self.job = job
        self.version = version
        self.book = book if book is not None else Book()
        self.agent = agent

    def __call__(self, question: str) -> float:
        self.book.note(self.job, question)
        if self.agent:
            answer = self.book.answer(self.job, question)
            if answer is None:
                raise Unanswered(question)
            self.traces.append(
                {
                    "index": len(self.traces),
                    "job": self.job,
                    **self.identity,
                    "prompt_version": self.version,
                    "request_text": question,
                    "started_unix_millis": time.time_ns() // 1_000_000,
                    "duration_millis": 0,
                    "tokens": {"input": 0, "output": 0},
                    "outcome": "answered",
                    "response_text": f"answered by a connected agent\n[yes: {answer:.4f}]",
                }
            )
            return answer
        return self._ask_model(question)

    def _ask_model(self, question: str) -> float:
        started = time.monotonic()
        trace = {
            "index": len(self.traces),
            "job": self.job,
            **self.identity,
            "prompt_version": self.version,
            "request_text": question,
            "started_unix_millis": time.time_ns() // 1_000_000,
            "tokens": {"input": 0, "output": 0},
            "outcome": "failed",
        }
        try:
            probability, thinking = self.runtime.yes_probability(question, THINK_TOKENS)
        except LeaseCancelled:
            trace["outcome"] = "cancelled"
            raise
        except (RuntimeError, TimeoutError, ValueError) as error:
            trace["detail"] = str(error)[:1000] or type(error).__name__
            raise
        finally:
            trace["duration_millis"] = int((time.monotonic() - started) * 1000)
            self.traces.append(trace)
        trace.update(outcome="answered", response_text=f"{thinking}\n[yes: {probability:.4f}]")
        return probability


def ending_version() -> str:
    return f"judge.v1/{prompt_digest('judge')}"


def threshold_of(check: dict) -> float | None:
    """Where a passing judge draws the line between its two answers: midway
    between its median scores for the two kinds of known case. Models differ
    in how sure they sound, so the line comes from this model's own scores;
    medians, so one odd case does not move it."""
    if not check.get("passed"):
        return None
    by_kind: dict[str, list[float]] = {}
    for case in check["cases"]:
        by_kind.setdefault(case["expected"], []).append((case["score"] + 1) / 2)
    if not by_kind.get("finished") or not by_kind.get("continues"):
        return None
    return (median(by_kind["finished"]) + median(by_kind["continues"])) / 2


def judge(
    runtime, script, candidates, memory, *, brief, count, identity, traces, progress, book=None
):
    """The self-check, then verdicts on candidate endings when it passes.

    The judge is the local model, or the connected agent when its book
    answers every known case; either way it is measured the same way."""
    recording = Recording(script)
    version = ending_version()
    book = book if book is not None else Book()
    candidates_asked = endings_to_judge(recording, script, candidates, memory, brief, count)
    # With nothing to decide, the model is not measured either.
    known = known_endings(recording, memory.get("chapters", [])) if candidates_asked else []
    known = interleaved(known)
    agent = book.covers("judge.ending", [recording.question(i) for i, _ in known])
    for index in candidates_asked:
        book.note("judge.ending", recording.question(index))
    ask_question = Asker(runtime, identity, traces, "judge.ending", version, book, agent)

    def ask(index: int) -> float | None:
        try:
            return ask_question(recording.question(index))
        except Unanswered:
            return None

    planned = (
        sum(kind == "finished" for _, kind in known),
        sum(kind == "continues" for _, kind in known),
    )
    total = len(known) + len(candidates_asked)
    scores: dict[int, float] = {}
    for done, (index, _) in enumerate(known):
        progress("judge_questions", done, total)
        scores[index] = ask(index)
        so_far = [(i, kind) for i, kind in known if i in scores]
        if (
            best_separation(
                [scores[i] for i, kind in so_far if kind == "finished"],
                [scores[i] for i, kind in so_far if kind == "continues"],
                planned,
            )
            < MIN_SEPARATION
        ):
            # The bar is out of reach: asking the rest would only cost time.
            break
    known = [(i, kind) for i, kind in known if i in scores]
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
        threshold = threshold_of(check)
        if threshold is not None:
            for done, index in enumerate(candidates_asked, start=len(known)):
                progress("judge_questions", done, total)
                probability = scores[index] if index in scores else ask(index)
                if probability is None:
                    # The agent has not answered this one yet: no verdict.
                    continue
                endings.append(
                    {
                        "after_unit": f"s{index}",
                        "score": round(2 * probability - 1, 4),
                        "verdict": "finished" if probability >= threshold else "continues",
                    }
                )
    progress("judge_questions", total, total)
    return {
        "judge": {
            "think_tokens": 0 if agent else THINK_TOKENS,
            "prompt_version": version,
            "answered_by": "agent" if agent else "model",
        },
        "self_check": {
            "endings": check,
            # No stage proposes a removal that needs judging yet, so the
            # removal check is not asked and stays closed.
            "removals": {"passed": False, "cases": []},
        },
        "endings": endings,
    }


#: The most endings the review asks about beyond those judged before planning.
MAX_REVIEW_ENDINGS = 12


class FinalEndings:
    """The judge's verdict on each final clip's ending, for the review.

    Before planning the judge reaches only the first candidates; the planner
    may choose others, and the review may repair a plan into another ending.
    A judge that passed its self-check is asked about each of those once, on
    the same scale; one that did not pass is never asked."""

    def __init__(self, runtime, script, judgments, identity, traces, book=None):
        self.recording = Recording(script)
        check = ((judgments or {}).get("self_check") or {}).get("endings") or {}
        self.threshold = threshold_of(check)
        # The review asks whichever judge passed before planning.
        agent = ((judgments or {}).get("judge") or {}).get("answered_by") == "agent"
        self.ask = Asker(runtime, identity, traces, "judge.ending", ending_version(), book, agent)
        self.probability: dict[str, float] = {
            case["unit"]: (case["score"] + 1) / 2 for case in check.get("cases", [])
        }
        for ending in (judgments or {}).get("endings", []):
            self.probability[ending["after_unit"]] = (ending["score"] + 1) / 2
        self.asked = 0

    def cuts_off(self, unit: str | None) -> bool:
        """Whether ending after `unit` leaves its speaker mid-thought."""
        if self.threshold is None or not unit:
            return False
        index = int(unit[1:])
        if index not in self.recording.units or not self.recording.same_voice_goes_on(index):
            return False
        if unit not in self.probability:
            if self.asked >= MAX_REVIEW_ENDINGS and not self.ask.agent:
                return False
            self.asked += 1
            try:
                self.probability[unit] = self.ask(self.recording.question(index))
            except Unanswered:
                # Left open for the agent's next answers; the rules stand.
                return False
        return self.probability[unit] < self.threshold


__all__ = [
    "Book",
    "Recording",
    "Unanswered",
    "best_separation",
    "endings_to_judge",
    "judge",
    "known_endings",
    "question_id",
    "separation",
]
