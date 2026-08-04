#!/usr/bin/env python3
"""Drive the whole product against a live daemon and a live worker fleet.

The Local Lock claim is "zero hidden cloud dependencies". Everything that had
ever been run offline stopped before the stages that would actually phone home:
the analyze gate detects shots on a silent video, the shell gate stops at
ingest, and the speech conformance harness imports the model classes in-process
rather than leasing them. This is the part nobody had run.

What it asserts is deliberately about *stages having happened*, not about
outputs being pretty. A pipeline that quietly skipped its recognizer and
published an honest "no speech here" manifest would be a green Local Lock for
a product that never loaded a model — so every leased stage is checked to be
present in the manifest's `stages` and absent from its `skipped`, and the
transcript is checked to contain words. Those two together are what make the
gate mean "a model ran, offline" rather than "nothing crashed".
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

from clipmill.ipc.v1 import daemon_pb2 as pb
from clipmill_eval.client import DaemonClient

# The stages that leave the daemon's process, named the way the manifest names
# them: by the artifact kind published, not the task kind that ran. Each loads
# pinned weights over a lease, which is the whole reason this gate exists.
LEASED_STAGES = (
    "speech.vad.v1",
    "speech.asr.v1",
    "speech.alignment.v1",
    "evidence.shots.v1",
)
TRANSCRIPT_KIND = "speech.transcript.v1"
RANKING_KIND = "ranking.set.v1"
# Analysis of a real recording is a decode, three models and a search. On a
# cold hosted runner with no GPU this is minutes, not seconds.
ANALYZE_TIMEOUT_SECONDS = 900.0
EXPORT_TIMEOUT_SECONDS = 300.0


class GateFailure(RuntimeError):
    """Something the gate exists to catch."""


def wait_for_job(client: DaemonClient, job_id: str, timeout_seconds: float) -> pb.Job:
    """Poll until the job settles, reporting what it was doing if it does not.

    The client's own `wait_for_job` has a 30-second ceiling suited to the
    stages that run inside the daemon. This one says which stage it was on
    when it gave up, because "analyze did not finish" is not a diagnosis.
    """

    deadline = time.monotonic() + timeout_seconds
    job = client.get_job(job_id)
    while time.monotonic() < deadline:
        job = client.get_job(job_id)
        if job.state in (pb.JOB_STATE_SUCCEEDED, pb.JOB_STATE_FAILED, pb.JOB_STATE_CANCELLED):
            return job
        time.sleep(1.0)
    running = [
        f"{task.kind}={pb.TaskState.Name(task.state)}"
        for task in job.tasks
        if task.state not in (pb.TASK_STATE_SUCCEEDED,)
    ]
    raise GateFailure(
        f"job {job_id} did not settle in {timeout_seconds:.0f}s; outstanding: {', '.join(running)}"
    )


def require_succeeded(job: pb.Job, what: str) -> None:
    if job.state != pb.JOB_STATE_SUCCEEDED:
        failed = [
            f"{task.kind}={pb.TaskState.Name(task.state)}"
            for task in job.tasks
            if task.state != pb.TASK_STATE_SUCCEEDED
        ]
        raise GateFailure(
            f"{what} ended {pb.JobState.Name(job.state)}"
            f" ({job.failure_detail or 'no detail'}); stages: {', '.join(failed) or 'none'}"
        )


def analysis_manifest(client: DaemonClient, project_id: str, job: pb.Job) -> dict:
    final = [task for task in job.tasks if task.kind == "analysis-manifest"]
    if not final or not final[0].output_artifact_id:
        raise GateFailure("the analysis published no manifest")
    return json.loads(client.read_artifact(project_id, final[0].output_artifact_id))


def published(manifest: dict) -> dict[str, str]:
    """Where each stage that ran left its document."""

    return {stage["kind"]: stage["artifact_id"] for stage in manifest.get("stages", [])}


def require_stages_ran(manifest: dict) -> None:
    """Every leased stage produced a document, and none was skipped.

    A skipped stage is an honest outcome the manifest is designed to record —
    a recording with no audio has no transcript — which is exactly why it has
    to be refused here. The fixture has speech and a cut in it, so a skip means
    a worker never took the lease, and that would otherwise pass as a green
    Local Lock over a pipeline that loaded no models at all.
    """

    ran = {stage.get("kind") for stage in manifest.get("stages", [])}
    skipped = {stage.get("kind"): stage.get("reason", "") for stage in manifest.get("skipped", [])}
    for kind in LEASED_STAGES:
        if kind in skipped:
            raise GateFailure(f"{kind} was skipped: {skipped[kind]!r} — no worker took the lease")
        if kind not in ran:
            raise GateFailure(f"{kind} is absent from the manifest; it never published")
    if not manifest.get("coverage", {}).get("analyzed"):
        raise GateFailure("the manifest does not claim the recording was analyzed")


def require_a_recognizer_ran(client: DaemonClient, project_id: str, manifest: dict) -> int:
    """The transcript carries words.

    Every stage can succeed and publish a well-formed empty document. Counting
    words is the difference between "the chain ran" and "a model produced
    something", and it is the assertion that would have caught the swallowed
    contract error that made every model worker fail silently.
    """

    address = published(manifest).get(TRANSCRIPT_KIND)
    if not address:
        raise GateFailure("the manifest names no transcript")
    transcript = json.loads(client.read_artifact(project_id, address))
    # The top-level list is the authoritative one; `segments` are index ranges
    # into it rather than copies, so counting there would count nothing.
    words = transcript.get("words", [])
    if not words:
        raise GateFailure("the transcript is empty; a recognizer ran but recognized nothing")
    return len(words)


def direct_the_top_clip(
    client: DaemonClient, project_id: str, source_id: str, manifest: dict
) -> str:
    address = published(manifest).get(RANKING_KIND)
    if not address:
        raise GateFailure("the manifest names no ranking")
    ranking = json.loads(client.read_artifact(project_id, address))
    # Ids, in order, rather than objects: the cohort holds the detail.
    selected = ranking.get("selected", [])
    if not selected:
        raise GateFailure("the ranking selected nothing to direct")
    candidate_id = selected[0]

    directed = client.direct_clip(project_id, source_id, candidate_id)
    document = json.loads(directed.doc.document_json)
    segments = document.get("video", {}).get("segments", [])
    if len(segments) != 1:
        raise GateFailure(f"a directed clip should be one segment, got {len(segments)}")
    if directed.end_ticks <= directed.start_ticks:
        raise GateFailure("the directed clip has no duration")
    return directed.doc.doc_id


def export(client: DaemonClient, project_id: str, doc_id: str, destination: Path) -> pb.Job:
    destination.mkdir(parents=True, exist_ok=True)
    job_id = client.export_clip(
        pb.ExportRequestV1(
            doc_id=doc_id,
            destination_dir=str(destination),
            # An export carries a claim about the footage and refuses to carry
            # a blank one. True here in the strictest sense: the recording was
            # synthesized by this repository's own fixture generator moments
            # ago, from the platform's voice and a pinned encoder.
            source_attestation="own_content",
            # A token from the accepted vocabulary, not prose: the manifest is
            # a rights document, and the daemon refuses a word it does not know
            # rather than passing a typo through as a statement about the work.
            # Captions are the one this run actually earns — a recognizer wrote
            # them — so claiming reframe or denoise here would be a false
            # disclosure in a gate about honesty.
            ai_assistance=["asr_captions"],
            title="lock-phase1",
        )
    )
    return wait_for_job(client, job_id, EXPORT_TIMEOUT_SECONDS)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--socket", type=Path, required=True)
    parser.add_argument("--data-dir", type=Path, required=True)
    parser.add_argument("--media", type=Path, required=True)
    parser.add_argument("--daemon-log", type=Path, required=True)
    options = parser.parse_args()

    client = DaemonClient(options.socket, timeout_seconds=60.0)

    project = client.create_project("lock-phase1")
    registered = client.register_source(project.project_id, options.media.resolve())
    source_id = registered.source.source_id
    print(f"    registered {options.media.name} as {source_id}")

    print("==> analyze: nineteen stages, five of them leased to worker processes")
    started = time.monotonic()
    job = wait_for_job(
        client,
        client.submit_analyze(project.project_id, source_id).job_id,
        ANALYZE_TIMEOUT_SECONDS,
    )
    require_succeeded(job, "analyze")
    elapsed = time.monotonic() - started
    succeeded = sum(1 for task in job.tasks if task.state == pb.TASK_STATE_SUCCEEDED)
    print(f"    {succeeded}/{len(job.tasks)} stages in {elapsed:.0f}s")

    manifest = analysis_manifest(client, project.project_id, job)
    require_stages_ran(manifest)
    words = require_a_recognizer_ran(client, project.project_id, manifest)
    print(f"    every leased stage published; the transcript carries {words} words")

    print("==> direct a clip from what the workers published")
    doc_id = direct_the_top_clip(client, project.project_id, source_id, manifest)
    print(f"    edit document {doc_id}")

    print("==> render and deliver it")
    destination = options.data_dir.parent / "exported"
    exported = export(client, project.project_id, doc_id, destination)
    require_succeeded(exported, "export")
    delivered = sorted(path for path in destination.rglob("*.mp4") if path.is_file())
    if not delivered:
        raise GateFailure(f"the export succeeded but wrote no video under {destination}")
    written = delivered[0]
    if written.stat().st_size <= 0:
        raise GateFailure(f"{written} is empty")
    print(f"    wrote {written.name} ({written.stat().st_size} bytes)")

    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except GateFailure as failure:
        print(f"lock-phase1: {failure}", file=sys.stderr)
        sys.exit(1)
