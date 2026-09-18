//! Real socket authorization of published render and filmstrip inventories.
//! Completed media tasks are stopped-store fixtures; this tests media
//! inventory and ownership, not encoding or WebView playback.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::{collections::BTreeMap, io::Write};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::ipc::v1::{
    ErrorCode, JobState, ReadArtifactRequest, Request, ResolveMediaRequest, ResolveMediaResponse,
    TaskState, request, response,
};
use clipmill_core::{ArtifactId, JobId, ProjectId, Sha256Digest, TaskId};
use clipmilld::{ArtifactCoordinator, Config, Daemon};
use rusqlite::{Connection, params};
use serde_json::{Map, json};
use sha2::{Digest, Sha256};
use support::{create, send, wait_until_ready, workspace_tempdir};
use tokio::sync::oneshot;

async fn publish_filmstrip(artifacts: &ArtifactCoordinator, project: &ProjectId) -> ArtifactId {
    publish_filmstrip_payloads(
        artifacts,
        project,
        include_bytes!("../../../contracts/fixtures/media.filmstrip/valid/minimal.json"),
        0,
    )
    .await
}

async fn publish_filmstrip_payloads(
    artifacts: &ArtifactCoordinator,
    project: &ProjectId,
    descriptor: &[u8],
    tile_count: usize,
) -> ArtifactId {
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "media.filmstrip.v1".to_owned(),
        source_fingerprint: Sha256Digest::from_bytes([42; 32]),
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: "ingest-filmstrip".to_owned(),
            implementation: "media-test@1".to_owned(),
            model_digest: None,
        },
        inputs: vec![],
        policy: NetworkPolicy::LocalLock,
        config: Map::from_iter([(
            "descriptor_digest".to_owned(),
            hex::encode(Sha256::digest(descriptor)).into(),
        )]),
        semantic_version: "1.0.0".to_owned(),
    })
    .expect("filmstrip recipe");
    let PrepareOutcome::Miss(staging) = artifacts.prepare(recipe).await.expect("prepare filmstrip")
    else {
        panic!("fresh filmstrip fixture")
    };
    let path: ArtifactPath = "index.json".parse().expect("descriptor path");
    staging
        .create_file(&path)
        .expect("create descriptor")
        .write_all(descriptor)
        .expect("write descriptor");
    let mut paths = vec![path];
    for index in 0..tile_count {
        let path: ArtifactPath = format!("tile_{index:05}.jpg").parse().expect("tile path");
        // Deliberately opaque bytes: this exercises inventory, not image decoding.
        staging
            .create_file(&path)
            .expect("tile file")
            .write_all(&vec![42; index % 17 + 1])
            .expect("write tile");
        paths.push(path);
    }
    artifacts
        .publish_project(
            project.clone(),
            staging.id().clone(),
            paths,
            BTreeMap::new(),
        )
        .await
        .expect("publish filmstrip descriptor")
        .artifact_id()
}

async fn publish_render(
    artifacts: &ArtifactCoordinator,
    project: &ProjectId,
    missing_video: bool,
) -> ArtifactId {
    let files: &[(&str, &[u8])] = &[
        ("clip.mp4", b"video fixture bytes"),
        (
            "clip.ass",
            b"[Script Info]\nTitle: internal burn-in input\n",
        ),
        ("clip.srt", b"1\n00:00:00,000 --> 00:00:01,000\nCaption\n"),
        ("clip.vtt", b"WEBVTT\n\n00:00.000 --> 00:01.000\nCaption\n"),
    ];
    let mut descriptor: serde_json::Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/render.clip/valid/first_slice.json"
    ))
    .expect("render contract fixture");
    descriptor["outputs"] = files.iter().map(|(path,bytes)| json!({
        "path":path,"bytes":bytes.len(),"sha256":format!("sha256:{}",hex::encode(Sha256::digest(bytes)))
    })).collect::<Vec<_>>().into();
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "render.clip.v1".to_owned(),
        source_fingerprint: Sha256Digest::from_bytes([42; 32]),
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: "render-clip".to_owned(),
            implementation: "media-test@1".to_owned(),
            model_digest: None,
        },
        inputs: vec![],
        policy: NetworkPolicy::LocalLock,
        config: Map::from_iter([("missing_video".to_owned(), missing_video.into())]),
        semantic_version: "1.0.0".to_owned(),
    })
    .expect("render fixture recipe");
    let PrepareOutcome::Miss(staging) = artifacts.prepare(recipe).await.expect("prepare render")
    else {
        panic!("fresh fixture")
    };
    let mut paths = Vec::new();
    for (name, bytes) in files {
        if missing_video && *name == "clip.mp4" {
            continue;
        }
        let path: ArtifactPath = name.parse().expect("fixture path");
        staging
            .create_file(&path)
            .expect("create payload")
            .write_all(bytes)
            .expect("write payload");
        paths.push(path);
    }
    let path: ArtifactPath = "render-manifest.json".parse().expect("descriptor path");
    staging
        .create_file(&path)
        .expect("create descriptor")
        .write_all(&serde_json::to_vec(&descriptor).expect("descriptor bytes"))
        .expect("write descriptor");
    paths.push(path);
    artifacts
        .publish_project(
            project.clone(),
            staging.id().clone(),
            paths,
            BTreeMap::new(),
        )
        .await
        .expect("publish render fixture")
        .artifact_id()
}

fn seed_outputs(config: &Config, project: &ProjectId, artifacts: &[(ArtifactId, &str)]) {
    let mut db = Connection::open(&config.paths.database).expect("stopped fixture database");
    let tx = db.transaction().expect("seed publication");
    let job = JobId::new().to_string();
    tx.execute("INSERT INTO jobs(job_id,project_id,kind,payload,state,created_unix_millis,updated_unix_millis)
        VALUES(?1,?2,'media-fixture',X'',?3,1,1)",params![job,project.to_string(),JobState::Succeeded as i32]).expect("media job");
    for (ordinal, (artifact, kind)) in artifacts.iter().enumerate() {
        tx.execute(
            "INSERT INTO tasks(task_id,job_id,ordinal,kind,input_kinds_json,output_kind,payload,
          input_key,state,cpu_threads,ram_bytes,accelerator_class,vram_bytes,disk_bytes,
          network_policy,thermal_class,determinism_class,checkpoint_support,preemption_cost,
          implementation,max_attempts,next_attempt_unix_millis,is_final,output_artifact_id,
          created_unix_millis,updated_unix_millis)
         VALUES(?1,?2,?3,'media-fixture','[]',?7,X'',?4,?5,
          1,0,'',0,0,'local-lock','burst','deterministic',0,0,'media-test@1',1,0,1,?6,1,1)",
            params![
                TaskId::new().to_string(),
                job,
                i64::try_from(ordinal).expect("small ordinal"),
                "00".repeat(32),
                TaskState::Succeeded as i32,
                artifact.to_string(),
                kind
            ],
        )
        .expect("media output");
    }
    tx.commit().expect("commit publication");
}

#[tokio::test]
async fn render_with_ass_resolves_playable_inventory_and_keeps_ownership_and_manifest_checks() {
    let temp = workspace_tempdir();
    let config = Config::from_sources(
        Some(temp.path().to_path_buf()),
        None,
        None,
        None,
        temp.path().to_path_buf(),
    )
    .expect("isolated media config");
    let daemon = Daemon::start(config.clone()).await.expect("daemon starts");
    let artifacts = daemon.artifact_coordinator();
    let (stop, stopped) = oneshot::channel();
    let running = tokio::spawn(daemon.serve_until(async {
        let _ = stopped.await;
    }));
    wait_until_ready(&config.paths.socket).await.expect("ready");
    let project: ProjectId = create(&config.paths.socket, "owner", "Owner")
        .await
        .expect("owner")
        .project_id
        .parse()
        .expect("id");
    let other = create(&config.paths.socket, "other", "Other")
        .await
        .expect("other")
        .project_id;
    let render = publish_render(&artifacts, &project, false).await;
    let incomplete = publish_render(&artifacts, &project, true).await;
    drop(artifacts);
    stop.send(()).expect("stop");
    running.await.expect("join").expect("stop daemon");
    seed_outputs(
        &config,
        &project,
        &[(render, "render.clip.v1"), (incomplete, "render.clip.v1")],
    );

    let daemon = Daemon::start(config.clone())
        .await
        .expect("daemon restarts");
    let (stop, stopped) = oneshot::channel();
    let running = tokio::spawn(daemon.serve_until(async {
        let _ = stopped.await;
    }));
    wait_until_ready(&config.paths.socket).await.expect("ready");
    for (project_id, artifact_id, expected) in [
        (project.to_string(), render, None),
        (other, render, Some(ErrorCode::NotFound)),
        (project.to_string(), incomplete, Some(ErrorCode::Internal)),
    ] {
        let reply = send(
            &config.paths.socket,
            Request {
                request_id: format!("resolve-{project_id}-{artifact_id}"),
                body: Some(request::Body::ResolveMedia(ResolveMediaRequest {
                    project_id,
                    artifact_id: artifact_id.to_string(),
                })),
            },
        )
        .await
        .expect("real socket reply");
        match (expected, reply.body) {
            (None, Some(response::Body::ResolveMedia(media))) => {
                assert_eq!(media.kind, "render.clip.v1");
                assert_eq!(
                    media
                        .files
                        .iter()
                        .map(|file| (file.path.as_str(), file.media_type.as_str()))
                        .collect::<Vec<_>>(),
                    vec![
                        ("clip.mp4", "video/mp4"),
                        ("clip.srt", "application/x-subrip"),
                        ("clip.vtt", "text/vtt")
                    ]
                );
                assert_eq!(media.files[0].bytes, 19);
            }
            (Some(code), Some(response::Body::Error(error))) => assert_eq!(error.code, code as i32),
            other => panic!("unexpected media authorization: {other:?}"),
        }
    }
    stop.send(()).expect("stop");
    running.await.expect("join").expect("stop daemon");
}

#[tokio::test]
async fn filmstrip_timing_document_is_readable_only_by_its_publishing_project() {
    let temp = workspace_tempdir();
    let config = Config::from_sources(
        Some(temp.path().to_path_buf()),
        None,
        None,
        None,
        temp.path().to_path_buf(),
    )
    .expect("isolated media config");
    let daemon = Daemon::start(config.clone()).await.expect("daemon starts");
    let artifacts = daemon.artifact_coordinator();
    let (stop, stopped) = oneshot::channel();
    let running = tokio::spawn(daemon.serve_until(async {
        let _ = stopped.await;
    }));
    wait_until_ready(&config.paths.socket).await.expect("ready");
    let project: ProjectId = create(&config.paths.socket, "owner", "Owner")
        .await
        .expect("owner")
        .project_id
        .parse()
        .expect("id");
    let other = create(&config.paths.socket, "other", "Other")
        .await
        .expect("other")
        .project_id;
    let filmstrip = publish_filmstrip(&artifacts, &project).await;
    drop(artifacts);
    stop.send(()).expect("stop");
    running.await.expect("join").expect("stop daemon");
    seed_outputs(&config, &project, &[(filmstrip, "media.filmstrip.v1")]);

    let daemon = Daemon::start(config.clone())
        .await
        .expect("daemon restarts");
    let (stop, stopped) = oneshot::channel();
    let running = tokio::spawn(daemon.serve_until(async {
        let _ = stopped.await;
    }));
    wait_until_ready(&config.paths.socket).await.expect("ready");
    for (project_id, owned) in [(project.to_string(), true), (other, false)] {
        let reply = send(
            &config.paths.socket,
            Request {
                request_id: format!("read-filmstrip-{project_id}"),
                body: Some(request::Body::ReadArtifact(ReadArtifactRequest {
                    project_id,
                    artifact_id: filmstrip.to_string(),
                    offset: 0,
                    length: 0,
                })),
            },
        )
        .await
        .expect("real descriptor socket reply");
        match (owned, reply.body) {
            (true, Some(response::Body::ReadArtifact(document))) => {
                assert_eq!(document.kind, "media.filmstrip.v1");
                assert_eq!(document.path, "index.json");
                assert_eq!(
                    document.chunk,
                    include_bytes!(
                        "../../../contracts/fixtures/media.filmstrip/valid/minimal.json"
                    )
                );
                let descriptor: serde_json::Value =
                    serde_json::from_slice(&document.chunk).expect("timing document");
                assert_eq!(descriptor["tiles"][1]["file"], "tile_00002.jpg");
                assert_eq!(descriptor["tiles"][1]["t_ticks"], 90_000);
            }
            (false, Some(response::Body::Error(error))) => {
                assert_eq!(error.code, ErrorCode::NotFound as i32);
            }
            other => panic!("unexpected descriptor authorization: {other:?}"),
        }
    }
    stop.send(()).expect("stop");
    running.await.expect("join").expect("stop daemon");
}

fn filmstrip_inventory(tile_count: usize, extra_file: Option<&str>) -> Vec<u8> {
    let mut descriptor: serde_json::Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/media.filmstrip/valid/minimal.json"
    ))
    .expect("filmstrip fixture");
    let mut tiles: Vec<_> = (0..tile_count)
        .map(|index| json!({"file": format!("tile_{index:05}.jpg"), "t_ticks": index * 90_000}))
        .collect();
    if let Some(file) = extra_file {
        tiles.push(json!({"file": file, "t_ticks": tile_count * 90_000}));
    }
    descriptor["coverage"]["end_ticks"] = (tiles.len() * 90_000).into();
    descriptor["tiles"] = tiles.into();
    serde_json::to_vec(&descriptor).expect("descriptor bytes")
}

fn assert_filmstrip_inventory(media: &ResolveMediaResponse, count: usize) {
    assert_eq!(media.files.len(), count);
    for (index, file) in media.files.iter().enumerate() {
        assert_eq!(file.path, format!("tile_{index:05}.jpg"));
        assert_eq!(file.bytes, u64::try_from(index % 17 + 1).expect("size"));
        assert_eq!(file.media_type, "image/jpeg");
    }
}

#[tokio::test]
async fn large_filmstrip_inventory_keeps_sizes_membership_path_and_ownership_checks() {
    const TILE_COUNT: usize = 2_680;
    let temp = workspace_tempdir();
    let config = Config::from_sources(
        Some(temp.path().to_path_buf()),
        None,
        None,
        None,
        temp.path().to_path_buf(),
    )
    .expect("isolated media config");
    let daemon = Daemon::start(config.clone()).await.expect("daemon starts");
    let artifacts = daemon.artifact_coordinator();
    let (stop, stopped) = oneshot::channel();
    let running = tokio::spawn(daemon.serve_until(async {
        let _ = stopped.await;
    }));
    wait_until_ready(&config.paths.socket).await.expect("ready");
    let project: ProjectId = create(&config.paths.socket, "owner", "Owner")
        .await
        .expect("owner")
        .project_id
        .parse()
        .expect("id");
    let other = create(&config.paths.socket, "other", "Other")
        .await
        .expect("other")
        .project_id;
    let large = publish_filmstrip_payloads(
        &artifacts,
        &project,
        &filmstrip_inventory(TILE_COUNT, None),
        TILE_COUNT,
    )
    .await;
    let missing = publish_filmstrip_payloads(
        &artifacts,
        &project,
        &filmstrip_inventory(1, Some("unpublished.jpg")),
        1,
    )
    .await;
    let invalid = publish_filmstrip_payloads(
        &artifacts,
        &project,
        &filmstrip_inventory(1, Some("../outside.jpg")),
        1,
    )
    .await;
    drop(artifacts);
    stop.send(()).expect("stop");
    running.await.expect("join").expect("stop daemon");
    seed_outputs(
        &config,
        &project,
        &[
            (large, "media.filmstrip.v1"),
            (missing, "media.filmstrip.v1"),
            (invalid, "media.filmstrip.v1"),
        ],
    );
    let daemon = Daemon::start(config.clone())
        .await
        .expect("daemon restarts");
    let (stop, stopped) = oneshot::channel();
    let running = tokio::spawn(daemon.serve_until(async {
        let _ = stopped.await;
    }));
    wait_until_ready(&config.paths.socket).await.expect("ready");
    for (project_id, artifact, error) in [
        (project.to_string(), large, None),
        (other, large, Some(ErrorCode::NotFound)),
        (project.to_string(), missing, Some(ErrorCode::Internal)),
        (project.to_string(), invalid, Some(ErrorCode::Internal)),
    ] {
        let reply = send(
            &config.paths.socket,
            Request {
                request_id: format!("resolve-large-{project_id}-{artifact}"),
                body: Some(request::Body::ResolveMedia(ResolveMediaRequest {
                    project_id,
                    artifact_id: artifact.to_string(),
                })),
            },
        )
        .await
        .expect("real inventory reply");
        match (error, reply.body) {
            (None, Some(response::Body::ResolveMedia(media))) => {
                assert_filmstrip_inventory(&media, TILE_COUNT);
            }
            (Some(code), Some(response::Body::Error(error))) => assert_eq!(error.code, code as i32),
            other => panic!("unexpected large inventory result: {other:?}"),
        }
    }
    stop.send(()).expect("stop");
    running.await.expect("join").expect("stop daemon");
}
