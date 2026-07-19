#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    collections::BTreeSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_artifacts::ArtifactStore;
use clipmill_contracts::proto::ipc::v1::{JobState, TaskState};
use clipmill_core::ArtifactId;
use rusqlite::Connection;
use tokio::time::sleep;

use support::{
    create, get_job, list, spawn_daemon, spawn_daemon_with_step_delay, submit_demo,
    wait_until_ready, workspace_tempdir,
};

async fn wait_for_job_boundary(socket: &std::path::Path, job_id: &str, boundary: usize) {
    if boundary == 0 || boundary == 4 {
        return;
    }
    for attempt in 0..1_500 {
        let job = get_job(socket, &format!("boundary-{boundary}-{attempt}"), job_id)
            .await
            .expect("query boundary job");
        let reached = match boundary {
            1 => job
                .tasks
                .iter()
                .any(|task| task.state == TaskState::Running as i32),
            2 => {
                job.state == JobState::Running as i32
                    && job
                        .tasks
                        .iter()
                        .any(|task| task.state == TaskState::Succeeded as i32)
            }
            3 => job.state == JobState::Succeeded as i32,
            _ => false,
        };
        if reached {
            return;
        }
        sleep(Duration::from_millis(2)).await;
    }
    panic!("job did not reach kill boundary {boundary}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run by tools/drills/kill-drill.sh"]
#[allow(clippy::too_many_lines)]
async fn acknowledged_jobs_and_projects_survive_random_hard_kills() {
    let iterations = std::env::var("CLIPMILL_KILL_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5);
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    let socket = temp.path().join("clipmilld.sock");
    let mut acknowledged_projects = BTreeSet::new();
    let mut acknowledged_jobs = BTreeSet::new();
    let mut jitter = SystemTime::now().duration_since(UNIX_EPOCH).map_or(
        u64::from(std::process::id()),
        |duration| {
            duration.as_secs() ^ u64::from(duration.subsec_nanos()) ^ u64::from(std::process::id())
        },
    );

    for iteration in 0..iterations {
        let mut daemon = spawn_daemon_with_step_delay(&data_dir, &socket, Some(20));
        wait_until_ready(&socket).await.expect("daemon ready");

        let guaranteed = create(
            &socket,
            &format!("guaranteed-{iteration}"),
            &format!("Guaranteed {iteration}"),
        )
        .await
        .expect("guaranteed create");
        acknowledged_projects.insert(guaranteed.project_id.clone());
        let guaranteed_job = submit_demo(
            &socket,
            &format!("guaranteed-job-{iteration}"),
            &guaranteed.project_id,
            format!("guaranteed payload {iteration}").as_bytes(),
        )
        .await
        .expect("guaranteed job submit");
        acknowledged_jobs.insert(guaranteed_job.job_id.clone());

        let optional_socket = socket.clone();
        let optional = tokio::spawn(async move {
            let project = create(
                &optional_socket,
                &format!("optional-{iteration}"),
                &format!("Optional {iteration}"),
            )
            .await?;
            let job = submit_demo(
                &optional_socket,
                &format!("optional-job-{iteration}"),
                &project.project_id,
                format!("optional payload {iteration}").as_bytes(),
            )
            .await?;
            Ok::<_, String>((project.project_id, job.job_id))
        });
        let boundary = iteration % 5;
        wait_for_job_boundary(&socket, &guaranteed_job.job_id, boundary).await;
        if boundary == 4 {
            jitter ^= jitter << 13;
            jitter ^= jitter >> 7;
            jitter ^= jitter << 17;
            sleep(Duration::from_millis(jitter % 25)).await;
        }
        daemon.kill().expect("SIGKILL daemon");
        let _status = daemon.wait().expect("wait for killed daemon");
        if let Ok(Ok((project_id, job_id))) = optional.await {
            acknowledged_projects.insert(project_id);
            acknowledged_jobs.insert(job_id);
        }
    }

    let mut daemon = spawn_daemon(&data_dir, &socket);
    wait_until_ready(&socket).await.expect("final daemon ready");
    let persisted: BTreeSet<_> = list(&socket, "final-list")
        .await
        .expect("list after restarts")
        .into_iter()
        .map(|project| project.project_id)
        .collect();
    assert!(acknowledged_projects.is_subset(&persisted));
    assert!(persisted.len() >= iterations);

    let mut acknowledged_artifacts = BTreeSet::new();
    for (index, job_id) in acknowledged_jobs.iter().enumerate() {
        let mut terminal = None;
        for attempt in 0..1_500 {
            let job = get_job(&socket, &format!("recover-{index}-{attempt}"), job_id)
                .await
                .expect("acknowledged job remains queryable");
            if matches!(
                JobState::try_from(job.state),
                Ok(JobState::Succeeded | JobState::Failed | JobState::Cancelled)
            ) {
                terminal = Some(job);
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
        let job = terminal.expect("job becomes consistent within 30 seconds");
        assert_eq!(job.state, JobState::Succeeded as i32);
        assert_eq!(job.output_artifact_ids.len(), 1);
        acknowledged_artifacts.insert(
            job.output_artifact_ids[0]
                .parse::<ArtifactId>()
                .expect("output artifact id"),
        );
    }
    daemon.kill().expect("stop final daemon");
    let _status = daemon.wait().expect("wait final daemon");

    let database = Connection::open(data_dir.join("state/clipmill.db")).expect("open database");
    let quick_check: String = database
        .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
        .expect("quick check");
    assert_eq!(quick_check, "ok");

    let (store, _recovery) = ArtifactStore::initialize(data_dir.join("artifacts"))
        .expect("recover artifact store after drill");
    for artifact_id in acknowledged_artifacts {
        let lease = store
            .open(artifact_id)
            .expect("acknowledged artifact opens");
        for path in lease.file_paths().expect("declared artifact paths") {
            let _verified = lease
                .open_verified(&path)
                .expect("acknowledged artifact payload verifies");
        }
    }
}
