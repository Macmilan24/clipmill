#!/usr/bin/env python3
"""Run local editorial analysis through a real daemon, workers and MP4 export.

Review the original and ranked cuts side by side with transcripts and local playback.
Supplied recordings receive working previews with rights unassessed. The generated
milestone talk also exercises the export package and sidecars.
Run under workers/editorial/.venv/bin/python after building clipmilld.
"""

import argparse
import hashlib
import html
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from contextlib import suppress
from datetime import date
from pathlib import Path

# Resolve the in-repository evaluation client before importing it.
# ruff: noqa: E402
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "eval/harness/src"))
from clipmill.ipc.v1 import daemon_pb2 as ipc
from clipmill_eval.artifacts import artifact_file, verify_artifact
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


def engine(client, operation, **args):
    call = {"op": operation, "args": args}
    if operation in {"render_preview", "reprompt"}:
        call["request_id"] = (
            f"drill-{hashlib.sha256(json.dumps(call, sort_keys=True).encode()).hexdigest()[:32]}"
        )
    response = client._call(
        ipc.Request(
            engine_call=ipc.EngineCallRequest(
                call_json=json.dumps(call),
            )
        )
    ).engine_call
    envelope = json.loads(response.result_json)
    if not envelope["ok"]:
        raise RuntimeError(f"{operation}: {envelope['error']}")
    return envelope["result"]


def document(client, project, artifact):
    return json.loads(client.read_artifact(project, artifact))


def manifest(client, run, job):
    analysis = document(client, run["project_id"], output(job, "analysis-manifest"))
    return {stage["kind"]: stage["artifact_id"] for stage in analysis["stages"]}


def ranking_v2(client, run, stages):
    ranking = document(client, run["project_id"], stages["ranking.set.v2"])
    assert ranking["schema_version"] == "clipmill.ranking.set.v2"
    listed = engine(client, "list_clips", **run)["clips"]
    assert [clip["clip_id"] for clip in listed] == [
        clip["plan"]["plan_id"] for clip in ranking["clips"]
    ], "The visible suggestions must match the published ranking, including an empty one."
    assert all(clip["quality"]["verdict"] == "accept" for clip in ranking["clips"])
    if not ranking["clips"] and not ranking.get("cutoff_reason"):
        raise RuntimeError("The empty ranking has no explanation.")
    return ranking


def render_cut(client, work, out, label, run, transcript, direct):
    preview = client._call(
        ipc.Request(preview_direct=ipc.PreviewDirectRequest(direct=direct))
    ).preview_direct
    assert preview.frame_count and preview.proxies, "Draft preview has no playable programme."
    directed = client._call(ipc.Request(direct_clip=direct)).direct_clip.doc
    assert directed.job_id == run["job_id"]
    saved = client._call(
        ipc.Request(
            get_preview_plan=ipc.GetPreviewPlanRequest(
                project_id=run["project_id"], doc_id=directed.doc_id
            )
        )
    ).get_preview_plan
    assert saved.frame_count == preview.frame_count and saved.segments == preview.segments
    draft = engine(
        client, "render_preview", doc_id=directed.doc_id, expected_revision=directed.revision
    )
    rendered = wait_job(client, draft["job_id"])
    artifact = verify_artifact(work, output(rendered, "render-clip"))
    assert artifact.kind == "render.clip.v1"
    metadata = json.loads(artifact_file(artifact, "render-manifest.json").read_text())
    assert metadata["rights"]["source_attestation"] == "Local working preview. Rights unassessed."
    destination = out / f"{label}.mp4"
    shutil.copyfile(artifact_file(artifact, "clip.mp4"), destination)
    subprocess.run(
        [str(ROOT / ".cache/bin/ffmpeg"), "-v", "error", "-i", str(destination), "-f", "null", "-"],
        check=True,
    )
    seconds = float(
        subprocess.check_output(
            [
                str(ROOT / ".cache/bin/ffprobe"),
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=nk=1:nw=1",
                str(destination),
            ],
            text=True,
        ).strip()
    )
    duration = sum(segment.out_ticks - segment.in_ticks for segment in preview.segments) / 90000
    assert abs(duration - seconds) < 0.2, f"Draft {duration:.3f}s, decoded {seconds:.3f}s"
    excerpts = []
    for segment in preview.segments:
        spoken = [
            word["text"]
            for word in transcript["words"]
            if segment.in_ticks
            <= (word["start_ticks"] + word["end_ticks"]) // 2
            < segment.out_ticks
        ]
        excerpts.append(
            {
                "in_seconds": segment.in_ticks / 90000,
                "out_seconds": segment.out_ticks / 90000,
                "text": " ".join(spoken),
            }
        )
    return {
        "doc_id": directed.doc_id,
        "revision": directed.revision,
        "movie": destination.name,
        "duration_seconds": seconds,
        "frames": preview.frame_count,
        "source_ranges": excerpts,
    }


def ranked_cut(client, work, out, label, run, ranking, transcript):
    if not ranking["clips"]:
        return {"empty": ranking["cutoff_reason"]}
    winner = ranking["clips"][0]
    plan = winner["plan"]
    detail = engine(client, "describe_clip", **run, clip_id=plan["plan_id"])
    assert detail["form"] == plan["form"]
    direct = ipc.DirectClipRequest(
        **run,
        candidate_id=plan["plan_id"],
        cut=ipc.CLIP_CUT_V1_CHOSEN,
        plan_json=json.dumps(plan),
        variation=True,
    )
    result = render_cut(client, work, out, label, run, transcript, direct)
    assert abs(result["duration_seconds"] - plan["duration_ticks"] / 90000) < 0.2
    return {
        **result,
        "title": plan.get("title", plan["form"]),
        "form": plan["form"],
        "keep_score": winner["keep_score"],
        "reasons": winner["reasons"],
        "quality": winner["quality"],
        "parts": plan["parts"],
    }


def review_files(out, cuts):
    def card(label, cut):
        heading = f"<h2>{html.escape(label)}</h2>"
        if "movie" not in cut:
            return f"<article>{heading}<p>{html.escape(cut['empty'])}</p></article>"
        cues = "".join(
            f"<p><small>{s['in_seconds']:.2f} to {s['out_seconds']:.2f} s in source</small>"
            f"{html.escape(s['text'])}</p>"
            for s in cut["source_ranges"]
        )
        reasons = " ".join(html.escape(reason) for reason in cut.get("reasons", []))
        return (
            f"<article>{heading}<h3>{html.escape(cut['title'])}</h3>"
            f"<video controls preload='metadata' src='{html.escape(cut['movie'])}'></video>"
            f"<p>{cut['duration_seconds']:.2f} s · {cut['frames']} frames</p>"
            f"<p>{reasons}</p>{cues}</article>"
        )

    content = "".join(card(label, cut) for label, cut in cuts)
    page = f"""<!doctype html>
<html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>Editorial cut comparison</title>
<style>
body {{ color:#e8e7e3; background:#232320; font:16px/1.5 system-ui; margin:24px }}
main {{ display:grid; gap:24px;
       grid-template-columns:repeat(auto-fit,minmax(min(300px,100%),1fr)) }}
video {{ width:100%; height:min(52vh,520px); background:#151513 }}
small {{ display:block; color:#bcbab4 }} h3 {{ font:inherit }}
</style>
<h1>Editorial cut comparison</h1><p>Local working previews. Rights unassessed.</p>
<main>{content}</main></html>"""
    (out / "review.html").write_text(page, encoding="utf-8")
    original, ranked = (cut for _, cut in cuts[:2])
    if "movie" not in original or "movie" not in ranked:
        return
    # Each source keeps its own selectable audio track. The shorter picture
    # turns black when that cut ends.
    duration = max(original["duration_seconds"], ranked["duration_seconds"])
    filters = []
    for i, cut in enumerate((original, ranked)):
        filters.append(
            f"[{i}:v]scale=404:720:force_original_aspect_ratio=decrease,pad=404:720:(ow-iw)/2:(oh-ih)/2,setsar=1,"
            f"tpad=stop_mode=add:stop_duration={duration - cut['duration_seconds']:.6f}[v{i}]"
        )
    filters.append("[v0][v1]hstack=inputs=2[pair]")
    subprocess.run(
        [
            str(ROOT / ".cache/bin/ffmpeg"),
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            str(out / original["movie"]),
            "-i",
            str(out / ranked["movie"]),
            "-filter_complex",
            ";".join(filters),
            "-map",
            "[pair]",
            "-map",
            "0:a:0",
            "-map",
            "1:a:0",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "22",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "copy",
            "-metadata:s:a:0",
            "handler_name=Original cut",
            "-metadata:s:a:1",
            "handler_name=Ranked cut",
            "-disposition:a:0",
            "default",
            "-disposition:a:1",
            "0",
            "-movflags",
            "+faststart",
            str(out / "comparison.mp4"),
        ],
        check=True,
    )


def export_fixture(client, out, cut):
    request = ipc.ExportRequestV1(
        doc_id=cut["doc_id"],
        destination_dir=str(out),
        source_attestation="own_content",
        gates_passed=["duration_60s"] if cut["duration_seconds"] > 60 else [],
        ai_assistance=["asr_captions", "reframe", "clip_selection"],
        index=1,
        date=date.today().isoformat(),
        title="editorial smoke",
        expected_revision=cut["revision"],
    )
    out.mkdir()
    validation = client._call(
        ipc.Request(plan_export=ipc.PlanExportRequest(request=request))
    ).plan_export.validation
    assert validation.passes, str(validation)
    job = client._call(ipc.Request(export_clip=ipc.ExportClipRequest(request=request))).export_clip
    wait_job(client, job.job_id)
    for pattern in ("*.mp4", "*.srt", "*.vtt"):
        assert next(out.glob(pattern)).stat().st_size > 20


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--recording", type=Path, help="Use this local recording in full with rights unassessed."
    )
    parser.add_argument(
        "--daemon",
        type=Path,
        default=Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/clipmilld",
    )
    parser.add_argument("--work", type=Path)
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Use the previously analyzed recording in --work/run.json.",
    )
    parser.add_argument(
        "--brief", default="Find the clearest self-contained idea with a concrete payoff."
    )
    parser.add_argument(
        "--reprompt",
        default="Focus on a specific claim or example and keep the cut under 45 seconds.",
    )
    parser.add_argument("--language", default="en")
    args = parser.parse_args()
    if args.resume and (not args.work or args.recording):
        parser.error("--resume uses the recording already registered in --work")
    os.umask(0o077)
    work = (args.work or Path(tempfile.mkdtemp(prefix="cm-editorial-", dir="/tmp"))).resolve()
    work.mkdir(parents=True, exist_ok=True, mode=0o700)
    source = (args.recording or ROOT / "target/milestone-1/talk.mp4").resolve()
    if not args.resume and not source.is_file():
        parser.error("Provide --recording or generate the milestone talk with gate-milestone-1.")
    out = Path(tempfile.mkdtemp(dir=work, prefix="review-"))
    print(f"editorial-live: private comparison in {out}", flush=True)
    env = {
        **os.environ,
        "CLIPMILL_MODELS_DIR": str(ROOT / "models/registry"),
        "CLIPMILL_WEIGHTS_DIR": str(ROOT / ".cache/models"),
        "CLIPMILL_SOCKET": str(work / "review.sock"),
        "CLIPMILL_EDITORIAL_CLOUD": "0",
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
                    str(work / "review.sock"),
                    "--ffprobe",
                    str(ROOT / ".cache/bin/ffprobe"),
                ],
            ),
            ("workers", [str(ROOT / "tools/run-workers.sh"), "--data-dir", str(work)]),
        ]:
            if name == "workers":
                deadline = time.monotonic() + 60
                while not (work / "review.sock").exists():
                    if time.monotonic() > deadline:
                        raise TimeoutError("daemon socket")
                    time.sleep(0.1)
            with (work / f"{name}.log").open("w") as log:
                children.append(
                    subprocess.Popen(
                        command, cwd=ROOT, env=env, stdout=log, stderr=log, start_new_session=True
                    )
                )
        client = DaemonClient(work / "review.sock", timeout_seconds=180)
        deadline = time.monotonic() + 600
        while True:
            readiness = client._call(
                ipc.Request(get_readiness=ipc.GetReadinessRequest(language=args.language))
            ).get_readiness
            stages = [s for s in readiness.stages if not s.stage.endswith("-cloud")]
            if stages and all(s.ready for s in stages):
                break
            if "editorial runtime is not ready" in (work / "workers.log").read_text():
                raise RuntimeError(f"Qwen runtime readiness failed; see {work / 'workers.log'}")
            if time.monotonic() > deadline:
                raise TimeoutError(str(readiness))
            time.sleep(2)
        if args.resume:
            run = json.loads((work / "run.json").read_text())
            run = {key: run[key] for key in ("project_id", "source_id", "job_id")}
            job = wait_job(client, run["job_id"])
        else:
            project = client.create_project("Editorial cut review")
            registered = client.register_source(project.project_id, source)
            payload = ipc.AnalyzeSourcePayloadV1(
                key_version="clipmill.analyze-source.v1",
                source_id=registered.source.source_id,
                language=args.language,
                local_editorial=True,
                editorial_prompt=args.brief,
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
            run = {
                "project_id": project.project_id,
                "source_id": registered.source.source_id,
                "job_id": submitted.job_id,
            }
            (work / "run.json").write_text(json.dumps(run))
            job = wait_job(client, submitted.job_id)
        assert not any(t.kind.endswith("-cloud") for t in job.tasks)
        stages = manifest(client, run, job)
        ranking = ranking_v2(client, run, stages)
        revised_job = engine(
            client, "reprompt", **run, prompt=args.reprompt, min_seconds=20, max_seconds=45, count=2
        )
        assert revised_job["job_id"] != job.job_id
        revised = {**run, "job_id": revised_job["job_id"]}
        transcript = document(client, run["project_id"], stages["speech.transcript.v1"])
        prior = (
            document(client, run["project_id"], stages["ranking.set.v1"])
            if "ranking.set.v1" in stages
            else {}
        )
        original = {"empty": "The original ranking selected no cut."}
        if prior.get("selected"):
            candidate = prior["selected"][0]
            direct = ipc.DirectClipRequest(
                **run, candidate_id=candidate, cut=ipc.CLIP_CUT_V1_CHOSEN, variation=True
            )
            original = render_cut(client, work, out, "original", run, transcript, direct)
            selected = next(c for c in prior["cohort"] if c["candidate_id"] == candidate)
            original.update(
                title=selected.get("title") or "Top moment from the original ranking",
                candidate_id=candidate,
            )
        ranked = ranked_cut(client, work, out, "ranked", run, ranking, transcript)
        completed = wait_job(client, revised["job_id"])
        assert completed.kind == "reprompt"
        later = manifest(client, revised, completed)
        for kind in ("speech.transcript.v1", "mind.memory.v1", "timeline.index.v1"):
            assert stages[kind] == later[kind], f"The revised brief lost its original {kind}"
        reranked = ranking_v2(client, revised, later)
        assert reranked["editorial_prompt"] == args.reprompt
        changed = ranked_cut(client, work, out, "revised", revised, reranked, transcript)
        cuts = [
            ("Original ranking", original),
            ("Editorial ranking", ranked),
            ("Revised brief", changed),
        ]
        review_files(out, cuts)
        report = {
            "run": run,
            "revised_run": revised,
            "brief": ranking.get("editorial_prompt"),
            "revised_brief": args.reprompt,
            "cuts": dict(cuts),
            "approved_count": len(ranking["clips"]),
            "revised_count": len(reranked["clips"]),
            "reused_evidence": True,
            "rights": "unassessed",
        }
        if args.recording is None and not args.resume:
            if "movie" not in ranked:
                raise RuntimeError(
                    f"The generated fixture produced no reviewed cut. See {out / 'review.html'}"
                )
            export_fixture(client, out / "delivered", ranked)
            report["fixture_export_attestation"] = "own_content"
            report["model_edge_checks"] = model_edge_checks(out, source)
        (out / "result.json").write_text(json.dumps(report, indent=2))
        print(
            f"editorial-live: verified {len(ranking['clips'])} ranked and "
            f"{len(reranked['clips'])} reprompted cuts. Open {out / 'review.html'}",
            flush=True,
        )
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
