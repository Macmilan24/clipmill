//! The render task: an Edit IR snapshot in, a publishable clip out.
//!
//! The compiler in `clipmill-render` decides everything (book ch. 17); this
//! module is the part that touches the world. It resolves the sources the
//! document names, stages the one font libass may see, runs the two FFmpeg
//! passes, and then *verifies the result rather than asserting it* — frame
//! count, stream shape, and loudness are re-read from the finished file
//! before anything is published, because a manifest that reports what the
//! encoder was asked for is not evidence of what it did.
//!
//! Rendering is model-free by construction. Nothing here loads a model,
//! reaches the network, or reads a clock.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, StagingArea, Timebase,
};
use clipmill_contracts::proto::ipc::v1::RenderClipPayloadV1;
use clipmill_core::{ArtifactId, Sha256Digest};
use clipmill_edit_ir::EditDocument;
use clipmill_render::{
    ASS_FILE, AiUseSummary, CLIP_FILE, CaptionWindow, EngineIdentity, LoudnessMeasurement,
    LoudnessReport, MANIFEST_FILE, MANIFEST_SCHEMA_VERSION, MeasuredLoudness, OutputFile,
    ProgramReport, ProgramSegment, RenderManifest, RenderPlan, RenderProfile, RightsAttestation,
    SRT_FILE, SourceInput, VTT_FILE,
};
use prost::Message;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    artifacts::ArtifactHandle,
    db::DbHandle,
    jobs::{LeasedTask, TaskExecutionError},
    media::{
        FFMPEG_BOM, FfmpegSpec, MediaError, MediaRunner, Prepared, ProgressSlot, abandon_staging,
        artifact_path, commit_staging, prepare_or_hit, read_descriptor, ticks_to_millis,
        verified_input_file, write_canonical_json,
    },
    sources::{SourceInspector, SourceProbeError},
};

pub(crate) const KIND_RENDER_CLIP: &str = "render-clip";
/// The disclosure vocabulary Phase 1 accepts. An unrecognised token is refused
/// rather than passed through: a manifest is a rights document, and a typo in
/// one is a false statement about the work.
pub(crate) const AI_ASSISTANCE_VOCABULARY: [&str; 4] =
    ["asr_captions", "reframe", "denoise", "silence_removal"];
const RENDER_BUDGET_BYTES: u64 = 16 * 1024 * 1024 * 1024;
/// Every render output except the manifest, which cannot contain its own hash.
const PUBLISHED_FILES: [&str; 4] = [CLIP_FILE, ASS_FILE, SRT_FILE, VTT_FILE];

pub(crate) fn is_render_kind(kind: &str) -> bool {
    kind == KIND_RENDER_CLIP
}

pub(crate) fn ai_assistance_is_known(token: &str) -> bool {
    AI_ASSISTANCE_VOCABULARY.contains(&token)
}

pub(crate) struct RenderContext<'a> {
    pub database: &'a DbHandle,
    pub artifacts: &'a ArtifactHandle,
    pub media: &'a MediaRunner,
    pub sources: &'a SourceInspector,
    pub fonts_dir: &'a Path,
}

pub(crate) async fn execute_render_task(
    context: &RenderContext<'_>,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    let payload = RenderClipPayloadV1::decode(task.payload.as_slice())
        .map_err(|_| TaskExecutionError::deterministic("render payload is not decodable"))?;
    let ir_artifact_id = payload
        .ir_artifact_id
        .parse::<ArtifactId>()
        .map_err(|_| TaskExecutionError::deterministic("render payload names no edit snapshot"))?;

    // The document is read from an immutable snapshot, never from the live
    // document, so an edit in flight cannot change what this render produces.
    let (lease, _) = verified_input_file(context.artifacts, ir_artifact_id, "edit-ir.json").await?;
    let projection = read_descriptor(&lease, "edit-ir.json")?;
    let document_bytes = serde_json_canonicalizer::to_vec(&projection)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let document = EditDocument::from_canonical_json(&document_bytes)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let document_digest = Sha256Digest::from_bytes(Sha256::digest(&document_bytes).into());
    drop(lease);

    let font = stage_font_source(context.fonts_dir)?;
    let inputs = resolve_sources(context, &task.project_id, &document).await?;
    let profile = RenderProfile::default();
    let plan = clipmill_render::compile(&document, &inputs, &profile)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;

    let ir_hash = format!("sha256:{document_digest}");
    let recipe = render_recipe(
        task,
        document_digest,
        &plan,
        &font,
        &payload,
        ir_artifact_id,
    )?;
    let staging = match prepare_or_hit(context.artifacts, recipe).await? {
        // A warm render is a lookup. Nothing is decoded, nothing is encoded.
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let result = render_into(
        context,
        &staging,
        &Rendered {
            plan: &plan,
            font: &font,
            payload: &payload,
            ir_artifact_id,
            ir_hash,
        },
        progress,
    )
    .await;
    match result {
        Ok(paths) => commit_staging(context.artifacts, staging_id, paths).await,
        Err(error) => {
            abandon_staging(context.artifacts, staging_id).await;
            Err(error)
        }
    }
}

/// The one font libass is permitted to resolve, with the digest the manifest
/// records as part of the engine's identity.
struct PinnedFont {
    path: PathBuf,
    file_name: String,
    family: String,
    sha256: String,
}

fn stage_font_source(fonts_dir: &Path) -> Result<PinnedFont, TaskExecutionError> {
    let family = clipmill_render::FONT_FAMILY;
    let file_name = format!("{family}-Bold.ttf");
    let path = fonts_dir.join(&file_name);
    let bytes = fs::read(&path).map_err(|_| {
        TaskExecutionError::deterministic(
            "the pinned caption font is not installed; run ./tools/fetch-ffmpeg.sh",
        )
    })?;
    Ok(PinnedFont {
        path,
        file_name,
        family: family.to_owned(),
        sha256: format!(
            "sha256:{}",
            Sha256Digest::from_bytes(Sha256::digest(&bytes).into())
        ),
    })
}

/// Resolve every source the document names to something the decoder can open.
///
/// Each observation is re-verified first: a document may have been authored
/// weeks ago, and rendering a file that has changed underneath it would
/// produce a clip whose manifest describes footage that no longer exists.
async fn resolve_sources(
    context: &RenderContext<'_>,
    project_id: &str,
    document: &EditDocument,
) -> Result<Vec<SourceInput>, TaskExecutionError> {
    let mut wanted = document
        .video
        .segments
        .iter()
        .map(|segment| segment.source_fingerprint.clone())
        .collect::<Vec<_>>();
    wanted.sort();
    wanted.dedup();

    let registered = context
        .database
        .list_sources(project_id.to_owned())
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let mut inputs = Vec::with_capacity(wanted.len());
    for fingerprint in wanted {
        let record = registered
            .iter()
            .find(|record| record.source_fingerprint == fingerprint)
            .ok_or_else(|| {
                TaskExecutionError::deterministic(
                    "the document names a source this project has not registered",
                )
            })?;
        context
            .sources
            .verify(&record.observation)
            .await
            .map_err(|error| match error {
                SourceProbeError::SourceChanged => {
                    TaskExecutionError::deterministic("SOURCE_CHANGED")
                }
                SourceProbeError::InvalidPath(_) | SourceProbeError::ProbeFailed(_) => {
                    TaskExecutionError::deterministic(error.to_string())
                }
                _ => TaskExecutionError::transient(error.to_string()),
            })?;
        let map: Value = serde_json::from_slice(&record.source_map_json)
            .map_err(|_| TaskExecutionError::deterministic("source map is not valid JSON"))?;
        let (width, height, has_audio) = frame_shape(&map)?;
        inputs.push(SourceInput {
            fingerprint,
            path: record.observation.absolute_path.clone(),
            width,
            height,
            has_audio,
            keyframe_ticks: keyframe_ticks(context, &record.source_id).await,
        });
    }
    Ok(inputs)
}

/// Display dimensions of the first video stream, and whether audio exists.
fn frame_shape(map: &Value) -> Result<(i64, i64, bool), TaskExecutionError> {
    let streams = map["streams"].as_array().map_or(&[][..], Vec::as_slice);
    let video = streams
        .iter()
        .find(|stream| stream["kind"] == "video")
        .ok_or_else(|| {
            TaskExecutionError::deterministic("the source carries no video to render")
        })?;
    let dimension = |primary: &str, fallback: &str| -> i64 {
        video["video"][primary]
            .as_i64()
            .or_else(|| video["video"][fallback].as_i64())
            .unwrap_or(0)
    };
    let width = dimension("display_width", "coded_width");
    let height = dimension("display_height", "coded_height");
    if width <= 0 || height <= 0 {
        return Err(TaskExecutionError::deterministic(
            "the source map states no usable frame size",
        ));
    }
    let has_audio = streams.iter().any(|stream| stream["kind"] == "audio");
    Ok((width, height, has_audio))
}

/// Video keyframe positions from the source's reference index, when it has
/// one. A source that was never ingested simply decodes from its start: a
/// missing index costs seek time, not correctness.
async fn keyframe_ticks(context: &RenderContext<'_>, source_id: &str) -> Vec<i64> {
    let Ok(Some(manifest_id)) = context
        .database
        .latest_source_job_artifact(source_id.to_owned(), "ingest-source".to_owned())
        .await
    else {
        return Vec::new();
    };
    let Ok(manifest_id) = manifest_id.parse::<ArtifactId>() else {
        return Vec::new();
    };
    let Ok((lease, _)) =
        verified_input_file(context.artifacts, manifest_id, "ingest-manifest.json").await
    else {
        return Vec::new();
    };
    let Ok(manifest) = read_descriptor(&lease, "ingest-manifest.json") else {
        return Vec::new();
    };
    let index_id = manifest["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|child| child["kind"] == "media.reference_index.v1")
        .and_then(|child| child["artifact_id"].as_str())
        .and_then(|value| value.parse::<ArtifactId>().ok());
    drop(lease);
    let Some(index_id) = index_id else {
        return Vec::new();
    };
    let Ok((lease, _)) =
        verified_input_file(context.artifacts, index_id, "reference-index.json").await
    else {
        return Vec::new();
    };
    let Ok(index) = read_descriptor(&lease, "reference-index.json") else {
        return Vec::new();
    };
    let mut ticks = index["video_keyframes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|keyframe| keyframe["pts_ticks"].as_i64())
        .collect::<Vec<_>>();
    ticks.sort_unstable();
    ticks.dedup();
    ticks
}

/// The render's artifact identity.
///
/// The plan's own determinants carry the pixels; the font digest and FFmpeg
/// identity carry the substrate. The rights attestation and disclosure are
/// here too — not because they change a frame, but because they are published
/// *inside* the artifact, and two different attestations must not collide on
/// one content address.
fn render_recipe(
    task: &LeasedTask,
    document_digest: Sha256Digest,
    plan: &RenderPlan,
    font: &PinnedFont,
    payload: &RenderClipPayloadV1,
    ir_artifact_id: ArtifactId,
) -> Result<ArtifactRecipe, TaskExecutionError> {
    let mut config = plan.recipe_config();
    config.insert("ffmpeg_bom".to_owned(), json!(FFMPEG_BOM));
    config.insert(
        "font".to_owned(),
        json!({"family": font.family, "sha256": font.sha256}),
    );
    config.insert(
        "rights".to_owned(),
        json!({
            "source_attestation": payload.source_attestation,
            "gates_passed": payload.gates_passed,
        }),
    );
    config.insert("ai_assistance".to_owned(), json!(payload.ai_assistance));
    ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: task.output_kind.clone(),
        source_fingerprint: document_digest,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest: None,
        },
        inputs: vec![ir_artifact_id],
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.render.clip.v1".to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))
}

/// Everything the staging half of a render needs, gathered so the signature
/// stays readable.
struct Rendered<'a> {
    plan: &'a RenderPlan,
    font: &'a PinnedFont,
    payload: &'a RenderClipPayloadV1,
    ir_artifact_id: ArtifactId,
    ir_hash: String,
}

async fn render_into(
    context: &RenderContext<'_>,
    staging: &StagingArea,
    rendered: &Rendered<'_>,
    progress: &ProgressSlot,
) -> Result<Vec<ArtifactPath>, TaskExecutionError> {
    let (plan, font) = (rendered.plan, rendered.font);
    let work = staging.path().to_path_buf();
    // libass reads the subtitle file and the font directory relative to the
    // working directory FFmpeg is given, so the graph never carries a path
    // and there is nothing to escape.
    let fonts_dir = work.join(clipmill_render::FONTS_DIR);
    fs::create_dir_all(&fonts_dir)
        .and_then(|()| fs::copy(&font.path, fonts_dir.join(&font.file_name)).map(|_| ()))
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    write_text(staging, ASS_FILE, &plan.ass)?;

    let duration_hint = ticks_to_millis(plan.duration_ticks);
    let measurement_report = context
        .media
        .run_ffmpeg(
            ffmpeg_spec(plan.measurement_args(), &work, duration_hint),
            progress.clone(),
        )
        .await
        .map_err(MediaError::into_task_error)?;
    let measured_input =
        LoudnessMeasurement::from_loudnorm_json(&measurement_report).ok_or_else(|| {
            TaskExecutionError::deterministic("the loudness measurement pass reported nothing")
        })?;

    let _encode = context
        .media
        .run_ffmpeg(
            ffmpeg_spec(plan.encode_args(measured_input), &work, duration_hint),
            progress.clone(),
        )
        .await
        .map_err(MediaError::into_task_error)?;

    let probed = probe_output(context, &work).await?;
    verify_output(plan, &probed)?;
    let measured_output = measure_output(context, &work, duration_hint, progress).await?;

    write_text(staging, SRT_FILE, &plan.srt)?;
    write_text(staging, VTT_FILE, &plan.vtt)?;

    // The font was an input to the render, not part of it. Removing it before
    // the manifest is written keeps the artifact to what was published.
    fs::remove_dir_all(&fonts_dir)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;

    let manifest = build_manifest(rendered, measured_input, measured_output, &work)?;
    let manifest_value = serde_json::to_value(&manifest)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    write_canonical_json(staging, &artifact_path(MANIFEST_FILE)?, &manifest_value)?;

    let mut paths = Vec::with_capacity(PUBLISHED_FILES.len() + 1);
    for name in PUBLISHED_FILES {
        paths.push(artifact_path(name)?);
    }
    paths.push(artifact_path(MANIFEST_FILE)?);
    Ok(paths)
}

fn ffmpeg_spec(args: Vec<String>, work: &Path, duration_hint_millis: u64) -> FfmpegSpec {
    FfmpegSpec {
        args: args.into_iter().map(Into::into).collect(),
        output_dir: work.to_path_buf(),
        duration_hint_millis,
        max_output_bytes: RENDER_BUDGET_BYTES,
        capture_stderr: true,
    }
}

fn write_text(staging: &StagingArea, name: &str, text: &str) -> Result<(), TaskExecutionError> {
    use std::io::Write;
    let mut file = staging
        .create_file(&artifact_path(name)?)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|error| TaskExecutionError::transient(error.to_string()))
}

async fn probe_output(
    context: &RenderContext<'_>,
    work: &Path,
) -> Result<Value, TaskExecutionError> {
    context
        .media
        .run_ffprobe_json(
            work.join(CLIP_FILE),
            ["-show_format", "-show_streams"]
                .into_iter()
                .map(Into::into)
                .collect(),
        )
        .await
        .map_err(MediaError::into_task_error)
}

/// Re-read the finished file and refuse to publish one that does not match
/// the plan. The joiner verifies, not hopes (book ch. 17).
fn verify_output(plan: &RenderPlan, probed: &Value) -> Result<(), TaskExecutionError> {
    let streams = probed["streams"]
        .as_array()
        .ok_or_else(|| TaskExecutionError::deterministic("the render produced no streams"))?;
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .ok_or_else(|| TaskExecutionError::deterministic("the render produced no video stream"))?;
    let profile = &plan.profile;
    if video["width"].as_i64() != Some(profile.width)
        || video["height"].as_i64() != Some(profile.height)
    {
        return Err(TaskExecutionError::deterministic(
            "the render is not the profile's frame size",
        ));
    }
    let expected_rate = format!("{}/{}", profile.frame_rate.num, profile.frame_rate.den);
    if video["r_frame_rate"].as_str() != Some(expected_rate.as_str()) {
        return Err(TaskExecutionError::deterministic(
            "the render is not at the profile's frame rate",
        ));
    }
    let frames = video["nb_frames"]
        .as_str()
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| video["nb_frames"].as_i64());
    if let Some(frames) = frames
        && frames != plan.frame_count
    {
        return Err(TaskExecutionError::deterministic(
            "the render does not carry the frame count the plan pinned",
        ));
    }
    if !streams.iter().any(|stream| stream["codec_type"] == "audio") {
        return Err(TaskExecutionError::deterministic(
            "the render produced no audio stream",
        ));
    }
    Ok(())
}

/// Measure the finished file with the same filter that measured its input, so
/// the manifest's output loudness is evidence rather than the target restated.
async fn measure_output(
    context: &RenderContext<'_>,
    work: &Path,
    duration_hint_millis: u64,
    progress: &ProgressSlot,
) -> Result<LoudnessMeasurement, TaskExecutionError> {
    let args = vec![
        "-i".to_owned(),
        CLIP_FILE.to_owned(),
        "-vn".to_owned(),
        "-af".to_owned(),
        "loudnorm=print_format=json".to_owned(),
        "-f".to_owned(),
        "null".to_owned(),
        "-v".to_owned(),
        "info".to_owned(),
        "-".to_owned(),
    ];
    let report = context
        .media
        .run_ffmpeg(
            ffmpeg_spec(args, work, duration_hint_millis),
            progress.clone(),
        )
        .await
        .map_err(MediaError::into_task_error)?;
    LoudnessMeasurement::from_loudnorm_json(&report)
        .ok_or_else(|| TaskExecutionError::deterministic("the finished clip could not be measured"))
}

fn build_manifest(
    rendered: &Rendered<'_>,
    measured_input: LoudnessMeasurement,
    measured_output: LoudnessMeasurement,
    work: &Path,
) -> Result<RenderManifest, TaskExecutionError> {
    let (plan, font, payload) = (rendered.plan, rendered.font, rendered.payload);
    let mut outputs = Vec::with_capacity(PUBLISHED_FILES.len());
    for name in PUBLISHED_FILES {
        let bytes = fs::read(work.join(name))
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        outputs.push(OutputFile {
            path: name.to_owned(),
            sha256: format!(
                "sha256:{}",
                Sha256Digest::from_bytes(Sha256::digest(&bytes).into())
            ),
            bytes: bytes.len() as u64,
        });
    }
    Ok(RenderManifest {
        schema_version: MANIFEST_SCHEMA_VERSION.to_owned(),
        ir_hash: rendered.ir_hash.clone(),
        ir_artifact_id: rendered.ir_artifact_id.to_string(),
        profile: plan.profile.clone(),
        engine: EngineIdentity {
            app: format!("clipmilld {}", env!("CARGO_PKG_VERSION")),
            ffmpeg: FFMPEG_BOM.to_owned(),
            font_sha256: font.sha256.clone(),
            font_family: font.family.clone(),
        },
        // libx264 at a fixed thread count with bitexact containers reproduces
        // bytes for identical inputs on one platform; across platforms only
        // the decoded result is promised, which the parity drill checks.
        determinism: "byte_stable".to_owned(),
        ai_use_summary: AiUseSummary {
            assistance: payload.ai_assistance.clone(),
            generated: Vec::new(),
            requires_youtube_ai_disclosure: false,
        },
        rights: RightsAttestation {
            source_attestation: payload.source_attestation.clone(),
            gates_passed: payload.gates_passed.clone(),
        },
        input_source_fingerprints: input_fingerprints(plan),
        program: ProgramReport {
            duration_ticks: plan.duration_ticks,
            frame_count: plan.frame_count,
            segments: plan.segments.iter().map(ProgramSegment::from).collect(),
        },
        loudness: LoudnessReport {
            target_lufs: plan.profile.loudness.integrated_lufs,
            target_true_peak_dbtp: plan.profile.loudness.true_peak_dbtp,
            measured_input: into_measured(measured_input),
            measured_output: into_measured(measured_output),
        },
        caption_windows: plan.cue_windows.iter().map(CaptionWindow::from).collect(),
        outputs,
    })
}

fn into_measured(measurement: LoudnessMeasurement) -> MeasuredLoudness {
    MeasuredLoudness {
        integrated_lufs: measurement.input_lufs,
        true_peak_dbtp: measurement.input_true_peak_dbtp,
        loudness_range_lu: measurement.input_range_lu,
    }
}

fn input_fingerprints(plan: &RenderPlan) -> Vec<String> {
    let mut seen = BTreeMap::new();
    for (index, segment) in plan.segments.iter().enumerate() {
        seen.entry(segment.source_fingerprint.clone())
            .or_insert(index);
    }
    let mut ordered = seen.into_iter().collect::<Vec<_>>();
    ordered.sort_by_key(|(_, index)| *index);
    ordered
        .into_iter()
        .map(|(fingerprint, _)| fingerprint)
        .collect()
}
