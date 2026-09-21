//! Record render inputs, measured output, rights, and model assistance.
//!
//! The manifest identifies the Edit IR, engine, font, measured loudness, rights
//! attestation, and model contributions needed to audit a delivered render.

use serde::{Deserialize, Serialize};

use crate::{plan::SegmentReport, profile::RenderProfile, subtitles::CueWindow};

pub const SCHEMA_VERSION: &str = "clipmill.render.clip.v1";

/// Identity of everything outside the document that could change the output.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EngineIdentity {
    pub app: String,
    /// The pinned FFmpeg substrate identity (decision R4).
    pub ffmpeg: String,
    /// Content hash of the single font libass was allowed to see.
    pub font_sha256: String,
    pub font_family: String,
}

/// Where a model's work appears in the result.
///
/// The document author supplies these lists from provenance. The renderer
/// records them without inferring model use from the finished media.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AiUseSummary {
    /// Model work that shaped existing footage: `asr_captions`, `reframe`, …
    pub assistance: Vec<String>,
    /// Synthesised imagery or audio; empty when none was used.
    pub generated: Vec<String>,
    pub requires_youtube_ai_disclosure: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RightsAttestation {
    /// What the user attested about the footage, echoed verbatim.
    pub source_attestation: String,
    pub gates_passed: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MeasuredLoudness {
    pub integrated_lufs: f64,
    pub true_peak_dbtp: f64,
    pub loudness_range_lu: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LoudnessReport {
    pub target_lufs: f64,
    pub target_true_peak_dbtp: f64,
    /// What the assembled program measured before normalisation.
    pub measured_input: MeasuredLoudness,
    /// What the finished file measures. Re-decoded from the output rather than
    /// predicted from the filter's arguments: the joiner verifies, not hopes.
    pub measured_output: MeasuredLoudness,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputFile {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProgramSegment {
    pub segment_id: String,
    pub source_fingerprint: String,
    pub in_ticks: i64,
    pub out_ticks: i64,
    pub layout: String,
    pub frame_count: i64,
}

impl From<&SegmentReport> for ProgramSegment {
    fn from(value: &SegmentReport) -> Self {
        Self {
            segment_id: value.segment_id.clone(),
            source_fingerprint: value.source_fingerprint.clone(),
            in_ticks: value.in_ticks,
            out_ticks: value.out_ticks,
            layout: value.layout.clone(),
            frame_count: value.frame_count,
        }
    }
}

/// The frames a cue occupies in the finished file. Recorded so that checking
/// captions against the IR is reading a number, not re-deriving one.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CaptionWindow {
    pub cue_id: String,
    pub first_frame: i64,
    pub end_frame: i64,
}

impl From<&CueWindow> for CaptionWindow {
    fn from(value: &CueWindow) -> Self {
        Self {
            cue_id: value.cue_id.clone(),
            first_frame: value.first_frame,
            end_frame: value.end_frame,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RenderManifest {
    pub schema_version: String,
    /// Content hash of the Edit IR snapshot this render read.
    pub ir_hash: String,
    pub ir_artifact_id: String,
    pub profile: RenderProfile,
    pub engine: EngineIdentity,
    /// `byte_stable` on a platform whose encoder reproduces bytes;
    /// `semantic` where only the decoded result is guaranteed to match.
    pub determinism: String,
    pub ai_use_summary: AiUseSummary,
    pub rights: RightsAttestation,
    /// Fingerprints of every source that contributed frames.
    pub input_source_fingerprints: Vec<String>,
    pub program: ProgramReport,
    pub loudness: LoudnessReport,
    pub caption_windows: Vec<CaptionWindow>,
    pub outputs: Vec<OutputFile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProgramReport {
    pub duration_ticks: i64,
    pub frame_count: i64,
    pub segments: Vec<ProgramSegment>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{AiUseSummary, EngineIdentity, RightsAttestation};

    #[test]
    fn an_untouched_render_discloses_nothing_by_default() {
        let summary = AiUseSummary::default();
        assert!(summary.assistance.is_empty());
        assert!(summary.generated.is_empty());
        assert!(!summary.requires_youtube_ai_disclosure);
    }

    #[test]
    fn engine_identity_round_trips_through_json() {
        let engine = EngineIdentity {
            app: "clipmill 0.0.1".to_owned(),
            ffmpeg: "ffmpeg-8.1.2-btb-n8.1.2".to_owned(),
            font_sha256: "sha256:abc".to_owned(),
            font_family: "Inter".to_owned(),
        };
        let text = serde_json::to_string(&engine).expect("serialize");
        let parsed: EngineIdentity = serde_json::from_str(&text).expect("parse");
        assert_eq!(parsed, engine);
    }

    #[test]
    fn rights_are_echoed_rather_than_summarised() {
        let rights = RightsAttestation {
            source_attestation: "own_content".to_owned(),
            gates_passed: vec!["duration_60s".to_owned()],
        };
        assert_eq!(rights.source_attestation, "own_content");
        assert_eq!(rights.gates_passed, vec!["duration_60s".to_owned()]);
    }
}
