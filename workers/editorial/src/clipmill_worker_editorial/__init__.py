"""Authenticated local editorial worker and shared lease-scoped execution."""

from __future__ import annotations

import argparse
import json
import logging
import os
import signal
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
    require_model,
)

from .inference import look, prompt_digest, propose, review

CAPABILITIES = (
    "editorial-look",
    "editorial-propose",
    "editorial-review",
)


def execute(context: TaskContext) -> tuple[str, ...]:
    """Local entrypoint: cloud leases are rejected before any runtime is opened."""
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
    operation = payload.stage.removeprefix("editorial-").removesuffix("-cloud")
    cloud = payload.stage.endswith("-cloud")
    if cloud != payload.HasField("cloud"):
        raise DeterministicTaskError("cloud permission does not match the task route")
    if cloud and cloud_runtime is None:
        raise DeterministicTaskError("this worker does not execute cloud tasks")
    if payload.prompt_digest != prompt_digest(operation) or payload.max_output_tokens != 2048:
        raise DeterministicTaskError(
            "editorial prompt or decoding policy differs from the planned recipe; update the worker"
        )
    model = None if cloud else require_model(context.lease, "editorial")
    inputs = LeaseInputs(context)
    if operation == "look":
        return execute_look(context, inputs, model, payload)
    windows_input = inputs.require("editorial.windows.v1")
    windows = json.loads(context.artifact_file(windows_input.artifact, "windows.json").read_text())
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
        from .runtime import LocalModel

        return LocalModel(model.root, context.cancellation)

    producer = {
        "stage": payload.stage,
        "implementation": f"clipmill-worker-editorial@0.1.1/{operation}{'-cloud' if cloud else ''}",
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
        fd = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW, 0o600)
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
    from .runtime import LocalModel

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
    identity = {"route": "local", "model": {"name": model.name, "digest": model.digest}}
    producer = {
        "stage": payload.stage,
        "implementation": "clipmill-worker-editorial@0.1.1/look",
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
        lambda: LocalModel(model.root, context.cancellation),
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
        "source_fingerprint": candidates["source_fingerprint"],
        "inputs": {"judgments_artifact_id": ji.artifact_id, "frames_artifact_id": fi.artifact_id},
        "producer": producer,
        "checks": checks,
    }
    trace = {
        "schema_version": "clipmill.editorial.trace.v1",
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


def main() -> int:
    return run_worker(
        execute,
        capabilities=CAPABILITIES,
        description="ClipMill local Qwen 3.5 editorial worker",
        backend="mlx",
        max_memory_bytes=5_977_071_067 + 2 * 1024**3,
    )


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
    client = WorkerClient(
        WorkerConfiguration(
            socket_path=worker_socket,
            shm_socket_path=shm_socket,
            identity=WorkerIdentity.load(arguments.identity),
            family="editorial",
            backend=backend,
            cpu_threads=1,
            max_memory_bytes=max_memory_bytes,
            capabilities=capabilities,
        )
    )
    client.run(executor, stop)
    return 0
