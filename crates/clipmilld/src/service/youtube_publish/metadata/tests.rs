#![allow(clippy::unwrap_used)]
use super::*;
use serde_json::{Value, json};

fn identity() -> GenerationIdentity {
    GenerationIdentity {
        key_version: crate::jobs::YOUTUBE_METADATA_KEY_VERSION,
        ir_artifact_id: format!("sha256:{}", "11".repeat(32)),
        model_name: "qwen3-5-editorial-mlx".into(),
        model_digest: format!("sha256:{}", "22".repeat(32)),
        prompt_digest: youtube_metadata_prompt_digest(),
        max_output_tokens: YOUTUBE_METADATA_MAX_OUTPUT_TOKENS,
    }
}
fn output() -> Value {
    let identity = identity();
    json!({
        "schema_version":"clipmill.publishing.metadata.v1",
        "ir_artifact_id":identity.ir_artifact_id,
        "producer":{"implementation":IMPLEMENTATION,"model":{"name":identity.model_name,"digest":identity.model_digest},"prompt_digest":identity.prompt_digest,"max_output_tokens":identity.max_output_tokens},
        "metadata":{"title":"Why SQLite Keeps Data Local","description":"The speaker explains why local files remain easy to move.","tags":["SQLite","local data"],"hashtags":["SQLite","LocalData"]}
    })
}
fn validate(value: Value) -> Result<YoutubeVideoMetadataV1, Error> {
    validated_metadata(
        serde_json::from_value(value).map_err(|_| Error::Protocol)?,
        &identity(),
        None,
    )
}
#[test]
fn generated_suggestion_binds_the_frozen_edit_model_prompt_and_policy() {
    assert!(validate(output()).is_ok());
    for path in [
        "/ir_artifact_id",
        "/producer/model/name",
        "/producer/model/digest",
        "/producer/prompt_digest",
        "/producer/implementation",
        "/schema_version",
    ] {
        let mut document = output();
        *document.pointer_mut(path).unwrap() = json!("different");
        assert!(validate(document).is_err(), "{path} must match its lease");
    }
    let mut document = output();
    document["producer"]["max_output_tokens"] = json!(2048);
    assert!(validate(document).is_err());
    let mut document = output();
    document["metadata"]["made_for_kids"] = json!(true);
    assert!(
        validate(document).is_err(),
        "model cannot choose a disclosure"
    );
}
#[test]
fn source_attribution_is_daemon_supplied_and_hashtags_are_separate() {
    let metadata = validated_metadata(
        serde_json::from_value(output()).unwrap(),
        &identity(),
        Some("https://www.youtube.com/watch?v=abcdefghijk"),
    )
    .unwrap();
    assert!(
        metadata
            .description
            .ends_with("Source: https://www.youtube.com/watch?v=abcdefghijk\n\n#SQLite #LocalData")
    );
    assert!(!metadata.made_for_kids && !metadata.contains_synthetic_media);
    let mut forged = output();
    forged["metadata"]["description"] = json!("Visit https://example.test for the full story.");
    assert!(validate(forged).is_err());
}
#[test]
fn generated_content_rejects_control_urls_duplicates_and_byte_overflow() {
    for title in [
        "",
        " Wrong ",
        "A\nB",
        "Watch WWW.example.test",
        "<b>Title</b>",
    ] {
        let mut document = output();
        document["metadata"]["title"] = json!(title);
        assert!(validate(document).is_err());
    }
    let mut document = output();
    document["metadata"]["description"] = json!("😀".repeat(1001));
    assert!(
        validate(document).is_err(),
        "UTF-8 bytes matter independently of characters"
    );
    let mut document = output();
    document["metadata"]["tags"] = json!(["SQLite", "sqlite"]);
    assert!(validate(document).is_err());
    let mut document = output();
    document["metadata"]["tags"] = json!(vec!["é".repeat(80), "界".repeat(80), "字".repeat(80)]);
    assert!(
        validate(document).is_err(),
        "combined tags must fit upload limits"
    );
    for hashtags in [
        json!(["#SQLite"]),
        json!(["SQL ite"]),
        json!(["SQLite", "sqlite"]),
        json!(["One", "Two", "Three", "Four"]),
    ] {
        let mut document = output();
        document["metadata"]["hashtags"] = hashtags;
        assert!(validate(document).is_err());
    }
}
#[test]
fn metadata_identity_has_no_export_request_or_time_and_tracks_edited_content() {
    let first = serde_json::to_vec(&identity()).unwrap();
    assert_eq!(first, serde_json::to_vec(&identity()).unwrap());
    let text = String::from_utf8(first.clone()).unwrap();
    assert!(!text.contains("export_job") && !text.contains("request") && !text.contains("time"));
    let mut changed = identity();
    changed.ir_artifact_id = format!("sha256:{}", "33".repeat(32));
    assert_ne!(first, serde_json::to_vec(&changed).unwrap());
}

#[tokio::test]
async fn unavailable_completed_artifact_preserves_manual_draft_without_retry_state() {
    let temp = tempfile::tempdir().unwrap();
    let actor =
        crate::db::DbActor::start(&temp.path().join("store.db"), &temp.path().join("backups"))
            .unwrap();
    let service = Service::new(actor.handle(), 1);
    let manual = YoutubeVideoMetadataV1 {
        title: "Current editable title".into(),
        description: "Current editable description".into(),
        ..Default::default()
    };
    let mut context = MetadataContext {
        project_id: "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
        ir_id: identity().ir_artifact_id,
        source_link: None,
        has_transcript: true,
        draft: DraftYoutubeMetadataResponse {
            metadata: Some(manual.clone()),
            ..Default::default()
        },
    };
    let job = JobRecord {
        job_id: clipmill_core::JobId::new().to_string(),
        project_id: context.project_id.clone(),
        source_id: None,
        kind: KIND.into(),
        state: JobState::Succeeded as i32,
        created_unix_millis: 1,
        updated_unix_millis: 2,
        tasks: Vec::new(),
        output_artifact_ids: Vec::new(),
        failure_class: 0,
        failure_detail: String::new(),
        export: None,
        content_profile: String::new(),
    };
    service
        .describe_metadata_job(&mut context, &identity(), &job)
        .await;
    assert_eq!(context.draft.generation_state, "unavailable");
    assert!(context.draft.generated_metadata.is_none());
    assert_eq!(context.draft.metadata, Some(manual));
    assert!(context.draft.generation_message.contains("manually"));
    drop(service);
    actor.shutdown().await.unwrap();
}

fn metadata_lease(at: u64) -> crate::jobs::LeaseRequest {
    crate::jobs::LeaseRequest {
        lease_id: clipmill_core::LeaseId::new().to_string(),
        daemon_epoch: ulid::Ulid::new().to_string(),
        now_unix_millis: at,
        expires_unix_millis: at + 15_000,
        worker_id: "metadata-worker-test".into(),
        capabilities: vec![KIND.into()],
        capacity: crate::jobs::ResourceCapacity {
            cpu_threads: 4,
            ram_bytes: 64 * 1024 * 1024 * 1024,
            disk_bytes: 1024 * 1024 * 1024,
            accelerator_mask: crate::jobs::accelerator_bit("metal").unwrap(),
            vram_bytes: 0,
        },
    }
}

#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "One sequential recovery scenario through the real database actor"
)]
async fn actual_repeated_deterministic_failures_stop_retry_and_keep_manual_metadata() {
    use clipmill_contracts::proto::worker::v1::FailureClass;
    let temp = tempfile::tempdir().unwrap();
    let actor =
        crate::db::DbActor::start(&temp.path().join("store.db"), &temp.path().join("backups"))
            .unwrap();
    let service = Service::new(actor.handle(), 1);
    let project = clipmill_core::ProjectId::new();
    service
        .database
        .create_project(
            "metadata-project".into(),
            [7; 32],
            crate::db::ProjectRecord {
                project_id: project.to_string(),
                name: "Metadata fixture".into(),
                created_unix_millis: 1,
            },
        )
        .await
        .unwrap();
    let models = crate::models::ModelRegistry::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry"),
    )
    .unwrap();
    let payload = serde_json::to_vec(&identity()).unwrap();
    let mut predecessor = String::new();
    for attempt in 0..4 {
        let at = 1000 + attempt * 100;
        let plan = JobPlan::youtube_metadata(
            &project,
            payload.clone(),
            identity().ir_artifact_id.parse().unwrap(),
            &models,
            at,
        )
        .unwrap();
        let job = service
            .database
            .publishing(PublishingCommand::MetadataJob {
                project_id: project.to_string(),
                payload: payload.clone(),
                job_id: String::new(),
                plan: Some(Box::new(plan)),
                retry_job_id: predecessor,
            })
            .await
            .unwrap()
            .metadata_job
            .unwrap();
        let leased = service
            .database
            .lease_next_task(metadata_lease(at + 1))
            .await
            .unwrap();
        if attempt < 3 {
            let task = leased.task.unwrap();
            service
                .database
                .fail_task(
                    task.lease_id,
                    FailureClass::Deterministic as i32,
                    "The generated reply is malformed".into(),
                    at + 2,
                )
                .await
                .unwrap();
        } else {
            assert!(
                leased.task.is_none(),
                "open breaker must not load the same failing model input again"
            );
            let terminal = service.database.get_job(job.job_id.clone()).await.unwrap();
            assert_eq!(terminal.state, JobState::Failed as i32);
            assert!(
                terminal
                    .failure_detail
                    .starts_with("circuit breaker open for ")
            );
            let manual = YoutubeVideoMetadataV1 {
                title: "My reviewed title".into(),
                ..Default::default()
            };
            let mut context = MetadataContext {
                project_id: project.to_string(),
                ir_id: identity().ir_artifact_id,
                source_link: None,
                has_transcript: true,
                draft: DraftYoutubeMetadataResponse {
                    metadata: Some(manual.clone()),
                    ..Default::default()
                },
            };
            service
                .describe_metadata_job(&mut context, &identity(), &terminal)
                .await;
            assert_eq!(context.draft.generation_state, "unavailable");
            assert!(context.draft.generation_message.contains("manually"));
            assert_eq!(context.draft.metadata, Some(manual));
        }
        predecessor = job.job_id;
    }
    drop(service);
    actor.shutdown().await.unwrap();
}
