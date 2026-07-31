//! Face-track documents built by hand, so a solver test says what it is about.
//!
//! Synthetic on purpose. The behaviours worth pinning — a still subject, a pan,
//! a detector flickering — are ones a real recording mixes together, and a test
//! over real frames would be testing the detector as much as the camera.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::cast_precision_loss)]

use std::num::NonZeroU64;

use clipmill_contracts::schemas::vision_face_track::{
    Box as FaceBox, Coverage, Producer, Sha256, Timebase, Track, VisionFaceTrack,
    VisionFaceTrackDetection,
};

pub const SECOND: u64 = 90_000;
/// Four samples a second, which is what ingest's frame pass produces.
pub const FRAME_TICKS: u64 = SECOND / 4;

fn digest(fill: char) -> Sha256 {
    format!(
        "sha256:{}",
        std::iter::repeat_n(fill, 64).collect::<String>()
    )
    .parse()
    .expect("a well-formed digest")
}

fn nonzero(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("a non-zero count")
}

/// One face box at a moment, centred on `(cx, cy)` and `height` tall.
pub fn face(t_ticks: u64, cx: f64, cy: f64, height: f64, score: f64) -> FaceBox {
    let width = height * 0.75;
    FaceBox {
        t_ticks,
        x: cx - width / 2.0,
        y: cy - height / 2.0,
        w: width,
        h: height,
        score,
        interpolated: None,
    }
}

/// A track that holds still, one sample per frame between two seconds.
pub fn track(track_id: u64, from_second: u64, to_second: u64, score: f64) -> Track {
    let boxes: Vec<FaceBox> = (from_second * 4..to_second * 4)
        .map(|index| face(index * FRAME_TICKS, 0.5, 0.4, 0.2, score))
        .collect();
    from_boxes(track_id, boxes)
}

/// A track from explicit boxes, with its summary fields derived rather than
/// stated — a fixture whose summary disagreed with its boxes would be testing
/// the fixture.
pub fn from_boxes(track_id: u64, boxes: Vec<FaceBox>) -> Track {
    let seen: Vec<&FaceBox> = boxes
        .iter()
        .filter(|item| item.interpolated != Some(true))
        .collect();
    let total: f64 = seen.iter().map(|item| item.score).sum();
    Track {
        track_id,
        first_ticks: boxes.first().map_or(0, |item| item.t_ticks),
        last_ticks: boxes.last().map_or(0, |item| item.t_ticks),
        frames_present: nonzero(seen.len().max(1) as u64),
        mean_score: if seen.is_empty() {
            0.0
        } else {
            total / seen.len() as f64
        },
        boxes,
    }
}

/// A document over the given tracks, examined and four frames a second.
pub fn document(tracks: Vec<Track>) -> VisionFaceTrack {
    let end = tracks
        .iter()
        .map(|track| track.last_ticks)
        .max()
        .unwrap_or(0);
    VisionFaceTrack {
        schema_version: serde_json::Value::String("clipmill.vision.face_track.v1".to_owned()),
        source_fingerprint: digest('a'),
        frames_artifact_id: digest('b'),
        producer: Producer {
            stage: "detect-faces".parse().expect("a stage name"),
            implementation: "test-fixture".parse().expect("an implementation"),
            model_digest: None,
        },
        detection: VisionFaceTrackDetection {
            score_threshold: 0.6,
            nms_iou: 0.3,
            input_width: 320,
            input_height: 320,
            match_iou: 0.5,
            recover_iou: 0.3,
            max_gap_frames: 6,
            min_track_frames: nonzero(4),
            frame_rate: Timebase {
                num: nonzero(4),
                den: nonzero(1),
            },
        },
        coverage: Coverage {
            start_ticks: 0,
            end_ticks: end,
            analyzed: true,
            frames_examined: None,
        },
        tracks,
    }
}
