//! Finding the documents a derivation stage was told to read.
//!
//! The three model-free stages that read published artifacts — the evidence
//! index, discovery, and ranking — all face one problem twice. Submitted as a
//! standalone job, their inputs were published by *earlier jobs* and arrive as
//! content addresses in the task payload, because a task's input artifacts are
//! the outputs of the tasks it depends on and a standalone job depends on
//! nothing. Run inside the analyze DAG, those same documents *are* the outputs
//! of tasks in the same plan, and arrive that way instead.
//!
//! Both routes are real and neither is a fallback. What matters is that they
//! agree: the artifact key is computed from the addresses whichever way they
//! were found, so the same documents processed through different routes hit
//! the same cache entry rather than producing two copies of one answer.
//!
//! Whichever route an input came by, it is checked against the artifact kind
//! its own manifest declares. A payload naming a transcript where an index
//! belongs, or a dependency list reordered upstream, is refused rather than
//! parsed into something plausible.
//!
//! Everything here needs a live artifact store, so it is exercised end to end
//! by `gate-evidence`, `gate-discovery`, and `gate-ranking` against a real
//! daemon rather than by unit tests against a mock that would only prove the
//! mock agrees with itself.

use std::collections::BTreeMap;

use clipmill_artifacts::ArtifactLease;
use clipmill_core::ArtifactId;

use crate::{
    artifacts::ArtifactHandle,
    jobs::{LeasedTask, TaskExecutionError},
    media,
};

/// One document a stage needs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Wanted<'a> {
    /// The artifact kind, which is what the lookup matches on.
    pub kind: &'static str,
    /// The address the payload named, or empty to take it from a dependency.
    pub address: &'a str,
    /// A missing optional input is an absence the stage reports rather than a
    /// failure — a source with no video has no shot cuts, and that is a
    /// different document from one whose shot detection was never run.
    pub required: bool,
}

impl<'a> Wanted<'a> {
    pub(crate) fn required(kind: &'static str, address: &'a str) -> Self {
        Self {
            kind,
            address,
            required: true,
        }
    }

    pub(crate) fn optional(kind: &'static str, address: &'a str) -> Self {
        Self {
            kind,
            address,
            required: false,
        }
    }
}

/// The documents a stage may read, opened and verified.
pub(crate) struct Resolved {
    /// In the order the caller asked for them, so the artifact key does not
    /// depend on a map's iteration order.
    order: Vec<&'static str>,
    found: BTreeMap<&'static str, (ArtifactId, ArtifactLease)>,
}

/// Open everything the stage was told to read.
pub(crate) async fn resolve(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
    wanted: &[Wanted<'_>],
) -> Result<Resolved, TaskExecutionError> {
    // The dependency outputs, indexed by the kind each one declares. Opened
    // once: a stage with three inputs would otherwise open the same artifact
    // three times to ask what it is.
    let mut by_kind: BTreeMap<String, (ArtifactId, ArtifactLease)> = BTreeMap::new();
    for artifact_id in &task.input_artifact_ids {
        let lease = artifacts
            .open(*artifact_id)
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        let kind = lease.kind().to_owned();
        if by_kind
            .insert(kind.clone(), (*artifact_id, lease))
            .is_some()
        {
            // Two inputs of one kind leave the stage no basis for choosing,
            // and picking either would be picking by position.
            return Err(TaskExecutionError::deterministic(format!(
                "this task was given two {kind} inputs and names no choice"
            )));
        }
    }

    let mut found = BTreeMap::new();
    let mut order = Vec::new();
    for entry in wanted {
        order.push(entry.kind);
        if entry.address.is_empty() {
            // The DAG route: whichever dependency published this kind.
            if let Some(pair) = by_kind.remove(entry.kind) {
                found.insert(entry.kind, pair);
            } else if entry.required {
                return Err(TaskExecutionError::deterministic(format!(
                    "this task has no {} to read",
                    entry.kind
                )));
            }
            continue;
        }
        // The standalone route: the address the payload named.
        let artifact_id: ArtifactId = entry.address.parse().map_err(|_| {
            TaskExecutionError::deterministic("input artifact id is not an address")
        })?;
        let lease = artifacts
            .open(artifact_id)
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        if lease.kind() != entry.kind {
            return Err(TaskExecutionError::deterministic(format!(
                "this task was pointed at a {}, not a {}",
                lease.kind(),
                entry.kind
            )));
        }
        found.insert(entry.kind, (artifact_id, lease));
    }
    Ok(Resolved { order, found })
}

impl Resolved {
    /// Read one document out of the artifact that carries it.
    pub(crate) fn read<T: serde::de::DeserializeOwned>(
        &self,
        kind: &'static str,
        file: &str,
    ) -> Result<T, TaskExecutionError> {
        let (_, lease) = self.found.get(kind).ok_or_else(|| {
            TaskExecutionError::deterministic(format!("this task has no {kind} to read"))
        })?;
        media::read_artifact_document(lease, file)
    }

    /// The same, for an input that may legitimately be absent.
    pub(crate) fn read_optional<T: serde::de::DeserializeOwned>(
        &self,
        kind: &'static str,
        file: &str,
    ) -> Result<Option<T>, TaskExecutionError> {
        match self.found.get(kind) {
            None => Ok(None),
            Some((_, lease)) => media::read_artifact_document(lease, file).map(Some),
        }
    }

    /// The address of one input, for the document that records what it read.
    pub(crate) fn address(&self, kind: &'static str) -> Result<String, TaskExecutionError> {
        self.found
            .get(kind)
            .map(|(artifact_id, _)| artifact_id.to_string())
            .ok_or_else(|| {
                TaskExecutionError::deterministic(format!("this task has no {kind} to read"))
            })
    }

    /// The address of an input that may be absent.
    pub(crate) fn optional_address(&self, kind: &'static str) -> Option<String> {
        self.found
            .get(kind)
            .map(|(artifact_id, _)| artifact_id.to_string())
    }

    /// Every address this stage read, in the order they were asked for.
    ///
    /// This is what the artifact key covers, so a stage that read three
    /// documents cannot be served a cache entry computed from two.
    pub(crate) fn addresses(&self) -> Vec<ArtifactId> {
        self.order
            .iter()
            .filter_map(|kind| self.found.get(kind).map(|(artifact_id, _)| *artifact_id))
            .collect()
    }
}
