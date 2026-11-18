"""Exercise the installed Studio executable and its project during a Windows upgrade."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path

from clipmill.ipc.v1 import daemon_pb2 as pb
from clipmill_eval.client import DaemonClient
from packaged_smoke import locate, make_recording, stop, wait_for_health


def check_mapped_studio(app_dir: Path, installer: Path) -> None:
    # CREATE_SUSPENDED maps the released Studio image before its first instruction.
    # This exercises Windows' actual running-image lock without needing a desktop.
    legacy = app_dir / "clipmill.exe"
    old_studio = subprocess.Popen([legacy], creationflags=0x00000004)
    try:
        try:
            with legacy.open("r+b"):
                raise RuntimeError("The running Studio executable was writable")
        except PermissionError:
            pass
        result = subprocess.run([installer, "/S", "/UPDATE"], timeout=300)
        if result.returncode != 2:
            raise RuntimeError(f"In-use installer returned {result.returncode} instead of 2")
        if old_studio.poll() is not None or (app_dir / "clipmill-studio.exe").exists():
            raise RuntimeError("The upgrade changed an installation still in use")
    finally:
        old_studio.terminate()
        old_studio.wait(timeout=30)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app-dir", required=True, type=Path)
    actions = parser.add_subparsers(dest="action", required=True)
    in_use = actions.add_parser("in-use")
    in_use.add_argument("--installer", required=True, type=Path)
    for action in ("create", "reopen"):
        project = actions.add_parser(action)
        project.add_argument("--work-dir", required=True, type=Path)
        project.add_argument("--version", required=True)
    args = parser.parse_args()
    if args.action == "in-use":
        check_mapped_studio(args.app_dir, args.installer)
        return
    work = args.work_dir
    receipt = work / "project.json"
    paths = locate(args.app_dir)
    data = Path(os.environ["LOCALAPPDATA"]) / "clipmill" / "ClipMill" / "data"
    env = dict(os.environ)
    for key in ("CLIPMILL_DATA_DIR", "CLIPMILL_SOCKET"):
        env.pop(key, None)
    command = [
        paths["daemon"],
        "--resources",
        paths["resources"],
        "--ffprobe",
        paths["ffprobe"],
        "--uv",
        paths["uv"],
    ]
    with (work / f"daemon-{args.action}.log").open("ab") as log:
        daemon = subprocess.Popen(command, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            client = DaemonClient(data / "run" / "clipmilld.sock")
            assert wait_for_health(client, daemon).daemon_version == args.version
            if args.action == "create":
                project = client.create_project("Installer upgrade")
                recording = make_recording(paths["ffmpeg"], work)
                registered = client.register_source(project.project_id, recording)
                source_id = registered.source.source_id
                job = client.submit_probe(project.project_id, source_id)
                assert client.wait_for_job(job.job_id, timeout_seconds=60).state == (
                    pb.JOB_STATE_SUCCEEDED
                )
                query = pb.GetSourceRequest(source_id=source_id)
                source = client._call(pb.Request(get_source=query)).get_source.source
                expected = {
                    "project": project.project_id,
                    "source": source_id,
                    "map": source.source_map_artifact_id,
                }
                expected["sha256"] = hashlib.sha256(
                    client.read_artifact(expected["project"], expected["map"])
                ).hexdigest()
                receipt.write_text(json.dumps(expected), encoding="utf-8")
            else:
                expected = json.loads(receipt.read_text(encoding="utf-8"))
                request = pb.GetProjectRequest(project_id=expected["project"])
                project = client._call(pb.Request(get_project=request)).get_project.project
                assert project.name == "Installer upgrade"
                query = pb.ListSourcesRequest(project_id=project.project_id)
                sources = client._call(pb.Request(list_sources=query)).list_sources.sources
                source = next(s for s in sources if s.source_id == expected["source"])
                assert source.source_map_artifact_id == expected["map"]
                content = client.read_artifact(project.project_id, expected["map"])
                assert hashlib.sha256(content).hexdigest() == expected["sha256"]
            assert stop(client, daemon) == 0
        finally:
            if daemon.poll() is None:
                daemon.kill()
                daemon.wait()
    print(f"ClipMill {args.version} verified project {expected['project']} and its source map")


if __name__ == "__main__":
    main()
