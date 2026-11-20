"""The ending judge: measured on known cases first, trusted only when it tells them apart."""

from clipmill_worker_editorial.judging import Recording, best_separation, judge, known_endings
from clipmill_worker_editorial.planning import Transcript, cut_short, source_options, variants
from clipmill_worker_sdk.gen.schemas.mind_judgments import MindJudgments

TICK = 90_000
SOURCE = "sha256:" + "a" * 64
LINES = [
    ("A", "What changed when you moved the team?"),
    ("B", "We stopped shipping on Fridays."),
    ("B", "And the incidents dropped by half."),
    ("A", "Did anyone push back?"),
    ("B", "Not yet."),
    ("B", "The sales team will when the quarter ends."),
    ("B", "But that is a problem for later."),
    ("A", "How did you measure it?"),
    ("B", "We counted pages per week."),
    ("B", "Because pages are what wake people up."),
    ("A", "What would you do differently?"),
    ("B", "Start sooner."),
]


def script():
    # Four seconds of speech and a one-second pause per line.
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


MEMORY = {
    "chapters": [
        {
            "chapter_id": "c1",
            "title": "Moving the team",
            "summary": "What changed.",
            "first_unit": "s0",
            "last_unit": "s11",
            "start_ticks": 0,
            "end_ticks": 60 * TICK,
        }
    ]
}


def plan(first, last, name):
    start, end = first * 5 * TICK, (last * 5 + 4) * TICK
    return {
        "plan_id": "plan_" + name * 24,
        "schema_version": "clipmill.mind.clip_plan.v1",
        "source_fingerprint": SOURCE,
        "title": "Moving the team",
        "form": "straight",
        "parts": [
            {
                "part_id": "p1",
                "role": "body",
                "start_ticks": start,
                "end_ticks": end,
                "units": [f"s{i}" for i in range(first, last + 1)],
            }
        ],
        "joins": [],
        "duration_ticks": end - start,
        "provenance": {"by": "mind"},
    }


def candidates(*rows):
    return {
        "length": {"min_ticks": TICK, "max_ticks": 60 * TICK},
        "candidates": [
            {"plan": first, "alternatives": list(rest), "scout": "pool"} for first, *rest in rows
        ],
    }


class Model:
    """Answers like a model that hears a connective as more to come."""

    def __init__(self, answer=None):
        self.asked = []
        self.answer = answer

    def yes_probability(self, question, think_tokens):
        assert think_tokens == 512
        self.asked.append(question)
        if self.answer is not None:
            return self.answer, "Thinking."
        clip, following = question.split("Next line:\n")
        last = clip.strip().splitlines()[-1]
        nxt = following.splitlines()[0]
        if nxt.split(": ", 1)[1].split()[0] in ("And", "But", "Because"):
            return 0.15, "The next line carries the thought on."
        if last.endswith("Not yet."):
            return 0.2, "The answer is held for the next line."
        if nxt[0] != last[0]:
            return 0.85, "The other speaker takes a turn."
        return 0.7, "It stands alone."


def run(model, doc):
    traces = []
    result = judge(
        model,
        script(),
        doc,
        MEMORY,
        brief="",
        count=10,
        identity={"route": "local", "model": {"name": "test-model"}},
        traces=traces,
        progress=lambda *_: None,
    )
    MindJudgments.model_validate(
        {
            "schema_version": "clipmill.mind.judgments.v1",
            "source_fingerprint": SOURCE,
            "inputs": {},
            **result,
        }
    )
    return result, traces


def test_known_endings_come_from_connectives_and_answered_turns():
    known = known_endings(Recording(script()), MEMORY["chapters"])
    # Five of the seven turns, spread through the recording.
    assert known == [
        (1, "continues"),
        (5, "continues"),
        (8, "continues"),
        (0, "finished"),
        (3, "finished"),
        (6, "finished"),
        (7, "finished"),
        (10, "finished"),
    ]


def test_a_model_that_passes_its_check_judges_each_offered_ending_once():
    model = Model()
    doc = candidates((plan(3, 4, "a"), plan(3, 6, "b")), (plan(3, 5, "c"),))
    result, traces = run(model, doc)
    check = result["self_check"]["endings"]
    assert check["passed"] and check["separation"] == 1.0
    assert result["endings"] == [
        {"after_unit": "s4", "score": -0.6, "verdict": "continues"},
        {"after_unit": "s5", "score": -0.7, "verdict": "continues"},
    ]
    # Eight known cases, then s4: s5 was already measured as a known case,
    # and s6 hands over to the other speaker, which needs no judge.
    assert len(model.asked) == 9
    assert [t["outcome"] for t in traces] == ["answered"] * 9
    assert {t["job"] for t in traces} == {"judge.ending"}


def test_the_planner_is_never_offered_an_ending_the_judge_heard_as_cut_off():
    doc = candidates((plan(3, 4, "a"), plan(3, 6, "b")))
    result, _ = run(Model(), doc)
    options = source_options(doc["candidates"][0], Transcript(script()), doc, "", cut_short(result))
    assert [p["plan_id"] for p in options.values()] == ["plan_" + "b" * 24]
    assert cut_short(None) == set()


def test_a_model_that_cannot_tell_the_known_cases_apart_decides_nothing():
    model = Model(answer=0.5)
    result, _ = run(model, candidates((plan(3, 4, "a"),)))
    check = result["self_check"]["endings"]
    assert not check["passed"] and check["separation"] == 0.5
    assert result["endings"] == []
    # Three of each kind already put the bar out of reach; the rest go unasked.
    assert len(model.asked) == 6
    assert len(check["cases"]) == 6


def test_with_no_ending_to_judge_the_model_is_not_asked_at_all():
    model = Model()
    result, _ = run(model, candidates((plan(3, 6, "b"),)))
    assert model.asked == []
    assert result["self_check"]["endings"] == {"passed": False, "cases": []}


def test_a_monologue_checks_the_judge_on_chapter_closes_instead_of_turns():
    source = script()
    for unit in source["units"]:
        unit["speaker"] = "B"
    chapters = [
        {**MEMORY["chapters"][0], "last_unit": f"s{i}", "end_ticks": (i * 5 + 4) * TICK}
        for i in (2, 6, 9)
    ]
    known = known_endings(Recording(source), chapters)
    assert [k for k in known if k[1] == "finished"] == [
        (2, "finished"),
        (6, "finished"),
        (9, "finished"),
    ]


def test_a_tightened_ending_lands_on_the_cut_point_the_cutter_measured():
    source = script()
    source["units"][11]["text"] = "Subscribe for part two."
    original = plan(7, 11, "d")
    transcript = Transcript(source)
    length = {"min_ticks": TICK, "max_ticks": 60 * TICK}
    pause = (9 * 5 + 4) * TICK + 30_000
    choices = variants(original, transcript, length, legal_ends={"s10": 50 * TICK, "s9": pause})
    # s10 is the last line before the sign-off; its cut point sits in the
    # pause after it, not at the last word's end.
    assert choices["original_ending"]["parts"][-1]["end_ticks"] == 50 * TICK
    assert "original_ending" not in variants(original, transcript, length, legal_ends={})
    assert (
        variants(original, transcript, length)["original_ending"]["parts"][-1]["end_ticks"]
        == source["units"][10]["end_ticks"]
    )


def test_in_a_quick_exchange_only_a_turn_to_a_question_counts_as_finished():
    source = script()
    for unit in source["units"]:
        unit["end_ticks"] = unit["start_ticks"] + 5 * TICK
    known = known_endings(Recording(source))
    # s2, s6 and s9 hand over to a question; s10's answer comes with no pause.
    assert [k for k in known if k[1] == "finished"] == [
        (2, "finished"),
        (6, "finished"),
        (9, "finished"),
    ]
    source["units"][3]["text"] = "Nobody pushed back."
    known = known_endings(Recording(source))
    assert (2, "finished") not in known


def test_a_model_that_sounds_unsure_is_judged_by_its_own_scale():
    class Hesitant(Model):
        """Orders the cases right but never says yes with any confidence."""

        def yes_probability(self, question, think_tokens):
            probability, thinking = super().yes_probability(question, think_tokens)
            return {0.15: 0.1, 0.85: 0.25, 0.2: 0.16, 0.7: 0.2}[probability], thinking

    result, _ = run(Hesitant(), candidates((plan(3, 4, "a"), plan(3, 6, "b"))))
    assert result["self_check"]["endings"]["passed"]
    # The line sits midway between its medians, 0.1 and 0.25: 0.16 is below.
    assert result["endings"] == [{"after_unit": "s4", "score": -0.68, "verdict": "continues"}]


def test_the_check_can_tell_when_its_bar_is_out_of_reach():
    # Nothing asked yet: anything is possible.
    assert best_separation([], [], (5, 5)) == 1.0
    # One finished case scored below one continuing case costs one pair in 25.
    assert best_separation([0.1], [0.3], (5, 5)) == 0.96
    assert best_separation([0.1, 0.1, 0.1], [0.3, 0.3], (5, 5)) == 0.76


class Review:
    """A critic that approves everything, and a planner that takes the first choice."""

    def __init__(self, runtime):
        self.runtime = runtime
        self.identity = {"route": "local", "model": {"name": "test-model"}}
        self.traces = []
        self.brief = ""
        self.taste = []

    def ask(self, job, data, _schema):
        if job == "plan":
            first = data["choices"][0]["choice"]
            return {
                "choice": first,
                "title": "Moving the team",
                "hook_text": "",
                "reasons": ["A complete answer."],
                "doubts": [],
                "joins": [],
            }, None
        if job.startswith("critic."):
            return {"edge_issues": []}, None
        return {
            "comprehension": "pass",
            "landing": "pass",
            "title_accuracy": "pass",
            "hook": "strong",
            "summary": "The speaker explains what changed.",
            "issues": [],
        }, None

    def progress(self, *_args):
        pass


def passing_judgments(endings=()):
    return {
        "self_check": {
            "endings": {
                "passed": True,
                "separation": 1.0,
                "cases": [
                    {"unit": f"s{i}", "expected": kind, "score": score}
                    for i, kind, score in [
                        (1, "continues", -0.7),
                        (5, "continues", -0.7),
                        (8, "continues", -0.7),
                        (2, "finished", 0.7),
                        (6, "finished", 0.7),
                        (9, "finished", 0.7),
                    ]
                ],
            },
            "removals": {"passed": False, "cases": []},
        },
        "endings": list(endings),
    }


def test_the_review_asks_about_a_final_ending_and_repairs_one_that_cuts_off():
    from clipmill_worker_editorial.critique import CUT_OFF, review_batch

    model = Model()
    ctx = Review(model)
    first, later = plan(3, 4, "a"), plan(3, 6, "b")
    doc = candidates((first, later))
    plans = {
        "plans": [
            {
                "candidate_id": first["plan_id"],
                "plan": first,
                "chapter": "Moving the team",
                "reasons": ["A complete answer."],
            }
        ]
    }
    [row] = review_batch(ctx, plans, doc, script(), judgments=passing_judgments())
    # "Not yet." was never judged before planning: the review asks once.
    assert len(model.asked) == 1 and "Not yet." in model.asked[0]
    assert row["quality"]["verdict"] == "accept" and row["quality"]["repaired"]
    assert row["plan"]["parts"][-1]["units"][-1] == "s6"
    assert {t["job"] for t in ctx.traces} == {"judge.ending"}

    # A judge that did not pass its check is never asked, and nothing changes.
    model = Model()
    unchecked = passing_judgments()
    unchecked["self_check"]["endings"]["passed"] = False
    [row] = review_batch(Review(model), plans, doc, script(), judgments=unchecked)
    assert model.asked == [] and row["plan"]["plan_id"] == first["plan_id"]
    assert CUT_OFF not in row["quality"]["issues"]
