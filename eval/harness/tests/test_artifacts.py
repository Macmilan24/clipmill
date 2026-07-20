import hashlib
import json
from pathlib import Path

import pytest
from clipmill_eval.artifacts import ArtifactVerificationError, verify_artifact


def artifact_fixture(data_dir: Path) -> tuple[str, Path]:
    artifact_id = "sha256:" + "1" * 64
    digest = artifact_id.removeprefix("sha256:")
    object_dir = data_dir / "artifacts" / "objects" / "sha256" / digest[:2] / digest
    object_dir.mkdir(parents=True)
    payload = b'{"mapping":{}}'
    (object_dir / "source-map.json").write_bytes(payload)
    manifest = {
        "schema_version": "clipmill.artifact.manifest.v1",
        "artifact_id": artifact_id,
        "kind": "evidence.source_map.v1",
        "producer": {"stage": "probe-source", "implementation": "fixture"},
        "files": [
            {
                "path": "source-map.json",
                "bytes": len(payload),
                "sha256": "sha256:" + hashlib.sha256(payload).hexdigest(),
            }
        ],
    }
    (object_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    return artifact_id, object_dir


def test_artifact_payload_hash_and_exact_file_set_are_verified(tmp_path: Path) -> None:
    artifact_id, object_dir = artifact_fixture(tmp_path)
    verified = verify_artifact(tmp_path, artifact_id)
    assert verified.stage == "probe-source"
    assert verified.kind == "evidence.source_map.v1"

    (object_dir / "source-map.json").write_bytes(b'{"mapping":[]}')
    with pytest.raises(ArtifactVerificationError, match="SHA-256"):
        verify_artifact(tmp_path, artifact_id)


def test_undeclared_files_and_symlinks_fail_closed(tmp_path: Path) -> None:
    artifact_id, object_dir = artifact_fixture(tmp_path)
    (object_dir / "extra.bin").write_bytes(b"extra")
    with pytest.raises(ArtifactVerificationError, match="undeclared"):
        verify_artifact(tmp_path, artifact_id)

    (object_dir / "extra.bin").unlink()
    payload = object_dir / "source-map.json"
    real = object_dir / "real.json"
    real.write_bytes(payload.read_bytes())
    payload.unlink()
    payload.symlink_to(real)
    with pytest.raises(ArtifactVerificationError, match="symlink"):
        verify_artifact(tmp_path, artifact_id)
