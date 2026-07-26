"""A worker reads the daemon's store without trusting it.

The store belongs to the daemon. A worker that read it blindly would turn a
corrupt object into a corrupt result and then publish that under a content
address asserting it is fine — the one failure a content-addressed store
cannot detect afterwards.
"""

import hashlib
import json
from pathlib import Path

import pytest
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk.artifacts import (
    ArtifactVerificationError,
    artifact_file,
    open_artifact,
)
from clipmill_worker_sdk.client import TaskContext


def artifact_fixture(artifact_root: Path, digest_seed: str = "1") -> tuple[str, Path]:
    artifact_id = "sha256:" + digest_seed * 64
    digest = artifact_id.removeprefix("sha256:")
    object_dir = artifact_root / "objects" / "sha256" / digest[:2] / digest
    object_dir.mkdir(parents=True)
    payload = b'{"segments":[]}'
    (object_dir / "result.json").write_bytes(payload)
    manifest = {
        "schema_version": "clipmill.artifact.manifest.v1",
        "artifact_id": artifact_id,
        "kind": "evidence.demo.seed.v1",
        "producer": {"stage": "demo-seed", "implementation": "fixture"},
        "files": [
            {
                "path": "result.json",
                "bytes": len(payload),
                "sha256": "sha256:" + hashlib.sha256(payload).hexdigest(),
            }
        ],
    }
    (object_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    return artifact_id, object_dir


def context_for(artifact_root: Path, artifact_id: str) -> TaskContext:
    lease = worker_pb2.TaskLease(
        task_id="tsk_01J00000000000000000000000",
        lease_id="lse_01J00000000000000000000000",
        kind="demo-left",
        input_artifact_ids=[artifact_id],
        artifact_root=str(artifact_root),
    )
    return TaskContext(lease=lease, staging=None, shared=None, cancellation=None)


def test_a_worker_opens_a_verified_input_and_reads_its_payload(tmp_path: Path) -> None:
    artifact_id, _ = artifact_fixture(tmp_path)
    context = context_for(tmp_path, artifact_id)

    artifact = context.open_artifact(artifact_id)
    assert artifact.kind == "evidence.demo.seed.v1"
    assert artifact.stage == "demo-seed"
    payload = context.artifact_file(artifact, "result.json").read_bytes()
    assert json.loads(payload) == {"segments": []}


def test_a_corrupted_payload_is_refused_rather_than_read(tmp_path: Path) -> None:
    artifact_id, object_dir = artifact_fixture(tmp_path)
    context = context_for(tmp_path, artifact_id)
    # One byte, changed after publication.
    (object_dir / "result.json").write_bytes(b'{"segments":[0]}')
    with pytest.raises(ArtifactVerificationError, match=r"SHA-256|size"):
        context.open_artifact(artifact_id)


def test_an_undeclared_file_smuggled_into_an_artifact_is_refused(tmp_path: Path) -> None:
    artifact_id, object_dir = artifact_fixture(tmp_path)
    context = context_for(tmp_path, artifact_id)
    (object_dir / "extra.json").write_bytes(b"{}")
    with pytest.raises(ArtifactVerificationError, match="undeclared"):
        context.open_artifact(artifact_id)


def test_a_worker_may_only_open_artifacts_its_lease_named(tmp_path: Path) -> None:
    artifact_id, _ = artifact_fixture(tmp_path)
    other_id, _ = artifact_fixture(tmp_path, digest_seed="2")
    context = context_for(tmp_path, artifact_id)
    # The other artifact verifies perfectly well; it simply is not this task's.
    assert open_artifact(tmp_path, other_id).kind == "evidence.demo.seed.v1"
    with pytest.raises(ArtifactVerificationError, match="does not name"):
        context.open_artifact(other_id)


def test_only_declared_payload_paths_resolve(tmp_path: Path) -> None:
    artifact_id, object_dir = artifact_fixture(tmp_path)
    artifact = open_artifact(tmp_path, artifact_id)
    # A file whose digest nobody checked must not be reachable by name.
    (object_dir / "sneaky.json").write_bytes(b"{}")
    with pytest.raises(ArtifactVerificationError, match="declared payload"):
        artifact_file(artifact, "sneaky.json")
    with pytest.raises(ArtifactVerificationError, match="declared payload"):
        artifact_file(artifact, "../../../etc/passwd")


def test_a_lease_without_a_store_root_cannot_open_anything(tmp_path: Path) -> None:
    artifact_id, _ = artifact_fixture(tmp_path)
    context = context_for(tmp_path, artifact_id)
    context.lease.artifact_root = ""
    with pytest.raises(ArtifactVerificationError, match="store root"):
        context.open_artifact(artifact_id)
