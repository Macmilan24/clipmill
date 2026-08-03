//! The documents this crate writes are the documents the schemas describe.
//!
//! Both published documents are hand-written structs here and generated types
//! in `clipmill-contracts`, which is two descriptions of one format and
//! therefore two things that can drift. The render manifest has the same shape
//! and the same risk.
//!
//! So the two are pinned to each other directly: serialise the struct this
//! crate writes, parse it with the type generated from the published schema,
//! and go back. A renamed field, a changed enum spelling, or a number that
//! stopped being an integer fails here rather than in a user's folder — and it
//! fails at `cargo test`, without a daemon, a render, or a disk.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use clipmill_contracts::schemas::{
    archive_index as archive_schema, export_package as package_schema,
};
use clipmill_export::{
    ArchiveEntry, ArchiveIndex, ArchivedSource, AudioSummary, DeliveredFile, Disclosure, EntryKind,
    ExportPackage, FileRole, VideoSummary,
};

fn index() -> ArchiveIndex {
    ArchiveIndex::new(
        "prj_01J".to_owned(),
        "Pricing Talk".to_owned(),
        1_764_000_000_000,
        "0.0.1".to_owned(),
        vec![ArchivedSource {
            source_id: "src_01J".to_owned(),
            fingerprint: "sha256:".to_owned() + &"1".repeat(64),
            display_name: "episode-14.mov".to_owned(),
        }],
        vec![
            ArchiveEntry {
                path: "state/project.json".to_owned(),
                kind: EntryKind::State,
                sha256: "a".repeat(64),
                bytes: 512,
            },
            ArchiveEntry {
                path: "docs/doc_01J/edit-ir.json".to_owned(),
                kind: EntryKind::EditDoc,
                sha256: "b".repeat(64),
                bytes: 4_096,
            },
            ArchiveEntry {
                path: "docs/doc_01J/commands.json".to_owned(),
                kind: EntryKind::CommandLog,
                sha256: "c".repeat(64),
                bytes: 128,
            },
            ArchiveEntry {
                path: "renders/render-manifest.json".to_owned(),
                kind: EntryKind::RenderManifest,
                sha256: "d".repeat(64),
                bytes: 2_048,
            },
            ArchiveEntry {
                path: "decisions/decisions.json".to_owned(),
                kind: EntryKind::Decisions,
                sha256: "e".repeat(64),
                bytes: 64,
            },
        ],
    )
}

fn package() -> ExportPackage {
    ExportPackage::new(
        "doc_01J".to_owned(),
        "Charging less is lying to yourself".to_owned(),
        "art_01J".to_owned(),
        VideoSummary {
            width: 1080,
            height: 1920,
            frame_rate_num: 30_000,
            frame_rate_den: 1_001,
            frame_count: 1_560,
            duration_ticks: 4_684_680,
        },
        AudioSummary {
            target_lufs: -14.0,
            measured_lufs: -14.08,
            measured_true_peak_dbtp: -1.4,
        },
        Disclosure {
            source_attestation: "own_content".to_owned(),
            gates_passed: vec!["duration_60s".to_owned()],
            ai_assistance: vec!["asr_captions".to_owned(), "reframe".to_owned()],
            requires_ai_disclosure: false,
        },
        vec![
            DeliveredFile {
                name: "03-charging-less.mp4".to_owned(),
                role: FileRole::Clip,
                sha256: "1".repeat(64),
                bytes: 12_000_000,
            },
            DeliveredFile {
                name: "03-charging-less.srt".to_owned(),
                role: FileRole::SubtitlesSrt,
                sha256: "2".repeat(64),
                bytes: 900,
            },
            DeliveredFile {
                name: "03-charging-less.vtt".to_owned(),
                role: FileRole::SubtitlesVtt,
                sha256: "3".repeat(64),
                bytes: 940,
            },
            DeliveredFile {
                name: "03-charging-less.jpg".to_owned(),
                role: FileRole::Thumbnail,
                sha256: "4".repeat(64),
                bytes: 68_000,
            },
            DeliveredFile {
                name: "03-charging-less.render-manifest.json".to_owned(),
                role: FileRole::RenderManifest,
                sha256: "5".repeat(64),
                bytes: 2_048,
            },
            DeliveredFile {
                name: "03-charging-less.metadata.json".to_owned(),
                role: FileRole::Metadata,
                sha256: "6".repeat(64),
                bytes: 1_100,
            },
            DeliveredFile {
                name: "03-charging-less.sha256".to_owned(),
                role: FileRole::Checksums,
                sha256: "7".repeat(64),
                bytes: 420,
            },
        ],
    )
}

#[test]
fn an_archive_index_parses_as_the_published_schema_and_returns_unchanged() {
    let written = serde_json::to_value(index()).expect("the index serialises");
    let parsed: archive_schema::ArchiveIndex =
        serde_json::from_value(written.clone()).expect("the published schema accepts it");
    let again = serde_json::to_value(&parsed).expect("the generated type serialises");
    assert_eq!(again, written);

    let back: ArchiveIndex = serde_json::from_value(again).expect("it comes back");
    assert_eq!(back, index());
}

#[test]
fn every_entry_kind_this_crate_writes_is_a_kind_the_schema_names() {
    // The vocabulary is closed on both sides, so a kind added here without
    // being added to the schema is a document nobody else can read.
    let written = serde_json::to_value(index()).expect("serialises");
    let parsed: archive_schema::ArchiveIndex =
        serde_json::from_value(written).expect("the schema accepts every kind in the fixture");
    assert_eq!(parsed.entries.len(), 5);
}

#[test]
fn an_export_package_parses_as_the_published_schema_and_returns_unchanged() {
    let written = serde_json::to_value(package()).expect("the package serialises");
    let parsed: package_schema::ExportPackage =
        serde_json::from_value(written.clone()).expect("the published schema accepts it");
    let again = serde_json::to_value(&parsed).expect("the generated type serialises");
    assert_eq!(again, written);

    let back: ExportPackage = serde_json::from_value(again).expect("it comes back");
    assert_eq!(back, package());
}

#[test]
fn every_delivered_role_this_crate_writes_is_a_role_the_schema_names() {
    let written = serde_json::to_value(package()).expect("serialises");
    let parsed: package_schema::ExportPackage =
        serde_json::from_value(written).expect("the schema accepts every role in the fixture");
    assert_eq!(parsed.files.len(), 7);
}

#[test]
fn the_version_constant_is_the_readers_job_because_the_generated_type_will_not_do_it() {
    // Worth stating rather than assuming. The schema pins `schema_version` with
    // a `const`, but the Rust generator renders a bare const as an unchecked
    // `serde_json::Value` — the published render manifest has the same shape,
    // so this is a property of the toolchain and not of these two schemas.
    //
    // The consequence: parsing an archive from a future format succeeds and
    // tells you nothing, so refusing one is the reader's job. `is_readable` is
    // that reader, and this test exists so that a later generator which *does*
    // enforce the constant is noticed rather than silently relied upon.
    let mut written = serde_json::to_value(index()).expect("serialises");
    written["schema_version"] = serde_json::Value::String("clipmill.archive_index.v9".to_owned());

    let parsed: archive_schema::ArchiveIndex =
        serde_json::from_value(written.clone()).expect("the generated type does not check it");
    assert_eq!(parsed.schema_version, "clipmill.archive_index.v9");

    let ours: ArchiveIndex = serde_json::from_value(written).expect("parses here too");
    assert!(
        !ours.is_readable(),
        "a v9 archive must not be treated as one this build can read"
    );
}

#[test]
fn an_entry_path_that_escapes_the_extraction_directory_is_refused_by_the_schema() {
    let mut written = serde_json::to_value(index()).expect("serialises");
    written["entries"][0]["path"] = serde_json::Value::String("/etc/passwd".to_owned());
    let parsed: Result<archive_schema::ArchiveIndex, _> = serde_json::from_value(written);
    assert!(parsed.is_err(), "an absolute entry path was accepted");
}
