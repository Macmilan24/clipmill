//! Replay saved editorial inputs offline; no worker, model, media, or database writes.
//!
//! `cargo run -p clipmill-editorial --example replay -- INPUT_DIR OUTPUT_DIR [JUDGMENTS_JSON]`
//!
//! `INPUT_DIR` accepts transcript/windows/proposals/index.json or the daemon-stage-prefixed
//! audit filenames. The default range is 20–90 seconds and ten requested clips. Optional
//! judgments must reference the candidates content digest reported by this replay; their
//! source/candidate identity is never rewritten. These are offline evidence files, not
//! artifacts published into the daemon's store. No private recording fixture is checked in.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clipmill_contracts::schemas::{
    editorial_judgments::EditorialJudgments,
    editorial_proposals::EditorialProposals,
    editorial_windows::{EditorialWindows, InvalidRegionReason},
    index_transcript::IndexTranscript,
    speech_transcript::SpeechTranscript,
};
use clipmill_discovery::{RankingInputs, Request, rank};
use clipmill_editorial::{review, validate};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use sha2::{Digest, Sha256};

const MIN_TICKS: u64 = 20 * 90_000;
const MAX_TICKS: u64 = 90 * 90_000;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if !(2..=3).contains(&arguments.len()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: replay INPUT_DIR OUTPUT_DIR [JUDGMENTS_JSON] (offline; 20–90s, ten clips)",
        )
        .into());
    }
    let input = PathBuf::from(&arguments[0]);
    let output = PathBuf::from(&arguments[1]);
    let transcript: SpeechTranscript = read(&input, "transcript.json", "speech-transcript")?;
    let windows: EditorialWindows = read(&input, "windows.json", "editorial-windows")?;
    let proposals: EditorialProposals = read(&input, "proposals.json", "editorial-propose")?;
    let index: IndexTranscript = read(&input, "index.json", "index-transcript")?;
    let validated = validate::proposals(
        &windows,
        &transcript,
        &proposals,
        proposals.inputs.windows_artifact_id.as_str(),
        MIN_TICKS,
        MAX_TICKS,
    )?;
    fs::create_dir_all(&output)?;
    let candidates_digest = write(&output, "candidates.json", &validated.candidates)?;
    write(&output, "validation-report.json", &validated.report)?;
    let ranking = rank(
        &validated.candidates,
        &index,
        &transcript,
        RankingInputs {
            candidates: &candidates_digest,
            index: windows.inputs.index_artifact_id.as_str(),
            transcript: windows.inputs.transcript_artifact_id.as_str(),
        },
        Request::default(),
        "clipmill-editorial-offline-replay@1",
    )?;
    let before_review = ranking.cohort.len();
    write(&output, "ranking-before-review.json", &ranking)?;
    let reviewed = if let Some(judgments_file) = arguments.get(2) {
        let bytes = fs::read(judgments_file)?;
        let judgments: EditorialJudgments = serde_json::from_slice(&bytes)?;
        let uncertain: Vec<_> = windows
            .invalid_regions
            .iter()
            .filter(|region| {
                matches!(
                    region.reason,
                    InvalidRegionReason::AlignmentUnavailable
                        | InvalidRegionReason::TimingInterpolated
                )
            })
            .map(|region| (region.start_ticks, region.end_ticks))
            .collect();
        let reviewed = review::apply(
            ranking,
            &judgments,
            &digest(&bytes),
            windows.sentences.len(),
            &uncertain,
        )?;
        write(&output, "ranking-reviewed.json", &reviewed)?;
        Some(reviewed)
    } else {
        None
    };
    let report = json!({
        "mode": "offline replay; no model calls or daemon publication",
        "source_fingerprint": windows.source_fingerprint,
        "duration_ticks": {"min": MIN_TICKS, "max": MAX_TICKS},
        "requested_count": Request::default().count,
        "window_count": windows.windows.len(),
        "candidates": validated.candidates.candidates.len(),
        "ranked_before_review": before_review,
        "semantic_review_applied": reviewed.is_some(),
        "ranked_after_review": reviewed.as_ref().map(|r| r.cohort.len()),
        "candidates_content_digest": candidates_digest,
        "digest_note": "Content digest of candidates.json for this offline replay, not a daemon artifact-store identity."
    });
    write(&output, "replay-report.json", &report)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn read<T: DeserializeOwned>(root: &Path, name: &str, stage: &str) -> Result<T, Box<dyn Error>> {
    let plain = root.join(name);
    let location = if plain.exists() {
        plain
    } else {
        root.join(format!("{stage}-{name}"))
    };
    Ok(serde_json::from_slice(&fs::read(location)?)?)
}

fn write(root: &Path, name: &str, value: &impl Serialize) -> Result<String, Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(root.join(name), &bytes)?;
    Ok(digest(&bytes))
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}
