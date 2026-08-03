//! The archive index: what is in the zip, and what the zip deliberately leaves
//! outside it.
//!
//! The book's rule is that the user's work is not held hostage to the
//! application that made it (ch. 10), which means the archive has to be
//! readable by somebody who does not have ClipMill — so its contents are
//! described by a published schema rather than by this code.
//!
//! Media is referenced, not copied. A project's sources are the largest thing
//! about it by three orders of magnitude and they already exist on the user's
//! disk; an archive that copied them would be an archive nobody makes twice.
//! What travels is everything that cannot be regenerated: the project state,
//! the edit documents, the command logs that explain how each document reached
//! its shape, and the render manifests that say what was delivered. Every
//! source is still *named*, with the fingerprint that identifies it, so a
//! re-import can tell you exactly which file it is looking for rather than
//! failing with a path that stopped being true.

use serde::{Deserialize, Serialize};

pub const ARCHIVE_SCHEMA_VERSION: &str = "clipmill.archive_index.v1";
/// Where the index lives inside the archive. Fixed, so a reader finds it
/// without searching.
pub const ARCHIVE_INDEX_FILE: &str = "archive-index.json";

/// What one entry in the archive is.
///
/// A closed set rather than a free string: a reader deciding what to do with a
/// file needs to know what it is, and "some JSON" is not an answer. An unknown
/// kind in a future archive is a reason for that reader to stop, which it can
/// only do if the vocabulary is stated.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    /// The project row, its jobs, and its sources, as the daemon holds them.
    State,
    /// One `edit_ir.v1` document, as it stands.
    EditDoc,
    /// The commands applied to one document, in order.
    CommandLog,
    /// One `render.clip.v1` manifest from a render that happened.
    RenderManifest,
    /// The clip decisions a user recorded on a candidate set.
    Decisions,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArchiveEntry {
    /// Path inside the archive, forward-slashed and relative.
    pub path: String,
    pub kind: EntryKind,
    pub sha256: String,
    pub bytes: u64,
}

/// A source the archive names but does not carry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArchivedSource {
    pub source_id: String,
    /// The content fingerprint, which is what identifies the file if it moved.
    pub fingerprint: String,
    /// What the user called it, for a human looking for it.
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArchiveIndex {
    pub schema: String,
    pub project_id: String,
    pub project_name: String,
    /// When the archive was made. Wall time is allowed here because an archive
    /// is project state rather than a derived artifact — and a person opening
    /// one in a year needs to know which of two it is.
    pub created_unix_millis: u64,
    /// The application version that wrote it, so a reader can tell whether it
    /// is older than the format it is reading.
    pub writer_version: String,
    /// Named, not carried. See the module note.
    pub sources: Vec<ArchivedSource>,
    pub entries: Vec<ArchiveEntry>,
}

impl ArchiveIndex {
    /// The index for a project, with its entries sorted so two archives of the
    /// same project agree on order.
    pub fn new(
        project_id: String,
        project_name: String,
        created_unix_millis: u64,
        writer_version: String,
        sources: Vec<ArchivedSource>,
        mut entries: Vec<ArchiveEntry>,
    ) -> Self {
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Self {
            schema: ARCHIVE_SCHEMA_VERSION.to_owned(),
            project_id,
            project_name,
            created_unix_millis,
            writer_version,
            sources,
            entries,
        }
    }

    /// Whether this index describes a format this build understands.
    pub fn is_readable(&self) -> bool {
        self.schema == ARCHIVE_SCHEMA_VERSION
    }

    /// The entry for a path, for a reader verifying what it extracted.
    pub fn entry(&self, path: &str) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|entry| entry.path == path)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{ARCHIVE_SCHEMA_VERSION, ArchiveEntry, ArchiveIndex, EntryKind};

    fn entry(path: &str) -> ArchiveEntry {
        ArchiveEntry {
            path: path.to_owned(),
            kind: EntryKind::EditDoc,
            sha256: "0".repeat(64),
            bytes: 12,
        }
    }

    fn index(entries: Vec<ArchiveEntry>) -> ArchiveIndex {
        ArchiveIndex::new(
            "prj_1".to_owned(),
            "Pricing Talk".to_owned(),
            1_700_000_000_000,
            "0.0.1".to_owned(),
            Vec::new(),
            entries,
        )
    }

    #[test]
    fn entries_come_out_in_one_order_whatever_order_they_went_in() {
        let forwards = index(vec![entry("a.json"), entry("b.json")]);
        let backwards = index(vec![entry("b.json"), entry("a.json")]);
        assert_eq!(forwards.entries, backwards.entries);
    }

    #[test]
    fn an_index_names_the_format_it_is() {
        assert_eq!(index(Vec::new()).schema, ARCHIVE_SCHEMA_VERSION);
        assert!(index(Vec::new()).is_readable());
    }

    #[test]
    fn an_archive_from_a_format_this_build_does_not_know_is_not_readable() {
        let mut future = index(Vec::new());
        future.schema = "clipmill.archive_index.v9".to_owned();
        assert!(!future.is_readable());
    }

    #[test]
    fn it_round_trips_through_json_with_the_kinds_spelled_out() {
        let original = index(vec![entry("docs/a.json")]);
        let text = serde_json::to_string(&original).expect("serialises");
        assert!(text.contains("\"edit_doc\""), "{text}");
        let back: ArchiveIndex = serde_json::from_str(&text).expect("deserialises");
        assert_eq!(back, original);
        assert_eq!(back.entry("docs/a.json").map(|found| found.bytes), Some(12));
    }
}
