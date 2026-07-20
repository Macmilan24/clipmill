import hashlib
from pathlib import Path
from types import SimpleNamespace

import clipmill_worker_echo
import clipmill_worker_sdk
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk import CancellationToken, StagingArea


def test_sdk_dependency_resolves() -> None:
    assert clipmill_worker_echo.__version__ == clipmill_worker_sdk.__version__


def test_echo_writes_the_daemon_recipe_output(tmp_path: Path) -> None:
    root = tmp_path / "stg_01J00000000000000000000000"
    root.mkdir(mode=0o700)
    payload = b"echo-payload"
    context = SimpleNamespace(
        cancellation=CancellationToken(),
        lease=worker_pb2.TaskLease(
            kind="demo-seed",
            payload=payload,
            input_artifact_ids=[],
        ),
        shared=SimpleNamespace(buffer=SimpleNamespace(to_pybytes=lambda: payload)),
        staging=StagingArea(root.name, str(root)),
        report_progress=lambda *_args: None,
    )
    assert clipmill_worker_echo.execute_echo(context) == ("result.json",)
    expected = (
        b'{"inputs":[],"kind":"demo-seed","payload_sha256":"sha256:'
        + hashlib.sha256(payload).hexdigest().encode()
        + b'"}'
    )
    assert (root / "result.json").read_bytes() == expected
