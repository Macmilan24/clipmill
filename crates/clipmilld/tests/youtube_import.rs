//! Explicit acquisition uses a deterministic local helper fixture here. The
//! socket, persistence, pinned media inspection and normal ingest are real.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

use clipmill_contracts::proto::ipc::v1::{
    DeleteProjectRequest, ErrorCode, GetLocalLockRequest, GetSourceRequest,
    GetYoutubeImportRequest, JobState, Request, StartYoutubeImportRequest,
    UpdateYoutubeImportRequest, YoutubeImportV1, request, response,
};
use fs2::FileExt;
use support::{
    create, get_job, list, send, signal_terminate, storage_stats, submit_ingest, wait_until_ready,
    workspace_tempdir,
};
use tempfile::TempDir;
use tokio::time::{sleep, timeout};

fn tool(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.cache/bin")
        .join(name)
        .canonicalize()
        .expect("run tools/fetch-ffmpeg.sh")
}

struct Harness {
    root: TempDir,
    child: Option<Child>,
    socket: PathBuf,
}
impl Harness {
    fn new() -> Self {
        let root = workspace_tempdir();
        let status = Command::new(tool("ffmpeg"))
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=320x180:rate=24:duration=2",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=2",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-c:a",
                "aac",
                "-shortest",
            ])
            .arg(root.path().join("fixture.mkv"))
            .status()
            .unwrap();
        assert!(status.success());
        fs::write(root.path().join("helper.py"), HELPER).unwrap();
        fs::set_permissions(
            root.path().join("helper.py"),
            fs::Permissions::from_mode(0o700),
        )
        .unwrap();
        let socket = root.path().join("daemon.sock");
        Self {
            root,
            socket,
            child: None,
        }
    }
    async fn start(&mut self) {
        self.child = Some(
            Command::new(env!("CARGO_BIN_EXE_clipmilld"))
                .arg("--data-dir")
                .arg(self.root.path().join("data"))
                .arg("--socket")
                .arg(&self.socket)
                .env("CLIPMILL_FFPROBE", tool("ffprobe"))
                .env(
                    "CLIPMILL_YOUTUBE_IMPORTER",
                    self.root.path().join("helper.py"),
                )
                .env("RUST_LOG", "error")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        );
        wait_until_ready(&self.socket).await.unwrap();
    }
    async fn stop(&mut self) {
        let mut child = self.child.take().unwrap();
        signal_terminate(&child).unwrap();
        timeout(Duration::from_secs(15), async {
            while child.try_wait().unwrap().is_none() {
                sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
    }
    fn crash(&mut self) {
        let mut child = self.child.take().unwrap();
        child.kill().unwrap();
        child.wait().unwrap();
    }
    fn launches(&self) -> usize {
        fs::read_to_string(self.root.path().join("launches"))
            .unwrap_or_default()
            .lines()
            .count()
    }
    fn hold(&self, value: bool) {
        let path = self.root.path().join("hold");
        if value {
            fs::write(path, "hold").unwrap();
        } else {
            fs::remove_file(path).unwrap();
        }
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

async fn call(socket: &Path, id: &str, body: request::Body) -> response::Body {
    send(
        socket,
        Request {
            request_id: id.into(),
            body: Some(body),
        },
    )
    .await
    .unwrap()
    .body
    .unwrap()
}
fn imported(body: response::Body) -> YoutubeImportV1 {
    match body {
        response::Body::YoutubeImport(value) => value.record.unwrap(),
        other => panic!("expected import: {other:?}"),
    }
}
fn start(project: &str, rights: bool) -> request::Body {
    request::Body::StartYoutubeImport(StartYoutubeImportRequest {
        project_id: project.into(),
        url: "https://youtu.be/NYFGCESmikA?si=drop&list=drop".into(),
        rights_confirmed: rights,
        max_height: 360,
    })
}
async fn record(socket: &Path, id: &str) -> YoutubeImportV1 {
    imported(
        call(
            socket,
            "read",
            request::Body::GetYoutubeImport(GetYoutubeImportRequest {
                import_id: id.into(),
            }),
        )
        .await,
    )
}
async fn wait_state(socket: &Path, id: &str, state: &str) -> YoutubeImportV1 {
    timeout(Duration::from_secs(30), async {
        loop {
            let value = record(socket, id).await;
            if value.state == state {
                return value;
            }
            assert!(
                !matches!(value.state.as_str(), "failed" | "cancelled"),
                "unexpected outcome: {value:?}"
            );
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap()
}
async fn update(socket: &Path, request_id: &str, id: &str, action: &str) -> YoutubeImportV1 {
    imported(
        call(
            socket,
            request_id,
            request::Body::UpdateYoutubeImport(UpdateYoutubeImportRequest {
                import_id: id.into(),
                action: action.into(),
            }),
        )
        .await,
    )
}

#[tokio::test]
#[ignore = "requires pinned FFmpeg/FFprobe; exercised by ingest-drill"]
#[allow(
    clippy::too_many_lines,
    reason = "one real-socket source lifecycle across restart and deletion"
)]
async fn import_registers_a_durable_source_and_uses_the_normal_ingest_path() {
    let mut h = Harness::new();
    h.start().await;
    let project = create(&h.socket, "project", "YouTube · NYFGCESmikA")
        .await
        .unwrap();
    let denied = call(&h.socket, "denied", start(&project.project_id, false)).await;
    assert!(
        matches!(denied, response::Body::Error(value) if value.code == ErrorCode::InvalidArgument as i32)
    );
    assert_eq!(h.launches(), 0);
    let mut invalid_quality = StartYoutubeImportRequest {
        project_id: project.project_id.clone(),
        url: "https://youtu.be/NYFGCESmikA".into(),
        rights_confirmed: true,
        max_height: 2160,
    };
    let invalid = call(
        &h.socket,
        "invalid-quality",
        request::Body::StartYoutubeImport(invalid_quality.clone()),
    )
    .await;
    assert!(
        matches!(invalid, response::Body::Error(value) if value.code == ErrorCode::InvalidArgument as i32)
    );
    assert_eq!(h.launches(), 0);
    let started = imported(call(&h.socket, "start", start(&project.project_id, true)).await);
    assert_eq!(
        started.canonical_url,
        "https://www.youtube.com/watch?v=NYFGCESmikA"
    );
    assert_eq!(
        imported(call(&h.socket, "start", start(&project.project_id, true)).await),
        started
    );
    assert_eq!(
        imported(call(&h.socket, "duplicate", start(&project.project_id, true)).await).import_id,
        started.import_id
    );
    let completed = wait_state(&h.socket, &started.import_id, "completed").await;
    assert_eq!(completed.max_height, 360);
    invalid_quality.max_height = 720;
    let conflict = call(
        &h.socket,
        "different-quality",
        request::Body::StartYoutubeImport(invalid_quality),
    )
    .await;
    assert!(
        matches!(conflict, response::Body::Error(value) if value.code == ErrorCode::Conflict as i32 && value.message.contains("new project"))
    );
    assert_eq!(completed.title, "Fixture interview");
    assert_eq!(completed.channel, "Fixture creator");
    assert!(completed.downloaded_bytes > 0);
    assert_eq!(h.launches(), 1);
    let response::Body::GetSource(source) = call(
        &h.socket,
        "source",
        request::Body::GetSource(GetSourceRequest {
            source_id: completed.source_id.clone(),
        }),
    )
    .await
    else {
        panic!("source missing")
    };
    assert!(
        !source.source_map_json.is_empty(),
        "saved probe is available without another registration"
    );
    let source = source.source.unwrap();
    let managed = PathBuf::from(&source.absolute_path);
    assert!(managed.starts_with(h.root.path().join("data/imports").join(&project.project_id)));
    assert_eq!(source.project_id, project.project_id);
    assert_eq!(
        list(&h.socket, "projects").await.unwrap()[0].name,
        "Fixture interview"
    );
    let response::Body::GetLocalLock(lock) = call(
        &h.socket,
        "lock",
        request::Body::GetLocalLock(GetLocalLockRequest {}),
    )
    .await
    else {
        panic!("lock missing")
    };
    let status = lock.status.unwrap();
    assert!(!status.engaged);
    assert_eq!(status.egress_attempts, 1);
    let storage = storage_stats(&h.socket, "storage").await.unwrap();
    assert!(
        storage
            .categories
            .iter()
            .any(|category| category.key == "imports"
                && category.bytes >= completed.downloaded_bytes)
    );
    let job = submit_ingest(&h.socket, "ingest", &project.project_id, &source.source_id)
        .await
        .unwrap();
    assert_eq!(job.tasks.len(), 9);
    timeout(Duration::from_secs(90), async {
        loop {
            let current = get_job(&h.socket, "job", &job.job_id).await.unwrap();
            if current.state == JobState::Succeeded as i32 {
                break;
            }
            assert_ne!(current.state, JobState::Failed as i32, "{current:?}");
            sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap();
    h.stop().await;
    h.start().await;
    assert_eq!(
        record(&h.socket, &started.import_id).await.source_id,
        source.source_id
    );
    assert_eq!(h.launches(), 1, "restart must not download again");
    call(
        &h.socket,
        "delete",
        request::Body::DeleteProject(DeleteProjectRequest {
            project_id: project.project_id,
        }),
    )
    .await;
    assert!(
        !managed.exists(),
        "deleting a project removes its retained managed original"
    );
    assert!(
        h.root.path().join("fixture.mkv").exists(),
        "external files stay untouched"
    );
    let orphan = create(&h.socket, "orphan", "Orphan lease").await.unwrap();
    let orphan_dir = h.root.path().join("data/imports").join(&orphan.project_id);
    let attempt_dir = orphan_dir.join("yt_orphan/attempt-held");
    fs::create_dir_all(&attempt_dir).unwrap();
    let lease = fs::File::create(attempt_dir.join(".import.lock")).unwrap();
    lease.lock_exclusive().unwrap();
    fs::write(attempt_dir.join("partial"), b"pending").unwrap();
    call(
        &h.socket,
        "delete-held",
        request::Body::DeleteProject(DeleteProjectRequest {
            project_id: orphan.project_id,
        }),
    )
    .await;
    assert!(
        attempt_dir.exists(),
        "an orphan's live lease prevents deletion"
    );
    drop(lease);
    h.stop().await;
    h.start().await;
    assert!(
        !orphan_dir.exists(),
        "startup retries deletion even after import rows are gone"
    );
    h.stop().await;
}

#[tokio::test]
#[ignore = "requires pinned FFmpeg/FFprobe; exercised by ingest-drill"]
async fn cancel_retry_and_crash_recovery_never_reuse_an_old_attempt() {
    let mut h = Harness::new();
    h.hold(true);
    h.start().await;
    let project = create(&h.socket, "project", "Chosen title").await.unwrap();
    let started = imported(call(&h.socket, "start", start(&project.project_id, true)).await);
    wait_state(&h.socket, &started.import_id, "processing").await;
    assert_eq!(
        update(&h.socket, "cancel", &started.import_id, "cancel")
            .await
            .state,
        "cancelled"
    );
    h.hold(false);
    let retried = update(&h.socket, "retry", &started.import_id, "retry").await;
    assert_eq!(retried.attempt, 1);
    assert_eq!(retried.max_height, 360);
    assert_eq!(
        update(&h.socket, "retry", &started.import_id, "retry").await,
        retried
    );
    wait_state(&h.socket, &started.import_id, "completed").await;
    assert_eq!(h.launches(), 2);
    let project2 = create(&h.socket, "project2", "Recovery").await.unwrap();
    h.hold(true);
    let second = imported(call(&h.socket, "start2", start(&project2.project_id, true)).await);
    wait_state(&h.socket, &second.import_id, "processing").await;
    h.crash();
    h.start().await;
    assert_eq!(
        record(&h.socket, &second.import_id).await.state,
        "interrupted"
    );
    sleep(Duration::from_millis(200)).await;
    assert_eq!(
        h.launches(),
        3,
        "recovery requires explicit retry before network work"
    );
    h.hold(false);
    update(&h.socket, "restart-retry", &second.import_id, "retry").await;
    let done = wait_state(&h.socket, &second.import_id, "completed").await;
    assert_eq!(done.attempt, 1);
    assert_eq!(h.launches(), 4);
    assert_eq!(
        list(&h.socket, "projects")
            .await
            .unwrap()
            .iter()
            .find(|p| p.project_id == project.project_id)
            .unwrap()
            .name,
        "Chosen title"
    );
    h.stop().await;
}

const HELPER: &str = r"#!/usr/bin/env python3
import fcntl,json,os,pathlib,shutil,signal,sys,threading,time
root=pathlib.Path(__file__).parent
def emit(value): print(json.dumps(value),flush=True)
if '--check' in sys.argv:
    emit({'event':'ready'})
    sys.exit(0)
parent=os.getppid()
def watch():
    while True:
        time.sleep(.05)
        if os.getppid()!=parent: os.killpg(os.getpgrp(),signal.SIGKILL)
threading.Thread(target=watch,daemon=True).start()
directory=pathlib.Path(sys.argv[sys.argv.index('--destination')+1])
assert sys.argv[sys.argv.index('--max-height')+1]=='360', 'quality not passed to helper'
lease=open(directory/'.import.lock','w')
fcntl.flock(lease,fcntl.LOCK_EX)
with open(root/'launches','a') as log: log.write(str(directory)+'\n')
emit({'event':'metadata','title':'Fixture interview','channel':'Fixture creator'})
emit({'event':'processing'})
while (root/'hold').exists(): time.sleep(.02)
shutil.copyfile(root/'fixture.mkv',directory/'source.mkv')
emit({'event':'complete','file_name':'source.mkv','video_id':'NYFGCESmikA','title':'Fixture interview','channel':'Fixture creator','duration_seconds':2,'byte_size':(directory/'source.mkv').stat().st_size})
";
