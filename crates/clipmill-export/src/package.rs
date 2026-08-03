//! The metadata file that ships beside the clip.
//!
//! An export is not one file. It is a clip, two sidecars, a thumbnail, a copy
//! of the render manifest, and this — the document that answers the questions
//! an upload form asks, in one place, without the user opening anything.
//!
//! It restates rather than reinterprets. Every number here was measured by the
//! renderer and recorded in the render manifest; this document copies them and
//! names the files they belong to. Nothing is computed a second time, because a
//! second computation is a second chance to disagree — and the disclosure
//! fields in particular are a claim about the work, where a disagreement is not
//! a bug but a false statement.

use serde::{Deserialize, Serialize};

pub const PACKAGE_SCHEMA_VERSION: &str = "clipmill.export.package.v1";
/// What the metadata file is called inside an export.
pub const PACKAGE_SUFFIX: &str = ".metadata.json";
pub const THUMBNAIL_SUFFIX: &str = ".jpg";
pub const CHECKSUMS_SUFFIX: &str = ".sha256";

/// What a delivered file is for.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRole {
    Clip,
    SubtitlesSrt,
    SubtitlesVtt,
    Thumbnail,
    RenderManifest,
    Metadata,
    Checksums,
}

impl FileRole {
    /// The extension a role is delivered under.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Clip => "mp4",
            Self::SubtitlesSrt => "srt",
            Self::SubtitlesVtt => "vtt",
            Self::Thumbnail => "jpg",
            Self::RenderManifest => "render-manifest.json",
            Self::Metadata => "metadata.json",
            Self::Checksums => "sha256",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DeliveredFile {
    /// The file's name inside the export folder. Never a path.
    pub name: String,
    pub role: FileRole,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VideoSummary {
    pub width: u32,
    pub height: u32,
    pub frame_rate_num: u32,
    pub frame_rate_den: u32,
    pub frame_count: i64,
    pub duration_ticks: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AudioSummary {
    pub target_lufs: f64,
    /// What the finished file measured, re-decoded by the renderer.
    pub measured_lufs: f64,
    pub measured_true_peak_dbtp: f64,
}

/// The rights claim and the model-use disclosure, echoed verbatim.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Disclosure {
    pub source_attestation: String,
    pub gates_passed: Vec<String>,
    /// Model work that shaped the footage, e.g. `asr_captions`.
    pub ai_assistance: Vec<String>,
    /// Whether a platform's synthetic-media question should be answered yes.
    /// Carried rather than inferred at upload time, because by then the person
    /// answering it may not be the person who made the clip.
    pub requires_ai_disclosure: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExportPackage {
    pub schema: String,
    pub doc_id: String,
    /// The clip's own words, when it has a title.
    pub title: String,
    /// The render this was delivered from, so a package can be traced back to
    /// the artifact that produced it.
    pub render_artifact_id: String,
    pub video: VideoSummary,
    pub audio: AudioSummary,
    pub disclosure: Disclosure,
    pub files: Vec<DeliveredFile>,
}

impl ExportPackage {
    pub fn new(
        doc_id: String,
        title: String,
        render_artifact_id: String,
        video: VideoSummary,
        audio: AudioSummary,
        disclosure: Disclosure,
        mut files: Vec<DeliveredFile>,
    ) -> Self {
        files.sort_by(|left, right| left.name.cmp(&right.name));
        Self {
            schema: PACKAGE_SCHEMA_VERSION.to_owned(),
            doc_id,
            title,
            render_artifact_id,
            video,
            audio,
            disclosure,
            files,
        }
    }
}

/// The `sha256sum`-compatible checksum file for a delivery.
///
/// The format is the one `sha256sum -c` reads, so verifying an export needs
/// nothing from this project — which is the entire point of writing it.
pub fn checksum_file(files: &[DeliveredFile]) -> String {
    let mut lines: Vec<String> = files
        .iter()
        .filter(|file| file.role != FileRole::Checksums)
        .map(|file| format!("{}  {}", file.sha256, file.name))
        .collect();
    lines.sort();
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{
        AudioSummary, DeliveredFile, Disclosure, ExportPackage, FileRole, VideoSummary,
        checksum_file,
    };

    fn file(name: &str, role: FileRole) -> DeliveredFile {
        DeliveredFile {
            name: name.to_owned(),
            role,
            sha256: "a".repeat(64),
            bytes: 10,
        }
    }

    fn package(files: Vec<DeliveredFile>) -> ExportPackage {
        ExportPackage::new(
            "doc_1".to_owned(),
            "Charging less".to_owned(),
            "art_1".to_owned(),
            VideoSummary {
                width: 1080,
                height: 1920,
                frame_rate_num: 30_000,
                frame_rate_den: 1_001,
                frame_count: 180,
                duration_ticks: 540_000,
            },
            AudioSummary {
                target_lufs: -14.0,
                measured_lufs: -14.1,
                measured_true_peak_dbtp: -1.2,
            },
            Disclosure {
                source_attestation: "own_content".to_owned(),
                gates_passed: vec!["duration_60s".to_owned()],
                ai_assistance: vec!["asr_captions".to_owned()],
                requires_ai_disclosure: false,
            },
            files,
        )
    }

    #[test]
    fn files_are_listed_in_one_order_whatever_order_they_were_delivered_in() {
        let forwards = package(vec![
            file("a.mp4", FileRole::Clip),
            file("a.srt", FileRole::SubtitlesSrt),
        ]);
        let backwards = package(vec![
            file("a.srt", FileRole::SubtitlesSrt),
            file("a.mp4", FileRole::Clip),
        ]);
        assert_eq!(forwards.files, backwards.files);
    }

    #[test]
    fn the_checksum_file_is_what_sha256sum_reads() {
        let text = checksum_file(&[
            file("clip.mp4", FileRole::Clip),
            file("clip.srt", FileRole::SubtitlesSrt),
        ]);
        assert_eq!(
            text,
            format!("{0}  clip.mp4\n{0}  clip.srt\n", "a".repeat(64))
        );
    }

    #[test]
    fn the_checksum_file_does_not_try_to_contain_its_own_hash() {
        let text = checksum_file(&[
            file("clip.mp4", FileRole::Clip),
            file("clip.sha256", FileRole::Checksums),
        ]);
        assert!(!text.contains("clip.sha256"), "{text}");
    }

    #[test]
    fn the_disclosure_survives_a_round_trip_through_json() {
        let original = package(vec![file("a.mp4", FileRole::Clip)]);
        let text = serde_json::to_string(&original).expect("serialises");
        let back: ExportPackage = serde_json::from_str(&text).expect("deserialises");
        assert_eq!(back.disclosure, original.disclosure);
        assert!(
            text.contains("\"subtitles_srt\"") || text.contains("\"clip\""),
            "{text}"
        );
    }
}
