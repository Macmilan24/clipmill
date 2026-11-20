"""Independent review of the edited speech, with a single source-grounded repair."""

from __future__ import annotations

import re
from typing import Literal

from pydantic import Field

from .comparison import missing_comparison
from .grounding import check_copy, contradicted_omission
from .planning import (
    Reply,
    ShortText,
    Transcript,
    chapter_of,
    cut_short,
    plan_one,
    source_options,
    unrelated_signoff,
)

MAX_SPLICE_NOTE = 140


class Splice(Reply):
    original_claim: str = Field(min_length=1, max_length=160)
    edited_claim: str = Field(min_length=1, max_length=160)
    omitted_condition: str = Field(max_length=180)
    effect: Literal[
        "preserved",
        "omitted_independent_material",
        "different_answer",
        "lost_qualification",
        "changed_claim",
        "false_chronology",
        "concealed_sponsorship",
        "uncertain",
    ]
    note: str = Field(min_length=1, max_length=MAX_SPLICE_NOTE)


class Finding(Reply):
    aspect: Literal["comprehension", "landing", "title_accuracy", "style"]
    detail: ShortText


class Review(Reply):
    summary: ShortText
    comprehension: Literal["pass", "reject"]
    landing: Literal["pass", "reject"]
    title_accuracy: Literal["pass", "reject"]
    hook: Literal["strong", "adequate", "weak"]
    issues: list[Finding] = Field(max_length=4)


class SourceFinding(Reply):
    aspect: Literal["meaning", "sponsorship"]
    detail: ShortText


class SourceReview(Reply):
    edge_issues: list[SourceFinding] = Field(max_length=4)


def rejected(message):
    return {
        "verdict": "reject",
        "hook": "weak",
        "summary": message,
        "joins": [],
        "issues": [message],
        "repaired": False,
    }


FAILURES = {
    "meaning": "The edit changes the meaning of the source.",
    "comprehension": "The setup does not stand alone.",
    "landing": "The ending does not deliver the moment.",
    "title_accuracy": "The title changes the meaning of the spoken words.",
    "sponsorship": "The edit misrepresents sponsored material.",
}
UNRELATED_ENDING = "The clip continues into unrelated promotion after its conclusion."
TITLE_ABSOLUTES = (
    (
        r"\b(?:only|sole|nothing else matters|all that matters)\b",
        r"\b(?:only|sole|solely|alone|exclusively|nothing else matters|"
        r"all (?:that matters|you need))\b",
    ),
    (r"\balways\b", r"\b(?:always|every (?:single )?time|without exception)\b"),
    (r"\bnever\b", r"\b(?:never|not once|at no (?:point|time))\b"),
    (r"\bguarantee(?:d|s)?\b", r"\b(?:guarantee(?:d|s)?|assured|certain(?:ly|ty)?)\b"),
)


def unsupported_absolute(title, playback):
    text = " ".join(line["text"] for line in playback)
    for absolute, support in TITLE_ABSOLUTES:
        introduced = re.search(absolute, title, re.IGNORECASE)
        if introduced and not re.search(support, text, re.IGNORECASE):
            return introduced.group()
    return None


def join_note(value):
    note = value["note"].strip()
    if len(note) < MAX_SPLICE_NOTE - 2 or note.endswith((".", "!", "?")):
        return note
    # Constrained local decoders can reach a JSON character limit mid-word.
    # The raw wording and the separate factual claims remain in the private trace.
    if value["effect"] == "lost_qualification" and value.get("omitted_condition"):
        condition = value["omitted_condition"]
        ending = "" if condition.endswith((".", "!", "?")) else "."
        return f"The cut omits the condition “{condition}”{ending}"
    return {
        "preserved": "Both statements keep their original meaning.",
        "omitted_independent_material": "The omitted passage makes a separate point.",
        "different_answer": "The reply answers a different question in the recording.",
        "changed_claim": "The joined statements change the speaker's original claim.",
        "false_chronology": "The join changes the sequence of events.",
        "concealed_sponsorship": "The join removes a sponsorship disclosure.",
    }.get(value["effect"], "The source audit could not establish that the meaning survived.")


def spoken_turns(lines):
    turns = []
    for line in lines:
        if turns and turns[-1]["speaker"] == line["speaker"]:
            turns[-1]["text"] += " " + line["text"]
        else:
            turns.append({"speaker": line["speaker"], "text": line["text"]})
    return turns


def judge(ctx, plan, transcript):
    evidence = transcript.evidence(plan)
    if len(evidence["splices"]) > 64:
        return rejected(
            "This edit exceeds the number of joins the critic can independently review."
        )
    # The viewer never hears unplayed words. A source check can veto even a
    # convincing-sounding cut, including a continuous span clipped at an edge.
    turns = spoken_turns(evidence["playback"])
    viewer, failure = ctx.ask("critic", {"public_title": plan["title"], "playback": turns}, Review)
    if failure:
        return rejected("The critic could not verify the edit.")
    edges = {
        "check": "outer_edges",
        "playback": turns,
        "recorded_before_opening": spoken_turns(evidence["opening_source"]["before"]),
        "recorded_after_ending": spoken_turns(evidence["closing_source"]["after"]),
    }
    audit, failure = ctx.ask("critic.edge", edges, SourceReview)
    if failure:
        return rejected("The critic could not verify the edit.")
    findings = [*viewer["issues"], *audit["edge_issues"]]
    reservations = [finding["detail"] for finding in findings]
    problem_areas = {finding["aspect"] for finding in findings}
    issues = []
    judgments = []
    # Present one actual cut at a time. Neighbouring passages and their
    # judgements from a different splice must never be mistaken for this join.
    for join in evidence["splices"]:
        transition = {
            "check": "one_join",
            "after": join["after"],
            "source_order": join["source_order"],
            "edited": spoken_turns([*join["heard_left"], *join["heard_right"]]),
        }
        if "removed" in join:
            transition["original"] = spoken_turns(
                [*join["heard_left"], *join["removed"], *join["heard_right"]]
            )
        else:
            transition["original_contexts"] = {
                "at_left": spoken_turns([*join["heard_left"], *join["left_source"]["after"]]),
                "at_right": spoken_turns([*join["right_source"]["before"], *join["heard_right"]]),
            }
        heard_question = join["heard_left"][-1]["text"].strip()
        if heard_question.endswith(("?", "\uff1f")):
            recorded = [
                line["text"]
                for line in join["right_source"]["before"]
                if line["text"].strip().endswith(("?", "\uff1f"))
            ]
            if recorded:
                transition["question_before_reply_in_edit"] = heard_question
                transition["question_before_reply_in_source"] = recorded[-1]
        value, failure = ctx.ask("critic.source", transition, Splice)
        if failure or not value or "effect" not in value:
            value = {"effect": "uncertain", "note": "The critic could not verify this join."}
        if comparison := missing_comparison(join):
            value = {"effect": "changed_claim", "note": comparison}
        elif value["effect"] == "lost_qualification":
            condition = value.get("omitted_condition", "").strip().casefold()
            original_text = transition.get("original") or [
                *transition["original_contexts"]["at_left"],
                *transition["original_contexts"]["at_right"],
            ]
            if (
                not condition
                or not any(condition in line["text"].casefold() for line in original_text)
                or any(condition in line["text"].casefold() for line in transition["edited"])
            ):
                value = {
                    "effect": "uncertain",
                    "note": "The source audit could not identify the omitted qualification.",
                }
        judgments.append(
            {
                "after": join["after"],
                "note": join_note(value),
                "verdict": "pass"
                if value["effect"] in ("preserved", "omitted_independent_material")
                else "reject",
            }
        )
    for area, reason in FAILURES.items():
        if area in problem_areas or viewer.get(area) == "reject":
            issues.append(reason)
    if absolute := unsupported_absolute(plan["title"], evidence["playback"]):
        issues.extend(
            [
                FAILURES["title_accuracy"],
                f"The title adds “{absolute}” without equivalent support in the spoken clip.",
            ]
        )
    witness, _, _ = check_copy(ctx, plan["title"], transcript, evidence["playback"])
    if witness:
        issues.extend(
            [
                FAILURES["title_accuracy"],
                f"The title draws its claim from unplayed line {witness['unit']}.",
            ]
        )
    if unrelated_signoff(evidence["playback"], ctx.brief) is not None:
        issues.append(UNRELATED_ENDING)
    if viewer["hook"] == "weak":
        reservations.append("The opening may need a stronger hook.")
    issues.extend(join["note"] for join in judgments if join["verdict"] == "reject")
    return {
        "verdict": "reject" if issues else "accept",
        "hook": viewer["hook"],
        "summary": viewer["summary"],
        "joins": judgments,
        "issues": list(dict.fromkeys([*issues, *reservations])),
        "repaired": False,
    }


def review_batch(ctx, plans, candidates, script, chapters=(), judgments=None):
    transcript = Transcript(script)
    refused = cut_short(judgments)
    originals = {c["plan"]["plan_id"]: c for c in candidates["candidates"]}
    reviewed = []
    for index, row in enumerate(plans["plans"]):
        ctx.progress("review_plans", index, len(plans["plans"]))
        plan = row["plan"]
        quality = judge(ctx, plan, transcript)
        reasons = row["reasons"]
        if quality["verdict"] == "reject":
            original = originals[row["candidate_id"]]
            choices = source_options(original, transcript, candidates, ctx.brief, refused)
            choices = {k: v for k, v in choices.items() if v["plan_id"] != plan["plan_id"]}
            if (
                FAILURES["title_accuracy"] in quality["issues"]
                and UNRELATED_ENDING not in quality["issues"]
                and all(j["verdict"] == "pass" for j in quality["joins"])
                and not any(
                    message in quality["issues"]
                    for area, message in FAILURES.items()
                    if area != "title_accuracy"
                )
            ):
                choices = {"retitle": plan}
            repaired = plan_one(ctx, choices, transcript, quality["issues"])
            if repaired:
                plan, reasons = repaired
                quality = judge(ctx, plan, transcript)
                quality["repaired"] = True
        if quality["verdict"] == "accept":
            playback = transcript.evidence(plan)["playback"]
            reasons = [r for r in reasons if check_copy(ctx, r, transcript, playback)[0] is None]
            if not reasons:
                reasons = [quality["summary"]]
            if quality["repaired"] and plan.get("doubts"):
                plan = {
                    **plan,
                    "doubts": [
                        note
                        for note in plan["doubts"]
                        if not contradicted_omission(ctx, note, playback)
                    ],
                }
        chapter = chapter_of(plan, chapters) if chapters else row["chapter"]
        reviewed.append(
            {**row, "chapter": chapter, "plan": plan, "reasons": reasons, "quality": quality}
        )
    ctx.progress("review_plans", len(reviewed), len(reviewed))
    return reviewed
