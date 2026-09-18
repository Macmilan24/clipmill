"""Bounded editorial operations, independently testable without model weights."""

from __future__ import annotations

import hashlib
import json
import time
from pathlib import Path
from typing import Annotated, Literal

from clipmill_worker_sdk import LeaseCancelled
from clipmill_worker_sdk.gen.schemas.editorial_proposals import Proposal
from pydantic import BaseModel, ConfigDict, Field, ValidationError, create_model

from .errors import BudgetExceeded, ProviderRequestError


class Moment(BaseModel):
    model_config = ConfigDict(extra="forbid")
    title: str = Field(min_length=1, max_length=120)
    hook: str = Field(min_length=1, max_length=500)
    setup: str = Field(min_length=1, max_length=500)
    payoff: str = Field(min_length=1, max_length=500)
    reason: str = Field(min_length=1, max_length=500)
    uncertainties: list[Annotated[str, Field(min_length=1, max_length=300)]] = Field(max_length=5)


def bounded_outline(outline, first, end):
    """Keep global context and nearby citations bounded on long recordings."""
    if len(outline) <= 32:
        return outline
    local = [
        i
        for i, t in enumerate(outline)
        if t["first_sentence_index"] < end
        and t["first_sentence_index"] + t["sentence_count"] > first
    ]
    chosen = set(local[:24])
    chosen.update(i * (len(outline) - 1) // 7 for i in range(8))
    return [outline[i] for i in sorted(chosen)]


def window_reply(sentences, first, end, duration):
    """Constrain model choices to full-sentence spans within the duration.

    This does not choose a moment: Qwen chooses one of the legal spans, or
    none. Rust still validates references, timing regions and duplicate cuts.
    The compact ranges in the prompt avoid repeating a quadratic span list.
    """
    refs = []
    ranges = []
    core = [s for s in sentences if first <= s["index"] < end]
    for a in core:
        ends = [
            b["index"]
            for b in core
            if b["index"] >= a["index"]
            and duration.min_ticks <= b["end_ticks"] - a["start_ticks"] <= duration.max_ticks
        ]
        if ends:
            refs.extend(f"{a['index']}:{b}" for b in ends)
            ranges.append({"first": a["index"], "last_min": min(ends), "last_max": max(ends)})
    if not refs:
        return None, ranges
    nomination = create_model("WindowMoment", __base__=Moment, span_ref=(Literal[tuple(refs)], ...))
    reply = create_model(
        "WindowReply",
        __config__=ConfigDict(extra="forbid"),
        proposals=(list[nomination], Field(max_length=5)),
    )
    return reply, ranges


class Reason(BaseModel):
    model_config = ConfigDict(extra="forbid")
    code: Literal[
        "unresolved_reference",
        "missing_question",
        "incomplete_payoff",
        "ad_read",
        "irrelevant_intro",
        "misleading_omission",
        "visual_dependency",
        "other",
    ]
    detail: str = Field(min_length=1, max_length=500)


class ReviewReply(BaseModel):
    model_config = ConfigDict(extra="forbid")
    status: Literal["accepted", "needs_review", "rejected"]
    reasons: list[Reason] = Field(max_length=8)
    visual_dependency: bool
    summary: str = Field(min_length=1, max_length=500)


class LookReply(BaseModel):
    model_config = ConfigDict(extra="forbid")
    answer: Literal["yes", "no", "unclear"]
    detail: str = Field(min_length=1, max_length=500)


def prompt_for(operation):
    return Path(__file__).with_name("prompts").joinpath(f"{operation}.v1.txt").read_text()


def prompt_digest(operation):
    return "sha256:" + hashlib.sha256(prompt_for(operation).encode()).hexdigest()


def call(
    runtime,
    operation,
    data,
    response_type,
    max_tokens,
    identity,
    traces,
    images=None,
    prompt_schema=None,
):
    text = (
        prompt_for(operation)
        + "\nAnswer schema:\n"
        + json.dumps(prompt_schema or response_type.model_json_schema())
        + "\nInput data:\n"
        + json.dumps(data, ensure_ascii=False)
    )
    start = time.monotonic()
    raw = ""
    tokens = {"input": 0, "output": 0}
    trace = {
        "index": len(traces),
        "job": operation,
        **identity,
        "prompt_version": f"{operation}.v1/{prompt_digest(operation)}",
        "request_text": text,
        "started_unix_millis": time.time_ns() // 1_000_000,
    }
    try:
        raw, tokens = runtime.generate(text, response_type.model_json_schema(), max_tokens, images)
        answer = response_type.model_validate_json(raw)
        trace["outcome"] = "none" if operation == "propose" and not answer.proposals else "answered"
        return answer.model_dump(mode="json", exclude_none=True), None
    except LeaseCancelled:
        trace["outcome"] = "cancelled"
        raise
    except (ValidationError, json.JSONDecodeError, ValueError) as e:
        # No extracting JSON from prose or repairing the model's answer silently.
        trace["outcome"] = "malformed"
        trace["detail"] = str(e)[:1000]
        return None, {
            "failure_class": "deterministic",
            "detail": "Malformed editorial reply: " + str(e)[:500],
        }
    except (RuntimeError, TimeoutError) as e:
        if isinstance(e, BudgetExceeded):
            trace["outcome"] = "budget"
            trace["detail"] = str(e)
            return None, {"failure_class": "budget", "detail": str(e)}
        trace["outcome"] = "failed"
        trace["detail"] = str(e)[:1000]
        return None, {
            "failure_class": (
                "deterministic"
                if isinstance(e, ProviderRequestError) and not e.retryable
                else "transient"
            ),
            "detail": "Editorial inference failed: " + str(e)[:500],
        }
    finally:
        trace.update(
            response_text=raw, tokens=tokens, duration_millis=int((time.monotonic() - start) * 1000)
        )
        if identity["route"] == "cloud":
            trace["cost_micro_usd"] = runtime.last_cost
        traces.append(trace)


def propose(
    runtime,
    windows,
    duration,
    max_tokens,
    identity,
    traces,
    cancellation,
    progress=lambda *args: None,
):
    answers = []
    for position, window in enumerate(windows.get("windows", [])):
        progress("windows", position, len(windows.get("windows", [])))
        cancellation.raise_if_cancelled()
        first = window["first_sentence_index"]
        end = first + window["sentence_count"]
        context_start = max(0, first - window["context_before_sentences"])
        context_end = end + window["context_after_sentences"]
        response_type, ranges = window_reply(windows.get("sentences", []), first, end, duration)
        if response_type is None:
            answers.append({"window_index": window["index"], "status": "none", "proposals": []})
            continue
        sentences = [
            {
                "index": s["index"],
                "text": s["text"],
                "start_seconds": s["start_ticks"] / 90000,
                "end_seconds": s["end_ticks"] / 90000,
            }
            for s in windows.get("sentences", [])
            if context_start <= s["index"] < context_end
        ]
        data = {
            "window": window,
            "sentences": sentences,
            "outline": bounded_outline(windows.get("outline", []), first, end),
            "duration_seconds": {
                "min": duration.min_ticks / 90000,
                "max": duration.max_ticks / 90000,
            },
            "proposal_ids": f"prop_{window['index']}_0, prop_{window['index']}_1, ...",
            "allowed_end_ranges": ranges,
        }
        prompt_schema = response_type.model_json_schema()
        prompt_schema["$defs"]["WindowMoment"]["properties"]["span_ref"] = {
            "type": "string",
            "description": "first:last sentence indexes from allowed_end_ranges",
        }
        answer, failure = call(
            runtime,
            "propose",
            data,
            response_type,
            max_tokens,
            {**identity, "window_index": window["index"]},
            traces,
            prompt_schema=prompt_schema,
        )
        if failure:
            answers.append(
                {
                    "window_index": window["index"],
                    "status": "malformed" if traces[-1]["outcome"] == "malformed" else "failed",
                    "failure": failure,
                    "proposals": [],
                }
            )
        else:
            # IDs belong to this call; the model supplies only references into speech.
            for i, p in enumerate(answer["proposals"]):
                first, last = map(int, p.pop("span_ref").split(":"))
                p.update(
                    first_sentence_index=first, sentence_count=last - first + 1, splittable=False
                )
                p["id"] = f"prop_{window['index']}_{i}"
                Proposal.model_validate(p)
            answers.append(
                {
                    "window_index": window["index"],
                    "status": "answered" if answer["proposals"] else "none",
                    **answer,
                }
            )
    return answers


def review(
    runtime,
    windows,
    candidates,
    max_tokens,
    identity,
    traces,
    cancellation,
    progress=lambda *args: None,
):
    answers = []
    sentences = windows.get("sentences", [])
    for position, candidate in enumerate(candidates.get("candidates", [])):
        progress("candidates", position, len(candidates.get("candidates", [])))
        cancellation.raise_if_cancelled()
        start = candidate["intervals"][0]["start_ticks"]
        end = candidate["intervals"][-1]["end_ticks"]
        positions = [
            i for i, s in enumerate(sentences) if s["start_ticks"] < end and s["end_ticks"] > start
        ]
        if not positions:
            raise ValueError("candidate has no transcript sentences")
        a = max(0, min(positions) - 2)
        b = min(len(sentences), max(positions) + 3)
        context = {"first_sentence_index": sentences[a]["index"], "sentence_count": b - a}
        data = {
            "candidate": candidate,
            "sentences": sentences[a:b],
            "clip_sentence_indexes": [sentences[i]["index"] for i in positions],
        }
        answer, failure = call(
            runtime,
            "review",
            data,
            ReviewReply,
            max_tokens,
            {**identity, "candidate_id": candidate["id"]},
            traces,
        )
        entry = {"candidate_id": candidate["id"], "context": context}
        if failure:
            entry.update(
                outcome="malformed" if traces[-1]["outcome"] == "malformed" else "failed",
                failure=failure,
                reasons=[],
            )
        else:
            # A problem code is stronger than a contradictory optimistic label.
            # Preserve the raw answer in the trace, but never present a flagged
            # omission or unresolved visual reference as ready without review.
            if any(reason["code"] == "visual_dependency" for reason in answer["reasons"]):
                answer["visual_dependency"] = True
            if answer["status"] == "accepted" and any(
                reason["code"] != "other" for reason in answer["reasons"]
            ):
                answer["status"] = "needs_review"
            entry.update(outcome="answered", **answer)
        answers.append(entry)
    return answers


class LazyVisualRuntime:
    """Load inside the traced call, so allocation failures are visible check failures."""

    def __init__(self, factory):
        self.factory = factory
        self.runtime = None

    def generate(self, *args):
        if self.runtime is None:
            self.runtime = self.factory()
        return self.runtime.generate(*args)

    def close(self):
        if self.runtime is not None:
            self.runtime.close()


def look(
    runtime_factory,
    candidates,
    judgments,
    frames,
    image_path,
    max_tokens,
    identity,
    traces,
    cancellation,
    progress=lambda *args: None,
):
    """At most four in-span frames; never send pictures from another moment."""
    by_id = {c["id"]: c for c in candidates.get("candidates", [])}
    checks = []
    runtime = LazyVisualRuntime(runtime_factory)
    try:
        for verdict in judgments.get("candidates", []):
            if (
                verdict.get("outcome") != "answered"
                or not (
                    verdict.get("visual_dependency")
                    or any(r["code"] == "visual_dependency" for r in verdict["reasons"])
                )
                or verdict.get("status") == "rejected"
            ):
                continue
            cancellation.raise_if_cancelled()
            candidate = by_id[verdict["candidate_id"]]
            a = candidate["intervals"][0]["start_ticks"]
            b = candidate["intervals"][-1]["end_ticks"]
            available = [f for f in frames.get("frames", []) if a <= f["t_ticks"] < b]
            if not available:
                # The downstream review already marks this candidate needs-review.
                continue
            selected = [
                available[i]
                for i in sorted({round(i * (len(available) - 1) / 3) for i in range(4)})
            ]
            question = (
                "Is the visual reference in this clip identifiable? "
                + verdict.get("summary", "")
                + " "
                + "; ".join(r["detail"] for r in verdict["reasons"])
            )
            answer, failure = call(
                runtime,
                "look",
                {"question": question, "frames": [{"t_ticks": f["t_ticks"]} for f in selected]},
                LookReply,
                max_tokens,
                {
                    **identity,
                    "candidate_id": candidate["id"],
                    "frames": [{"t_ticks": f["t_ticks"]} for f in selected],
                },
                traces,
                [str(image_path(f["file"])) for f in selected],
            )
            entry = {
                "candidate_id": candidate["id"],
                "frames": [{"t_ticks": f["t_ticks"]} for f in selected],
                "question": question,
            }
            if failure:
                entry.update(outcome=traces[-1]["outcome"], failure=failure)
            else:
                entry.update(outcome="answered", **answer)
            checks.append(entry)
    finally:
        runtime.close()
    return checks
