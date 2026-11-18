"""Text embeddings for meaning search (plan steps 1.8 and 2.9).

Two stages on one model. `text-embed` turns a recording's speech (as passages)
and its remembered events (as titles) into unit vectors; `text-embed-query`
turns the words someone searched for into one, so a search is a dot product.
"""

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
    TaskContext,
    WorkerClient,
    WorkerConfiguration,
    WorkerIdentity,
)
from clipmill_worker_sdk.documents import canonical_bytes
from clipmill_worker_sdk.gen.schemas.text_embeddings import TextEmbeddings
from clipmill_worker_sdk.gen.schemas.text_query_vector import TextQueryVector
from clipmill_worker_sdk.weights import ModelVerificationError, require_model

from .encoder import IMPLEMENTATION, Encoder
from .passages import passages

__version__ = "0.1.0"
CAPABILITIES = ("text-embed", "text-embed-query")
KEY_VERSION = "clipmill.text-embed-stage.v1"
MAX_QUERY_CHARS = 500

LOGGER = logging.getLogger(__name__)


def _payload(context: TaskContext) -> daemon_pb2.TextEmbedPayloadV1:
    payload = daemon_pb2.TextEmbedPayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        raise DeterministicTaskError("task payload is not a text embedding payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != context.lease.kind:
        raise DeterministicTaskError("task payload does not describe this embedding stage")
    return payload


def _encoder(context: TaskContext):
    try:
        model = require_model(context.lease, "embed")
    except ModelVerificationError as error:
        raise DeterministicTaskError(str(error)) from error
    return model, Encoder(model)


def execute(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = _payload(context)
    if context.lease.kind == "text-embed-query":
        return _query(context, payload)
    return _recording(context, payload)


def _query(context: TaskContext, payload) -> tuple[str, ...]:
    query = payload.query.strip()
    if not query or len(query) > MAX_QUERY_CHARS:
        raise DeterministicTaskError("a search is between 1 and 500 characters")
    model, encoder = _encoder(context)
    vector = encoder.encode([f"query: {query}"])[0]
    document = TextQueryVector(
        schema_version="clipmill.text.query_vector.v1",
        model_digest=model.digest,
        query=query,
        vector=[round(float(value), 6) for value in vector],
    )
    context.staging.write_bytes("vector.json", canonical_bytes(document))
    return ("vector.json",)


def _recording(context: TaskContext, payload) -> tuple[str, ...]:
    inputs = LeaseInputs(context)
    index_input = inputs.require("index.transcript.v1")
    index = json.loads(context.artifact_file(index_input.artifact, "index.json").read_text())
    memory_input = inputs.optional("mind.memory.v1")
    memory = (
        json.loads(context.artifact_file(memory_input.artifact, "memory.json").read_text())
        if memory_input
        else None
    )
    fingerprint = payload.source_fingerprint or index["source_fingerprint"]
    if index["source_fingerprint"] != fingerprint or (
        memory and memory["source_fingerprint"] != fingerprint
    ):
        raise DeterministicTaskError("the embedding inputs describe different recordings")

    units: list[dict] = []
    texts: list[str] = []
    for passage in passages(index.get("sentences", [])):
        units.append(
            {
                "kind": "passage",
                "id": f"s{passage.first}-s{passage.last}",
                "first_unit": passage.first,
                "last_unit": passage.last,
            }
        )
        texts.append(f"passage: {passage.text}")
    for event in (memory or {}).get("events", []):
        units.append(
            {
                "kind": "event",
                "id": event["event_id"],
                "first_unit": int(event["first_unit"][1:]),
                "last_unit": int(event["last_unit"][1:]),
            }
        )
        texts.append(f"passage: {event['title']}")

    model, encoder = _encoder(context)

    def progress(done: int, total: int) -> None:
        context.cancellation.raise_if_cancelled()
        context.report_progress("passages", done, total)

    vectors = encoder.encode(texts, on_batch=progress)
    document = TextEmbeddings(
        schema_version="clipmill.text.embeddings.v1",
        source_fingerprint=fingerprint,
        producer={
            "stage": "text-embed",
            "implementation": f"clipmill-worker-embed@{__version__}+{IMPLEMENTATION}",
            "model_digest": model.digest,
        },
        model={"name": model.name, "dims": encoder.dims, "pooling": "mean"},
        dtype="float16",
        vectors_file="vectors.f16",
        units=units,
    )
    context.staging.write_bytes("vectors.f16", vectors.astype("<f2").tobytes())
    context.staging.write_bytes("embeddings.json", canonical_bytes(document))
    return ("embeddings.json", "vectors.f16")


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill text embedding worker")
    parser.add_argument(
        "--identity",
        type=Path,
        default=os.environ.get("CLIPMILL_WORKER_IDENTITY"),
        required=os.environ.get("CLIPMILL_WORKER_IDENTITY") is None,
    )
    parser.add_argument("--data-dir", type=Path, default=os.environ.get("CLIPMILL_DATA_DIR"))
    parser.add_argument(
        "--worker-socket", type=Path, default=os.environ.get("CLIPMILL_WORKER_SOCKET")
    )
    parser.add_argument("--shm-socket", type=Path)
    parser.add_argument("--once", action="store_true", help="run at most one lease, then exit")
    arguments = parser.parse_args()

    worker_socket = arguments.worker_socket
    if worker_socket is None:
        if arguments.data_dir is None:
            parser.error("--worker-socket or --data-dir is required")
        worker_socket = arguments.data_dir / "run" / "clipmill-workers.sock"
    shm_socket = arguments.shm_socket or worker_socket.parent / "clipmill-shm.sock"

    logging.basicConfig(level=os.environ.get("CLIPMILL_WORKER_LOG", "INFO"))
    stop = threading.Event()
    signal.signal(signal.SIGINT, lambda *_: stop.set())
    signal.signal(signal.SIGTERM, lambda *_: stop.set())

    client = WorkerClient(
        WorkerConfiguration(
            socket_path=worker_socket,
            shm_socket_path=shm_socket,
            identity=WorkerIdentity.load(arguments.identity),
            family="embed",
            capabilities=CAPABILITIES,
            backend="onnx-cpu",
            cpu_threads=1,
            max_memory_bytes=512 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute)
    else:
        client.run(execute, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "execute", "main"]
