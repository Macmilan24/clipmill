//! How much disk this installation is using, where it is, and freeing it.
//!
//! Artifacts are re-derivable; model weights are expensive to fetch; state is
//! durable; imported originals are retained until their project is deleted;
//! database backups are kept from before each migration; temporary files are
//! scratch that stopped work and interrupted downloads left behind. The
//! artifact catalogue supplies its own totals. Other categories are counted by
//! directory walks off the async runtime, without reading file contents.
//!
//! Clean-up here is limited to what nobody is using: scratch that has not
//! changed for an hour, interrupted downloads nobody is resuming, and every
//! database backup but the newest. Generated media is the artifact store's to
//! collect, and model weights the library's to remove.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use clipmill_artifacts::StoreUsage;

/// Stable identifiers a caller keys off. The wording on screen is its own.
pub(crate) const ARTIFACTS: &str = "artifacts";
pub(crate) const MODELS: &str = "models";
pub(crate) const STATE: &str = "state";
pub(crate) const IMPORTS: &str = "imports";
pub(crate) const BACKUPS: &str = "backups";
pub(crate) const TEMPORARY: &str = "temporary";

/// Scratch younger than this may belong to work still running: a long ingest
/// or render keeps writing into its private directory for as long as it runs.
pub(crate) const SCRATCH_QUIET: Duration = Duration::from_hours(1);
/// Where the model library stages a download, inside the weights directory.
const PARTIAL: &str = ".partial";

/// The directories a storage report covers.
///
/// Held apart from `Config` so the report depends on these paths rather than
/// on everything the daemon was configured with, and so a test can point it
/// at a temporary tree.
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
    /// Database backups taken before migrations, inside `state`.
    pub backups: PathBuf,
    /// Private scratch directories stages write into, inside `state`.
    pub scratch: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Category {
    pub bytes: u64,
    pub items: u64,
}

impl Category {
    fn add(&mut self, other: Self) {
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.items = self.items.saturating_add(other.items);
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Report {
    pub artifacts: Category,
    pub models: Category,
    pub state: Category,
    pub imports: Category,
    pub backups: Category,
    pub temporary: Category,
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
    pub imports: PathBuf,
    pub artifacts: PathBuf,
    pub models: PathBuf,
    pub state: PathBuf,
    pub backups: PathBuf,
    pub temporary: PathBuf,
}

impl StorageDirs {
    /// Measure everything but the artifacts, which the store answers for.
    ///
    /// Blocking: a handful of directory walks and one filesystem query. Call
    /// it off the runtime.
    pub(crate) fn measure(&self, artifacts: StoreUsage) -> Report {
        let partial = self.weights.join(PARTIAL);
        let mut excluded = vec![self.backups.clone()];
        excluded.extend(self.scratch.iter().cloned());
        let mut temporary = walk(&partial);
        for directory in &self.scratch {
            temporary.add(walk(directory));
        }
        Report {
            artifacts: Category {
                bytes: artifacts.bytes,
                items: artifacts.objects,
            },
            models: walk_excluding(&self.weights, &[partial]),
            state: walk_excluding(&self.state, &excluded),
            imports: walk(&self.data.join("imports")),
            backups: walk(&self.backups),
            temporary,
            available_bytes: fs2::available_space(&self.data).ok(),
            paths: ReportPaths {
                imports: self.data.join("imports"),
                artifacts: self.artifacts.clone(),
                models: self.weights.clone(),
                state: self.state.clone(),
                backups: self.backups.clone(),
                temporary: self.state.clone(),
            },
        }
    }

    /// Remove scratch nothing has written to for `quiet`, and every
    /// interrupted download except those of the models in `downloading`.
    ///
    /// Blocking. An entry is judged by the newest file anywhere inside it, so
    /// a render still writing into a directory created hours ago is kept.
    pub(crate) fn clean_temporary(
        &self,
        now: SystemTime,
        quiet: Duration,
        downloading: &[String],
    ) -> Category {
        let mut freed = Category::default();
        for directory in &self.scratch {
            for entry in entries(directory) {
                let newest = newest_modification(&entry);
                let settled = newest
                    .and_then(|time| now.duration_since(time).ok())
                    .is_some_and(|age| age >= quiet);
                if settled {
                    freed.add(remove(&entry));
                }
            }
        }
        for entry in entries(&self.weights.join(PARTIAL)) {
            let model = entry
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            if !downloading.contains(&model) {
                freed.add(remove(&entry));
            }
        }
        freed
    }

    /// Keep the newest `keep` database backups and remove the rest.
    ///
    /// Backups are taken before each schema migration so a failed one can be
    /// undone by hand; the newest is the one that could still matter.
    pub(crate) fn clean_backups(&self, keep: usize) -> Category {
        let mut backups = entries(&self.backups)
            .into_iter()
            .filter(|path| fs::symlink_metadata(path).is_ok_and(|meta| meta.is_file()))
            .map(|path| {
                let modified = fs::metadata(&path)
                    .and_then(|meta| meta.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                (modified, path)
            })
            .collect::<Vec<_>>();
        // Newest first; a tie falls back to the name, which carries the time.
        backups.sort_by(|left, right| right.0.cmp(&left.0).then(right.1.cmp(&left.1)));
        let mut freed = Category::default();
        for (_, path) in backups.into_iter().skip(keep) {
            freed.add(remove(&path));
        }
        freed
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
    walk_excluding(root, &[])
}

/// The same walk, skipping whole subtrees another category answers for.
fn walk_excluding(root: &Path, excluded: &[PathBuf]) -> Category {
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
            let path = entry.path();
            if excluded.contains(&path) {
                continue;
            }
            if kind.is_dir() {
                pending.push(path);
            } else if let Ok(metadata) = entry.metadata() {
                total.bytes += metadata.len();
                total.items += 1;
            }
        }
    }
    total
}

fn entries(directory: &Path) -> Vec<PathBuf> {
    fs::read_dir(directory)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default()
}

/// The newest modification anywhere in a tree, links judged as themselves.
fn newest_modification(root: &Path) -> Option<SystemTime> {
    let mut newest = fs::symlink_metadata(root)
        .and_then(|meta| meta.modified())
        .ok();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = fs::symlink_metadata(entry.path()) else {
                continue;
            };
            if let Ok(modified) = metadata.modified() {
                newest = Some(newest.map_or(modified, |current| current.max(modified)));
            }
            if metadata.is_dir() {
                pending.push(entry.path());
            }
        }
    }
    newest
}

/// Remove one entry — a file, a link, or a whole directory — and say what it
/// held. A link is removed as a link; whatever it points at is not ours.
fn remove(path: &Path) -> Category {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Category::default();
    };
    let measured = if metadata.is_dir() {
        walk(path)
    } else {
        Category {
            bytes: metadata.len(),
            items: 1,
        }
    };
    let removed = if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    match removed {
        Ok(()) => measured,
        Err(error) => {
            tracing::warn!(%error, "a temporary entry could not be removed");
            Category::default()
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::{
        fs,
        path::Path,
        time::{Duration, SystemTime},
    };

    use clipmill_artifacts::StoreUsage;
    use tempfile::tempdir;

    use super::{Category, StorageDirs, walk};

    fn dirs(root: &Path) -> StorageDirs {
        let state = root.join("state");
        StorageDirs {
            artifacts: root.join("artifacts"),
            data: root.to_path_buf(),
            backups: state.join("backups"),
            scratch: vec![state.join("media-scratch"), state.join("probe-scratch")],
            state,
            weights: root.join("weights"),
        }
    }

    fn write(path: &Path, bytes: usize) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![0_u8; bytes]).unwrap();
    }

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
    /// walking the objects, which is the whole reason the store keeps it. Each
    /// byte is counted once: backups and scratch are not also state, and an
    /// interrupted download is temporary rather than a model.
    #[test]
    fn every_byte_lands_in_exactly_one_category() {
        let root = tempdir().expect("temp");
        let dirs = dirs(root.path());
        write(&dirs.state.join("clipmill.db"), 64);
        write(&dirs.backups.join("clipmill-v1-to-v2.db"), 300);
        write(&dirs.scratch[0].join("task/frame.raw"), 50);
        write(&root.path().join("imports/project/attempt/source.mkv"), 128);
        write(&dirs.weights.join("whisper-base/ggml-base.bin"), 1000);
        write(
            &dirs.weights.join(".partial/qwen/model.safetensors.part"),
            700,
        );

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
        assert_eq!(
            report.backups,
            Category {
                bytes: 300,
                items: 1
            }
        );
        assert_eq!(
            report.temporary,
            Category {
                bytes: 750,
                items: 2
            }
        );
        assert_eq!(
            report.models,
            Category {
                bytes: 1000,
                items: 1
            }
        );
        assert_eq!(
            report.imports,
            Category {
                bytes: 128,
                items: 1
            }
        );
        assert_eq!(report.paths.imports, root.path().join("imports"));
        // A temporary directory sits on a real filesystem, so this is readable.
        assert!(report.available_bytes.is_some());
    }

    /// Scratch is removed only once nothing inside it has changed for the
    /// quiet period, and a download somebody is running keeps its bytes.
    #[test]
    fn temporary_clean_up_spares_live_work_and_running_downloads() {
        let root = tempdir().expect("temp");
        let dirs = dirs(root.path());
        write(&dirs.scratch[0].join("old-task/frame.raw"), 40);
        write(&dirs.scratch[1].join("probe.json"), 10);
        write(&dirs.weights.join(".partial/stale/model.part"), 30);
        write(&dirs.weights.join(".partial/running/model.part"), 20);

        let now = SystemTime::now();
        let nothing = dirs.clean_temporary(now, Duration::from_hours(1), &["running".to_owned()]);
        assert_eq!(
            nothing,
            Category {
                bytes: 30,
                items: 1
            },
            "only the stale download"
        );
        assert!(dirs.scratch[0].join("old-task/frame.raw").exists());
        assert!(dirs.weights.join(".partial/running/model.part").exists());

        let later = now + Duration::from_hours(2);
        let settled = dirs.clean_temporary(later, Duration::from_hours(1), &["running".to_owned()]);
        assert_eq!(
            settled,
            Category {
                bytes: 50,
                items: 2
            }
        );
        assert!(!dirs.scratch[0].join("old-task").exists());
        assert!(
            dirs.scratch[0].exists(),
            "the scratch directory itself stays"
        );
        assert!(dirs.weights.join(".partial/running/model.part").exists());
    }

    #[test]
    fn backup_clean_up_keeps_the_newest() {
        let root = tempdir().expect("temp");
        let dirs = dirs(root.path());
        for (index, name) in [
            "clipmill-v8-to-v9.db",
            "clipmill-v9-to-v11.db",
            "clipmill-v11-to-v12.db",
        ]
        .iter()
        .enumerate()
        {
            let path = dirs.backups.join(name);
            write(&path, 100);
            let file = fs::File::options().write(true).open(&path).unwrap();
            file.set_modified(
                SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000 + index as u64 * 1_000),
            )
            .unwrap();
        }

        let freed = dirs.clean_backups(1);

        assert_eq!(
            freed,
            Category {
                bytes: 200,
                items: 2
            }
        );
        assert!(dirs.backups.join("clipmill-v11-to-v12.db").exists());
        assert!(!dirs.backups.join("clipmill-v8-to-v9.db").exists());
    }
}
