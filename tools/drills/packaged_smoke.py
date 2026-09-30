#!/usr/bin/env python3
"""Check that an installed ClipMill runs, with nothing set up by hand.

Given the folder an installer put the app in, this finds the daemon, the
FFmpeg, FFprobe and uv beside it, and the resources the app carries; starts the
daemon on an empty data folder exactly as the app's shell does (``--resources``,
``--ffprobe``, ``--uv``); and then, over the daemon's own control plane:

1. reads its health, and checks its engine is managed and has parts to run;
2. installs every part, as Set up does: Python and the components' packages
   come from the network;
3. waits until every worker part's process runs;
4. registers a short recording made with the app's own FFmpeg, which probes
   it, and runs a probe job through the scheduler;
5. asks the daemon to stop, and checks that it does.

It prints what it found as JSON and exits non-zero at the first failure, after
printing the daemon's, the installer's and any failing worker's logs. It runs on
any platform; the windows-smoke workflow runs it on a release build's Windows
installer.

    uv run --project eval/harness python tools/drills/packaged_smoke.py \\
        --app-dir DIR --work-dir DIR
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

from clipmill.ipc.v1 import daemon_pb2 as pb
from clipmill_eval.client import DaemonClient

EXE = ".exe" if sys.platform == "win32" else ""
START_TIMEOUT = 90
INSTALL_TIMEOUT = 45 * 60
WORKERS_TIMEOUT = 5 * 60
PROBE_TIMEOUT = 180
STOP_TIMEOUT = 90


class SmokeFailure(RuntimeError):
    pass


def say(message: str) -> None:
    print(f"smoke: {message}", flush=True)


def find_one(root: Path, name: str) -> Path:
    found = sorted(path for path in root.rglob(name) if path.is_file())
    if not found:
        raise SmokeFailure(f"{name} is not in {root}")
    return found[0]


def locate(app_dir: Path) -> dict[str, Path]:
    """The installed programs and resources, wherever the platform put them."""
    manifest = find_one(app_dir, "engine.json")
    return {
        "daemon": find_one(app_dir, f"clipmilld{EXE}"),
        "ffmpeg": find_one(app_dir, f"ffmpeg{EXE}"),
        "ffprobe": find_one(app_dir, f"ffprobe{EXE}"),
        "uv": find_one(app_dir, f"uv{EXE}"),
        # resources/engine/engine.json
        "resources": manifest.parent.parent,
    }


def tail(path: Path, lines: int = 60) -> str:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as error:
        return f"({path}: {error})"
    return "\n".join(text.splitlines()[-lines:])


def wait_for_health(client: DaemonClient, daemon: subprocess.Popen[bytes]) -> pb.HealthResponse:
    deadline = time.monotonic() + START_TIMEOUT
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        if daemon.poll() is not None:
            raise SmokeFailure(f"the daemon exited with {daemon.returncode} before answering")
        try:
            return client.health()
        except (OSError, ValueError, RuntimeError) as error:
            last_error = error
            time.sleep(1)
    raise SmokeFailure(f"the daemon did not answer in {START_TIMEOUT}s: {last_error}")


def install(client: DaemonClient, logs: Path) -> dict[str, dict[str, object]]:
    engine = client.engine()
    if not engine.managed:
        raise SmokeFailure("the daemon did not recognise a packaged app: its engine is not managed")
    if engine.unavailable:
        raise SmokeFailure(f"the engine cannot be installed here: {engine.unavailable}")
    if not engine.parts:
        raise SmokeFailure("the engine has no parts for this computer")
    say(f"engine: Python {engine.python_version}, parts {[part.name for part in engine.parts]}")
    started = time.monotonic()
    client.install_engine()
    seen: dict[str, str] = {}
    deadline = started + INSTALL_TIMEOUT
    while time.monotonic() < deadline:
        engine = client.engine()
        for part in engine.parts:
            summary = f"{part.state} {part.detail}".strip()
            if seen.get(part.name) != summary:
                seen[part.name] = summary
                say(f"  {part.name}: {summary}")
        failed = [part for part in engine.parts if part.state == "failed"]
        if failed:
            raise SmokeFailure(
                "install failed: "
                + "; ".join(f"{part.name}: {part.detail}" for part in failed)
                + "\n--- engine-install.log ---\n"
                + tail(logs / "engine-install.log")
            )
        if all(part.state == "installed" for part in engine.parts):
            say(f"installed every part in {time.monotonic() - started:.0f}s")
            return {
                part.name: {"installed_bytes": part.installed_bytes, "tool": part.tool}
                for part in engine.parts
            }
        time.sleep(5)
    raise SmokeFailure(f"the install did not finish in {INSTALL_TIMEOUT}s: {seen}")


def wait_for_workers(client: DaemonClient) -> dict[str, str]:
    deadline = time.monotonic() + WORKERS_TIMEOUT
    while time.monotonic() < deadline:
        workers = [part for part in client.engine().parts if not part.tool]
        if all(part.process == "running" for part in workers):
            return {part.name: part.process for part in workers}
        time.sleep(3)
    workers = [part for part in client.engine().parts if not part.tool]
    stuck = [part for part in workers if part.process != "running"]
    raise SmokeFailure(
        "workers did not run: "
        + "; ".join(f"{part.name} {part.process} after {part.restarts} restarts" for part in stuck)
        + "".join(
            f"\n--- {part.name} ({part.log_path}) ---\n{tail(Path(part.log_path))}"
            for part in stuck
            if part.log_path
        )
    )


def make_recording(ffmpeg: Path, folder: Path) -> Path:
    recording = folder / "smoke recording.mp4"
    subprocess.run(
        [
            str(ffmpeg),
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=3:size=320x240:rate=30",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=3",
            "-shortest",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            str(recording),
        ],
        check=True,
    )
    return recording


def probe(client: DaemonClient, recording: Path) -> dict[str, object]:
    project = client.create_project("Packaged smoke")
    registered = client.register_source(project.project_id, recording)
    source_id = registered.source.source_id
    job = client.submit_probe(project.project_id, source_id)
    deadline = time.monotonic() + PROBE_TIMEOUT
    while time.monotonic() < deadline:
        job = client.get_job(job.job_id)
        if job.state in (pb.JOB_STATE_SUCCEEDED, pb.JOB_STATE_FAILED, pb.JOB_STATE_CANCELLED):
            break
        time.sleep(1)
    if job.state != pb.JOB_STATE_SUCCEEDED:
        raise SmokeFailure(f"the probe job ended {pb.JobState.Name(job.state)}")
    return {"source_id": source_id, "job": pb.JobState.Name(job.state)}


def stop(client: DaemonClient, daemon: subprocess.Popen[bytes]) -> int:
    client.shutdown()
    try:
        return daemon.wait(timeout=STOP_TIMEOUT)
    except subprocess.TimeoutExpired as error:
        raise SmokeFailure(f"the daemon did not stop within {STOP_TIMEOUT}s of Shutdown") from error


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--app-dir", type=Path, required=True, help="where the app is installed")
    parser.add_argument("--work-dir", type=Path, required=True, help="an empty scratch folder")
    parser.add_argument(
        "--data-dir",
        type=Path,
        default=None,
        help="the daemon's data folder (default: a new one in the work folder)",
    )
    options = parser.parse_args()
    work = options.work_dir.resolve()
    work.mkdir(parents=True, exist_ok=True)
    data = options.data_dir.resolve() if options.data_dir else work / "data"
    found = locate(options.app_dir.resolve())
    say("found " + ", ".join(f"{name}={path}" for name, path in found.items()))

    daemon_log = work / "daemon.log"
    environment = dict(os.environ, CLIPMILL_DATA_DIR=str(data))
    with daemon_log.open("wb") as log:
        daemon = subprocess.Popen(
            [
                str(found["daemon"]),
                "--resources",
                str(found["resources"]),
                "--ffprobe",
                str(found["ffprobe"]),
                "--uv",
                str(found["uv"]),
            ],
            env=environment,
            stdout=log,
            stderr=subprocess.STDOUT,
        )
    client = DaemonClient(data / "run" / "clipmilld.sock", timeout_seconds=30)
    report: dict[str, object] = {"platform": sys.platform}
    try:
        health = wait_for_health(client, daemon)
        report["daemon_version"] = health.daemon_version
        say(f"daemon {health.daemon_version} answered")
        report["parts"] = install(client, data / "logs")
        report["workers"] = wait_for_workers(client)
        say(f"workers running: {sorted(report['workers'])}")
        report["probe"] = probe(client, make_recording(found["ffmpeg"], work))
        say("the recording was registered and probed")
        report["exit_code"] = stop(client, daemon)
        say(f"the daemon stopped with {report['exit_code']}")
    except (SmokeFailure, OSError, RuntimeError, ValueError, subprocess.SubprocessError) as error:
        print(f"smoke: FAILED: {error}", file=sys.stderr, flush=True)
        print(f"--- daemon.log ---\n{tail(daemon_log, 120)}", file=sys.stderr, flush=True)
        if daemon.poll() is None:
            daemon.kill()
        return 1
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
