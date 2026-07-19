#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_core::{ArtifactId, ProjectId, Sha256Digest};
use clipmilld::{Config, Daemon};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use tokio::{sync::oneshot, time::sleep};

use support::{create, list, wait_until_ready, workspace_tempdir};

const PAYLOAD_BYTES: usize = 1024 * 1024;
const PAYLOAD_NAME: &str = "payload.bin";

#[derive(Clone, Debug, Eq, PartialEq)]
struct Acknowledgement {
    project_id: ProjectId,
    artifact_id: ArtifactId,
    iteration: usize,
    sequence: usize,
}

fn config(data_dir: &Path, socket: &Path) -> Config {
    Config::from_sources(
        Some(data_dir.to_path_buf()),
        Some(socket.to_path_buf()),
        Some(OsString::from("/ignored/env")),
        None,
        PathBuf::from("/ignored/default"),
    )
    .expect("cache drill config")
}

fn recipe(iteration: usize, sequence: usize) -> ArtifactRecipe {
    let mut fingerprint = Sha256::new();
    fingerprint.update(format!("clipmill-cache-drill:{iteration}:{sequence}"));
    let mut config = Map::new();
    config.insert(
        "iteration".to_owned(),
        Value::from(u64::try_from(iteration).expect("iteration fits u64")),
    );
    config.insert(
        "sequence".to_owned(),
        Value::from(u64::try_from(sequence).expect("sequence fits u64")),
    );
    ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "drill.cache.v1".to_owned(),
        source_fingerprint: Sha256Digest::from_bytes(fingerprint.finalize().into()),
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: "cache-drill".to_owned(),
            implementation: "clipmilld-cache-drill@1".to_owned(),
            model_digest: None,
        },
        inputs: Vec::new(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "1".to_owned(),
    })
    .expect("cache drill recipe")
}

fn payload(iteration: usize, sequence: usize) -> Vec<u8> {
    let pattern = format!("clipmill-cache-drill:{iteration}:{sequence}\n").into_bytes();
    let mut payload = Vec::with_capacity(PAYLOAD_BYTES);
    while payload.len() < PAYLOAD_BYTES {
        payload.extend_from_slice(&pattern);
    }
    payload.truncate(PAYLOAD_BYTES);
    payload
}

fn publish_acknowledgement(directory: &Path, acknowledgement: &Acknowledgement) {
    let name = format!(
        "ack-{:04}-{:08}.txt",
        acknowledgement.iteration, acknowledgement.sequence
    );
    let temporary = directory.join(format!(".{name}.tmp-{}", std::process::id()));
    let published = directory.join(name);
    let contents = format!(
        "{}\n{}\n{}\n{}\n",
        acknowledgement.project_id,
        acknowledgement.artifact_id,
        acknowledgement.iteration,
        acknowledgement.sequence
    );
    let mut options = OpenOptions::new();
    options.create_new(true).write(true).mode(0o600);
    let mut file = options.open(&temporary).expect("create acknowledgement");
    file.write_all(contents.as_bytes())
        .expect("write acknowledgement");
    file.sync_all().expect("sync acknowledgement");
    drop(file);
    fs::rename(&temporary, &published).expect("publish acknowledgement");
    File::open(directory)
        .expect("open acknowledgement directory")
        .sync_all()
        .expect("sync acknowledgement directory");
}

fn read_acknowledgements(directory: &Path) -> Vec<Acknowledgement> {
    let mut paths = fs::read_dir(directory)
        .expect("read acknowledgement directory")
        .map(|entry| entry.expect("acknowledgement entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "txt"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let contents = fs::read_to_string(path).expect("read acknowledgement");
            let fields = contents.lines().collect::<Vec<_>>();
            assert_eq!(fields.len(), 4, "complete acknowledgement record");
            Acknowledgement {
                project_id: fields[0].parse().expect("acknowledged project id"),
                artifact_id: fields[1].parse().expect("acknowledged artifact id"),
                iteration: fields[2].parse().expect("acknowledged iteration"),
                sequence: fields[3].parse().expect("acknowledged sequence"),
            }
        })
        .collect()
}

fn committed_artifact_ids(data_dir: &Path) -> Vec<ArtifactId> {
    let objects = data_dir.join("artifacts/objects/sha256");
    let mut ids = Vec::new();
    for prefix in fs::read_dir(objects).expect("read object prefixes") {
        let prefix = prefix.expect("object prefix").path();
        for object in fs::read_dir(prefix).expect("read object directory") {
            let object = object.expect("object entry").path();
            let digest = object
                .file_name()
                .and_then(|name| name.to_str())
                .expect("UTF-8 object name");
            ids.push(
                format!("sha256:{digest}")
                    .parse()
                    .expect("canonical object id"),
            );
        }
    }
    ids.sort_unstable();
    ids
}

fn spawn_cache_child(
    data_dir: &Path,
    socket: &Path,
    acknowledgements: &Path,
    iteration: usize,
) -> Child {
    Command::new(std::env::current_exe().expect("current test executable"))
        .arg("--ignored")
        .arg("--exact")
        .arg("cache_drill_child")
        .arg("--nocapture")
        .env("CLIPMILL_CACHE_CHILD", "1")
        .env("CLIPMILL_CACHE_DATA_DIR", data_dir)
        .env("CLIPMILL_CACHE_SOCKET", socket)
        .env("CLIPMILL_CACHE_ACK_DIR", acknowledgements)
        .env("CLIPMILL_CACHE_ITERATION", iteration.to_string())
        .env("RUST_LOG", "error")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cache drill child")
}

async fn wait_for_acknowledgement(child: &mut Child, directory: &Path, previous: usize) {
    for _attempt in 0..2_000 {
        if read_acknowledgements(directory).len() > previous {
            return;
        }
        if let Some(status) = child.try_wait().expect("poll cache drill child") {
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut stream) = child.stdout.take() {
                stream
                    .read_to_string(&mut stdout)
                    .expect("read child stdout");
            }
            if let Some(mut stream) = child.stderr.take() {
                stream
                    .read_to_string(&mut stderr)
                    .expect("read child stderr");
            }
            panic!(
                "cache drill child exited before acknowledgement ({status})\n{stdout}\n{stderr}"
            );
        }
        sleep(Duration::from_millis(5)).await;
    }
    let _kill_result = child.kill();
    let _wait_result = child.wait();
    panic!("cache drill child did not acknowledge within ten seconds");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run by tools/drills/cache-drill.sh"]
async fn acknowledged_artifacts_survive_random_hard_kills() {
    let iterations = std::env::var("CLIPMILL_CACHE_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5);
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    let socket = temp.path().join("clipmilld.sock");
    let acknowledgements = temp.path().join("acknowledgements");
    fs::create_dir(&acknowledgements).expect("create acknowledgement directory");
    File::open(temp.path())
        .expect("open drill directory")
        .sync_all()
        .expect("sync drill directory");
    let mut jitter = SystemTime::now().duration_since(UNIX_EPOCH).map_or(
        u64::from(std::process::id()),
        |duration| {
            duration.as_secs() ^ u64::from(duration.subsec_nanos()) ^ u64::from(std::process::id())
        },
    );

    for iteration in 0..iterations {
        let previous = read_acknowledgements(&acknowledgements).len();
        let mut child = spawn_cache_child(&data_dir, &socket, &acknowledgements, iteration);
        wait_for_acknowledgement(&mut child, &acknowledgements, previous).await;
        jitter ^= jitter << 13;
        jitter ^= jitter >> 7;
        jitter ^= jitter << 17;
        sleep(Duration::from_millis(jitter % 31)).await;
        child.kill().expect("SIGKILL cache drill child");
        let status = child.wait().expect("wait for killed cache drill child");
        assert!(!status.success(), "hard-killed child must not exit cleanly");
    }

    let daemon = Daemon::start(config(&data_dir, &socket))
        .await
        .expect("final recovery starts");
    let coordinator = daemon.artifact_coordinator();
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(daemon.serve_until(async {
        let _stopped = stopped.await;
    }));
    wait_until_ready(&socket)
        .await
        .expect("recovered daemon ready");

    let acknowledged = read_acknowledgements(&acknowledgements);
    assert!(acknowledged.len() >= iterations);
    let roots = coordinator
        .artifact_roots()
        .await
        .expect("read recovered roots")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let projects = list(&socket, "cache-drill-final-list")
        .await
        .expect("list recovered projects")
        .into_iter()
        .map(|project| project.project_id)
        .collect::<BTreeSet<_>>();
    let payload_path = PAYLOAD_NAME.parse::<ArtifactPath>().expect("payload path");

    for acknowledgement in &acknowledged {
        assert!(roots.contains(&acknowledgement.artifact_id));
        assert!(projects.contains(acknowledgement.project_id.as_str()));
        let lease = coordinator
            .open(acknowledgement.artifact_id)
            .await
            .expect("open acknowledged artifact");
        let mut file = lease
            .open_verified(&payload_path)
            .expect("verify acknowledged payload");
        let mut actual = Vec::new();
        file.read_to_end(&mut actual)
            .expect("read verified payload");
        assert_eq!(
            actual,
            payload(acknowledgement.iteration, acknowledgement.sequence)
        );
    }

    for artifact_id in committed_artifact_ids(&data_dir) {
        let lease = coordinator
            .open(artifact_id)
            .await
            .expect("every visible object is catalogued");
        for path in lease.file_paths().expect("visible object file set") {
            drop(
                lease
                    .open_verified(&path)
                    .expect("every visible payload verifies"),
            );
        }
    }
    assert_eq!(
        fs::read_dir(data_dir.join("artifacts/staging"))
            .expect("read recovered staging")
            .count(),
        0,
        "startup must quarantine all interrupted staging directories"
    );

    let _shutdown_sent = shutdown.send(());
    task.await
        .expect("recovered daemon joins")
        .expect("recovered daemon stops");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "cache drill process helper"]
async fn cache_drill_child() {
    if std::env::var_os("CLIPMILL_CACHE_CHILD").as_deref() != Some("1".as_ref()) {
        return;
    }
    let data_dir =
        PathBuf::from(std::env::var_os("CLIPMILL_CACHE_DATA_DIR").expect("child data directory"));
    let socket = PathBuf::from(std::env::var_os("CLIPMILL_CACHE_SOCKET").expect("child socket"));
    let acknowledgements = PathBuf::from(
        std::env::var_os("CLIPMILL_CACHE_ACK_DIR").expect("child acknowledgement directory"),
    );
    let iteration = std::env::var("CLIPMILL_CACHE_ITERATION")
        .expect("child iteration")
        .parse::<usize>()
        .expect("numeric child iteration");

    let daemon = Daemon::start(config(&data_dir, &socket))
        .await
        .expect("child daemon starts");
    let coordinator = daemon.artifact_coordinator();
    let _server = tokio::spawn(daemon.serve_until(std::future::pending::<()>()));
    wait_until_ready(&socket).await.expect("child daemon ready");
    let project = create(
        &socket,
        &format!("cache-drill-project-{iteration}"),
        &format!("Cache drill {iteration}"),
    )
    .await
    .expect("child project");
    let project_id = project
        .project_id
        .parse::<ProjectId>()
        .expect("child project id");
    let path = PAYLOAD_NAME.parse::<ArtifactPath>().expect("payload path");

    for sequence in 0.. {
        let staging = match coordinator
            .prepare(recipe(iteration, sequence))
            .await
            .expect("prepare child artifact")
        {
            PrepareOutcome::Miss(staging) => staging,
            other => panic!("unique drill recipe must miss, got {other:?}"),
        };
        {
            let mut file = staging.create_file(&path).expect("create staged payload");
            file.write_all(&payload(iteration, sequence))
                .expect("write staged payload");
        }
        let lease = coordinator
            .publish_project(
                project_id.clone(),
                staging.id().clone(),
                vec![path.clone()],
                BTreeMap::new(),
            )
            .await
            .expect("durably publish child artifact");
        publish_acknowledgement(
            &acknowledgements,
            &Acknowledgement {
                project_id: project_id.clone(),
                artifact_id: lease.artifact_id(),
                iteration,
                sequence,
            },
        );
    }
}
