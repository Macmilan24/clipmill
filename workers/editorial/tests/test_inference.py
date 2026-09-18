import json
from types import SimpleNamespace

import pytest
from clipmill_worker_editorial.inference import ReviewReply, call, propose, review
from clipmill_worker_sdk import CancellationToken, LeaseCancelled


class Runtime:
    def __init__(self, raw):
        self.raw = raw
        self.prompts = []

    def generate(self, prompt, *args):
        self.prompts.append(prompt)
        return self.raw, {"input": 100, "output": 20}


class SequenceRuntime(Runtime):
    def __init__(self, replies):
        super().__init__(None)
        self.replies = iter(replies)

    def generate(self, prompt, *args):
        self.prompts.append(prompt)
        reply = next(self.replies)
        if isinstance(reply, Exception):
            raise reply
        return reply, {"input": 100, "output": 20}


IDENTITY = {"route": "local", "model": {"name": "test"}}


def test_visual_stage_reads_the_declared_ingest_frame_index(tmp_path):
    from clipmill_worker_editorial import execute_look

    descriptors = {
        "discovery.candidates.v1": (
            "candidates.json",
            {"source_fingerprint": "source", "candidates": []},
        ),
        "editorial.judgments.v1": (
            "judgments.json",
            {
                "source_fingerprint": "source",
                "candidates": [],
                "inputs": {"candidates_artifact_id": "discovery.candidates.v1"},
            },
        ),
        "media.frames.v1": ("index.json", {"source_fingerprint": "source", "frames": []}),
    }
    for kind, (name, document) in descriptors.items():
        (tmp_path / kind).mkdir()
        (tmp_path / kind / name).write_text(json.dumps(document))

    def artifact_file(kind, name):
        assert name == descriptors[kind][0], "Read only the declared payload name"
        return tmp_path / kind / name

    outputs = {}
    context = SimpleNamespace(
        artifact_file=artifact_file,
        cancellation=CancellationToken(),
        staging=SimpleNamespace(write_bytes=lambda name, data: outputs.update({name: data})),
        report_progress=lambda *args: None,
    )
    inputs = SimpleNamespace(require=lambda kind: SimpleNamespace(artifact=kind, artifact_id=kind))
    execute_look(
        context,
        inputs,
        SimpleNamespace(name="qwen", digest="digest", root=tmp_path),
        SimpleNamespace(stage="editorial-look", prompt_digest="digest"),
    )
    assert json.loads(outputs["looks.json"])["checks"] == []


@pytest.mark.parametrize("failure_phase", ["load", "generate", "cancel"])
def test_failed_optional_visual_check_does_not_erase_nonvisual_results(
    tmp_path, monkeypatch, failure_phase
):
    from clipmill_worker_editorial import execute_look

    ids = ["cand_0000000000000001", "cand_0000000000000002"]
    documents = {
        "discovery.candidates.v1": (
            "candidates.json",
            {
                "source_fingerprint": "source",
                "candidates": [
                    {"id": candidate_id, "intervals": [{"start_ticks": 0, "end_ticks": 90000}]}
                    for candidate_id in ids
                ],
            },
        ),
        "editorial.judgments.v1": (
            "judgments.json",
            {
                "source_fingerprint": "source",
                "inputs": {"candidates_artifact_id": "discovery.candidates.v1"},
                "candidates": [
                    {
                        "candidate_id": candidate_id,
                        "outcome": "answered",
                        "status": "accepted",
                        "visual_dependency": i == 1,
                        "summary": "A complete moment",
                        "reasons": [],
                    }
                    for i, candidate_id in enumerate(ids)
                ],
            },
        ),
        "media.frames.v1": (
            "index.json",
            {
                "source_fingerprint": "source",
                "frames": [{"t_ticks": 45000, "file": "frame.jpg"}],
            },
        ),
    }
    for kind, (name, document) in documents.items():
        (tmp_path / kind).mkdir()
        (tmp_path / kind / name).write_text(json.dumps(document))

    class BrokenVisual:
        def __init__(self, *args):
            if failure_phase == "load":
                raise RuntimeError("visual allocation unavailable")

        def generate(self, *args):
            if failure_phase == "cancel":
                raise LeaseCancelled("cancelled")
            raise RuntimeError("visual inference unavailable")

        def close(self):
            pass

    monkeypatch.setattr("clipmill_worker_editorial.runtime.LocalModel", BrokenVisual)
    outputs = {}
    context = SimpleNamespace(
        artifact_file=lambda kind, name: tmp_path / kind / name,
        cancellation=CancellationToken(),
        staging=SimpleNamespace(write_bytes=lambda name, data: outputs.update({name: data})),
        report_progress=lambda *args: None,
        lease=SimpleNamespace(artifact_root=str(tmp_path / "artifacts"), task_id="tsk_VISUAL"),
    )
    args = (
        context,
        SimpleNamespace(require=lambda kind: SimpleNamespace(artifact=kind, artifact_id=kind)),
        SimpleNamespace(name="qwen", digest="digest", root=tmp_path),
        SimpleNamespace(stage="editorial-look", prompt_digest="digest"),
    )
    if failure_phase == "cancel":
        with pytest.raises(LeaseCancelled):
            execute_look(*args)
        assert outputs == {}
    else:
        assert execute_look(*args) == ("looks.json", "trace.json")
        checks = json.loads(outputs["looks.json"])["checks"]
        assert len(checks) == 1 and checks[0]["candidate_id"] == ids[1]
        assert checks[0]["outcome"] == "failed" and "answer" not in checks[0]
        assert checks[0]["failure"]["failure_class"] == "transient"
        assert json.loads(outputs["trace.json"])["calls"][0]["outcome"] == "failed"
        assert (tmp_path / "state/editorial-traces/tsk_VISUAL.json").exists()


def test_long_recording_outline_is_bounded_but_keeps_local_and_global_citations():
    from clipmill_worker_editorial.inference import bounded_outline

    outline = [{"first_sentence_index": i * 10, "sentence_count": 10} for i in range(1000)]
    selected = bounded_outline(outline, 5010, 5050)
    assert len(selected) <= 32
    assert outline[0] in selected and outline[-1] in selected
    assert all(outline[i] in selected for i in range(501, 505))


def window():
    return {
        "outline": [],
        "sentences": [
            {"index": 0, "start_ticks": 0, "end_ticks": 90000, "text": "A complete sentence."}
        ],
        "windows": [
            {
                "index": 0,
                "first_sentence_index": 0,
                "sentence_count": 1,
                "context_before_sentences": 0,
                "context_after_sentences": 0,
            }
        ],
    }


def test_none_and_model_failure_are_distinct():
    args = (window(), SimpleNamespace(min_ticks=90000, max_ticks=900000), 2048, IDENTITY)
    traces = []
    rows = propose(Runtime('{"proposals":[]}'), *args, traces, CancellationToken())
    assert rows[0]["status"] == "none" and traces[0]["outcome"] == "none"
    traces = []
    rows = propose(Runtime("Here is my answer: {}"), *args, traces, CancellationToken())
    assert (
        rows[0]["status"] == "malformed" and rows[0]["failure"]["failure_class"] == "deterministic"
    )
    assert traces[0]["response_text"] == "Here is my answer: {}"


def test_bad_window_preserves_failure_and_does_not_skip_later_windows():
    windows = window()
    windows["windows"] = [{**windows["windows"][0], "index": i} for i in range(3)]
    traces = []
    runtime = SequenceRuntime(["broken JSON", TimeoutError("call deadline"), '{"proposals":[]}'])
    rows = propose(
        runtime,
        windows,
        SimpleNamespace(min_ticks=90000, max_ticks=900000),
        2048,
        IDENTITY,
        traces,
        CancellationToken(),
    )
    assert [r["window_index"] for r in rows] == [0, 1, 2]
    assert [r["status"] for r in rows] == ["malformed", "failed", "none"]
    assert rows[0]["failure"]["failure_class"] == "deterministic"
    assert rows[1]["failure"]["failure_class"] == "transient"
    assert "failure" not in rows[2]
    assert [t["outcome"] for t in traces] == ["malformed", "failed", "none"]


def test_cancellation_still_aborts_instead_of_publishing_partial_answers():
    windows = window()
    windows["windows"] = [{**windows["windows"][0], "index": i} for i in range(3)]
    traces = []
    runtime = SequenceRuntime(['{"proposals":[]}', LeaseCancelled("cancelled")])
    with pytest.raises(LeaseCancelled):
        propose(
            runtime,
            windows,
            SimpleNamespace(min_ticks=90000, max_ticks=900000),
            2048,
            IDENTITY,
            traces,
            CancellationToken(),
        )
    assert len(runtime.prompts) == 2
    assert [t["outcome"] for t in traces] == ["none", "cancelled"]


def test_proposer_can_only_choose_spans_that_meet_requested_duration():
    from clipmill_worker_editorial.inference import window_reply
    from pydantic import ValidationError

    sentences = [
        {"index": i, "text": "Sentence", "start_ticks": i * 90000, "end_ticks": (i + 1) * 90000}
        for i in range(4)
    ]
    duration = SimpleNamespace(min_ticks=2 * 90000, max_ticks=3 * 90000)
    response, ranges = window_reply(sentences, 1, 4, duration)
    assert ranges == [
        {"first": 1, "last_min": 2, "last_max": 3},
        {"first": 2, "last_min": 3, "last_max": 3},
    ]
    moment = dict(
        span_ref="1:2",
        title="Moment",
        hook="Hook",
        setup="Setup",
        payoff="Payoff",
        reason="Reason",
        uncertainties=[],
    )
    response.model_validate({"proposals": [moment]})
    for invalid in ("0:2", "1:1", "1:4"):
        with pytest.raises(ValidationError):
            response.model_validate({"proposals": [{**moment, "span_ref": invalid}]})
    w = {
        "outline": [],
        "sentences": sentences,
        "windows": [
            {
                "index": 0,
                "first_sentence_index": 1,
                "sentence_count": 3,
                "context_before_sentences": 1,
                "context_after_sentences": 0,
            }
        ],
    }
    rows = propose(
        Runtime(json.dumps({"proposals": [moment]})),
        w,
        duration,
        2048,
        IDENTITY,
        [],
        CancellationToken(),
    )
    proposal = rows[0]["proposals"][0]
    assert proposal["first_sentence_index"] == 1 and proposal["sentence_count"] == 2
    assert "span_ref" not in proposal and proposal["id"] == "prop_0_0"


def test_review_includes_neighbours_and_cannot_invent_an_enum():
    runtime = Runtime('{"status":"viral","reasons":[],"visual_dependency":false,"summary":"Great"}')
    traces = []
    rows = review(
        runtime,
        window(),
        {
            "candidates": [
                {
                    "id": "cand_0000000000000001",
                    "intervals": [{"start_ticks": 0, "end_ticks": 90000}],
                }
            ]
        },
        2048,
        IDENTITY,
        traces,
        CancellationToken(),
    )
    assert rows[0]["outcome"] == "malformed"
    assert "A complete sentence." in runtime.prompts[0]


def test_bad_review_does_not_drop_other_candidates():
    runtime = SequenceRuntime(
        [
            RuntimeError("temporarily unavailable"),
            '{"status":"accepted","reasons":[],"visual_dependency":false,"summary":"Complete"}',
        ]
    )
    candidates = {
        "candidates": [
            {"id": f"cand_{i:016x}", "intervals": [{"start_ticks": 0, "end_ticks": 90000}]}
            for i in (1, 2)
        ]
    }
    traces = []
    rows = review(runtime, window(), candidates, 2048, IDENTITY, traces, CancellationToken())
    assert [r["candidate_id"] for r in rows] == [c["id"] for c in candidates["candidates"]]
    assert rows[0]["outcome"] == "failed" and "status" not in rows[0]
    assert rows[1]["outcome"] == "answered" and rows[1]["status"] == "accepted"
    assert len(traces) == 2


def test_contradictory_accepted_review_keeps_its_problem_and_visual_check():
    raw = json.dumps(
        {
            "status": "accepted",
            "visual_dependency": False,
            "summary": "The explanation depends on a diagram",
            "reasons": [{"code": "visual_dependency", "detail": "The diagram is not described"}],
        }
    )
    traces = []
    rows = review(
        Runtime(raw),
        window(),
        {
            "candidates": [
                {
                    "id": "cand_0000000000000001",
                    "intervals": [{"start_ticks": 0, "end_ticks": 90000}],
                }
            ],
        },
        2048,
        IDENTITY,
        traces,
        CancellationToken(),
    )
    assert rows[0]["status"] == "needs_review"
    assert rows[0]["visual_dependency"] is True
    assert rows[0]["reasons"][0]["code"] == "visual_dependency"
    assert json.loads(traces[0]["response_text"])["status"] == "accepted"


def test_failed_runtime_is_not_a_rejection():
    class Broken:
        def generate(self, *args):
            raise RuntimeError("model unavailable")

    traces = []
    answer, failure = call(Broken(), "review", {}, ReviewReply, 2048, IDENTITY, traces)
    assert answer is None and failure["failure_class"] == "transient"
    assert traces[0]["outcome"] == "failed"


def test_strict_response_rejects_extra_fields():
    traces = []
    answer, failure = call(
        Runtime(
            json.dumps(
                {
                    "status": "accepted",
                    "reasons": [],
                    "visual_dependency": False,
                    "summary": "Complete",
                    "score": 99,
                }
            )
        ),
        "review",
        {},
        ReviewReply,
        2048,
        IDENTITY,
        traces,
    )
    assert answer is None and failure


def test_visual_check_reads_only_flagged_candidates_and_in_span_frames():
    from clipmill_worker_editorial.inference import look

    runtime = Runtime('{"answer":"yes","detail":"The diagram is visible."}')
    runtime.close = lambda: None
    candidates = {
        "candidates": [
            {"id": "cand_0000000000000001", "intervals": [{"start_ticks": 100, "end_ticks": 500}]}
        ]
    }
    verdict = {
        "candidate_id": "cand_0000000000000001",
        "outcome": "answered",
        "status": "needs_review",
        "visual_dependency": True,
        "summary": "A diagram",
        "reasons": [],
    }
    frames = {
        "frames": [{"t_ticks": t, "file": f"{t}.jpg"} for t in [0, 100, 200, 300, 400, 500, 600]]
    }
    paths = []

    def image_path(name):
        paths.append(name)
        return name

    checks = look(
        lambda: runtime,
        candidates,
        {"candidates": [verdict]},
        frames,
        image_path,
        2048,
        IDENTITY,
        [],
        CancellationToken(),
    )
    assert paths == ["100.jpg", "200.jpg", "300.jpg", "400.jpg"]
    assert checks[0]["answer"] == "yes"
    verdict["visual_dependency"] = False
    assert (
        look(
            lambda: pytest.fail("must not load a visual model"),
            candidates,
            {"candidates": [verdict]},
            frames,
            image_path,
            2048,
            IDENTITY,
            [],
            CancellationToken(),
        )
        == []
    )


def test_failed_reply_stays_out_of_success_cache(tmp_path):
    from clipmill_worker_editorial import fail_with_trace
    from clipmill_worker_sdk import RetryableTaskError

    context = SimpleNamespace(
        lease=SimpleNamespace(
            artifact_root=str(tmp_path / "artifacts"), task_id="tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"
        )
    )
    trace = {"calls": [{"outcome": "failed", "response_text": "diagnostic"}]}
    with pytest.raises(RetryableTaskError, match="unavailable"):
        fail_with_trace(
            context, [{"failure": {"failure_class": "transient", "detail": "unavailable"}}], trace
        )
    path = tmp_path / "state/editorial-traces/tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV.json"
    assert json.loads(path.read_text()) == trace
    assert path.stat().st_mode & 0o777 == 0o600
    assert not (tmp_path / "artifacts").exists()


@pytest.mark.parametrize(
    "usable",
    [{"status": "none"}, {"status": "answered"}, {"outcome": "answered", "status": "accepted"}],
)
def test_partial_answers_can_publish_with_preserved_failure_diagnostics(tmp_path, usable):
    from clipmill_worker_editorial import fail_with_trace

    failed = {"status": "failed", "failure": {"failure_class": "transient", "detail": "timeout"}}
    answers = [failed, usable]
    context = SimpleNamespace(
        lease=SimpleNamespace(artifact_root=str(tmp_path / "artifacts"), task_id="tsk_PARTIAL")
    )
    trace = {"calls": [{"outcome": "failed"}, {"outcome": "answered"}]}
    fail_with_trace(context, answers, trace)
    assert answers == [failed, usable]
    assert json.loads((tmp_path / "state/editorial-traces/tsk_PARTIAL.json").read_text()) == trace


def test_local_worker_does_not_import_cloud_adapter_even_when_inference_fails():
    import subprocess
    import sys

    result = subprocess.run(
        [
            sys.executable,
            "-c",
            """
import sys
from clipmill_worker_editorial import CAPABILITIES
from clipmill_worker_editorial.inference import call, ReviewReply
assert CAPABILITIES == ('editorial-look', 'editorial-propose', 'editorial-review')
class Broken:
    def generate(self, *args):
        raise RuntimeError('local model unavailable')
call(Broken(), 'review', {}, ReviewReply, 2048, {'route': 'local'}, [])
assert 'clipmill_worker_editorial.cloud' not in sys.modules
assert 'clipmill_worker_editorial.cloud_worker' not in sys.modules
""",
        ],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize("cloud_entrypoint", [False, True])
def test_local_and_cloud_workers_reject_each_others_leases(cloud_entrypoint):
    from clipmill.ipc.v1 import daemon_pb2
    from clipmill_worker_editorial import execute as local_execute
    from clipmill_worker_editorial.cloud_worker import execute as cloud_execute
    from clipmill_worker_sdk import DeterministicTaskError

    stage = "editorial-look" if cloud_entrypoint else "editorial-propose-cloud"
    payload = daemon_pb2.EditorialStagePayloadV1(
        key_version="clipmill.editorial-stage.v1", stage=stage
    )
    context = SimpleNamespace(
        lease=SimpleNamespace(payload=payload.SerializeToString(), kind=stage),
        cancellation=CancellationToken(),
    )
    with pytest.raises(DeterministicTaskError, match="invalid editorial lease"):
        (cloud_execute if cloud_entrypoint else local_execute)(context)


def test_cancellation_is_not_reported_as_a_failed_model():
    class Cancelled:
        def generate(self, *args):
            raise LeaseCancelled("cancelled")

    traces = []
    with pytest.raises(LeaseCancelled):
        call(Cancelled(), "review", {}, ReviewReply, 2048, IDENTITY, traces)
    assert traces[0]["outcome"] == "cancelled"


@pytest.mark.parametrize("profile", ["interview", "scripted"])
def test_content_profile_reaches_proposal_and_review_prompts(profile):
    windows = {**window(), "content_profile": profile}
    proposal_runtime = Runtime('{"proposals":[]}')
    propose(
        proposal_runtime,
        windows,
        SimpleNamespace(min_ticks=90000, max_ticks=900000),
        2048,
        IDENTITY,
        [],
        CancellationToken(),
    )
    reviewer = Runtime(
        '{"status":"accepted","reasons":[],"visual_dependency":false,"summary":"Complete"}'
    )
    review(
        reviewer,
        windows,
        {
            "candidates": [
                {
                    "id": "cand_0000000000000001",
                    "intervals": [{"start_ticks": 0, "end_ticks": 90000}],
                }
            ]
        },
        2048,
        IDENTITY,
        [],
        CancellationToken(),
    )
    for runtime in (proposal_runtime, reviewer):
        supplied = json.loads(runtime.prompts[0].split("\nInput data:\n")[1])
        assert supplied["content_profile"] == profile
        assert "SCRIPTED:" in runtime.prompts[0] and "INTERVIEW:" in runtime.prompts[0]


def test_old_tasks_default_to_interview_and_unknown_profiles_are_refused():
    from clipmill_worker_editorial import content_profile
    from clipmill_worker_sdk import DeterministicTaskError

    assert content_profile("") == "interview"
    assert content_profile("scripted") == "scripted"
    with pytest.raises(DeterministicTaskError, match="content profile"):
        content_profile("guess-and-upload")


@pytest.mark.parametrize(
    "codes,expected",
    [
        (["visual_dependency"], "needs_review"),
        (["transcript_uncertain"], "needs_review"),
        (["visual_dependency", "transcript_uncertain"], "needs_review"),
        (["visual_dependency", "incomplete_payoff"], "rejected"),
        (["transcript_uncertain", "misleading_omission"], "rejected"),
    ],
)
def test_uncertainty_is_reviewable_but_evidenced_bad_cuts_stay_declined(codes, expected):
    runtime = Runtime(
        json.dumps(
            {
                "status": "rejected",
                "reasons": [
                    {"code": code, "detail": "Specific evidence to inspect"} for code in codes
                ],
                "visual_dependency": False,
                "summary": "Check the exchange",
            }
        )
    )
    candidates = {
        "candidates": [
            {"id": "cand_0000000000000001", "intervals": [{"start_ticks": 0, "end_ticks": 90000}]}
        ]
    }
    traces = []
    rows = review(runtime, window(), candidates, 2048, IDENTITY, traces, CancellationToken())
    assert rows[0]["status"] == expected
    assert json.loads(traces[0]["response_text"])["status"] == "rejected", (
        "preserve original judgment for audit"
    )


@pytest.mark.parametrize("profile", ["interview", "scripted"])
def test_visual_evidence_remains_available_for_a_declined_moment(profile):
    from clipmill_worker_editorial.inference import look, prompt_digest

    candidate = {
        "id": "cand_0000000000000001",
        "intervals": [{"start_ticks": 0, "end_ticks": 90000}],
    }
    runtime = Runtime('{"answer":"yes","detail":"The referenced item is visible"}')
    runtime.close = lambda: None
    judgment = {
        "candidate_id": candidate["id"],
        "outcome": "answered",
        "status": "rejected",
        "visual_dependency": True,
        "reasons": [{"code": "incomplete_payoff", "detail": "The answer ends early"}],
        "summary": "An incomplete visual explanation",
    }
    traces = []
    checks = look(
        lambda: runtime,
        {"candidates": [candidate]},
        {"content_profile": profile, "candidates": [judgment]},
        {"frames": [{"t_ticks": 45000, "file": "frame.jpg"}]},
        lambda path: path,
        2048,
        IDENTITY,
        traces,
        CancellationToken(),
    )
    assert len(checks) == 1
    assert checks[0]["candidate_id"] == candidate["id"]
    instruction, supplied = runtime.prompts[0].split("\nInput data:\n")
    assert json.loads(supplied)["content_profile"] == profile
    assert "SCRIPTED:" in instruction and "INTERVIEW:" in instruction
    assert "Still frames cannot verify spoken dialogue" in instruction
    assert "do not impose an interview format" in instruction
    assert "A missing referent in sparse frames is not proof" in instruction
    assert traces[0]["prompt_version"] == f"look.v1/{prompt_digest('look')}"
    assert judgment["status"] == "rejected", "pictures must not override missing speech"
