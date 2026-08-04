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
        "versions": {
            "artifact_manifest": "clipmill.artifact.manifest.v1",
            "evaluation": "clipmill.eval.run.v1",
            "source_map": "clipmill.source_map.v1",
        },
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
        cold_millis = _elapsed_millis(cold_started)
        warm_started = time.monotonic_ns()
        try:
            client.register_source(project_id, path)
        except DaemonClientError as warm_error:
            _require_expected_failure(item, warm_error.message)
            if warm_error.code != error.code:
                raise EvaluationError(
                    f"hostile item {item.item_id} changed failure code on warm run"
                ) from warm_error
        else:
            raise EvaluationError(
                f"hostile item {item.item_id} unexpectedly registered on warm run"
            )
        return {
            "item_id": item.item_id,
            "expected_result": item.expected_result,
            "observed_result": "structured_failure",
            "failure_code": error.code,
            "failure": item.expected_failure,
            "cold_millis": cold_millis,
            "warm_millis": _elapsed_millis(warm_started),
            "warm_observed_result": "structured_failure",
        }
    source = registered.source
    job = client.submit_probe(project_id, source.source_id)
    job = client.wait_for_job(job.job_id)
    if item.expected_result == "structured_failure":
        if job.state != daemon_pb2.JOB_STATE_FAILED:
            raise EvaluationError(f"hostile item {item.item_id} unexpectedly succeeded")
        _require_expected_failure(item, job.failure_detail)
        cold_millis = _elapsed_millis(cold_started)
        warm_started = time.monotonic_ns()
        warm_registration = client.register_source(project_id, path)
        if not warm_registration.observation_cache_hit:
            raise EvaluationError(f"hostile item {item.item_id} missed warm observation cache")
        warm_job = client.submit_probe(project_id, warm_registration.source.source_id)
        warm_job = client.wait_for_job(warm_job.job_id)
        if warm_job.state != daemon_pb2.JOB_STATE_FAILED:
            raise EvaluationError(f"hostile item {item.item_id} changed outcome on warm run")
        _require_expected_failure(item, warm_job.failure_detail)
        if warm_job.failure_class != job.failure_class:
            raise EvaluationError(f"hostile item {item.item_id} changed failure class on warm run")
        return {
            "item_id": item.item_id,
            "expected_result": item.expected_result,
            "observed_result": "structured_failure",
            "failure_code": job.failure_class,
            "failure": item.expected_failure,
            "cold_millis": cold_millis,
            "warm_millis": _elapsed_millis(warm_started),
            "warm_observed_result": "structured_failure",
        }
    if job.state != daemon_pb2.JOB_STATE_SUCCEEDED or len(job.output_artifact_ids) != 1:
        raise EvaluationError(f"valid item {item.item_id} probe failed: {job.failure_detail}")
    cold_artifact_id = job.output_artifact_ids[0]
    source_map = verify_artifact(data_dir, cold_artifact_id)
    version_evidence = _verify_source_map(source_map, source.source_fingerprint)
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
        "cold_source_map_artifact_id": cold_artifact_id,
        "warm_source_map_artifact_id": warm_job.output_artifact_ids[0],
        **version_evidence,
        "cold_millis": cold_millis,
        "warm_millis": _elapsed_millis(warm_started),
        "warm_cache_hit": True,
    }


def _verify_source_map(artifact: VerifiedArtifact, source_fingerprint: str) -> dict[str, Any]:
    if artifact.kind != "evidence.source_map.v1" or artifact.stage != "probe-source":
        raise EvaluationError("source-map artifact kind or producer is invalid")
    manifest = artifact.manifest
    producer = manifest.get("producer")
    recipe = manifest.get("recipe")
    config = recipe.get("config") if isinstance(recipe, dict) else None
    if (
        manifest.get("source_fingerprint") != source_fingerprint
        or manifest.get("policy") != "local-lock"
        or not isinstance(producer, dict)
        or not isinstance(producer.get("implementation"), str)
        or not producer["implementation"]
        or not isinstance(recipe, dict)
        or recipe.get("key_version") != "clipmill.artifact.key.v1"
        or recipe.get("semantic_version") != "clipmill.source_map.v1"
        or not isinstance(config, dict)
        or config.get("ffmpeg_bom") != "ffmpeg-8.1.2-btb-n8.1.2"
        or config.get("probe_algorithm") != "clipmill.ffprobe.normalize.v1"
        or config.get("mapping_algorithm") != "clipmill.source-map.mapping.v1"
    ):
        raise EvaluationError("source-map recipe, policy, or source identity is invalid")
    payload = artifact.object_directory / "source-map.json"
    try:
        value = json.loads(payload.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise EvaluationError(f"source-map payload is invalid: {error}") from error
    if (
        not isinstance(value, dict)
        or value.get("schema_version") != "clipmill.source_map.v1"
        or not isinstance(value.get("mapping"), dict)
        or not isinstance(value.get("streams"), list)
    ):
        raise EvaluationError("new Phase 0 source map omitted exact timestamp mapping")
    segments = value["mapping"].get("segments")
    chapters = value.get("chapters", [])
    if not isinstance(segments, list) or not isinstance(chapters, list):
        raise EvaluationError("source-map metric collections are invalid")
    return {
        "artifact_key_version": recipe["key_version"],
        "ffmpeg_bom": config["ffmpeg_bom"],
        "mapping_algorithm": config["mapping_algorithm"],
        "probe_algorithm": config["probe_algorithm"],
        "producer_implementation": producer["implementation"],
        "source_map_schema_version": value["schema_version"],
        "source_map_metrics": {
            "chapters": len(chapters),
            "mapping_segments": len(segments),
            "streams": len(value["streams"]),
        },
    }


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


# ---- W26: recall against annotated ground truth -----------------------------
#
# Kept apart from `run_corpus` on purpose. That function is the Phase-0 ingest
# protocol — cold and warm registration, byte-for-byte cache identity — and it
# must keep meaning what it meant. This is a different claim about a different
# stage, and folding the two together would make one failing suite look like
# the other.


CANDIDATES_KIND = "discovery.candidates.v1"
RANKING_KIND = "ranking.set.v1"
INDEX_KIND = "index.transcript.v1"


def analyze_and_read(
    client: DaemonClient,
    project_id: str,
    source_id: str,
    *,
    timeout_seconds: float,
) -> dict[str, dict[str, Any]]:
    """Run the analyze DAG for one source and read back what it published.

    The artifacts are found by the kind each task *declares* rather than by
    guessing from the task name: a plan states the observation before the work
    runs, which is the only way to tell "the ranking failed" from "there was no
    ranking task".
    """

    job = client.submit_analyze(project_id, source_id)
    finished = client.wait_for_job(job.job_id, timeout_seconds=timeout_seconds)
    if finished.state != daemon_pb2.JOB_STATE_SUCCEEDED:
        raise EvaluationError(
            f"analyze did not succeed for {source_id}: {finished.failure_detail or finished.state}"
        )
    wanted = {CANDIDATES_KIND, RANKING_KIND, INDEX_KIND}
    documents: dict[str, dict[str, Any]] = {}
    for task in finished.tasks:
        if task.output_kind in wanted and task.output_artifact_id:
            raw = client.read_artifact(project_id, task.output_artifact_id)
            documents[task.output_kind] = json.loads(raw)
    missing = wanted - set(documents) - {INDEX_KIND}
    if missing:
        raise EvaluationError(f"analyze published nothing for {sorted(missing)}")
    return documents


def verify_candidates(candidates: dict[str, Any]) -> None:
    """The three guarantees discovery makes, checked before anything is scored.

    A recall number computed over a candidate set that broke its own contract
    would be a number about something other than discovery, so this runs first
    and refuses rather than reporting.
    """

    entries = candidates.get("candidates")
    if not isinstance(entries, list):
        raise EvaluationError("the candidate set carries no candidates array")
    clusters = {
        entry.get("id") for entry in candidates.get("clusters", []) if isinstance(entry, dict)
    }
    for entry in entries:
        identifier = entry.get("id", "?")
        if not entry.get("evidence"):
            raise EvaluationError(f"{identifier} carries no evidence to walk back to")
        if not entry.get("cluster_id"):
            raise EvaluationError(f"{identifier} belongs to no cluster")
        if clusters and entry["cluster_id"] not in clusters:
            raise EvaluationError(f"{identifier} names a cluster that is not in the set")
        lattice = entry.get("boundary_lattice") or {}
        if not lattice.get("starts") or not lattice.get("ends"):
            raise EvaluationError(f"{identifier} carries no legal boundary lattice")


def verify_ranking(ranking: dict[str, Any], candidates: dict[str, Any]) -> None:
    """What a ranked set must be true of before its recall means anything."""

    known = {entry.get("id") for entry in candidates.get("candidates", [])}
    ranked = ranking.get("cohort")
    selected = ranking.get("selected")
    if not isinstance(ranked, list) or not isinstance(selected, list):
        raise EvaluationError("the ranking set is missing its cohort or its selection")
    cohort = {entry.get("candidate_id") for entry in ranked}
    unknown = cohort - known
    if unknown:
        raise EvaluationError(f"the ranking scored candidates discovery never proposed: {unknown}")
    outside = set(selected) - cohort
    if outside:
        raise EvaluationError(f"the ranking selected clips outside its own cohort: {outside}")
    if len(set(selected)) != len(selected):
        raise EvaluationError("the ranking selected the same clip twice")
    # A set smaller than requested is allowed and is the honest answer; a set
    # larger than requested is a bug that would inflate recall for free.
    requested = ranking.get("requested", {}).get("count")
    if isinstance(requested, int) and len(selected) > requested:
        raise EvaluationError(
            f"the ranking returned {len(selected)} clips for a request of {requested}"
        )
