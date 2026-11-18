"""Authenticated local editorial worker and shared lease-scoped execution."""

from __future__ import annotations

import argparse
import json
import logging
import os
import platform
import signal
import sys
import threading
from pathlib import Path

from clipmill.ipc.v1 import daemon_pb2
from clipmill_worker_sdk import (
    DeterministicTaskError,
    LeaseInputs,
    RetryableTaskError,
    TaskContext,
    WorkerClient,
    WorkerConfiguration,
    WorkerIdentity,
    canonical_bytes,
    memory_ceiling,
    require_model,
)

from .inference import look, prompt_digest, propose, review
from .runtime import implementation_suffix

CAPABILITIES = (
    "editorial-look",
    "editorial-memory",
    "editorial-propose",
    "editorial-review",
    "editorial-score",
    "youtube-metadata",
)
# The mind's stages: local only, and keyed on their own output budgets.
MIND_OPERATIONS = ("memory", "score")
MAX_OUTPUT_TOKENS = {"memory": 4096}


def execute(context: TaskContext) -> tuple[str, ...]:
    """Local entrypoint: cloud leases are rejected before any runtime is opened."""
    if context.lease.kind == "youtube-metadata":
        from .metadata import execute_metadata

        return execute_metadata(context)
    return execute_stage(context, CAPABILITIES)


def execute_stage(context: TaskContext, capabilities, cloud_runtime=None) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = daemon_pb2.EditorialStagePayloadV1.FromString(context.lease.payload)
    if (
        payload.key_version != "clipmill.editorial-stage.v1"
        or payload.stage not in capabilities
        or payload.stage != context.lease.kind
    ):
        raise DeterministicTaskError("invalid editorial lease")
    profile = content_profile(getattr(payload, "content_profile", ""))
    operation = payload.stage.removeprefix("editorial-").removesuffix("-cloud")
    cloud = payload.stage.endswith("-cloud")
    if cloud != payload.HasField("cloud"):
        raise DeterministicTaskError("cloud permission does not match the task route")
    if cloud and cloud_runtime is None:
        raise DeterministicTaskError("this worker does not execute cloud tasks")
    if payload.prompt_digest != prompt_digest(
        operation
    ) or payload.max_output_tokens != MAX_OUTPUT_TOKENS.get(operation, 2048):
        raise DeterministicTaskError(
            "editorial prompt or decoding policy differs from the planned recipe; update the worker"
        )
    if cloud and operation in MIND_OPERATIONS:
        raise DeterministicTaskError("the mind's stages run on this computer only")
    model = None if cloud else require_model(context.lease, "editorial")
    inputs = LeaseInputs(context)
    if operation == "look":
        return execute_look(context, inputs, model, payload)
    if operation in MIND_OPERATIONS:
        return execute_mind(context, inputs, model, payload, operation)
    windows_input = inputs.require("editorial.windows.v1")
    windows = json.loads(context.artifact_file(windows_input.artifact, "windows.json").read_text())
    if content_profile(windows.get("content_profile", "")) != profile:
        raise DeterministicTaskError("editorial context and task use different content profiles")
    identity = (
        {"route": "cloud", "model": {"name": payload.cloud.model, "provider": "anthropic"}}
        if cloud
        else {"route": "local", "model": {"name": model.name, "digest": model.digest}}
    )

    def make_runtime():
        if cloud:
            return cloud_runtime(
                payload.cloud,
                Path(context.lease.artifact_root).parent / "state" / "editorial-budgets",
                context.lease.job_id,
                context.cancellation,
            )
        from .runtime import acquire_runtime

        return acquire_runtime(model, context.cancellation)

    route = "-cloud" if cloud else implementation_suffix(model)
    producer = {
        "stage": payload.stage,
        "implementation": f"clipmill-worker-editorial@0.2.0/{operation}{route}",
        **identity,
        "prompt_version": f"{operation}.v1/{payload.prompt_digest}",
        "decoding": {
            "temperature_milli": 0,
            "seed": 0,
            "max_output_tokens": 2048,
            "constrained": True,
        },
    }
    document_kind = "proposals" if operation == "propose" else "judgments"
    document = {
        "schema_version": f"clipmill.editorial.{document_kind}.v1",
        "content_profile": profile,
        "source_fingerprint": windows["source_fingerprint"],
        "producer": producer,
        "inputs": {"windows_artifact_id": windows_input.artifact_id},
    }
    traces = []
    runtime = None
    try:
        # Empty recordings and empty candidate sets need no model allocation.
        if operation == "propose":
            if windows.get("windows", []):
                runtime = make_runtime()
            document["windows"] = propose(
                runtime,
                windows,
                payload.duration,
                2048,
                identity,
                traces,
                context.cancellation,
                context.report_progress,
            )
            output = "proposals.json"
        else:
            candidates_input = inputs.require("discovery.candidates.v1")
            candidates = json.loads(
                context.artifact_file(candidates_input.artifact, "candidates.json").read_text()
            )
            if candidates["source_fingerprint"] != windows["source_fingerprint"]:
                raise DeterministicTaskError("candidates and context describe different sources")
            if content_profile(candidates.get("content_profile", "")) != profile:
                raise DeterministicTaskError(
                    "candidates and context use different content profiles"
                )
            document["inputs"]["candidates_artifact_id"] = candidates_input.artifact_id
            if candidates.get("candidates", []):
                runtime = make_runtime()
            document["candidates"] = review(
                runtime,
                windows,
                candidates,
                2048,
                identity,
                traces,
                context.cancellation,
                context.report_progress,
            )
            output = "judgments.json"
        trace = {
            "schema_version": "clipmill.editorial.trace.v1",
            "content_profile": profile,
            "source_fingerprint": windows["source_fingerprint"],
            "producer": {"stage": payload.stage, "implementation": producer["implementation"]},
            "calls": traces,
        }
        if cloud and runtime is not None:
            trace["budget"] = {
                "cap_micro_usd": payload.cloud.budget_micro_usd,
                "spent_micro_usd": runtime.budget.spent,
            }
        answers = document.get("windows", document.get("candidates", []))
        fail_with_trace(context, answers, trace)
        context.staging.write_bytes(output, canonical_bytes(document))
        context.staging.write_bytes("trace.json", canonical_bytes(trace))
        context.report_progress("editorial_calls", len(traces), len(traces))
        return (output, "trace.json")
    finally:
        if runtime is not None:
            runtime.close()


def execute_mind(context, inputs, model, payload, operation):
    """The Recording Memory or the sentence markup, from the annotated script."""
    from clipmill_worker_sdk.gen.schemas.mind_memory import MindMemory
    from clipmill_worker_sdk.gen.schemas.mind_script import MindScript
    from clipmill_worker_sdk.gen.schemas.mind_unit_scores import MindUnitScores

    from .memory import build_memory
    from .runtime import acquire_runtime
    from .score import score_units

    script_input = inputs.require("mind.script.v1")
    script = json.loads(context.artifact_file(script_input.artifact, "script.json").read_text())
    MindScript.model_validate(script)
    identity = {"route": "local", "model": {"name": model.name, "digest": model.digest}}
    implementation = f"clipmill-worker-editorial@0.2.0/{operation}{implementation_suffix(model)}"
    traces = []
    # Borrowed from the resident model, so memory, markup and the stages after
    # them reuse one load; closing hands it back.
    runtime = acquire_runtime(model, context.cancellation) if script["windows"] else None
    try:
        build, schema, output = (
            (build_memory, MindMemory, "memory.json")
            if operation == "memory"
            else (score_units, MindUnitScores, "unit_scores.json")
        )
        document = build(
            runtime,
            script,
            source_fingerprint=script["source_fingerprint"],
            implementation=implementation,
            model=model.name,
            max_tokens=payload.max_output_tokens,
            identity=identity,
            traces=traces,
            cancellation=context.cancellation,
            progress=context.report_progress,
        )
    finally:
        if runtime is not None:
            runtime.close()
    # A document this worker wrote and its own contract refuses is a bug here,
    # not something a retry fixes.
    try:
        schema.model_validate(document)
    except ValueError as error:
        raise DeterministicTaskError(f"{output} does not meet its contract: {error}") from error
    if script["windows"] and traces and all(t.get("outcome") != "answered" for t in traces):
        detail = next((t.get("detail") for t in traces if t.get("detail")), "no answer")
        raise RetryableTaskError(f"the model answered none of the {operation} calls: {detail}")
    trace = {
        "schema_version": "clipmill.editorial.trace.v1",
        "content_profile": "interview",
        "source_fingerprint": script["source_fingerprint"],
        "producer": {"stage": payload.stage, "implementation": implementation},
        "calls": traces,
    }
    context.staging.write_bytes(output, canonical_bytes(document))
    context.staging.write_bytes("trace.json", canonical_bytes(trace))
    context.report_progress("editorial_calls", len(traces), len(traces))
    return (output, "trace.json")


def content_profile(value):
    profile = value or "interview"
    if profile not in ("interview", "scripted"):
        raise DeterministicTaskError("unsupported editorial content profile")
    return profile


def fail_with_trace(context, answers, trace, *, allow_all_failed=False):
    failed = next((a for a in answers if a.get("failure")), None)
    if failed:
        # Keep failure diagnostics even when other rows permit partial publication.
        # The daemon excludes incomplete artifacts from reusable success entries;
        # consumers still receive every row's explicit status and failure reason.
        root = Path(context.lease.artifact_root).parent / "state" / "editorial-traces"
        root.mkdir(mode=0o700, parents=True, exist_ok=True)
        import re

        if not re.fullmatch(r"tsk_[A-Za-z0-9]+", context.lease.task_id):
            raise DeterministicTaskError("invalid diagnostic task identity")
        destination = root / (context.lease.task_id + ".json")
        # O_NOFOLLOW is POSIX only, and O_BINARY Windows only, where a
        # descriptor otherwise rewrites the bytes as text.
        flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_BINARY", 0)
        fd = os.open(destination, flags | getattr(os, "O_NOFOLLOW", 0), 0o600)
        with os.fdopen(fd, "wb") as handle:
            handle.write(canonical_bytes(trace))
        if allow_all_failed or any(
            a.get("status") in ("answered", "none") or a.get("outcome") == "answered"
            for a in answers
        ):
            return
        error = failed["failure"]
        kind = (
            RetryableTaskError if error["failure_class"] == "transient" else DeterministicTaskError
        )
        raise kind(error["detail"])


def execute_look(context, inputs, model, payload):
    from .runtime import acquire_runtime

    def read(kind, name):
        entry = inputs.require(kind)
        return entry, json.loads(context.artifact_file(entry.artifact, name).read_text())

    ci, candidates = read("discovery.candidates.v1", "candidates.json")
    ji, judgments = read("editorial.judgments.v1", "judgments.json")
    fi, frames = read("media.frames.v1", "index.json")
    if (
        candidates["source_fingerprint"] != judgments["source_fingerprint"]
        or frames["source_fingerprint"] != candidates["source_fingerprint"]
        or judgments["inputs"]["candidates_artifact_id"] != ci.artifact_id
    ):
        raise DeterministicTaskError("visual check inputs describe different sources or candidates")
    profile = content_profile(getattr(payload, "content_profile", ""))
    if any(
        content_profile(document.get("content_profile", "")) != profile
        for document in (candidates, judgments)
    ):
        raise DeterministicTaskError("visual check inputs use different content profiles")
    identity = {"route": "local", "model": {"name": model.name, "digest": model.digest}}
    producer = {
        "stage": payload.stage,
        "implementation": f"clipmill-worker-editorial@0.2.0/look{implementation_suffix(model)}",
        **identity,
        "prompt_version": f"look.v1/{payload.prompt_digest}",
        "decoding": {
            "temperature_milli": 0,
            "seed": 0,
            "max_output_tokens": 2048,
            "constrained": True,
        },
    }
    traces = []
    checks = look(
        lambda: acquire_runtime(model, context.cancellation),
        candidates,
        judgments,
        frames,
        lambda name: context.artifact_file(fi.artifact, name),
        2048,
        identity,
        traces,
        context.cancellation,
        context.report_progress,
    )
    document = {
        "schema_version": "clipmill.editorial.looks.v1",
        "content_profile": profile,
        "source_fingerprint": candidates["source_fingerprint"],
        "inputs": {"judgments_artifact_id": ji.artifact_id, "frames_artifact_id": fi.artifact_id},
        "producer": producer,
        "checks": checks,
    }
    trace = {
        "schema_version": "clipmill.editorial.trace.v1",
        "content_profile": profile,
        "source_fingerprint": candidates["source_fingerprint"],
        "producer": {"stage": payload.stage, "implementation": producer["implementation"]},
        "calls": traces,
    }
    # Visual checks are optional evidence. Their failure keeps affected clips
    # in needs-review; it must not discard already reviewed nonvisual moments.
    fail_with_trace(context, checks, trace, allow_all_failed=True)
    context.staging.write_bytes("looks.json", canonical_bytes(document))
    context.staging.write_bytes("trace.json", canonical_bytes(trace))
    return ("looks.json", "trace.json")


# The worker's memory ceiling follows the machine, because it loads whichever
# model its lease binds and each task declares that model's own memory. The
# floor is only what the smallest model needs: floored at the 9B instead, an
# 8 GB Mac's worker offered more than the daemon's budget and was never given
# any work, not even for the 4B. A machine whose memory cannot be read offers
# the floor; CLIPMILL_WORKER_MEMORY_BYTES raises it.

#: Weights plus runtime allowance of the smallest editorial model each runtime
#: loads, as its manifest declares: Qwen3.5 4B, in
#: models/registry/qwen3-5-4b-editorial-{mlx,gguf}.toml.
SMALLEST_MODEL_BYTES = {
    "mlx": 3_061_129_077 + 1_610_612_736,
    "llama.cpp": 3_413_361_504 + 1_610_612_736,
}


def main() -> int:
    backend = local_backend()
    return run_worker(
        execute,
        capabilities=CAPABILITIES,
        description="ClipMill local editorial worker",
        backend=backend,
        max_memory_bytes=memory_ceiling(SMALLEST_MODEL_BYTES[backend]),
    )


def local_backend() -> str:
    """The runtime this machine's editorial models run on: MLX on Apple
    silicon, llama.cpp everywhere else."""
    return "mlx" if sys.platform == "darwin" and platform.machine() == "arm64" else "llama.cpp"


def run_worker(executor, *, capabilities, description, backend, max_memory_bytes) -> int:
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument(
        "--identity",
        type=Path,
        default=os.environ.get("CLIPMILL_WORKER_IDENTITY"),
        required=os.environ.get("CLIPMILL_WORKER_IDENTITY") is None,
    )
    parser.add_argument("--data-dir", type=Path, default=os.environ.get("CLIPMILL_DATA_DIR"))
    parser.add_argument(
        "--worker-socket",
        type=Path,
        default=os.environ.get("CLIPMILL_WORKER_SOCKET"),
    )
    parser.add_argument("--shm-socket", type=Path)
    arguments = parser.parse_args()
    data_dir = arguments.data_dir
    worker_socket = arguments.worker_socket
    if worker_socket is None:
        if data_dir is None:
            parser.error("--worker-socket or --data-dir is required")
        worker_socket = data_dir / "run" / "clipmill-workers.sock"
    shm_socket = arguments.shm_socket
    if shm_socket is None:
        if data_dir is None:
            shm_socket = worker_socket.parent / "clipmill-shm.sock"
        else:
            shm_socket = data_dir / "run" / "clipmill-shm.sock"

    logging.basicConfig(level=os.environ.get("CLIPMILL_WORKER_LOG", "INFO"))
    stop = threading.Event()

    def request_stop(_signum: int, _frame: object) -> None:
        stop.set()

    signal.signal(signal.SIGINT, request_stop)
    signal.signal(signal.SIGTERM, request_stop)
    from .runtime import RESIDENT

    client = WorkerClient(
        WorkerConfiguration(
            socket_path=worker_socket,
            shm_socket_path=shm_socket,
            identity=WorkerIdentity.load(arguments.identity),
            family="editorial",
            backend=backend,
            cpu_threads=1,
            has_resident_model=lambda: RESIDENT.resident is not None,
            release_resident_model=RESIDENT.close,
            max_memory_bytes=max_memory_bytes,
            capabilities=capabilities,
        )
    )
    try:
        client.run(executor, stop)
    finally:
        RESIDENT.close()
    return 0
