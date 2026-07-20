"""Cold/warm source-evidence evaluation runner and canonical run manifests."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from clipmill.ipc.v1 import daemon_pb2

from .artifacts import VerifiedArtifact, verify_artifact
from .client import DaemonClient, DaemonClientError
from .corpus import CorpusItem, VerifiedCorpus
from .profiles import verify_device_profile
from .signing import canonical_json


class EvaluationError(RuntimeError):
    """The daemon or an evaluation invariant failed."""


def run_corpus(
    client: DaemonClient,
    data_dir: Path,
    corpus: VerifiedCorpus,
) -> dict[str, Any]:
    started_unix_millis = _unix_millis()
    health = client.health()
    if not health.local_lock:
        raise EvaluationError("evaluation requires Local Lock")
    profile_response = client.get_device_profile()
    profile = verify_device_profile(profile_response.profile_json)
    profile_artifact = verify_artifact(data_dir, profile_response.artifact_id)
    _verify_profile_payload(profile_artifact, profile_response.profile_json)
    project = client.create_project(f"evaluation-{corpus.corpus_id}")
    item_results = [
        _run_item(client, data_dir, corpus, project.project_id, item) for item in corpus.items
    ]
    completed_unix_millis = _unix_millis()
    return {
        "schema_version": "clipmill.eval.run.v1",
        "corpus_id": corpus.corpus_id,
        "corpus_signing_key": corpus.signing_public_key.hex(),
        "daemon_version": health.daemon_version,
        "hardware_profile": {
            "artifact_id": profile_response.artifact_id,
            "hardware_fingerprint": profile.hardware_fingerprint,
            "measurement_generation": profile.measurement_generation,
        },
        "items": item_results,
        "policy": "local-lock",
        "started_unix_millis": started_unix_millis,
        "completed_unix_millis": completed_unix_millis,
    }


def write_run_manifest(path: Path, manifest: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_json(manifest) + b"\n")


def _run_item(
    client: DaemonClient,
    data_dir: Path,
    corpus: VerifiedCorpus,
    project_id: str,
    item: CorpusItem,
) -> dict[str, Any]:
    path = corpus.path_for(item)
    cold_started = time.monotonic_ns()
    try:
        registered = client.register_source(project_id, path)
    except DaemonClientError as error:
        if item.expected_result != "structured_failure":
            raise EvaluationError(f"valid item {item.item_id} failed: {error}") from error
        _require_expected_failure(item, error.message)
        return {
            "item_id": item.item_id,
            "expected_result": item.expected_result,
            "observed_result": "structured_failure",
            "failure_code": error.code,
            "failure": item.expected_failure,
            "cold_millis": _elapsed_millis(cold_started),
        }
    source = registered.source
    job = client.submit_probe(project_id, source.source_id)
    job = client.wait_for_job(job.job_id)
    if item.expected_result == "structured_failure":
        if job.state != daemon_pb2.JOB_STATE_FAILED:
            raise EvaluationError(f"hostile item {item.item_id} unexpectedly succeeded")
        _require_expected_failure(item, job.failure_detail)
        return {
            "item_id": item.item_id,
            "expected_result": item.expected_result,
            "observed_result": "structured_failure",
            "failure_code": job.failure_class,
            "failure": item.expected_failure,
            "cold_millis": _elapsed_millis(cold_started),
        }
    if job.state != daemon_pb2.JOB_STATE_SUCCEEDED or len(job.output_artifact_ids) != 1:
        raise EvaluationError(f"valid item {item.item_id} probe failed: {job.failure_detail}")
    cold_artifact_id = job.output_artifact_ids[0]
    source_map = verify_artifact(data_dir, cold_artifact_id)
    _verify_source_map(source_map)
    cold_millis = _elapsed_millis(cold_started)

    warm_started = time.monotonic_ns()
    warm_registration = client.register_source(project_id, path)
    if not warm_registration.observation_cache_hit:
        raise EvaluationError(f"warm registration missed for {item.item_id}")
    if warm_registration.source.source_id != source.source_id:
        raise EvaluationError(f"warm registration changed source ID for {item.item_id}")
    warm_job = client.submit_probe(project_id, source.source_id)
    warm_job = client.wait_for_job(warm_job.job_id)
    if warm_job.state != daemon_pb2.JOB_STATE_SUCCEEDED or list(warm_job.output_artifact_ids) != [
        cold_artifact_id
    ]:
        raise EvaluationError(f"warm artifact identity changed for {item.item_id}")
    verify_artifact(data_dir, cold_artifact_id)
    return {
        "item_id": item.item_id,
        "expected_result": item.expected_result,
        "observed_result": "success",
        "source_fingerprint": source.source_fingerprint,
        "source_map_artifact_id": cold_artifact_id,
        "cold_millis": cold_millis,
        "warm_millis": _elapsed_millis(warm_started),
        "warm_cache_hit": True,
    }


def _verify_source_map(artifact: VerifiedArtifact) -> None:
    if artifact.kind != "evidence.source_map.v1" or artifact.stage != "probe-source":
        raise EvaluationError("source-map artifact kind or producer is invalid")
    payload = artifact.object_directory / "source-map.json"
    try:
        value = json.loads(payload.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise EvaluationError(f"source-map payload is invalid: {error}") from error
    if not isinstance(value, dict) or not isinstance(value.get("mapping"), dict):
        raise EvaluationError("new Phase 0 source map omitted exact timestamp mapping")


def _verify_profile_payload(artifact: VerifiedArtifact, expected: str) -> None:
    if artifact.kind != "evidence.device_profile.v1" or artifact.stage != "device-profile":
        raise EvaluationError("device-profile artifact kind or producer is invalid")
    try:
        actual = (artifact.object_directory / "profile.json").read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise EvaluationError(f"device-profile payload is unreadable: {error}") from error
    if actual != expected:
        raise EvaluationError("device-profile RPC bytes differ from its verified artifact")


def _require_expected_failure(item: CorpusItem, observed: str) -> None:
    expected = item.expected_failure
    if expected is None or expected.casefold() not in observed.casefold():
        raise EvaluationError(f"hostile item {item.item_id} failed differently: {observed}")


def _unix_millis() -> int:
    return time.time_ns() // 1_000_000


def _elapsed_millis(started_ns: int) -> int:
    return max(0, (time.monotonic_ns() - started_ns) // 1_000_000)
