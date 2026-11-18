"""Exercise task creation, transport changes, token isolation, revocation and restart."""

from __future__ import annotations

import argparse
import json
import os
import socket
import subprocess
import tempfile
import time
from contextlib import ExitStack, contextmanager
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from clipmill.ipc.v1 import daemon_pb2 as pb
from clipmill_eval.client import DaemonClient

ROOT = Path(__file__).resolve().parents[3]
CLIENT = Path(__file__).with_name("client.mjs")
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve() / "debug"


@contextmanager
def process(arguments, env=None):
    child = subprocess.Popen(arguments, stdout=subprocess.DEVNULL, env=env)
    try:
        yield child
    finally:
        child.terminate()
        try:
            child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()


def wait_for(probe, child):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if child.poll() is not None:
            raise RuntimeError("The local test process exited during startup")
        try:
            probe()
            return
        except OSError:
            time.sleep(0.1)
    raise TimeoutError("The local test process did not start")


def client_token(local, name):
    created = local._call(
        pb.Request(
            create_agent_client=pb.CreateAgentClientRequest(
                name=name,
                scopes=["read", "edit"],
            )
        )
    ).agent_client_created
    return created.client.client_id, created.token


def sdk(**values):
    result = subprocess.run(
        ["node", CLIENT],
        input=json.dumps(values),
        capture_output=True,
        text=True,
        timeout=60,
    )
    if result.returncode:
        raise RuntimeError(result.stderr.replace(values["token"], "[redacted]"))
    return json.loads(result.stdout) if result.stdout else None


def test_http_retries(url, token):
    def create(name):
        request = Request(
            url + "/v2/create_project",
            data=json.dumps({"name": name}).encode(),
            headers={
                "Authorization": "Bearer " + token,
                "Content-Type": "application/json",
                "Idempotency-Key": '"example-key"',
            },
        )
        with urlopen(request, timeout=10) as response:
            return json.load(response)["result"]["project_id"]

    first = create("HTTP retry")
    assert first == create("HTTP retry")
    try:
        create("Conflicting write")
    except HTTPError as error:
        assert error.code == 409 and json.load(error)["error"]["code"] == "conflict"
    else:
        raise AssertionError("A reused key accepted different arguments")


def finish_analysis(local, args, project, source, work, env):
    with ExitStack() as workers:
        for kind in ["shots", "faces"]:
            executable = ROOT / "workers" / kind / ".venv/bin" / ("clipmill-worker-" + kind)
            workers.enter_context(
                process(
                    [
                        executable,
                        "--data-dir",
                        work / "state",
                        "--worker-socket",
                        work / "w.sock",
                        "--shm-socket",
                        work / "state/run/clipmill-shm.sock",
                        "--identity",
                        work / (kind + ".json"),
                    ],
                    env,
                )
            )
        finished = sdk(
            phase="start", key="completed-analysis", project=project, source=source, **args
        )
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            job = local.get_job(finished["taskId"])
            if job.state in (pb.JOB_STATE_FAILED, pb.JOB_STATE_CANCELLED, pb.JOB_STATE_SUCCEEDED):
                assert job.state == pb.JOB_STATE_SUCCEEDED, job.failure_detail
                break
            time.sleep(0.3)
        else:
            raise TimeoutError(
                "Unfinished analysis stages: "
                + ", ".join(
                    f"{task.kind}={pb.TaskState.Name(task.state)}"
                    for task in job.tasks
                    if task.state != pb.TASK_STATE_SUCCEEDED
                )
            )
        sdk(phase="terminal", **{**args, **finished})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--with-workers",
        action="store_true",
        help="complete a real analysis with the installed shots and faces workers",
    )
    options = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="clipmill-mcp-") as folder:
        work = Path(folder)
        # Short endpoint paths work even on hosts with long temporary directory names.
        ipc = work / "d.sock"
        probe = ROOT / ".cache/bin/ffprobe"
        daemon = [
            TARGET / "clipmilld",
            "--data-dir",
            work / "state",
            "--socket",
            ipc,
            "--worker-socket",
            work / "w.sock",
            "--ffprobe",
            probe,
        ]
        env = {
            **os.environ,
            "CLIPMILL_MODELS_DIR": str(ROOT / "models/registry"),
            "CLIPMILL_WEIGHTS_DIR": str(ROOT / ".cache/models"),
            "CLIPMILL_WORKER_LOG": "WARNING",
        }
        if options.with_workers:
            for kind in ["shots", "faces"]:
                subprocess.run(
                    [
                        TARGET / "clipmill-worker-keygen",
                        "--data-dir",
                        work / "state",
                        "--identity",
                        work / (kind + ".json"),
                    ],
                    stdout=subprocess.DEVNULL,
                    check=True,
                )
        local = DaemonClient(ipc)
        media = work / "fixture.mp4"
        subprocess.run(
            [
                ROOT / ".cache/bin/ffmpeg",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=160x90:duration=3",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                media,
            ],
            check=True,
        )
        with process(daemon, env) as child:
            wait_for(local.health, child)
            project = local.create_project("MCP conformance").project_id
            source = local.register_source(project, media).source.source_id
            owner, token = client_token(local, "owner")
            _, other = client_token(local, "other")
            args = dict(socket=str(ipc), cli=str(TARGET / "clipmill"), token=token)
            started = sdk(phase="start", project=project, source=source, **args)
        with process(daemon, env) as child:
            wait_for(local.health, child)
            with socket.socket() as ephemeral:
                ephemeral.bind(("127.0.0.1", 0))
                port = ephemeral.getsockname()[1]
            with process(
                [TARGET / "clipmill", "--socket", ipc, "serve", "--port", str(port)]
            ) as gateway:

                def gateway_ready():
                    with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                        pass

                wait_for(gateway_ready, gateway)
                args.update(started, url=f"http://127.0.0.1:{port}/mcp")
                sdk(phase="other", **{**args, "token": other})
                sdk(phase="cancel", **args)
                test_http_retries(f"http://127.0.0.1:{port}", token)
                if options.with_workers:
                    finish_analysis(local, args, project, source, work, env)
                local._call(
                    pb.Request(revoke_agent_client=pb.RevokeAgentClientRequest(client_id=owner))
                )
                sdk(phase="revoked", **args)
    print("MCP conformance passed for both protocol generations and transports")


if __name__ == "__main__":
    main()
