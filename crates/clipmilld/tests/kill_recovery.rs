#![cfg(unix)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

mod support;

use std::{
    collections::BTreeSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::time::sleep;

use support::{create, list, spawn_daemon, wait_until_ready, workspace_tempdir};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run by tools/drills/kill-drill.sh"]
async fn acknowledged_projects_survive_random_hard_kills() {
    let iterations = std::env::var("CLIPMILL_KILL_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5);
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    let socket = temp.path().join("clipmilld.sock");
    let mut acknowledged = BTreeSet::new();
    let mut jitter = SystemTime::now().duration_since(UNIX_EPOCH).map_or(
        u64::from(std::process::id()),
        |duration| {
            duration.as_secs() ^ u64::from(duration.subsec_nanos()) ^ u64::from(std::process::id())
        },
    );

    for iteration in 0..iterations {
        let mut daemon = spawn_daemon(&data_dir, &socket);
        wait_until_ready(&socket).await.expect("daemon ready");

        let guaranteed = create(
            &socket,
            &format!("guaranteed-{iteration}"),
            &format!("Guaranteed {iteration}"),
        )
        .await
        .expect("guaranteed create");
        acknowledged.insert(guaranteed.project_id);

        let optional_socket = socket.clone();
        let optional = tokio::spawn(async move {
            create(
                &optional_socket,
                &format!("optional-{iteration}"),
                &format!("Optional {iteration}"),
            )
            .await
        });
        jitter ^= jitter << 13;
        jitter ^= jitter >> 7;
        jitter ^= jitter << 17;
        sleep(Duration::from_millis(jitter % 8)).await;
        daemon.kill().expect("SIGKILL daemon");
        let _status = daemon.wait().expect("wait for killed daemon");
        if let Ok(Ok(project)) = optional.await {
            acknowledged.insert(project.project_id);
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
    assert!(acknowledged.is_subset(&persisted));
    assert!(persisted.len() >= iterations);
    daemon.kill().expect("stop final daemon");
    let _status = daemon.wait().expect("wait final daemon");
}
