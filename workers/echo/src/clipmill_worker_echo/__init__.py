"""Standalone reference worker for the durable ``demo-dag`` task family."""

from __future__ import annotations

import argparse
import hashlib
import json
import logging
import os
import signal
import threading
import time
from pathlib import Path

from clipmill_worker_sdk import (
    DeterministicTaskError,
    TaskContext,
    WorkerClient,
    WorkerConfiguration,
    WorkerIdentity,
)

__version__ = "0.0.1"
CAPABILITIES = ("demo-join", "demo-left", "demo-right", "demo-seed")
_DELAY_LOCK = threading.Lock()
_DELAY_ONCE_CONSUMED = False


def execute_echo(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    delay_ms = _task_delay_ms()
    deadline = time.monotonic() + max(delay_ms, 0) / 1000
    while time.monotonic() < deadline:
        context.cancellation.raise_if_cancelled()
        time.sleep(min(0.05, max(deadline - time.monotonic(), 0)))
    if context.lease.kind not in CAPABILITIES:
        raise DeterministicTaskError("unsupported demo task")
    if context.shared.buffer is None or context.shared.buffer.to_pybytes() != context.lease.payload:
        raise DeterministicTaskError("shared payload mismatch")
    output = {
        "inputs": list(context.lease.input_artifact_ids),
        "kind": context.lease.kind,
        "payload_sha256": f"sha256:{hashlib.sha256(context.lease.payload).hexdigest()}",
    }
    encoded = json.dumps(
        output,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode()
    context.staging.write_bytes("result.json", encoded)
    context.report_progress("items", 1, 1)
    return ("result.json",)


def _task_delay_ms() -> int:
    global _DELAY_ONCE_CONSUMED

    always = int(os.environ.get("CLIPMILL_ECHO_DELAY_MS", "0"))
    once = int(os.environ.get("CLIPMILL_ECHO_DELAY_ONCE_MS", "0"))
    with _DELAY_LOCK:
        if once > 0 and not _DELAY_ONCE_CONSUMED:
            _DELAY_ONCE_CONSUMED = True
            return min(once, 30_000)
    return min(max(always, 0), 30_000)


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill authenticated echo worker")
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
            family="echo",
            capabilities=CAPABILITIES,
        )
    )
    client.run(execute_echo, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "execute_echo", "main"]
