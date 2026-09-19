"""The offline gate must verify footage, without forbidding camera-cut framing."""

import json
import runpy
from pathlib import Path

import pytest
from clipmill.ipc.v1 import daemon_pb2 as pb

drill = runpy.run_path(str(Path(__file__).resolve().parents[3] / "tools/drills/lock_phase1.py"))
GateFailure = drill["GateFailure"]
direct_the_top_clip = drill["direct_the_top_clip"]
require_directed_span = drill["require_directed_span"]
SOURCE = "sha256:" + "1" * 64
START = 600 * 90_000
MID = 610 * 90_000
END = 620 * 90_000
CHOSEN = {"start_ticks": START, "end_ticks": END}


def segment(start: int, end: int, source: str = SOURCE) -> dict:
    return {"in_ticks": start, "out_ticks": end, "source_fingerprint": source}


def response(segments: list[dict], start: int = START, end: int = END) -> pb.DirectClipResponse:
    return pb.DirectClipResponse(
        doc=pb.EditDoc(
            doc_id="directed", document_json=json.dumps({"video": {"segments": segments}})
        ),
        start_ticks=start,
        end_ticks=end,
    )


@pytest.mark.parametrize(
    "segments", [[segment(START, END)], [segment(START, MID), segment(MID, END)]]
)
def test_directed_gate_accepts_one_or_multiple_shots_covering_the_chosen_span(segments):
    class FixtureClient:
        def read_artifact(self, project_id, artifact_id):
            assert (project_id, artifact_id) == ("project", "ranking")
            return json.dumps(
                {
                    "source_fingerprint": SOURCE,
                    "selected": ["selected"],
                    "cohort": [
                        {
                            "candidate_id": "other",
                            "boundary": {"chosen": {"start_ticks": 0, "end_ticks": 1}},
                        },
                        {"candidate_id": "selected", "boundary": {"chosen": CHOSEN}},
                    ],
                }
            )

        def direct_clip(self, project_id, source_id, candidate_id):
            assert (project_id, source_id, candidate_id) == ("project", "source", "selected")
            return response(segments)

    manifest = {"stages": [{"kind": "ranking.set.v1", "artifact_id": "ranking"}]}
    assert direct_the_top_clip(FixtureClient(), "project", "source", manifest) == "directed"


@pytest.mark.parametrize(
    "segments",
    [
        [],
        [segment(START, MID), segment(MID, END, "another-source")],
        [segment(START, MID), segment(MID + 1, END)],
        [segment(START, MID), segment(MID - 1, END)],
        [segment(MID, END), segment(START, MID)],
        [segment(START, MID), segment(START, MID)],
        [segment(START, START)],
        [segment(START, START - 1)],
        [segment(START + 1, END)],
        [segment(START, END - 1)],
        [segment(START, END + 1)],
        [segment(START, MID), segment(MID + 0.5, END)],
    ],
    ids=[
        "empty",
        "wrong-source",
        "gap",
        "overlap",
        "reversed",
        "repeated",
        "zero",
        "negative",
        "late-head",
        "missing-tail",
        "extra-tail",
        "fractional-tick",
    ],
)
def test_directed_gate_rejects_changed_or_discontinuous_footage(segments):
    directed = response(segments)
    with pytest.raises(GateFailure):
        require_directed_span(directed, json.loads(directed.doc.document_json), SOURCE, CHOSEN)


@pytest.mark.parametrize("start,end", [(START, MID), (START + 1, END), (START, END + 1)])
def test_directed_response_must_report_the_entire_chosen_span(start, end):
    directed = response([segment(START, MID), segment(MID, END)], start, end)
    with pytest.raises(GateFailure, match="response does not cover"):
        require_directed_span(directed, json.loads(directed.doc.document_json), SOURCE, CHOSEN)
