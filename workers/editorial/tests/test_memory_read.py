import json

from clipmill_worker_editorial.memory import build_memory
from clipmill_worker_sdk import CancellationToken
from clipmill_worker_sdk.gen.schemas.mind_memory import MindMemory

S = 90_000


class Scripted:
    """Answers each pass from a queue, and keeps what it was asked."""

    def __init__(self, *answers):
        self.answers = list(answers)
        self.prompts = []

    def generate(self, prompt, schema, max_tokens, images=None):
        self.prompts.append(prompt)
        return json.dumps(self.answers.pop(0)), {"input": 1, "output": 1}


def script():
    units = [
        {
            "index": i,
            "start_ticks": i * 6 * S,
            "end_ticks": i * 6 * S + 5 * S,
            "speaker": "A" if i in (0, 3, 20) else "B",
            "text": f"Sentence number {i} of the recording.",
        }
        for i in range(30)
    ]
    return {
        "level": "compact",
        "digest": "sha256:" + "c" * 64,
        "units": units,
        "windows": [
            {"text": "s0-s14 ...", "units": list(range(15))},
            {"text": "s15-s29 ...", "units": list(range(15, 30))},
        ],
    }


def event(kind, first, last, title, names=(), mood="neutral"):
    return {
        "kind": kind,
        "title": title,
        "mood": mood,
        "first_unit": f"s{first}",
        "last_unit": f"s{last}",
        "names": list(names),
    }


def build(runtime):
    return build_memory(
        runtime,
        script(),
        source_fingerprint="sha256:" + "a" * 64,
        implementation="clipmill-worker-editorial@0.1.0+test",
        model="test-model",
        max_tokens=2000,
        identity={"route": "local"},
        traces=[],
        cancellation=CancellationToken(),
    )


def test_two_passes_make_a_memory_with_a_distant_link():
    runtime = Scripted(
        {
            "events": [
                event("question", 3, 3, "What saved the company?", ["Northwind"]),
                event("story", 4, 9, "Losing everything", mood="funny"),
                event("story", 9, 4, "A span that runs backwards"),
            ],
            "entities": [{"name": "Northwind", "kind": "organization", "aliases": ["North Wind"]}],
        },
        {
            "events": [event("answer", 21, 24, "A customer lent them money", ["north wind"])],
            "entities": [{"name": "NORTHWIND", "kind": "organization", "aliases": ["NW"]}],
        },
        {
            "chapters": [
                {
                    "title": "Starting again",
                    "summary": "The rebuild.",
                    "first_event": "m3",
                    "last_event": "m3",
                },
                {
                    "title": "Losing it",
                    "summary": "The collapse.",
                    "first_event": "m1",
                    "last_event": "m2",
                },
            ],
            "links": [
                {
                    "kind": "question_answer",
                    "from_event": "m1",
                    "to_event": "m3",
                    "note": "Answered later.",
                },
                {
                    "kind": "setup_payoff",
                    "from_event": "m3",
                    "to_event": "m2",
                    "note": "Backwards.",
                },
                {"kind": "question_answer", "from_event": "m1", "to_event": "m3", "note": "Twice."},
            ],
        },
    )
    memory = build(runtime)
    MindMemory.model_validate(memory)

    assert [e["event_id"] for e in memory["events"]] == ["m1", "m2", "m3"]
    question, story, answer = memory["events"]
    assert (question["first_unit"], question["start_ticks"]) == ("s3", 18 * S)
    assert answer["end_ticks"] == 24 * 6 * S + 5 * S
    assert story["speakers"] == ["B"]
    # One entity across both windows, found by its alias in the second.
    (northwind,) = memory["entities"]
    assert northwind["aliases"] == ["North Wind", "NW"]
    assert northwind["events"] == ["m1", "m3"]
    assert question["entities"] == answer["entities"] == ["e1"]
    # Chapters reordered into time and stretched to tile the recording.
    assert [(c["first_unit"], c["last_unit"]) for c in memory["chapters"]] == [
        ("s0", "s20"),
        ("s21", "s29"),
    ]
    assert [c["title"] for c in memory["chapters"]] == ["Losing it", "Starting again"]
    assert answer["chapter_id"] == "c2"
    # The forward link kept once; the backwards one recorded, not lost.
    assert [(k["kind"], k["from_event"], k["to_event"]) for k in memory["links"]] == [
        ("question_answer", "m1", "m3")
    ]
    assert {d["code"] for d in memory["dropped"]} == {"backwards_span", "backwards_link"}
    # The second window could cite only its own sentences.
    assert '"citable_units": "s15 to s29"' in runtime.prompts[1]


class Raw(Scripted):
    """Answers with text as given, JSON or not."""

    def generate(self, prompt, schema, max_tokens, images=None):
        answer = self.answers.pop(0)
        text = answer if isinstance(answer, str) else json.dumps(answer)
        return text, {"input": 1, "output": 1}


def test_a_window_the_model_could_not_answer_is_recorded():
    memory = build(Raw({"events": [], "entities": []}, "not json at all"))
    MindMemory.model_validate(memory)
    assert memory["events"] == []
    assert memory["chapters"] == []
    assert [d["code"] for d in memory["dropped"]] == ["unread_window"]
