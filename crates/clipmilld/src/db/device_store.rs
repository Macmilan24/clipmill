use clipmill_core::ArtifactId;
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use super::{StoreError, job_store};
use crate::jobs::{JobPlan, SYSTEM_PROJECT_ID, TaskEventRecord};

pub(super) const CREATE_V5_TABLES: &str = "
    ALTER TABLE projects ADD COLUMN is_system INTEGER NOT NULL DEFAULT 0
        CHECK(is_system IN (0, 1));

    INSERT INTO projects(project_id, name, created_unix_millis, is_system)
    VALUES ('prj_00000000000000000000000000', 'ClipMill system', 0, 1);

    CREATE TABLE device_profile_generations (
        hardware_fingerprint TEXT NOT NULL
            CHECK(
                length(hardware_fingerprint) = 71
                AND substr(hardware_fingerprint, 1, 7) = 'sha256:'
                AND substr(hardware_fingerprint, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        measurement_generation INTEGER NOT NULL
            CHECK(measurement_generation >= 1),
        job_id TEXT NOT NULL UNIQUE REFERENCES jobs(job_id) ON DELETE CASCADE,
        state INTEGER NOT NULL CHECK(state IN (0, 1, 2)),
        profile_json TEXT,
        artifact_id TEXT
            CHECK(
                artifact_id IS NULL OR (
                    length(artifact_id) = 71
                    AND substr(artifact_id, 1, 7) = 'sha256:'
                    AND substr(artifact_id, 8) NOT GLOB '*[^0-9a-f]*'
                )
            ),
        created_unix_millis INTEGER NOT NULL CHECK(created_unix_millis >= 0),
        updated_unix_millis INTEGER NOT NULL CHECK(updated_unix_millis >= 0),
        PRIMARY KEY(hardware_fingerprint, measurement_generation)
    ) STRICT, WITHOUT ROWID;

    CREATE UNIQUE INDEX device_profile_one_inflight
        ON device_profile_generations(hardware_fingerprint)
        WHERE state = 0;

    CREATE INDEX device_profiles_by_fingerprint_generation
        ON device_profile_generations(
            hardware_fingerprint, measurement_generation DESC
        );

    CREATE TABLE device_profile_requests (
        request_id TEXT PRIMARY KEY
            REFERENCES request_dedup(request_id) ON DELETE CASCADE,
        request_hash BLOB NOT NULL CHECK(length(request_hash) = 32),
        hardware_fingerprint TEXT NOT NULL,
        measurement_generation INTEGER NOT NULL,
        response_blob BLOB,
        created_unix_millis INTEGER NOT NULL CHECK(created_unix_millis >= 0),
        completed_unix_millis INTEGER CHECK(completed_unix_millis >= 0),
        FOREIGN KEY(hardware_fingerprint, measurement_generation)
            REFERENCES device_profile_generations(
                hardware_fingerprint, measurement_generation
            )
    ) STRICT;

    CREATE TABLE system_artifact_roots (
        root_kind TEXT PRIMARY KEY CHECK(length(root_kind) BETWEEN 1 AND 128),
        artifact_id TEXT NOT NULL
            CHECK(
                length(artifact_id) = 71
                AND substr(artifact_id, 1, 7) = 'sha256:'
                AND substr(artifact_id, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        updated_unix_millis INTEGER NOT NULL CHECK(updated_unix_millis >= 0)
    ) STRICT, WITHOUT ROWID;

    CREATE INDEX system_artifact_roots_by_artifact
        ON system_artifact_roots(artifact_id);
";

const PROFILE_PENDING: i64 = 0;
const PROFILE_SUCCEEDED: i64 = 1;
const PROFILE_FAILED: i64 = 2;
const ACTIVE_PROFILE_ROOT: &str = "active-device-profile";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeviceProfileRecord {
    pub hardware_fingerprint: String,
    pub measurement_generation: u64,
    pub job_id: String,
    pub state: DeviceProfileState,
    pub profile_json: Option<String>,
    pub artifact_id: Option<ArtifactId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeviceProfileState {
    Pending,
    Succeeded,
    Failed,
}

#[derive(Debug)]
pub(crate) enum BeginDeviceProfile {
    Response {
        bytes: Vec<u8>,
        record: DeviceProfileRecord,
    },
    Profile {
        record: DeviceProfileRecord,
        events: Vec<TaskEventRecord>,
    },
}

pub(super) fn begin_device_profile(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    hardware_fingerprint: &str,
    remeasure: bool,
    now: u64,
) -> Result<BeginDeviceProfile, StoreError> {
    hardware_fingerprint
        .parse::<ArtifactId>()
        .map_err(|_| StoreError::InvalidData("device fingerprint is invalid"))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

    if let Some((stored_hash, response, fingerprint, generation)) = transaction
        .query_row(
            "SELECT request_hash, response_blob, hardware_fingerprint,
                    measurement_generation
             FROM device_profile_requests WHERE request_id = ?1",
            [request_id],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Option<Vec<u8>>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?
    {
        if stored_hash.as_slice() != request_hash {
            return Err(StoreError::Conflict);
        }
        if let Some(response) = response {
            let record = profile_record(&transaction, &fingerprint, generation)?;
            transaction.commit()?;
            return Ok(BeginDeviceProfile::Response {
                bytes: response,
                record,
            });
        }
        let record = profile_record(&transaction, &fingerprint, generation)?;
        transaction.commit()?;
        return Ok(BeginDeviceProfile::Profile {
            record,
            events: Vec::new(),
        });
    }

    let request_claim: Option<Vec<u8>> = transaction
        .query_row(
            "SELECT request_hash FROM request_dedup WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .optional()?;
    if request_claim.is_some() {
        return Err(StoreError::Conflict);
    }

    let mut events = Vec::new();
    let record = if remeasure {
        None
    } else {
        active_profile(&transaction, hardware_fingerprint)?
    }
    .or(inflight_profile(&transaction, hardware_fingerprint)?)
    .map_or_else(
        || create_profile_job(&transaction, hardware_fingerprint, now, &mut events),
        Ok,
    )?;

    let now_sql = sqlite_u64(now, "device request timestamp")?;
    transaction.execute(
        "INSERT INTO request_dedup(
            request_id, request_hash, response_blob, completed_unix_millis
         ) VALUES (?1, ?2, X'', ?3)",
        params![request_id, request_hash.as_slice(), now_sql],
    )?;
    transaction.execute(
        "INSERT INTO device_profile_requests(
            request_id, request_hash, hardware_fingerprint,
            measurement_generation, created_unix_millis
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            request_id,
            request_hash.as_slice(),
            record.hardware_fingerprint,
            sqlite_u64(record.measurement_generation, "device profile generation")?,
            now_sql
        ],
    )?;
    transaction.commit()?;
    Ok(BeginDeviceProfile::Profile { record, events })
}

pub(super) fn profile_for_job(
    connection: &Connection,
    job_id: &str,
) -> Result<DeviceProfileRecord, StoreError> {
    let (fingerprint, generation): (String, i64) = connection
        .query_row(
            "SELECT hardware_fingerprint, measurement_generation
             FROM device_profile_generations WHERE job_id = ?1",
            [job_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or(StoreError::NotFound)?;
    profile_record(connection, &fingerprint, generation)
}

pub(super) fn current_profile(
    connection: &Connection,
    hardware_fingerprint: &str,
) -> Result<Option<DeviceProfileRecord>, StoreError> {
    active_profile(connection, hardware_fingerprint)
}

pub(super) fn store_profile_json(
    connection: &mut Connection,
    job_id: &str,
    profile_json: &str,
    now: u64,
) -> Result<DeviceProfileRecord, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (fingerprint, generation, state, existing): (String, i64, i64, Option<String>) =
        transaction
            .query_row(
                "SELECT hardware_fingerprint, measurement_generation, state, profile_json
             FROM device_profile_generations WHERE job_id = ?1",
                [job_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?
            .ok_or(StoreError::NotFound)?;
    if state != PROFILE_PENDING {
        return Err(StoreError::Conflict);
    }
    if let Some(existing) = existing {
        if existing != profile_json {
            return Err(StoreError::Conflict);
        }
    } else {
        transaction.execute(
            "UPDATE device_profile_generations
             SET profile_json = ?1, updated_unix_millis = ?2 WHERE job_id = ?3",
            params![
                profile_json,
                sqlite_u64(now, "device measurement timestamp")?,
                job_id
            ],
        )?;
    }
    let record = profile_record(&transaction, &fingerprint, generation)?;
    transaction.commit()?;
    Ok(record)
}

pub(super) fn finish_device_request(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    artifact_id: ArtifactId,
    response: &[u8],
    now: u64,
) -> Result<Vec<u8>, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (stored_hash, stored_response, fingerprint, generation): (
        Vec<u8>,
        Option<Vec<u8>>,
        String,
        i64,
    ) = transaction
        .query_row(
            "SELECT request_hash, response_blob, hardware_fingerprint,
                    measurement_generation
             FROM device_profile_requests WHERE request_id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?
        .ok_or(StoreError::NotFound)?;
    if stored_hash.as_slice() != request_hash {
        return Err(StoreError::Conflict);
    }
    if let Some(stored_response) = stored_response {
        if stored_response == response {
            transaction.commit()?;
            return Ok(stored_response);
        }
        return Err(StoreError::Conflict);
    }
    let record = profile_record(&transaction, &fingerprint, generation)?;
    if record.state != DeviceProfileState::Succeeded
        || record.artifact_id != Some(artifact_id)
        || record.profile_json.is_none()
    {
        return Err(StoreError::Conflict);
    }
    let now_sql = sqlite_u64(now, "device response timestamp")?;
    transaction.execute(
        "UPDATE request_dedup
         SET response_blob = ?1, completed_unix_millis = ?2 WHERE request_id = ?3",
        params![response, now_sql, request_id],
    )?;
    transaction.execute(
        "UPDATE device_profile_requests
         SET response_blob = ?1, completed_unix_millis = ?2 WHERE request_id = ?3",
        params![response, now_sql, request_id],
    )?;
    transaction.commit()?;
    Ok(response.to_vec())
}

pub(super) fn activate_profile(
    transaction: &Transaction<'_>,
    job_id: &str,
    artifact_id: ArtifactId,
    now: u64,
) -> Result<(), StoreError> {
    let profile_json: Option<String> = transaction
        .query_row(
            "SELECT profile_json FROM device_profile_generations
             WHERE job_id = ?1 AND state = 0",
            [job_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    if profile_json.is_none() {
        return Err(StoreError::InvalidData(
            "device profile completed without measured profile JSON",
        ));
    }
    let now_sql = sqlite_u64(now, "device profile completion timestamp")?;
    transaction.execute(
        "UPDATE device_profile_generations
         SET state = 1, artifact_id = ?1, updated_unix_millis = ?2
         WHERE job_id = ?3 AND state = 0",
        params![artifact_id.to_string(), now_sql, job_id],
    )?;
    transaction.execute(
        "INSERT INTO system_artifact_roots(root_kind, artifact_id, updated_unix_millis)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(root_kind) DO UPDATE SET
            artifact_id = excluded.artifact_id,
            updated_unix_millis = excluded.updated_unix_millis",
        params![ACTIVE_PROFILE_ROOT, artifact_id.to_string(), now_sql],
    )?;
    Ok(())
}

pub(super) fn fail_profile(
    transaction: &Transaction<'_>,
    job_id: &str,
    now: u64,
) -> Result<(), StoreError> {
    transaction.execute(
        "UPDATE device_profile_generations
         SET state = 2, updated_unix_millis = ?1 WHERE job_id = ?2 AND state = 0",
        params![sqlite_u64(now, "device profile failure timestamp")?, job_id],
    )?;
    Ok(())
}

fn create_profile_job(
    transaction: &Transaction<'_>,
    hardware_fingerprint: &str,
    now: u64,
    events: &mut Vec<TaskEventRecord>,
) -> Result<DeviceProfileRecord, StoreError> {
    let generation: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(measurement_generation), 0) + 1
         FROM device_profile_generations WHERE hardware_fingerprint = ?1",
        [hardware_fingerprint],
        |row| row.get(0),
    )?;
    let generation_u64 = u64::try_from(generation)
        .map_err(|_| StoreError::InvalidData("device profile generation is invalid"))?;
    let plan = JobPlan::device_profile(hardware_fingerprint.to_owned(), generation_u64, now);
    if plan.project_id != SYSTEM_PROJECT_ID {
        return Err(StoreError::InvalidData("device job project is invalid"));
    }
    *events = job_store::insert_job_plan(transaction, &plan)?;
    let now_sql = sqlite_u64(now, "device profile timestamp")?;
    transaction.execute(
        "INSERT INTO device_profile_generations(
            hardware_fingerprint, measurement_generation, job_id, state,
            created_unix_millis, updated_unix_millis
         ) VALUES (?1, ?2, ?3, 0, ?4, ?4)",
        params![hardware_fingerprint, generation, plan.job_id, now_sql],
    )?;
    profile_record(transaction, hardware_fingerprint, generation)
}

fn active_profile(
    connection: &Connection,
    hardware_fingerprint: &str,
) -> Result<Option<DeviceProfileRecord>, StoreError> {
    let generation: Option<i64> = connection
        .query_row(
            "SELECT d.measurement_generation
             FROM device_profile_generations d
             JOIN system_artifact_roots r
               ON r.root_kind = ?1 AND r.artifact_id = d.artifact_id
             WHERE d.hardware_fingerprint = ?2 AND d.state = 1
             ORDER BY d.measurement_generation DESC LIMIT 1",
            params![ACTIVE_PROFILE_ROOT, hardware_fingerprint],
            |row| row.get(0),
        )
        .optional()?;
    generation
        .map(|generation| profile_record(connection, hardware_fingerprint, generation))
        .transpose()
}

fn inflight_profile(
    connection: &Connection,
    hardware_fingerprint: &str,
) -> Result<Option<DeviceProfileRecord>, StoreError> {
    let generation: Option<i64> = connection
        .query_row(
            "SELECT measurement_generation FROM device_profile_generations
             WHERE hardware_fingerprint = ?1 AND state = 0",
            [hardware_fingerprint],
            |row| row.get(0),
        )
        .optional()?;
    generation
        .map(|generation| profile_record(connection, hardware_fingerprint, generation))
        .transpose()
}

fn profile_record(
    connection: &Connection,
    hardware_fingerprint: &str,
    generation: i64,
) -> Result<DeviceProfileRecord, StoreError> {
    let (job_id, state, profile_json, artifact_id): (String, i64, Option<String>, Option<String>) =
        connection
            .query_row(
                "SELECT job_id, state, profile_json, artifact_id
                 FROM device_profile_generations
                 WHERE hardware_fingerprint = ?1 AND measurement_generation = ?2",
                params![hardware_fingerprint, generation],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?
            .ok_or(StoreError::NotFound)?;
    let state = match state {
        PROFILE_PENDING => DeviceProfileState::Pending,
        PROFILE_SUCCEEDED => DeviceProfileState::Succeeded,
        PROFILE_FAILED => DeviceProfileState::Failed,
        _ => return Err(StoreError::InvalidData("device profile state is invalid")),
    };
    let artifact_id = artifact_id
        .map(|value| {
            value
                .parse()
                .map_err(|_| StoreError::InvalidData("device profile artifact id is invalid"))
        })
        .transpose()?;
    Ok(DeviceProfileRecord {
        hardware_fingerprint: hardware_fingerprint.to_owned(),
        measurement_generation: u64::try_from(generation)
            .map_err(|_| StoreError::InvalidData("device profile generation is invalid"))?,
        job_id,
        state,
        profile_json,
        artifact_id,
    })
}

fn sqlite_u64(value: u64, label: &'static str) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidData(label))
}
