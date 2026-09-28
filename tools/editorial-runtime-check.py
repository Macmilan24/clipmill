#!/usr/bin/env python3
"""Verify one local editorial JSON generation before admitting Metal workers.

The model checked is the one the daemon plans for editorial work: the person's
choice in Models, or the bundled Qwen3.5 9B.

This is a runtime readiness check, not a model comparison or quality benchmark.
Run with workers/editorial/.venv/bin/python while the daemon is running.
"""

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tomllib
from pathlib import Path

# Resolve the in-repository evaluation client before importing it.
# ruff: noqa: E402
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "eval/harness/src"))
from clipmill.worker.v1 import worker_pb2
from clipmill_eval.client import DaemonClient
from clipmill_eval.profiles import verify_device_profile
from clipmill_worker_editorial.runtime_check import RUNTIME, SCHEMA_VERSION, prove

DEFAULT_MODEL = "qwen3-5-editorial-mlx"


def chosen_manifest(data_dir):
    """The editorial model the daemon plans: the person's choice, else Qwen.

    A choice names either a bundled manifest or one pinned in the app, which
    the daemon keeps as JSON under the state directory.
    """

    registry = Path(os.environ.get("CLIPMILL_MODELS_DIR", ROOT / "models/registry"))
    name = DEFAULT_MODEL
    try:
        choices = json.loads((data_dir / "state/model-choices.json").read_text())
        if choices.get("schema_version") == "clipmill.model_choices.v1":
            name = choices.get("choices", {}).get("editorial") or DEFAULT_MODEL
    except (OSError, ValueError, AttributeError):
        name = DEFAULT_MODEL
    pinned = data_dir / "state/models" / f"{name}.json"
    if pinned.is_file():
        return json.loads(pinned.read_text())
    bundled = registry / f"{name}.toml"
    if bundled.is_file():
        return tomllib.loads(bundled.read_text())
    return tomllib.loads((registry / f"{DEFAULT_MODEL}.toml").read_text())


def binding(data_dir):
    weights = Path(os.environ.get("CLIPMILL_WEIGHTS_DIR", ROOT / ".cache/models"))
    manifest = chosen_manifest(data_dir)
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
    model_binding = binding(args.data_dir)
    destination = args.data_dir / "state/editorial-runtime.json"
    try:
        previous = json.loads(destination.read_text())
    except (OSError, ValueError):
        previous = {}
    if not isinstance(previous, dict):
        previous = {}
    if (
        previous.get("schema_version") == SCHEMA_VERSION
        and previous.get("runtime") == RUNTIME
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
            f"editorial-runtime: a real JSON generation by {model_binding.name} passed; "
            "Metal worker admission refreshed"
        )
        return
    # The same proof a packaged app's daemon runs in its editorial component.
    prove(model_binding, destination, profile.hardware_fingerprint)


if __name__ == "__main__":
    main()
