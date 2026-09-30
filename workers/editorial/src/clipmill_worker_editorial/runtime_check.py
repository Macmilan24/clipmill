"""Prove the local editorial model runs on this computer before Metal work.

The daemon admits a Metal worker only after one real generation by the model
it plans for editorial work has succeeded on this machine: a receipt naming the
machine's hardware fingerprint and the model's digest, which the next device
measurement reads (selection.rs). This is a runtime check, not a benchmark of
editorial quality.

A packaged app's daemon runs this module in the editorial component's own
environment, handing it the model binding it would lease:

    python -I -m clipmill_worker_editorial.runtime_check \\
        --binding <ModelBinding protobuf> --receipt <state/editorial-runtime.json> \\
        --fingerprint <hardware fingerprint>

A development checkout runs tools/editorial-runtime-check.py, which calls
`prove` the same way.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk import CancellationToken
from clipmill_worker_sdk.weights import verify_model

from .runtime import MLX_RUNTIME, open_runtime, runtime_name

SCHEMA_VERSION = "clipmill.editorial.runtime.v1"
# The MLX runtime's name, which tools/editorial-runtime-check.py compares.
RUNTIME = MLX_RUNTIME
READY_SCHEMA = {
    "type": "object",
    "properties": {"ready": {"const": True}},
    "required": ["ready"],
    "additionalProperties": False,
}
# A representative reply for the speed Models reports: a prompt of about
# 1,500 tokens and an answer of about 80. The readiness reply is too short to
# say how fast a long prompt is read, and a GPU reads one many times faster.
MEASURE_PROMPT = "Read the passage, then list twenty words from it.\n\n" + " ".join(
    ["The editor watches the talk and marks the moments worth keeping."] * 120
)
MEASURE_SCHEMA = {
    "type": "object",
    "properties": {
        "words": {
            "type": "array",
            "items": {"type": "string", "maxLength": 16},
            "minItems": 20,
            "maxItems": 20,
        }
    },
    "required": ["words"],
    "additionalProperties": False,
}


def prove(binding: worker_pb2.ModelBinding, receipt: Path, fingerprint: str) -> dict:
    """Generate one JSON reply with the pinned model and write the receipt.

    Every file is hashed against its pin before the model loads. The receipt
    is written only after the reply parses, privately and atomically.
    """

    model = verify_model(binding)
    started = time.monotonic()
    runtime = open_runtime(model, CancellationToken())
    try:
        raw, tokens = runtime.generate(
            'Reply with the JSON object {"ready":true}.', READY_SCHEMA, 32
        )
        if json.loads(raw) != {"ready": True}:
            raise RuntimeError("the model did not complete the readiness JSON")
        # llama.cpp reports its speed; MLX does not, and the Mac needs no warning.
        timings = {}
        if hasattr(runtime, "last_timings"):
            raw, _ = runtime.generate(MEASURE_PROMPT, MEASURE_SCHEMA, 256)
            json.loads(raw)
            timings = runtime.last_timings
    finally:
        runtime.close()
    document = {
        "schema_version": SCHEMA_VERSION,
        "runtime": runtime_name(model),
        "hardware_fingerprint": fingerprint,
        "model_digest": model.digest,
        "validated": True,
        "elapsed_millis": max(1, int((time.monotonic() - started) * 1000)),
        "peak_resident_bytes": _peak_resident_bytes(),
        "tokens": tokens,
    }
    if timings:
        # What llama-server measured for the representative reply: reading
        # the prompt and writing the answer, in tokens a second.
        document["prompt_tokens_per_second"] = timings.get("prompt_per_second", 0)
        document["output_tokens_per_second"] = timings.get("predicted_per_second", 0)
    pending = receipt.with_suffix(".pending")
    # O_NOFOLLOW is POSIX only; on Windows the data folder's access list is
    # the boundary.
    flags = os.O_CREAT | os.O_TRUNC | os.O_WRONLY | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(pending, flags, 0o600)
    with os.fdopen(descriptor, "w") as handle:
        json.dump(document, handle)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(pending, receipt)
    return document


def _peak_resident_bytes() -> int:
    """This process's peak memory where the platform reports it (macOS in
    bytes, Linux in kilobytes), and 0 on Windows. For a GGUF model the
    weights live in llama-server, so this is the worker alone."""
    try:
        import resource
    except ImportError:
        return 0
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return peak if sys.platform == "darwin" else peak * 1024


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binding", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--fingerprint", required=True)
    arguments = parser.parse_args()
    binding = worker_pb2.ModelBinding.FromString(arguments.binding.read_bytes())
    document = prove(binding, arguments.receipt, arguments.fingerprint)
    print(
        f"editorial-runtime: {binding.name} replied in {document['elapsed_millis']} ms",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
