//! Machine-wide admission runs in the same transaction that creates the lease.
//! An idle model still owns its reservation until its worker confirms unloading.

use rusqlite::{OptionalExtension, Transaction, params};

use super::{SelectedTaskRow, StoreError, sqlite_u64};
use crate::jobs::{LeaseRequest, ResourceCapacity, WorkerResidency};

#[derive(Clone, Copy, Default)]
struct Budget {
    cpu: i64,
    ram: i64,
    disk: i64,
    vram: i64,
}

impl Budget {
    fn read(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            cpu: row.get(0)?,
            ram: row.get(1)?,
            disk: row.get(2)?,
            vram: row.get(3)?,
        })
    }

    fn available(machine: ResourceCapacity) -> Self {
        Self {
            cpu: i64::from(machine.cpu_threads),
            ram: i64::try_from(machine.ram_bytes).unwrap_or(i64::MAX),
            disk: i64::try_from(machine.disk_bytes).unwrap_or(i64::MAX),
            vram: i64::try_from(machine.vram_bytes).unwrap_or(i64::MAX),
        }
    }

    fn without(self, used: Self) -> Self {
        Self {
            cpu: self.cpu - used.cpu,
            ram: self.ram - used.ram,
            disk: self.disk - used.disk,
            vram: self.vram - used.vram,
        }
    }

    fn fits(self, t: &SelectedTaskRow) -> bool {
        self.cpu >= t.cpu_threads
            && self.ram >= t.ram_bytes
            && self.disk >= t.disk_bytes
            && self.vram >= t.vram_bytes
    }
}

pub(super) struct Admission {
    free: Budget,
    background: Budget,
    release: bool,
    own_model: Option<String>,
}

impl Admission {
    pub(super) fn read(tx: &Transaction<'_>, request: &LeaseRequest) -> Result<Self, StoreError> {
        let now = sqlite_u64(request.now_unix_millis, "worker poll")?;
        // Both tables belong to the daemon connection. Durable leases already
        // record active work; ephemeral allocations and waiting clients do not survive it.
        tx.execute_batch(
            "CREATE TEMP TABLE IF NOT EXISTS idle_models(
                worker TEXT PRIMARY KEY, model TEXT NOT NULL, ram INTEGER NOT NULL,
                disk INTEGER NOT NULL, vram INTEGER NOT NULL, reclaim INTEGER NOT NULL DEFAULT 0,
                disconnected_until INTEGER);
             CREATE TEMP TABLE IF NOT EXISTS priority_wait(
                worker TEXT PRIMARY KEY, task TEXT NOT NULL, expires INTEGER NOT NULL);",
        )?;
        tx.execute(
            "DELETE FROM idle_models WHERE disconnected_until <= ?1",
            [now],
        )?;
        tx.execute(
            "DELETE FROM priority_wait WHERE expires <= ?1 OR worker=?2",
            params![now, request.worker_id],
        )?;
        match request.residency {
            WorkerResidency::Cold => {
                tx.execute(
                    "DELETE FROM idle_models WHERE worker=?1",
                    [&request.worker_id],
                )?;
            }
            WorkerResidency::Warm => {
                // A daemon that restarted can meet a still-loaded peer. Until it
                // unloads, charge its entire authenticated working-memory ceiling.
                tx.execute("INSERT INTO idle_models(worker,model,ram,disk,vram,reclaim)
                    VALUES (?1,'',?2,0,?3,1) ON CONFLICT(worker) DO UPDATE SET disconnected_until=NULL",
                    params![request.worker_id, sqlite_u64(request.capacity.ram_bytes,"worker memory")?,
                            sqlite_u64(request.capacity.vram_bytes,"worker video memory")?])?;
            }
            WorkerResidency::Stateless | WorkerResidency::Legacy => {}
        }
        let own: Option<(String, bool)> = tx
            .query_row(
                "SELECT model,reclaim FROM idle_models WHERE worker=?1",
                [&request.worker_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let active = tx.query_row(
            "SELECT COALESCE(SUM(t.cpu_threads),0), COALESCE(SUM(t.ram_bytes),0),
                    COALESCE(SUM(t.disk_bytes),0), COALESCE(SUM(t.vram_bytes),0)
             FROM task_leases l JOIN tasks t ON t.task_id=l.task_id
             WHERE l.status=1 OR (l.status=4 AND l.expires_unix_millis>?1)",
            [now],
            Budget::read,
        )?;
        let idle = tx.query_row(
            "WITH running AS (
                SELECT l.worker_id, SUM(t.ram_bytes) AS ram, SUM(t.disk_bytes) AS disk,
                       SUM(t.vram_bytes) AS vram
                FROM task_leases l JOIN tasks t ON t.task_id=l.task_id
                WHERE l.status=1 OR (l.status=4 AND l.expires_unix_millis>?2)
                GROUP BY l.worker_id)
             SELECT 0, COALESCE(SUM(MAX(0,m.ram-COALESCE(r.ram,0))),0),
                    COALESCE(SUM(MAX(0,m.disk-COALESCE(r.disk,0))),0),
                    COALESCE(SUM(MAX(0,m.vram-COALESCE(r.vram,0))),0)
             FROM idle_models m LEFT JOIN running r ON r.worker_id=m.worker
             WHERE m.worker<>?1",
            params![request.worker_id, now],
            Budget::read,
        )?;
        let protected = tx
            .query_row(
                "SELECT t.cpu_threads,t.ram_bytes,t.disk_bytes,t.vram_bytes FROM priority_wait w
             JOIN tasks t ON t.task_id=w.task JOIN jobs j ON j.job_id=t.job_id
             WHERE t.state IN (?1,?2) AND j.cancel_requested=0
             ORDER BY j.created_unix_millis, j.job_id, t.ordinal LIMIT 1",
                params![
                    super::TaskState::Planned as i32,
                    super::TaskState::Retryable as i32
                ],
                Budget::read,
            )
            .optional()?
            .unwrap_or_default();
        let free = Budget::available(request.machine)
            .without(active)
            .without(idle);
        Ok(Self {
            free,
            background: free.without(protected),
            release: matches!(request.residency, WorkerResidency::Warm)
                && own.as_ref().is_some_and(|(_, reclaim)| *reclaim),
            own_model: own.map(|(model, _)| model),
        })
    }

    pub(super) fn fits(&self, task: &SelectedTaskRow) -> bool {
        !self.release
            && (if task.interactive {
                self.free
            } else {
                self.background
            })
            .fits(task)
    }

    pub(super) fn changing_model(&self, request: &LeaseRequest, task: &SelectedTaskRow) -> bool {
        matches!(request.residency, WorkerResidency::Warm)
            && self.own_model.as_deref().is_some_and(|model| {
                crate::implementations::lookup(&task.implementation)
                    .is_none_or(|i| i.model != model)
            })
    }

    pub(super) fn note(
        &self,
        tx: &Transaction<'_>,
        request: &LeaseRequest,
        blocked: Option<&(String, bool)>,
        selected: Option<&SelectedTaskRow>,
    ) -> Result<bool, StoreError> {
        if let Some((id, interactive)) = blocked {
            if *interactive {
                tx.execute(
                    "INSERT OR REPLACE INTO priority_wait(worker,task,expires) VALUES (?1,?2,?3)",
                    params![
                        request.worker_id,
                        id,
                        sqlite_u64(request.expires_unix_millis, "priority wait expiry")?
                    ],
                )?;
            }
            // CPU-only contention benefits from waiting for the running task.
            // Ask idle models to unload only if retained memory or disk can help.
            tx.execute("UPDATE idle_models SET reclaim=1 WHERE worker<>?1
                AND NOT EXISTS (SELECT 1 FROM task_leases l WHERE l.status=1 AND l.worker_id=idle_models.worker)
                AND EXISTS (SELECT 1 FROM tasks t WHERE t.task_id=?2 AND
                    (t.ram_bytes>?3 OR t.disk_bytes>?4 OR t.vram_bytes>?5))",
                params![request.worker_id, id, self.free.ram, self.free.disk, self.free.vram])?;
        }
        if let Some(task) = selected
            && !matches!(request.residency, WorkerResidency::Stateless)
            && let Some(implementation) = crate::implementations::lookup(&task.implementation)
            && implementation.capability == "editorial"
        {
            tx.execute(
                "INSERT INTO idle_models(worker,model,ram,disk,vram) VALUES (?1,?2,?3,?4,?5)
                ON CONFLICT(worker) DO UPDATE SET model=excluded.model,
                ram=MAX(ram,excluded.ram),disk=MAX(disk,excluded.disk),vram=MAX(vram,excluded.vram),
                disconnected_until=NULL",
                params![
                    request.worker_id,
                    implementation.model,
                    task.ram_bytes,
                    task.disk_bytes,
                    task.vram_bytes
                ],
            )?;
        }
        Ok(self.release)
    }
}

/// A current SDK unloads at session teardown. A disconnected older worker may
/// still be emptying its 90-second idle cache; keep its last declaration through that interval.
pub(in crate::db) fn disconnect(
    db: &rusqlite::Connection,
    worker: &str,
    now: u64,
) -> Result<(), StoreError> {
    if db.query_row(
        "SELECT COUNT(*) FROM sqlite_temp_schema WHERE name='idle_models'",
        [],
        |r| r.get::<_, i64>(0),
    )? != 0
    {
        db.execute(
            "UPDATE idle_models SET disconnected_until=?2 WHERE worker=?1",
            params![
                worker,
                sqlite_u64(now.saturating_add(95_000), "worker disconnect grace")?
            ],
        )?;
        db.execute("DELETE FROM priority_wait WHERE worker=?1", [worker])?;
    }
    Ok(())
}
