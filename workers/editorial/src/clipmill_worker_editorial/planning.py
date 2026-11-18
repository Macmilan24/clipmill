"""Constrained editing choices and source context for the independent join review."""

from __future__ import annotations

import bisect
import copy
import hashlib
import re
import textwrap
from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field, create_model

from .inference import call

ShortText = Annotated[str, Field(min_length=1, max_length=240)]
SENTENCE_END = re.compile(r"[.!?\u3002\uff01\uff1f][\"'\u2019\u201d)]*$")
EXHAUSTIVE = re.compile(r"\b(?:every time|all (?:the )?(?:times|mentions))\b", re.IGNORECASE)


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
        self.by_id = {u["index"]: u for u in self.units}

    def in_range(self, start, end):
        first = bisect.bisect_left(self.starts, start)
        last = bisect.bisect_left(self.starts, end)
        return [i for i in range(first, last) if self.units[i]["end_ticks"] <= end]

    def line(self, i):
        u = self.units[i]
        return {"unit": f"s{u['index']}", "speaker": u.get("speaker", ""), "text": u["text"]}

    def segment(self, start, end):
        return [self.line(i) for i in self.in_range(start, end)]

    def opening_quote(self, plan):
        part = plan["parts"][0]
        removed = part.get("removed", [])
        end = removed[0]["start_ticks"] if removed else part["end_ticks"]
        lines = self.segment(part["start_ticks"], end)
        words = []
        for line in lines:
            if line["speaker"] != lines[0]["speaker"]:
                break
            words.append(line["text"].strip())
            if SENTENCE_END.search(words[-1]):
                break
        return textwrap.shorten(" ".join(words), width=160, placeholder="")

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
                first = kept[0]
                if previous is not None:
                    earlier, after = previous
                    prev = earlier[-1]
                    join = {
                        "after": after,
                        "source_order": (
                            "consecutive"
                            if first == prev + 1
                            else "forward"
                            if first > prev
                            else "reordered"
                        ),
                        "left_source": self.neighbours(prev),
                        "right_source": self.neighbours(first),
                        "heard_left": [self.line(i) for i in earlier[-4:]],
                        "heard_right": [self.line(i) for i in kept[:4]],
                    }
                    if cut_index or prev < first <= prev + 9:
                        join["removed"] = [self.line(i) for i in range(prev + 1, first)]
                    joins.append(join)
                played.extend(self.line(i) for i in kept)
                label = (
                    part["part_id"] if resume is None else f"{part['part_id']}.cut{cut_index + 1}"
                )
                previous = (kept, label)
                start = resume
        return {
            "playback": played,
            "splices": joins,
            "opening_source": self.neighbours(
                self.in_range(plan["parts"][0]["start_ticks"], plan["parts"][0]["end_ticks"])[0]
            ),
            "closing_source": self.neighbours(previous[0][-1]),
        }

    def neighbours(self, i):
        return {
            "before": [self.line(j) for j in range(max(0, i - 2), i)],
            "line": self.line(i),
            "after": [self.line(j) for j in range(i + 1, min(i + 3, len(self.units)))],
        }


# Match promotional closing sentences only at the beginning of a spoken line.
# A reporting sentence such as "They asked people to subscribe" is substantive speech.
SIGNOFF = re.compile(
    r"^(?:(?:so|and|please|remember to|don't forget to)\s+)*"
    r"(?:like and subscribe|subscribe(?:[.!?]|$| (?:for (?:part|more|the next|future)|"
    r"to (?:my|our|this|the) channel|and (?:like|hit|turn on)|below)\b)|"
    r"follow (?:me|us) (?:for more|on (?:instagram|tiktok|youtube|twitter))\b|"
    r"(?:hit|tap|ring) (?:the|that) (?:notification )?bell\b|thanks for watching\b)",
    re.IGNORECASE,
)
PROMOTION_TOPIC = re.compile(
    r"\b(?:subscri(?:be|ber|bers|bing|ption|ptions)|followers?|sign.?up|calls?[ -]to[ -]action|"
    r"CTA|sign.?off|outro|channel promotion|notification bell)\b",
    re.IGNORECASE,
)


def unrelated_signoff(playback, brief):
    """Find stand-alone end promotion after a passage about something else."""
    split = len(playback)
    while split and SIGNOFF.match(playback[split - 1]["text"].strip()):
        split -= 1
    if split == len(playback) or not split:
        return None
    topic = " ".join([brief, *(line["text"] for line in playback[:split])])
    return None if PROMOTION_TOPIC.search(topic) else split


def montage_requested(brief):
    text = brief.casefold()
    if re.search(r"\b(?:no|avoid|without|skip|not (?:a |the )?)\s*montage\b", text):
        return False
    return bool(
        EXHAUSTIVE.search(text)
        or re.search(
            r"\b(?:montage of|"
            r"(?:make|build|create|assemble|as|into|want) (?:a |the )?(?:short |theme )?montage)\b",
            text,
        )
    )


def geometry_id(plan):
    """Use the same edit identity as the cutter, including every omitted interval."""
    pieces = [plan["source_fingerprint"], plan["form"]]
    for part in plan["parts"]:
        pieces.append(f"{part['role']}:{part['start_ticks']}:{part['end_ticks']}")
        pieces.extend(f"cut:{r['start_ticks']}:{r['end_ticks']}" for r in part.get("removed", []))
    return "plan_" + hashlib.sha256("|".join(pieces).encode()).hexdigest()[:24]


def end_on_point(original, transcript, length, brief):
    played = transcript.evidence(original)["playback"]
    split = unrelated_signoff(played, brief)
    if split is None:
        return None
    last_unit = played[split - 1]["unit"]
    source_id = int(last_unit[1:])
    unit = transcript.by_id[source_id]
    end = unit["end_ticks"]
    plan = copy.deepcopy(original)
    retained = next(
        i + 1
        for i, part in enumerate(plan["parts"])
        if part["start_ticks"] <= unit["start_ticks"] and end <= part["end_ticks"]
    )
    plan["parts"] = plan["parts"][:retained]
    plan["joins"] = plan["joins"][: retained - 1]
    last = plan["parts"][-1]
    if end - last["start_ticks"] < 108_000:
        return None
    last["end_ticks"] = end
    omitted = [r for r in last.get("removed", []) if r["end_ticks"] < end]
    if omitted:
        last["removed"] = omitted
    else:
        last.pop("removed", None)
    discarded = {line["unit"] for line in played[split:]}
    if "units" in last:
        last["units"] = [u for u in last["units"] if u not in discarded]
    if "evidence" in plan:
        plan["evidence"] = [u for u in plan["evidence"] if u not in discarded]
    if retained == 1:
        plan["form"] = "tightened" if omitted else "straight"
        last["role"] = "body"
    elif plan["form"] == "stitch" and all(j["kind"] != "distant" for j in plan["joins"]):
        return None
    duration = sum(
        p["end_ticks"]
        - p["start_ticks"]
        - sum(r["end_ticks"] - r["start_ticks"] for r in p.get("removed", []))
        for p in plan["parts"]
    )
    if not length["min_ticks"] <= duration <= length["max_ticks"]:
        return None
    plan.update(plan_id=geometry_id(plan), duration_ticks=duration)
    return plan


def variants(original, transcript, length, alternatives=(), brief=""):
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
        plan["plan_id"] = geometry_id(plan)
        result[f"part_{pos}_uncut"] = plan
    for name, plan in list(result.items()):
        if tighter := end_on_point(plan, transcript, length, brief):
            result[f"{name}_ending"] = tighter
    if montage_requested(brief):
        return {key: choice for key, choice in result.items() if choice["form"] == "theme_montage"}
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
        parts = []
        for part in plan["parts"]:
            spoken = transcript.evidence({"parts": [part]})["playback"]
            lines.update((line["unit"], line) for line in spoken)
            parts.append(
                {
                    "part": part["part_id"],
                    "role": part["role"],
                    "units": [line["unit"] for line in spoken],
                }
            )
        presentations.append(
            {
                "choice": name,
                "form": plan["form"],
                "parts_in_playback_order": parts,
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
    first = transcript.opening_quote(plan)
    hook = answer["hook_text"].strip()
    if first:
        plan["hook_text"] = hook if hook and first.startswith(hook) else first
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


def covers_named_entity(plan, entities, brief):
    evidence = set(plan.get("evidence", []))
    for entity in entities:
        events = set(entity["events"])
        if entity["entity_id"] not in evidence or not events or not events <= evidence:
            continue
        labels = [entity["name"], *entity.get("aliases", [])]
        if any(
            re.search(rf"(?<!\w){re.escape(label)}(?!\w)", brief, re.IGNORECASE)
            for label in labels
            if label
        ):
            return True
    return False


def plan_batch(ctx, candidates, memory, script, count):
    transcript = Transcript(script)
    chapters = memory["chapters"]
    pool = candidates["candidates"]
    if montage_requested(ctx.brief):
        montages = [row for row in pool if row["plan"]["form"] == "theme_montage"]
        requested = [row for row in montages if row["scout"] == "brief"]
        if not requested and EXHAUSTIVE.search(ctx.brief):
            # Generic theme scouting may select just five highlights. Only use
            # that fallback for an explicitly named entity if every event it
            # cites in recording memory survived in the proposed montage.
            pool = [
                row
                for row in montages
                if covers_named_entity(row["plan"], memory.get("entities", []), ctx.brief)
            ]
        else:
            pool = requested or montages
    sources = shortlist(pool, chapters, min(30, count * 2 + 2))
    rows = []
    for index, source in enumerate(sources):
        ctx.progress("plan_candidates", index, len(sources))
        options = variants(
            source["plan"],
            transcript,
            candidates["length"],
            source.get("alternatives", []),
            ctx.brief,
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
