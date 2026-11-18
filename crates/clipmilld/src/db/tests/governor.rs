use super::{
    Connection, JobPlan, LeaseId, ResourceCapacity, TempDir, create_project, database, job_store,
    project,
};
use crate::jobs::{LeaseRequest, LeaseSelection, WorkerResidency};
use clipmill_core::{ArtifactId, ProjectId};

const GIB: u64 = 1 << 30;

fn setup() -> (TempDir, Connection, ProjectId) {
    let temp = TempDir::new().unwrap();
    let (_, mut db) = database(&temp);
    let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "edit desk", 1);
    create_project(&mut db, "create", &[1; 32], &project).unwrap();
    (temp, db, project.project_id.parse().unwrap())
}

fn queue(db: &mut Connection, project: &ProjectId, job: &str, stage: &str, ram: u64, at: u64) {
    let mut plan = JobPlan::demo(project, Vec::new(), at);
    plan.tasks.truncate(1);
    plan.kind = job.into();
    let task = &mut plan.tasks[0];
    task.is_final = true;
    task.kind = stage.into();
    task.output_kind = crate::recipes::lookup(stage).unwrap().output_kind.into();
    task.resources.ram_bytes = ram;
    if stage == "editorial-score" {
        task.implementation = "clipmill-worker-editorial@0.2.0/score".into();
    }
    let key = format!("job-{at}");
    job_store::submit_job(db, &key, &[2; 32], &plan).unwrap();
}

fn poll(
    db: &mut Connection,
    worker: &str,
    stage: &str,
    residency: WorkerResidency,
    now: u64,
) -> LeaseSelection {
    // Exactly the budget a 16 GiB Mac receives after editor/OS headroom.
    let capacity = ResourceCapacity::for_device("macos", 8, 16 * GIB, 8 * GIB, 10 * GIB);
    assert_eq!(capacity.ram_bytes, 12 * GIB);
    job_store::lease_next_task_for_worker(
        db,
        &LeaseRequest {
            lease_id: LeaseId::new().to_string(),
            daemon_epoch: "01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
            worker_id: worker.into(),
            now_unix_millis: now,
            expires_unix_millis: now + 15_000,
            capacity,
            machine: capacity,
            residency,
            capabilities: vec![stage.into()],
            foreign_implementations: Vec::new(),
        },
    )
    .unwrap()
}

fn complete(db: &mut Connection, lease: &str, now: u64) {
    let artifact: ArtifactId = format!("sha256:{}", "e".repeat(64)).parse().unwrap();
    job_store::complete_task(db, lease, artifact, &[0; 32], &[], now).unwrap();
}

#[test]
fn interactive_work_reclaims_an_idle_model_only_after_unload_acknowledgement() {
    let (_temp, mut db, project) = setup();
    queue(
        &mut db,
        &project,
        "analyze-source",
        "editorial-score",
        8 * GIB,
        1,
    );
    let running = poll(
        &mut db,
        "editor",
        "editorial-score",
        WorkerResidency::Cold,
        5,
    )
    .task
    .unwrap();
    queue(&mut db, &project, "export-clip", "demo-left", 5 * GIB, 6);
    assert!(
        poll(
            &mut db,
            "builtin",
            "demo-left",
            WorkerResidency::Stateless,
            7
        )
        .task
        .is_none()
    );
    complete(&mut db, &running.lease_id, 8);
    let waiting = poll(
        &mut db,
        "builtin",
        "demo-left",
        WorkerResidency::Stateless,
        9,
    );
    assert!(waiting.task.is_none());
    let reclaim = poll(
        &mut db,
        "editor",
        "editorial-score",
        WorkerResidency::Warm,
        10,
    );
    assert!(reclaim.release_resident_model && reclaim.task.is_none());
    assert!(
        poll(
            &mut db,
            "builtin",
            "demo-left",
            WorkerResidency::Stateless,
            11
        )
        .task
        .is_none()
    );
    assert!(
        !poll(
            &mut db,
            "editor",
            "editorial-score",
            WorkerResidency::Cold,
            12
        )
        .release_resident_model
    );
    let next = poll(
        &mut db,
        "builtin",
        "demo-left",
        WorkerResidency::Stateless,
        13,
    )
    .task
    .unwrap();
    assert_eq!(next.attempt, 1);
}

#[test]
fn separate_workers_and_builtins_share_capacity_and_protect_an_interactive_waiter() {
    let (_temp, mut db, project) = setup();
    queue(&mut db, &project, "analyze-source", "demo-seed", 8 * GIB, 1);
    queue(
        &mut db,
        &project,
        "analyze-source",
        "demo-right",
        4 * GIB,
        2,
    );
    queue(&mut db, &project, "export-clip", "demo-left", 6 * GIB, 3);
    let background = poll(&mut db, "cpu", "demo-seed", WorkerResidency::Stateless, 6)
        .task
        .unwrap();
    assert!(
        poll(
            &mut db,
            "builtin",
            "demo-left",
            WorkerResidency::Stateless,
            7
        )
        .task
        .is_none()
    );
    // The 4 GiB analysis task fits on its own, but would delay the user's export.
    assert!(
        poll(
            &mut db,
            "another",
            "demo-right",
            WorkerResidency::Stateless,
            8
        )
        .task
        .is_none()
    );
    complete(&mut db, &background.lease_id, 9);
    let export = poll(
        &mut db,
        "builtin",
        "demo-left",
        WorkerResidency::Stateless,
        10,
    )
    .task
    .unwrap();
    assert_eq!(export.attempt, 1);
    assert!(
        poll(
            &mut db,
            "another",
            "demo-right",
            WorkerResidency::Stateless,
            11
        )
        .task
        .is_some()
    );
}

#[test]
fn a_warm_worker_can_reuse_its_own_reservation_without_double_charging() {
    let (_temp, mut db, project) = setup();
    queue(
        &mut db,
        &project,
        "analyze-source",
        "editorial-score",
        8 * GIB,
        1,
    );
    queue(
        &mut db,
        &project,
        "analyze-source",
        "editorial-score",
        8 * GIB,
        2,
    );
    let first = poll(
        &mut db,
        "editor",
        "editorial-score",
        WorkerResidency::Cold,
        5,
    )
    .task
    .unwrap();
    complete(&mut db, &first.lease_id, 6);
    let next = poll(
        &mut db,
        "editor",
        "editorial-score",
        WorkerResidency::Warm,
        7,
    );
    assert!(next.task.is_some() && !next.release_resident_model);
}

#[test]
fn cancellation_keeps_the_recent_lease_charged_while_the_worker_stops() {
    let (_temp, mut db, project) = setup();
    queue(&mut db, &project, "analyze-source", "demo-seed", 8 * GIB, 1);
    let running = poll(&mut db, "cpu", "demo-seed", WorkerResidency::Stateless, 5)
        .task
        .unwrap();
    queue(&mut db, &project, "export-clip", "demo-left", 6 * GIB, 6);
    job_store::cancel_job(&mut db, "cancel-analysis", &[4; 32], &running.job_id, 7).unwrap();
    assert!(
        poll(
            &mut db,
            "builtin",
            "demo-left",
            WorkerResidency::Stateless,
            8
        )
        .task
        .is_none()
    );
    assert!(
        poll(
            &mut db,
            "builtin",
            "demo-left",
            WorkerResidency::Stateless,
            15_006
        )
        .task
        .is_some()
    );
}
