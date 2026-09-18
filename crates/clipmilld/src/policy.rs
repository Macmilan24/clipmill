//! Local Lock reports task-policy activity in this daemon session.
//!
//! The registry declares available local and optional cloud capabilities.
//! Registration grants no cloud run permission: explicit analysis consent,
//! registry-matching task policy and a separately enabled cloud worker are
//! required. `engaged` means no network-allowed task has started in this daemon
//! session; it does not mean cloud recipes are absent or the OS denied sockets.
//!
//! The legacy IPC field `egress_attempts` counts network-allowed task starts,
//! including cache hits, not requests or bytes. It stays nonzero until restart.
//! See `docs/local-lock.md` for the independent offline namespace proof and the
//! desktop enforcement limits.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::recipes;

/// What the Settings screen shows, and what Health answers with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalLockStatus {
    pub engaged: bool,
    pub stages: u32,
    pub network_allowed_stages: u32,
    pub egress_attempts: u64,
}

/// The token a task's resource declaration carries when it may not reach the
/// network. The scheduler writes it; this module only reads it.
pub(crate) const LOCAL_LOCK: &str = "local-lock";

#[derive(Debug, Default)]
pub(crate) struct LocalLockPolicy {
    egress_attempts: AtomicU64,
}

impl LocalLockPolicy {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Offer a starting task's declared network policy to the counter.
    ///
    /// Called on the path that starts work rather than on the path that plans
    /// it: a plan that was written and never run has reached nothing, and
    /// counting it would make the number mean something other than what the
    /// screen says it means.
    pub(crate) fn note_task_start(&self, declared: &str) {
        if declared != LOCAL_LOCK {
            self.egress_attempts.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(crate) fn status(&self) -> LocalLockStatus {
        let (stages, network_allowed_stages) = recipes::network_census();
        let egress_attempts = self.egress_attempts.load(Ordering::Relaxed);
        LocalLockStatus {
            // Session status: available cloud adapters do not imply they ran.
            // Once cloud work starts, remain disengaged for this daemon session.
            engaged: egress_attempts == 0,
            stages,
            network_allowed_stages,
            egress_attempts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LOCAL_LOCK, LocalLockPolicy};

    #[test]
    fn a_fresh_daemon_is_locked_and_says_how_many_stages_it_checked() {
        let policy = LocalLockPolicy::new();
        let status = policy.status();
        assert!(status.engaged);
        assert_eq!(
            status.network_allowed_stages,
            crate::recipes::network_census().1
        );
        assert_eq!(status.egress_attempts, 0);
        // The count is the registry's, so it is never zero — a lock over
        // nothing is not a lock.
        assert!(status.stages > 0);
    }

    #[test]
    fn a_task_under_the_lock_is_not_an_egress_attempt() {
        let policy = LocalLockPolicy::new();
        for _ in 0..10 {
            policy.note_task_start(LOCAL_LOCK);
        }
        assert_eq!(policy.status().egress_attempts, 0);
        assert!(policy.status().engaged);
    }

    #[test]
    fn a_task_declaring_anything_else_is_counted_and_breaks_the_claim() {
        let policy = LocalLockPolicy::new();
        policy.note_task_start("network-allowed");
        let status = policy.status();
        assert_eq!(status.egress_attempts, 1);
        assert!(
            !status.engaged,
            "the lock cannot still read engaged after cloud work started"
        );
    }

    #[test]
    fn an_unrecognised_declaration_counts_rather_than_being_assumed_harmless() {
        // A typo in a resource declaration is not evidence of being offline.
        let policy = LocalLockPolicy::new();
        policy.note_task_start("local_lock");
        assert_eq!(policy.status().egress_attempts, 1);
    }
}
