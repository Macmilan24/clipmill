//! What the lattice must be: legal, wide, and not pre-chosen.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use clipmill_contracts::schemas::discovery_candidates as contract;

use crate::fixture::{self, SECOND};

use super::{Boundaries, inside_a_word};

fn duration(min_seconds: u64, max_seconds: u64) -> contract::DurationRange {
    contract::DurationRange {
        min_ticks: std::num::NonZeroU64::new(min_seconds * SECOND).expect("a length"),
        max_ticks: std::num::NonZeroU64::new(max_seconds * SECOND).expect("a length"),
    }
}

#[test]
fn a_tick_on_a_word_edge_is_a_boundary_not_an_interruption() {
    let words = vec![
        fixture::word(0, (0, 9_000), "the"),
        fixture::word(1, (9_000, 18_000), "renderer"),
    ];
    // Strictly inside.
    assert!(inside_a_word(&words, 4_500));
    assert!(inside_a_word(&words, 13_000));
    // On an edge, which is exactly where a cut is safe.
    assert!(!inside_a_word(&words, 0));
    assert!(!inside_a_word(&words, 9_000));
    assert!(!inside_a_word(&words, 18_000));
    // In the silence after.
    assert!(!inside_a_word(&words, 30_000));
}

/// The one structural term. A shot cut was measured against pixels, which know
/// nothing about speech, so it is the only boundary source that can land
/// mid-word — and the only one this can reject.
#[test]
fn a_shot_cut_inside_a_word_never_enters_the_lattice() {
    let interview = fixture::interview();
    let clean = Boundaries::gather(&interview.index, &interview.transcript.words);
    assert_eq!(clean.mid_word_rejects, 0);

    let mut index = interview.index.clone();
    // Move the cut to the middle of the first word, which runs 0 to 45000.
    index.edges = vec![fixture::shot_cut(20_000)];
    let dirty = Boundaries::gather(&index, &interview.transcript.words);
    assert_eq!(dirty.mid_word_rejects, 1);
    assert!(!dirty.starts.contains(&20_000));
    assert!(!dirty.ends.contains(&20_000));
}

/// Sentence and utterance edges come from the index and are word-aligned by
/// construction, so nothing the index published should ever be rejected.
#[test]
fn every_gathered_boundary_is_structurally_legal() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    for at in boundaries.starts.iter().chain(boundaries.ends.iter()) {
        assert!(
            !inside_a_word(&interview.transcript.words, *at),
            "{at} falls inside a word"
        );
    }
}

/// The recording's own bounds are always legal: a clip may start at the
/// beginning and end at the end, whatever else was detected.
#[test]
fn the_recording_bounds_are_always_boundaries() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    assert!(
        boundaries
            .starts
            .contains(&interview.index.coverage.start_ticks)
    );
    assert!(
        boundaries
            .ends
            .contains(&interview.index.coverage.end_ticks)
    );
}

#[test]
fn every_pair_the_lattice_publishes_satisfies_phi() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    let want = duration(15, 180);
    let coverage = (0, 130 * SECOND);
    let seed = (10 * SECOND, 40 * SECOND);
    let lattice = boundaries.expand(seed, &want, coverage).expect("a lattice");

    // Not merely "some pair is legal": at least one legal pair exists for
    // every published point, which is what makes the point worth publishing.
    for start in &lattice.starts {
        assert!(
            lattice
                .ends
                .iter()
                .any(|end| super::legality(*start, *end, &want, coverage).is_ok()),
            "{start} pairs with nothing legal"
        );
    }
    for end in &lattice.ends {
        assert!(
            lattice
                .starts
                .iter()
                .any(|start| super::legality(*start, *end, &want, coverage).is_ok()),
            "{end} pairs with nothing legal"
        );
    }
}

/// Discovery keeps the whole lattice. A single legal pair would be discovery
/// making the boundary decision with less information than ranking has.
#[test]
fn the_lattice_is_wider_than_the_interval_it_settled_on() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    let lattice = boundaries
        .expand(
            (10 * SECOND, 40 * SECOND),
            &duration(15, 180),
            (0, 130 * SECOND),
        )
        .expect("a lattice");
    assert!(lattice.starts.len() > 1 || lattice.ends.len() > 1);
    assert!(lattice.starts.contains(&lattice.interval.0));
    assert!(lattice.ends.contains(&lattice.interval.1));
}

/// The interval discovery settles on is the tightest legal one containing the
/// seed: the one that adds least to what the proposer actually pointed at.
#[test]
fn the_settled_interval_adds_as_little_as_the_lattice_allows() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    let want = duration(15, 180);
    let coverage = (0, 130 * SECOND);
    let lattice = boundaries
        .expand((10 * SECOND, 40 * SECOND), &want, coverage)
        .expect("a lattice");
    let span = lattice.interval.1 - lattice.interval.0;
    for start in &lattice.starts {
        for end in &lattice.ends {
            if super::legality(*start, *end, &want, coverage).is_ok() {
                assert!(end - start >= span, "a tighter legal interval existed");
            }
        }
    }
}

/// A seed with nowhere legal to grow is a fact about the recording, not a
/// failure. Asking for clips longer than the recording is the clearest case.
#[test]
fn a_seed_with_no_legal_interval_yields_nothing() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    assert!(
        boundaries
            .expand(
                (10 * SECOND, 40 * SECOND),
                &duration(600, 900),
                (0, 130 * SECOND)
            )
            .is_none()
    );
}

/// Rejections are counted by reason rather than listed. The pairs are the
/// product of two sets, so enumerating them would make the artifact quadratic
/// in the recording's length to record something recomputable.
#[test]
fn rejections_are_counted_by_reason_and_the_reasons_are_real() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    // A narrow window, so both duration terms fire.
    let lattice = boundaries
        .expand(
            (30 * SECOND, 35 * SECOND),
            &duration(20, 30),
            (0, 130 * SECOND),
        )
        .expect("a lattice");
    let published = lattice.published(0);
    let reasons = published
        .phi_rejects
        .iter()
        .map(|reject| reject.reason)
        .collect::<Vec<_>>();
    assert!(reasons.contains(&contract::PhiRejectReason::TooShort));
    assert!(reasons.contains(&contract::PhiRejectReason::TooLong));
    // Counts are non-zero by type, and a reason that never fired is absent
    // rather than present with a zero.
    assert!(!reasons.contains(&contract::PhiRejectReason::MidWord));
}

#[test]
fn the_structural_rejects_reach_the_published_lattice() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    let lattice = boundaries
        .expand(
            (10 * SECOND, 40 * SECOND),
            &duration(15, 180),
            (0, 130 * SECOND),
        )
        .expect("a lattice");
    let published = lattice.published(3);
    let mid_word = published
        .phi_rejects
        .iter()
        .find(|reject| reject.reason == contract::PhiRejectReason::MidWord)
        .expect("the structural term is reported");
    assert_eq!(mid_word.count.get(), 3);
}

#[test]
fn expanding_the_same_seed_twice_gives_the_same_lattice() {
    let interview = fixture::interview();
    let boundaries = Boundaries::gather(&interview.index, &interview.transcript.words);
    let once = boundaries
        .expand(
            (10 * SECOND, 40 * SECOND),
            &duration(15, 180),
            (0, 130 * SECOND),
        )
        .expect("a lattice");
    let twice = boundaries
        .expand(
            (10 * SECOND, 40 * SECOND),
            &duration(15, 180),
            (0, 130 * SECOND),
        )
        .expect("a lattice");
    assert_eq!(once.starts, twice.starts);
    assert_eq!(once.ends, twice.ends);
    assert_eq!(once.interval, twice.interval);
}
