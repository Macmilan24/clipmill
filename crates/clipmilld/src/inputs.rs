//! Finding the documents a derivation stage was told to read.
//!
//! The four model-free stages that read published artifacts — the evidence
//! index, discovery, ranking, and the analysis fan-in — reach their inputs by
//! two routes that arrive at one place. Submitted as a standalone job, what they
//! read was published by *earlier jobs*, so the plan declares its addresses.
//! Run inside the analyze DAG, those same documents are the outputs of tasks in
//! the same plan, so a dependency carries them. Either way the daemon delivers
//! one list on the lease, and this module reads it.
//!
//! That the two routes converge before this point is the property that matters.
//! A stage that resolved a payload address on one route and a dependency on the
//! other would compute two keys for one piece of work — and the artifact key
//! covers the input list, so those two keys would be two addresses for one
//! observation. Nothing in a content-addressed store can notice that afterwards.
//!
//! Inputs are matched to what a stage asked for by the artifact kind each one's
//! own manifest declares, never by position. A plan that declared a transcript
//! where an index belongs, or a dependency list reordered upstream, is refused
//! rather than parsed into something plausible.
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

/// One document a stage needs, named by the artifact kind it must be.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Wanted {
    pub kind: &'static str,
    /// A missing optional input is an absence the stage reports rather than a
    /// failure — a source with no video has no shot cuts, and that is a
    /// different document from one whose shot detection was never run.
    pub required: bool,
}

impl Wanted {
    pub(crate) fn required(kind: &'static str) -> Self {
        Self {
            kind,
            required: true,
        }
    }

    pub(crate) fn optional(kind: &'static str) -> Self {
        Self {
            kind,
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
    wanted: &[Wanted],
) -> Result<Resolved, TaskExecutionError> {
    // Everything the lease delivered, indexed by the kind each one declares.
    // Opened once: a stage with three inputs would otherwise open the same
    // artifact three times to ask what it is.
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
        match by_kind.remove(entry.kind) {
            Some(pair) => {
                found.insert(entry.kind, pair);
            }
            None if entry.required => {
                return Err(TaskExecutionError::deterministic(format!(
                    "this task has no {} to read",
                    entry.kind
                )));
            }
            None => {}
        }
    }
    // An input nobody asked for is a plan that believes this stage reads
    // something it does not. Refused rather than ignored: the artifact key
    // covers only what was read, so a stage silently handed a fourth document
    // would key as though it had three.
    if let Some((kind, _)) = by_kind.into_iter().next() {
        return Err(TaskExecutionError::deterministic(format!(
            "this task was given a {kind} it does not read"
        )));
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
