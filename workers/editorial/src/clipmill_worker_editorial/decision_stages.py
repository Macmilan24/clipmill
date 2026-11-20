"""Lease adapter for planning, independent review and ranking over the cutter's plans."""

from __future__ import annotations

import json

from clipmill_worker_sdk import DeterministicTaskError, RetryableTaskError, canonical_bytes
from clipmill_worker_sdk.gen.schemas.editorial_trace import EditorialTrace
from clipmill_worker_sdk.gen.schemas.mind_candidates import MindCandidates
from clipmill_worker_sdk.gen.schemas.mind_clip_plan import MindClipPlan
from clipmill_worker_sdk.gen.schemas.mind_judgments import MindJudgments
from clipmill_worker_sdk.gen.schemas.mind_memory import MindMemory
from clipmill_worker_sdk.gen.schemas.mind_plans import MindPlans
from clipmill_worker_sdk.gen.schemas.mind_quality_report import MindQualityReport
from clipmill_worker_sdk.gen.schemas.mind_script import MindScript
from clipmill_worker_sdk.gen.schemas.ranking_set_v2 import RankingSetV2

from .critique import review_batch
from .judging import judge
from .planning import Calls, plan_batch
from .rank_plans import rank_batch
from .runtime import implementation_suffix, text_session


def execute(context, inputs, model, payload, operation):
    inputs_by_kind = {}

    def read(kind, filename, contract):
        item = inputs.require(kind)
        document = json.loads(context.artifact_file(item.artifact, filename).read_bytes())
        contract.model_validate(document)
        inputs_by_kind[kind] = item.artifact_id
        return document

    script = read("mind.script.v1", "script.json", MindScript)
    source = script["source_fingerprint"]

    def check_source(document):
        if document["source_fingerprint"] != source:
            raise DeterministicTaskError("The editorial inputs describe different recordings.")

    def check_plan(plan):
        MindClipPlan.model_validate(plan)
        check_source(plan)

    def check_pairing(document, names):
        if any(document["inputs"].get(k) != inputs_by_kind[k] for k in names):
            raise DeterministicTaskError("The editorial inputs came from different analysis runs.")

    memory = read("mind.memory.v1", "memory.json", MindMemory)
    check_source(memory)
    if operation in ("judge", "plan", "critic"):
        candidates = read("mind.candidates.v1", "candidates.json", MindCandidates)
        check_source(candidates)
        if any(
            candidates["inputs"].get(key) != inputs_by_kind[kind]
            for key, kind in (
                ("script_artifact_id", "mind.script.v1"),
                ("memory_artifact_id", "mind.memory.v1"),
            )
        ):
            raise DeterministicTaskError("The cutter read a different script or recording memory.")
        for row in candidates["candidates"]:
            check_plan(row["plan"])
            for alternate in row.get("alternatives", []):
                check_plan(alternate)
        needed = bool(candidates["candidates"])
    judgments = None
    if operation in ("plan", "critic") and inputs.optional("mind.judgments.v1"):
        # Older analyses planned without a judge; their plans stand as they were.
        judgments = read("mind.judgments.v1", "judgments.json", MindJudgments)
        check_source(judgments)
        check_pairing(judgments, ("mind.script.v1", "mind.memory.v1", "mind.candidates.v1"))
    if operation == "critic":
        plans = read("mind.plans.v1", "plans.json", MindPlans)
        check_source(plans)
        check_pairing(plans, ("mind.script.v1", "mind.memory.v1", "mind.candidates.v1"))
        needed = bool(plans["plans"])
    elif operation == "rank":
        report = read("mind.quality_report.v1", "quality.json", MindQualityReport)
        check_source(report)
        check_pairing(report, ("mind.script.v1", "mind.memory.v1"))
        needed = (
            len(
                {
                    r["plan"]["plan_id"]
                    for r in report["reviews"]
                    if r["quality"]["verdict"] == "accept"
                }
            )
            > 1
        )
    count = min(100, payload.requested_count or 10)
    with text_session(model, context.cancellation, source, needed=needed) as runtime:
        ctx = Calls(runtime, payload, model, context.cancellation, context.report_progress)
        # The same short outline precedes every planning and comparison request.
        # The independent critic receives only the title and measured source text.
        ctx.chapters = [
            {"title": chapter["title"], "summary": chapter["summary"][:140]}
            for chapter in memory["chapters"][:30]
        ]
        if operation == "judge":
            kind, filename, schema = (
                "clipmill.mind.judgments.v1",
                "judgments.json",
                MindJudgments,
            )
            row_key = None
            document = judge(
                runtime,
                script,
                candidates,
                memory,
                brief=ctx.brief,
                count=count,
                identity=ctx.identity,
                traces=ctx.traces,
                progress=context.report_progress,
            )
        elif operation == "plan":
            kind, filename, schema = "clipmill.mind.plans.v1", "plans.json", MindPlans
            row_key = "plans"
            document = {row_key: plan_batch(ctx, candidates, memory, script, count, judgments)}
        elif operation == "critic":
            kind, filename, schema = (
                "clipmill.mind.quality_report.v1",
                "quality.json",
                MindQualityReport,
            )
            row_key = "reviews"
            checks: dict = {}
            document = {
                row_key: review_batch(
                    ctx, plans, candidates, script, memory["chapters"], judgments, checks
                ),
                **checks,
            }
        else:
            kind, filename, schema = "clipmill.ranking.set.v2", "ranking.json", RankingSetV2
            row_key = "clips"
            document = rank_batch(ctx, report, script, count)
    if any(trace["outcome"] != "answered" for trace in ctx.traces):
        # A missing ballot or review is an incomplete computation. Publishing
        # it under the successful recipe would make the omission a permanent hit.
        raise RetryableTaskError(f"The editorial model did not finish every {operation} call.")
    document.update(schema_version=kind, source_fingerprint=source, inputs=inputs_by_kind)
    schema.model_validate(document)
    for row in document[row_key] if row_key else []:
        check_plan(row["plan"])
    implementation = f"clipmill-worker-editorial@0.2.0/{operation}{implementation_suffix(model)}"
    trace = {
        "schema_version": "clipmill.editorial.trace.v1",
        "content_profile": payload.content_profile or "interview",
        "source_fingerprint": source,
        "producer": {
            "stage": payload.stage,
            "implementation": implementation,
        },
        "calls": ctx.traces,
    }
    EditorialTrace.model_validate(trace)
    context.staging.write_bytes(filename, canonical_bytes(document))
    context.staging.write_bytes("trace.json", canonical_bytes(trace))
    return filename, "trace.json"
