#!/usr/bin/env python3
"""Run local editorial analysis through a real daemon, workers and MP4 export.

This is a functional smoke test, not an editorial quality benchmark. No cloud
route is enabled. Uses a supplied recording, or the existing synthesized talk.
Run under workers/editorial/.venv/bin/python after building clipmilld.
"""

import argparse
import json
import os
import signal
import subprocess
import sys
import tempfile
import time
from contextlib import suppress
from pathlib import Path

# Resolve the in-repository evaluation client before importing it.
# ruff: noqa: E402
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "eval/harness/src"))
from clipmill.ipc.v1 import daemon_pb2 as ipc
from clipmill_eval.client import DaemonClient


def wait_job(client, job_id, timeout=1800):
    deadline = time.monotonic() + timeout
    previous = None
    while time.monotonic() < deadline:
        job = client.get_job(job_id)
        state = [(t.kind, t.state) for t in job.tasks]
        if state != previous:
            print(state, flush=True)
            previous = state
        if job.state == ipc.JOB_STATE_SUCCEEDED:
            return job
        if job.state in (ipc.JOB_STATE_FAILED, ipc.JOB_STATE_CANCELLED):
            raise RuntimeError(str(job))
        time.sleep(1)
    raise TimeoutError(f"job {job_id} did not finish")


def output(job, kind):
    task = next(t for t in job.tasks if t.kind == kind)
    return task.output_artifact_id


def model_edge_checks(work, source):
    """Exercise real empty proposals and image input, not comparative quality."""
    from clipmill_worker_editorial.inference import look, propose
    from clipmill_worker_editorial.runtime import LocalModel
    from clipmill_worker_sdk import CancellationToken

    cancellation = CancellationToken()
    runtime = LocalModel(ROOT / ".cache/models/qwen3-5-editorial-mlx", cancellation)
    trace = []
    identity = {"route": "local", "model": {"name": "qwen3-5-editorial-mlx"}}
    lines = [
        "Testing, one two.",
        "We are still setting up.",
        "Please wait.",
        "The guest has not arrived.",
        "We have not started the interview.",
        "Testing again.",
    ]
    windows = {
        "outline": [],
        "sentences": [
            {
                "index": i,
                "text": line,
                "start_ticks": i * 5 * 90000,
                "end_ticks": (i + 1) * 5 * 90000,
            }
            for i, line in enumerate(lines)
        ],
        "windows": [
            {
                "index": 0,
                "first_sentence_index": 0,
                "sentence_count": len(lines),
                "context_before_sentences": 0,
                "context_after_sentences": 0,
            }
        ],
    }
    try:
        answers = propose(
            runtime,
            windows,
            ipc.ClipDurationV1(min_ticks=20 * 90000, max_ticks=90 * 90000),
            2048,
            identity,
            trace,
            cancellation,
        )
        assert answers[0]["status"] == "none", f"Setup chatter was not declined: {answers}"
        frame = work / "visual-smoke.png"
        subprocess.run(
            [
                str(ROOT / ".cache/bin/ffmpeg"),
                "-v",
                "error",
                "-y",
                "-i",
                str(source),
                "-frames:v",
                "1",
                str(frame),
            ],
            check=True,
        )
        candidate_id = "cand_0000000000000001"
        checks = look(
            lambda: runtime,
            {
                "candidates": [
                    {"id": candidate_id, "intervals": [{"start_ticks": 0, "end_ticks": 90000}]}
                ]
            },
            {
                "candidates": [
                    {
                        "candidate_id": candidate_id,
                        "outcome": "answered",
                        "status": "needs_review",
                        "visual_dependency": True,
                        "summary": "The speaker refers to a diagram.",
                        "reasons": [],
                    }
                ]
            },
            {"frames": [{"t_ticks": 0, "file": frame.name}]},
            lambda _: frame,
            2048,
            identity,
            trace,
            cancellation,
        )
        assert checks and checks[0]["outcome"] == "answered", str(checks)
        return {"setup_chatter": "none", "visual_check": checks[0]["answer"]}
    finally:
        runtime.close()
        path = work / "model-edge-trace.json"
        fd = os.open(path, os.O_CREAT | os.O_TRUNC | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
        with os.fdopen(fd, "w") as handle:
            json.dump(trace, handle, indent=2)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--recording", type=Path, default=ROOT / "target/milestone-1/talk.mp4")
    parser.add_argument(
        "--daemon",
        type=Path,
        default=Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/clipmilld",
    )
    parser.add_argument("--work", type=Path)
    args = parser.parse_args()
    work = args.work or Path(tempfile.mkdtemp(prefix="cm-editorial-", dir="/private/tmp"))
    work.mkdir(parents=True, exist_ok=True)
    print(f"editorial-live: evidence and output in {work}", flush=True)
    source = work / "talk.mp4"
    subprocess.run(
        [
            str(ROOT / ".cache/bin/ffmpeg"),
            "-v",
            "error",
            "-y",
            "-i",
            str(args.recording),
            "-t",
            "180",
            "-c",
            "copy",
            str(source),
        ],
        check=True,
    )
    env = {
        **os.environ,
        "CLIPMILL_MODELS_DIR": str(ROOT / "models/registry"),
        "CLIPMILL_WEIGHTS_DIR": str(ROOT / ".cache/models"),
        "CLIPMILL_SOCKET": str(work / "d.sock"),
    }
    subprocess.run(
        [str(ROOT / "tools/run-workers.sh"), "--enrol-only", "--data-dir", str(work)],
        cwd=ROOT,
        env=env,
        check=True,
    )
    children = []
    try:
        for name, command in [
            (
                "daemon",
                [
                    str(args.daemon),
                    "--data-dir",
                    str(work),
                    "--socket",
                    str(work / "d.sock"),
                    "--ffprobe",
                    str(ROOT / ".cache/bin/ffprobe"),
                ],
            ),
            ("workers", [str(ROOT / "tools/run-workers.sh"), "--data-dir", str(work)]),
        ]:
            if name == "workers":
                deadline = time.monotonic() + 60
                while not (work / "d.sock").exists():
                    if time.monotonic() > deadline:
                        raise TimeoutError("daemon socket")
                    time.sleep(0.1)
            with (work / f"{name}.log").open("w") as log:
                children.append(
                    subprocess.Popen(
                        command, cwd=ROOT, env=env, stdout=log, stderr=log, start_new_session=True
                    )
                )
        client = DaemonClient(work / "d.sock", timeout_seconds=180)
        deadline = time.monotonic() + 600
        while True:
            readiness = client._call(
                ipc.Request(get_readiness=ipc.GetReadinessRequest())
            ).get_readiness
            stages = [s for s in readiness.stages if not s.stage.endswith("-cloud")]
            if stages and all(s.ready for s in stages):
                break
            if "editorial runtime is not ready" in (work / "workers.log").read_text():
                raise RuntimeError(f"Qwen runtime readiness failed; see {work / 'workers.log'}")
            if time.monotonic() > deadline:
                raise TimeoutError(str(readiness))
            time.sleep(2)
        project = client.create_project("Editorial live smoke")
        registered = client.register_source(project.project_id, source)
        source_id = registered.source.source_id
        payload = ipc.AnalyzeSourcePayloadV1(
            key_version="clipmill.analyze-source.v1",
            source_id=source_id,
            language="en",
            local_editorial=True,
            duration=ipc.ClipDurationV1(min_ticks=20 * 90000, max_ticks=90 * 90000),
            count=5,
        )
        submitted = client._call(
            ipc.Request(
                submit_job=ipc.SubmitJobRequest(
                    project_id=project.project_id,
                    kind="analyze-source",
                    payload=payload.SerializeToString(deterministic=True),
                )
            )
        ).submit_job.job
        job = wait_job(client, submitted.job_id)
        assert not any(t.kind.endswith("-cloud") for t in job.tasks)
        ranking = json.loads(
            client.read_artifact(project.project_id, output(job, "rank-candidates"))
        )
        (work / "ranking.json").write_text(json.dumps(ranking, indent=2))
        assert ranking.get("selected"), (
            "This complete talk must produce at least one reviewable moment"
        )
        assert all(
            row["review"]["status"] in ("accepted", "needs_review") for row in ranking["cohort"]
        )
        candidate_id = ranking["selected"][0]
        directed = client._call(
            ipc.Request(
                direct_clip=ipc.DirectClipRequest(
                    project_id=project.project_id,
                    source_id=source_id,
                    candidate_id=candidate_id,
                    job_id=job.job_id,
                    cut=ipc.CLIP_CUT_V1_CHOSEN,
                    approve=True,
                )
            )
        ).direct_clip
        assert directed.doc.job_id == job.job_id
        assert directed.end_ticks - directed.start_ticks >= 20 * 90000
        request = ipc.ExportRequestV1(
            doc_id=directed.doc.doc_id,
            destination_dir=str(work / "delivered"),
            source_attestation="own_content",
            gates_passed=["duration_60s"],
            ai_assistance=["asr_captions"],
            index=1,
            date="2026-09-18",
            title="editorial smoke",
            expected_revision=directed.doc.revision,
        )
        (work / "delivered").mkdir(exist_ok=True)
        planned = client._call(
            ipc.Request(plan_export=ipc.PlanExportRequest(request=request))
        ).plan_export
        assert planned.validation.passes, str(planned.validation)
        exported = client._call(
            ipc.Request(export_clip=ipc.ExportClipRequest(request=request))
        ).export_clip
        assert exported.revision == directed.doc.revision
        wait_job(client, exported.job_id)
        movie = next((work / "delivered").glob("*.mp4"))
        subprocess.run(
            [str(ROOT / ".cache/bin/ffmpeg"), "-v", "error", "-i", str(movie), "-f", "null", "-"],
            check=True,
        )
        probe = json.loads(
            subprocess.check_output(
                [
                    str(ROOT / ".cache/bin/ffprobe"),
                    "-v",
                    "error",
                    "-show_streams",
                    "-show_format",
                    "-of",
                    "json",
                    str(movie),
                ]
            )
        )
        duration = float(probe["format"]["duration"])
        assert abs(duration - (directed.end_ticks - directed.start_ticks) / 90000) < 0.2
        for suffix in ("*.srt", "*.vtt"):
            assert next((work / "delivered").glob(suffix)).stat().st_size > 20
        edges = model_edge_checks(work, source)
        result = {
            "passed": True,
            "route": "local",
            "model": "Qwen3.5-9B",
            "project_id": project.project_id,
            "run_id": job.job_id,
            "candidate_id": candidate_id,
            "doc_id": directed.doc.doc_id,
            "revision": exported.revision,
            "snapshot": exported.ir_artifact_id,
            "movie": str(movie),
            "duration_seconds": duration,
            "model_edge_checks": edges,
            "quality_benchmark": False,
            "native_window_clicked": False,
        }
        (work / "result.json").write_text(json.dumps(result, indent=2))
        print(json.dumps(result, indent=2), flush=True)
    finally:
        for child in reversed(children):
            with suppress(ProcessLookupError):
                os.killpg(child.pid, signal.SIGTERM)
        for child in children:
            try:
                child.wait(timeout=15)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()


if __name__ == "__main__":
    main()
