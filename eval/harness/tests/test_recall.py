"""The metrics, against numbers worked out by hand.

Every expectation here is arithmetic somebody can check on paper. That matters
more than usual: these are the numbers the project will quote about itself, and
a metric that is merely self-consistent would pass a test written against its
own output while reporting something nobody meant.
"""

from __future__ import annotations

import pytest
from clipmill_eval.annotation import Annotation, Exclusion, Interval, Moment
from clipmill_eval.recall import (
    Clip,
    duplicate_rate,
    edge_error,
    iou,
    match_moments,
    score_corpus,
    score_item,
)

SECOND = 90_000


def span(start_seconds: float, end_seconds: float) -> Interval:
    return Interval(int(start_seconds * SECOND), int(end_seconds * SECOND))


def moment(
    moment_id: str,
    start: float,
    end: float,
    importance: str = "strong",
    alternatives: tuple[Interval, ...] = (),
) -> Moment:
    return Moment(
        moment_id=moment_id,
        span=span(start, end),
        importance=importance,
        alternatives=alternatives,
    )


def annotation(*moments: Moment, exclusions: tuple[Exclusion, ...] = ()) -> Annotation:
    return Annotation(
        source_fingerprint="sha256:" + "1" * 64,
        annotator_id="sami",
        annotated_unix_millis=0,
        timebase_num=1,
        timebase_den=SECOND,
        moments=moments,
        exclusions=exclusions,
        item_id="item",
    )


def clip(candidate_id: str, start: float, end: float) -> Clip:
    return Clip(candidate_id=candidate_id, span=span(start, end))


class TestIou:
    def test_identical_spans_overlap_completely(self) -> None:
        assert iou(span(0, 10), span(0, 10)) == 1.0

    def test_disjoint_spans_do_not_overlap_at_all(self) -> None:
        assert iou(span(0, 10), span(10, 20)) == 0.0
        assert iou(span(0, 10), span(30, 40)) == 0.0

    def test_half_overlap_is_a_third_not_a_half(self) -> None:
        # 0..10 and 5..15 share 5 s and cover 15 s. The number people expect
        # here is 0.5 and the number IoU gives is 1/3; the test exists to make
        # that explicit rather than to be discovered when a bar is set.
        assert iou(span(0, 10), span(5, 15)) == pytest.approx(1 / 3)

    def test_the_iou_half_threshold_needs_two_thirds_overlap(self) -> None:
        # 0..10 against 0..15: 10 shared, 15 covered, exactly 2/3.
        assert iou(span(0, 10), span(0, 15)) == pytest.approx(2 / 3)
        # A span a third longer at each end drops below the threshold.
        assert iou(span(0, 10), span(0, 30)) == pytest.approx(1 / 3)

    def test_an_empty_span_overlaps_nothing_including_itself(self) -> None:
        assert iou(Interval(5, 5), Interval(5, 5)) == 0.0
        assert iou(Interval(5, 5), span(0, 10)) == 0.0

    def test_it_does_not_care_which_way_round_it_is_asked(self) -> None:
        assert iou(span(3, 9), span(5, 15)) == iou(span(5, 15), span(3, 9))


class TestAlternatives:
    def test_a_cut_on_an_alternative_is_found_when_the_preferred_span_is_not(self) -> None:
        # Preferred 0..10, alternative 20..30. A clip at 20..30 shares nothing
        # with the preferred span and everything with the alternative.
        target = moment("m1", 0, 10, alternatives=(span(20, 30),))
        matches = match_moments([clip("c1", 20, 30)], [target])
        assert matches[0].matched
        assert matches[0].best_iou == 1.0

    def test_edge_error_is_measured_against_one_alternative_not_a_mix(self) -> None:
        # Preferred 0..10, alternative 4..20. A cut at 0..20 is 0 from the
        # preferred start and 0 from the alternative end — but no annotator
        # offered 0..20, so the error must not be reported as zero.
        target = moment("m1", 0, 10, alternatives=(span(4, 20),))
        measured = edge_error(span(0, 20), target)
        # Against the preferred span: 0 + 10 s. Against the alternative: 4 + 0 s.
        assert measured == 4 * SECOND

    def test_edge_error_is_zero_only_for_a_cut_somebody_offered(self) -> None:
        target = moment("m1", 0, 10, alternatives=(span(1, 11),))
        assert edge_error(span(1, 11), target) == 0
        assert edge_error(span(0, 10), target) == 0
        assert edge_error(span(0, 11), target) == SECOND


class TestMatching:
    def test_one_clip_may_satisfy_two_moments(self) -> None:
        # Forcing a one-to-one assignment would report 1/2 here, which is not
        # what happened: both moments were on screen.
        clips = [clip("c1", 0, 10)]
        matches = match_moments(clips, [moment("m1", 0, 10), moment("m2", 1, 10)])
        assert [match.matched for match in matches] == [True, True]

    def test_a_near_miss_is_reported_with_the_overlap_it_reached(self) -> None:
        matches = match_moments([clip("c1", 0, 30)], [moment("m1", 0, 10)])
        assert not matches[0].matched
        assert matches[0].best_iou == pytest.approx(1 / 3)
        assert matches[0].edge_error_ticks is None

    def test_an_unmatched_moment_reports_no_edge_error(self) -> None:
        # An error against a moment nobody found is not a measurement, and
        # folding a zero in would drag the median toward good news.
        matches = match_moments([], [moment("m1", 0, 10)])
        assert matches[0].edge_error_ticks is None


class TestDuplicates:
    def test_the_second_cut_of_one_moment_is_the_duplicate(self) -> None:
        rate, count = duplicate_rate([clip("c1", 0, 10), clip("c2", 0, 10)])
        assert count == 1
        assert rate == 0.5

    def test_three_cuts_of_one_moment_count_twice(self) -> None:
        _rate, count = duplicate_rate([clip("c1", 0, 10), clip("c2", 0, 10), clip("c3", 0, 10)])
        assert count == 2

    def test_distinct_moments_are_not_duplicates(self) -> None:
        rate, count = duplicate_rate([clip("c1", 0, 10), clip("c2", 40, 50)])
        assert (rate, count) == (0.0, 0)

    def test_an_empty_set_has_no_duplicate_rate_rather_than_a_division(self) -> None:
        assert duplicate_rate([]) == (0.0, 0)


class TestItemScore:
    def test_only_the_top_k_are_scored_for_recall(self) -> None:
        clips = [clip(f"c{index}", 100 + index, 105 + index) for index in range(10)]
        clips.append(clip("late", 0, 10))
        scored = score_item(annotation(moment("m1", 0, 10)), clips, recall_k=10)
        assert scored.moments_recalled == 0
        # And it is found once the board is allowed to be longer.
        assert score_item(annotation(moment("m1", 0, 10)), clips, recall_k=11).moments_recalled == 1

    def test_a_recording_with_nothing_worth_clipping_is_perfect_recall(self) -> None:
        # Not a division by zero and not a failure: the annotator said there was
        # nothing, and nothing was missed.
        scored = score_item(annotation(), [clip("c1", 0, 10)])
        assert scored.recall == 1.0

    def test_an_excluded_span_offered_is_counted_apart_from_recall(self) -> None:
        excluded = (Exclusion(span=span(0, 10), reason="rights"),)
        scored = score_item(
            annotation(moment("m1", 40, 50), exclusions=excluded),
            [clip("c1", 0, 10), clip("c2", 40, 50)],
        )
        assert scored.moments_recalled == 1
        assert scored.excluded_offered == 1

    def test_multi_moment_is_about_recordings_with_more_than_one(self) -> None:
        single = score_item(annotation(moment("m1", 0, 10)), [clip("c1", 0, 10)])
        assert not single.multi_moment
        assert not single.all_moments_recalled


class TestCorpusScore:
    def test_recall_pools_over_moments_rather_than_averaging_recordings(self) -> None:
        # One recording with nine moments, all found; one with a single moment,
        # missed. Pooled: 9/10. Averaged over recordings: (1.0 + 0.0)/2 = 0.5.
        many = annotation(*[moment(f"m{index}", index * 20, index * 20 + 10) for index in range(9)])
        found = [clip(f"c{index}", index * 20, index * 20 + 10) for index in range(9)]
        one = annotation(moment("solo", 0, 10))
        corpus = score_corpus([score_item(many, found), score_item(one, [])])
        assert corpus.moments_total == 10
        assert corpus.recall == pytest.approx(0.9)

    def test_multi_moment_recall_excludes_recordings_that_cannot_fail_it(self) -> None:
        pair = annotation(moment("m1", 0, 10), moment("m2", 40, 50))
        half = score_item(pair, [clip("c1", 0, 10)])
        single = score_item(annotation(moment("s", 0, 10)), [clip("c1", 0, 10)])
        corpus = score_corpus([half, single])
        # One recording is in the population, and it did not give up both, so
        # the single-moment recording must not rescue the number.
        assert corpus.report()["multi_moment_items"] == 1
        assert corpus.multi_moment_recall == 0.0

        # And it reaches one when the second moment is found too.
        whole = score_item(pair, [clip("c1", 0, 10), clip("c2", 40, 50)])
        assert score_corpus([whole, single]).multi_moment_recall == 1.0

    def test_recall_by_importance_separates_the_failure_that_matters(self) -> None:
        # Every acceptable moment found, the essential one missed: an aggregate
        # of 2/3 that hides a product nobody can use.
        document = annotation(
            moment("must", 0, 10, importance="essential"),
            moment("nice1", 40, 50, importance="acceptable"),
            moment("nice2", 80, 90, importance="acceptable"),
        )
        corpus = score_corpus([score_item(document, [clip("c1", 40, 50), clip("c2", 80, 90)])])
        assert corpus.recall == pytest.approx(2 / 3)
        by_grade = corpus.recall_by_importance()
        assert by_grade["essential"] == 0.0
        assert by_grade["acceptable"] == 1.0

    def test_the_report_states_the_protocol_it_was_measured_under(self) -> None:
        corpus = score_corpus([score_item(annotation(moment("m1", 0, 10)), [clip("c1", 0, 10)])])
        report = corpus.report()
        assert report["protocol"] == {
            "recall_k": 10,
            "duplicate_k": 5,
            "iou_threshold": 0.5,
        }
        assert report["schema_version"] == "clipmill.eval.recall.v1"

    def test_edge_error_percentiles_come_back_in_milliseconds(self) -> None:
        # One moment found exactly, one found a second late at each end.
        exact = score_item(annotation(moment("m1", 0, 10)), [clip("c1", 0, 10)])
        late = score_item(annotation(moment("m2", 0, 10)), [clip("c2", 1, 11)])
        report = score_corpus([exact, late]).report()
        assert report["boundary_edge_error_millis"]["measured"] == 2
        # Errors are 0 and 2 s; the nearest-rank median of two values is the
        # first, and p90 is the second.
        assert report["boundary_edge_error_millis"]["median"] == 0
        assert report["boundary_edge_error_millis"]["p90"] == 2000

    def test_a_report_names_every_miss_so_a_failure_can_be_looked_at(self) -> None:
        corpus = score_corpus([score_item(annotation(moment("m1", 0, 10)), [clip("c1", 0, 30)])])
        report = corpus.report()
        assert report["items"][0]["misses"] == [
            {"moment_id": "m1", "best_iou": pytest.approx(1 / 3)}
        ]
