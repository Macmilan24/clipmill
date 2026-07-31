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

use clipmill_contracts::proto::ipc::v1::{Job, Project, Source, Task};
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
