//! What the crop path has to be true of, against trajectories built to test it.
//!
//! Synthetic rather than recorded, because the behaviours worth pinning are
//! ones a real clip mixes together: a subject who holds still, one who walks
//! across the frame, one the detector loses for a moment. A test over real
//! footage would be measuring the detector as much as the camera.
//!
//! The two properties the gate names are here: **jerk is bounded** — the camera
//! does not lurch, which is the failure the book says users punish hardest —
//! and **containment**, the share of observed faces the crop actually holds.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]

use clipmill_reframe::{FitReason, FocusGate, Keyframe, Weights, solve};

mod support;
use support::{SECOND, document, drifting, flickering, still, walking};

const ASPECT: (u32, u32) = (9, 16);

/// Resample the sparse keyframes back to a dense path the way a player would,
/// which is the only reading of them that matters.
fn interpolate(keyframes: &[Keyframe], at: u64) -> (f64, f64, f64) {
    if keyframes.is_empty() {
        return (0.5, 0.5, 1.0);
    }
    let first = keyframes[0];
    if at <= first.t_ticks {
        return (first.center_x, first.center_y, first.scale);
    }
    for pair in keyframes.windows(2) {
        let (left, right) = (pair[0], pair[1]);
        if at <= right.t_ticks {
            let span = (right.t_ticks - left.t_ticks) as f64;
            let ratio = if span <= 0.0 {
                0.0
            } else {
                (at - left.t_ticks) as f64 / span
            };
            return (
                left.center_x + (right.center_x - left.center_x) * ratio,
                left.center_y + (right.center_y - left.center_y) * ratio,
                left.scale + (right.scale - left.scale) * ratio,
            );
        }
    }
    let last = keyframes[keyframes.len() - 1];
    (last.center_x, last.center_y, last.scale)
}

/// Largest third difference of the centre path, sampled at the frame rate.
///
/// Jerk rather than acceleration because acceleration is what the objective
/// already penalises; jerk is what survives it, and a bounded one is the
/// difference between a camera that moves and a camera that snaps.
fn peak_jerk(keyframes: &[Keyframe], from: u64, to: u64, step: u64) -> f64 {
    let samples: Vec<f64> = (from..=to)
        .step_by(step as usize)
        .map(|at| interpolate(keyframes, at).0)
        .collect();
    let mut worst = 0.0_f64;
    for window in samples.windows(4) {
        let jerk = window[3] - 3.0 * window[2] + 3.0 * window[1] - window[0];
        worst = worst.max(jerk.abs());
    }
    worst
}

#[test]
fn a_still_subject_produces_a_still_camera() {
    let doc = document(vec![still(0, 0, 8, 0.5, 0.4)]);
    let path = solve(
        &doc,
        0,
        8 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");

    assert!(!path.fit, "a clear single subject should be followed");
    // Two keyframes: a camera that is not moving needs no more than its ends.
    assert_eq!(path.keyframes.len(), 2, "{:?}", path.keyframes);
    assert!((path.keyframes[0].center_x - path.keyframes[1].center_x).abs() < 1e-6);
    assert!(path.containment > 0.99, "containment {}", path.containment);
}

/// The property the gate names. A subject crossing the frame is exactly when a
/// naive tracker snaps, and the damping terms exist to stop it.
#[test]
fn a_walking_subject_is_followed_without_lurching() {
    let doc = document(vec![walking(0, 0, 12, 0.25, 0.75)]);
    let path = solve(
        &doc,
        0,
        12 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");

    assert!(!path.fit);
    assert!(
        path.containment >= 0.98,
        "containment {} below the gate's floor",
        path.containment
    );
    let jerk = peak_jerk(&path.keyframes, 0, 12 * SECOND, SECOND / 4);
    assert!(jerk < 0.02, "peak jerk {jerk} — the camera lurched");
}

/// A detector flickering between two positions is the chasing case. The camera
/// should hold its line, not follow the noise.
#[test]
fn a_flickering_detection_does_not_move_the_camera_much() {
    let steady = solve(
        &document(vec![still(0, 0, 8, 0.5, 0.4)]),
        0,
        8 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");
    let noisy = solve(
        &document(vec![flickering(0, 0, 8, 0.5, 0.4, 0.06)]),
        0,
        8 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");

    let steady_at = interpolate(&steady.keyframes, 4 * SECOND).0;
    let noisy_at = interpolate(&noisy.keyframes, 4 * SECOND).0;
    assert!(
        (steady_at - noisy_at).abs() < 0.02,
        "a ±0.06 flicker moved the camera by {}",
        (steady_at - noisy_at).abs()
    );
    let jerk = peak_jerk(&noisy.keyframes, 0, 8 * SECOND, SECOND / 4);
    assert!(
        jerk < 0.02,
        "peak jerk {jerk} — the camera chased the flicker"
    );
}

/// The crop never leaves the picture, however far the subject goes.
#[test]
fn the_crop_stays_inside_the_frame() {
    let doc = document(vec![drifting(0, 0, 10, 0.02, 0.98)]);
    let path = solve(
        &doc,
        0,
        10 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");

    for at in (0..=10 * SECOND).step_by((SECOND / 4) as usize) {
        let (x, y, scale) = interpolate(&path.keyframes, at);
        let half_w = (scale * (9.0 / 16.0) / (16.0 / 9.0)) / 2.0;
        let half_h = scale / 2.0;
        assert!(
            x - half_w >= -1e-6 && x + half_w <= 1.0 + 1e-6,
            "crop left the frame horizontally at {at}: x={x} half={half_w}"
        );
        assert!(
            y - half_h >= -1e-6 && y + half_h <= 1.0 + 1e-6,
            "crop left the frame vertically at {at}: y={y} half={half_h}"
        );
    }
}

/// A speed limit expressed in frame-widths per second means the same thing at
/// any resolution, which is the whole reason it is normalized.
#[test]
fn the_camera_never_exceeds_the_speed_it_was_given() {
    let doc = document(vec![walking(0, 0, 6, 0.1, 0.9)]);
    let weights = Weights {
        max_speed_per_second: 0.05,
        ..Weights::default()
    };
    let path = solve(
        &doc,
        0,
        6 * SECOND,
        ASPECT.0,
        ASPECT.1,
        weights,
        FocusGate::default(),
    )
    .expect("a solve");

    let step = SECOND / 4;
    let mut previous = interpolate(&path.keyframes, 0).0;
    for at in (step..=6 * SECOND).step_by(step as usize) {
        let now = interpolate(&path.keyframes, at).0;
        let seconds = step as f64 / SECOND as f64;
        assert!(
            (now - previous).abs() <= 0.05 * seconds + 1e-6,
            "moved {} in {seconds}s, limit {}",
            (now - previous).abs(),
            0.05 * seconds
        );
        previous = now;
    }
}

/// The same evidence twice is the same path twice. An artifact addressed by
/// content cannot afford a solver that drifts.
#[test]
fn the_same_evidence_produces_the_same_path() {
    let doc = document(vec![walking(0, 0, 9, 0.2, 0.8)]);
    let first = solve(
        &doc,
        0,
        9 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");
    let second = solve(
        &doc,
        0,
        9 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");
    assert_eq!(first.keyframes, second.keyframes);
}

#[test]
fn a_refused_solve_is_a_centred_frame_with_a_reason() {
    let doc = document(vec![still(0, 0, 2, 0.5, 0.4)]);
    let path = solve(
        &doc,
        0,
        10 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");

    assert!(path.fit);
    assert_eq!(path.fit_reason, Some(FitReason::TooIntermittent));
    assert_eq!(path.track_id, None);
    assert_eq!(path.keyframes.len(), 2);
    for frame in &path.keyframes {
        assert!((frame.center_x - 0.5).abs() < 1e-9);
        assert!((frame.scale - 1.0).abs() < 1e-9);
    }
}

#[test]
fn an_empty_span_and_a_degenerate_aspect_are_refused() {
    let doc = document(vec![still(0, 0, 4, 0.5, 0.4)]);
    assert!(
        solve(
            &doc,
            5 * SECOND,
            5 * SECOND,
            9,
            16,
            Weights::default(),
            FocusGate::default()
        )
        .is_err()
    );
    assert!(
        solve(
            &doc,
            0,
            SECOND,
            0,
            16,
            Weights::default(),
            FocusGate::default()
        )
        .is_err()
    );
}

/// Sparse output, and sparse in a way that survives being read back: the
/// interpolated path must match the solved one everywhere, not just at the
/// keyframes that were kept.
#[test]
fn the_keyframes_reproduce_the_path_they_were_reduced_from() {
    let doc = document(vec![walking(0, 0, 12, 0.2, 0.8)]);
    let path = solve(
        &doc,
        0,
        12 * SECOND,
        ASPECT.0,
        ASPECT.1,
        Weights::default(),
        FocusGate::default(),
    )
    .expect("a solve");

    assert!(
        path.keyframes.len() < 48,
        "48 samples reduced to only {}",
        path.keyframes.len()
    );
    // Monotone in time, which a player relies on.
    for pair in path.keyframes.windows(2) {
        assert!(pair[1].t_ticks > pair[0].t_ticks);
    }
}
