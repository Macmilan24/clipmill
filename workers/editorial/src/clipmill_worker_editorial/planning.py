"""Constrained editing choices and source context for the independent join review."""

from __future__ import annotations

import bisect
import copy
import hashlib
from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field, create_model

from .inference import call

ShortText = Annotated[str, Field(min_length=1, max_length=240)]


class Reply(BaseModel):
    model_config = ConfigDict(extra="forbid")


class Technique(Reply):
    after: str = Field(min_length=1, max_length=4)
    framing: Literal["keep", "punch_in", "punch_out"]
    crossfade_ms: int = Field(ge=0, le=120)


class PlanReply(Reply):
    choice: str = Field(min_length=1, max_length=25)
    title: str = Field(max_length=120)
    hook_text: str = Field(max_length=160)
    reasons: list[ShortText] = Field(min_length=1, max_length=3)
    doubts: list[ShortText] = Field(max_length=3)
    joins: list[Technique] = Field(max_length=11)


class Transcript:
    """Source-ordered sentences. Time lookup never assumes contiguous sentence ids."""

    def __init__(self, script):
        self.units = sorted(script["units"], key=lambda u: (u["start_ticks"], u["index"]))
        self.starts = [u["start_ticks"] for u in self.units]

    def in_range(self, start, end):
        first = bisect.bisect_left(self.starts, start)
        last = bisect.bisect_left(self.starts, end)
        return [i for i in range(first, last) if self.units[i]["end_ticks"] <= end]

    def line(self, i):
        u = self.units[i]
        return {"unit": f"s{u['index']}", "speaker": u.get("speaker", ""), "text": u["text"]}

    def segment(self, start, end):
        return [self.line(i) for i in self.in_range(start, end)]

    def evidence(self, plan):
        """Played words, both source neighbours and every omitted word at a splice.

        Build from source times rather than the plan's claims about unit ids.
        A review never receives the planner's notes, event labels or taste examples.
        """
        played = []
        joins = []
        previous = None
        for part in plan["parts"]:
            start = part["start_ticks"]
            boundaries = [(r["start_ticks"], r["end_ticks"]) for r in part.get("removed", [])]
            for cut_index, (end, resume) in enumerate([*boundaries, (part["end_ticks"], None)]):
                kept = self.in_range(start, end)
                if not kept:
                    raise ValueError("An edit segment contains no complete spoken sentence.")
                first, last = kept[0], kept[-1]
                if previous is not None:
                    prev, after = previous
                    join = {
                        "after": after,
                        "left_source": self.neighbours(prev),
                        "right_source": self.neighbours(first),
                    }
                    if cut_index:
                        join["removed"] = [self.line(i) for i in range(prev + 1, first)]
                    joins.append(join)
                played.extend(self.line(i) for i in kept)
                label = (
                    part["part_id"] if resume is None else f"{part['part_id']}.cut{cut_index + 1}"
                )
                previous = (last, label)
                start = resume
        return {
            "playback": played,
            "splices": joins,
            "opening_source": self.neighbours(
                self.in_range(plan["parts"][0]["start_ticks"], plan["parts"][0]["end_ticks"])[0]
            ),
            "closing_source": self.neighbours(previous[0]),
        }

    def neighbours(self, i):
        return {
            "before": [self.line(j) for j in range(max(0, i - 2), i)],
            "line": self.line(i),
            "after": [self.line(j) for j in range(i + 1, min(i + 3, len(self.units)))],
        }


def variants(original, transcript, length, alternatives=()):
    """Offer only geometries derived from the cutter's validated sentence boundaries."""
    result = {"original": copy.deepcopy(original)}
    for i, alternate in enumerate(alternatives, start=1):
        result[f"alternate_{i}"] = alternate
    parts = original["parts"]
    for pos, part in enumerate(parts, start=1):
        if len(parts) == 1 and not part.get("removed"):
            continue
        duration = part["end_ticks"] - part["start_ticks"]
        if not length["min_ticks"] <= duration <= length["max_ticks"]:
            continue
        restored = {
            **part,
            "part_id": "p1",
            "role": "body",
            "units": [
                u["unit"] for u in transcript.segment(part["start_ticks"], part["end_ticks"])
            ],
        }
        restored.pop("removed", None)
        restored.pop("why", None)
        plan = {
            **original,
            "form": "straight",
            "parts": [restored],
            "joins": [],
            "duration_ticks": duration,
            "evidence": restored["units"],
        }
        # Same geometry identity as the Rust cutter for a single uncut body.
        key = (
            f"{plan['source_fingerprint']}|straight|body:{part['start_ticks']}:{part['end_ticks']}"
        )
        digest = hashlib.sha256(key.encode()).hexdigest()[:24]
        plan["plan_id"] = f"plan_{digest}"
        result[f"part_{pos}_uncut"] = plan
    return result


def chapter_of(plan, chapters):
    # Attribute a cold-open to its body, even if its lifted hook came from elsewhere.
    part = max(plan["parts"], key=lambda p: p["end_ticks"] - p["start_ticks"])
    for chapter in chapters:
        if chapter["start_ticks"] <= part["start_ticks"] < chapter["end_ticks"]:
            return chapter["title"]
    return "Unchaptered"


def shortlist(candidates, chapters, count):
    """Round-robin chapters; within one chapter preserve the cutter's score order."""
    groups = {}
    for row in candidates:
        groups.setdefault(chapter_of(row["plan"], chapters), []).append(row)
    selected = []
    for index in range(max((len(plans) for plans in groups.values()), default=0)):
        selected.extend(plans[index] for plans in groups.values() if len(plans) > index)
        if len(selected) >= count:
            break
    return selected[:count]


class Calls:
    """Every model call honours lease cancellation and records its own prompt digest."""

    def __init__(self, runtime, payload, model, cancellation, progress):
        self.runtime = runtime
        self.identity = {"route": "local", "model": {"name": model.name, "digest": model.digest}}
        self.cancellation = cancellation
        self.progress = progress
        self.max_tokens = payload.max_output_tokens
        self.brief = payload.editorial_prompt
        self.taste = list(payload.editorial_taste)
        self.traces = []
        self.chapters = []

    def ask(self, operation, data, reply):
        self.cancellation.raise_if_cancelled()
        prefix = None
        if operation in ("plan", "compare"):
            prefix = {
                "previous_decisions": self.taste,
                "recording_chapters": self.chapters,
            }
            data = {key: value for key, value in data.items() if key not in prefix}
            data["brief"] = self.brief or (
                "Choose a self-contained moment in any topic or offered form."
            )
        prompt_schema = PlanReply.model_json_schema() if operation == "plan" else None
        if operation == "critic":
            prompt_schema = reply.model_json_schema()
            prompt_schema["$defs"]["SpliceJudgments"] = {
                "type": "object",
                "description": "Judge each given splice ID. Empty for uninterrupted speech.",
            }
        return call(
            self.runtime,
            operation,
            data,
            reply,
            self.max_tokens,
            self.identity,
            self.traces,
            fixed_context=prefix,
            prompt_schema=prompt_schema,
        )


def plan_one(ctx, options, transcript, repair=None):
    presentations = []
    lines = {}
    for name, plan in options.items():
        spoken = transcript.evidence(plan)["playback"]
        lines.update((line["unit"], line) for line in spoken)
        presentations.append(
            {
                "choice": name,
                "form": plan["form"],
                "playback_units": [line["unit"] for line in spoken],
                "joins_after": [j["after"] for j in plan["joins"]],
            }
        )
    if not presentations:
        return None
    answer, failure = ctx.ask(
        "plan",
        {
            "brief": ctx.brief,
            "previous_decisions": ctx.taste,
            "spoken_lines": list(lines.values()),
            "choices": presentations,
            "repair_findings": repair or [],
        },
        create_model(
            "LegalPlanChoice",
            __base__=PlanReply,
            choice=(Literal[tuple([*options, "decline"])], ...),
        ),
    )
    if failure or answer["choice"] not in options or not answer["title"].strip():
        return None
    plan = copy.deepcopy(options[answer["choice"]])
    plan["title"] = answer["title"].strip()
    # Generated titles can be new. A quote cannot be new or moved from later on.
    first = transcript.evidence(plan)["playback"][0]["text"]
    hook = answer["hook_text"].strip()
    if first.strip():
        plan["hook_text"] = hook if hook and hook in first else first[:160]
    else:
        plan.pop("hook_text", None)
    plan["doubts"] = answer["doubts"]
    plan["provenance"] = {
        "by": "mind",
        "models": [ctx.identity["model"]["name"]],
        "prompts": ["plan.v1"],
    }
    techniques = {t["after"]: t for t in answer["joins"]}
    for join in plan["joins"]:
        if technique := techniques.get(join["after"]):
            join.update(framing=technique["framing"], crossfade_ms=technique["crossfade_ms"])
    return plan, answer["reasons"]


def plan_batch(ctx, candidates, memory, script, count):
    transcript = Transcript(script)
    chapters = memory["chapters"]
    sources = shortlist(candidates["candidates"], chapters, min(30, count * 2 + 2))
    rows = []
    for index, source in enumerate(sources):
        ctx.progress("plan_candidates", index, len(sources))
        options = variants(
            source["plan"], transcript, candidates["length"], source.get("alternatives", [])
        )
        result = plan_one(ctx, options, transcript)
        if result:
            plan, reasons = result
            rows.append(
                {
                    "candidate_id": source["plan"]["plan_id"],
                    "plan": plan,
                    "chapter": chapter_of(plan, chapters),
                    "reasons": reasons,
                }
            )
    ctx.progress("plan_candidates", len(sources), len(sources))
    return rows
