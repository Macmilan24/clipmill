"""Behavior at the edit boundaries a span-based reviewer would never see."""

from clipmill_worker_editorial.critique import join_note, judge, review_batch
from clipmill_worker_editorial.planning import Transcript, covers_named_entity, plan_one, variants
from clipmill_worker_editorial.rank_plans import rank_batch, strengths

TICK = 90_000
SOURCE = "sha256:" + "a" * 64


class Calls:
    def __init__(self, answer):
        self.identity = {"model": {"name": "test-model"}}
        self.brief = "pricing"
        self.taste = ["Kept an example with a concrete price."]
        self.answer = answer
        self.calls = []

    def ask(self, job, data, _schema):
        self.calls.append((job, data))
        return self.answer(job, data), None

    def progress(self, *_args):
        pass


def script():
    lines = [
        ("A", "Did the service always cost five dollars?"),
        ("B", "It cost five dollars in March."),
        ("B", "But only for the first month."),
        ("B", "The normal price was fifteen."),
        ("A", "Was the new package profitable?"),
        ("B", "Yes, after the first quarter."),
    ]
    return {
        "units": [
            {
                "index": i,
                "text": text,
                "speaker": who,
                "start_ticks": i * 5 * TICK,
                "end_ticks": (i + 1) * 5 * TICK,
            }
            for i, (who, text) in enumerate(lines)
        ]
    }


def part(id_, first, end):
    return {
        "part_id": id_,
        "role": "body",
        "start_ticks": first * 5 * TICK,
        "end_ticks": end * 5 * TICK,
    }


def plan(parts, form="straight", name="1"):
    return {
        "plan_id": "plan_" + name * 16,
        "source_fingerprint": SOURCE,
        "schema_version": "clipmill.mind.clip_plan.v1",
        "title": "The price in March",
        "form": form,
        "parts": parts,
        "joins": [{"after": p["part_id"], "kind": "distant"} for p in parts[:-1]],
        "duration_ticks": sum(
            p["end_ticks"]
            - p["start_ticks"]
            - sum(r["end_ticks"] - r["start_ticks"] for r in p.get("removed", []))
            for p in parts
        ),
        "provenance": {"by": "mind"},
    }


def approval():
    return {
        "comprehension": "pass",
        "landing": "pass",
        "title_accuracy": "pass",
        "hook": "strong",
        "summary": "The speaker gives the price and its condition.",
        "issues": [],
    }


def source_approval():
    return {"edge_issues": []}


def test_wrong_question_is_checked_against_original_question_without_planner_reasons():
    draft = plan([part("p1", 0, 1), part("p2", 5, 6)], "stitch")
    draft["parts"][0]["why"] = "Private planner justification must never reach the critic."
    draft["evidence"] = ["An authoritative but incorrect claim that these lines belong together"]

    def answer(job, evidence):
        if job == "critic":
            assert set(evidence) == {"public_title", "playback"}
            assert "profitable" not in str(evidence["playback"])
            return approval()
        assert "public_title" not in evidence
        if evidence["check"] == "outer_edges":
            return source_approval()
        assert "profitable" in str(evidence["original"]) and "profitable" not in str(
            evidence["edited"]
        )
        assert "five dollars" in evidence["edited"][0]["text"]
        assert "profitable" in evidence["question_before_reply_in_source"]
        assert "Private planner" not in str(evidence) and "authoritative" not in str(evidence)
        return {
            "effect": "different_answer",
            "note": "The answer belonged to a different question.",
        }

    result = judge(Calls(answer), draft, Transcript(script()))
    assert result["verdict"] == "reject" and "different question" in result["issues"][0]


def test_removed_qualification_can_be_restored_once_and_independently_rechecked():
    piece = part("p1", 1, 4)
    piece["removed"] = [{"start_ticks": 10 * TICK, "end_ticks": 15 * TICK, "reason": "tangent"}]
    draft = plan([piece], "tightened")

    def answer(job, data):
        if job == "critic":
            return approval()
        if job == "plan":
            assert "qualification" in str(data["repair_findings"])
            return {
                "choice": "part_1_uncut",
                "title": "The introductory price",
                "hook_text": "INVENTED",
                "reasons": ["Includes the price condition."],
                "doubts": [],
                "joins": [],
            }
        if data["check"] == "one_join":
            assert data["after"] == "p1.cut1" and "first month" in str(data["original"])
            assert "first month" not in str(data["edited"])
            return {
                "effect": "lost_qualification",
                "omitted_condition": "But only for the first month.",
                "note": "A price qualification was removed.",
            }
        return source_approval()

    ctx = Calls(answer)
    rows = review_batch(
        ctx,
        {
            "plans": [
                {"candidate_id": draft["plan_id"], "plan": draft, "chapter": "Price", "reasons": []}
            ]
        },
        {
            "length": {"min_ticks": 5 * TICK, "max_ticks": 20 * TICK},
            "candidates": [{"plan": draft}],
        },
        script(),
    )
    assert [name for name, _ in ctx.calls] == [
        "critic",
        "critic.edge",
        "critic.source",
        "plan",
        "critic",
        "critic.edge",
    ]
    assert rows[0]["quality"]["repaired"] and rows[0]["quality"]["verdict"] == "accept"
    assert rows[0]["plan"]["hook_text"] == "It cost five dollars in March."
    assert rows[0]["plan"]["plan_id"] != draft["plan_id"]


def test_missing_boundary_fails_closed_and_no_rejected_plan_is_ranked():
    draft = plan([part("p1", 0, 1), part("p2", 5, 6)], "stitch")
    ctx = Calls(lambda job, _: source_approval() if job.startswith("critic.") else approval())
    quality = judge(ctx, draft, Transcript(script()))
    assert quality["verdict"] == "reject"
    result = rank_batch(ctx, {"reviews": [{"plan": draft, "quality": quality}]}, script(), 4)
    assert result["clips"] == [] and result["cutoff_reason"]


def test_consecutive_spoken_lines_get_the_identical_original_and_edited_text_at_the_join():
    draft = plan([part("p1", 0, 2), part("p2", 2, 4)], "tightened")
    draft["joins"][0]["kind"] = "adjacent"

    def answer(job, data):
        if job == "critic":
            return approval()
        if data["check"] == "outer_edges":
            return source_approval()
        assert data["source_order"] == "consecutive" and data["original"] == data["edited"]
        assert "profitable" not in str(data)
        return {
            "effect": "preserved",
            "note": "The original price and its qualification remain together.",
        }

    result = judge(Calls(answer), draft, Transcript(script()))
    assert result["verdict"] == "accept" and result["joins"][0]["after"] == "p1"


def test_an_independent_omission_passes_and_an_unsubstantiated_condition_stays_unverified():
    source = script()
    for unit, text in zip(
        source["units"],
        ["Tickets cost ten dollars.", "I grew up by the river.", "The show starts at nine."],
        strict=False,
    ):
        unit["text"] = text
    draft = plan([part("p1", 0, 1), part("p2", 2, 3)], "stitch")
    vote = {
        "effect": "omitted_independent_material",
        "note": "The personal anecdote never changes the ticket price or start time.",
    }

    def answer(job, data):
        if job == "critic":
            return approval()
        if data["check"] == "outer_edges":
            return source_approval()
        assert "river" in str(data["original"]) and "river" not in str(data["edited"])
        return vote

    assert judge(Calls(answer), draft, Transcript(source))["verdict"] == "accept"
    vote.update(effect="lost_qualification", omitted_condition="except on weekends")
    quality = judge(Calls(answer), draft, Transcript(source))
    assert quality["verdict"] == "reject" and "could not identify" in quality["joins"][0]["note"]


def test_source_audit_can_veto_an_intelligible_unspliced_cut_missing_its_qualification():
    draft = plan([part("p1", 0, 2)])

    def answer(job, data):
        if job == "critic":
            assert "first month" not in str(data)
            return approval()
        assert data["check"] == "outer_edges"
        assert "first month" in data["recorded_after_ending"][0]["text"]
        return {
            "edge_issues": [{"aspect": "meaning", "detail": "The original price was temporary."}],
        }

    result = judge(Calls(answer), draft, Transcript(script()))
    assert result["verdict"] == "reject" and "The original price was temporary." in result["issues"]


def test_a_model_note_cut_off_at_the_grammar_limit_has_a_complete_public_explanation():
    value = {
        "effect": "lost_qualification",
        "omitted_condition": "only for the first month",
        "note": "An unfinished sentence " * 6 + "at",
    }
    assert join_note(value) == "The cut omits the condition “only for the first month”."


def test_a_misleading_title_finding_requires_one_retitle_even_if_the_checkbox_passed():
    draft = plan([part("p1", 1, 4)])

    def answer(job, data):
        if job.startswith("critic."):
            return source_approval()
        if job == "plan":
            assert any(choice["choice"] == "retitle" for choice in data["choices"])
            return {
                "choice": "retitle",
                "title": "An introductory price",
                "hook_text": "",
                "reasons": ["The speaker clarifies the first-month price."],
                "doubts": [],
                "joins": [],
            }
        result = approval()
        if data["public_title"] == draft["title"]:
            result["issues"] = [
                {
                    "aspect": "title_accuracy",
                    "detail": "The title omits the introductory condition.",
                }
            ]
        return result

    ctx = Calls(answer)
    (row,) = review_batch(
        ctx,
        {
            "plans": [
                {
                    "candidate_id": draft["plan_id"],
                    "plan": draft,
                    "chapter": "Pricing",
                    "reasons": [],
                }
            ]
        },
        {"length": {"min_ticks": TICK, "max_ticks": 60 * TICK}, "candidates": [{"plan": draft}]},
        script(),
    )
    assert [job for job, _ in ctx.calls] == [
        "critic",
        "critic.edge",
        "plan",
        "critic",
        "critic.edge",
    ]
    assert row["quality"]["verdict"] == "accept" and row["quality"]["repaired"]
    assert row["plan"]["plan_id"] == draft["plan_id"]


def test_position_only_votes_cancel_and_stronger_wins_raise_bradley_terry_strength():
    drafts = [plan([part("p1", 0, 2)], name="1"), plan([part("p1", 3, 5)], name="2")]
    ctx = Calls(lambda *_: {"winner": "A", "reason": "First one."})
    rows = [
        {
            "plan": draft,
            "chapter": str(i),
            "reasons": ["Specific."],
            "quality": {"verdict": "accept", "hook": "strong", "summary": "Complete."},
        }
        for i, draft in enumerate(drafts)
    ]
    ranked = rank_batch(ctx, {"reviews": rows}, script(), 2)
    assert len(ctx.calls) == 2
    assert ctx.calls[0][1]["A"] == ctx.calls[1][1]["B"]
    assert ranked["comparisons"][0]["share_a"] == 0.5
    assert ranked["clips"][0]["strength"] == ranked["clips"][1]["strength"]
    a, b, c = strengths(3, [(0, 1, 1), (1, 2, 1), (0, 2, 1)])
    assert a > b > c > 0


def test_planner_cannot_invent_source_ranges_and_declines_unsupplied_choice():
    draft = plan([part("p1", 1, 4)])
    ctx = Calls(lambda *_: {"choice": "fabricated", "title": "New range"})
    transcript = Transcript(script())
    assert (
        plan_one(
            ctx, variants(draft, transcript, {"min_ticks": 1, "max_ticks": 20 * TICK}), transcript
        )
        is None
    )
    assert ctx.calls[0][1]["previous_decisions"] == ctx.taste


def test_trailing_subscription_promotion_ends_at_the_preceding_spoken_conclusion():
    source = script()
    source["units"][-2]["text"] = "Fix the feedback loop first, then scale the output."
    source["units"][-1]["text"] = "Subscribe for part two."
    original = plan([part("p1", 0, 6)])
    transcript = Transcript(source)
    length = {"min_ticks": 10 * TICK, "max_ticks": 60 * TICK}
    choices = variants(original, transcript, length)
    clean = choices["original_ending"]
    assert clean["parts"][-1]["end_ticks"] == source["units"][-2]["end_ticks"]
    assert clean["plan_id"] != original["plan_id"]
    ctx = Calls(lambda job, _: source_approval() if job.startswith("critic.") else approval())
    assert judge(ctx, original, transcript)["verdict"] == "reject"
    assert judge(ctx, clean, transcript)["verdict"] == "accept"
    ctx.brief = "End on the subscription call to action"
    assert "original_ending" not in variants(original, transcript, length, brief=ctx.brief)
    assert judge(ctx, original, transcript)["verdict"] == "accept"
    source["units"][0]["text"] = "Here is how listeners can subscribe to the programme."
    ctx.brief = ""
    assert judge(ctx, original, Transcript(source))["verdict"] == "accept"
    source["units"][0]["text"] = "We can monitor the service's uptime."
    source["units"][-1]["text"] = "Subscribe to the alerts to receive outage notices."
    assert judge(ctx, original, Transcript(source))["verdict"] == "accept"


def test_explicit_montage_brief_cannot_be_fulfilled_by_a_single_span():
    original = plan([part("p1", 0, 6)])
    choices = variants(
        original,
        Transcript(script()),
        {"min_ticks": TICK, "max_ticks": 60 * TICK},
        brief="Make a montage of every time they talk about pricing",
    )
    assert not choices


def test_an_exhaustive_entity_montage_must_include_all_its_known_recording_events():
    entity = {
        "entity_id": "e1",
        "name": "Northwind",
        "aliases": ["NW"],
        "events": ["m1", "m3", "m8"],
    }
    assert not covers_named_entity(
        {"evidence": ["e1", "m1", "m3"]}, [entity], "Every time they mention NW"
    )
    complete = {"evidence": ["e1", "m1", "m3", "m8"]}
    assert covers_named_entity(complete, [entity], "Every time they mention NW")
    assert not covers_named_entity(
        complete, [entity], "Every time they mention a different company"
    )


def test_a_separate_promotion_part_can_be_removed_at_its_join():
    source = script()
    source["units"][-1]["text"] = "Subscribe for part two."
    draft = plan([part("p1", 0, 4), part("p2", 5, 6)], "stitch")
    result = variants(draft, Transcript(source), {"min_ticks": 10 * TICK, "max_ticks": 60 * TICK})[
        "original_ending"
    ]
    assert result["parts"] == [draft["parts"][0]]
    assert result["form"] == "straight" and result["joins"] == []
    assert result["duration_ticks"] == source["units"][3]["end_ticks"]


def test_opening_quote_rejoins_asr_fragments_from_the_same_spoken_sentence_only():
    source = script()
    source["units"][0].update(text="A large model", speaker="B")
    source["units"][1]["text"] = "needs memory for its weights."
    full = Transcript(source).opening_quote(plan([part("p1", 0, 4)]))
    assert full == "A large model needs memory for its weights."
    clipped = Transcript(source).opening_quote(plan([part("p1", 0, 1), part("p2", 5, 6)], "stitch"))
    assert clipped == "A large model"
