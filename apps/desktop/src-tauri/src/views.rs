//! What the renderer is handed, and why it is not the wire type.
//!
//! The generated protobuf types do not serialize to JSON, and if they did the
//! renderer would be coupled to field numbering it has no business knowing. So
//! each thing a screen renders crosses this boundary as a plain record with the
//! names TypeScript expects.
//!
//! Nothing is interpreted on the way. States stay the integers the contract
//! defines rather than becoming strings this layer invented; progress keeps its
//! unit and both counts; a failure keeps its class beside its detail. A screen
//! that wants a word for `state` gets it from the generated enum on its own
//! side, where the contract is still the authority.
//!
//! The one document that does not appear here is an artifact's contents. Those
//! cross as the bytes the daemon published, parsed by the renderer with the
//! generated schema type, so the JSON Schema stays the only contract between
//! the two ends.

use clipmill_contracts::proto::ipc::v1::{
    AnalyzeSourcePayloadV1, ClipDurationV1, ExportArchiveResponse, ExportRequestV1, ExportSeverity,
    ExportSummaryV1, GetLocalLockResponse, GetStorageStatsResponse, Job, PlanExportResponse,
    Project, RegisterSourceResponse, ResolveMediaResponse, Source, Task,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ProjectView {
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub name: String,
    #[serde(rename = "createdUnixMillis")]
    pub created_unix_millis: u64,
}

impl From<Project> for ProjectView {
    fn from(project: Project) -> Self {
        Self {
            project_id: project.project_id,
            name: project.name,
            created_unix_millis: project.created_unix_millis,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SourceView {
    #[serde(rename = "sourceId")]
    pub source_id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    /// Shown to the user, who chose it. It never travels into a job payload —
    /// the daemon resolves a source id to its own record instead.
    #[serde(rename = "absolutePath")]
    pub absolute_path: String,
    #[serde(rename = "byteSize")]
    pub byte_size: u64,
    #[serde(rename = "sourceFingerprint")]
    pub source_fingerprint: String,
    #[serde(rename = "sourceMapArtifactId")]
    pub source_map_artifact_id: String,
    #[serde(rename = "createdUnixMillis")]
    pub created_unix_millis: u64,
}

impl From<Source> for SourceView {
    fn from(source: Source) -> Self {
        Self {
            source_id: source.source_id,
            project_id: source.project_id,
            absolute_path: source.absolute_path,
            byte_size: source.byte_size,
            source_fingerprint: source.source_fingerprint,
            source_map_artifact_id: source.source_map_artifact_id,
            created_unix_millis: source.created_unix_millis,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TaskView {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub kind: String,
    /// What this task publishes, e.g. `media.filmstrip.v1`. A screen looks for
    /// the observation it wants by kind rather than by the daemon's name for the
    /// work that produces it.
    #[serde(rename = "outputKind")]
    pub output_kind: String,
    pub state: i32,
    pub attempt: u32,
    #[serde(rename = "maxAttempts")]
    pub max_attempts: u32,
    #[serde(rename = "waitReason")]
    pub wait_reason: String,
    /// Empty until the task publishes. A screen that showed an address before
    /// there was one would be promising a document nobody can open.
    #[serde(rename = "outputArtifactId")]
    pub output_artifact_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressView>,
}

#[derive(Debug, Serialize)]
pub struct ProgressView {
    pub unit: String,
    pub done: u64,
    /// Zero means the stage knows how far it has come and not how far there is
    /// to go. A bar drawn from that would be inventing the denominator.
    pub total: u64,
}

impl From<Task> for TaskView {
    fn from(task: Task) -> Self {
        Self {
            task_id: task.task_id,
            kind: task.kind,
            output_kind: task.output_kind,
            state: task.state,
            attempt: task.attempt,
            max_attempts: task.max_attempts,
            wait_reason: task.wait_reason,
            output_artifact_id: task.output_artifact_id,
            progress: task.progress.map(|progress| ProgressView {
                unit: progress.unit,
                done: progress.done,
                total: progress.total,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JobView {
    #[serde(rename = "contentProfile")]
    pub content_profile: String,
    #[serde(rename = "jobId")]
    pub job_id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub kind: String,
    pub state: i32,
    #[serde(rename = "createdUnixMillis")]
    pub created_unix_millis: u64,
    #[serde(rename = "updatedUnixMillis")]
    pub updated_unix_millis: u64,
    pub tasks: Vec<TaskView>,
    /// The artifacts this job rooted. An analysis roots exactly one: its
    /// manifest, which names every stage underneath.
    #[serde(rename = "outputArtifactIds")]
    pub output_artifact_ids: Vec<String>,
    #[serde(rename = "failureClass")]
    pub failure_class: i32,
    /// Kept beside the class rather than replacing it: the class says whether a
    /// retry could help, and the detail says what actually went wrong.
    #[serde(rename = "failureDetail")]
    pub failure_detail: String,
    /// The recording the job ran over, or empty for a job not about one. A
    /// project can hold several recordings, and which one an analysis belongs
    /// to is the first thing a screen reading it needs to know.
    #[serde(rename = "sourceId")]
    pub source_id: String,
    /// What an export job is delivering, off its own payload; absent for
    /// every other kind. It is what lets the export screen find a document's
    /// export again after it was left or the application relaunched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<ExportSummaryView>,
}

/// The identity of an export, as its job carries it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummaryView {
    pub doc_id: String,
    pub revision: u64,
    pub ir_artifact_id: String,
    pub destination_dir: String,
}

impl From<ExportSummaryV1> for ExportSummaryView {
    fn from(summary: ExportSummaryV1) -> Self {
        Self {
            doc_id: summary.doc_id,
            revision: summary.revision,
            ir_artifact_id: summary.ir_artifact_id,
            destination_dir: summary.destination_dir,
        }
    }
}

impl From<Job> for JobView {
    fn from(job: Job) -> Self {
        Self {
            job_id: job.job_id,
            project_id: job.project_id,
            kind: job.kind,
            state: job.state,
            created_unix_millis: job.created_unix_millis,
            updated_unix_millis: job.updated_unix_millis,
            tasks: job.tasks.into_iter().map(Into::into).collect(),
            output_artifact_ids: job.output_artifact_ids,
            failure_class: job.failure_class,
            failure_detail: job.failure_detail,
            source_id: job.source_id,
            export: job.export.map(Into::into),
            content_profile: job.content_profile,
        }
    }
}

/// A source the daemon has just registered, and whether it had to probe it.
#[derive(Debug, Serialize)]
pub struct RegisteredSourceView {
    pub source: SourceView,
    /// True when an unchanged observation avoided another FFprobe run. Worth
    /// surfacing: it is why picking the same file twice is instant.
    #[serde(rename = "observationCacheHit")]
    pub observation_cache_hit: bool,
    /// The probe, inline, because the artifact carrying it is not published
    /// until the analysis runs — and a screen has to show a duration before
    /// anyone commits to running one.
    #[serde(rename = "sourceMapJson")]
    pub source_map_json: String,
}

impl TryFrom<RegisterSourceResponse> for RegisteredSourceView {
    type Error = &'static str;

    fn try_from(registered: RegisterSourceResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            observation_cache_hit: registered.observation_cache_hit,
            source_map_json: registered.source_map_json,
            source: registered
                .source
                .ok_or("the daemon registered no source")?
                .into(),
        })
    }
}

/// What a screen asks for when it starts an analysis.
///
/// Ticks are the contract's unit and the renderer speaks them, so nothing here
/// converts seconds: a screen that offered "15 to 60 seconds" already turned
/// that into the timebase the daemon keys against, and doing it twice is how the
/// two ends come to disagree.
#[derive(Debug, serde::Deserialize)]
pub struct AnalyzeRequest {
    #[serde(rename = "sourceId")]
    pub source_id: String,
    /// BCP 47 primary subtag, or empty to let the recognizer decide.
    pub language: String,
    #[serde(rename = "minTicks")]
    pub min_ticks: u64,
    #[serde(rename = "maxTicks")]
    pub max_ticks: u64,
    /// Zero leaves the daemon's default, so a caller with no opinion needs none.
    pub count: u64,
    #[serde(default, rename = "localEditorial")]
    pub local_editorial: bool,
    #[serde(default, rename = "contentProfile")]
    pub content_profile: String,
    #[serde(default, rename = "cloudEditorial")]
    pub cloud_editorial: Option<CloudEditorialRequest>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloudEditorialRequest {
    pub transcript_consent: bool,
    pub budget_micro_usd: u64,
    pub model: String,
}

impl AnalyzeRequest {
    pub fn into_payload(self) -> AnalyzeSourcePayloadV1 {
        AnalyzeSourcePayloadV1 {
            key_version: ANALYZE_SOURCE_KEY_VERSION.to_owned(),
            source_id: self.source_id,
            language: self.language,
            duration: Some(ClipDurationV1 {
                min_ticks: self.min_ticks,
                max_ticks: self.max_ticks,
            }),
            count: self.count,
            diversity_milli: 0,
            local_editorial: self.local_editorial,
            content_profile: self.content_profile,
            cloud_editorial: self.cloud_editorial.map(|c| {
                clipmill_contracts::proto::ipc::v1::EditorialCloudV1 {
                    transcript_consent: c.transcript_consent,
                    budget_micro_usd: c.budget_micro_usd,
                    model: c.model,
                }
            }),
        }
    }
}

/// The payload version the daemon accepts for an analysis. Stated here because
/// the shell composes the payload; a mismatch is refused at submit.
const ANALYZE_SOURCE_KEY_VERSION: &str = "clipmill.analyze-source.v1";

/// Permission to stream a media artifact, and what it holds.
///
/// The inventory is the point. A filmstrip names its tiles, a render names its
/// outputs, and a screen cannot build a URL for a file it does not know the name
/// of — guessing at `strip_00001.jpg` would be a renderer reimplementing a
/// producer's naming convention. No bytes and no paths cross here: the URL a
/// screen builds from this goes to the media protocol, which opens the object
/// itself.
#[derive(Debug, Serialize)]
pub struct MediaArtifactView {
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub kind: String,
    pub files: Vec<MediaFileView>,
}

#[derive(Debug, Serialize)]
pub struct MediaFileView {
    /// Name inside the artifact, e.g. `proxy.mp4`. Not a filesystem path.
    pub path: String,
    pub bytes: u64,
    #[serde(rename = "mediaType")]
    pub media_type: String,
}

impl From<ResolveMediaResponse> for MediaArtifactView {
    fn from(resolved: ResolveMediaResponse) -> Self {
        Self {
            artifact_id: resolved.artifact_id,
            kind: resolved.kind,
            files: resolved
                .files
                .into_iter()
                .map(|file| MediaFileView {
                    path: file.path,
                    bytes: file.bytes,
                    media_type: file.media_type,
                })
                .collect(),
        }
    }
}

/// What this installation is using on disk, by category.
#[derive(Debug, Serialize)]
pub struct StorageStatsView {
    pub categories: Vec<StorageCategoryView>,
    /// Absent when the filesystem would not say, which is a different answer
    /// from zero. The wire carries the two apart for exactly this reason, and
    /// collapsing them here would tell a screen the disk is full.
    #[serde(rename = "availableBytes", skip_serializing_if = "Option::is_none")]
    pub available_bytes: Option<u64>,
    /// How long an unreferenced artifact is kept before collection may take
    /// it. Shown, not adjustable.
    #[serde(rename = "retentionGraceSeconds")]
    pub retention_grace_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct StorageCategoryView {
    /// `artifacts`, `models`, or `state`. The wording on screen is the
    /// renderer's; this is what it keys off.
    pub key: String,
    pub bytes: u64,
    pub items: u64,
    /// Where this category lives, so a size is somewhere a user can go.
    pub path: String,
}

impl From<GetStorageStatsResponse> for StorageStatsView {
    fn from(stats: GetStorageStatsResponse) -> Self {
        Self {
            available_bytes: stats.available_known.then_some(stats.available_bytes),
            retention_grace_seconds: stats.retention_grace_seconds,
            categories: stats
                .categories
                .into_iter()
                .map(|category| StorageCategoryView {
                    key: category.key,
                    bytes: category.bytes,
                    items: category.items,
                    path: category.path,
                })
                .collect(),
        }
    }
}

/// One published document, as the daemon holds it.
#[derive(Debug, Serialize)]
pub struct DocumentView {
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    /// Echoed so the renderer can refuse a document it did not ask for rather
    /// than parsing it and finding out.
    pub kind: String,
    /// The canonical JSON the daemon published, unparsed. The renderer reads it
    /// with the generated schema type, which keeps the JSON Schema the only
    /// contract between the two ends.
    pub json: String,
}

/// What the renderer asks for when a clip is approved.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire request flags mirror the existing IPC contract"
)]
pub struct DirectClipInput {
    pub project_id: String,
    pub source_id: String,
    pub candidate_id: String,
    /// `chosen`, `alternative`, or `exact`. Anything else is read as `chosen`,
    /// because a request that named a cut nobody defined has not asked for a
    /// different one.
    pub cut: String,
    #[serde(default)]
    pub style_ref: String,
    /// Read only for `exact`, and snapped to the lattice by the daemon.
    #[serde(default)]
    pub start_ticks: u64,
    #[serde(default)]
    pub end_ticks: u64,
    /// Build a second document beside the one this candidate already has,
    /// rather than reopening it.
    #[serde(default)]
    pub variation: bool,
    /// Record the approval in the same write as the document.
    #[serde(default)]
    pub approve: bool,
    #[serde(default)]
    pub allow_declined: bool,
    #[serde(default)]
    pub manual_span: bool,
    /// The analysis run the candidate belongs to; every stage the director
    /// reads is taken from it. Empty takes the newest run over the source.
    #[serde(default)]
    pub job_id: String,
}

impl From<DirectClipInput> for clipmill_contracts::proto::ipc::v1::DirectClipRequest {
    fn from(input: DirectClipInput) -> Self {
        Self {
            project_id: input.project_id,
            source_id: input.source_id,
            candidate_id: input.candidate_id,
            cut: match input.cut.as_str() {
                "alternative" => 2,
                "exact" => 3,
                _ => 1,
            },
            style_ref: input.style_ref,
            start_ticks: input.start_ticks,
            end_ticks: input.end_ticks,
            variation: input.variation,
            approve: input.approve,
            allow_declined: input.allow_declined,
            manual_span: input.manual_span,
            job_id: input.job_id,
        }
    }
}

/// The document a directed clip produced, and where its cut landed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectedClipView {
    pub doc_id: String,
    pub project_id: String,
    pub source_id: String,
    pub candidate_id: String,
    /// The run the document was cut from; empty when it was not recorded.
    pub job_id: String,
    pub revision: u64,
    pub document_json: String,
    /// Where the cut actually landed, which is not always where it was asked
    /// for: a hand-set boundary is moved onto the lattice first.
    pub start_ticks: u64,
    pub end_ticks: u64,
    /// Why the director did what it did, in sentences.
    pub decisions: Vec<String>,
    /// True when the document already existed and came back as it stands.
    pub reopened: bool,
}

impl From<clipmill_contracts::proto::ipc::v1::DirectClipResponse> for DirectedClipView {
    fn from(reply: clipmill_contracts::proto::ipc::v1::DirectClipResponse) -> Self {
        let doc = reply.doc.unwrap_or_default();
        Self {
            doc_id: doc.doc_id,
            project_id: doc.project_id,
            source_id: doc.source_id,
            candidate_id: doc.candidate_id,
            job_id: doc.job_id,
            revision: doc.revision,
            document_json: doc.document_json,
            start_ticks: reply.start_ticks,
            end_ticks: reply.end_ticks,
            decisions: reply.decisions,
            reopened: reply.reopened,
        }
    }
}

/// One decision, as the interface names it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipDecisionView {
    pub candidate_id: String,
    pub decision: String,
    pub decided_unix_millis: u64,
}

impl From<clipmill_contracts::proto::ipc::v1::ClipDecisionRecordV1> for ClipDecisionView {
    fn from(record: clipmill_contracts::proto::ipc::v1::ClipDecisionRecordV1) -> Self {
        Self {
            candidate_id: record.candidate_id,
            decision: decision_word(record.decision),
            decided_unix_millis: record.decided_unix_millis,
        }
    }
}

/// The wire code for a decision word. Zero is "unspecified", which the daemon
/// refuses — a word this shell does not recognize is not silently turned into
/// one of the three real answers.
pub fn decision_code(word: &str) -> i32 {
    match word {
        "rejected" => 1,
        "kept" => 2,
        "approved" => 3,
        _ => 0,
    }
}

/// The word for a wire code, for a renderer that should never see an integer.
pub fn decision_word(code: i32) -> String {
    match code {
        1 => "rejected",
        2 => "kept",
        3 => "approved",
        _ => "unspecified",
    }
    .to_owned()
}

/// One point of a proposed crop path, normalized against the source frame.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CropKeyframeView {
    pub t_ticks: u64,
    pub center_x: f64,
    pub center_y: f64,
    pub scale: f64,
}

/// A crop path proposal, and what it is worth.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CropPathView {
    pub keyframes: Vec<CropKeyframeView>,
    /// True when nobody earned the frame and this is the fitted rectangle. The
    /// reason travels with it, because "why is this not tracking" is the first
    /// thing anyone asks.
    pub fit: bool,
    pub fit_reason: String,
    pub containment: f64,
}

impl From<clipmill_contracts::proto::ipc::v1::SolveCropPathResponse> for CropPathView {
    fn from(reply: clipmill_contracts::proto::ipc::v1::SolveCropPathResponse) -> Self {
        Self {
            keyframes: reply
                .keyframes
                .into_iter()
                .map(|keyframe| CropKeyframeView {
                    t_ticks: keyframe.t_ticks,
                    center_x: keyframe.center_x,
                    center_y: keyframe.center_y,
                    scale: keyframe.scale,
                })
                .collect(),
            fit: reply.fit,
            fit_reason: reply.fit_reason,
            containment: reply.containment,
        }
    }
}

/// The plan the player applies. Everything that could be computed already was,
/// by the code that renders — so nothing here is a number the renderer works
/// out for itself.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPlanView {
    pub revision: u64,
    pub rate_num: u32,
    pub rate_den: u32,
    pub frame_count: i64,
    /// One entry per frame; null where the layout is fit and the whole picture
    /// is shown, which is a different statement from a crop covering it.
    pub crops: Vec<Option<[i64; 4]>>,
    pub secondary_crops: Vec<Option<[i64; 4]>>,
    pub caption_style: Option<PreviewCaptionStyleView>,
    pub cues: Vec<PreviewCueView>,
    pub gain: Vec<PreviewGainView>,
    pub width: i64,
    pub height: i64,
    /// The program's segments, each mapped to its source: the numbers every
    /// seek, scrub and trim go through.
    pub segments: Vec<PreviewSegmentView>,
    /// The sources the segments name, with the frame the crops are measured in.
    pub sources: Vec<PreviewSourceView>,
    /// The proxy for each source that has one.
    pub proxies: Vec<PreviewProxyView>,
    /// Which cue list `cues` came from: `burn_in` or `reading`.
    pub presentation: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewCaptionStyleView {
    pub style_ref: String,
    pub font_family: String,
    pub font_size: u32,
    pub spoken: String,
    pub unspoken: String,
    pub outline: String,
    pub shadow: String,
    pub outline_width: u32,
    pub shadow_depth: u32,
    pub bold: bool,
    pub boxed: bool,
    pub margin_horizontal: u32,
    pub margin_vertical: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSegmentView {
    pub segment_id: String,
    pub source_fingerprint: String,
    pub in_ticks: i64,
    pub out_ticks: i64,
    pub program_start_ticks: i64,
    pub has_two_up_paths: bool,
    pub framing_warning: String,
    pub first_frame: i64,
    pub end_frame: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSourceView {
    pub source_fingerprint: String,
    pub source_id: String,
    pub display_width: i64,
    pub display_height: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewProxyView {
    pub source_fingerprint: String,
    pub artifact_id: String,
    pub file: String,
    pub coverage_start_ticks: i64,
    pub coverage_end_ticks: i64,
    pub width: i64,
    pub height: i64,
    pub rate_num: u32,
    pub rate_den: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewCueView {
    pub cue_id: String,
    pub first_frame: i64,
    pub end_frame: i64,
    pub region: String,
    pub karaoke: bool,
    pub lead_in_centis: i64,
    pub lines: Vec<Vec<PreviewWordView>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewWordView {
    pub text: String,
    pub hold_centis: i64,
    /// The word's identity across both presentations; empty for a word the
    /// document never gave one.
    pub word_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewGainView {
    pub frame: i64,
    pub gain_db: f64,
}

impl From<clipmill_contracts::proto::ipc::v1::GetPreviewPlanResponse> for PreviewPlanView {
    #[allow(
        clippy::too_many_lines,
        reason = "field-for-field conversion of the daemon preview contract"
    )]
    fn from(reply: clipmill_contracts::proto::ipc::v1::GetPreviewPlanResponse) -> Self {
        Self {
            revision: reply.revision,
            rate_num: reply.rate_num,
            rate_den: reply.rate_den,
            frame_count: reply.frame_count,
            crops: reply
                .crops
                .into_iter()
                .map(|crop| {
                    crop.present
                        .then_some([crop.x, crop.y, crop.width, crop.height])
                })
                .collect(),
            caption_style: reply.caption_style.map(|style| PreviewCaptionStyleView {
                style_ref: style.style_ref,
                font_family: style.font_family,
                font_size: style.font_size,
                spoken: style.spoken,
                unspoken: style.unspoken,
                outline: style.outline,
                shadow: style.shadow,
                outline_width: style.outline_width,
                shadow_depth: style.shadow_depth,
                bold: style.bold,
                boxed: style.boxed,
                margin_horizontal: style.margin_horizontal,
                margin_vertical: style.margin_vertical,
            }),
            secondary_crops: reply
                .secondary_crops
                .into_iter()
                .map(|crop| {
                    crop.present
                        .then_some([crop.x, crop.y, crop.width, crop.height])
                })
                .collect(),
            cues: reply
                .cues
                .into_iter()
                .map(|cue| PreviewCueView {
                    cue_id: cue.cue_id,
                    first_frame: cue.first_frame,
                    end_frame: cue.end_frame,
                    region: cue.region,
                    karaoke: cue.karaoke,
                    lead_in_centis: cue.lead_in_centis,
                    lines: cue
                        .lines
                        .into_iter()
                        .map(|line| {
                            line.words
                                .into_iter()
                                .map(|word| PreviewWordView {
                                    text: word.text,
                                    hold_centis: word.hold_centis,
                                    word_id: word.word_id,
                                })
                                .collect()
                        })
                        .collect(),
                })
                .collect(),
            gain: reply
                .gain
                .into_iter()
                .map(|point| PreviewGainView {
                    frame: point.frame,
                    gain_db: point.gain_db,
                })
                .collect(),
            width: reply.width,
            height: reply.height,
            segments: reply
                .segments
                .into_iter()
                .map(|segment| PreviewSegmentView {
                    segment_id: segment.segment_id,
                    source_fingerprint: segment.source_fingerprint,
                    in_ticks: segment.in_ticks,
                    out_ticks: segment.out_ticks,
                    program_start_ticks: segment.program_start_ticks,
                    has_two_up_paths: segment.has_two_up_paths,
                    framing_warning: segment.framing_warning,
                    first_frame: segment.first_frame,
                    end_frame: segment.end_frame,
                })
                .collect(),
            sources: reply
                .sources
                .into_iter()
                .map(|source| PreviewSourceView {
                    source_fingerprint: source.source_fingerprint,
                    source_id: source.source_id,
                    display_width: source.display_width,
                    display_height: source.display_height,
                })
                .collect(),
            proxies: reply
                .proxies
                .into_iter()
                .map(|proxy| PreviewProxyView {
                    source_fingerprint: proxy.source_fingerprint,
                    artifact_id: proxy.artifact_id,
                    file: proxy.file,
                    coverage_start_ticks: proxy.coverage_start_ticks,
                    coverage_end_ticks: proxy.coverage_end_ticks,
                    width: proxy.width,
                    height: proxy.height,
                    rate_num: proxy.rate_num,
                    rate_den: proxy.rate_den,
                })
                .collect(),
            presentation: reply.presentation,
        }
    }
}

/// One edit document, as a list shows it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditDocView {
    pub doc_id: String,
    pub project_id: String,
    /// The source it was cut from and the candidate it was built for; both
    /// empty for a document that was handed in whole rather than directed.
    pub source_id: String,
    pub candidate_id: String,
    /// The run it was cut from; empty when it was not recorded.
    pub job_id: String,
    pub revision: u64,
    pub created_unix_millis: u64,
    pub updated_unix_millis: u64,
}

impl From<clipmill_contracts::proto::ipc::v1::EditDoc> for EditDocView {
    fn from(doc: clipmill_contracts::proto::ipc::v1::EditDoc) -> Self {
        Self {
            doc_id: doc.doc_id,
            project_id: doc.project_id,
            source_id: doc.source_id,
            candidate_id: doc.candidate_id,
            job_id: doc.job_id,
            revision: doc.revision,
            created_unix_millis: doc.created_unix_millis,
            updated_unix_millis: doc.updated_unix_millis,
        }
    }
}

/// What applying a command produced: the new revision, and the way back.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedCommandView {
    pub doc_id: String,
    pub revision: u64,
    /// The command that undoes this one. Held by the renderer as its undo
    /// stack, because the daemon deliberately keeps none.
    pub inverse_command_json: String,
}

impl From<clipmill_contracts::proto::ipc::v1::ApplyEditCommandResponse> for AppliedCommandView {
    fn from(reply: clipmill_contracts::proto::ipc::v1::ApplyEditCommandResponse) -> Self {
        let doc = reply.doc.unwrap_or_default();
        Self {
            doc_id: doc.doc_id,
            revision: doc.revision,
            inverse_command_json: reply.inverse_command_json,
        }
    }
}

/// What an export would do, before it does it.
#[derive(Debug, Serialize)]
pub struct ExportPlanView {
    pub passes: bool,
    pub findings: Vec<ExportFindingView>,
    /// The resolved filename stem, which is the naming preview.
    pub stem: String,
    #[serde(rename = "fileNames")]
    pub file_names: Vec<String>,
    #[serde(rename = "estimatedBytes")]
    pub estimated_bytes: u64,
    /// Absent when free space could not be read, which is not the same answer
    /// as a full disk.
    #[serde(rename = "availableBytes", skip_serializing_if = "Option::is_none")]
    pub available_bytes: Option<u64>,
    /// The revision this plan was computed over: what the person is reviewing,
    /// and what the export must be of.
    pub revision: u64,
}

#[derive(Debug, Serialize)]
pub struct ExportFindingView {
    pub code: String,
    /// `blocking` or `advisory`. Sent as the word rather than the number so a
    /// screen cannot mistake one enum ordinal for another.
    pub severity: String,
    pub detail: String,
}

impl From<PlanExportResponse> for ExportPlanView {
    fn from(plan: PlanExportResponse) -> Self {
        let validation = plan.validation.unwrap_or_default();
        Self {
            passes: validation.passes,
            findings: validation
                .findings
                .into_iter()
                .map(|finding| ExportFindingView {
                    code: finding.code,
                    // Anything that is not explicitly advisory is treated as
                    // blocking, including a severity this build does not know:
                    // a finding nobody can classify must not be the one that
                    // quietly lets an export through.
                    severity: if ExportSeverity::try_from(finding.severity)
                        == Ok(ExportSeverity::Advisory)
                    {
                        "advisory".to_owned()
                    } else {
                        "blocking".to_owned()
                    },
                    detail: finding.detail,
                })
                .collect(),
            stem: plan.stem,
            file_names: plan.file_names,
            estimated_bytes: plan.estimated_bytes,
            available_bytes: plan.available_known.then_some(plan.available_bytes),
            revision: plan.revision,
        }
    }
}

/// What an export froze when it was queued.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedExportView {
    /// The job to watch: its two tasks are the render and the delivery.
    pub job_id: String,
    /// The revision rendered, and the immutable snapshot it was frozen as.
    pub revision: u64,
    pub ir_artifact_id: String,
    /// The folder, resolved, the files land in; the delivered package names
    /// its files relative to it.
    pub destination_dir: String,
}

impl From<clipmill_contracts::proto::ipc::v1::ExportClipResponse> for QueuedExportView {
    fn from(reply: clipmill_contracts::proto::ipc::v1::ExportClipResponse) -> Self {
        Self {
            job_id: reply.job_id,
            revision: reply.revision,
            ir_artifact_id: reply.ir_artifact_id,
            destination_dir: reply.destination_dir,
        }
    }
}

/// Where an archive was written, and what it hashes to.
#[derive(Debug, Serialize)]
pub struct ArchiveView {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    #[serde(rename = "entryCount")]
    pub entry_count: u32,
}

impl From<ExportArchiveResponse> for ArchiveView {
    fn from(archive: ExportArchiveResponse) -> Self {
        Self {
            path: archive.path,
            sha256: archive.sha256,
            bytes: archive.bytes,
            entry_count: archive.entry_count,
        }
    }
}

/// Whether this installation is offline, and the evidence for it.
#[derive(Debug, Serialize)]
pub struct LocalLockView {
    pub engaged: bool,
    pub stages: u32,
    #[serde(rename = "networkAllowedStages")]
    pub network_allowed_stages: u32,
    #[serde(rename = "egressAttempts")]
    pub egress_attempts: u64,
}

impl From<GetLocalLockResponse> for LocalLockView {
    fn from(reply: GetLocalLockResponse) -> Self {
        let status = reply.status.unwrap_or_default();
        Self {
            engaged: status.engaged,
            stages: status.stages,
            network_allowed_stages: status.network_allowed_stages,
            egress_attempts: status.egress_attempts,
        }
    }
}

/// Whether an analysis could run right now, stage by stage.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessView {
    pub ready: bool,
    pub decoder_present: bool,
    pub decoder_path: String,
    pub stages: Vec<StageReadinessView>,
    pub workers: Vec<WorkerPresenceView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageReadinessView {
    /// The task kind, e.g. `speech-asr`.
    pub stage: String,
    pub capability: String,
    pub implementation: String,
    pub model: String,
    pub backend: String,
    pub model_present: bool,
    pub missing_files: Vec<String>,
    pub worker_present: bool,
    pub ready: bool,
    /// What to do about it, when not ready.
    pub remedy: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerPresenceView {
    pub worker_id: String,
    pub family: String,
    pub capabilities: Vec<String>,
    pub backend: String,
    pub since_unix_millis: u64,
}

impl From<clipmill_contracts::proto::ipc::v1::GetReadinessResponse> for ReadinessView {
    fn from(reply: clipmill_contracts::proto::ipc::v1::GetReadinessResponse) -> Self {
        Self {
            ready: reply.ready,
            decoder_present: reply.decoder_present,
            decoder_path: reply.decoder_path,
            stages: reply
                .stages
                .into_iter()
                .map(|stage| StageReadinessView {
                    stage: stage.stage,
                    capability: stage.capability,
                    implementation: stage.implementation,
                    model: stage.model,
                    backend: stage.backend,
                    model_present: stage.model_present,
                    missing_files: stage.missing_files,
                    worker_present: stage.worker_present,
                    ready: stage.ready,
                    remedy: stage.remedy,
                })
                .collect(),
            workers: reply
                .workers
                .into_iter()
                .map(|worker| WorkerPresenceView {
                    worker_id: worker.worker_id,
                    family: worker.family,
                    capabilities: worker.capabilities,
                    backend: worker.backend,
                    since_unix_millis: worker.since_unix_millis,
                })
                .collect(),
        }
    }
}

/// What the Export screen asks for, in the shape the renderer sends it.
///
/// A deserializable twin of the wire message rather than the wire message
/// itself: the generated proto types carry no serde derives, and giving the
/// renderer its own struct keeps the field names camel-cased on the side that
/// reads them and snake-cased on the side that transmits them.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequestInput {
    pub doc_id: String,
    pub destination_dir: String,
    #[serde(default)]
    pub naming_pattern: String,
    #[serde(default)]
    pub source_attestation: String,
    #[serde(default)]
    pub gates_passed: Vec<String>,
    #[serde(default)]
    pub ai_assistance: Vec<String>,
    #[serde(default)]
    pub index: u32,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub title: String,
    /// The revision the person reviewed; the daemon refuses to export any
    /// other. Absent takes the current revision.
    #[serde(default)]
    pub expected_revision: Option<u64>,
}

impl From<ExportRequestInput> for ExportRequestV1 {
    fn from(input: ExportRequestInput) -> Self {
        Self {
            doc_id: input.doc_id,
            destination_dir: input.destination_dir,
            naming_pattern: input.naming_pattern,
            source_attestation: input.source_attestation,
            gates_passed: input.gates_passed,
            ai_assistance: input.ai_assistance,
            index: input.index,
            date: input.date,
            title: input.title,
            expected_revision: input.expected_revision,
        }
    }
}

impl From<ExportRequestV1> for ExportRequestInput {
    fn from(request: ExportRequestV1) -> Self {
        Self {
            doc_id: request.doc_id,
            destination_dir: request.destination_dir,
            naming_pattern: request.naming_pattern,
            source_attestation: request.source_attestation,
            gates_passed: request.gates_passed,
            ai_assistance: request.ai_assistance,
            index: request.index,
            date: request.date,
            title: request.title,
            expected_revision: request.expected_revision,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBatchView {
    pub batch_id: String,
    pub created_unix_millis: u64,
    pub items: Vec<ExportBatchItemView>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBatchItemView {
    pub index: u32,
    pub project_id: String,
    pub request: ExportRequestInput,
    pub state: String,
    pub attempt: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queued: Option<QueuedExportView>,
    pub error: String,
}
impl TryFrom<clipmill_contracts::proto::ipc::v1::ExportBatchV1> for ExportBatchView {
    type Error = String;
    fn try_from(
        batch: clipmill_contracts::proto::ipc::v1::ExportBatchV1,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            batch_id: batch.batch_id,
            created_unix_millis: batch.created_unix_millis,
            items: batch
                .items
                .into_iter()
                .map(|item| {
                    Ok(ExportBatchItemView {
                        index: item.index,
                        project_id: item.project_id,
                        request: item
                            .request
                            .ok_or("Saved batch item has no request")?
                            .into(),
                        state: item.state,
                        attempt: item.attempt,
                        queued: item.queued.map(Into::into),
                        error: item.error,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        })
    }
}
