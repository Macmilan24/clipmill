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
    AnalyzeSourcePayloadV1, ClipDurationV1, GetStorageStatsResponse, Job, Project,
    RegisterSourceResponse, ResolveMediaResponse, Source, Task,
};
use serde::Serialize;

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
}

#[derive(Debug, Serialize)]
pub struct StorageCategoryView {
    /// `artifacts`, `models`, or `state`. The wording on screen is the
    /// renderer's; this is what it keys off.
    pub key: String,
    pub bytes: u64,
    pub items: u64,
}

impl From<GetStorageStatsResponse> for StorageStatsView {
    fn from(stats: GetStorageStatsResponse) -> Self {
        Self {
            available_bytes: stats.available_known.then_some(stats.available_bytes),
            categories: stats
                .categories
                .into_iter()
                .map(|category| StorageCategoryView {
                    key: category.key,
                    bytes: category.bytes,
                    items: category.items,
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
