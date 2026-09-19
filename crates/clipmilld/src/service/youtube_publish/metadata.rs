//! Optional local suggestions are durable model work, never an upload action.
use super::verified::MetadataContext;
use super::{Error, Metadata, PublishingCommand, Service, YoutubeVideoMetadataV1, now};
use crate::jobs::{
    JobPlan, JobRecord, YOUTUBE_METADATA_MAX_OUTPUT_TOKENS, youtube_metadata_prompt_digest,
};
use clipmill_contracts::proto::ipc::v1::{
    DraftYoutubeMetadataRequest, DraftYoutubeMetadataResponse, JobState, TaskState,
};
use clipmill_core::{ArtifactId, ProjectId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::Read;

const KIND: &str = "youtube-metadata";
const OUTPUT: &str = "publishing.metadata.v1";
const IMPLEMENTATION: &str = "clipmill-worker-editorial@0.2.0/metadata";

#[derive(Debug, Serialize)]
struct GenerationIdentity {
    key_version: &'static str,
    ir_artifact_id: String,
    model_name: String,
    model_digest: String,
    prompt_digest: String,
    max_output_tokens: u32,
}

impl Service {
    pub(super) async fn youtube_metadata_draft(
        &self,
        request: &DraftYoutubeMetadataRequest,
    ) -> Result<DraftYoutubeMetadataResponse, Error> {
        if !matches!(
            request.generation_action.as_str(),
            "" | "start" | "status" | "cancel" | "retry"
        ) {
            return Err(Error::Invalid("Choose a supported metadata action."));
        }
        if matches!(request.generation_action.as_str(), "cancel" | "retry")
            && request.generation_job_id.is_empty()
        {
            return Err(Error::Invalid(
                "Choose the metadata suggestion to cancel or retry.",
            ));
        }
        if !request.generation_job_id.is_empty()
            && request
                .generation_job_id
                .parse::<clipmill_core::JobId>()
                .is_err()
        {
            return Err(Error::Invalid("The metadata job identity is invalid."));
        }
        let mut context = self.youtube_metadata_context(request).await?;
        let Some(identity) = self.metadata_identity(&context.ir_id) else {
            unavailable(
                &mut context.draft,
                "Local Qwen is not configured. You can still edit and upload the current metadata.",
            );
            return Ok(context.draft);
        };
        context.draft.model_name.clone_from(&identity.model_name);
        let payload = serde_json::to_vec(&identity).map_err(|_| Error::Protocol)?;
        let mut job = self
            .find_metadata_job(&context.project_id, &payload, &request.generation_job_id)
            .await?;
        if let Some(saved) = &job {
            context.draft.generation_job_id.clone_from(&saved.job_id);
        }
        if request.generation_action == "cancel" {
            let saved = job.as_ref().ok_or(Error::Invalid(
                "The metadata suggestion was not found for this rendered clip.",
            ))?;
            let cancelled = self
                .database
                .cancel_job(
                    format!("youtube-metadata-cancel-{}", saved.job_id),
                    Sha256::digest(saved.job_id.as_bytes()).into(),
                    saved.job_id.clone(),
                    now(),
                )
                .await
                .map_err(|_| Error::Protocol)?;
            self.events.publish_all(cancelled.events);
            job = self
                .find_metadata_job(&context.project_id, &payload, &saved.job_id)
                .await?;
        } else if request.generation_action == "retry"
            || (request.generation_action == "start" && job.is_none())
        {
            if !context.has_transcript {
                unavailable(
                    &mut context.draft,
                    "This rendered clip has no saved caption text for Qwen to use. Write the metadata manually.",
                );
                return Ok(context.draft);
            }
            if let Some(message) = self.metadata_unavailable() {
                unavailable(&mut context.draft, &message);
                return Ok(context.draft);
            }
            job = self.start_metadata_job(request, &context, &payload).await?;
        }
        if let Some(job) = job {
            self.describe_metadata_job(&mut context, &identity, &job)
                .await;
        } else if let Some(message) = self.metadata_unavailable() {
            unavailable(&mut context.draft, &message);
        }
        Ok(context.draft)
    }

    async fn start_metadata_job(
        &self,
        request: &DraftYoutubeMetadataRequest,
        context: &MetadataContext,
        payload: &[u8],
    ) -> Result<Option<JobRecord>, Error> {
        // Only starting new work needs the expensive video integrity pass.
        // Status polls and successful suggestions remain cheap local reads.
        let export = self
            .verified_youtube_export(&request.export_job_id, request.expected_revision)
            .await?;
        if export.ir_id != context.ir_id || export.render_id != context.draft.render_artifact_id {
            return Err(Error::Invalid(
                "The rendered snapshot changed. Refresh this export before generating metadata.",
            ));
        }
        let project = context
            .project_id
            .parse::<ProjectId>()
            .map_err(|_| Error::Protocol)?;
        let ir = context
            .ir_id
            .parse::<ArtifactId>()
            .map_err(|_| Error::Protocol)?;
        let plan = JobPlan::youtube_metadata(&project, payload.into(), ir, &self.models, now())
            .map_err(Error::Invalid)?;
        let submitted = self
            .database
            .publishing(PublishingCommand::MetadataJob {
                project_id: context.project_id.clone(),
                payload: payload.into(),
                job_id: String::new(),
                plan: Some(Box::new(plan)),
                retry_job_id: if request.generation_action == "retry" {
                    request.generation_job_id.clone()
                } else {
                    String::new()
                },
            })
            .await
            .map_err(|error| match error {
                crate::db::StoreError::PublishingConflict(message) => Error::Invalid(message),
                _ => Error::Protocol,
            })?;
        self.events.publish_all(submitted.events);
        if let Some(scheduler) = &self.scheduler {
            scheduler.notify();
        }
        Ok(submitted.metadata_job)
    }

    fn metadata_identity(&self, ir_id: &str) -> Option<GenerationIdentity> {
        let implementation = crate::implementations::candidates_for_stage(KIND).next()?;
        let model = self.models.get(implementation.model)?;
        Some(GenerationIdentity {
            key_version: crate::jobs::YOUTUBE_METADATA_KEY_VERSION,
            ir_artifact_id: ir_id.into(),
            model_name: model.name.clone(),
            model_digest: format!("sha256:{}", model.digest()),
            prompt_digest: youtube_metadata_prompt_digest(),
            max_output_tokens: YOUTUBE_METADATA_MAX_OUTPUT_TOKENS,
        })
    }

    fn metadata_unavailable(&self) -> Option<String> {
        let implementation = crate::implementations::candidates_for_stage(KIND).next()?;
        if self.scheduler.is_none() {
            return Some("The local task scheduler is unavailable. Restart ClipMill or keep editing manually.".into());
        }
        let (present, missing) = self.storage.as_ref().map_or_else(
            || (false, vec![implementation.model.to_owned()]),
            |dirs| self.model_files_present(implementation.model, &dirs.weights),
        );
        let worker_present = self
            .roster
            .lock()
            .is_ok_and(|roster| roster.values().any(|worker| worker.serves(KIND)));
        let mut readiness = crate::service::stage_readiness(
            KIND,
            "editorial",
            implementation.name,
            implementation.model,
            implementation.backend,
            present,
            missing,
            worker_present,
        );
        crate::service::check_editorial_capacity(
            &mut readiness,
            self.scheduler
                .as_ref()
                .map(crate::jobs::SchedulerHandle::machine_capacity),
            self.models
                .get(implementation.model)
                .map(|model| model.memory.resident_bytes()),
        );
        (!readiness.ready).then_some(readiness.remedy)
    }

    async fn find_metadata_job(
        &self,
        project: &str,
        payload: &[u8],
        id: &str,
    ) -> Result<Option<JobRecord>, Error> {
        self.database
            .publishing(PublishingCommand::MetadataJob {
                project_id: project.into(),
                payload: payload.into(),
                job_id: id.into(),
                plan: None,
                retry_job_id: String::new(),
            })
            .await
            .map(|reply| reply.metadata_job)
            .map_err(|error| match error {
                crate::db::StoreError::NotFound => Error::Invalid(
                    "The metadata suggestion does not belong to this rendered clip and model.",
                ),
                _ => Error::Protocol,
            })
    }

    async fn describe_metadata_job(
        &self,
        context: &mut MetadataContext,
        identity: &GenerationIdentity,
        job: &JobRecord,
    ) {
        context.draft.generation_job_id.clone_from(&job.job_id);
        let (state, message) = match JobState::try_from(job.state) {
            Ok(JobState::Succeeded) => {
                match self.read_generated_metadata(job, identity, context.source_link.as_deref()).await {
                    Ok(metadata) => {
                        context.draft.generated_metadata = Some(metadata);
                        ("succeeded", "Suggestion ready. Review it before applying.".into())
                    }
                    Err(_) => ("unavailable", "The saved Qwen suggestion failed its integrity or content checks. Continue editing the metadata manually; your existing fields are unchanged.".into()),
                }
            }
            Ok(JobState::Failed) if job.failure_detail.starts_with("circuit breaker open for ") => (
                "unavailable",
                "Qwen repeatedly could not write metadata for this saved clip. Further identical retries are stopped. Continue editing the metadata manually.".into(),
            ),
            Ok(JobState::Failed) => (
                "failed",
                format!(
                    "Qwen could not finish this suggestion. {}",
                    safe_detail(&job.failure_detail)
                ),
            ),
            Ok(JobState::Cancelled | JobState::CancelRequested) => (
                "cancelled",
                "Generation cancelled. Your editable metadata is unchanged.".into(),
            ),
            Ok(JobState::Running)
                if job
                    .tasks
                    .iter()
                    .any(|task| task.state == TaskState::Running as i32) =>
            {
                (
                    "running",
                    "Qwen is writing metadata from this rendered clip’s saved transcript.".into(),
                )
            }
            _ => {
                let message = self.metadata_unavailable().unwrap_or_else(|| {
                    "Waiting for the local Qwen worker. Other model work may finish first.".into()
                });
                ("queued", message)
            }
        };
        state.clone_into(&mut context.draft.generation_state);
        context.draft.generation_message = message;
    }

    async fn read_generated_metadata(
        &self,
        job: &JobRecord,
        identity: &GenerationIdentity,
        source: Option<&str>,
    ) -> Result<YoutubeVideoMetadataV1, Error> {
        if job.kind != KIND || job.tasks.len() != 1 {
            return Err(Error::Protocol);
        }
        let task = &job.tasks[0];
        if task.kind != KIND
            || task.output_kind != OUTPUT
            || task.state != TaskState::Succeeded as i32
        {
            return Err(Error::Protocol);
        }
        let artifact = task
            .output_artifact_id
            .parse()
            .map_err(|_| Error::Protocol)?;
        let lease = self
            .artifacts
            .as_ref()
            .ok_or(Error::Protocol)?
            .open(artifact)
            .await
            .map_err(|_| Error::Protocol)?;
        if lease.kind() != OUTPUT {
            return Err(Error::Protocol);
        }
        let document = tokio::task::spawn_blocking(move || {
            let mut file = lease
                .open_verified(&"metadata.json".parse().map_err(|_| Error::Protocol)?)
                .map_err(|_| Error::Protocol)?;
            let mut bytes = Vec::new();
            file.by_ref()
                .take(32 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| Error::Protocol)?;
            if bytes.len() > 32 * 1024 {
                return Err(Error::Protocol);
            }
            serde_json::from_slice::<GeneratedDocument>(&bytes).map_err(|_| Error::Protocol)
        })
        .await
        .map_err(|_| Error::Protocol)??;
        validated_metadata(document, identity, source)
    }
}

fn unavailable(draft: &mut DraftYoutubeMetadataResponse, message: &str) {
    "unavailable".clone_into(&mut draft.generation_state);
    message.clone_into(&mut draft.generation_message);
}
fn safe_detail(message: &str) -> String {
    message
        .chars()
        .filter(|ch| !ch.is_control())
        .take(500)
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedDocument {
    schema_version: String,
    ir_artifact_id: String,
    producer: GeneratedProducer,
    metadata: GeneratedMetadata,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedProducer {
    implementation: String,
    model: GeneratedModel,
    prompt_digest: String,
    max_output_tokens: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedModel {
    name: String,
    digest: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedMetadata {
    title: String,
    description: String,
    tags: Vec<String>,
    hashtags: Vec<String>,
}

fn validated_metadata(
    document: GeneratedDocument,
    identity: &GenerationIdentity,
    source: Option<&str>,
) -> Result<YoutubeVideoMetadataV1, Error> {
    let producer = document.producer;
    if document.schema_version != "clipmill.publishing.metadata.v1"
        || document.ir_artifact_id != identity.ir_artifact_id
        || producer.implementation != IMPLEMENTATION
        || producer.model.name != identity.model_name
        || producer.model.digest != identity.model_digest
        || producer.prompt_digest != identity.prompt_digest
        || producer.max_output_tokens != identity.max_output_tokens
    {
        return Err(Error::Protocol);
    }
    let GeneratedMetadata {
        title,
        mut description,
        tags,
        hashtags,
    } = document.metadata;
    if !valid_text(&title, false)
        || !valid_text(&description, true)
        || title.chars().count() > 100
        || description.chars().count() > 2000
        || description.len() > 4000
        || tags.len() > 12
        || hashtags.len() > 3
        || tags.iter().any(|tag| {
            !valid_text(tag, false) || tag.chars().count() > 80 || tag.contains([',', '"', '#'])
        })
        || hashtags.iter().any(|tag| !valid_hashtag(tag))
        || has_duplicates(&tags)
        || has_duplicates(&hashtags)
    {
        return Err(Error::Protocol);
    }
    if let Some(source) = source {
        description.push_str("\n\nSource: ");
        description.push_str(source);
    }
    if !hashtags.is_empty() {
        description.push_str("\n\n");
        description.push_str(
            &hashtags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    let metadata = Metadata {
        title,
        description,
        tags,
        made_for_kids: false,
        contains_synthetic_media: false,
    };
    metadata.validate()?;
    Ok(YoutubeVideoMetadataV1 {
        title: metadata.title,
        description: metadata.description,
        tags: metadata.tags,
        made_for_kids: false,
        contains_synthetic_media: false,
    })
}

fn valid_text(text: &str, multiline: bool) -> bool {
    let lower = text.to_ascii_lowercase();
    !text.is_empty()
        && text.trim() == text
        && !text.contains(['<', '>'])
        && !text
            .chars()
            .any(|ch| ch.is_control() && !(multiline && ch == '\n'))
        && !lower.contains("http://")
        && !lower.contains("https://")
        && !lower.contains("www.")
}
fn valid_hashtag(tag: &str) -> bool {
    (2..=50).contains(&tag.len())
        && tag.as_bytes()[0].is_ascii_alphabetic()
        && tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
fn has_duplicates(values: &[String]) -> bool {
    values
        .iter()
        .map(|value| value.to_lowercase())
        .collect::<BTreeSet<_>>()
        .len()
        != values.len()
}

#[cfg(test)]
mod tests;
