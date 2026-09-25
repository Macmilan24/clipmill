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

use clipmill_edit_ir::{
    CropEasing, CropKeyframe, CropRect, EditDocument, LayoutState, VideoSegment,
    crop_along_keyframes,
};
use std::ops::Range;

use crate::{
    plan::{RenderError, SourceInput},
    profile::{FONTS_DIR, RenderProfile},
    timing::{FrameRate, ticks_to_seconds},
};

/// Bounded resampler slack, immediately trimmed to the segment's allocation
/// on the global program frame grid. This never inserts extra program frames
/// or audio. A low-rate/VFR source holds its final available image until the
/// quantized boundary, as normal frame-rate conversion does.
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
    /// Phase of this segment’s first frame on the global program grid.
    pub video_offset_ticks: i64,
}

/// Consecutive sections over the same source window share one input decoder.
/// Their trims still select exact, separate ranges inside that input.
pub(crate) fn decoder_groups(spans: &[DecodeSpan]) -> Vec<Range<usize>> {
    let mut groups = Vec::new();
    let mut start = 0;
    for index in 1..=spans.len() {
        let same_decoder = index < spans.len()
            && spans[index].source_fingerprint == spans[index - 1].source_fingerprint
            && spans[index].seek_ticks == spans[index - 1].seek_ticks
            && spans[index].trim_start_ticks == spans[index - 1].trim_end_ticks;
        if !same_decoder {
            if start < index {
                groups.push(start..index);
            }
            start = index;
        }
    }
    groups
}

/// The crop rectangle a segment shows on one of its own frames.
///
/// This is the only sanctioned crop interpolation, and it lives with the
/// document: `clipmill_edit_ir::crop_along`, keyed here by frame. The
/// preview plan calls it; the emitted expression mirrors it; the
/// parity drill compares the two; and a trim evaluates the same arithmetic
/// in ticks to keep the crop at a new boundary. A second implementation
/// anywhere is a parity bug with a head start.
pub fn crop_rect_at(path: &[CropKeyframe], rate: FrameRate, frame: i64) -> Option<CropRect> {
    let by_frame: Vec<CropKeyframe> = path
        .iter()
        .map(|keyframe| CropKeyframe {
            t_ticks: rate.frame_at(keyframe.t_ticks),
            rect: keyframe.rect,
            easing: keyframe.easing,
        })
        .collect();
    crop_along_keyframes(&by_frame, frame)
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
    let mut video_spans = Vec::new();
    let mut audio_labels = Vec::new();

    let inputs = decoder_inputs(request);
    chains.extend(inputs.chains);
    let (video_inputs, audio_inputs) = (inputs.video, inputs.audio);

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
        if !request.audio_only && span.frame_count > 0 {
            let label = format!("v{index}");
            chains.push(format!(
                "{}trim=start={start}:end={end},setpts=PTS-STARTPTS-{offset}/TB,\
                 fps={num}/{den}:start_time=0,{TAIL_PAD},trim=end_frame={frames},setpts=PTS-STARTPTS,format=yuv420p[t{index}]",
                video_inputs[index],
                num = rate.num,
                den = rate.den,
                frames = span.frame_count,
                offset = ticks_to_seconds(span.video_offset_ticks),
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
            video_spans.push(span);
        }
        let label = format!("a{index}");
        if span.has_audio {
            chains.push(format!(
                "{}atrim=start={start}:end={end},asetpts=PTS-STARTPTS,\
                 aformat=sample_fmts=fltp:sample_rates={rate_hz}:channel_layouts=stereo[{label}]",
                audio_inputs[index],
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

    // Quantize boundaries once on the complete program. AV concat pads shorter
    // audio to each independently-rounded video span, accumulating silence and
    // losing the tail across many cuts; keep their clocks independent instead.
    chains.push(concat_chain(&audio_labels, false));
    if !request.audio_only {
        apply_transitions(
            request.document,
            rate,
            &video_spans,
            &mut video_labels,
            &mut chains,
        );
        chains.push(concat_chain(&video_labels, true));
    }

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
                format!("[vcat]subtitles=filename={file}:fontsdir={FONTS_DIR}[vout]")
            }
            None => "[vcat]null[vout]".to_owned(),
        };
        chains.push(burn);
    }

    Ok(FilterGraph {
        graph: chains.join(";"),
        video_label: "[vout]".to_owned(),
        audio_label: "[aout]".to_owned(),
    })
}

/// What each span reads its frames and samples from.
///
/// A span whose decoder is shared with its neighbours reads one output of a
/// `split`/`asplit` over that decoder; a span alone on its input reads the
/// input directly. The chains are the splits themselves.
struct DecoderInputs {
    video: Vec<String>,
    audio: Vec<String>,
    chains: Vec<String>,
}

fn decoder_inputs(request: &GraphRequest<'_>) -> DecoderInputs {
    let mut inputs = DecoderInputs {
        video: vec![String::new(); request.spans.len()],
        audio: vec![String::new(); request.spans.len()],
        chains: Vec::new(),
    };
    for (input_index, group) in decoder_groups(request.spans).iter().enumerate() {
        let video_members: Vec<usize> = group
            .clone()
            .filter(|&i| request.spans[i].frame_count > 0)
            .collect();
        if !request.audio_only && !video_members.is_empty() {
            if let [only] = video_members[..] {
                inputs.video[only] = format!("[{input_index}:v]");
            } else {
                let labels = video_members
                    .iter()
                    .map(|i| format!("[decode_v{i}]"))
                    .collect::<Vec<_>>()
                    .concat();
                inputs.chains.push(format!(
                    "[{input_index}:v]split={}{labels}",
                    video_members.len(),
                ));
                for i in video_members {
                    inputs.video[i] = format!("[decode_v{i}]");
                }
            }
        }
        if request.spans[group.start].has_audio {
            if group.len() == 1 {
                inputs.audio[group.start] = format!("[{input_index}:a]");
            } else {
                let labels = group
                    .clone()
                    .map(|i| format!("[decode_a{i}]"))
                    .collect::<Vec<_>>()
                    .concat();
                inputs
                    .chains
                    .push(format!("[{input_index}:a]asplit={}{labels}", group.len()));
                for i in group.clone() {
                    inputs.audio[i] = format!("[decode_a{i}]");
                }
            }
        }
    }
    inputs
}

fn concat_chain(labels: &[String], video: bool) -> String {
    format!(
        "[{}]concat=n={}:v={}:a={}[{}cat]",
        labels.join("]["),
        labels.len(),
        u8::from(video),
        u8::from(!video),
        if video { "v" } else { "a" },
    )
}

/// Blend a clone of the outgoing final composed frame over the incoming
/// picture. Padding is confined to the overlay branch; the main segment
/// allocation, audio concat and captions keep their original clocks.
fn apply_transitions(
    document: &EditDocument,
    rate: FrameRate,
    spans: &[&DecodeSpan],
    labels: &mut [String],
    chains: &mut Vec<String>,
) {
    for transition in crate::transitions::transitions(document, rate) {
        let Some(index) = spans
            .iter()
            .position(|span| span.segment_id == transition.incoming_segment_id)
        else {
            continue;
        };
        if index == 0 {
            continue;
        }
        let frames = transition.end_frame - transition.first_frame;
        let outgoing_frames = spans[index - 1].frame_count;
        chains.push(format!(
            "[{}]split=2[cut_keep_{index}][cut_tail_{index}]",
            labels[index - 1]
        ));
        chains.push(format!(
            "[cut_tail_{index}]trim=start_frame={last}:end_frame={outgoing_frames},\
             setpts=PTS-STARTPTS,tpad=stop_mode=clone:stop={padding},format=yuva420p,\
             fade=t=out:start_frame=0:nb_frames={frames}:alpha=1[cut_hold_{index}]",
            last = outgoing_frames - 1,
            padding = frames - 1,
        ));
        chains.push(format!(
            "[{}][cut_hold_{index}]overlay=eof_action=pass:repeatlast=0:format=yuv420[cut_mix_{index}]",
            labels[index]
        ));
        labels[index - 1] = format!("cut_keep_{index}");
        labels[index] = format!("cut_mix_{index}");
    }
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
            let crop = crop_filter(segment, source, profile, &segment.layout.crop_path, height)?;
            Ok(vec![format!(
                "[t{index}]{crop},scale={width}:{height},setsar=1,format=yuv420p[{label}]"
            )])
        }
        LayoutState::TwoUp => {
            let viewport_height = height / 2;
            let upper = crop_filter(
                segment,
                source,
                profile,
                &segment.layout.crop_path,
                viewport_height,
            )?;
            let lower = crop_filter(
                segment,
                source,
                profile,
                &segment.layout.secondary_crop_path,
                viewport_height,
            )?;
            Ok(vec![
                format!("[t{index}]split=2[t{index}upper][t{index}lower]"),
                format!(
                    "[t{index}upper]{upper},scale={width}:{viewport_height},setsar=1[t{index}u]"
                ),
                format!(
                    "[t{index}lower]{lower},scale={width}:{viewport_height},setsar=1[t{index}l]"
                ),
                format!("[t{index}u][t{index}l]vstack=inputs=2,format=yuv420p[{label}]"),
            ])
        }
    }
}

fn crop_filter(
    segment: &VideoSegment,
    source: &SourceInput,
    profile: &RenderProfile,
    path: &[CropKeyframe],
    viewport_height: i64,
) -> Result<String, RenderError> {
    let first = path
        .first()
        .ok_or_else(|| RenderError::SpeakerFillWithoutCropPath(segment.segment_id.clone()))?;
    let (crop_width, crop_height) = (first.rect.width, first.rect.height);
    let zooming = path
        .iter()
        .any(|keyframe| keyframe.rect.width != crop_width || keyframe.rect.height != crop_height);
    // An exactly 9:16 rectangle does not exist at every integer height — at
    // 1080 the ideal width is 607.5 — so the window is allowed to miss the
    // output's aspect by up to one source pixel, which scaling absorbs
    // invisibly. Anything wider than that is a framing mistake, not rounding,
    // and stretching faces to hide it would be the wrong kindness.
    if path.iter().any(|keyframe| {
        (keyframe.rect.width * viewport_height - keyframe.rect.height * profile.width).abs()
            > viewport_height
    }) {
        return Err(RenderError::CropAspectMismatch(segment.segment_id.clone()));
    }
    for keyframe in path {
        if keyframe.rect.x + keyframe.rect.width > source.width
            || keyframe.rect.y + keyframe.rect.height > source.height
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
    if zooming {
        // crop's w/h are fixed, so scale the source anew on each frame and
        // take a fixed output-sized window from it. Unlike zoompan, this
        // preserves the source rectangle's aspect instead of stretching a
        // landscape window into portrait. The final scale below is a no-op.
        let height = axis_expression(path, &frames, |rect| rect.height, "n");
        let x = axis_expression(path, &frames, |rect| rect.x, "n");
        let y = axis_expression(path, &frames, |rect| rect.y, "n");
        return Ok(format!(
            "scale=w='ceil(iw*{viewport_height}/({height})/2)*2':h='ceil(ih*{viewport_height}/({height})/2)*2':eval=frame,\
             crop=w={width}:h={viewport_height}:x='floor(({x})*{viewport_height}/({height}))':\
             y='floor(({y})*{viewport_height}/({height}))':exact=1",
            viewport_height = viewport_height,
            width = profile.width,
        ));
    }
    Ok(format!(
        "crop=w={crop_width}:h={crop_height}:x='{}':y='{}':exact=1",
        axis_expression(path, &frames, |rect| rect.x, "n"),
        axis_expression(path, &frames, |rect| rect.y, "n"),
    ))
}

/// A piecewise-linear expression over the output frame index, mirroring
/// [`crop_rect_at`] branch for branch.
fn axis_expression(
    path: &[CropKeyframe],
    frames: &[i64],
    axis: impl Fn(&CropRect) -> i64 + Copy,
    variable: &str,
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
            let ratio = format!("({variable}-{start})/({end}-{start})");
            let eased = match path[index].easing {
                CropEasing::Linear => ratio,
                CropEasing::EaseIn => format!("pow({ratio}\\,2)"),
                CropEasing::EaseOut => format!("(1-pow((1-{ratio})\\,2))"),
                CropEasing::EaseInOut => format!("({ratio}*{ratio}*(3-2*{ratio}))"),
            };
            format!("floor({from}+({to}-{from})*{eased})")
        };
        expression = format!("if(lt({variable}\\,{end})\\,{value}\\,{expression})");
    }
    let first_frame = frames.first().copied().unwrap_or(0);
    if first_frame > 0 {
        expression = format!(
            "if(lt({variable}\\,{first_frame})\\,{}\\,{expression})",
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
            easing: clipmill_edit_ir::CropEasing::Linear,
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
