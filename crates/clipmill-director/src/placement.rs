//! One caption lane per clip, chosen against the actual composed face boxes.
//! This protects detected faces; it makes no claim about undetected graphics.
use crate::{Request, as_f64, as_i64};
use clipmill_contracts::schemas::vision_face_track::VisionFaceTrack;
use clipmill_edit_ir::{CaptionRegion, CropRect, LayoutState, VideoSegment, crop_along};

pub(crate) fn caption_lane(
    segments: &[VideoSegment],
    faces: Option<&VisionFaceTrack>,
    request: &Request,
) -> CaptionRegion {
    if segments
        .iter()
        .all(|s| s.layout.state == LayoutState::TwoUp)
    {
        return CaptionRegion::Center;
    }
    let Some(faces) = faces else {
        return CaptionRegion::LowerSafe;
    };
    // Approximate two-line occupied bands using the pinned default typography.
    // Prefer the familiar bottom lane whenever it is as clear as the others.
    let lanes = [
        (CaptionRegion::LowerSafe, 0.73, 0.91),
        (CaptionRegion::UpperSafe, 0.09, 0.27),
        (CaptionRegion::Center, 0.43, 0.57),
    ];
    let mut costs = [0.0; 3];
    for segment in segments {
        for face in faces.tracks.iter().flat_map(|track| &track.boxes) {
            let tick = as_i64(face.t_ticks);
            if tick < segment.in_ticks
                || tick >= segment.out_ticks
                || face.interpolated == Some(true)
            {
                continue;
            }
            let full = CropRect {
                x: 0,
                y: 0,
                width: request.frame.width,
                height: request.frame.height,
            };
            let paths = [
                &segment.layout.crop_path,
                &segment.layout.secondary_crop_path,
            ];
            let viewports = if segment.layout.state == LayoutState::TwoUp {
                2
            } else {
                1
            };
            for (viewport, path) in paths.iter().enumerate().take(viewports) {
                let keyed: Vec<_> = path.iter().map(|key| (key.t_ticks, key.rect)).collect();
                let crop = if segment.layout.state == LayoutState::Fit {
                    full
                } else {
                    crop_along(&keyed, tick - segment.in_ticks).unwrap_or(full)
                };
                let left = face.x * as_f64(request.frame.width);
                let right = (face.x + face.w) * as_f64(request.frame.width);
                if right < as_f64(crop.x) || left > as_f64(crop.x + crop.width) {
                    continue;
                }
                let mut top =
                    (face.y * as_f64(request.frame.height) - as_f64(crop.y)) / as_f64(crop.height);
                let mut bottom = ((face.y + face.h) * as_f64(request.frame.height)
                    - as_f64(crop.y))
                    / as_f64(crop.height);
                if segment.layout.state == LayoutState::Fit {
                    let source_aspect = as_f64(request.frame.width) / as_f64(request.frame.height);
                    let output_aspect =
                        f64::from(request.aspect.width) / f64::from(request.aspect.height);
                    let displayed_height = (output_aspect / source_aspect).min(1.0);
                    top = (1.0 - displayed_height) / 2.0 + top * displayed_height;
                    bottom = (1.0 - displayed_height) / 2.0 + bottom * displayed_height;
                }
                if viewports == 2 {
                    top = top / 2.0 + if viewport == 0 { 0.0 } else { 0.5 };
                    bottom = bottom / 2.0 + if viewport == 0 { 0.0 } else { 0.5 };
                }
                for (index, (_, low, high)) in lanes.iter().enumerate() {
                    costs[index] += (bottom.min(*high) - top.max(*low)).max(0.0) * face.score;
                }
            }
        }
    }
    let best = (0..costs.len())
        .min_by(|a, b| costs[*a].total_cmp(&costs[*b]))
        .unwrap_or(0);
    lanes[best].0
}
