"""An edit against its plain version: measured first, and shipped only when it wins both ways."""

import copy

from clipmill_worker_editorial.critique import review_batch
from clipmill_worker_editorial.judging import Book, question_id
from clipmill_worker_editorial.planning import Transcript, geometry_id
from clipmill_worker_editorial.versus import Passages, Versus, edited, plain_version
from clipmill_worker_sdk.gen.schemas.mind_quality_report import MindQualityReport

TICK = 90_000
SOURCE = "sha256:" + "b" * 64
LINES = [
    ("A", "Why did the shop close?"),
    ("B", "Rent doubled in one year."),
    ("B", "And the landlord would not negotiate."),
    ("B", "So we moved everything online."),
    ("A", "Did that work?"),
    ("B", "It worked for a while."),
    ("B", "But shipping costs ate the margin."),
    ("B", "We closed the site last spring."),
    ("A", "What would you change?"),
    ("B", "I would sign a longer lease."),
    ("B", "That's why I tell people to read the contract."),
    ("A", "Good advice. What came next?"),
    ("B", "A new shop in a smaller town."),
    ("B", "And the rent there is fair."),
    ("B", "Customers came back quickly."),
    ("A", "Any regrets?"),
    ("B", "Only that we waited so long."),
    ("B", "Because waiting cost us two years."),
    ("B", "We should have moved at once."),
    ("A", "Thanks for coming on."),
]
#: Six seconds of speech and a one-second pause per line.
STEP, SPEECH = 7, 6


def script():
    return {
        "units": [
            {
                "index": i,
                "text": text,
                "speaker": who,
                "start_ticks": i * STEP * TICK,
                "end_ticks": (i * STEP + SPEECH) * TICK,
            }
            for i, (who, text) in enumerate(LINES)
        ]
    }


def start(i):
    return i * STEP * TICK


def end(i):
    return (i * STEP + SPEECH) * TICK


def cold_open():
    """The middle line lifted to the front, and taken out of the body."""
    plan = {
        "schema_version": "clipmill.mind.clip_plan.v1",
        "source_fingerprint": SOURCE,
        "title": "Shipping ate the margin",
        "form": "cold_open",
        "parts": [
            {"part_id": "p1", "role": "hook", "start_ticks": start(6), "end_ticks": end(6)},
            {
                "part_id": "p2",
                "role": "body",
                "start_ticks": start(5),
                "end_ticks": end(7),
                "removed": [{"start_ticks": end(5), "end_ticks": start(7), "reason": "repetition"}],
            },
        ],
        "joins": [{"after_part": "p1", "kind": "cut", "technique": "hard_cut"}],
        "duration_ticks": 3 * SPEECH * TICK,
        "provenance": {"by": "mind"},
    }
    plan["plan_id"] = geometry_id(plan)
    return plan


LIFTED = "B: But shipping costs ate the margin. It worked"
PLAIN = "B: It worked for a while. But shipping"


class Judge:
    """Prefers the version with more of the passage, which is what the known
    pairs reward; `favourite`, when set, wins whatever its length."""

    def __init__(self, favourite=None, first_always=False):
        self.favourite = favourite
        self.first_always = first_always
        self.asked = []

    def score(self, version):
        bonus = 1000 if self.favourite and version.startswith(self.favourite) else 0
        return len(version.split()) + bonus

    def yes_probability(self, question, think_tokens):
        assert think_tokens == 512
        self.asked.append(question)
        if self.first_always:
            return 0.8, "The first one."
        a = question.split("Version A:\n")[1].split("\n\nVersion B:")[0]
        b = question.split("Version B:\n")[1].split("\n\nA viewer")[0]
        return (0.9 if self.score(a) > self.score(b) else 0.1), "Compared."


IDENTITY = {"route": "local", "model": {"name": "test-model"}}


def test_known_pairs_are_passages_cut_off_or_opened_on_a_pointer():
    pairs = Passages(script()).pairs()
    assert [(p["kind"], p["whole"], p["broken"]) for p in pairs] == [
        ("cut_off_ending", (1, 3), (1, 1)),
        ("dangling_start", (9, 10), (10, 10)),
        ("cut_off_ending", (5, 7), (5, 5)),
        ("cut_off_ending", (12, 14), (12, 12)),
    ]


def test_the_plain_version_is_the_body_played_whole():
    transcript = Transcript(script())
    edit = cold_open()
    plain = plain_version(edit, transcript)
    assert edited(edit) and not edited(plain)
    assert plain["form"] == "straight" and plain["joins"] == []
    assert [(p["start_ticks"], p["end_ticks"]) for p in plain["parts"]] == [(start(5), end(7))]
    assert plain["parts"][0]["units"] == ["s5", "s6", "s7"]
    assert plain["plan_id"] == geometry_id(plain) != edit["plan_id"]
    # A straight clip has nothing to compare, and a montage was asked for.
    assert plain_version(plain, transcript) is None
    assert plain_version({**edit, "form": "theme_montage"}, transcript) is None


def test_a_passing_judge_keeps_an_edit_only_when_it_wins_both_ways():
    transcript = Transcript(script())
    edit = cold_open()
    plain = plain_version(edit, transcript)
    # Prefers the version that opens on the lifted line.
    versus = Versus(Judge(favourite=LIFTED), script(), (), IDENTITY, [])
    verdict = versus.prefers_edit(edit, plain, transcript)
    assert versus.check["passed"] and versus.check["separation"] == 1.0
    assert versus.check["answered_by"] == "model"
    assert verdict["kept"] == "edit" and verdict["reason"] == "edit_preferred"
    assert verdict["edit_first"] == 0.9 and verdict["plain_first"] == 0.1
    # A judge that prefers the plain version keeps the plain one.
    versus = Versus(Judge(favourite=PLAIN), script(), (), IDENTITY, [])
    verdict = versus.prefers_edit(edit, plain, transcript)
    assert verdict == {
        "plain_plan_id": plain["plan_id"],
        "kept": "plain",
        "reason": "plain_preferred",
        "edit_first": 0.1,
        "plain_first": 0.9,
    }


def test_a_judge_that_always_picks_the_first_version_is_never_asked_about_an_edit():
    transcript = Transcript(script())
    edit = cold_open()
    plain = plain_version(edit, transcript)
    judge = Judge(first_always=True)
    versus = Versus(judge, script(), (), IDENTITY, [])
    verdict = versus.prefers_edit(edit, plain, transcript)
    assert not versus.check["passed"]
    assert verdict["kept"] == "plain" and verdict["reason"] == "not_measured"
    # It stopped once the bar was out of reach, and never saw the edit.
    assert len(judge.asked) < 2 * len(Passages(script()).pairs())
    assert not any(LIFTED in q for q in judge.asked)


class Critic:
    """A critic that approves every clip, and a planner that takes the first choice."""

    def __init__(self, runtime):
        self.runtime = runtime
        self.identity = IDENTITY
        self.traces = []
        self.brief = ""
        self.taste = []

    def ask(self, job, data, _schema):
        if job == "plan":
            return {
                "choice": data["choices"][0]["choice"],
                "title": "Shipping ate the margin",
                "hook_text": "",
                "reasons": ["A complete answer."],
                "doubts": [],
                "joins": [],
            }, None
        if job == "critic.edge":
            return {"edge_issues": []}, None
        if job == "critic.source":
            return {"effect": "preserved", "note": "The line keeps its meaning."}, None
        return {
            "comprehension": "pass",
            "landing": "pass",
            "title_accuracy": "pass",
            "hook": "strong",
            "summary": "Shipping costs closed the site.",
            "issues": [],
        }, None

    def progress(self, *_args):
        pass


def review(runtime, book=None):
    edit = cold_open()
    report = {}
    rows = review_batch(
        Critic(runtime),
        {
            "plans": [
                {
                    "candidate_id": edit["plan_id"],
                    "plan": edit,
                    "chapter": "Closing the shop",
                    "reasons": ["Shipping costs closed the site."],
                }
            ]
        },
        {
            "length": {"min_ticks": 5 * TICK, "max_ticks": 60 * TICK},
            "candidates": [{"plan": edit, "alternatives": [], "scout": "pool"}],
        },
        script(),
        report=report,
        book=book,
    )
    MindQualityReport.model_validate(
        {
            "schema_version": "clipmill.mind.quality_report.v1",
            "source_fingerprint": SOURCE,
            "inputs": {},
            "reviews": rows,
            **report,
        }
    )
    return rows, report


def test_the_review_ships_the_plain_version_when_it_wins():
    [row], report = review(Judge(favourite=PLAIN))
    assert row["plan"]["form"] == "straight"
    assert row["quality"]["verdict"] == "accept"
    assert row["quality"]["compared"]["kept"] == "plain"
    assert report["comparison_check"]["passed"]


def test_the_review_keeps_the_edit_when_it_wins():
    [row], _ = review(Judge(favourite=LIFTED))
    assert row["plan"]["form"] == "cold_open"
    assert row["quality"]["compared"]["reason"] == "edit_preferred"


def test_a_connected_agent_answers_through_the_book_and_is_measured_the_same_way():
    # A first run lists every question, known cases mixed in and unmarked.
    book = Book()
    review(Judge(first_always=True), book)
    listed = book.open()
    assert {q["kind"] for q in listed} == {"judge.compare"}
    assert all(q["question_id"] == question_id(q["kind"], q["text"]) for q in listed)
    # An agent answers them all as the lifted-line judge would.
    agent = Judge(favourite=LIFTED)
    answers = {q["question_id"]: agent.yes_probability(q["text"], 512)[0] for q in listed}
    local = Judge(first_always=True)
    book = Book(answers)
    [row], report = review(local, book)
    assert report["comparison_check"]["answered_by"] == "agent"
    assert report["comparison_check"]["passed"]
    assert row["plan"]["form"] == "cold_open"
    # The local model was never asked a comparison, and nothing is left open.
    assert not any("Version A" in q for q in local.asked)
    assert book.open() == []


def test_without_the_agents_answer_to_an_edit_the_plain_version_ships_meanwhile():
    book = Book()
    review(Judge(first_always=True), book)
    agent = Judge(favourite=LIFTED)
    # The agent answered the known pairs but not the two questions about the edit.
    answers = {
        q["question_id"]: agent.yes_probability(q["text"], 512)[0]
        for q in book.open()
        if LIFTED not in q["text"]
    }
    book = Book(answers)
    [row], report = review(Judge(first_always=True), book)
    assert report["comparison_check"]["answered_by"] == "agent"
    assert row["quality"]["compared"]["reason"] == "awaiting_agent"
    assert row["plan"]["form"] == "straight"
    assert [q["kind"] for q in book.open()] == ["judge.compare", "judge.compare"]


def test_copying_a_plan_does_not_change_its_plain_version():
    transcript = Transcript(script())
    edit = cold_open()
    assert plain_version(copy.deepcopy(edit), transcript) == plain_version(edit, transcript)


def test_the_review_trace_records_its_comparison_questions():
    from clipmill_worker_sdk.gen.schemas.editorial_trace import EditorialTrace

    runtime = Judge(favourite=LIFTED)
    critic = Critic(runtime)
    edit = cold_open()
    review_batch(
        critic,
        {
            "plans": [
                {
                    "candidate_id": edit["plan_id"],
                    "plan": edit,
                    "chapter": "Closing the shop",
                    "reasons": ["Shipping costs closed the site."],
                }
            ]
        },
        {
            "length": {"min_ticks": 5 * TICK, "max_ticks": 60 * TICK},
            "candidates": [{"plan": edit, "alternatives": [], "scout": "pool"}],
        },
        script(),
    )
    jobs = {call["job"] for call in critic.traces}
    assert "judge.compare" in jobs
    EditorialTrace.model_validate(
        {
            "schema_version": "clipmill.editorial.trace.v1",
            "content_profile": "interview",
            "source_fingerprint": SOURCE,
            "producer": {
                "stage": "editorial-critic",
                "implementation": "clipmill-worker-editorial@0.2.0/critic",
            },
            "calls": critic.traces,
        }
    )
