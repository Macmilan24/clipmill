"""What an annotation has to be before a number is computed from it.

An annotation is the only input to this project that nothing derives — it is a
person's opinion, and every recall figure is measured against it. A broken one
produces a number rather than an error, and a number computed from a broken
annotation is worse than no number because somebody will quote it.
"""

from __future__ import annotations

import json

import pytest
from clipmill_eval.annotation import (
    AnnotationError,
    load_annotation,
    load_annotations,
    parse_annotation,
)
from clipmill_eval.scoring import ScoringError, clips_from_ranking, meets_bar
from clipmill_eval.worksheet import Line, render_transcript, sentences_from_index, skeleton

FINGERPRINT = "sha256:" + "a" * 64


def document(**overrides: object) -> dict:
    base = {
        "schema_version": "clipmill.eval_annotation.v1",
        "source_fingerprint": FINGERPRINT,
        "annotator_id": "sami",
        "annotated_unix_millis": 1_700_000_000_000,
        "timebase": {"num": 1, "den": 90_000},
        "moments": [
            {
                "moment_id": "m1",
                "span": {"start_ticks": 0, "end_ticks": 900_000},
                "importance": "essential",
            }
        ],
        "exclusions": [],
    }
    base.update(overrides)
    return base


class TestParsing:
    def test_a_well_formed_document_comes_back_whole(self) -> None:
        parsed = parse_annotation(document())
        assert parsed.annotator_id == "sami"
        assert parsed.moments[0].importance == "essential"
        assert parsed.moments[0].acceptable_spans() == (parsed.moments[0].span,)

    def test_a_document_from_another_format_is_refused(self) -> None:
        with pytest.raises(AnnotationError, match="schema_version"):
            parse_annotation(document(schema_version="clipmill.eval_annotation.v9"))

    def test_a_span_that_runs_backwards_is_refused(self) -> None:
        broken = document()
        broken["moments"][0]["span"] = {"start_ticks": 900_000, "end_ticks": 0}
        with pytest.raises(AnnotationError, match="end after it starts"):
            parse_annotation(broken)

    def test_an_empty_span_is_refused_rather_than_scored_as_zero_overlap(self) -> None:
        broken = document()
        broken["moments"][0]["span"] = {"start_ticks": 900_000, "end_ticks": 900_000}
        with pytest.raises(AnnotationError):
            parse_annotation(broken)

    def test_an_importance_nobody_defined_is_refused(self) -> None:
        broken = document()
        broken["moments"][0]["importance"] = "very important"
        with pytest.raises(AnnotationError, match="importance"):
            parse_annotation(broken)

    def test_two_moments_cannot_share_an_id(self) -> None:
        broken = document()
        broken["moments"].append(dict(broken["moments"][0]))
        with pytest.raises(AnnotationError, match="duplicate moment_id"):
            parse_annotation(broken)

    def test_a_hook_outside_its_moment_is_a_slip_and_is_refused(self) -> None:
        broken = document()
        broken["moments"][0]["hook_ticks"] = 5_000_000
        with pytest.raises(AnnotationError, match="hook_ticks"):
            parse_annotation(broken)

    def test_an_exclusion_needs_a_reason_from_the_vocabulary(self) -> None:
        broken = document(
            exclusions=[{"span": {"start_ticks": 0, "end_ticks": 10}, "reason": "did not like it"}]
        )
        with pytest.raises(AnnotationError, match="exclusion reason"):
            parse_annotation(broken)

    def test_a_boolean_is_not_an_integer_here(self) -> None:
        # True is an int in Python and would sail through as 1.
        broken = document(annotated_unix_millis=True)
        with pytest.raises(AnnotationError):
            parse_annotation(broken)

    def test_no_moments_is_a_real_answer_rather_than_an_error(self) -> None:
        parsed = parse_annotation(document(moments=[]))
        assert parsed.moments == ()


class TestLoading:
    def test_annotations_load_in_a_stable_order(self, tmp_path) -> None:
        for name in ("zebra", "alpha", "middle"):
            payload = document(item_id=name)
            (tmp_path / f"{name}.json").write_text(json.dumps(payload), encoding="utf-8")
        loaded = load_annotations(tmp_path)
        assert [annotation.item_id for annotation in loaded] == ["alpha", "middle", "zebra"]

    def test_an_empty_directory_is_an_error_rather_than_a_perfect_score(self, tmp_path) -> None:
        with pytest.raises(AnnotationError, match="holds no annotations"):
            load_annotations(tmp_path)

    def test_the_failing_file_is_named(self, tmp_path) -> None:
        path = tmp_path / "broken.json"
        path.write_text("{not json", encoding="utf-8")
        with pytest.raises(AnnotationError, match=r"broken\.json"):
            load_annotation(path)


class TestScoringAdapter:
    def ranking(self, **overrides: object) -> dict:
        base = {
            "cohort": [
                {
                    "candidate_id": "cand_1",
                    "boundary": {"chosen": {"start_ticks": 0, "end_ticks": 900_000}},
                }
            ],
            "selected": ["cand_1"],
        }
        base.update(overrides)
        return base

    def test_the_chosen_boundary_is_what_gets_scored(self) -> None:
        clips = clips_from_ranking(self.ranking())
        assert clips[0].span.end_ticks == 900_000

    def test_selecting_a_clip_outside_the_cohort_is_refused(self) -> None:
        with pytest.raises(ScoringError, match="not in the cohort"):
            clips_from_ranking(self.ranking(selected=["cand_missing"]))

    def test_a_candidate_with_no_chosen_boundary_is_refused(self) -> None:
        broken = self.ranking()
        broken["cohort"][0]["boundary"] = {}
        with pytest.raises(ScoringError, match="no chosen boundary"):
            clips_from_ranking(broken)


class TestBar:
    def report(self, **overrides: object) -> dict:
        base = {
            "recall": 1.0,
            "moments_recalled": 3,
            "moments_total": 3,
            "recall_by_importance": {"essential": 1.0},
            "multi_moment_recall": 1.0,
            "duplicate_rate": 0.2,
            "excluded_offered": 0,
            "boundary_edge_error_millis": {"median": 0, "p90": 0, "measured": 3},
        }
        base.update(overrides)
        return base

    def test_a_report_that_meets_everything_has_no_failures(self) -> None:
        assert meets_bar(self.report(), {"min_recall": 1.0, "max_duplicate_rate": 0.6}) == []

    def test_a_bar_names_only_what_it_constrains(self) -> None:
        # An empty bar is not a bar that passes everything at zero — it is a
        # bar that makes no claim, which is the state before a baseline exists.
        assert meets_bar(self.report(recall=0.0), {}) == []

    def test_each_shortfall_is_reported_separately(self) -> None:
        failures = meets_bar(
            self.report(recall=0.5, duplicate_rate=0.9),
            {"min_recall": 0.8, "max_duplicate_rate": 0.6},
        )
        assert len(failures) == 2
        assert any("recall" in failure for failure in failures)
        assert any("duplicate" in failure for failure in failures)

    def test_an_excluded_span_offered_fails_whatever_the_bar_says(self) -> None:
        failures = meets_bar(self.report(excluded_offered=1), {})
        assert failures == ["1 excluded spans were offered"]

    def test_a_grade_the_corpus_lacks_fails_rather_than_passing_vacuously(self) -> None:
        failures = meets_bar(
            self.report(recall_by_importance={"acceptable": 1.0}),
            {"min_recall_by_importance": {"essential": 1.0}},
        )
        assert failures == ["the bar names essential moments and the corpus holds none"]


class TestWorksheet:
    def index(self) -> dict:
        return {
            "sentences": [
                {"start_ticks": 0, "end_ticks": 90_000, "text": "First line."},
                {"start_ticks": 90_000, "end_ticks": 180_000, "text": "Second line."},
            ]
        }

    def test_sentences_come_through_with_their_ticks(self) -> None:
        lines = sentences_from_index(self.index())
        assert [line.text for line in lines] == ["First line.", "Second line."]
        assert lines[1].start_ticks == 90_000

    def test_an_index_with_no_sentences_is_an_error(self) -> None:
        with pytest.raises(ValueError, match="no sentences"):
            sentences_from_index({"sentences": []})

    def test_a_pipe_in_the_transcript_does_not_break_the_table(self) -> None:
        rendered = render_transcript("item", [Line(0, 90_000, "a | b")])
        # One escaped pipe in the text, and the row still has the columns the
        # header promised.
        row = next(line for line in rendered.splitlines() if "a " in line)
        assert r"a \| b" in row

    def test_the_skeleton_is_a_document_the_parser_accepts(self) -> None:
        parsed = parse_annotation(skeleton(FINGERPRINT, "sami", 0, "item"))
        assert parsed.moments == ()
        assert parsed.item_id == "item"
