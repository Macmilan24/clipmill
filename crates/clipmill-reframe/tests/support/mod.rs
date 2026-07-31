//! Synthetic trajectories: one behaviour each, so a failure names itself.
#![allow(
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_precision_loss
)]

use std::num::NonZeroU64;

use clipmill_contracts::schemas::vision_face_track::{
    Box as FaceBox, Coverage, Producer, Sha256, Timebase, Track, VisionFaceTrack,
    VisionFaceTrackDetection,
};

pub const SECOND: u64 = 90_000;
/// Ingest samples four frames a second, and so does every fixture here.
pub const FRAME_TICKS: u64 = SECOND / 4;
const FACE_HEIGHT: f64 = 0.2;

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

fn face(t_ticks: u64, cx: f64, cy: f64) -> FaceBox {
    let width = FACE_HEIGHT * 0.75;
    FaceBox {
        t_ticks,
        x: cx - width / 2.0,
        y: cy - FACE_HEIGHT / 2.0,
        w: width,
        h: FACE_HEIGHT,
        score: 0.93,
        interpolated: None,
    }
}

fn assemble(track_id: u64, boxes: Vec<FaceBox>) -> Track {
    let seen = boxes.len().max(1) as u64;
    let mean = if boxes.is_empty() {
        0.0
    } else {
        boxes.iter().map(|item| item.score).sum::<f64>() / boxes.len() as f64
    };
    Track {
        track_id,
        first_ticks: boxes.first().map_or(0, |item| item.t_ticks),
        last_ticks: boxes.last().map_or(0, |item| item.t_ticks),
        frames_present: nonzero(seen),
        mean_score: mean,
        boxes,
    }
}

fn frames(from_second: u64, to_second: u64) -> impl Iterator<Item = (u64, f64)> {
    let first = from_second * 4;
    let last = to_second * 4;
    let total = (last - first).max(1) as f64;
    (first..last).map(move |index| (index * FRAME_TICKS, (index - first) as f64 / total))
}

/// Somebody sitting still.
pub fn still(track_id: u64, from: u64, to: u64, cx: f64, cy: f64) -> Track {
    assemble(
        track_id,
        frames(from, to).map(|(at, _)| face(at, cx, cy)).collect(),
    )
}

/// Somebody crossing the frame at a constant speed — the case a naive tracker
/// snaps on.
pub fn walking(track_id: u64, from: u64, to: u64, start_x: f64, end_x: f64) -> Track {
    assemble(
        track_id,
        frames(from, to)
            .map(|(at, ratio)| face(at, start_x + (end_x - start_x) * ratio, 0.4))
            .collect(),
    )
}

/// The same crossing, but far enough to push the crop against both edges.
pub fn drifting(track_id: u64, from: u64, to: u64, start_x: f64, end_x: f64) -> Track {
    walking(track_id, from, to, start_x, end_x)
}

/// A still subject whose detected box jitters by `amplitude` every other frame.
/// This is chasing bait.
pub fn flickering(track_id: u64, from: u64, to: u64, cx: f64, cy: f64, amplitude: f64) -> Track {
    assemble(
        track_id,
        frames(from, to)
            .enumerate()
            .map(|(index, (at, _))| {
                let offset = if index % 2 == 0 {
                    amplitude
                } else {
                    -amplitude
                };
                face(at, cx + offset, cy)
            })
            .collect(),
    )
}

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
