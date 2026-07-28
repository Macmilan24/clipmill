use std::collections::{BTreeMap, BTreeSet, VecDeque};

use clipmill_contracts::proto::{
    ipc::v1::{CancelJobResponse, JobState, Response, SubmitJobResponse, TaskState, response},
    worker::v1::FailureClass,
};
use clipmill_core::{ArtifactId, JobId, ProjectId, TaskId};
use prost::Message;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};

use super::{StoreError, remember, replay};
use crate::jobs::{
    EventFilter, JobPlan, JobRecord, LeaseRequest, LeaseSelection, LeasedTask, ResourceDeclaration,
    TaskCompletion, TaskEventRecord, TaskRecord, accelerator_bit, is_terminal_job,
};

#[derive(Debug)]
struct SelectedTaskRow {
    task_id: String,
    job_id: String,
    project_id: String,
    source_id: Option<String>,
    kind: String,
    output_kind: String,
    payload: Vec<u8>,
    implementation: String,
    attempt: i64,
    cpu_threads: i64,
    ram_bytes: i64,
    accelerator_class: String,
    vram_bytes: i64,
    disk_bytes: i64,
    network_policy: String,
    thermal_class: String,
    determinism_class: String,
    checkpoint_support: bool,
    preemption_cost: i64,
}

type FailedTaskRow = (String, String, String, String, String, Vec<u8>, i64, i64);
type CompletionLeaseRow = (String, i64, Option<Vec<u8>>, Option<Vec<u8>>);
type CompletionReplayRow = (String, Option<Vec<u8>>, Option<Vec<u8>>);

pub(super) const CREATE_V3_TABLES: &str = "
    CREATE TABLE jobs (
        job_id TEXT PRIMARY KEY
            CHECK(length(job_id) = 30 AND substr(job_id, 1, 4) = 'job_'),
        project_id TEXT NOT NULL
            REFERENCES projects(project_id) ON DELETE CASCADE,
        kind TEXT NOT NULL CHECK(length(kind) BETWEEN 1 AND 128),
        payload BLOB NOT NULL,
        state INTEGER NOT NULL CHECK(state BETWEEN 1 AND 6),
        cancel_requested INTEGER NOT NULL DEFAULT 0 CHECK(cancel_requested IN (0, 1)),
        failure_class INTEGER NOT NULL DEFAULT 0 CHECK(failure_class BETWEEN 0 AND 4),
        failure_detail TEXT NOT NULL DEFAULT '',
        created_unix_millis INTEGER NOT NULL CHECK(created_unix_millis >= 0),
        updated_unix_millis INTEGER NOT NULL CHECK(updated_unix_millis >= 0)
    ) STRICT;

    CREATE INDEX jobs_by_project_created
        ON jobs(project_id, created_unix_millis DESC, job_id DESC);

    CREATE TABLE tasks (
        task_id TEXT PRIMARY KEY
            CHECK(length(task_id) = 30 AND substr(task_id, 1, 4) = 'tsk_'),
        job_id TEXT NOT NULL REFERENCES jobs(job_id) ON DELETE CASCADE,
        ordinal INTEGER NOT NULL CHECK(ordinal >= 0),
        kind TEXT NOT NULL CHECK(length(kind) BETWEEN 1 AND 128),
        input_kinds_json TEXT NOT NULL
            CHECK(json_valid(input_kinds_json) AND json_type(input_kinds_json) = 'array'),
        output_kind TEXT NOT NULL CHECK(length(output_kind) BETWEEN 1 AND 128),
        payload BLOB NOT NULL,
        input_key TEXT NOT NULL
            CHECK(length(input_key) = 64 AND input_key NOT GLOB '*[^0-9a-f]*'),
        state INTEGER NOT NULL CHECK(state BETWEEN 1 AND 7),
        cpu_threads INTEGER NOT NULL CHECK(cpu_threads > 0),
        ram_bytes INTEGER NOT NULL CHECK(ram_bytes >= 0),
        accelerator_class TEXT NOT NULL,
        vram_bytes INTEGER NOT NULL CHECK(vram_bytes >= 0),
        disk_bytes INTEGER NOT NULL CHECK(disk_bytes >= 0),
        network_policy TEXT NOT NULL CHECK(network_policy IN ('local-lock', 'network-allowed')),
        thermal_class TEXT NOT NULL,
        determinism_class TEXT NOT NULL,
        checkpoint_support INTEGER NOT NULL CHECK(checkpoint_support IN (0, 1)),
        preemption_cost INTEGER NOT NULL CHECK(preemption_cost >= 0),
        implementation TEXT NOT NULL CHECK(length(implementation) BETWEEN 1 AND 256),
        max_attempts INTEGER NOT NULL CHECK(max_attempts BETWEEN 1 AND 100),
        attempt INTEGER NOT NULL DEFAULT 0 CHECK(attempt BETWEEN 0 AND 100),
        next_attempt_unix_millis INTEGER NOT NULL CHECK(next_attempt_unix_millis >= 0),
        is_final INTEGER NOT NULL CHECK(is_final IN (0, 1)),
        progress_unit TEXT NOT NULL DEFAULT '',
        progress_done INTEGER NOT NULL DEFAULT 0 CHECK(progress_done >= 0),
        progress_total INTEGER NOT NULL DEFAULT 0 CHECK(progress_total >= 0),
        wait_reason TEXT NOT NULL DEFAULT '',
        output_artifact_id TEXT,
        failure_class INTEGER NOT NULL DEFAULT 0 CHECK(failure_class BETWEEN 0 AND 4),
        failure_detail TEXT NOT NULL DEFAULT '',
        created_unix_millis INTEGER NOT NULL CHECK(created_unix_millis >= 0),
        updated_unix_millis INTEGER NOT NULL CHECK(updated_unix_millis >= 0),
        UNIQUE(job_id, ordinal)
    ) STRICT;

    CREATE INDEX tasks_by_runnable
        ON tasks(state, next_attempt_unix_millis, job_id, ordinal);

    CREATE TABLE task_dependencies (
        task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
        depends_on_task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
        dependency_ordinal INTEGER NOT NULL CHECK(dependency_ordinal >= 0),
        CHECK(task_id <> depends_on_task_id),
        PRIMARY KEY(task_id, depends_on_task_id),
        UNIQUE(task_id, dependency_ordinal)
    ) STRICT, WITHOUT ROWID;

    CREATE TABLE task_leases (
        lease_id TEXT PRIMARY KEY
            CHECK(length(lease_id) = 30 AND substr(lease_id, 1, 4) = 'lse_'),
        task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
        worker_id TEXT NOT NULL,
        daemon_epoch TEXT NOT NULL CHECK(length(daemon_epoch) = 26),
        status INTEGER NOT NULL CHECK(status BETWEEN 1 AND 4),
        acquired_unix_millis INTEGER NOT NULL CHECK(acquired_unix_millis >= 0),
        expires_unix_millis INTEGER NOT NULL CHECK(expires_unix_millis >= acquired_unix_millis),
        last_heartbeat_unix_millis INTEGER NOT NULL CHECK(last_heartbeat_unix_millis >= acquired_unix_millis),
        completion_hash BLOB CHECK(completion_hash IS NULL OR length(completion_hash) = 32),
        completion_response BLOB
    ) STRICT;

    CREATE UNIQUE INDEX one_active_lease_per_task
        ON task_leases(task_id) WHERE status = 1;
    CREATE INDEX task_leases_by_expiry
        ON task_leases(status, expires_unix_millis);

    CREATE TABLE task_events (
        event_id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
        job_id TEXT NOT NULL REFERENCES jobs(job_id) ON DELETE CASCADE,
        task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
        state INTEGER NOT NULL CHECK(state BETWEEN 1 AND 7),
        attempt INTEGER NOT NULL CHECK(attempt >= 0),
        progress_unit TEXT NOT NULL DEFAULT '',
        progress_done INTEGER NOT NULL DEFAULT 0 CHECK(progress_done >= 0),
        progress_total INTEGER NOT NULL DEFAULT 0 CHECK(progress_total >= 0),
        wait_reason TEXT NOT NULL DEFAULT '',
        failure_class INTEGER NOT NULL DEFAULT 0 CHECK(failure_class BETWEEN 0 AND 4),
        at_unix_millis INTEGER NOT NULL CHECK(at_unix_millis >= 0)
    ) STRICT;

    CREATE INDEX task_events_by_job_cursor ON task_events(job_id, event_id);
    CREATE INDEX task_events_by_project_cursor ON task_events(project_id, event_id);

    CREATE TABLE task_artifact_roots (
        task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
        artifact_id TEXT NOT NULL
            CHECK(
                length(artifact_id) = 71
                AND substr(artifact_id, 1, 7) = 'sha256:'
                AND substr(artifact_id, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        PRIMARY KEY(task_id, artifact_id)
    ) STRICT, WITHOUT ROWID;

    CREATE INDEX task_artifact_roots_by_artifact ON task_artifact_roots(artifact_id);

    CREATE TABLE circuit_breakers (
        stage TEXT NOT NULL,
        implementation TEXT NOT NULL,
        input_key TEXT NOT NULL,
        deterministic_failures INTEGER NOT NULL CHECK(deterministic_failures >= 0),
        open INTEGER NOT NULL CHECK(open IN (0, 1)),
        updated_unix_millis INTEGER NOT NULL CHECK(updated_unix_millis >= 0),
        PRIMARY KEY(stage, implementation, input_key)
    ) STRICT, WITHOUT ROWID;
";

/// Artifacts a task reads that no task in its own plan produced.
///
/// Separate from `task_dependencies` because the two say different things: a
/// dependency is an ordering constraint whose output happens to be an input,
/// while this is an input with no ordering to constrain — it was published
/// before this job existed. Both end up in the lease's input list, declared
/// first, because that list's order is part of the artifact key.
pub(super) const CREATE_V7_TABLES: &str = "
    CREATE TABLE task_input_artifacts (
        task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
        artifact_id TEXT NOT NULL
            CHECK(
                length(artifact_id) = 71
                AND substr(artifact_id, 1, 7) = 'sha256:'
                AND substr(artifact_id, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        input_ordinal INTEGER NOT NULL CHECK(input_ordinal >= 0),
        PRIMARY KEY(task_id, artifact_id),
        UNIQUE(task_id, input_ordinal)
    ) STRICT, WITHOUT ROWID;
";

#[derive(Debug)]
pub(crate) struct MutationResult {
    pub bytes: Vec<u8>,
    pub events: Vec<TaskEventRecord>,
}

#[allow(clippy::too_many_lines)]
pub(super) fn submit_job(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    plan: &JobPlan,
) -> Result<MutationResult, StoreError> {
    validate_plan(plan)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(MutationResult {
            bytes: response,
            events: Vec::new(),
        });
    }
    let project_exists: bool = transaction.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM projects WHERE project_id = ?1 AND is_system = 0
         )",
        [&plan.project_id],
        |row| row.get(0),
    )?;
    if !project_exists {
        return Err(StoreError::NotFound);
    }
    if let Some(source_id) = &plan.source_id {
        let source_matches: bool = transaction.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sources WHERE source_id = ?1 AND project_id = ?2
             )",
            params![source_id, plan.project_id],
            |row| row.get(0),
        )?;
        if !source_matches {
            return Err(StoreError::NotFound);
        }
    }
    let events = insert_job_plan(&transaction, plan)?;
    let job = get_job_tx(&transaction, &plan.job_id)?;
    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::SubmitJob(SubmitJobResponse {
            job_id: plan.job_id.clone(),
            job: Some(job.into()),
        })),
    }
    .encode_to_vec();
    remember(
        &transaction,
        request_id,
        request_hash,
        &response,
        plan.created_unix_millis,
    )?;
    transaction.commit()?;
    Ok(MutationResult {
        bytes: response,
        events,
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "one insert per table a plan touches, in dependency order"
)]
pub(super) fn insert_job_plan(
    transaction: &Transaction<'_>,
    plan: &JobPlan,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    validate_plan(plan)?;
    let now = sqlite_u64(plan.created_unix_millis, "job timestamp")?;
    transaction.execute(
        "INSERT INTO jobs(
            job_id, project_id, kind, source_id, payload, state,
            created_unix_millis, updated_unix_millis
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            plan.job_id,
            plan.project_id,
            plan.kind,
            plan.source_id,
            plan.payload,
            JobState::Planned as i32,
            now
        ],
    )?;
    for task in &plan.tasks {
        let resources = &task.resources;
        let wait_reason = if task.dependencies.is_empty() {
            "waiting: admission"
        } else {
            "waiting: dependencies"
        };
        transaction.execute(
            "INSERT INTO tasks(
                task_id, job_id, ordinal, kind, input_kinds_json, output_kind,
                payload, input_key, state,
                cpu_threads, ram_bytes, accelerator_class, vram_bytes, disk_bytes,
                network_policy, thermal_class, determinism_class, checkpoint_support,
                preemption_cost, implementation, max_attempts, next_attempt_unix_millis,
                is_final, wait_reason, created_unix_millis, updated_unix_millis
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?25
             )",
            params![
                task.task_id,
                plan.job_id,
                i64::from(task.ordinal),
                task.kind,
                serde_json::to_string(&task.input_kinds)
                    .map_err(|_| StoreError::InvalidData("task input kinds are invalid"))?,
                task.output_kind,
                task.payload,
                hex::encode(Sha256::digest(&task.payload)),
                TaskState::Planned as i32,
                i64::from(task.resources.cpu_threads),
                sqlite_u64(resources.ram_bytes, "task RAM")?,
                resources.accelerator_class,
                sqlite_u64(resources.vram_bytes, "task VRAM")?,
                sqlite_u64(resources.disk_bytes, "task disk")?,
                resources.network_policy,
                resources.thermal_class,
                resources.determinism_class,
                i32::from(resources.checkpoint_support),
                i64::from(resources.preemption_cost),
                task.implementation,
                i64::from(task.max_attempts),
                now,
                i32::from(task.is_final),
                wait_reason,
                now,
            ],
        )?;
    }
    for task in &plan.tasks {
        for (ordinal, dependency) in task.dependencies.iter().enumerate() {
            transaction.execute(
                "INSERT INTO task_dependencies(task_id, depends_on_task_id, dependency_ordinal)
                 VALUES (?1, ?2, ?3)",
                params![task.task_id, dependency, sqlite_usize(ordinal)?],
            )?;
        }
        for (ordinal, artifact_id) in task.input_artifact_ids.iter().enumerate() {
            transaction.execute(
                "INSERT INTO task_input_artifacts(task_id, artifact_id, input_ordinal)
                 VALUES (?1, ?2, ?3)",
                params![task.task_id, artifact_id, sqlite_usize(ordinal)?],
            )?;
        }
    }
    let mut events = Vec::with_capacity(plan.tasks.len());
    for task in &plan.tasks {
        let wait_reason = if task.dependencies.is_empty() {
            "waiting: admission"
        } else {
            "waiting: dependencies"
        };
        events.push(insert_event(
            transaction,
            &plan.project_id,
            &plan.job_id,
            &task.task_id,
            TaskState::Planned as i32,
            0,
            "",
            0,
            0,
            wait_reason,
            FailureClass::Unspecified as i32,
            plan.created_unix_millis,
        )?);
    }
    Ok(events)
}

pub(super) fn get_job(connection: &Connection, job_id: &str) -> Result<JobRecord, StoreError> {
    let visible: bool = connection.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM jobs j JOIN projects p ON p.project_id = j.project_id
            WHERE j.job_id = ?1 AND p.is_system = 0
         )",
        [job_id],
        |row| row.get(0),
    )?;
    if !visible {
        return Err(StoreError::NotFound);
    }
    get_job_from(connection, job_id)
}

pub(super) fn list_jobs(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<JobRecord>, StoreError> {
    let project_exists: bool = connection.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM projects WHERE project_id = ?1 AND is_system = 0
         )",
        [project_id],
        |row| row.get(0),
    )?;
    if !project_exists {
        return Err(StoreError::NotFound);
    }
    let mut statement = connection.prepare(
        "SELECT job_id FROM jobs WHERE project_id = ?1
         ORDER BY created_unix_millis DESC, job_id DESC",
    )?;
    let ids = statement
        .query_map([project_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|job_id| get_job_from(connection, &job_id))
        .collect()
}

pub(super) fn cancel_job(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    job_id: &str,
    now: u64,
) -> Result<MutationResult, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(MutationResult {
            bytes: response,
            events: Vec::new(),
        });
    }
    let existing = get_job_tx(&transaction, job_id)?;
    let mut events = Vec::new();
    if !is_terminal_job(existing.state) {
        let now_sql = sqlite_u64(now, "cancel timestamp")?;
        let mut statement = transaction.prepare(
            "SELECT task_id, attempt FROM tasks
             WHERE job_id = ?1 AND state NOT IN (?2, ?3, ?4)
             ORDER BY ordinal",
        )?;
        let tasks = statement
            .query_map(
                params![
                    job_id,
                    TaskState::Succeeded as i32,
                    TaskState::Failed as i32,
                    TaskState::Cancelled as i32
                ],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);
        for (task_id, attempt) in tasks {
            transaction.execute(
                "UPDATE tasks SET state = ?1, wait_reason = '', updated_unix_millis = ?2
                 WHERE task_id = ?3",
                params![TaskState::Cancelled as i32, now_sql, task_id],
            )?;
            transaction.execute(
                "UPDATE task_leases SET status = 4 WHERE task_id = ?1 AND status = 1",
                [&task_id],
            )?;
            let project_id = existing.project_id.clone();
            events.push(insert_event(
                &transaction,
                &project_id,
                job_id,
                &task_id,
                TaskState::Cancelled as i32,
                u32_from_i64(attempt, "task attempt")?,
                "",
                0,
                0,
                "",
                FailureClass::Unspecified as i32,
                now,
            )?);
        }
        transaction.execute(
            "UPDATE jobs SET state = ?1, cancel_requested = 1, updated_unix_millis = ?2
             WHERE job_id = ?3",
            params![JobState::Cancelled as i32, now_sql, job_id],
        )?;
        transaction.execute(
            "DELETE FROM task_artifact_roots
             WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id = ?1)",
            [job_id],
        )?;
    }
    let job = get_job_tx(&transaction, job_id)?;
    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::CancelJob(CancelJobResponse {
            job: Some(job.into()),
        })),
    }
    .encode_to_vec();
    remember(&transaction, request_id, request_hash, &response, now)?;
    transaction.commit()?;
    Ok(MutationResult {
        bytes: response,
        events,
    })
}

pub(super) fn recover_jobs(
    connection: &mut Connection,
    _daemon_epoch: &str,
    now: u64,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now_sql = sqlite_u64(now, "recovery timestamp")?;
    let mut statement = transaction.prepare(
        "SELECT t.task_id, t.job_id, j.project_id, t.attempt
         FROM tasks t JOIN jobs j ON j.job_id = t.job_id
         WHERE t.state IN (?1, ?2)
         ORDER BY t.job_id, t.ordinal",
    )?;
    let interrupted = statement
        .query_map(
            params![TaskState::Admitted as i32, TaskState::Running as i32],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    let mut events = Vec::new();
    for (task_id, job_id, project_id, attempt) in interrupted {
        let recovered_attempt = attempt.saturating_sub(1);
        let state = TaskState::Retryable;
        let failure = FailureClass::Unspecified as i32;
        transaction.execute(
            "UPDATE tasks SET state = ?1, attempt = ?2, next_attempt_unix_millis = ?3,
                wait_reason = ?4, failure_class = ?5, updated_unix_millis = ?3
             WHERE task_id = ?6",
            params![
                state as i32,
                recovered_attempt,
                now_sql,
                "retry: daemon restart",
                failure,
                task_id
            ],
        )?;
        events.push(insert_event(
            &transaction,
            &project_id,
            &job_id,
            &task_id,
            state as i32,
            u32_from_i64(recovered_attempt, "task attempt")?,
            "",
            0,
            0,
            "retry: daemon restart",
            failure,
            now,
        )?);
    }
    transaction.execute("UPDATE task_leases SET status = 3 WHERE status = 1", [])?;
    aggregate_all_jobs(&transaction, now)?;
    transaction.commit()?;
    Ok(events)
}

#[allow(clippy::too_many_lines)]
pub(super) fn lease_next_task_for_worker(
    connection: &mut Connection,
    request: &LeaseRequest,
) -> Result<LeaseSelection, StoreError> {
    let capability_filter = format!(",{},", request.capabilities.join(","));
    let now = request.now_unix_millis;
    let expires = request.expires_unix_millis;
    let capacity = request.capacity;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let mut events = fail_open_circuit_breakers(&transaction, now)?;
    let selected: Option<SelectedTaskRow> = transaction
        .query_row(
            "SELECT t.task_id, t.job_id, j.project_id, j.source_id,
                    t.kind, t.output_kind, t.payload,
                    t.implementation, t.attempt, t.cpu_threads, t.ram_bytes,
                    t.accelerator_class, t.vram_bytes, t.disk_bytes, t.network_policy,
                    t.thermal_class, t.determinism_class, t.checkpoint_support,
                    t.preemption_cost
             FROM tasks t JOIN jobs j ON j.job_id = t.job_id
             WHERE t.state IN (?1, ?2)
               AND t.next_attempt_unix_millis <= ?3
               AND j.cancel_requested = 0
               AND j.state IN (?4, ?5)
               AND NOT EXISTS (
                    SELECT 1 FROM task_dependencies d
                    JOIN tasks dependency ON dependency.task_id = d.depends_on_task_id
                    WHERE d.task_id = t.task_id AND dependency.state <> ?6
               )
               AND NOT EXISTS (
                    SELECT 1 FROM circuit_breakers c
                    WHERE c.stage = t.kind AND c.implementation = t.implementation
                      AND c.input_key = t.input_key AND c.open = 1
               )
               AND t.cpu_threads <= ?7
               AND t.ram_bytes <= ?8
               AND t.disk_bytes <= ?9
               AND (
                    (t.accelerator_class = '' AND t.vram_bytes = 0)
                    OR (
                        t.accelerator_class <> ''
                        AND t.vram_bytes <= ?10
                        AND (
                            (t.accelerator_class = 'videotoolbox' AND (?11 & ?12) <> 0)
                            OR (t.accelerator_class = 'vaapi' AND (?11 & ?13) <> 0)
                            OR (t.accelerator_class = 'cuda' AND (?11 & ?14) <> 0)
                            OR (t.accelerator_class = 'vulkan' AND (?11 & ?15) <> 0)
                            OR (t.accelerator_class = 'metal' AND (?11 & ?16) <> 0)
                        )
                    )
               )
               AND t.network_policy = 'local-lock'
               AND instr(?17, ',' || t.kind || ',') > 0
             ORDER BY j.created_unix_millis, j.job_id, t.ordinal
             LIMIT 1",
            params![
                TaskState::Planned as i32,
                TaskState::Retryable as i32,
                sqlite_u64(now, "lease timestamp")?,
                JobState::Planned as i32,
                JobState::Running as i32,
                TaskState::Succeeded as i32,
                i64::from(capacity.cpu_threads),
                sqlite_u64(capacity.ram_bytes, "available task RAM")?,
                sqlite_u64(capacity.disk_bytes, "available task disk")?,
                sqlite_u64(capacity.vram_bytes, "available task VRAM")?,
                i64::from(capacity.accelerator_mask),
                i64::from(accelerator_bit("videotoolbox").unwrap_or(0)),
                i64::from(accelerator_bit("vaapi").unwrap_or(0)),
                i64::from(accelerator_bit("cuda").unwrap_or(0)),
                i64::from(accelerator_bit("vulkan").unwrap_or(0)),
                i64::from(accelerator_bit("metal").unwrap_or(0)),
                capability_filter,
            ],
            |row| {
                Ok(SelectedTaskRow {
                    task_id: row.get(0)?,
                    job_id: row.get(1)?,
                    project_id: row.get(2)?,
                    source_id: row.get(3)?,
                    kind: row.get(4)?,
                    output_kind: row.get(5)?,
                    payload: row.get(6)?,
                    implementation: row.get(7)?,
                    attempt: row.get(8)?,
                    cpu_threads: row.get(9)?,
                    ram_bytes: row.get(10)?,
                    accelerator_class: row.get(11)?,
                    vram_bytes: row.get(12)?,
                    disk_bytes: row.get(13)?,
                    network_policy: row.get(14)?,
                    thermal_class: row.get(15)?,
                    determinism_class: row.get(16)?,
                    checkpoint_support: row.get(17)?,
                    preemption_cost: row.get(18)?,
                })
            },
        )
        .optional()?;
    let Some(selected) = selected else {
        transaction.commit()?;
        return Ok(LeaseSelection { task: None, events });
    };
    let next_attempt = selected
        .attempt
        .checked_add(1)
        .ok_or(StoreError::InvalidData("task attempt overflow"))?;
    let now_sql = sqlite_u64(now, "lease timestamp")?;
    let expires_sql = sqlite_u64(expires, "lease expiry")?;
    transaction.execute(
        "UPDATE tasks SET state = ?1, attempt = ?2, wait_reason = '', updated_unix_millis = ?3
         WHERE task_id = ?4",
        params![
            TaskState::Admitted as i32,
            next_attempt,
            now_sql,
            selected.task_id
        ],
    )?;
    events.push(insert_event(
        &transaction,
        &selected.project_id,
        &selected.job_id,
        &selected.task_id,
        TaskState::Admitted as i32,
        u32_from_i64(next_attempt, "task attempt")?,
        "",
        0,
        0,
        "",
        FailureClass::Unspecified as i32,
        now,
    )?);
    transaction.execute(
        "INSERT INTO task_leases(
            lease_id, task_id, worker_id, daemon_epoch, status,
            acquired_unix_millis, expires_unix_millis, last_heartbeat_unix_millis
         ) VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?5)",
        params![
            request.lease_id,
            selected.task_id,
            request.worker_id,
            request.daemon_epoch,
            now_sql,
            expires_sql
        ],
    )?;
    transaction.execute(
        "UPDATE tasks SET state = ?1, updated_unix_millis = ?2 WHERE task_id = ?3",
        params![TaskState::Running as i32, now_sql, selected.task_id],
    )?;
    transaction.execute(
        "UPDATE jobs SET state = ?1, updated_unix_millis = ?2
         WHERE job_id = ?3 AND state = ?4",
        params![
            JobState::Running as i32,
            now_sql,
            selected.job_id,
            JobState::Planned as i32
        ],
    )?;
    events.push(insert_event(
        &transaction,
        &selected.project_id,
        &selected.job_id,
        &selected.task_id,
        TaskState::Running as i32,
        u32_from_i64(next_attempt, "task attempt")?,
        "items",
        0,
        1,
        "",
        FailureClass::Unspecified as i32,
        now,
    )?);
    // What this task reads, in one list: the addresses its plan declared, then
    // the outputs of the tasks it depends on. Order is fixed and part of the
    // artifact key, which is what lets a stage reached by both routes — declared
    // when it runs alone, delivered by a dependency inside a larger plan — key
    // to one address instead of two.
    let mut statement = transaction.prepare(
        "SELECT artifact_id FROM task_input_artifacts
         WHERE task_id = ?1
         ORDER BY input_ordinal",
    )?;
    let declared_inputs = statement
        .query_map([&selected.task_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    let mut statement = transaction.prepare(
        "SELECT dependency.output_artifact_id
         FROM task_dependencies d
         JOIN tasks dependency ON dependency.task_id = d.depends_on_task_id
         WHERE d.task_id = ?1
         ORDER BY d.dependency_ordinal",
    )?;
    let encoded_inputs = statement
        .query_map([&selected.task_id], |row| row.get::<_, Option<String>>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    let mut input_artifact_ids = declared_inputs
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|_| StoreError::InvalidData("declared input artifact id is invalid"))
        })
        .collect::<Result<Vec<ArtifactId>, StoreError>>()?;
    for value in encoded_inputs {
        input_artifact_ids.push(
            value
                .ok_or(StoreError::InvalidData(
                    "succeeded dependency omitted artifact",
                ))?
                .parse()
                .map_err(|_| StoreError::InvalidData("dependency artifact id is invalid"))?,
        );
    }
    transaction.commit()?;
    Ok(LeaseSelection {
        task: Some(LeasedTask {
            project_id: selected.project_id,
            job_id: selected.job_id,
            source_id: selected.source_id,
            task_id: selected.task_id,
            lease_id: request.lease_id.clone(),
            kind: selected.kind,
            output_kind: selected.output_kind,
            payload: selected.payload,
            implementation: selected.implementation,
            attempt: u32_from_i64(next_attempt, "task attempt")?,
            input_artifact_ids,
            resources: ResourceDeclaration {
                cpu_threads: u32_from_i64(selected.cpu_threads, "task CPU threads")?,
                ram_bytes: u64_from_i64(selected.ram_bytes, "task RAM")?,
                accelerator_class: selected.accelerator_class,
                vram_bytes: u64_from_i64(selected.vram_bytes, "task VRAM")?,
                disk_bytes: u64_from_i64(selected.disk_bytes, "task disk")?,
                network_policy: selected.network_policy,
                thermal_class: selected.thermal_class,
                determinism_class: selected.determinism_class,
                checkpoint_support: selected.checkpoint_support,
                preemption_cost: u32_from_i64(selected.preemption_cost, "task preemption cost")?,
            },
        }),
        events,
    })
}

#[cfg(test)]
pub(super) fn lease_next_task(
    connection: &mut Connection,
    lease_id: &str,
    daemon_epoch: &str,
    now: u64,
    expires: u64,
    capacity: crate::jobs::ResourceCapacity,
) -> Result<LeaseSelection, StoreError> {
    lease_next_task_for_worker(
        connection,
        &LeaseRequest {
            lease_id: lease_id.to_owned(),
            daemon_epoch: daemon_epoch.to_owned(),
            now_unix_millis: now,
            expires_unix_millis: expires,
            capacity,
            worker_id: "builtin-fixture".to_owned(),
            capabilities: [
                "probe-source",
                "device-profile",
                "demo-seed",
                "demo-left",
                "demo-right",
                "demo-join",
            ]
            .map(str::to_owned)
            .to_vec(),
        },
    )
}

pub(super) fn heartbeat_task_with_progress(
    connection: &mut Connection,
    lease_id: &str,
    now: u64,
    expires: u64,
    progress: Option<&clipmill_contracts::proto::worker::v1::ProgressUnits>,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    if let Some(progress) = progress
        && (progress.unit.is_empty()
            || progress.unit.chars().count() > 64
            || progress.unit.chars().any(char::is_control)
            || (progress.total > 0 && progress.done > progress.total))
    {
        return Err(StoreError::InvalidData("worker progress is invalid"));
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now = sqlite_u64(now, "heartbeat timestamp")?;
    let expires = sqlite_u64(expires, "heartbeat expiry")?;
    let updated = transaction.execute(
        "UPDATE task_leases
         SET last_heartbeat_unix_millis = ?1, expires_unix_millis = ?2
         WHERE lease_id = ?3 AND status = 1
           AND expires_unix_millis > ?1",
        params![now, expires, lease_id],
    )?;
    if updated != 1 {
        return Err(StoreError::Conflict);
    }
    let mut events = Vec::new();
    if let Some(progress) = progress {
        let (task_id, job_id, project_id, attempt): (String, String, String, i64) = transaction
            .query_row(
                "SELECT t.task_id, t.job_id, j.project_id, t.attempt
                 FROM task_leases l
                 JOIN tasks t ON t.task_id = l.task_id
                 JOIN jobs j ON j.job_id = t.job_id
                 WHERE l.lease_id = ?1",
                [lease_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?;
        transaction.execute(
            "UPDATE tasks SET progress_unit = ?1, progress_done = ?2,
                progress_total = ?3, updated_unix_millis = ?4 WHERE task_id = ?5",
            params![
                progress.unit,
                sqlite_u64(progress.done, "progress done")?,
                sqlite_u64(progress.total, "progress total")?,
                now,
                task_id
            ],
        )?;
        events.push(insert_event(
            &transaction,
            &project_id,
            &job_id,
            &task_id,
            TaskState::Running as i32,
            u32_from_i64(attempt, "task attempt")?,
            &progress.unit,
            progress.done,
            progress.total,
            "",
            FailureClass::Unspecified as i32,
            u64_from_i64(now, "heartbeat timestamp")?,
        )?);
    }
    transaction.commit()?;
    Ok(events)
}

#[cfg(test)]
pub(super) fn heartbeat_task(
    connection: &mut Connection,
    lease_id: &str,
    now: u64,
    expires: u64,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    heartbeat_task_with_progress(connection, lease_id, now, expires, None)
}

pub(super) fn replay_task_completion(
    connection: &Connection,
    lease_id: &str,
    worker_id: &str,
    completion_hash: &[u8; 32],
) -> Result<Option<Vec<u8>>, StoreError> {
    let row: Option<CompletionReplayRow> = connection
        .query_row(
            "SELECT worker_id, completion_hash, completion_response
             FROM task_leases WHERE lease_id = ?1",
            [lease_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let Some((stored_worker_id, stored_hash, response)) = row else {
        return Err(StoreError::NotFound);
    };
    if stored_worker_id != worker_id {
        return Err(StoreError::Conflict);
    }
    let Some(stored_hash) = stored_hash else {
        return Ok(None);
    };
    if stored_hash.as_slice() != completion_hash {
        return Err(StoreError::Conflict);
    }
    response.map(Some).ok_or(StoreError::InvalidData(
        "completed lease omitted durable response",
    ))
}

#[allow(clippy::too_many_lines)]
pub(super) fn complete_task(
    connection: &mut Connection,
    lease_id: &str,
    artifact_id: ArtifactId,
    completion_hash: &[u8; 32],
    completion_response: &[u8],
    now: u64,
) -> Result<TaskCompletion, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let lease: Option<CompletionLeaseRow> = transaction
        .query_row(
            "SELECT task_id, status, completion_hash, completion_response
             FROM task_leases WHERE lease_id = ?1",
            [lease_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?;
    let Some((task_id, status, cached_hash, cached_response)) = lease else {
        return Err(StoreError::NotFound);
    };
    if status == 2 {
        if cached_hash.as_deref() == Some(completion_hash.as_slice()) {
            let response = cached_response.ok_or(StoreError::InvalidData(
                "completed lease omitted durable response",
            ))?;
            transaction.commit()?;
            return Ok(TaskCompletion {
                response,
                events: Vec::new(),
            });
        }
        return Err(StoreError::Conflict);
    }
    if status != 1 {
        return Err(StoreError::Conflict);
    }
    let (job_id, project_id, job_kind, source_id, attempt, is_final, cancel_requested): (
        String,
        String,
        String,
        Option<String>,
        i64,
        bool,
        bool,
    ) = transaction.query_row(
        "SELECT t.job_id, j.project_id, j.kind, j.source_id, t.attempt,
                t.is_final, j.cancel_requested
         FROM tasks t JOIN jobs j ON j.job_id = t.job_id WHERE t.task_id = ?1",
        [&task_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        },
    )?;
    if cancel_requested {
        return Err(StoreError::Conflict);
    }
    let now_sql = sqlite_u64(now, "completion timestamp")?;
    transaction.execute(
        "UPDATE task_leases SET status = 2, completion_hash = ?1, completion_response = ?2
         WHERE lease_id = ?3",
        params![completion_hash.as_slice(), completion_response, lease_id],
    )?;
    transaction.execute(
        "UPDATE tasks SET state = ?1, progress_unit = 'items', progress_done = 1,
            progress_total = 1, wait_reason = '', output_artifact_id = ?2,
            failure_class = 0, failure_detail = '', updated_unix_millis = ?3
         WHERE task_id = ?4 AND state = ?5",
        params![
            TaskState::Succeeded as i32,
            artifact_id.to_string(),
            now_sql,
            task_id,
            TaskState::Running as i32
        ],
    )?;
    transaction.execute(
        "INSERT OR IGNORE INTO task_artifact_roots(task_id, artifact_id) VALUES (?1, ?2)",
        params![task_id, artifact_id.to_string()],
    )?;
    let event = insert_event(
        &transaction,
        &project_id,
        &job_id,
        &task_id,
        TaskState::Succeeded as i32,
        u32_from_i64(attempt, "task attempt")?,
        "items",
        1,
        1,
        "",
        FailureClass::Unspecified as i32,
        now,
    )?;
    let remaining: i64 = transaction.query_row(
        "SELECT count(*) FROM tasks WHERE job_id = ?1 AND state <> ?2",
        params![job_id, TaskState::Succeeded as i32],
        |row| row.get(0),
    )?;
    if remaining == 0 {
        let final_artifact = if is_final {
            artifact_id.to_string()
        } else {
            transaction.query_row(
                "SELECT output_artifact_id FROM tasks WHERE job_id = ?1 AND is_final = 1",
                [&job_id],
                |row| row.get(0),
            )?
        };
        if job_kind == "device-profile" {
            let final_artifact_id = final_artifact
                .parse::<ArtifactId>()
                .map_err(|_| StoreError::InvalidData("device artifact id is invalid"))?;
            super::device_store::activate_profile(&transaction, &job_id, final_artifact_id, now)?;
        } else {
            transaction.execute(
                "INSERT OR IGNORE INTO project_artifact_roots(project_id, artifact_id)
                 VALUES (?1, ?2)",
                params![project_id, final_artifact],
            )?;
            if let Some(source_id) = source_id {
                transaction.execute(
                    "INSERT OR IGNORE INTO source_artifact_roots(source_id, artifact_id)
                     VALUES (?1, ?2)",
                    params![source_id, final_artifact],
                )?;
                // Only the probe job publishes the source map; other
                // source-scoped jobs (ingest) root their own final artifact
                // without stealing the source-map pointer.
                if job_kind == "probe-source" {
                    transaction.execute(
                        "UPDATE sources SET source_map_artifact_id = ?1 WHERE source_id = ?2",
                        params![final_artifact, source_id],
                    )?;
                }
            }
        }
        transaction.execute(
            "UPDATE jobs SET state = ?1, updated_unix_millis = ?2 WHERE job_id = ?3",
            params![JobState::Succeeded as i32, now_sql, job_id],
        )?;
        transaction.execute(
            "DELETE FROM task_artifact_roots
             WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id = ?1)",
            [&job_id],
        )?;
    }
    transaction.commit()?;
    Ok(TaskCompletion {
        response: completion_response.to_vec(),
        events: vec![event],
    })
}

#[allow(clippy::too_many_lines)]
pub(super) fn fail_task(
    connection: &mut Connection,
    lease_id: &str,
    failure_class: i32,
    detail: &str,
    now: u64,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    Ok(fail_task_inner(connection, lease_id, failure_class, detail, now, None)?.events)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn complete_failed_task(
    connection: &mut Connection,
    lease_id: &str,
    failure_class: i32,
    detail: &str,
    completion_hash: &[u8; 32],
    completion_response: &[u8],
    now: u64,
) -> Result<TaskCompletion, StoreError> {
    fail_task_inner(
        connection,
        lease_id,
        failure_class,
        detail,
        now,
        Some((completion_hash, completion_response)),
    )
}

#[allow(clippy::too_many_lines)]
fn fail_task_inner(
    connection: &mut Connection,
    lease_id: &str,
    failure_class: i32,
    detail: &str,
    now: u64,
    completion: Option<(&[u8; 32], &[u8])>,
) -> Result<TaskCompletion, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let row: Option<FailedTaskRow> = transaction
        .query_row(
            "SELECT t.task_id, t.job_id, j.project_id, t.kind, t.implementation,
                    t.payload, t.attempt, t.max_attempts
             FROM task_leases l JOIN tasks t ON t.task_id = l.task_id
             JOIN jobs j ON j.job_id = t.job_id
             WHERE l.lease_id = ?1 AND l.status = 1",
            [lease_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            },
        )
        .optional()?;
    let Some((task_id, job_id, project_id, kind, implementation, payload, attempt, max_attempts)) =
        row
    else {
        return Err(StoreError::Conflict);
    };
    let transient = failure_class == FailureClass::Transient as i32;
    let retry = transient && attempt < max_attempts;
    let state = if retry {
        TaskState::Retryable
    } else {
        TaskState::Failed
    };
    let backoff_seconds = 1_u64 << u32::try_from(attempt.saturating_sub(1)).unwrap_or(0).min(2);
    let next_attempt = now.saturating_add(backoff_seconds * 1_000);
    let now_sql = sqlite_u64(now, "failure timestamp")?;
    if let Some((completion_hash, completion_response)) = completion {
        transaction.execute(
            "UPDATE task_leases SET status = 3, completion_hash = ?1,
                completion_response = ?2 WHERE lease_id = ?3",
            params![completion_hash.as_slice(), completion_response, lease_id],
        )?;
    } else {
        transaction.execute(
            "UPDATE task_leases SET status = 3 WHERE lease_id = ?1",
            [lease_id],
        )?;
    }
    transaction.execute(
        "UPDATE tasks SET state = ?1, next_attempt_unix_millis = ?2,
            wait_reason = ?3, failure_class = ?4, failure_detail = ?5,
            updated_unix_millis = ?6 WHERE task_id = ?7",
        params![
            state as i32,
            sqlite_u64(next_attempt, "next retry timestamp")?,
            if retry {
                "retry: transient failure"
            } else {
                ""
            },
            failure_class,
            detail,
            now_sql,
            task_id
        ],
    )?;
    if failure_class == FailureClass::Deterministic as i32 {
        let input_key = hex::encode(Sha256::digest(payload));
        transaction.execute(
            "INSERT INTO circuit_breakers(
                stage, implementation, input_key, deterministic_failures, open, updated_unix_millis
             ) VALUES (?1, ?2, ?3, 1, 0, ?4)
             ON CONFLICT(stage, implementation, input_key) DO UPDATE SET
                deterministic_failures = deterministic_failures + 1,
                open = deterministic_failures + 1 >= 3,
                updated_unix_millis = excluded.updated_unix_millis",
            params![kind, implementation, input_key, now_sql],
        )?;
    }
    let mut events = vec![insert_event(
        &transaction,
        &project_id,
        &job_id,
        &task_id,
        state as i32,
        u32_from_i64(attempt, "task attempt")?,
        "",
        0,
        0,
        if retry {
            "retry: transient failure"
        } else {
            ""
        },
        failure_class,
        now,
    )?];
    if !retry {
        transaction.execute(
            "UPDATE jobs SET state = ?1, failure_class = ?2, failure_detail = ?3,
                updated_unix_millis = ?4 WHERE job_id = ?5",
            params![
                JobState::Failed as i32,
                failure_class,
                detail,
                now_sql,
                job_id
            ],
        )?;
        if kind == "device-profile" {
            super::device_store::fail_profile(&transaction, &job_id, now)?;
        }
        events.extend(cancel_remaining_tasks(
            &transaction,
            &project_id,
            &job_id,
            &task_id,
            now,
        )?);
        transaction.execute(
            "DELETE FROM task_artifact_roots
             WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id = ?1)",
            [&job_id],
        )?;
    }
    transaction.commit()?;
    Ok(TaskCompletion {
        response: completion.map_or_else(Vec::new, |(_, response)| response.to_vec()),
        events,
    })
}

#[allow(clippy::too_many_lines)]
pub(super) fn expire_task_leases(
    connection: &mut Connection,
    now: u64,
    daemon_epoch: &str,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let mut statement = transaction.prepare(
        "SELECT l.lease_id, t.task_id, t.job_id, j.project_id, t.attempt, t.max_attempts
         FROM task_leases l JOIN tasks t ON t.task_id = l.task_id
         JOIN jobs j ON j.job_id = t.job_id
         WHERE l.status = 1 AND (l.expires_unix_millis <= ?1 OR l.daemon_epoch <> ?2)",
    )?;
    let expired = statement
        .query_map(
            params![sqlite_u64(now, "expiry timestamp")?, daemon_epoch],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    let mut events = Vec::new();
    for (lease_id, task_id, job_id, project_id, attempt, max_attempts) in expired {
        let state = if attempt < max_attempts {
            TaskState::Retryable
        } else {
            TaskState::Failed
        };
        transaction.execute(
            "UPDATE task_leases SET status = 3 WHERE lease_id = ?1",
            [&lease_id],
        )?;
        transaction.execute(
            "UPDATE tasks SET state = ?1, next_attempt_unix_millis = ?2,
                wait_reason = ?3, failure_class = ?4, failure_detail = ?5,
                updated_unix_millis = ?2
             WHERE task_id = ?6 AND state = ?7",
            params![
                state as i32,
                sqlite_u64(now, "expiry timestamp")?,
                if state == TaskState::Retryable {
                    "retry: lease expired"
                } else {
                    ""
                },
                FailureClass::Transient as i32,
                if state == TaskState::Retryable {
                    "task lease expired"
                } else {
                    "task lease expired after maximum attempts"
                },
                task_id,
                TaskState::Running as i32,
            ],
        )?;
        events.push(insert_event(
            &transaction,
            &project_id,
            &job_id,
            &task_id,
            state as i32,
            u32_from_i64(attempt, "task attempt")?,
            "",
            0,
            0,
            if state == TaskState::Retryable {
                "retry: lease expired"
            } else {
                ""
            },
            FailureClass::Transient as i32,
            now,
        )?);
        if state == TaskState::Failed {
            transaction.execute(
                "UPDATE jobs SET state = ?1, failure_class = ?2,
                    failure_detail = 'task lease expired after maximum attempts',
                    updated_unix_millis = ?3 WHERE job_id = ?4",
                params![
                    JobState::Failed as i32,
                    FailureClass::Transient as i32,
                    sqlite_u64(now, "expiry timestamp")?,
                    job_id
                ],
            )?;
            events.extend(cancel_remaining_tasks(
                &transaction,
                &project_id,
                &job_id,
                &task_id,
                now,
            )?);
            transaction.execute(
                "DELETE FROM task_artifact_roots
                 WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id = ?1)",
                [&job_id],
            )?;
        }
    }
    transaction.commit()?;
    Ok(events)
}

/// The final artifact of the most recent succeeded job of `kind` for a source.
///
/// The render compiler uses it to find a source's reference index without the
/// caller having to thread artifact ids through the request: a client should
/// be able to ask for a render of a document, not first have to know which
/// ingest job produced which keyframe map. Absence is a normal answer — a
/// source that was never ingested simply decodes from its start.
pub(super) fn latest_source_job_artifact(
    connection: &Connection,
    source_id: &str,
    kind: &str,
) -> Result<Option<String>, StoreError> {
    let artifact_id = connection
        .query_row(
            "SELECT t.output_artifact_id
             FROM jobs j
             JOIN tasks t ON t.job_id = j.job_id
             WHERE j.source_id = ?1 AND j.kind = ?2 AND j.state = ?3
               AND t.is_final = 1 AND t.output_artifact_id IS NOT NULL
             ORDER BY j.created_unix_millis DESC, j.job_id DESC
             LIMIT 1",
            params![source_id, kind, JobState::Succeeded as i32],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten();
    Ok(artifact_id)
}

pub(super) fn current_event_id(connection: &Connection) -> Result<u64, StoreError> {
    let value: i64 = connection.query_row(
        "SELECT COALESCE(MAX(event_id), 0) FROM task_events",
        [],
        |row| row.get(0),
    )?;
    u64_from_i64(value, "event id")
}

pub(super) fn list_events(
    connection: &Connection,
    after_event_id: u64,
    filter: &EventFilter,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    let after = sqlite_u64(after_event_id, "event cursor")?;
    let mut statement = connection.prepare(
        "SELECT event_id, project_id, job_id, task_id, state, attempt,
                progress_unit, progress_done, progress_total, wait_reason,
                failure_class, at_unix_millis
         FROM task_events
         WHERE event_id > ?1
           AND (?2 IS NULL OR project_id = ?2)
           AND (?3 IS NULL OR job_id = ?3)
         ORDER BY event_id ASC",
    )?;
    let rows = statement.query_map(
        params![after, filter.project_id, filter.job_id],
        event_from_row,
    )?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn fail_open_circuit_breakers(
    transaction: &Transaction<'_>,
    now: u64,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    let mut statement = transaction.prepare(
        "SELECT t.task_id, t.job_id, j.project_id, t.attempt, t.kind, t.implementation
         FROM tasks t
         JOIN jobs j ON j.job_id = t.job_id
         JOIN circuit_breakers c
           ON c.stage = t.kind AND c.implementation = t.implementation
          AND c.input_key = t.input_key AND c.open = 1
         WHERE t.state IN (?1, ?2)
           AND j.state IN (?3, ?4)
           AND j.cancel_requested = 0
         ORDER BY j.created_unix_millis, j.job_id, t.ordinal",
    )?;
    let blocked = statement
        .query_map(
            params![
                TaskState::Planned as i32,
                TaskState::Retryable as i32,
                JobState::Planned as i32,
                JobState::Running as i32,
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);

    let now_sql = sqlite_u64(now, "circuit-breaker timestamp")?;
    let mut failed_jobs = BTreeSet::new();
    let mut events = Vec::new();
    for (task_id, job_id, project_id, attempt, stage, implementation) in blocked {
        if !failed_jobs.insert(job_id.clone()) {
            continue;
        }
        let detail = format!("circuit breaker open for {stage}/{implementation}");
        let updated = transaction.execute(
            "UPDATE tasks SET state = ?1, wait_reason = '', failure_class = ?2,
                failure_detail = ?3, updated_unix_millis = ?4
             WHERE task_id = ?5 AND state IN (?6, ?7)",
            params![
                TaskState::Failed as i32,
                FailureClass::Deterministic as i32,
                detail,
                now_sql,
                task_id,
                TaskState::Planned as i32,
                TaskState::Retryable as i32,
            ],
        )?;
        if updated == 0 {
            continue;
        }
        events.push(insert_event(
            transaction,
            &project_id,
            &job_id,
            &task_id,
            TaskState::Failed as i32,
            u32_from_i64(attempt, "task attempt")?,
            "",
            0,
            0,
            "",
            FailureClass::Deterministic as i32,
            now,
        )?);
        transaction.execute(
            "UPDATE jobs SET state = ?1, failure_class = ?2, failure_detail = ?3,
                updated_unix_millis = ?4 WHERE job_id = ?5",
            params![
                JobState::Failed as i32,
                FailureClass::Deterministic as i32,
                detail,
                now_sql,
                job_id,
            ],
        )?;
        events.extend(cancel_remaining_tasks(
            transaction,
            &project_id,
            &job_id,
            &task_id,
            now,
        )?);
        transaction.execute(
            "DELETE FROM task_artifact_roots
             WHERE task_id IN (SELECT task_id FROM tasks WHERE job_id = ?1)",
            [&job_id],
        )?;
    }
    Ok(events)
}

#[allow(clippy::too_many_lines)]
fn validate_plan(plan: &JobPlan) -> Result<(), StoreError> {
    plan.job_id
        .parse::<JobId>()
        .map_err(|_| StoreError::InvalidData("job plan id is invalid"))?;
    plan.project_id
        .parse::<ProjectId>()
        .map_err(|_| StoreError::InvalidData("job project id is invalid"))?;
    if let Some(source_id) = &plan.source_id {
        source_id
            .parse::<clipmill_core::SourceId>()
            .map_err(|_| StoreError::InvalidData("job source id is invalid"))?;
    }
    if !valid_label(&plan.kind, 128) {
        return Err(StoreError::InvalidData("job kind is invalid"));
    }
    if plan.tasks.is_empty() || plan.tasks.len() > 1024 {
        return Err(StoreError::InvalidData("job plan task count is invalid"));
    }
    let ids = plan
        .tasks
        .iter()
        .map(|task| task.task_id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != plan.tasks.len() {
        return Err(StoreError::InvalidData(
            "job plan contains duplicate task ids",
        ));
    }
    let ordinals = plan
        .tasks
        .iter()
        .map(|task| task.ordinal)
        .collect::<BTreeSet<_>>();
    if ordinals.len() != plan.tasks.len() {
        return Err(StoreError::InvalidData(
            "job plan contains duplicate ordinals",
        ));
    }
    if plan.tasks.iter().filter(|task| task.is_final).count() != 1 {
        return Err(StoreError::InvalidData(
            "job plan must have exactly one final task",
        ));
    }
    let mut incoming = BTreeMap::<&str, usize>::new();
    let mut children = BTreeMap::<&str, Vec<&str>>::new();
    for task in &plan.tasks {
        task.task_id
            .parse::<TaskId>()
            .map_err(|_| StoreError::InvalidData("task id is invalid"))?;
        // The registry is the authority on which stages exist. Refusing here
        // means an unregistered kind never reaches a worker that cannot serve
        // it, and never publishes under an address nobody assigned it.
        if !crate::recipes::plan_declaration_is_registered(&task.kind, &task.output_kind) {
            return Err(StoreError::InvalidData(
                "task kind is not a registered stage, or does not publish its registered artifact kind",
            ));
        }
        if !valid_label(&task.kind, 128)
            || !valid_label(&task.output_kind, 128)
            || task.input_kinds.iter().any(|kind| !valid_label(kind, 128))
            || task.input_kinds.len() != task.dependencies.len()
        {
            return Err(StoreError::InvalidData("task artifact kinds are invalid"));
        }
        // A declared input has to be an address. A path would name different
        // bytes on a different machine while producing the same artifact key,
        // which is the one failure a content-addressed store cannot notice
        // afterwards.
        if task
            .input_artifact_ids
            .iter()
            .any(|artifact_id| artifact_id.parse::<ArtifactId>().is_err())
        {
            return Err(StoreError::InvalidData(
                "task declares an input that is not a content address",
            ));
        }
        let distinct = task
            .input_artifact_ids
            .iter()
            .collect::<BTreeSet<_>>()
            .len();
        if distinct != task.input_artifact_ids.len() {
            return Err(StoreError::InvalidData(
                "task declares the same input artifact twice",
            ));
        }
        let resources = &task.resources;
        if resources.cpu_threads == 0
            || !matches!(
                resources.network_policy.as_str(),
                "local-lock" | "network-allowed"
            )
            || !valid_label(&resources.thermal_class, 64)
            || !valid_label(&resources.determinism_class, 64)
            || !valid_label(&task.implementation, 256)
            || !(1..=100).contains(&task.max_attempts)
        {
            return Err(StoreError::InvalidData(
                "task resource declaration is invalid",
            ));
        }
        incoming.insert(&task.task_id, task.dependencies.len());
        for dependency in &task.dependencies {
            if !ids.contains(dependency.as_str()) || dependency == &task.task_id {
                return Err(StoreError::InvalidData("job plan dependency is invalid"));
            }
            children.entry(dependency).or_default().push(&task.task_id);
        }
    }
    let mut ready = incoming
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect::<VecDeque<_>>();
    let mut visited = 0;
    while let Some(id) = ready.pop_front() {
        visited += 1;
        if let Some(task_children) = children.get(id) {
            for child in task_children {
                let count = incoming
                    .get_mut(child)
                    .ok_or(StoreError::InvalidData("job plan dependency is invalid"))?;
                *count = count.saturating_sub(1);
                if *count == 0 {
                    ready.push_back(child);
                }
            }
        }
    }
    if visited != plan.tasks.len() {
        return Err(StoreError::InvalidData("job plan contains a cycle"));
    }
    Ok(())
}

fn valid_label(value: &str, maximum: usize) -> bool {
    let count = value.chars().count();
    count > 0 && count <= maximum && !value.chars().any(char::is_control)
}

fn get_job_from(connection: &Connection, job_id: &str) -> Result<JobRecord, StoreError> {
    let header = connection
        .query_row(
            "SELECT job_id, project_id, kind, state, created_unix_millis,
                    updated_unix_millis, failure_class, failure_detail
             FROM jobs WHERE job_id = ?1",
            [job_id],
            job_header_from_row,
        )
        .optional()?
        .ok_or(StoreError::NotFound)?;
    complete_job_record(connection, header)
}

fn get_job_tx(transaction: &Transaction<'_>, job_id: &str) -> Result<JobRecord, StoreError> {
    let header = transaction
        .query_row(
            "SELECT job_id, project_id, kind, state, created_unix_millis,
                    updated_unix_millis, failure_class, failure_detail
             FROM jobs WHERE job_id = ?1",
            [job_id],
            job_header_from_row,
        )
        .optional()?
        .ok_or(StoreError::NotFound)?;
    complete_job_record(transaction, header)
}

fn complete_job_record(
    connection: &Connection,
    header: JobHeader,
) -> Result<JobRecord, StoreError> {
    let mut statement = connection.prepare(
        "SELECT task_id, kind, state, attempt, max_attempts, progress_unit,
                progress_done, progress_total, wait_reason,
                COALESCE(output_artifact_id, '')
         FROM tasks WHERE job_id = ?1 ORDER BY ordinal",
    )?;
    let tasks = statement
        .query_map([&header.job_id], task_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    let output_artifact_ids = tasks
        .iter()
        .filter(|task| {
            task.state == TaskState::Succeeded as i32 && !task.output_artifact_id.is_empty()
        })
        .filter_map(|task| {
            connection
                .query_row(
                    "SELECT is_final FROM tasks WHERE task_id = ?1",
                    [&task.task_id],
                    |row| row.get::<_, bool>(0),
                )
                .ok()
                .and_then(|is_final| is_final.then_some(task.output_artifact_id.clone()))
        })
        .collect();
    Ok(JobRecord {
        job_id: header.job_id,
        project_id: header.project_id,
        kind: header.kind,
        state: header.state,
        created_unix_millis: header.created_unix_millis,
        updated_unix_millis: header.updated_unix_millis,
        tasks,
        output_artifact_ids,
        failure_class: header.failure_class,
        failure_detail: header.failure_detail,
    })
}

struct JobHeader {
    job_id: String,
    project_id: String,
    kind: String,
    state: i32,
    created_unix_millis: u64,
    updated_unix_millis: u64,
    failure_class: i32,
    failure_detail: String,
}

fn job_header_from_row(row: &Row<'_>) -> rusqlite::Result<JobHeader> {
    Ok(JobHeader {
        job_id: row.get(0)?,
        project_id: row.get(1)?,
        kind: row.get(2)?,
        state: row.get(3)?,
        created_unix_millis: sql_u64(row, 4)?,
        updated_unix_millis: sql_u64(row, 5)?,
        failure_class: row.get(6)?,
        failure_detail: row.get(7)?,
    })
}

fn task_from_row(row: &Row<'_>) -> rusqlite::Result<TaskRecord> {
    Ok(TaskRecord {
        task_id: row.get(0)?,
        kind: row.get(1)?,
        state: row.get(2)?,
        attempt: sql_u32(row, 3)?,
        max_attempts: sql_u32(row, 4)?,
        progress_unit: row.get(5)?,
        progress_done: sql_u64(row, 6)?,
        progress_total: sql_u64(row, 7)?,
        wait_reason: row.get(8)?,
        output_artifact_id: row.get(9)?,
    })
}

#[allow(clippy::too_many_arguments)]
fn insert_event(
    transaction: &Transaction<'_>,
    project_id: &str,
    job_id: &str,
    task_id: &str,
    state: i32,
    attempt: u32,
    progress_unit: &str,
    progress_done: u64,
    progress_total: u64,
    wait_reason: &str,
    failure_class: i32,
    at: u64,
) -> Result<TaskEventRecord, StoreError> {
    transaction.execute(
        "INSERT INTO task_events(
            project_id, job_id, task_id, state, attempt, progress_unit,
            progress_done, progress_total, wait_reason, failure_class, at_unix_millis
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            project_id,
            job_id,
            task_id,
            state,
            i64::from(attempt),
            progress_unit,
            sqlite_u64(progress_done, "progress")?,
            sqlite_u64(progress_total, "progress total")?,
            wait_reason,
            failure_class,
            sqlite_u64(at, "event timestamp")?,
        ],
    )?;
    let event_id = u64::try_from(transaction.last_insert_rowid())
        .map_err(|_| StoreError::InvalidData("event id exceeds range"))?;
    Ok(TaskEventRecord {
        event_id,
        project_id: project_id.to_owned(),
        job_id: job_id.to_owned(),
        task_id: task_id.to_owned(),
        state,
        attempt,
        progress_unit: progress_unit.to_owned(),
        progress_done,
        progress_total,
        wait_reason: wait_reason.to_owned(),
        failure_class,
        at_unix_millis: at,
    })
}

fn event_from_row(row: &Row<'_>) -> rusqlite::Result<TaskEventRecord> {
    Ok(TaskEventRecord {
        event_id: sql_u64(row, 0)?,
        project_id: row.get(1)?,
        job_id: row.get(2)?,
        task_id: row.get(3)?,
        state: row.get(4)?,
        attempt: sql_u32(row, 5)?,
        progress_unit: row.get(6)?,
        progress_done: sql_u64(row, 7)?,
        progress_total: sql_u64(row, 8)?,
        wait_reason: row.get(9)?,
        failure_class: row.get(10)?,
        at_unix_millis: sql_u64(row, 11)?,
    })
}

fn cancel_remaining_tasks(
    transaction: &Transaction<'_>,
    project_id: &str,
    job_id: &str,
    failed_task_id: &str,
    now: u64,
) -> Result<Vec<TaskEventRecord>, StoreError> {
    let mut statement = transaction.prepare(
        "SELECT task_id, attempt FROM tasks
         WHERE job_id = ?1 AND task_id <> ?2 AND state NOT IN (?3, ?4, ?5)
         ORDER BY ordinal",
    )?;
    let rows = statement
        .query_map(
            params![
                job_id,
                failed_task_id,
                TaskState::Succeeded as i32,
                TaskState::Failed as i32,
                TaskState::Cancelled as i32
            ],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    let mut events = Vec::new();
    for (task_id, attempt) in rows {
        transaction.execute(
            "UPDATE tasks SET state = ?1, wait_reason = '', updated_unix_millis = ?2
             WHERE task_id = ?3",
            params![
                TaskState::Cancelled as i32,
                sqlite_u64(now, "cancel timestamp")?,
                task_id
            ],
        )?;
        transaction.execute(
            "UPDATE task_leases SET status = 4 WHERE task_id = ?1 AND status = 1",
            [&task_id],
        )?;
        events.push(insert_event(
            transaction,
            project_id,
            job_id,
            &task_id,
            TaskState::Cancelled as i32,
            u32_from_i64(attempt, "task attempt")?,
            "",
            0,
            0,
            "",
            FailureClass::Unspecified as i32,
            now,
        )?);
    }
    Ok(events)
}

fn aggregate_all_jobs(transaction: &Transaction<'_>, now: u64) -> Result<(), StoreError> {
    let mut statement =
        transaction.prepare("SELECT job_id FROM jobs WHERE state IN (?1, ?2, ?3)")?;
    let ids = statement
        .query_map(
            params![
                JobState::Planned as i32,
                JobState::Running as i32,
                JobState::CancelRequested as i32
            ],
            |row| row.get::<_, String>(0),
        )?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    for job_id in ids {
        let failed: i64 = transaction.query_row(
            "SELECT count(*) FROM tasks WHERE job_id = ?1 AND state = ?2",
            params![job_id, TaskState::Failed as i32],
            |row| row.get(0),
        )?;
        if failed > 0 {
            transaction.execute(
                "UPDATE jobs SET state = ?1, failure_class = ?2,
                    failure_detail = 'interrupted task exhausted attempts', updated_unix_millis = ?3
                 WHERE job_id = ?4",
                params![
                    JobState::Failed as i32,
                    FailureClass::Transient as i32,
                    sqlite_u64(now, "aggregate timestamp")?,
                    job_id
                ],
            )?;
        }
    }
    Ok(())
}

fn sqlite_u64(value: u64, label: &'static str) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidData(label))
}

fn sqlite_usize(value: usize) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidData("collection index exceeds range"))
}

fn u64_from_i64(value: i64, label: &'static str) -> Result<u64, StoreError> {
    u64::try_from(value).map_err(|_| StoreError::InvalidData(label))
}

fn u32_from_i64(value: i64, label: &'static str) -> Result<u32, StoreError> {
    u32::try_from(value).map_err(|_| StoreError::InvalidData(label))
}

fn sql_u64(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}

fn sql_u32(row: &Row<'_>, index: usize) -> rusqlite::Result<u32> {
    let value: i64 = row.get(index)?;
    u32::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}
