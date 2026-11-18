"""Independent review of the edited speech, with a single source-grounded repair."""

from __future__ import annotations

from typing import Literal

from pydantic import Field, create_model

from .planning import (
    Reply,
    ShortText,
    Transcript,
    chapter_of,
    plan_one,
    unrelated_signoff,
    variants,
)


class Splice(Reply):
    note: str = Field(min_length=1, max_length=140)
    effect: Literal[
        "preserved",
        "different_answer",
        "lost_qualification",
        "changed_claim",
        "false_chronology",
        "concealed_sponsorship",
        "uncertain",
    ]


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


def judge(ctx, plan, transcript):
    evidence = transcript.evidence(plan)
    ids = [boundary["after"] for boundary in evidence["splices"]]
    if len(ids) > 64:
        return rejected(
            "This edit exceeds the number of joins the critic can independently review."
        )
    # The viewer must not hear the rest of the episode. A separate source audit
    # can read the omitted context and veto even a convincing-sounding edit.
    turns = []
    for line in evidence["playback"]:
        if turns and turns[-1]["speaker"] == line["speaker"]:
            turns[-1]["text"] += " " + line["text"]
        else:
            turns.append({"speaker": line["speaker"], "text": line["text"]})
    viewer, failure = ctx.ask("critic", {"public_title": plan["title"], "playback": turns}, Review)
    if failure:
        return rejected("The critic could not verify the edit.")
    # A fixed property per boundary also covers the empty set. An open-ended list
    # encouraged small models to invent splices or silently skip a difficult one.
    boundaries = create_model(
        "SpliceJudgments", __base__=Reply, **{key: (Splice, ...) for key in ids}
    )
    reply = create_model("CutSourceReview", __base__=SourceReview, joins=(boundaries, ...))
    audit, failure = ctx.ask("critic.source", evidence, reply)
    if failure:
        return rejected("The critic could not verify the edit.")
    findings = [*viewer["issues"], *audit["edge_issues"]]
    reservations = [finding["detail"] for finding in findings]
    problem_areas = {finding["aspect"] for finding in findings}
    issues = []
    judgments = [
        {
            "after": key,
            "note": value["note"],
            "verdict": "pass" if value["effect"] == "preserved" else "reject",
        }
        for key, value in audit["joins"].items()
    ]
    addressed = list(audit["joins"])
    complete = len(addressed) == len(ids) and set(addressed) == set(ids)
    if not complete:
        issues.append("The critic did not judge each edit boundary exactly once.")
    for area, reason in FAILURES.items():
        if area in problem_areas or viewer.get(area) == "reject":
            issues.append(reason)
    if unrelated_signoff(evidence["playback"], ctx.brief) is not None:
        issues.append("The clip continues into unrelated promotion after its conclusion.")
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


def review_batch(ctx, plans, candidates, script, chapters=()):
    transcript = Transcript(script)
    originals = {c["plan"]["plan_id"]: c for c in candidates["candidates"]}
    reviewed = []
    for index, row in enumerate(plans["plans"]):
        ctx.progress("review_plans", index, len(plans["plans"]))
        plan = row["plan"]
        quality = judge(ctx, plan, transcript)
        reasons = row["reasons"]
        if quality["verdict"] == "reject":
            original = originals[row["candidate_id"]]
            choices = variants(
                original["plan"],
                transcript,
                candidates["length"],
                original.get("alternatives", []),
                ctx.brief,
            )
            choices = {k: v for k, v in choices.items() if v["plan_id"] != plan["plan_id"]}
            if (
                FAILURES["title_accuracy"] in quality["issues"]
                and all(j["verdict"] == "pass" for j in quality["joins"])
                and not any(
                    message in quality["issues"]
                    for area, message in FAILURES.items()
                    if area != "title_accuracy"
                )
            ):
                choices["retitle"] = plan
            repaired = plan_one(ctx, choices, transcript, quality["issues"])
            if repaired:
                plan, reasons = repaired
                quality = judge(ctx, plan, transcript)
                quality["repaired"] = True
        chapter = chapter_of(plan, chapters) if chapters else row["chapter"]
        reviewed.append(
            {**row, "chapter": chapter, "plan": plan, "reasons": reasons, "quality": quality}
        )
    ctx.progress("review_plans", len(reviewed), len(reviewed))
    return reviewed
