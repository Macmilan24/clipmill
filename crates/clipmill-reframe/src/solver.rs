//! The virtual camera: a crop path that follows a face without chasing it.
//!
//! Ch. 18's objective, with the terms this phase can evaluate:
//!
//! ```text
//! min_x  Σ_t [ w_s‖x_t − c_t‖² + w_v‖x_t − x_{t−1}‖² + w_a‖x_t − 2x_{t−1} + x_{t−2}‖² ]
//! ```
//!
//! `c_t` is where the subject is, so the subject term is containment written as
//! a least-squares pull rather than as a barrier; the velocity and acceleration
//! terms are the "human operator" damping. The protected-region term the book
//! also lists is absent, because protected regions are text and graphics nobody
//! has detected at this phase, and a term recorded as never firing reads like a
//! term that was checked.
//!
//! Every one of those terms exists to prevent **chasing** — the camera lurching
//! at every detection flicker, which the book names as the failure users punish
//! hardest — and the acceleration weight does the heaviest lifting.
//!
//! Written out, the objective is a quadratic in x, so its stationary point is
//! one linear system. The acceleration term couples each sample to the two
//! either side and nothing further, which makes that system pentadiagonal:
//! banded Cholesky, O(n) rather than O(n³), microseconds for a clip. x and y are
//! independent under this objective and are solved as two such systems.
//!
//! Three things the quadratic cannot express are applied afterwards, and are
//! projections rather than parts of the optimum — stated plainly because the
//! result is then a feasible near-optimal path rather than a constrained
//! optimum, and a reader deserves to know which they have:
//!
//! 1. the crop must stay inside the frame,
//! 2. the camera may not exceed a speed, in frame-widths per second so the
//!    behaviour does not change with resolution,
//! 3. the face must not be smaller than a floor of the crop, which is what
//!    fixes the scale.

use clipmill_contracts::schemas::vision_face_track::{Track, VisionFaceTrack};

use crate::banded::{Banded, BandedError};
use crate::tracks::{FitReason, Focus, FocusGate, resolve};

const TICKS_PER_SECOND: f64 = 90_000.0;

/// Ticks as seconds.
///
/// The cast is exact for anything anybody will ever edit: a double holds every
/// integer to 2^53, which at this timebase is over three thousand years of
/// recording. The lint is right in general and wrong here, so it is silenced
/// once, in the one place the conversion happens.
#[allow(
    clippy::cast_precision_loss,
    reason = "ticks stay far inside a double's exact integer range"
)]
fn seconds(ticks: u64) -> f64 {
    ticks as f64 / TICKS_PER_SECOND
}

/// A count as a divisor, for the same reason.
#[allow(
    clippy::cast_precision_loss,
    reason = "sample counts are thousands, not quadrillions"
)]
fn count(value: usize) -> f64 {
    value as f64
}

/// The objective's weights, and the two limits applied after it.
#[derive(Clone, Copy, Debug)]
pub struct Weights {
    pub subject: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub zoom: f64,
    /// Largest camera speed, in frame-widths per second.
    pub max_speed_per_second: f64,
}

impl Default for Weights {
    /// Ratios rather than absolutes: what matters is how hard the damping pulls
    /// against the subject term, and only the ratio decides that.
    ///
    /// Acceleration is weighted an order of magnitude above velocity because it
    /// is what stops a lurch, while velocity alone only slows one down. Subject
    /// at 1.0 is the reference every other weight is read against. Speed at 0.6
    /// frame-widths per second is a fast but human pan — past about one, the
    /// move reads as a whip rather than an operator.
    fn default() -> Self {
        Self {
            subject: 1.0,
            velocity: 4.0,
            acceleration: 40.0,
            zoom: 8.0,
            max_speed_per_second: 0.6,
        }
    }
}

/// How much of the crop's height the face must occupy, at least.
///
/// The hard constraint ch. 18 names as "minimum face size". Below this the
/// subject is technically contained and practically unreadable, which is the
/// same as not being framed at all.
const MIN_FACE_FRACTION: f64 = 0.16;

/// One point on the solved path. Normalized against the source frame, so the
/// same path drives any resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe {
    pub t_ticks: u64,
    pub center_x: f64,
    pub center_y: f64,
    /// Crop height as a share of the source height; width follows the aspect.
    pub scale: f64,
}

/// What a solve produced, including when it produced a fitted frame.
#[derive(Clone, Debug)]
pub struct CropPath {
    pub keyframes: Vec<Keyframe>,
    /// True when no track earned the frame and this is the fitted rectangle.
    pub fit: bool,
    pub fit_reason: Option<FitReason>,
    pub track_id: Option<u64>,
    /// Share of the followed face's frames that stayed inside the crop.
    /// Reported rather than asserted: a path can be optimal and still fail to
    /// contain a subject who left.
    pub containment: f64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SolveError {
    #[error("the span to solve over is empty")]
    EmptySpan,
    #[error("the requested aspect has a zero side")]
    DegenerateAspect,
    #[error("the crop path system could not be solved: {0}")]
    Banded(#[from] BandedError),
}

/// The crop that fills as much of the frame as the aspect allows, centred.
///
/// What a fitted frame is, and also the starting point every solved path is a
/// deviation from.
fn fitted(aspect_width: u32, aspect_height: u32, span: (u64, u64)) -> Vec<Keyframe> {
    let _ = (aspect_width, aspect_height);
    vec![
        Keyframe {
            t_ticks: span.0,
            center_x: 0.5,
            center_y: 0.5,
            scale: 1.0,
        },
        Keyframe {
            t_ticks: span.1,
            center_x: 0.5,
            center_y: 0.5,
            scale: 1.0,
        },
    ]
}

/// The samples of one track that fall inside a span, in time order.
fn samples(track: &Track, start: u64, end: u64) -> Vec<(u64, f64, f64, f64)> {
    let mut found: Vec<(u64, f64, f64, f64)> = track
        .boxes
        .iter()
        .filter(|box_| box_.t_ticks >= start && box_.t_ticks <= end)
        .map(|box_| {
            (
                box_.t_ticks,
                box_.x + box_.w / 2.0,
                box_.y + box_.h / 2.0,
                box_.h,
            )
        })
        .collect();
    found.sort_by_key(|(at, ..)| *at);
    found
}

/// Solve one axis: the pentadiagonal normal equations of the objective above.
///
/// `targets` is `c_t`. The returned path is the unconstrained optimum; the
/// caller applies the projections.
fn solve_axis(targets: &[f64], weights: Weights) -> Result<Vec<f64>, BandedError> {
    let count = targets.len();
    let mut matrix = Banded::new(count, 2);
    let mut rhs = vec![0.0_f64; count];

    // Subject: w_s (x_t − c_t)².
    for index in 0..count {
        matrix.add(index, index, 2.0 * weights.subject);
        rhs[index] += 2.0 * weights.subject * targets[index];
    }
    // Velocity: w_v (x_t − x_{t−1})².
    for index in 1..count {
        let weight = 2.0 * weights.velocity;
        matrix.add(index, index, weight);
        matrix.add(index - 1, index - 1, weight);
        matrix.add(index, index - 1, -weight);
    }
    // Acceleration: w_a (x_t − 2x_{t−1} + x_{t−2})², whose stencil is what
    // makes the band two wide.
    for index in 2..count {
        let weight = 2.0 * weights.acceleration;
        let stencil = [(index, 1.0), (index - 1, -2.0), (index - 2, 1.0)];
        // Only the lower triangle. The matrix stores one slot per symmetric
        // pair, so walking the full outer product would add every off-diagonal
        // entry twice and hand the factorization a matrix nobody assembled.
        for (row, left) in stencil {
            for (column, right) in stencil {
                if row >= column {
                    matrix.add(row, column, weight * left * right);
                }
            }
        }
    }
    matrix.solve(&rhs)
}

/// Hold the crop inside the frame.
///
/// A projection onto the feasible box rather than a constrained solve. The
/// targets handed to the solver are already clamped, so the optimum only leaves
/// the box by the overshoot the damping produces at the ends, and clamping there
/// costs a fraction of a pixel of optimality in exchange for never proposing a
/// crop that reads off the edge of the picture.
fn contain(path: &mut [f64], half_extent: f64) {
    let low = half_extent.min(0.5);
    let high = (1.0 - half_extent).max(0.5);
    for value in path.iter_mut() {
        *value = value.clamp(low, high);
    }
}

/// Hold the camera below a speed, forwards then backwards.
///
/// Two passes because one is not symmetric: limiting only forwards lets the
/// path arrive late and then jump, which is the lurch the limit exists to
/// prevent. Applied after the solve, so it is a projection like containment.
fn limit_speed(path: &mut [f64], times: &[u64], max_per_second: f64) {
    if max_per_second <= 0.0 || path.len() < 2 {
        return;
    }
    for index in 1..path.len() {
        let elapsed = seconds(times[index].saturating_sub(times[index - 1]));
        let allowed = max_per_second * elapsed;
        let delta = path[index] - path[index - 1];
        if delta.abs() > allowed {
            path[index] = path[index - 1] + delta.signum() * allowed;
        }
    }
    for index in (0..path.len() - 1).rev() {
        let elapsed = seconds(times[index + 1].saturating_sub(times[index]));
        let allowed = max_per_second * elapsed;
        let delta = path[index] - path[index + 1];
        if delta.abs() > allowed {
            path[index] = path[index + 1] + delta.signum() * allowed;
        }
    }
}

/// Reduce a dense path to the fewest keyframes that reproduce it.
///
/// Douglas–Peucker against the same linear interpolation the player will do, so
/// the tolerance means what it says: no interpolated point differs from the
/// solved path by more than this. A still camera reduces to two keyframes, which
/// is what makes an untracked clip cost nothing to store or to read.
fn simplify(dense: &[Keyframe], tolerance: f64) -> Vec<Keyframe> {
    if dense.len() <= 2 {
        return dense.to_vec();
    }
    let mut keep = vec![false; dense.len()];
    keep[0] = true;
    keep[dense.len() - 1] = true;
    let mut pending = vec![(0_usize, dense.len() - 1)];
    while let Some((first, last)) = pending.pop() {
        if last <= first + 1 {
            continue;
        }
        let span = seconds(dense[last].t_ticks - dense[first].t_ticks);
        let mut worst = 0.0_f64;
        let mut worst_at = first;
        for index in (first + 1)..last {
            let ratio = if span <= 0.0 {
                0.0
            } else {
                seconds(dense[index].t_ticks - dense[first].t_ticks) / span
            };
            let mix = |from: f64, to: f64| from + (to - from) * ratio;
            let error = (dense[index].center_x - mix(dense[first].center_x, dense[last].center_x))
                .abs()
                .max(
                    (dense[index].center_y - mix(dense[first].center_y, dense[last].center_y))
                        .abs(),
                )
                .max((dense[index].scale - mix(dense[first].scale, dense[last].scale)).abs());
            if error > worst {
                worst = error;
                worst_at = index;
            }
        }
        if worst > tolerance {
            keep[worst_at] = true;
            pending.push((first, worst_at));
            pending.push((worst_at, last));
        }
    }
    dense
        .iter()
        .zip(keep)
        .filter_map(|(frame, kept)| kept.then_some(*frame))
        .collect()
}

/// The largest keyframe error a reader would not notice, as a share of the
/// frame. Below a third of a percent, a crop edge moves less than a pixel on a
/// 1080-tall source.
const SIMPLIFY_TOLERANCE: f64 = 0.003;

/// How tall and wide the crop is at each sample.
///
/// Solved before the centre because it fixes the box the centre has to stay
/// inside, and smoothed the same way and for the same reason: a crop that
/// breathes is as distracting as one that lurches.
fn solve_extent(
    observed: &[(u64, f64, f64, f64)],
    aspect_width: u32,
    aspect_height: u32,
    weights: Weights,
) -> Result<(Vec<f64>, Vec<f64>), BandedError> {
    let targets: Vec<f64> = observed
        .iter()
        .map(|(_, _, _, face_height)| (face_height / MIN_FACE_FRACTION).clamp(0.2, 1.0))
        .collect();
    let zoom_weights = Weights {
        velocity: weights.zoom,
        acceleration: weights.zoom * 4.0,
        ..weights
    };
    let mut scale = solve_axis(&targets, zoom_weights)?;
    for value in &mut scale {
        *value = value.clamp(0.2, 1.0);
    }
    // Width as a share of the source width, given the requested aspect and the
    // source's own. A 9:16 crop from a 16:9 source at full height is 0.316 wide.
    let source_aspect = 16.0 / 9.0;
    let crop_aspect = f64::from(aspect_width) / f64::from(aspect_height);
    let widths = scale
        .iter()
        .map(|height| (height * crop_aspect / source_aspect).min(1.0))
        .collect();
    Ok((scale, widths))
}

/// Solve a crop path over one span.
pub fn solve(
    document: &VisionFaceTrack,
    start: u64,
    end: u64,
    aspect_width: u32,
    aspect_height: u32,
    weights: Weights,
    gate: FocusGate,
) -> Result<CropPath, SolveError> {
    if end <= start {
        return Err(SolveError::EmptySpan);
    }
    if aspect_width == 0 || aspect_height == 0 {
        return Err(SolveError::DegenerateAspect);
    }

    let focus = resolve(document, start, end, gate);
    let Focus::Track { track_id, .. } = focus else {
        let Focus::Fit { reason } = focus else {
            unreachable!("Focus has two variants and one was just matched")
        };
        return Ok(CropPath {
            keyframes: fitted(aspect_width, aspect_height, (start, end)),
            fit: true,
            fit_reason: Some(reason),
            track_id: None,
            containment: 0.0,
        });
    };

    let Some(track) = document
        .tracks
        .iter()
        .find(|candidate| candidate.track_id == track_id)
    else {
        return Ok(CropPath {
            keyframes: fitted(aspect_width, aspect_height, (start, end)),
            fit: true,
            fit_reason: Some(FitReason::NoneInSpan),
            track_id: None,
            containment: 0.0,
        });
    };

    let observed = samples(track, start, end);
    if observed.len() < 2 {
        return Ok(CropPath {
            keyframes: fitted(aspect_width, aspect_height, (start, end)),
            fit: true,
            fit_reason: Some(FitReason::TooIntermittent),
            track_id: None,
            containment: 0.0,
        });
    }

    let (scale, widths) = solve_extent(&observed, aspect_width, aspect_height, weights)?;

    let times: Vec<u64> = observed.iter().map(|(at, ..)| *at).collect();
    let mut x_targets: Vec<f64> = Vec::with_capacity(observed.len());
    let mut y_targets: Vec<f64> = Vec::with_capacity(observed.len());
    for (index, (_, cx, cy, _)) in observed.iter().enumerate() {
        let half_w = widths[index] / 2.0;
        let half_h = scale[index] / 2.0;
        x_targets.push(cx.clamp(half_w.min(0.5), (1.0 - half_w).max(0.5)));
        y_targets.push(cy.clamp(half_h.min(0.5), (1.0 - half_h).max(0.5)));
    }

    let mut xs = solve_axis(&x_targets, weights)?;
    let mut ys = solve_axis(&y_targets, weights)?;
    limit_speed(&mut xs, &times, weights.max_speed_per_second);
    limit_speed(&mut ys, &times, weights.max_speed_per_second);
    // Containment is applied last so nothing after it can push the crop back
    // out of the frame.
    let widest = widths.iter().copied().fold(0.0_f64, f64::max);
    let tallest = scale.iter().copied().fold(0.0_f64, f64::max);
    contain(&mut xs, widest / 2.0);
    contain(&mut ys, tallest / 2.0);

    // What the path is worth: the share of observed faces it actually holds.
    let mut held = 0_u32;
    for (index, (_, cx, cy, face_height)) in observed.iter().enumerate() {
        let half_w = widths[index] / 2.0;
        let half_h = scale[index] / 2.0;
        let face_half = face_height / 2.0;
        if (cx - xs[index]).abs() + face_half <= half_w + f64::EPSILON
            && (cy - ys[index]).abs() + face_half <= half_h + f64::EPSILON
        {
            held += 1;
        }
    }
    let containment = f64::from(held) / count(observed.len());

    let dense: Vec<Keyframe> = times
        .iter()
        .enumerate()
        .map(|(index, at)| Keyframe {
            t_ticks: *at,
            center_x: xs[index],
            center_y: ys[index],
            scale: scale[index],
        })
        .collect();

    Ok(CropPath {
        keyframes: simplify(&dense, SIMPLIFY_TOLERANCE),
        fit: false,
        fit_reason: None,
        track_id: Some(track_id),
        containment,
    })
}
