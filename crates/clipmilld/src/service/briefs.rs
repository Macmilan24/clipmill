//! New editorial briefs reuse the finished recording's observations.

use std::collections::BTreeMap;

use clipmill_contracts::proto::ipc::v1::{
    AnalysisStagePayloadV1, AnalyzeSourcePayloadV1, ClipDurationV1, Response, SkippedStageV1,
    response,
};
use prost::Message;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::api_v2::{
    ApiError, ApiResult, Run, arg_str, canonical_json, job_json, opt_str, requested_duration,
};
use super::{Service, unix_millis};
use crate::db::StoreError;
use crate::jobs::JobPlan;

pub(super) fn requested_count(args: &Value) -> Result<Option<u64>, ApiError> {
    args.get("count")
        .map(|v| {
            v.as_u64()
                .filter(|n| (1..=30).contains(n))
                .ok_or_else(|| ApiError::invalid("count must be between 1 and 30"))
        })
        .transpose()
}

pub(super) fn store_error(error: &StoreError) -> ApiError {
    match error {
        StoreError::Conflict => ApiError::new("conflict", error.to_string()),
        StoreError::NotFound => ApiError::not_found(error.to_string()),
        _ => ApiError::new("unavailable", error.to_string()),
    }
}

pub(super) fn submitted(bytes: &[u8]) -> ApiResult {
    if let Some(response::Body::SubmitJob(response)) =
        Response::decode(bytes).ok().and_then(|r| r.body)
        && let Some(job) = response.job
    {
        return Ok(job_json(&job));
    }
    Err(ApiError::new("internal", "the saved job could not be read"))
}

impl Service {
    pub(super) async fn run_manifest(
        &self,
        run: &Run,
    ) -> Result<(BTreeMap<String, String>, AnalysisStagePayloadV1), ApiError> {
        let id = run
            .artifact("analysis.manifest.v1")
            .ok_or_else(|| ApiError::not_found("this run has no analysis manifest"))?;
        let doc = self.document(&run.project_id, id).await?;
        let fingerprint = doc["source_fingerprint"]
            .as_str()
            .ok_or_else(|| ApiError::invalid("missing recording fingerprint"))?;
        let entries = doc["stages"]
            .as_array()
            .ok_or_else(|| ApiError::invalid("missing analysis stages"))?;
        let mut artifacts = BTreeMap::new();
        for entry in entries {
            let kind = arg_str(entry, "kind")?;
            let address = arg_str(entry, "artifact_id")?;
            if address.parse::<clipmill_core::ArtifactId>().is_err()
                || artifacts
                    .insert(kind.to_owned(), address.to_owned())
                    .is_some()
            {
                return Err(ApiError::invalid(
                    "invalid or duplicate stage in the analysis manifest",
                ));
            }
        }
        let skipped = doc["skipped"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|s| {
                Ok(SkippedStageV1 {
                    kind: arg_str(s, "kind")?.to_owned(),
                    reason: arg_str(s, "reason")?.to_owned(),
                })
            })
            .collect::<Result<_, ApiError>>()?;
        Ok((
            artifacts,
            AnalysisStagePayloadV1 {
                source_fingerprint: fingerprint.to_owned(),
                skipped,
                ..Default::default()
            },
        ))
    }

    pub(super) async fn reprompt(&self, args: &Value, id: Option<String>) -> ApiResult {
        let id = id.unwrap_or_else(|| format!("v2_{}", ulid::Ulid::new()));
        let hash: [u8; 32] = Sha256::digest(canonical_json(args).as_bytes()).into();
        if let Some(bytes) = self
            .database
            .replay_request(id.clone(), hash)
            .await
            .map_err(|error| store_error(&error))?
        {
            return submitted(&bytes);
        }
        let run = self.run(args).await?;
        let (artifacts, manifest) = self.run_manifest(&run).await?;
        let source = self
            .database
            .get_source(run.source_id.clone())
            .await
            .map_err(|error| store_error(&error))?;
        if source.project_id != run.project_id
            || source.source_fingerprint != manifest.source_fingerprint
        {
            return Err(ApiError::new(
                "conflict",
                "the selected recording has changed since this analysis",
            ));
        }
        let previous = run.job.analysis.unwrap_or_default();
        // A round of answers keeps the brief the earlier run had.
        let prompt = match opt_str(args, "prompt") {
            Some(prompt) => prompt.trim().to_owned(),
            None if args.get("answers").is_some() => previous.editorial_prompt.clone(),
            None => return Err(ApiError::invalid("missing string argument `prompt`")),
        };
        let judge_answers = super::api_v2::judge_answers(args, &previous.judge_answers)?;
        let duration = requested_duration(args)?.or_else(|| {
            (previous.max_ticks > 0).then_some(ClipDurationV1 {
                min_ticks: previous.min_ticks,
                max_ticks: previous.max_ticks,
            })
        });
        let project = run
            .project_id
            .parse()
            .map_err(|_| ApiError::invalid("invalid project"))?;
        let request = AnalyzeSourcePayloadV1 {
            key_version: super::ANALYZE_SOURCE_KEY_VERSION.to_owned(),
            source_id: run.source_id.clone(),
            duration,
            count: requested_count(args)?.unwrap_or(previous.count),
            language: previous.language,
            content_profile: run.job.content_profile,
            local_editorial: true,
            editorial_prompt: prompt,
            judge_answers,
            editorial_taste: self
                .database
                .editorial_taste(run.project_id)
                .await
                .map_err(|error| store_error(&error))?,
            ..Default::default()
        };
        let now = unix_millis().map_err(|e| ApiError::new("internal", e))?;
        let plan = JobPlan::reprompt(
            &project,
            run.source_id,
            (&artifacts, manifest),
            &request,
            &self.models,
            &self.planning_bindings(),
            now,
        )
        .map_err(ApiError::invalid)?;
        self.enqueue_engine_job(id, hash, plan).await
    }

    pub(super) async fn enqueue_engine_job(
        &self,
        id: String,
        hash: [u8; 32],
        plan: JobPlan,
    ) -> ApiResult {
        let result = self
            .database
            .submit_job(id, hash, plan, self.engine_actor.clone())
            .await
            .map_err(|error| store_error(&error))?;
        self.events.publish_all(result.events);
        if let Some(scheduler) = &self.scheduler {
            scheduler.notify();
        }
        submitted(&result.bytes)
    }

    pub(super) async fn create_from_plan(&self, args: &Value, id: Option<String>) -> ApiResult {
        if args.get("plan").is_none() {
            return self.create_clip(args, id).await;
        }
        let run = self.run(args).await?;
        let mut plan = args["plan"].clone();
        let plan_id = arg_str(&plan, "plan_id")?.to_owned();
        if opt_str(args, "clip_id").is_some_and(|id| id != plan_id) {
            return Err(ApiError::invalid("clip_id and plan_id must agree"));
        }
        plan.sort_all_objects();
        self.direct_api_plan(&run, args, &plan_id, plan.to_string(), id)
            .await
    }
}
