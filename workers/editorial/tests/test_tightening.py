"""Leaving a passage out by meaning: only what code allows, only past instruments that passed."""

from clipmill_worker_editorial.critique import review_batch
from clipmill_worker_editorial.planning import Transcript
from clipmill_worker_editorial.tightening import Gates, Lines, options, removal_cases
from clipmill_worker_sdk.gen.schemas.mind_quality_report import MindQualityReport

TICK = 90_000
SOURCE = "sha256:" + "a" * 64
LINES = [
    ("A", "Welcome back to the show."),
    ("A", "Why did you leave the company?"),
    ("B", "I left because the pay was flat."),
    ("B", "Um, yeah."),
    ("B", "The office was in an old warehouse by the river."),
    ("B", "The work stopped growing."),
    ("B", "My manager never noticed."),
    ("B", "That's why I quit."),
    ("A", "Was it hard?"),
    ("B", "It was."),
    ("B", "Uh, right."),
    ("B", "We tried to"),
    ("B", "We tried to fix it for a year."),
    ("A", "What happened next?"),
    ("B", "I started my own firm."),
    ("B", "Okay."),
    ("B", "It is doing well."),
]
ASIDE = {
    "first_unit": "s4",
    "last_unit": "s4",
    "start_ticks": (3 * 5 + 4) * TICK,
    "end_ticks": 5 * 5 * TICK,
}


def script():
    return {
        "units": [
            {
                "index": i,
                "text": text,
                "speaker": who,
                "start_ticks": i * 5 * TICK,
                "end_ticks": (i * 5 + 4) * TICK,
            }
            for i, (who, text) in enumerate(LINES)
        ]
    }


def answer_plan():
    start, end = 2 * 5 * TICK, (6 * 5 + 4) * TICK
    return {
        "plan_id": "plan_" + "c" * 24,
        "schema_version": "clipmill.mind.clip_plan.v1",
        "source_fingerprint": SOURCE,
        "title": "Why the pay ended it",
        "form": "straight",
        "parts": [
            {
                "part_id": "p1",
                "role": "body",
                "start_ticks": start,
                "end_ticks": end,
                "units": ["s2", "s3", "s4", "s5", "s6"],
            }
        ],
        "joins": [],
        "duration_ticks": end - start,
        "provenance": {"by": "mind"},
    }


def candidates(removable=(ASIDE,)):
    plan = answer_plan()
    return {
        "length": {"min_ticks": 5 * TICK, "max_ticks": 60 * TICK},
        "candidates": [{"plan": plan, "alternatives": [], "scout": "pool"}],
        "removable": list(removable),
    }


class Meter:
    """A model whose next line is easier to predict right after a question,
    or right after the line the speaker points back at."""

    has_meter = True

    def __init__(self, flat=False):
        self.flat = flat
        self.scored = 0
        self.asked = []

    def continuation_logprobs(self, context, _target):
        self.scored += 1
        last = context.splitlines()[-1]
        related = "?" in last or "never noticed" in last
        return [-1.0 if related and not self.flat else -2.0]

    def yes_probability(self, question, _think_tokens):
        self.asked.append(question)
        return 0.5, "Unsure."


class Editor:
    """A critic that approves, and a planner that leaves out the first passage."""

    def __init__(self, runtime, choice="p1"):
        self.runtime = runtime
        self.choice = choice
        self.identity = {"route": "local", "model": {"name": "test-model"}}
        self.traces = []
        self.brief = ""
        self.taste = []
        self.jobs = []

    def ask(self, job, data, _schema):
        self.jobs.append(job)
        if job == "tighten":
            assert [p["units"] for p in data["passages"]] == [["s4"]]
            return {"choice": self.choice, "reason": "The warehouse is an aside."}, None
        if job == "critic.edge":
            return {"edge_issues": []}, None
        if job == "critic.source":
            return {"effect": "preserved", "note": "The answer still follows the pay."}, None
        return {
            "comprehension": "pass",
            "landing": "pass",
            "title_accuracy": "pass",
            "hook": "strong",
            "summary": "The speaker explains why they left.",
            "issues": [],
        }, None

    def progress(self, *_args):
        pass


def review(editor, doc):
    report = {}
    rows = review_batch(
        editor,
        {
            "plans": [
                {
                    "candidate_id": answer_plan()["plan_id"],
                    "plan": answer_plan(),
                    "chapter": "Leaving",
                    "reasons": ["The speaker explains why they left."],
                }
            ]
        },
        doc,
        script(),
        report=report,
    )
    return rows, report


def test_known_removals_come_from_questions_pointers_filler_and_restarts():
    cases = {c["first_unit"]: c["expected"] for c in removal_cases(Lines(script()))}
    assert cases == {
        "s1": "dangling",
        "s6": "dangling",
        "s8": "dangling",
        "s13": "dangling",
        "s3": "intact",
        "s10": "intact",
        "s11": "intact",
        "s15": "intact",
    }


def test_a_meter_that_passes_lets_the_planner_leave_out_an_aside(monkeypatch):
    from clipmill_worker_editorial import critique

    made = []
    original = critique.plain_version

    def record(plan, transcript):
        made.append(plan)
        return original(plan, transcript)

    monkeypatch.setattr(critique, "plain_version", record)
    meter = Meter()
    editor = Editor(meter)
    [row], report = review(editor, candidates())
    plan, quality = row["plan"], row["quality"]
    tightened = made[-1]
    assert tightened["form"] == "tightened"
    assert tightened["parts"][0]["removed"] == [
        {"start_ticks": ASIDE["start_ticks"], "end_ticks": ASIDE["end_ticks"], "reason": "tangent"}
    ]
    assert tightened["parts"][0]["units"] == ["s2", "s3", "s5", "s6"]
    assert tightened["duration_ticks"] == answer_plan()["duration_ticks"] - 6 * TICK
    assert tightened["plan_id"] != answer_plan()["plan_id"]
    # This short exchange holds too few known pairs to measure a comparison,
    # so the shorter clip is not shipped on trust: its plain version is.
    assert quality["verdict"] == "accept"
    assert quality["compared"]["kept"] == "plain"
    assert quality["compared"]["reason"] == "not_measured"
    assert plan["form"] == "straight"
    assert plan["parts"][0]["start_ticks"] == answer_plan()["parts"][0]["start_ticks"]
    assert plan["parts"][0]["end_ticks"] == answer_plan()["parts"][0]["end_ticks"]
    assert report["removal_checks"]["meter"]["passed"]
    assert report["removal_checks"]["meter"]["separation"] == 1.0
    # The meter passed, so the slower judge was never measured.
    assert "judge" not in report["removal_checks"] and meter.asked == []
    MindQualityReport.model_validate(
        {
            "schema_version": "clipmill.mind.quality_report.v1",
            "source_fingerprint": SOURCE,
            "inputs": {},
            "reviews": [row],
            **report,
        }
    )


def test_with_no_instrument_passing_the_planner_is_never_asked():
    meter = Meter(flat=True)
    editor = Editor(meter)
    [row], report = review(editor, candidates())
    assert "tighten" not in editor.jobs
    assert row["plan"]["plan_id"] == answer_plan()["plan_id"]
    assert not report["removal_checks"]["meter"]["passed"]
    # The judge was tried next, and it stopped once it could not pass.
    assert not report["removal_checks"]["judge"]["passed"]
    assert len(meter.asked) < 8
    assert {t["job"] for t in editor.traces} == {"judge.removal"}


def test_the_planner_may_keep_everything_and_nothing_is_measured_without_passages():
    editor = Editor(Meter(), choice="none")
    [row], _ = review(editor, candidates())
    assert row["plan"]["plan_id"] == answer_plan()["plan_id"]

    meter = Meter()
    [row], report = review(Editor(meter), candidates(removable=()))
    assert meter.scored == 0 and report == {}


def test_only_passages_inside_the_clip_with_lines_on_both_sides_are_offered():
    transcript = Transcript(script())
    length = {"min_ticks": 5 * TICK, "max_ticks": 60 * TICK}
    first_line = {**ASIDE, "first_unit": "s2", "last_unit": "s2"}
    outside = {**ASIDE, "first_unit": "s9", "last_unit": "s9"}
    found, _ = options(answer_plan(), transcript, [ASIDE, first_line, outside], length)
    assert [p["first_unit"] for p in found] == ["s4"]
    # A clip that would fall under the shortest length keeps everything.
    found, _ = options(answer_plan(), transcript, [ASIDE], {**length, "min_ticks": 20 * TICK})
    assert found == []


def test_a_passing_meter_still_refuses_a_passage_the_next_line_relies_on():
    gates = Gates(Meter(), script(), {"route": "local"}, [])
    assert gates.ready()
    assert gates.allows(["B: I left."], ["B: The office was old."], "B: The work stopped.")
    assert not gates.allows(["B: I left."], ["B: My manager never noticed."], "B: That's why.")


def test_a_removal_question_shows_what_is_left_out():
    text = Gates.question(
        ["A: Why did you leave?", "B: The pay was flat."],
        ["B: Um, yeah."],
        "B: That's why I quit.",
    )
    assert "Left out:\nB: Um, yeah." in text
    assert "Kept lines before:\nA: Why did you leave?\nB: The pay was flat." in text
    assert "Kept line after:\nB: That's why I quit." in text


def test_a_bare_that_after_a_passage_is_no_known_case():
    source = script()
    # "That proves it." can point at the line before the passage as easily
    # as at the passage, so taking out s6 before it proves nothing.
    source["units"][7]["text"] = "That proves it."
    cases = {c["first_unit"]: c["expected"] for c in removal_cases(Lines(source))}
    assert "s6" not in cases
