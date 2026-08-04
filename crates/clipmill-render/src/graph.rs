//! Lowering the Edit IR to an FFmpeg filter graph.
//!
//! The graph is a *value*: the compiler produces it, the recipe pins it, and
//! the manifest states it. Nothing downstream may improvise a filter, because
//! a filter that is not in the recipe is a pixel change that does not change
//! the content address.
//!
//! Crop paths are the delicate part. The same integer interpolation runs in
//! Rust (for the preview plan and for tests) and inside the emitted
//! expression, so the rectangle on frame *n* is one number rather than two
//! that happen to agree. The expression uses `floor` and the Rust side uses
//! Euclidean division for the same reason: both round toward negative
//! infinity, and the operands are small enough that FFmpeg's double
//! arithmetic is exact.

use clipmill_edit_ir::{CropKeyframe, CropRect, EditDocument, LayoutState, VideoSegment};

use crate::{
    plan::{RenderError, SourceInput},
    profile::{FONTS_DIR, RenderProfile},
    timing::{FrameRate, ticks_to_seconds},
};

/// Holds the last frame so the encoder can always reach the planned count.
///
/// `-frames:v` is a cap, not a pad: FFmpeg stops at the number given and never
/// invents a frame to reach it. So the plan's count was only ever authoritative
/// when the graph happened to produce at least that many, which is true exactly
/// when `fps` is a no-op — that is, when the source is already at the render
/// target. It is not, for anything a phone or a screen recorder produced, and
/// resampling 30 to 30000/1001 yields fewer frames than the span asks for. The
/// render then refused its own output for being two frames short.
///
/// Padding rather than predicting: deriving the count the way `fps` derives it
/// would mean reimplementing FFmpeg's resampler in Rust and keeping it in step
/// across upgrades, and a prediction can drift where a constraint cannot.
///
/// This is unreachable whenever the graph already satisfies the count, because
/// `-frames:v` truncates before the pad is ever drawn from. So a source already
/// at the target rate encodes to the same bytes it did before this existed —
/// measured against the pinned encoder rather than argued, identical SHA-256
/// with and without it, which is what says the goldens do not move.
///
/// Where it sits is the safety of it, and
/// `the_tail_pad_holds_the_program_and_never_a_span` holds it there: a pad that
/// stops on end-of-input never stops, so one placed inside a span chain would
/// hold that span forever and `concat` would never reach the next.
///
/// Bounded at one second, and that bound is load-bearing rather than tidy. An
/// unbounded pad (`stop=-1`) deadlocks the graph as soon as audio is mapped
/// beside it: measured against the pinned encoder, the encode stalls at 396 of
/// the 495 frames it was asked for and never returns, which reaches the daemon
/// as a duration-scaled deadline rather than as anything a reader could place.
/// A second is far more than the shortfall can be — resampling 30 to
/// 30000/1001 loses a tenth of a percent, so even the 180-second ceiling on a
/// clip is about five frames — and a graph short by more than that is wrong in
/// a way the count should still refuse.
///
/// The cost, stated rather than hidden: a clip from a 30 fps source can end on
/// up to two cloned frames, about 67 ms of held final image.
const TAIL_PAD: &str = "tpad=stop_mode=clone:stop_duration=1";

/// One decode span: an input file pre-seeked to a keyframe, then trimmed
/// exactly. The keyframe comes from the source's reference index, so the
/// decoder starts at a point it can actually start at and the trim discards
/// the run-up (book ch. 12's exact seek).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodeSpan {
    pub segment_id: String,
    pub source_fingerprint: String,
    pub seek_ticks: i64,
    pub trim_start_ticks: i64,
    pub trim_end_ticks: i64,
    pub has_audio: bool,
    pub frame_count: i64,
}

/// The crop rectangle a segment shows on one of its own frames.
///
/// This is the only sanctioned crop interpolation. The preview plan (W24)
/// calls it; the emitted expression mirrors it; the parity drill compares the
/// two. A second implementation anywhere is a parity bug with a head start.
pub fn crop_rect_at(path: &[CropKeyframe], rate: FrameRate, frame: i64) -> Option<CropRect> {
    let first = path.first()?;
    let last = path.last()?;
    if frame <= rate.frame_at(first.t_ticks) {
        return Some(first.rect);
    }
    if frame >= rate.frame_at(last.t_ticks) {
        return Some(last.rect);
    }
    for pair in path.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        let start = rate.frame_at(before.t_ticks);
        let end = rate.frame_at(after.t_ticks);
        if frame < start || frame >= end || end <= start {
            continue;
        }
        let span = end - start;
        let offset = frame - start;
        return Some(CropRect {
            x: interpolate(before.rect.x, after.rect.x, offset, span),
            y: interpolate(before.rect.y, after.rect.y, offset, span),
            width: before.rect.width,
            height: before.rect.height,
        });
    }
    Some(last.rect)
}

fn interpolate(from: i64, to: i64, offset: i64, span: i64) -> i64 {
    if span <= 0 {
        return from;
    }
    from + ((to - from) * offset).div_euclid(span)
}

/// Where the second pass's loudness normalisation is substituted in.
///
/// The encode graph is compiled before the measurement runs, so it holds this
/// slot rather than a filter. Keeping it visible means the artifact recipe
/// pins the graph *without* the measured numbers — which is right, because the
/// measurement is derived from the same inputs and would otherwise have to be
/// known before the cache could be consulted.
pub const LOUDNORM_SLOT: &str = "@loudnorm@";

/// The complete `-filter_complex` value plus the labels its outputs carry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterGraph {
    pub graph: String,
    pub video_label: String,
    pub audio_label: String,
}

pub(crate) struct GraphRequest<'a> {
    pub document: &'a EditDocument,
    pub profile: &'a RenderProfile,
    pub spans: &'a [DecodeSpan],
    pub sources: &'a [SourceInput],
    /// Name of the ASS file in the working directory, when captions burn in.
    pub subtitle_file: Option<&'a str>,
    /// Loudness normalisation filter. The measurement pass knows its own; the
    /// encode pass cannot, because its arguments are what the measurement pass
    /// is for — so the encode graph carries [`LOUDNORM_SLOT`] until it does.
    pub loudnorm: Option<String>,
    /// Drop the video half of the graph entirely (the measurement pass only
    /// needs audio, and skipping the decode is most of its cost).
    pub audio_only: bool,
}

pub(crate) fn build(request: &GraphRequest<'_>) -> Result<FilterGraph, RenderError> {
    let rate = request.profile.rate();
    let mut chains: Vec<String> = Vec::new();
    let mut video_labels = Vec::new();
    let mut audio_labels = Vec::new();

    for (index, span) in request.spans.iter().enumerate() {
        let segment = request
            .document
            .video
            .segments
            .iter()
            .find(|segment| segment.segment_id == span.segment_id)
            .ok_or_else(|| RenderError::UnknownSegment(span.segment_id.clone()))?;
        let start = ticks_to_seconds(span.trim_start_ticks);
        let end = ticks_to_seconds(span.trim_end_ticks);
        if !request.audio_only {
            let label = format!("v{index}");
            chains.push(format!(
                "[{index}:v]trim=start={start}:end={end},setpts=PTS-STARTPTS,\
                 fps={num}/{den},format=yuv420p[t{index}]",
                num = rate.num,
                den = rate.den,
            ));
            let source = source_for(request.sources, &segment.source_fingerprint)?;
            chains.extend(layout_chains(
                segment,
                source,
                request.profile,
                index,
                &label,
            )?);
            video_labels.push(label);
        }
        let label = format!("a{index}");
        if span.has_audio {
            chains.push(format!(
                "[{index}:a]atrim=start={start}:end={end},asetpts=PTS-STARTPTS,\
                 aformat=sample_fmts=fltp:sample_rates={rate_hz}:channel_layouts=stereo[{label}]",
                rate_hz = request.profile.audio_sample_rate,
            ));
        } else {
            // A silent source still occupies its span; generating silence keeps
            // the concat aligned instead of shortening the program.
            chains.push(format!(
                "anullsrc=r={rate_hz}:cl=stereo,atrim=end={duration},asetpts=PTS-STARTPTS,\
                 aformat=sample_fmts=fltp[{label}]",
                rate_hz = request.profile.audio_sample_rate,
                duration = ticks_to_seconds(span.trim_end_ticks - span.trim_start_ticks),
            ));
        }
        audio_labels.push(label);
    }

    if audio_labels.is_empty() {
        return Err(RenderError::EmptyProgram);
    }

    let count = audio_labels.len();
    let concat_inputs = (0..count)
        .map(|index| {
            let video = video_labels
                .get(index)
                .map_or_else(String::new, |label| format!("[{label}]"));
            format!("{video}[{}]", audio_labels[index])
        })
        .collect::<Vec<_>>()
        .concat();
    let video_flag = i32::from(!request.audio_only);
    let concat_outputs = if request.audio_only {
        "[acat]".to_owned()
    } else {
        "[vcat][acat]".to_owned()
    };
    chains.push(format!(
        "{concat_inputs}concat=n={count}:v={video_flag}:a=1{concat_outputs}"
    ));

    let mut audio_chain = vec!["[acat]".to_owned()];
    if let Some(gain) = gain_filter(request.document) {
        audio_chain.push(gain);
        audio_chain.push(",".to_owned());
    }
    audio_chain.push(
        request
            .loudnorm
            .clone()
            .unwrap_or_else(|| LOUDNORM_SLOT.to_owned()),
    );
    audio_chain.push("[aout]".to_owned());
    chains.push(audio_chain.concat());

    if !request.audio_only {
        let burn = match request.subtitle_file {
            // libass sees exactly one directory holding exactly one pinned
            // font, so the render cannot pick up whatever the host installed.
            Some(file) => {
                format!("[vcat]subtitles=filename={file}:fontsdir={FONTS_DIR},{TAIL_PAD}[vout]")
            }
            None => format!("[vcat]{TAIL_PAD}[vout]"),
        };
        chains.push(burn);
    }

    Ok(FilterGraph {
        graph: chains.join(";"),
        video_label: "[vout]".to_owned(),
        audio_label: "[aout]".to_owned(),
    })
}

fn source_for<'a>(
    sources: &'a [SourceInput],
    fingerprint: &str,
) -> Result<&'a SourceInput, RenderError> {
    sources
        .iter()
        .find(|source| source.fingerprint == fingerprint)
        .ok_or_else(|| RenderError::UnresolvedSource(fingerprint.to_owned()))
}

/// The chain that turns one decoded, CFR-normalised segment into an output
/// frame: either a followed crop or a letterbox over its own blurred fill.
fn layout_chains(
    segment: &VideoSegment,
    source: &SourceInput,
    profile: &RenderProfile,
    index: usize,
    label: &str,
) -> Result<Vec<String>, RenderError> {
    let (width, height) = (profile.width, profile.height);
    match segment.layout.state {
        LayoutState::Fit => Ok(vec![
            format!("[t{index}]split=2[t{index}bg][t{index}fg]"),
            format!(
                "[t{index}bg]scale={width}:{height}:force_original_aspect_ratio=increase,\
                 crop={width}:{height},gblur=sigma={sigma}[t{index}bgb]",
                sigma = profile.fit_background_sigma,
            ),
            format!(
                "[t{index}fg]scale={width}:{height}:force_original_aspect_ratio=decrease\
                 [t{index}fgs]"
            ),
            format!(
                "[t{index}bgb][t{index}fgs]overlay=x=(W-w)/2:y=(H-h)/2,setsar=1,\
                 format=yuv420p[{label}]"
            ),
        ]),
        LayoutState::SpeakerFill => {
            let crop = crop_filter(segment, source, profile)?;
            Ok(vec![format!(
                "[t{index}]{crop},scale={width}:{height},setsar=1,format=yuv420p[{label}]"
            )])
        }
    }
}

fn crop_filter(
    segment: &VideoSegment,
    source: &SourceInput,
    profile: &RenderProfile,
) -> Result<String, RenderError> {
    let path = &segment.layout.crop_path;
    let first = path
        .first()
        .ok_or_else(|| RenderError::SpeakerFillWithoutCropPath(segment.segment_id.clone()))?;
    let (crop_width, crop_height) = (first.rect.width, first.rect.height);
    if path
        .iter()
        .any(|keyframe| keyframe.rect.width != crop_width || keyframe.rect.height != crop_height)
    {
        // A crop window that changes size mid-segment is a zoom. FFmpeg
        // evaluates crop's width and height once per configuration, so a zoom
        // cannot be expressed here honestly; Phase 2 owns it.
        return Err(RenderError::ZoomingCropPath(segment.segment_id.clone()));
    }
    // An exactly 9:16 rectangle does not exist at every integer height — at
    // 1080 the ideal width is 607.5 — so the window is allowed to miss the
    // output's aspect by up to one source pixel, which scaling absorbs
    // invisibly. Anything wider than that is a framing mistake, not rounding,
    // and stretching faces to hide it would be the wrong kindness.
    let aspect_error = (crop_width * profile.height - crop_height * profile.width).abs();
    if aspect_error > profile.height {
        return Err(RenderError::CropAspectMismatch(segment.segment_id.clone()));
    }
    for keyframe in path {
        if keyframe.rect.x + crop_width > source.width
            || keyframe.rect.y + crop_height > source.height
        {
            return Err(RenderError::CropOutsideFrame(segment.segment_id.clone()));
        }
    }
    let rate = profile.rate();
    let mut frames = Vec::with_capacity(path.len());
    for keyframe in path {
        let frame = rate.frame_at(keyframe.t_ticks);
        if frames.last().is_some_and(|last| *last == frame) {
            return Err(RenderError::CropKeyframesTooDense(
                segment.segment_id.clone(),
            ));
        }
        frames.push(frame);
    }
    Ok(format!(
        "crop=w={crop_width}:h={crop_height}:x='{}':y='{}'",
        axis_expression(path, &frames, |rect| rect.x),
        axis_expression(path, &frames, |rect| rect.y),
    ))
}

/// A piecewise-linear expression over the output frame index, mirroring
/// [`crop_rect_at`] branch for branch.
fn axis_expression(
    path: &[CropKeyframe],
    frames: &[i64],
    axis: impl Fn(&CropRect) -> i64 + Copy,
) -> String {
    let Some(last) = path.last() else {
        return "0".to_owned();
    };
    let mut expression = axis(&last.rect).to_string();
    for index in (0..path.len().saturating_sub(1)).rev() {
        let (from, to) = (axis(&path[index].rect), axis(&path[index + 1].rect));
        let (start, end) = (frames[index], frames[index + 1]);
        let value = if from == to {
            from.to_string()
        } else {
            format!("floor({from}+({to}-{from})*(n-{start})/({end}-{start}))")
        };
        expression = format!("if(lt(n\\,{end})\\,{value}\\,{expression})");
    }
    let first_frame = frames.first().copied().unwrap_or(0);
    if first_frame > 0 {
        expression = format!(
            "if(lt(n\\,{first_frame})\\,{}\\,{expression})",
            axis(&path[0].rect)
        );
    }
    expression
}

/// The program-time gain curve as a `volume` expression, or nothing when the
/// document carries no automation.
fn gain_filter(document: &EditDocument) -> Option<String> {
    let curve = &document.audio.gain_curve;
    let last = curve.last()?;
    let mut expression = format_db(last.gain_db);
    for index in (0..curve.len().saturating_sub(1)).rev() {
        let (before, after) = (curve[index], curve[index + 1]);
        let start = ticks_to_seconds(before.t_ticks);
        let end = ticks_to_seconds(after.t_ticks);
        let value = if before.gain_db.to_bits() == after.gain_db.to_bits() {
            format_db(before.gain_db)
        } else {
            format!(
                "({from}+({to}-{from})*(t-{start})/({end}-{start}))",
                from = format_db(before.gain_db),
                to = format_db(after.gain_db),
            )
        };
        expression = format!("if(lt(t\\,{end})\\,{value}\\,{expression})");
    }
    let first = curve.first()?;
    if first.t_ticks > 0 {
        expression = format!(
            "if(lt(t\\,{})\\,{}\\,{expression})",
            ticks_to_seconds(first.t_ticks),
            format_db(first.gain_db),
        );
    }
    Some(format!(
        "volume=eval=frame:volume='pow(10\\,({expression})/20)'"
    ))
}

/// Decibels with fixed precision, so the same curve always writes the same
/// expression and therefore hashes to the same recipe.
fn format_db(value: f64) -> String {
    format!("{value:.4}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_edit_ir::{CropKeyframe, CropRect};

    use super::{crop_rect_at, format_db};
    use crate::timing::FrameRate;

    const RATE: FrameRate = FrameRate::NTSC_30;
    const FRAME_TICKS: i64 = 3_003;

    fn keyframe(frame: i64, x: i64) -> CropKeyframe {
        CropKeyframe {
            t_ticks: frame * FRAME_TICKS,
            rect: CropRect {
                x,
                y: 0,
                width: 608,
                height: 1_080,
            },
        }
    }

    #[test]
    fn a_crop_path_holds_before_and_after_its_ends() {
        let path = vec![keyframe(10, 100), keyframe(20, 200)];
        assert_eq!(crop_rect_at(&path, RATE, 0).expect("rect").x, 100);
        assert_eq!(crop_rect_at(&path, RATE, 10).expect("rect").x, 100);
        assert_eq!(crop_rect_at(&path, RATE, 20).expect("rect").x, 200);
        assert_eq!(crop_rect_at(&path, RATE, 99).expect("rect").x, 200);
    }

    #[test]
    fn a_crop_path_interpolates_by_whole_pixels() {
        let path = vec![keyframe(0, 0), keyframe(10, 100)];
        for frame in 0..=10 {
            assert_eq!(
                crop_rect_at(&path, RATE, frame).expect("rect").x,
                frame * 10
            );
        }
    }

    /// Interpolation must floor in both directions of travel, because the
    /// emitted expression uses `floor` and the two have to agree exactly.
    #[test]
    fn interpolation_floors_when_moving_backwards() {
        let path = vec![keyframe(0, 100), keyframe(3, 0)];
        assert_eq!(crop_rect_at(&path, RATE, 1).expect("rect").x, 66);
        assert_eq!(crop_rect_at(&path, RATE, 2).expect("rect").x, 33);
    }

    #[test]
    fn an_empty_path_has_no_rectangle() {
        assert!(crop_rect_at(&[], RATE, 0).is_none());
    }

    #[test]
    fn decibels_format_stably() {
        assert_eq!(format_db(-14.0), "-14.0000");
        assert_eq!(format_db(0.5), "0.5000");
    }
}
