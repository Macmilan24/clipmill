#!/usr/bin/env python3
"""Verify one local Qwen JSON generation before admitting Metal workers.

This is a runtime readiness check, not a model comparison or quality benchmark.
Run with workers/editorial/.venv/bin/python while the daemon is running.
"""

import argparse
import hashlib
import json
import os
import resource
import subprocess
import sys
import time
import tomllib
from pathlib import Path

# Resolve the in-repository evaluation client before importing it.
# ruff: noqa: E402
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "eval/harness/src"))
from clipmill.worker.v1 import worker_pb2
from clipmill_eval.client import DaemonClient
from clipmill_eval.profiles import verify_device_profile
from clipmill_worker_editorial.runtime import LocalModel
from clipmill_worker_sdk import CancellationToken
from clipmill_worker_sdk.weights import verify_model


def binding():
    registry = Path(os.environ.get("CLIPMILL_MODELS_DIR", ROOT / "models/registry"))
    weights = Path(os.environ.get("CLIPMILL_WEIGHTS_DIR", ROOT / ".cache/models"))
    manifest = tomllib.loads((registry / "qwen3-5-editorial-mlx.toml").read_text())
    digest = hashlib.sha256(b"clipmill.model.identity.v1\0")
    for field in [
        manifest["name"],
        manifest["source"]["repo"],
        manifest["source"]["revision"],
        manifest["quantization"],
    ]:
        digest.update(field.encode() + b"\0")
    for entry in sorted(manifest["files"], key=lambda f: (f["path"], f["sha256"])):
        digest.update(entry["path"].encode() + b"\0" + entry["sha256"].encode() + b"\0")
    return worker_pb2.ModelBinding(
        name=manifest["name"],
        capability="editorial",
        digest="sha256:" + digest.hexdigest(),
        root=str(weights / manifest["name"]),
        files=[worker_pb2.ModelFile(**f) for f in manifest["files"]],
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data-dir", type=Path, required=True)
    parser.add_argument("--socket", type=Path)
    parser.add_argument("--generate-only", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    socket = args.socket or Path(
        os.environ.get("CLIPMILL_SOCKET", str(args.data_dir / "run/clipmilld.sock"))
    )
    client = DaemonClient(socket, timeout_seconds=180)
    profile = verify_device_profile(client.get_device_profile().profile_json)
    model_binding = binding()
    destination = args.data_dir / "state/editorial-runtime.json"
    try:
        previous = json.loads(destination.read_text())
    except (OSError, ValueError):
        previous = {}
    if not isinstance(previous, dict):
        previous = {}
    if (
        previous.get("schema_version") == "clipmill.editorial.runtime.v1"
        and previous.get("runtime") == "mlx-vlm@0.7.1/clipmill-json-v2"
        and isinstance(previous.get("elapsed_millis"), int)
        and previous["elapsed_millis"] > 0
        and isinstance(previous.get("peak_resident_bytes"), int)
        and previous["peak_resident_bytes"] > 0
        and previous.get("hardware_fingerprint") == profile.hardware_fingerprint
        and previous.get("model_digest") == model_binding.digest
        and previous.get("validated") is True
    ):
        client.get_device_profile(remeasure=True)
        print("editorial-runtime: existing successful check matches this machine and model")
        return
    if not args.generate_only:
        # Measure after the inference process exits. MLX/processor allocations
        # may outlive Python objects; measuring inside that process counts the
        # readiness check itself as unavailable memory for all future work.
        subprocess.run(
            [
                sys.executable,
                __file__,
                "--data-dir",
                str(args.data_dir),
                "--socket",
                str(socket),
                "--generate-only",
            ],
            check=True,
        )
        client.get_device_profile(remeasure=True)
        print(
            "editorial-runtime: real Qwen JSON generation passed; Metal worker admission refreshed"
        )
        return
    model = verify_model(model_binding)
    started = time.monotonic()
    runtime = LocalModel(model.root, CancellationToken())
    try:
        raw, tokens = runtime.generate(
            'Reply with the JSON object {"ready":true}.',
            {
                "type": "object",
                "properties": {"ready": {"const": True}},
                "required": ["ready"],
                "additionalProperties": False,
            },
            32,
        )
        if json.loads(raw) != {"ready": True}:
            raise RuntimeError("model did not complete the readiness JSON")
    finally:
        runtime.close()
    document = {
        "schema_version": "clipmill.editorial.runtime.v1",
        "runtime": "mlx-vlm@0.7.1/clipmill-json-v2",
        "hardware_fingerprint": profile.hardware_fingerprint,
        "model_digest": model.digest,
        "validated": True,
        "elapsed_millis": max(1, int((time.monotonic() - started) * 1000)),
        "peak_resident_bytes": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
        "tokens": tokens,
    }
    pending = destination.with_suffix(".pending")
    fd = os.open(pending, os.O_CREAT | os.O_TRUNC | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "w") as f:
        json.dump(document, f)
        f.flush()
        os.fsync(f.fileno())
    os.replace(pending, destination)


if __name__ == "__main__":
    main()
