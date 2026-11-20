//! The decision stages share one graph between an original analysis and a new brief.
//! Source observations are immutable. A reprompt borrows the original observations
//! and roots the new cuts alongside them in a new analysis manifest.

use super::{
    AnalysisStagePayloadV1, AnalyzeSourcePayloadV1, BTreeMap, Builtin, EditorialStagePayloadV1,
    JobId, JobPlan, Message, ProjectId, TaskId, TaskSpec, analysis, builtin_task,
    editorial_duration, editorial_worker_task, mind,
};
use crate::{models::ModelRegistry, selection::Bindings};

const CUT_INPUTS: &[&str] = &[
    "timeline.index.v1",
    "mind.script.v1",
    "mind.memory.v1",
    "mind.unit_scores.v1",
];

const DECISIONS: &[(&str, &[&str])] = &[
    (
        "judge",
        &["mind.script.v1", "mind.memory.v1", "mind.candidates.v1"],
    ),
    (
        "plan",
        &[
            "mind.script.v1",
            "mind.memory.v1",
            "mind.candidates.v1",
            "mind.judgments.v1",
        ],
    ),
    (
        "critic",
        &[
            "mind.script.v1",
            "mind.memory.v1",
            "mind.candidates.v1",
            "mind.judgments.v1",
            "mind.plans.v1",
        ],
    ),
    (
        "rank",
        &["mind.script.v1", "mind.memory.v1", "mind.quality_report.v1"],
    ),
];

pub(super) fn validate(request: &AnalyzeSourcePayloadV1) -> Result<(), &'static str> {
    if request.editorial_prompt.chars().count() > 1000 {
        return Err("The editorial brief must fit in 1000 characters");
    }
    if request.editorial_taste.len() > 12
        || request
            .editorial_taste
            .iter()
            .any(|example| example.chars().count() > 240)
    {
        return Err("Use at most twelve short local preference examples");
    }
    Ok(())
}

/// Attach the cutter and the four decision stages (judge, plan, critic, rank),
/// reading each input either from this job's task graph or from an older analysis. Only dependencies are declared as input kinds:
/// the daemon separately leases and verifies pinned artifact addresses.
pub(super) fn build(
    request: &AnalyzeSourcePayloadV1,
    models: &ModelRegistry,
    bindings: &Bindings,
    stages: &mut Vec<(String, String)>,
    existing: &BTreeMap<String, String>,
) -> Result<Vec<TaskSpec>, &'static str> {
    let cut = builtin_task(
        TaskId::new().to_string(),
        Builtin {
            kind: mind::KIND_CUT,
            output_kind: "mind.candidates.v1",
            implementation: mind::CUT_IMPLEMENTATION,
            input_kinds: Vec::new(),
            dependencies: Vec::new(),
            payload: EditorialStagePayloadV1 {
                key_version: mind::MIND_STAGE_KEY_VERSION.to_owned(),
                stage: mind::KIND_CUT.to_owned(),
                duration: Some(editorial_duration(request)),
                editorial_prompt: request.editorial_prompt.clone(),
                ..Default::default()
            }
            .encode_to_vec(),
            ram_mib: 256,
        },
    );
    let mut tasks = Vec::new();
    let mut attach = |mut task: TaskSpec, kinds: &[&str]| -> Result<(), &'static str> {
        for &kind in kinds {
            if let Some((_, id)) = stages.iter().find(|(k, _)| k == kind) {
                task.input_kinds.push(kind.to_owned());
                task.dependencies.push(id.clone());
            } else {
                let address = existing
                    .get(kind)
                    .ok_or("The selected analysis lacks a required editorial artifact")?;
                task.input_artifact_ids.push(address.clone());
            }
        }
        stages.push((task.output_kind.clone(), task.task_id.clone()));
        tasks.push(task);
        Ok(())
    };
    attach(cut, CUT_INPUTS)?;
    for &(operation, kinds) in DECISIONS {
        attach(
            editorial_worker_task(operation, Vec::new(), Vec::new(), request, models, bindings)?,
            kinds,
        )?;
    }
    Ok(tasks)
}

impl JobPlan {
    pub(crate) fn reprompt(
        project_id: &ProjectId,
        source_id: String,
        (artifacts, mut manifest): (&BTreeMap<String, String>, AnalysisStagePayloadV1),
        request: &AnalyzeSourcePayloadV1,
        models: &ModelRegistry,
        bindings: &Bindings,
        now: u64,
    ) -> Result<Self, &'static str> {
        validate(request)?;
        if request.cloud_editorial.is_some() || !request.local_editorial {
            return Err("Choose local editorial for a new brief");
        }
        let mut stages = Vec::new();
        let mut tasks = build(request, models, bindings, &mut stages, artifacts)?;
        let carried: Vec<_> = artifacts
            .iter()
            .filter(|(kind, _)| !stages.iter().any(|(fresh, _)| fresh == *kind))
            .collect();
        super::ANALYSIS_STAGE_KEY_VERSION.clone_into(&mut manifest.key_version);
        analysis::KIND_MANIFEST.clone_into(&mut manifest.stage);
        manifest
            .skipped
            .retain(|s| !stages.iter().any(|(fresh, _)| fresh == &s.kind));
        let mut root = builtin_task(
            TaskId::new().to_string(),
            Builtin {
                kind: analysis::KIND_MANIFEST,
                output_kind: "analysis.manifest.v1",
                implementation: analysis::IMPLEMENTATION,
                input_kinds: stages.iter().map(|(kind, _)| kind.clone()).collect(),
                dependencies: stages.into_iter().map(|(_, id)| id).collect(),
                payload: manifest.encode_to_vec(),
                ram_mib: 64,
            },
        );
        root.input_artifact_ids = carried
            .into_iter()
            .map(|(_, address)| address.clone())
            .collect();
        root.is_final = true;
        tasks.push(root);
        for (ordinal, task) in tasks.iter_mut().enumerate() {
            task.ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
        }
        Ok(Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: "reprompt".to_owned(),
            source_id: Some(source_id),
            payload: request.encode_to_vec(),
            created_unix_millis: now,
            tasks,
        })
    }
}
