import json

from clipmill_worker_editorial.score import score_units
from clipmill_worker_sdk import CancellationToken
from clipmill_worker_sdk.gen.schemas.mind_unit_scores import MindUnitScores


class Scripted:
    def __init__(self, *answers):
        self.answers = list(answers)
        self.prompts = []

    def generate(self, prompt, schema, max_tokens, images=None):
        self.prompts.append(prompt)
        answer = self.answers.pop(0)
        return (answer if isinstance(answer, str) else json.dumps(answer)), {
            "input": 1,
            "output": 1,
        }


def script():
    return {
        "level": "compact",
        "digest": "sha256:" + "c" * 64,
        "units": [
            {"index": i, "start_ticks": i, "end_ticks": i + 1, "speaker": "A", "text": "x"}
            for i in range(12)
        ],
        "windows": [
            {"text": "s0-s5", "units": list(range(6))},
            {"text": "s6-s11", "units": list(range(6, 12))},
        ],
    }


def score(runtime):
    return score_units(
        runtime,
        script(),
        source_fingerprint="sha256:" + "a" * 64,
        implementation="clipmill-worker-editorial@0.1.0+test",
        model="test-model",
        max_tokens=1000,
        identity={"route": "local"},
        traces=[],
        cancellation=CancellationToken(),
    )


def test_marks_become_a_row_for_every_sentence():
    runtime = Scripted(
        {
            "hooks": [{"unit": "s1", "level": "good"}, {"unit": "s1", "level": "strong"}],
            "landings": [{"unit": "s5", "level": "strong"}],
            "needs": [{"unit": "s5", "on": "s1"}, {"unit": "s1", "on": "s5"}],
        },
        "the model wandered off",
    )
    scores = score(runtime)
    MindUnitScores.model_validate(scores)
    rows = {row["unit"]: row for row in scores["units"]}
    assert len(rows) == 12
    assert rows["s1"]["hook"] == "strong"
    assert rows["s5"]["landing"] == "strong"
    # What a cut may leave out is found by the cutter in code, not marked here.
    assert all(row["trim"] == "keep" for row in rows.values())
    assert rows["s5"]["needs"] == "s1"
    assert "needs" not in rows["s1"]
    assert scores["unscored"][0]["first_unit"] == "s6"
    assert '"citable_units": "s6 to s11"' in runtime.prompts[1]


def test_the_markup_asks_for_no_cut_labels():
    runtime = Scripted(
        {"hooks": [], "landings": [], "needs": []},
        {"hooks": [], "landings": [], "needs": []},
    )
    rows = score(runtime)["units"]
    assert all(row["trim"] == "keep" for row in rows)
    prompt = runtime.prompts[0]
    assert all(f"s{i} [0:00] A x" in prompt for i in range(6))
    assert "cuts" not in prompt
