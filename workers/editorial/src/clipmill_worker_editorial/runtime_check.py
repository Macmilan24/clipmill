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
import resource
import sys
import time
from pathlib import Path

from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk import CancellationToken
from clipmill_worker_sdk.weights import verify_model

from .runtime import LocalModel

SCHEMA_VERSION = "clipmill.editorial.runtime.v1"
RUNTIME = "mlx-vlm@0.7.1/clipmill-json-v2"
READY_SCHEMA = {
    "type": "object",
    "properties": {"ready": {"const": True}},
    "required": ["ready"],
    "additionalProperties": False,
}


def prove(binding: worker_pb2.ModelBinding, receipt: Path, fingerprint: str) -> dict:
    """Generate one JSON reply with the pinned model and write the receipt.

    Every file is hashed against its pin before the model loads. The receipt
    is written only after the reply parses, privately and atomically.
    """

    model = verify_model(binding)
    started = time.monotonic()
    runtime = LocalModel(model.root, CancellationToken())
    try:
        raw, tokens = runtime.generate(
            'Reply with the JSON object {"ready":true}.', READY_SCHEMA, 32
        )
        if json.loads(raw) != {"ready": True}:
            raise RuntimeError("the model did not complete the readiness JSON")
    finally:
        runtime.close()
    document = {
        "schema_version": SCHEMA_VERSION,
        "runtime": RUNTIME,
        "hardware_fingerprint": fingerprint,
        "model_digest": model.digest,
        "validated": True,
        "elapsed_millis": max(1, int((time.monotonic() - started) * 1000)),
        "peak_resident_bytes": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
        "tokens": tokens,
    }
    pending = receipt.with_suffix(".pending")
    descriptor = os.open(pending, os.O_CREAT | os.O_TRUNC | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
    with os.fdopen(descriptor, "w") as handle:
        json.dump(document, handle)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(pending, receipt)
    return document


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
