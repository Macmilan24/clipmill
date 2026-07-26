//! The render plan: everything the executor needs, decided before anything
//! runs.
//!
//! Compilation is a pure function of the Edit IR, the resolved sources, and
//! the profile. That is what makes a render cacheable — the plan is derived
//! from the recipe's inputs, so an identical plan is an identical output and a
//! warm render is a lookup rather than a re-encode.

use std::collections::BTreeMap;

use clipmill_edit_ir::{CaptionAnimation, EditDocument, LayoutState};
use serde_json::{Value, json};
use thiserror::Error;

use crate::{
    graph::{self, DecodeSpan, FilterGraph, GraphRequest},
    profile::RenderProfile,
    subtitles::{self, CueWindow, unrenderable_character},
    timing::ticks_to_seconds,
};

/// Payload file names inside a `render.clip.v1` artifact.
pub const CLIP_FILE: &str = "clip.mp4";
pub const ASS_FILE: &str = "clip.ass";
pub const SRT_FILE: &str = "clip.srt";
pub const VTT_FILE: &str = "clip.vtt";
pub const MANIFEST_FILE: &str = "render-manifest.json";

/// A source the document's segments may reference, resolved to something the
/// decoder can open.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInput {
    /// `sha256:`-prefixed source fingerprint, matching the IR's segments.
    pub fingerprint: String,
    /// Absolute local path. Control-plane only: it never reaches a recipe, a
    /// manifest, or a log.
    pub path: String,
    pub width: i64,
    pub height: i64,
    pub has_audio: bool,
    /// Source duration in ticks. Zero means the observation did not state one,
    /// in which case spans are taken on trust rather than refused.
    pub duration_ticks: i64,
    /// Video keyframe positions in edit ticks, from the source's reference
    /// index. Empty means "seek from the start" rather than "guess".
    pub keyframe_ticks: Vec<i64>,
}

impl SourceInput {
    /// The last keyframe at or before `ticks` — the point a decoder can start
    /// from and still reproduce the requested frame exactly.
    fn seek_target(&self, ticks: i64) -> i64 {
        self.keyframe_ticks
            .iter()
            .copied()
            .filter(|keyframe| *keyframe <= ticks)
            .max()
            .unwrap_or(0)
            .max(0)
    }
}

/// Measured loudness from the first pass, fed to the second.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessMeasurement {
    pub input_lufs: f64,
    pub input_true_peak_dbtp: f64,
    pub input_range_lu: f64,
    pub input_threshold_lufs: f64,
    pub target_offset_lu: f64,
}

impl LoudnessMeasurement {
    /// Parse the JSON object `loudnorm=print_format=json` writes to stderr.
    /// Only these five numbers are read; the surrounding diagnostics may name
    /// local paths and are never retained.
    pub fn from_loudnorm_json(text: &str) -> Option<Self> {
        let start = text.rfind("\"input_i\"")?;
        let open = text[..start].rfind('{')?;
        let close = text[open..].find('}')? + open;
        let value: Value = serde_json::from_str(&text[open..=close]).ok()?;
        let number = |key: &str| -> Option<f64> { value[key].as_str()?.trim().parse().ok() };
        Some(Self {
            input_lufs: number("input_i")?,
            input_true_peak_dbtp: number("input_tp")?,
            input_range_lu: number("input_lra")?,
            input_threshold_lufs: number("input_thresh")?,
            target_offset_lu: number("target_offset")?,
        })
    }
}

/// One rendered segment, recorded so the manifest can state what was used.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentReport {
    pub segment_id: String,
    pub source_fingerprint: String,
    pub in_ticks: i64,
    pub out_ticks: i64,
    pub layout: String,
    pub frame_count: i64,
}

#[derive(Clone, Debug)]
pub struct RenderPlan {
    pub profile: RenderProfile,
    pub spans: Vec<DecodeSpan>,
    pub segments: Vec<SegmentReport>,
    pub graph: FilterGraph,
    pub measurement_graph: FilterGraph,
    pub ass: String,
    pub srt: String,
    pub vtt: String,
    pub cue_windows: Vec<CueWindow>,
    pub duration_ticks: i64,
    pub frame_count: i64,
    paths: BTreeMap<String, String>,
}

impl RenderPlan {
    /// The determinants of the output, for the artifact recipe. Two plans with
    /// equal configs must produce equal bytes, so everything that can change a
    /// pixel appears here — and nothing that cannot, which is why local paths
    /// are absent.
    pub fn recipe_config(&self) -> serde_json::Map<String, Value> {
        let mut config = serde_json::Map::new();
        config.insert(
            "algorithm".to_owned(),
            json!("clipmill.render.clip.compile.v1"),
        );
        config.insert(
            "profile".to_owned(),
            serde_json::to_value(&self.profile).unwrap_or(Value::Null),
        );
        config.insert("filter_graph".to_owned(), json!(self.graph.graph));
        config.insert(
            "measurement_graph".to_owned(),
            json!(self.measurement_graph.graph),
        );
        config.insert("frame_count".to_owned(), json!(self.frame_count));
        config.insert("duration_ticks".to_owned(), json!(self.duration_ticks));
        config.insert(
            "segments".to_owned(),
            json!(
                self.segments
                    .iter()
                    .map(|segment| json!({
                        "segment_id": segment.segment_id,
                        "source_fingerprint": segment.source_fingerprint,
                        "in_ticks": segment.in_ticks,
                        "out_ticks": segment.out_ticks,
                        "layout": segment.layout,
                    }))
                    .collect::<Vec<_>>()
            ),
        );
        config.insert("captions_ass".to_owned(), json!(self.ass));
        config
    }

    /// FFmpeg arguments for the loudness measurement pass. Audio only: the
    /// measurement never needed the pixels, and skipping the decode is most of
    /// what makes two passes affordable.
    pub fn measurement_args(&self) -> Vec<String> {
        let mut args = self.input_args();
        args.extend([
            "-filter_complex".to_owned(),
            self.measurement_graph.graph.clone(),
            "-map".to_owned(),
            self.measurement_graph.audio_label.clone(),
            "-f".to_owned(),
            "null".to_owned(),
            // loudnorm reports its measurement at info level.
            "-v".to_owned(),
            "info".to_owned(),
            "-".to_owned(),
        ]);
        args
    }

    /// FFmpeg arguments for the encode.
    ///
    /// The determinism flags are the profile's contract: one encoder thread so
    /// libx264's slice decisions cannot depend on scheduling, and bitexact on
    /// every layer so the container carries no build string and no clock.
    pub fn encode_args(&self, measurement: LoudnessMeasurement) -> Vec<String> {
        let graph = self.encode_graph(measurement);
        let profile = &self.profile;
        let mut args = self.input_args();
        args.extend([
            "-filter_complex".to_owned(),
            graph.graph,
            "-map".to_owned(),
            graph.video_label,
            "-map".to_owned(),
            graph.audio_label,
            "-c:v".to_owned(),
            profile.video_codec.clone(),
            "-preset".to_owned(),
            profile.preset.clone(),
            "-crf".to_owned(),
            profile.crf.to_string(),
            "-pix_fmt".to_owned(),
            profile.pixel_format.clone(),
            "-profile:v".to_owned(),
            "high".to_owned(),
            "-r".to_owned(),
            format!("{}/{}", profile.frame_rate.num, profile.frame_rate.den),
            "-frames:v".to_owned(),
            self.frame_count.to_string(),
            "-c:a".to_owned(),
            profile.audio_codec.clone(),
            "-b:a".to_owned(),
            profile.audio_bitrate.to_string(),
            "-ar".to_owned(),
            profile.audio_sample_rate.to_string(),
            "-ac".to_owned(),
            profile.audio_channels.to_string(),
            "-threads".to_owned(),
            "1".to_owned(),
            "-fflags".to_owned(),
            "+bitexact".to_owned(),
            "-flags:v".to_owned(),
            "+bitexact".to_owned(),
            "-flags:a".to_owned(),
            "+bitexact".to_owned(),
            "-map_metadata".to_owned(),
            "-1".to_owned(),
            "-video_track_timescale".to_owned(),
            profile.frame_rate.num.to_string(),
            "-movflags".to_owned(),
            "+faststart".to_owned(),
            "-f".to_owned(),
            "mp4".to_owned(),
            CLIP_FILE.to_owned(),
        ]);
        args
    }

    /// The plan's graph with the measured loudness substituted into its slot.
    fn encode_graph(&self, measurement: LoudnessMeasurement) -> FilterGraph {
        let loudness = &self.profile.loudness;
        let loudnorm = format!(
            "loudnorm=I={target}:TP={peak}:LRA={range}:measured_I={input_i:.6}:\
             measured_TP={input_tp:.6}:measured_LRA={input_lra:.6}:\
             measured_thresh={input_thresh:.6}:offset={offset:.6}:linear=true:print_format=summary,\
             aformat=sample_fmts=fltp:sample_rates={rate_hz}:channel_layouts=stereo",
            target = loudness.integrated_lufs,
            peak = loudness.true_peak_dbtp,
            range = loudness.range_lu,
            input_i = measurement.input_lufs,
            input_tp = measurement.input_true_peak_dbtp,
            input_lra = measurement.input_range_lu,
            input_thresh = measurement.input_threshold_lufs,
            offset = measurement.target_offset_lu,
            rate_hz = self.profile.audio_sample_rate,
        );
        let mut graph = self.graph.clone();
        graph.graph = graph.graph.replace(graph::LOUDNORM_SLOT, &loudnorm);
        graph
    }

    fn input_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        for span in &self.spans {
            let path = self
                .paths
                .get(&span.source_fingerprint)
                .cloned()
                .unwrap_or_default();
            if span.seek_ticks > 0 {
                args.push("-ss".to_owned());
                args.push(ticks_to_seconds(span.seek_ticks));
            }
            args.push("-i".to_owned());
            args.push(path);
        }
        args
    }
}

/// Everything about the caption track that must hold before an encoder is
/// asked to burn it in.
fn check_captions(document: &EditDocument, duration_ticks: i64) -> Result<(), RenderError> {
    for cue in &document.captions.cues {
        if cue.start_ticks >= duration_ticks {
            return Err(RenderError::CueOutsideProgram(cue.cue_id.clone()));
        }
        if matches!(cue.anim, CaptionAnimation::Karaoke) && cue.word_count() == 0 {
            return Err(RenderError::CueOutsideProgram(cue.cue_id.clone()));
        }
        for word in cue.words() {
            if let Some(character) = unrenderable_character(&word.text) {
                return Err(RenderError::UnrenderableCaptionText {
                    cue_id: cue.cue_id.clone(),
                    character,
                });
            }
        }
    }
    Ok(())
}

pub fn compile(
    document: &EditDocument,
    sources: &[SourceInput],
    profile: &RenderProfile,
) -> Result<RenderPlan, RenderError> {
    document.validate()?;
    if document.video.segments.is_empty() {
        return Err(RenderError::EmptyProgram);
    }
    if document.captions.style_ref != profile.caption_style.style_ref
        && !document.captions.cues.is_empty()
    {
        return Err(RenderError::UnknownCaptionStyle(
            document.captions.style_ref.clone(),
        ));
    }
    let rate = profile.rate();
    let duration_ticks = document.program_duration_ticks();
    check_captions(document, duration_ticks)?;

    let mut spans = Vec::with_capacity(document.video.segments.len());
    let mut segments = Vec::with_capacity(document.video.segments.len());
    let mut paths = BTreeMap::new();
    for segment in &document.video.segments {
        let source = sources
            .iter()
            .find(|source| source.fingerprint == segment.source_fingerprint)
            .ok_or_else(|| RenderError::UnresolvedSource(segment.source_fingerprint.clone()))?;
        if source.duration_ticks > 0 && segment.out_ticks > source.duration_ticks {
            // Catch this here rather than letting the encoder run for minutes
            // and then produce a file one frame-count check short of the plan.
            return Err(RenderError::SegmentPastEndOfSource(
                segment.segment_id.clone(),
            ));
        }
        let seek_ticks = source.seek_target(segment.in_ticks);
        spans.push(DecodeSpan {
            segment_id: segment.segment_id.clone(),
            source_fingerprint: segment.source_fingerprint.clone(),
            seek_ticks,
            trim_start_ticks: segment.in_ticks - seek_ticks,
            trim_end_ticks: segment.out_ticks - seek_ticks,
            has_audio: source.has_audio,
            frame_count: rate.frame_count(segment.duration_ticks()),
        });
        segments.push(SegmentReport {
            segment_id: segment.segment_id.clone(),
            source_fingerprint: segment.source_fingerprint.clone(),
            in_ticks: segment.in_ticks,
            out_ticks: segment.out_ticks,
            layout: match segment.layout.state {
                LayoutState::Fit => "fit",
                LayoutState::SpeakerFill => "speaker_fill",
            }
            .to_owned(),
            frame_count: rate.frame_count(segment.duration_ticks()),
        });
        paths.insert(segment.source_fingerprint.clone(), source.path.clone());
    }

    let burn = (!document.captions.cues.is_empty()).then_some(ASS_FILE);
    let graph = graph::build(&GraphRequest {
        document,
        profile,
        spans: &spans,
        sources,
        subtitle_file: burn,
        loudnorm: None,
        audio_only: false,
    })?;
    let measurement_graph = graph::build(&GraphRequest {
        document,
        profile,
        spans: &spans,
        sources,
        subtitle_file: None,
        loudnorm: Some(format!(
            "loudnorm=I={}:TP={}:LRA={}:print_format=json",
            profile.loudness.integrated_lufs,
            profile.loudness.true_peak_dbtp,
            profile.loudness.range_lu,
        )),
        audio_only: true,
    })?;

    Ok(RenderPlan {
        ass: subtitles::write_ass(&document.captions, profile),
        srt: subtitles::write_srt(&document.captions, rate),
        vtt: subtitles::write_vtt(&document.captions, rate),
        cue_windows: subtitles::cue_windows(&document.captions, rate),
        frame_count: rate.frame_count(duration_ticks),
        duration_ticks,
        profile: profile.clone(),
        spans,
        segments,
        graph,
        measurement_graph,
        paths,
    })
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("the edit document is not renderable: {0}")]
    Document(#[from] clipmill_edit_ir::DocumentError),
    #[error("an empty program has nothing to render")]
    EmptyProgram,
    #[error("no registered source matches fingerprint {0}")]
    UnresolvedSource(String),
    #[error("segment {0} reaches past the end of its source")]
    SegmentPastEndOfSource(String),
    #[error("the plan referenced segment {0}, which the document does not contain")]
    UnknownSegment(String),
    #[error("caption style {0} is not available to this render profile")]
    UnknownCaptionStyle(String),
    #[error("cue {0} starts after the program ends")]
    CueOutsideProgram(String),
    #[error("cue {cue_id} carries {character:?}, which cannot be rendered as caption text")]
    UnrenderableCaptionText { cue_id: String, character: char },
    #[error("segment {0} asks for speaker fill without a crop path")]
    SpeakerFillWithoutCropPath(String),
    #[error("segment {0} has a crop path that changes size; zooming is not supported")]
    ZoomingCropPath(String),
    #[error("segment {0} has a crop path whose aspect ratio is not the output's")]
    CropAspectMismatch(String),
    #[error("segment {0} has a crop rectangle reaching outside the source frame")]
    CropOutsideFrame(String),
    #[error("segment {0} has two crop keyframes on the same output frame")]
    CropKeyframesTooDense(String),
}
