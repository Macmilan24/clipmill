//! Garbage collection of the artifact store, one pass at a time.
//!
//! Two things ask for a pass: the schedule, every six hours with the retention
//! period as its grace, and a person freeing space from Settings, now and with
//! a grace of minutes. Both run through this one loop. The store yields a pass
//! whenever another command is waiting for it, so two collectors taking turns
//! would each yield to the other's retry and neither would finish.
//!
//! A pass that yields to foreground work is retried after a pause, from fresh
//! roots and reader pins. The store keeps the verification it had done, so the
//! retry carries on from there instead of starting over.

use std::time::{Duration, Instant, SystemTime};

use clipmill_artifacts::GcReport;
use thiserror::Error;
use tokio::{
    sync::{mpsc, oneshot},
    time::sleep,
};

use crate::{artifacts::ArtifactHandle, db::DbHandle};

/// How often the scheduled pass runs.
const INTERVAL: Duration = Duration::from_hours(6);
/// The quiet interval before a deferred scheduled pass is retried.
const IDLE_RETRY: Duration = Duration::from_secs(2);
/// The pause before a deferred clean-up is retried. Shorter, because somebody
/// is waiting for it; cheap, because the retry resumes rather than restarts.
const CLEAN_UP_RETRY: Duration = Duration::from_millis(250);
const REQUEST_CAPACITY: usize = 8;

/// Asks the collection loop for a clean-up now.
#[derive(Clone, Debug)]
pub(crate) struct Collector {
    requests: mpsc::Sender<CleanUp>,
}

/// A clean-up somebody is waiting for.
#[derive(Debug)]
pub(crate) struct CleanUp {
    grace: Duration,
    reply: oneshot::Sender<Result<Freed, CleanUpError>>,
}

/// What a clean-up removed, over every attempt its pass took.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Freed {
    pub(crate) objects: u64,
    pub(crate) bytes: u64,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub(crate) enum CleanUpError {
    #[error("the artifact roots could not be read")]
    Roots,
    /// The store refused to collect, which it does rather than delete anything
    /// while a reachable object is missing or corrupt.
    #[error("{0}")]
    Refused(String),
    #[error("the collector stopped")]
    Stopped,
}

impl Collector {
    pub(crate) fn new() -> (Self, mpsc::Receiver<CleanUp>) {
        let (requests, received) = mpsc::channel(REQUEST_CAPACITY);
        (Self { requests }, received)
    }

    /// Collect every object no root reaches that is older than `grace`, and
    /// wait for the pass to finish.
    ///
    /// The pass carries on if the caller stops waiting: somebody asked for the
    /// space back, and it is freed whether or not they are still looking.
    pub(crate) async fn clean_up(&self, grace: Duration) -> Result<Freed, CleanUpError> {
        let (reply, answered) = oneshot::channel();
        self.requests
            .send(CleanUp { grace, reply })
            .await
            .map_err(|_| CleanUpError::Stopped)?;
        answered.await.map_err(|_| CleanUpError::Stopped)?
    }
}

/// Run collection passes until `stopped`: at startup, every [`INTERVAL`] after
/// a pass completes, and whenever a clean-up is asked for.
pub(crate) async fn run(
    artifacts: ArtifactHandle,
    database: DbHandle,
    retention: Duration,
    mut requests: mpsc::Receiver<CleanUp>,
    mut stopped: oneshot::Receiver<()>,
) {
    let mut delay = Duration::ZERO;
    // The clean-ups the pass in progress will answer, and what it has freed
    // for them so far.
    let mut waiting: Vec<CleanUp> = Vec::new();
    let mut freed = Freed::default();
    'passes: loop {
        tokio::select! {
            biased;
            _ = &mut stopped => break,
            Some(request) = requests.recv(), if waiting.is_empty() => waiting.push(request),
            () = sleep(delay) => {}
        }
        while let Ok(request) = requests.try_recv() {
            waiting.push(request);
        }
        let grace = waiting
            .iter()
            .map(|request| request.grace)
            .fold(retention, Duration::min);
        let started = Instant::now();
        let Ok(roots) = database.list_artifact_roots().await else {
            tracing::warn!(
                operation = "gc",
                latency_ms = latency_millis(started),
                result = "error",
                "cannot read artifact GC roots"
            );
            answer(&mut waiting, &Err(CleanUpError::Roots));
            freed = Freed::default();
            delay = INTERVAL;
            continue;
        };
        let attempt = artifacts.collect(roots, SystemTime::now(), grace);
        tokio::pin!(attempt);
        let collected = loop {
            tokio::select! {
                biased;
                _ = &mut stopped => break 'passes,
                result = &mut attempt => break Some(result),
                Some(request) = requests.recv() => {
                    // Asked for during a scheduled pass, a clean-up would
                    // otherwise wait out the whole scan before starting its
                    // own. Dropping this attempt makes the store yield, keeping
                    // what it verified, and the clean-up's pass resumes that.
                    let sooner = request.grace < grace;
                    waiting.push(request);
                    if sooner {
                        break None;
                    }
                }
            }
        };
        let Some(collected) = collected else {
            delay = Duration::ZERO;
            continue;
        };
        // A clean-up that joined during this attempt is answered by it too.
        let requested = !waiting.is_empty();
        match collected {
            Ok(report) => {
                log_pass(&report, started, requested);
                if requested {
                    freed.objects = freed
                        .objects
                        .saturating_add(u64::try_from(report.deleted).unwrap_or(u64::MAX));
                    freed.bytes = freed.bytes.saturating_add(report.deleted_bytes);
                }
                if report.deferred {
                    delay = if waiting.is_empty() {
                        IDLE_RETRY
                    } else {
                        CLEAN_UP_RETRY
                    };
                } else {
                    answer(&mut waiting, &Ok(freed));
                    freed = Freed::default();
                    delay = INTERVAL;
                }
            }
            Err(error) => {
                tracing::warn!(
                    operation = "gc",
                    latency_ms = latency_millis(started),
                    result = "error",
                    requested,
                    "artifact garbage collection aborted"
                );
                answer(&mut waiting, &Err(CleanUpError::Refused(error.to_string())));
                freed = Freed::default();
                delay = INTERVAL;
            }
        }
    }
}

fn answer(waiting: &mut Vec<CleanUp>, outcome: &Result<Freed, CleanUpError>) {
    for request in waiting.drain(..) {
        // Nobody listening any more is fine: the space is freed either way.
        let _unheard = request.reply.send(outcome.clone());
    }
}

fn log_pass(report: &GcReport, started: Instant, requested: bool) {
    tracing::info!(
        operation = "gc",
        latency_ms = latency_millis(started),
        result = "ok",
        requested,
        reachable = report.reachable,
        grace_preserved = report.preserved_by_grace,
        deleted = report.deleted,
        deleted_bytes = report.deleted_bytes,
        quarantine_deleted = report.quarantine_deleted,
        deferred = report.deferred,
        "artifact garbage collection pass"
    );
}

fn latency_millis(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}
