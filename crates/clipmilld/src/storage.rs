//! How much disk this installation is using, and where the answer comes from.
//!
//! Three categories, because the three answers a user acts on are different
//! actions. Artifacts are re-derivable and can be collected. Model weights are
//! expensive to fetch and should not be. State is small and must not be touched.
//! One total would tell nobody what to do about it.
//!
//! Only one of the three is cheap to know. The artifact store already holds
//! every manifest in memory and can sum declared sizes without touching disk;
//! the other two are directory trees that have to be walked. They are small —
//! a handful of weight files, a database and its backups — so the walk is
//! bounded, but it is still a walk, which is why this runs off the async runtime
//! and not on it.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clipmill_artifacts::StoreUsage;

/// Stable identifiers a caller keys off. The wording on screen is its own.
pub(crate) const ARTIFACTS: &str = "artifacts";
pub(crate) const MODELS: &str = "models";
pub(crate) const STATE: &str = "state";

/// The directories a storage report covers.
///
/// Held apart from `Config` so the report depends on three paths rather than on
/// everything the daemon was configured with, and so a test can point it at a
/// temporary tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StorageDirs {
    /// Whose free space is reported. The artifact store lives under it.
    pub data: PathBuf,
    /// The content-addressed store itself, which is the directory a user is
    /// pointed at when the artifacts figure is the large one.
    pub artifacts: PathBuf,
    pub state: PathBuf,
    /// Downloaded model weights. Often outside the data directory, and often
    /// absent entirely on a fresh install.
    pub weights: PathBuf,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Category {
    pub bytes: u64,
    pub items: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Report {
    pub artifacts: Category,
    pub models: Category,
    pub state: Category,
    /// Free space on the volume holding the data directory, when the filesystem
    /// would say. `None` and zero are different answers and must not be
    /// collapsed: one means "could not be read", the other means "full".
    pub available_bytes: Option<u64>,
    /// Where each category lives. Carried with the sizes because a size a user
    /// cannot go and look at is a number they can do nothing about.
    pub paths: ReportPaths,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ReportPaths {
    pub artifacts: PathBuf,
    pub models: PathBuf,
    pub state: PathBuf,
}

impl StorageDirs {
    /// Measure everything but the artifacts, which the store answers for.
    ///
    /// Blocking: two directory walks and one filesystem query. Call it off the
    /// runtime.
    pub(crate) fn measure(&self, artifacts: StoreUsage) -> Report {
        Report {
            artifacts: Category {
                bytes: artifacts.bytes,
                items: artifacts.objects,
            },
            models: walk(&self.weights),
            state: walk(&self.state),
            available_bytes: fs2::available_space(&self.data).ok(),
            paths: ReportPaths {
                artifacts: self.artifacts.clone(),
                models: self.weights.clone(),
                state: self.state.clone(),
            },
        }
    }
}

/// Every regular file under a directory, summed.
///
/// A directory that does not exist reports zero rather than failing: a fresh
/// install has downloaded no weights, and "nothing there" is the correct answer
/// rather than an error a screen would have to explain. Symlinks are counted by
/// their own size and not followed, so a link into the artifact store cannot
/// make state look enormous — or, worse, send the walk in a circle.
fn walk(root: &Path) -> Category {
    let mut total = Category::default();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                pending.push(entry.path());
            } else if let Ok(metadata) = entry.metadata() {
                total.bytes += metadata.len();
                total.items += 1;
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::fs;

    use clipmill_artifacts::StoreUsage;
    use tempfile::tempdir;

    use super::{Category, StorageDirs, walk};

    #[test]
    fn a_walk_counts_every_file_at_every_depth() {
        let root = tempdir().expect("temp");
        fs::write(root.path().join("top.bin"), [0_u8; 10]).expect("write");
        fs::create_dir_all(root.path().join("a/b")).expect("dirs");
        fs::write(root.path().join("a/one.bin"), [0_u8; 20]).expect("write");
        fs::write(root.path().join("a/b/two.bin"), [0_u8; 30]).expect("write");

        assert_eq!(
            walk(root.path()),
            Category {
                bytes: 60,
                items: 3
            }
        );
    }

    /// A fresh install has downloaded no weights. That is an answer, not a
    /// failure, and a screen should not have to explain it.
    #[test]
    fn a_directory_that_is_not_there_reports_nothing() {
        let root = tempdir().expect("temp");
        assert_eq!(
            walk(&root.path().join("never-created")),
            Category::default()
        );
    }

    /// The store answers for artifacts; nothing here re-derives that figure by
    /// walking the objects, which is the whole reason the store keeps it.
    #[test]
    fn the_store_answers_for_artifacts_and_the_walk_answers_for_the_rest() {
        let root = tempdir().expect("temp");
        let state = root.path().join("state");
        fs::create_dir_all(&state).expect("dirs");
        fs::write(state.join("clipmill.db"), [0_u8; 64]).expect("write");

        let dirs = StorageDirs {
            artifacts: root.path().join("artifacts"),
            data: root.path().to_path_buf(),
            state,
            weights: root.path().join("weights"),
        };
        let report = dirs.measure(StoreUsage {
            objects: 7,
            bytes: 4096,
        });

        assert_eq!(
            report.artifacts,
            Category {
                bytes: 4096,
                items: 7
            }
        );
        assert_eq!(
            report.state,
            Category {
                bytes: 64,
                items: 1
            }
        );
        assert_eq!(report.models, Category::default());
        // A temporary directory sits on a real filesystem, so this is readable.
        assert!(report.available_bytes.is_some());
    }
}
