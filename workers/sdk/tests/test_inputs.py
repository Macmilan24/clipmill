"""A stage finds what it reads by what each artifact is.

The lease delivers one list, whichever route filled it: addresses the plan
declared when the stage runs on its own, dependency outputs when it runs inside a
larger plan. Positions in that list mean nothing — inside an analysis a
recognition lease carries the audio as well as the voice activity — so the only
safe way to read it is by kind.
"""

import hashlib
import json
from pathlib import Path

import pytest
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk.client import TaskContext
from clipmill_worker_sdk.inputs import LeaseInputs, MissingInputError, require_input


def artifact_fixture(artifact_root: Path, kind: str, seed: str) -> str:
    artifact_id = "sha256:" + seed * 64
    digest = artifact_id.removeprefix("sha256:")
    object_dir = artifact_root / "objects" / "sha256" / digest[:2] / digest
    object_dir.mkdir(parents=True, exist_ok=True)
    payload = json.dumps({"kind": kind}).encode("utf-8")
    (object_dir / "result.json").write_bytes(payload)
    manifest = {
        "schema_version": "clipmill.artifact.manifest.v1",
        "artifact_id": artifact_id,
        "kind": kind,
        "producer": {"stage": "fixture", "implementation": "fixture"},
        "files": [
            {
                "path": "result.json",
                "bytes": len(payload),
                "sha256": "sha256:" + hashlib.sha256(payload).hexdigest(),
            }
        ],
    }
    (object_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    return artifact_id


def context_for(artifact_root: Path, artifact_ids: list[str]) -> TaskContext:
    lease = worker_pb2.TaskLease(
        task_id="tsk_01J00000000000000000000000",
        lease_id="lse_01J00000000000000000000000",
        kind="speech-asr",
        input_artifact_ids=artifact_ids,
        artifact_root=str(artifact_root),
    )
    return TaskContext(lease=lease, staging=None, shared=None, cancellation=None)


def test_an_input_is_found_by_kind_and_not_by_position(tmp_path: Path) -> None:
    audio = artifact_fixture(tmp_path, "media.audio_16k.v1", "1")
    activity = artifact_fixture(tmp_path, "speech.vad.v1", "2")

    # The same two artifacts in either order resolve identically, which is what
    # lets one plan deliver them declared-first and another dependency-first.
    for order in ([audio, activity], [activity, audio]):
        inputs = LeaseInputs(context_for(tmp_path, order))
        assert inputs.require("media.audio_16k.v1").artifact_id == audio
        assert inputs.require("speech.vad.v1").artifact_id == activity
        assert inputs.kinds() == ("media.audio_16k.v1", "speech.vad.v1")


def test_a_missing_input_names_what_was_delivered_instead(tmp_path: Path) -> None:
    audio = artifact_fixture(tmp_path, "media.audio_16k.v1", "1")
    inputs = LeaseInputs(context_for(tmp_path, [audio]))

    with pytest.raises(MissingInputError) as raised:
        inputs.require("speech.vad.v1")
    # The message has to say what turned up, because the failure is almost always
    # a plan that wired the wrong artifact rather than a store that lost one.
    assert "media.audio_16k.v1" in str(raised.value)
    assert "speech.vad.v1" in str(raised.value)


def test_a_lease_with_nothing_says_so_rather_than_naming_a_kind(tmp_path: Path) -> None:
    inputs = LeaseInputs(context_for(tmp_path, []))
    with pytest.raises(MissingInputError) as raised:
        inputs.require("media.audio_16k.v1")
    assert "nothing" in str(raised.value)


def test_an_absent_optional_input_is_none_rather_than_a_failure(tmp_path: Path) -> None:
    audio = artifact_fixture(tmp_path, "media.audio_16k.v1", "1")
    inputs = LeaseInputs(context_for(tmp_path, [audio]))

    # A source with no video has no shot cuts. That is a fact a stage reports,
    # not one it fails over.
    assert inputs.optional("evidence.shots.v1") is None
    assert inputs.optional("media.audio_16k.v1") is not None


def test_two_inputs_of_one_kind_are_refused_rather_than_chosen_between(
    tmp_path: Path,
) -> None:
    first = artifact_fixture(tmp_path, "speech.vad.v1", "1")
    second = artifact_fixture(tmp_path, "speech.vad.v1", "2")

    # Taking the first would be right most of the time, which is worse than
    # failing: the stage would publish under an address naming inputs it did not
    # read, and nothing downstream could tell.
    with pytest.raises(MissingInputError):
        LeaseInputs(context_for(tmp_path, [first, second]))


def test_require_input_resolves_a_single_input_stage(tmp_path: Path) -> None:
    proxy = artifact_fixture(tmp_path, "media.proxy.v1", "3")
    found = require_input(context_for(tmp_path, [proxy]), "media.proxy.v1")
    assert found.kind == "media.proxy.v1"
    assert found.artifact_id == proxy
    assert found.artifact.kind == "media.proxy.v1"
