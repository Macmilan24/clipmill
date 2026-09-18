//! What a trim keeps at its new edges.
//!
//! A trim moves a segment's window, and everything anchored to the window
//! moves with it. Two things used to fall off the edge instead: the crop the
//! path held at the new in point — a static crop is one keyframe at zero,
//! and advancing the head past it left a `speaker_fill` segment with no path
//! at all, which the preview drew as fit and the renderer refused — and the
//! gain the curve held at the boundary, which the renderer then replaced with
//! a hold of whatever point came next. Both are evaluated and kept, and the
//! undo of a trim that kept them is exact.
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use clipmill_edit_ir::{
    CropKeyframe, CropRect, EditCommand, EditDocument, GainPoint, Layout, LayoutState,
    TICKS_PER_SECOND, VideoSegment, crop_along, gain_at,
};

const SOURCE: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const SECOND: i64 = TICKS_PER_SECOND;

fn rect(x: i64) -> CropRect {
    CropRect {
        x,
        y: 0,
        width: 608,
        height: 1080,
    }
}

/// One segment playing source 10s–20s, cropped as asked.
fn document(path: Vec<CropKeyframe>) -> EditDocument {
    let mut document = EditDocument::default();
    document.video.segments = vec![VideoSegment {
        segment_id: "seg_1".to_owned(),
        source_fingerprint: SOURCE.to_owned(),
        in_ticks: 10 * SECOND,
        out_ticks: 20 * SECOND,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: path,
        },
    }];
    document.validate().expect("a valid document");
    document
}

fn trim(document: &mut EditDocument, in_ticks: i64, out_ticks: i64) -> EditCommand {
    EditCommand::Trim {
        segment_id: "seg_1".to_owned(),
        in_ticks,
        out_ticks,
    }
    .apply(document)
    .expect("the trim applies")
}

fn path_of(document: &EditDocument) -> Vec<(i64, CropRect)> {
    document.video.segments[0]
        .layout
        .crop_path
        .iter()
        .map(|keyframe| (keyframe.t_ticks, keyframe.rect))
        .collect()
}

/// The reproduction: a static crop is one keyframe at zero, and advancing
/// the head past it must leave the crop, not the segment's claim to one.
#[test]
fn a_static_crop_survives_a_head_trim() {
    let before = document(vec![CropKeyframe {
        t_ticks: 0,
        rect: rect(236),
    }]);
    let mut document = before.clone();
    let inverse = trim(&mut document, 12 * SECOND, 20 * SECOND);

    assert_eq!(path_of(&document), vec![(0, rect(236))]);
    assert_eq!(
        document.video.segments[0].layout.state,
        LayoutState::SpeakerFill
    );
    document.validate().expect("still valid");

    inverse.apply(&mut document).expect("undo");
    assert_eq!(document, before, "the undo is exact");
}

/// A path that moves keeps its shape inside the new window, and where the
/// camera stood at each new edge is written down as a keyframe there —
/// evaluated by the same arithmetic the renderer draws with.
#[test]
fn a_moving_crop_keeps_its_position_at_both_new_edges() {
    let path = vec![
        CropKeyframe {
            t_ticks: 0,
            rect: rect(100),
        },
        CropKeyframe {
            t_ticks: 4 * SECOND,
            rect: rect(500),
        },
        CropKeyframe {
            t_ticks: 8 * SECOND,
            rect: rect(900),
        },
    ];
    let before = document(path.clone());
    let mut document = before.clone();
    // Source 11s–17s: local 1s and 7s of the old path, both between keyframes.
    let inverse = trim(&mut document, 11 * SECOND, 17 * SECOND);

    let old_positions: Vec<(i64, CropRect)> = path
        .iter()
        .map(|keyframe| (keyframe.t_ticks, keyframe.rect))
        .collect();
    let at_new_in = crop_along(&old_positions, SECOND).unwrap();
    let at_new_out = crop_along(&old_positions, 7 * SECOND).unwrap();
    assert_eq!(at_new_in.x, 200, "a quarter of the way from 100 to 500");
    assert_eq!(
        at_new_out.x, 800,
        "three quarters of the way from 500 to 900"
    );
    assert_eq!(
        path_of(&document),
        vec![
            (0, at_new_in),
            (3 * SECOND, rect(500)),
            (6 * SECOND, at_new_out)
        ],
        "the interior keyframe shifted; the edges hold what the path held there"
    );
    document.validate().expect("still valid");

    // The undo is the whole arrangement — the outer keyframes are not
    // recoverable from the window alone — and it is exact; and the redo
    // lands where the trim did.
    assert!(matches!(inverse, EditCommand::RestoreArrangement { .. }));
    let redo = inverse.apply(&mut document).expect("undo");
    assert_eq!(document, before);
    redo.apply(&mut document).expect("redo");
    assert_eq!(path_of(&document).len(), 3);
    assert_eq!(path_of(&document)[0], (0, at_new_in));
}

/// A trim that only extends the window changes no keyframe and undoes with
/// the narrow inverse; a trim that drops one does not.
#[test]
fn only_a_trim_that_changes_the_path_needs_the_whole_arrangement_back() {
    let mut document = document(vec![CropKeyframe {
        t_ticks: 0,
        rect: rect(236),
    }]);
    let inverse = trim(&mut document, 10 * SECOND, 22 * SECOND);
    assert!(matches!(inverse, EditCommand::Trim { .. }));
    let inverse = trim(&mut document, 11 * SECOND, 22 * SECOND);
    assert!(matches!(inverse, EditCommand::RestoreArrangement { .. }));
}

/// The reproduction: a ramp from 0 dB to 6 dB over three seconds, cut one
/// second in, still opens at 2 dB and reaches 6 dB two seconds later.
#[test]
fn a_head_trim_keeps_the_gain_the_audio_had_at_its_new_start() {
    let mut document = document(Vec::new());
    document.video.segments[0].layout.state = LayoutState::Fit;
    document.audio.gain_curve = vec![
        GainPoint {
            t_ticks: 0,
            gain_db: 0.0,
        },
        GainPoint {
            t_ticks: 3 * SECOND,
            gain_db: 6.0,
        },
    ];
    let before = document.clone();
    let inverse = trim(&mut document, 11 * SECOND, 20 * SECOND);

    let curve = &document.audio.gain_curve;
    assert_eq!(curve.len(), 2, "{curve:?}");
    assert_eq!(curve[0].t_ticks, 0);
    assert!((curve[0].gain_db - 2.0).abs() < 1e-9, "{curve:?}");
    assert_eq!(curve[1].t_ticks, 2 * SECOND);
    assert!((curve[1].gain_db - 6.0).abs() < 1e-9);
    // The envelope the kept audio had, second by second.
    for (t, expected) in [
        (0, 2.0),
        (SECOND, 4.0),
        (2 * SECOND, 6.0),
        (5 * SECOND, 6.0),
    ] {
        let heard = gain_at(curve, t).unwrap();
        let was = gain_at(&before.audio.gain_curve, t + SECOND).unwrap();
        assert!((heard - expected).abs() < 1e-9 && (heard - was).abs() < 1e-9);
    }
    document.validate().expect("still valid");

    inverse.apply(&mut document).expect("undo");
    assert_eq!(document, before, "the undo is exact");
}

/// A tail cut that takes the point a ramp was heading for keeps the value
/// the ramp had reached on the last tick that stays.
#[test]
fn a_tail_trim_keeps_the_gain_the_audio_had_at_its_new_end() {
    let mut document = document(Vec::new());
    document.video.segments[0].layout.state = LayoutState::Fit;
    document.audio.gain_curve = vec![
        GainPoint {
            t_ticks: 0,
            gain_db: 0.0,
        },
        GainPoint {
            t_ticks: 3 * SECOND,
            gain_db: 6.0,
        },
    ];
    let before = document.clone();
    let inverse = trim(&mut document, 10 * SECOND, 12 * SECOND);

    let curve = &document.audio.gain_curve;
    assert_eq!(curve.len(), 2, "{curve:?}");
    assert_eq!(curve[0].t_ticks, 0);
    assert_eq!(curve[1].t_ticks, 2 * SECOND - 1, "the last tick that stays");
    let was = gain_at(&before.audio.gain_curve, 2 * SECOND - 1).unwrap();
    assert!((curve[1].gain_db - was).abs() < 1e-9);
    assert!((was - 4.0).abs() < 1e-3);
    document.validate().expect("still valid");

    inverse.apply(&mut document).expect("undo");
    assert_eq!(document, before, "the undo is exact");
}

/// A curve wholly after the cut is held backwards by the renderer either
/// way, so nothing is added to it.
#[test]
fn a_curve_the_cut_never_reaches_is_left_alone() {
    let mut document = document(Vec::new());
    document.video.segments[0].layout.state = LayoutState::Fit;
    document.audio.gain_curve = vec![GainPoint {
        t_ticks: 5 * SECOND,
        gain_db: -3.0,
    }];
    let inverse = trim(&mut document, 11 * SECOND, 20 * SECOND);
    assert_eq!(
        document.audio.gain_curve,
        vec![GainPoint {
            t_ticks: 4 * SECOND,
            gain_db: -3.0
        }]
    );
    assert!(matches!(inverse, EditCommand::Trim { .. }));
}
