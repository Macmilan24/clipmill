//! Delivery: the task that turns a render into files somebody keeps.
//!
//! Everything about *what* to deliver was decided in `clipmill-export`, which
//! touches nothing. This module is the part that touches the world: it reads
//! the render artifact, resolves the destination, copies the clip and its
//! sidecars under the names the pattern chose, pulls one frame for a thumbnail,
//! and writes the two documents that describe the result.
//!
//! Two rules shape the ordering. **The files are written every time**, cache hit
//! or not — the package artifact is a description of a delivery, and a user who
//! deleted their export and asked again must get it back rather than a cache
//! identity. And **nothing is written into the destination until every byte is
//! ready**: each file lands under a temporary name in the destination directory
//! and is renamed into place, so an interrupted export leaves no half-file with
//! a plausible size.
//!
//! The destination is a local directory or nothing. A half-written export over
//! a link that dropped is a corrupt file that looks finished, and Phase 1 has
//! no way to tell one from the other after the fact — so the filesystem is
//! asked what it is before anything is written.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, Producer, RecipeSpec, StagingArea, Timebase,
};
use clipmill_contracts::proto::ipc::v1::{DeliverExportPayloadV1, ExportRequestV1};
use clipmill_core::{ArtifactId, Sha256Digest};
use clipmill_export::{
    AudioSummary, DeliveredFile, Disclosure, EntryKind, ExportPackage, FileRole, Pattern,
    VideoSummary, checksum_file, digest_of,
};
use clipmill_render::{CLIP_FILE, MANIFEST_FILE, RenderManifest, SRT_FILE, VTT_FILE};
use prost::Message;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    artifacts::ArtifactHandle,
    jobs::{LeasedTask, TaskExecutionError},
    media::{
        FfmpegSpec, MediaError, MediaRunner, Prepared, ProgressSlot, abandon_staging,
        artifact_path, commit_staging, prepare_or_hit, read_descriptor, verified_input_file,
        write_canonical_json,
    },
};

pub(crate) const KIND_DELIVER_EXPORT: &str = "deliver-export";
/// A thumbnail is one JPEG. The budget is generous enough for a frame of a
/// 1080x1920 picture and small enough that a runaway encode is refused.
const THUMBNAIL_BUDGET_BYTES: u64 = 32 * 1024 * 1024;
/// The suffix a file wears while it is still being written. Nothing with this
/// on the end is a delivered file.
const PARTIAL_SUFFIX: &str = ".clipmill-partial";

pub(crate) fn is_export_kind(kind: &str) -> bool {
    kind == KIND_DELIVER_EXPORT
}

pub(crate) struct ExportContext<'a> {
    pub artifacts: &'a ArtifactHandle,
    pub media: &'a MediaRunner,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DestinationError {
    #[error("an export destination must be an absolute path")]
    NotAbsolute,
    #[error("the export destination is not a directory")]
    NotADirectory,
    #[error("the export destination could not be created or read: {0}")]
    Unusable(String),
    #[error(
        "the export destination is on a network filesystem ({0}); Phase 1 writes to local disks \
         only, because a transfer that drops leaves a file that looks finished"
    )]
    NotLocal(String),
}

/// Resolve and check a destination directory before anything is written to it.
///
/// Creating it is part of checking it: a user who typed a folder that does not
/// exist yet meant for one to exist, and finding that out after the render
/// would waste the expensive half of the work.
pub(crate) fn resolve_destination(raw: &str) -> Result<PathBuf, DestinationError> {
    let path = Path::new(raw);
    if !path.is_absolute() || raw.starts_with("\\\\") {
        return Err(DestinationError::NotAbsolute);
    }
    if path.exists() {
        if !path.is_dir() {
            return Err(DestinationError::NotADirectory);
        }
    } else {
        fs::create_dir_all(path).map_err(|error| DestinationError::Unusable(error.to_string()))?;
    }
    if let Some(kind) = network_filesystem(path) {
        return Err(DestinationError::NotLocal(kind));
    }
    Ok(path.to_path_buf())
}

/// The filesystem's own name for itself, when it is one that lives over a wire.
///
/// Asked of the kernel rather than guessed from the path, because a mounted
/// share looks like an ordinary directory and the shape of a path says nothing
/// about where it goes. Returns `None` both for local filesystems and for the
/// case where the question could not be asked — a destination this could not
/// classify is allowed rather than refused, because refusing every unrecognised
/// filesystem would refuse working local disks.
#[cfg(target_os = "macos")]
fn network_filesystem(path: &Path) -> Option<String> {
    /// The names macOS mounts network shares under.
    const REMOTE: [&str; 5] = ["nfs", "smbfs", "afpfs", "webdav", "ftp"];

    let statfs = nix::sys::statfs::statfs(path).ok()?;
    let name = statfs.filesystem_type_name().to_lowercase();
    REMOTE.contains(&name.as_str()).then_some(name)
}

#[cfg(target_os = "linux")]
fn network_filesystem(path: &Path) -> Option<String> {
    use nix::sys::statfs::{CODA_SUPER_MAGIC, FsType, NFS_SUPER_MAGIC, SMB_SUPER_MAGIC, statfs};

    // Linux answers with a magic number rather than a name, so the comparison
    // is against constants. `cifs` is the one that matters in practice — it is
    // how a modern kernel mounts an SMB share, while `SMB_SUPER_MAGIC` is the
    // long-deprecated smbfs — and it is the one nix does not export, so it is
    // written out here. The value is the ASCII "\xFFSMB" the kernel uses.
    const CIFS_MAGIC: FsType = FsType(0xFF53_4D42);

    let measured = statfs(path).ok()?;
    let kind = measured.filesystem_type();
    let name = if kind == NFS_SUPER_MAGIC {
        "nfs"
    } else if kind == SMB_SUPER_MAGIC || kind == CIFS_MAGIC {
        "smb"
    } else if kind == CODA_SUPER_MAGIC {
        "coda"
    } else {
        return None;
    };
    Some(name.to_owned())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn network_filesystem(_path: &Path) -> Option<String> {
    // Windows is a Phase 2 target. When it arrives the answer is
    // GetDriveType/WNetGetUniversalName, and until then this must not claim to
    // have checked.
    None
}

/// Deliver one render.
pub(crate) async fn execute_deliver_task(
    context: &ExportContext<'_>,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    let payload = DeliverExportPayloadV1::decode(task.payload.as_slice())
        .map_err(|_| TaskExecutionError::deterministic("export payload is not decodable"))?;
    let request = payload
        .request
        .ok_or_else(|| TaskExecutionError::deterministic("export payload names no export"))?;

    // Exactly one input: the render this delivers. More than one would mean a
    // plan that changed under us, and delivering the first would be a guess.
    let [render_id] = task.input_artifact_ids.as_slice() else {
        return Err(TaskExecutionError::deterministic(
            "a delivery reads exactly one render",
        ));
    };
    let render_id = *render_id;

    let (lease, _) = verified_input_file(context.artifacts, render_id, MANIFEST_FILE).await?;
    let manifest_value = read_descriptor(&lease, MANIFEST_FILE)?;
    let manifest: RenderManifest = serde_json::from_value(manifest_value)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;

    let destination = resolve_destination(&request.destination_dir)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let stem = stem_for(&request, &manifest, render_id)?;

    let delivered = deliver_files(
        context,
        &Delivery {
            lease: &lease,
            destination: &destination,
            stem: &stem,
            request: &request,
            manifest: &manifest,
            render_id,
        },
        progress,
    )
    .await?;
    drop(lease);

    publish_package(context, task, &request, &manifest, render_id, delivered).await
}

/// The filename stem, resolved by the same code the preview calls.
fn stem_for(
    request: &ExportRequestV1,
    manifest: &RenderManifest,
    render_id: ArtifactId,
) -> Result<String, TaskExecutionError> {
    let pattern = naming_pattern(&request.naming_pattern)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    Ok(pattern.resolve(&clipmill_export::Fields {
        project: String::new(),
        clip: request.title.clone(),
        index: request.index.max(1),
        duration_seconds: seconds_of(manifest.program.duration_ticks),
        date: request.date.clone(),
        address: render_id.to_string(),
    }))
}

/// Parse a pattern, treating an empty one as "no opinion" rather than as an
/// error — a caller with nothing to say gets the default.
pub(crate) fn naming_pattern(raw: &str) -> Result<Pattern, clipmill_export::PatternError> {
    if raw.trim().is_empty() {
        Pattern::parse(Pattern::DEFAULT)
    } else {
        Pattern::parse(raw)
    }
}

fn seconds_of(ticks: i64) -> u64 {
    u64::try_from(ticks.max(0) / 90_000).unwrap_or(0)
}

/// Everything one delivery needs, gathered so the signature stays readable.
struct Delivery<'a> {
    /// The render, held open. Every file is read back through it rather than
    /// off the directory, so each one's digest is re-verified on the way out:
    /// an artifact that rotted since it was written is refused here rather
    /// than delivered.
    lease: &'a clipmill_artifacts::ArtifactLease,
    destination: &'a Path,
    stem: &'a str,
    request: &'a ExportRequestV1,
    manifest: &'a RenderManifest,
    render_id: ArtifactId,
}

/// What the export writes, in delivery order. The clip first because it is the
/// thing; the checksums last because they describe everything before them.
async fn deliver_files(
    context: &ExportContext<'_>,
    delivery: &Delivery<'_>,
    progress: &ProgressSlot,
) -> Result<Vec<DeliveredFile>, TaskExecutionError> {
    let mut delivered = Vec::new();
    for (source, role) in [
        (CLIP_FILE, FileRole::Clip),
        (SRT_FILE, FileRole::SubtitlesSrt),
        (VTT_FILE, FileRole::SubtitlesVtt),
        (MANIFEST_FILE, FileRole::RenderManifest),
    ] {
        let verified = delivery
            .lease
            .verified_path(&artifact_path(source)?)
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        let bytes =
            fs::read(verified).map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        delivered.push(place(delivery, role, &bytes)?);
    }

    let thumbnail = render_thumbnail(context, delivery, progress).await?;
    delivered.push(place(delivery, FileRole::Thumbnail, &thumbnail)?);

    // The package names every file including itself, so its own entry is added
    // after it is written and its digest is the digest of what landed.
    let package = ExportPackage::new(
        delivery.request.doc_id.clone(),
        delivery.request.title.clone(),
        delivery.render_id.to_string(),
        video_of(delivery.manifest),
        audio_of(delivery.manifest),
        disclosure_of(delivery.request, delivery.manifest),
        delivered.clone(),
    );
    let package_bytes = serde_json::to_vec_pretty(&package)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    delivered.push(place(delivery, FileRole::Metadata, &package_bytes)?);

    let checksums = checksum_file(&delivered);
    delivered.push(place(delivery, FileRole::Checksums, checksums.as_bytes())?);
    Ok(delivered)
}

/// Write one file into the destination through a temporary name.
///
/// The rename is what makes an interrupted export leave nothing rather than
/// something: a partial file under the delivered name is indistinguishable from
/// a finished one, and a user would find that out by uploading it.
fn place(
    delivery: &Delivery<'_>,
    role: FileRole,
    bytes: &[u8],
) -> Result<DeliveredFile, TaskExecutionError> {
    let name = format!("{}.{}", delivery.stem, role.extension());
    let final_path = delivery.destination.join(&name);
    let partial = delivery.destination.join(format!("{name}{PARTIAL_SUFFIX}"));
    fs::write(&partial, bytes)
        .and_then(|()| fs::rename(&partial, &final_path))
        .map_err(|error| {
            let _removed = fs::remove_file(&partial);
            TaskExecutionError::transient(error.to_string())
        })?;
    Ok(DeliveredFile {
        name,
        role,
        sha256: digest_of(bytes),
        bytes: bytes.len() as u64,
    })
}

/// One frame, for a poster image.
///
/// Taken from the clip that was delivered rather than from the source, because
/// a thumbnail of footage the viewer will not see is a thumbnail of the wrong
/// picture — the crop, the captions, and the colour are all decided by the
/// render.
async fn render_thumbnail(
    context: &ExportContext<'_>,
    delivery: &Delivery<'_>,
    progress: &ProgressSlot,
) -> Result<Vec<u8>, TaskExecutionError> {
    let work = delivery.destination.to_path_buf();
    let output = format!("{}.thumbnail{PARTIAL_SUFFIX}", delivery.stem);
    // A tenth of the way in, so a clip that opens on a fade does not deliver a
    // black poster, and clamped so a very short clip still lands inside itself.
    let at = (delivery.manifest.program.duration_ticks / 10).clamp(0, 90_000 * 3);
    let args = vec![
        "-nostdin".to_owned(),
        "-hide_banner".to_owned(),
        "-y".to_owned(),
        "-ss".to_owned(),
        format!("{}.{:03}", at / 90_000, (at % 90_000) * 1000 / 90_000),
        "-i".to_owned(),
        delivery
            .lease
            .verified_path(&artifact_path(CLIP_FILE)?)
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?
            .to_string_lossy()
            .into_owned(),
        "-frames:v".to_owned(),
        "1".to_owned(),
        "-q:v".to_owned(),
        "3".to_owned(),
        output.clone(),
    ];
    let _report = context
        .media
        .run_ffmpeg(
            FfmpegSpec {
                args: args.into_iter().map(Into::into).collect(),
                output_dir: work.clone(),
                duration_hint_millis: 1_000,
                max_output_bytes: THUMBNAIL_BUDGET_BYTES,
                capture_stderr: true,
            },
            progress.clone(),
        )
        .await
        .map_err(MediaError::into_task_error)?;
    let path = work.join(&output);
    let bytes =
        fs::read(&path).map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let _removed = fs::remove_file(&path);
    Ok(bytes)
}

fn video_of(manifest: &RenderManifest) -> VideoSummary {
    VideoSummary {
        // The profile states these as i64 and the package as u32, which is
        // wide enough for any picture and narrow enough to be a size. A
        // negative or absurd value would be a corrupt manifest, so it saturates
        // rather than wrapping into a plausible number.
        width: u32::try_from(manifest.profile.width).unwrap_or(0),
        height: u32::try_from(manifest.profile.height).unwrap_or(0),
        frame_rate_num: u32::try_from(manifest.profile.frame_rate.num).unwrap_or(0),
        frame_rate_den: u32::try_from(manifest.profile.frame_rate.den).unwrap_or(0),
        frame_count: manifest.program.frame_count,
        duration_ticks: manifest.program.duration_ticks,
    }
}

fn audio_of(manifest: &RenderManifest) -> AudioSummary {
    AudioSummary {
        target_lufs: manifest.loudness.target_lufs,
        measured_lufs: manifest.loudness.measured_output.integrated_lufs,
        measured_true_peak_dbtp: manifest.loudness.measured_output.true_peak_dbtp,
    }
}

/// The rights claim and the disclosure, taken from the manifest rather than
/// from the request.
///
/// They are the same values — the request is what produced the manifest — but
/// the manifest is what the renderer recorded about what it actually made, and
/// a delivery that restated the request could describe a render that was keyed
/// from something else.
fn disclosure_of(request: &ExportRequestV1, manifest: &RenderManifest) -> Disclosure {
    Disclosure {
        source_attestation: manifest.rights.source_attestation.clone(),
        gates_passed: manifest.rights.gates_passed.clone(),
        ai_assistance: if manifest.ai_use_summary.assistance.is_empty() {
            request.ai_assistance.clone()
        } else {
            manifest.ai_use_summary.assistance.clone()
        },
        requires_ai_disclosure: manifest.ai_use_summary.requires_youtube_ai_disclosure,
    }
}

/// Record what was delivered, as an artifact.
///
/// The files are already on disk by the time this runs. A cache hit here means
/// "this exact delivery has been described before", which is true and costs
/// nothing — it does not mean the files were skipped, because they were not.
async fn publish_package(
    context: &ExportContext<'_>,
    task: &LeasedTask,
    request: &ExportRequestV1,
    manifest: &RenderManifest,
    render_id: ArtifactId,
    delivered: Vec<DeliveredFile>,
) -> Result<ArtifactId, TaskExecutionError> {
    let package = ExportPackage::new(
        request.doc_id.clone(),
        request.title.clone(),
        render_id.to_string(),
        video_of(manifest),
        audio_of(manifest),
        disclosure_of(request, manifest),
        delivered,
    );
    let value = serde_json::to_value(&package)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;

    let mut config = serde_json::Map::new();
    config.insert("destination".to_owned(), json!(request.destination_dir));
    config.insert("naming_pattern".to_owned(), json!(request.naming_pattern));
    config.insert("index".to_owned(), json!(request.index));
    config.insert("date".to_owned(), json!(request.date));
    config.insert("title".to_owned(), json!(request.title));

    let mut hasher = Sha256::new();
    hasher.update(b"clipmill.export.package.v1\0");
    hasher.update(render_id.to_string().as_bytes());
    let source_fingerprint = Sha256Digest::from_bytes(hasher.finalize().into());

    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: task.output_kind.clone(),
        source_fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest: None,
        },
        inputs: vec![render_id],
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.export.package.v1".to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;

    let staging = match prepare_or_hit(context.artifacts, recipe).await? {
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    match write_package(&staging, &value) {
        Ok(paths) => commit_staging(context.artifacts, staging_id, paths).await,
        Err(error) => {
            abandon_staging(context.artifacts, staging_id).await;
            Err(error)
        }
    }
}

fn write_package(
    staging: &StagingArea,
    value: &serde_json::Value,
) -> Result<Vec<ArtifactPath>, TaskExecutionError> {
    let path = artifact_path("export-package.json")?;
    write_canonical_json(staging, &path, value)?;
    Ok(vec![path])
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

    use super::{
        ArchiveInputs, DestinationError, build_archive, naming_pattern, resolve_destination,
        seconds_of,
    };

    #[test]
    fn a_relative_destination_is_refused_before_anything_is_written() {
        assert!(matches!(
            resolve_destination("exports"),
            Err(DestinationError::NotAbsolute)
        ));
    }

    #[test]
    fn a_unc_path_is_refused_even_where_it_is_not_a_path_at_all() {
        // On Unix this is not absolute either, but naming the case keeps the
        // refusal true when Windows arrives.
        assert!(matches!(
            resolve_destination("\\\\server\\share\\clips"),
            Err(DestinationError::NotAbsolute)
        ));
    }

    #[test]
    fn a_destination_that_is_a_file_is_refused_rather_than_written_into() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let file = dir.path().join("not-a-folder");
        std::fs::write(&file, b"x").expect("the file writes");
        assert!(matches!(
            resolve_destination(&file.to_string_lossy()),
            Err(DestinationError::NotADirectory)
        ));
    }

    #[test]
    fn a_destination_that_does_not_exist_yet_is_created() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let wanted = dir.path().join("clips").join("september");
        let resolved = resolve_destination(&wanted.to_string_lossy()).expect("it is usable");
        assert!(resolved.is_dir());
    }

    #[test]
    fn an_empty_pattern_means_the_default_rather_than_an_error() {
        assert_eq!(
            naming_pattern("   ").expect("the default parses").source(),
            clipmill_export::Pattern::DEFAULT
        );
    }

    #[test]
    fn a_pattern_the_user_wrote_is_the_pattern_that_is_used() {
        assert_eq!(
            naming_pattern("{clip}_{date}").expect("parses").source(),
            "{clip}_{date}"
        );
    }

    #[test]
    fn a_pattern_nobody_can_resolve_is_an_error_rather_than_a_silent_default() {
        assert!(naming_pattern("{episode}").is_err());
    }

    /// Enough of a project to archive: one source, one document with a command
    /// applied to it, one decision, one job.
    struct Fixture {
        project: crate::db::ProjectRecord,
        sources: Vec<crate::db::SourceRecord>,
        docs: Vec<crate::db::EditDocRecord>,
        logs: Vec<(String, String, Vec<crate::db::EditCommandRecord>)>,
        decisions: Vec<(String, crate::db::DecisionRecord)>,
    }

    fn inputs() -> Fixture {
        let project = crate::db::ProjectRecord {
            project_id: "prj_1".to_owned(),
            name: "Pricing Talk".to_owned(),
            created_unix_millis: 1_700_000_000_000,
        };
        let sources = vec![crate::db::SourceRecord {
            source_id: "src_1".to_owned(),
            project_id: "prj_1".to_owned(),
            observation: crate::sources::FileObservation {
                absolute_path: "/Users/sami/Movies/episode-14.mov".to_owned(),
                byte_size: 8_000_000_000,
                sample_sha256: "1".repeat(64),
                device_id: 1,
                inode: 2,
                modified_unix_nanos: 3,
            },
            source_fingerprint: format!("sha256:{}", "1".repeat(64)),
            source_map_json: b"{}".to_vec(),
            source_map_artifact_id: "art_map".to_owned(),
            created_unix_millis: 1_700_000_000_000,
        }];
        let document = include_str!("../../clipmill-export/tests/fixtures/short.json");
        let docs = vec![crate::db::EditDocRecord {
            doc_id: "doc_1".to_owned(),
            project_id: "prj_1".to_owned(),
            revision: 2,
            document_json: document.to_owned(),
            created_unix_millis: 1_700_000_000_000,
            updated_unix_millis: 1_700_000_100_000,
        }];
        let logs = vec![(
            "doc_1".to_owned(),
            document.to_owned(),
            vec![crate::db::EditCommandRecord {
                revision: 1,
                command_json: r#"{"op":"set_layout","segment_id":"seg_1","state":"fit"}"#
                    .to_owned(),
                inverse_json: r#"{"op":"set_layout","segment_id":"seg_1","state":"speaker_fill"}"#
                    .to_owned(),
            }],
        )];
        let decisions = vec![(
            "src_1".to_owned(),
            crate::db::DecisionRecord {
                candidate_id: "cand_1".to_owned(),
                decision: crate::db::Decision::Approved,
                decided_unix_millis: 1_700_000_050_000,
            },
        )];
        Fixture {
            project,
            sources,
            docs,
            logs,
            decisions,
        }
    }

    fn archive() -> (Vec<u8>, u32) {
        let fixture = inputs();
        build_archive(&ArchiveInputs {
            project: &fixture.project,
            sources: &fixture.sources,
            docs: &fixture.docs,
            logs: &fixture.logs,
            decisions: &fixture.decisions,
            jobs: &[],
            created_unix_millis: 1_700_000_200_000,
            writer_version: "0.0.1",
        })
        .expect("the archive assembles")
    }

    /// Pull one entry out of a stored zip by walking the local headers.
    ///
    /// Deliberately not the writer's own bookkeeping: reading the bytes back
    /// the way an unrelated tool would is the only way this test can fail when
    /// the writer is wrong about its own output.
    fn extract(bytes: &[u8], wanted: &str) -> Option<Vec<u8>> {
        let mut at = 0_usize;
        while at + 30 <= bytes.len() && &bytes[at..at + 4] == b"PK\x03\x04" {
            let size = u32::from_le_bytes(bytes[at + 18..at + 22].try_into().ok()?) as usize;
            let name_length = u16::from_le_bytes(bytes[at + 26..at + 28].try_into().ok()?) as usize;
            let extra = u16::from_le_bytes(bytes[at + 28..at + 30].try_into().ok()?) as usize;
            let name_at = at + 30;
            let name = std::str::from_utf8(&bytes[name_at..name_at + name_length]).ok()?;
            let data_at = name_at + name_length + extra;
            if name == wanted {
                return Some(bytes[data_at..data_at + size].to_vec());
            }
            at = data_at + size;
        }
        None
    }

    fn index_of(bytes: &[u8]) -> clipmill_export::ArchiveIndex {
        let raw = extract(bytes, clipmill_export::ARCHIVE_INDEX_FILE).expect("the index is in it");
        serde_json::from_slice(&raw).expect("the index parses")
    }

    #[test]
    fn every_entry_the_index_names_is_in_the_archive_at_the_digest_it_claims() {
        let (bytes, count) = archive();
        let index = index_of(&bytes);
        assert!(index.is_readable());
        // Four documents plus the index itself.
        assert_eq!(count, 5);
        assert_eq!(index.entries.len(), 4);
        for entry in &index.entries {
            let found = extract(&bytes, &entry.path)
                .unwrap_or_else(|| panic!("{} is named but not present", entry.path));
            assert_eq!(
                found.len() as u64,
                entry.bytes,
                "{} has the wrong size",
                entry.path
            );
            assert_eq!(
                clipmill_export::digest_of(&found),
                entry.sha256,
                "{} does not hash to what the index says",
                entry.path
            );
        }
    }

    #[test]
    fn the_archived_document_is_the_document_the_daemon_holds() {
        // Re-import equivalence: what comes out of the zip parses as an Edit IR
        // document and is the one that went in, not a re-serialisation that
        // dropped a field on the way.
        let (bytes, _) = archive();
        let raw = extract(&bytes, "docs/doc_1/edit-ir.json").expect("the document is in it");
        let wrapper: serde_json::Value = serde_json::from_slice(&raw).expect("it parses");
        assert_eq!(wrapper["revision"], 2);
        let document = clipmill_edit_ir::EditDocument::from_canonical_json(
            wrapper["document"].to_string().as_bytes(),
        )
        .expect("the archived document is an Edit IR document");
        let fixture = inputs();
        let live = clipmill_edit_ir::EditDocument::from_canonical_json(
            fixture.docs[0].document_json.as_bytes(),
        )
        .expect("the live document parses");
        assert_eq!(
            document.to_canonical_json().expect("canonical"),
            live.to_canonical_json().expect("canonical")
        );
    }

    #[test]
    fn the_command_log_carries_the_document_it_was_applied_to() {
        // A list of commands without the thing they were applied to replays to
        // nothing, which is the whole reason the initial document travels.
        let (bytes, _) = archive();
        let raw = extract(&bytes, "docs/doc_1/commands.json").expect("the log is in it");
        let log: serde_json::Value = serde_json::from_slice(&raw).expect("it parses");
        assert!(log["initial_document"].is_object());
        assert_eq!(log["commands"][0]["revision"], 1);
        assert_eq!(log["commands"][0]["command"]["op"], "set_layout");
        // The inverse travels too, so the history can be walked backwards.
        assert_eq!(log["commands"][0]["inverse"]["state"], "speaker_fill");
    }

    #[test]
    fn sources_are_named_and_not_carried() {
        let (bytes, _) = archive();
        let index = index_of(&bytes);
        assert_eq!(index.sources.len(), 1);
        assert_eq!(index.sources[0].display_name, "episode-14.mov");
        assert!(index.sources[0].fingerprint.starts_with("sha256:"));
        // Eight gigabytes of footage is described in an archive of kilobytes.
        assert!(
            bytes.len() < 128 * 1024,
            "the archive carried media: {} bytes",
            bytes.len()
        );
    }

    #[test]
    fn two_archives_of_the_same_project_are_the_same_file() {
        // The writer contributes no time of its own, so the only thing that can
        // differ is the timestamp the caller supplied.
        assert_eq!(archive().0, archive().0);
    }

    /// Leave two archives on disk for the drill to open with an unrelated
    /// reader.
    ///
    /// A zip this project verifies with its own code is a zip nobody else has
    /// agreed to. The gate opens these with Python's `zipfile` — CRCs, central
    /// directory, and all — which is the only check that can fail when the
    /// writer is wrong about its own format.
    #[test]
    fn an_archive_is_left_where_an_unrelated_reader_can_open_it() {
        // Tests run with the crate root as the working directory, so the
        // workspace target directory is reached from the manifest rather than
        // from wherever cargo happened to put us.
        let work =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/export-drill");
        let work = work.as_path();
        std::fs::create_dir_all(work).expect("the drill directory is creatable");
        let (first, _) = archive();
        let (second, _) = archive();
        std::fs::write(work.join("project.zip"), &first).expect("the archive writes");
        std::fs::write(work.join("again.zip"), &second).expect("the second archive writes");
        assert_eq!(first, second);
    }

    #[test]
    fn duration_rounds_down_to_whole_seconds_and_never_underflows() {
        assert_eq!(seconds_of(90_000 * 52), 52);
        assert_eq!(seconds_of(89_999), 0);
        assert_eq!(seconds_of(-5), 0);
    }
}

/// Every role an export delivers, in the order it delivers them.
///
/// One list, read by the delivery and by the preview. Two lists would let a
/// preview promise a file the export does not write.
pub(crate) const DELIVERED_ROLES: [FileRole; 7] = [
    FileRole::Clip,
    FileRole::SubtitlesSrt,
    FileRole::SubtitlesVtt,
    FileRole::RenderManifest,
    FileRole::Thumbnail,
    FileRole::Metadata,
    FileRole::Checksums,
];

/// Check a destination without creating anything.
///
/// The preview's version of [`resolve_destination`]. A folder that does not
/// exist yet is not a problem — the export will make it — so the only answers
/// are "this is unusable" and "carry on".
pub(crate) fn probe_destination(raw: &str) -> Result<(), DestinationError> {
    let path = Path::new(raw);
    if raw.trim().is_empty() || !path.is_absolute() || raw.starts_with("\\\\") {
        return Err(DestinationError::NotAbsolute);
    }
    if !path.exists() {
        // Nothing to check yet, and nothing to create in a preview. The
        // nearest existing parent is what the export will land beside, so that
        // is what gets asked about.
        let Some(parent) = path.ancestors().find(|candidate| candidate.exists()) else {
            return Err(DestinationError::Unusable(
                "no part of this path exists".to_owned(),
            ));
        };
        return match network_filesystem(parent) {
            Some(kind) => Err(DestinationError::NotLocal(kind)),
            None => Ok(()),
        };
    }
    if !path.is_dir() {
        return Err(DestinationError::NotADirectory);
    }
    match network_filesystem(path) {
        Some(kind) => Err(DestinationError::NotLocal(kind)),
        None => Ok(()),
    }
}

/// Write a file through a temporary name, as the delivery does.
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let partial = path.with_extension(format!(
        "{}{PARTIAL_SUFFIX}",
        path.extension().unwrap_or_default().to_string_lossy()
    ));
    match fs::write(&partial, bytes).and_then(|()| fs::rename(&partial, path)) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _removed = fs::remove_file(&partial);
            Err(error)
        }
    }
}

/// A filename for a project's archive, from the project's own name.
pub(crate) fn archive_stem(project: &crate::db::ProjectRecord) -> String {
    // The naming code already knows how to turn a person's words into a
    // filename; reusing it means an archive and an export sanitize the same
    // way, rather than two functions disagreeing about apostrophes.
    Pattern::parse("{clip}").map_or_else(
        |_| project.project_id.clone(),
        |pattern| {
            pattern.resolve(&clipmill_export::Fields {
                project: String::new(),
                clip: project.name.clone(),
                index: 1,
                duration_seconds: 0,
                date: String::new(),
                address: project.project_id.clone(),
            })
        },
    )
}

/// Everything the archive carries, gathered by the caller that can read state.
pub(crate) struct ArchiveInputs<'a> {
    pub project: &'a crate::db::ProjectRecord,
    pub sources: &'a [crate::db::SourceRecord],
    pub docs: &'a [crate::db::EditDocRecord],
    /// One entry per document: its id, the document it started as, and every
    /// command applied since.
    pub logs: &'a [(String, String, Vec<crate::db::EditCommandRecord>)],
    pub decisions: &'a [(String, crate::db::DecisionRecord)],
    pub jobs: &'a [crate::jobs::JobRecord],
    pub created_unix_millis: u64,
    pub writer_version: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ArchiveError {
    #[error("the archive could not be assembled: {0}")]
    Serialise(String),
    #[error(transparent)]
    Zip(#[from] clipmill_export::ZipError),
}

/// Build the archive: its documents, its index, and the zip holding both.
///
/// The index is written last and describes everything before it, which is why
/// it cannot describe itself — a reader verifies the entries against the index
/// and the index against nothing, exactly as a checksum file works.
pub(crate) fn build_archive(inputs: &ArchiveInputs<'_>) -> Result<(Vec<u8>, u32), ArchiveError> {
    let mut writer = clipmill_export::ZipWriter::new();
    let mut entries = Vec::new();

    add(
        &mut writer,
        &mut entries,
        "state/project.json",
        EntryKind::State,
        &state_document(inputs),
    )?;

    for doc in inputs.docs {
        let document: serde_json::Value = serde_json::from_str(&doc.document_json)
            .map_err(|error| ArchiveError::Serialise(error.to_string()))?;
        add(
            &mut writer,
            &mut entries,
            &format!("docs/{}/edit-ir.json", doc.doc_id),
            EntryKind::EditDoc,
            &json!({ "revision": doc.revision, "document": document }),
        )?;
    }

    for (doc_id, initial, log) in inputs.logs {
        add(
            &mut writer,
            &mut entries,
            &format!("docs/{doc_id}/commands.json"),
            EntryKind::CommandLog,
            &command_log_document(initial, log)?,
        )?;
    }

    if !inputs.decisions.is_empty() {
        add(
            &mut writer,
            &mut entries,
            "decisions/decisions.json",
            EntryKind::Decisions,
            &decisions_document(inputs.decisions),
        )?;
    }

    let index = clipmill_export::ArchiveIndex::new(
        inputs.project.project_id.clone(),
        inputs.project.name.clone(),
        inputs.created_unix_millis,
        inputs.writer_version.to_owned(),
        inputs
            .sources
            .iter()
            .map(|source| clipmill_export::ArchivedSource {
                source_id: source.source_id.clone(),
                fingerprint: source.source_fingerprint.clone(),
                display_name: Path::new(&source.observation.absolute_path)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            })
            .collect(),
        entries,
    );
    let index_bytes = serde_json::to_vec_pretty(&index)
        .map_err(|error| ArchiveError::Serialise(error.to_string()))?;
    writer.add(clipmill_export::ARCHIVE_INDEX_FILE, &index_bytes)?;

    let count = u32::try_from(index.entries.len() + 1).unwrap_or(u32::MAX);
    Ok((writer.finish()?, count))
}

/// The project row, its jobs, and its sources, as the daemon holds them.
///
/// The one part of an archive that is state rather than a document: everything
/// else in the zip already exists as JSON somewhere, and this has to be written
/// out from rows.
fn state_document(inputs: &ArchiveInputs<'_>) -> serde_json::Value {
    json!({
        "project": {
            "project_id": inputs.project.project_id,
            "name": inputs.project.name,
            "created_unix_millis": inputs.project.created_unix_millis,
        },
        "jobs": inputs.jobs.iter().map(|job| json!({
            "job_id": job.job_id,
            "kind": job.kind,
            "state": job.state,
            "created_unix_millis": job.created_unix_millis,
            "output_artifact_ids": job.output_artifact_ids,
        })).collect::<Vec<_>>(),
        "sources": inputs.sources.iter().map(|source| json!({
            "source_id": source.source_id,
            "fingerprint": source.source_fingerprint,
            "absolute_path": source.observation.absolute_path,
            "byte_size": source.observation.byte_size,
        })).collect::<Vec<_>>(),
    })
}

/// One document's history: what it started as, and every command since.
///
/// The initial document travels with the log because a list of commands
/// without the thing they were applied to replays to nothing. Inverses travel
/// too, so a reader can walk the history backwards as well as forwards.
fn command_log_document(
    initial: &str,
    log: &[crate::db::EditCommandRecord],
) -> Result<serde_json::Value, ArchiveError> {
    let initial_value: serde_json::Value = serde_json::from_str(initial)
        .map_err(|error| ArchiveError::Serialise(error.to_string()))?;
    let commands = log
        .iter()
        .map(|entry| {
            Ok(json!({
                "revision": entry.revision,
                "command": serde_json::from_str::<serde_json::Value>(&entry.command_json)
                    .map_err(|error| ArchiveError::Serialise(error.to_string()))?,
                "inverse": serde_json::from_str::<serde_json::Value>(&entry.inverse_json)
                    .map_err(|error| ArchiveError::Serialise(error.to_string()))?,
            }))
        })
        .collect::<Result<Vec<_>, ArchiveError>>()?;
    Ok(json!({ "initial_document": initial_value, "commands": commands }))
}

/// What a user decided about each candidate, with the recording it came from.
fn decisions_document(decisions: &[(String, crate::db::DecisionRecord)]) -> serde_json::Value {
    json!({
        "decisions": decisions
            .iter()
            .map(|(source_id, record)| json!({
                "source_id": source_id,
                "candidate_id": record.candidate_id,
                "decision": format!("{:?}", record.decision).to_lowercase(),
                "decided_unix_millis": record.decided_unix_millis,
            }))
            .collect::<Vec<_>>(),
    })
}

fn add(
    writer: &mut clipmill_export::ZipWriter,
    entries: &mut Vec<clipmill_export::ArchiveEntry>,
    path: &str,
    kind: EntryKind,
    value: &serde_json::Value,
) -> Result<(), ArchiveError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| ArchiveError::Serialise(error.to_string()))?;
    writer.add(path, &bytes)?;
    entries.push(clipmill_export::ArchiveEntry {
        path: path.to_owned(),
        kind,
        sha256: digest_of(&bytes),
        bytes: bytes.len() as u64,
    });
    Ok(())
}
