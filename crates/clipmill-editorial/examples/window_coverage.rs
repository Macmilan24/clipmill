//! Audit duration-valid span coverage without loading a model or publishing artifacts.
//! `cargo run -p clipmill-editorial --example window_coverage -- INDEX_JSON [MAX_SECONDS]`
use clipmill_contracts::schemas::{
    editorial_windows::EditorialWindows, index_transcript::IndexTranscript,
};
use clipmill_editorial::{Budget, Inputs, windows};
use serde_json::json;
use std::{error::Error, fs, io};

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if !(1..=2).contains(&arguments.len()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: window_coverage INDEX_JSON [MAX_SECONDS]",
        )
        .into());
    }
    let index: IndexTranscript = serde_json::from_slice(&fs::read(&arguments[0])?)?;
    let maximum = arguments
        .get(1)
        .map(|s| s.parse::<u64>())
        .transpose()?
        .unwrap_or(90)
        * 90_000;
    let inputs = Inputs {
        index: "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        transcript: index.inputs.transcript_artifact_id.as_str(),
    };
    let old = windows(
        &index,
        inputs,
        Budget {
            max_clip_ticks: 0,
            ..Budget::DEFAULT
        },
        "offline-window-audit",
    )?;
    let covered = windows(
        &index,
        inputs,
        Budget {
            max_clip_ticks: maximum,
            ..Budget::DEFAULT
        },
        "offline-window-audit",
    )?;
    let mut eligible = 0;
    let mut before = 0;
    let mut after = 0;
    for first in &index.sentences {
        for last in index.sentences.iter().skip(usize::try_from(first.index)?) {
            let duration = last.end_ticks.saturating_sub(first.start_ticks);
            if (20 * 90_000..=maximum).contains(&duration) {
                eligible += 1;
                before += usize::from(!fits(&old, first.index, last.index));
                after += usize::from(!fits(&covered, first.index, last.index));
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "mode": "offline sentence-span coverage; no assessment of editorial quality",
            "eligible_spans": eligible, "uncovered_before": before, "uncovered_after": after,
            "windows_before": old.windows.len(), "windows_after": covered.windows.len(),
            "largest_window_words": covered.windows.iter().map(|w| w.word_count.get()).max().unwrap_or(0),
        }))?
    );
    if after > 0 {
        return Err(io::Error::other("eligible sentence spans are still uncovered").into());
    }
    Ok(())
}
fn fits(windows: &EditorialWindows, first: u64, last: u64) -> bool {
    windows.windows.iter().any(|window| {
        window.first_sentence_index <= first
            && window.first_sentence_index + window.sentence_count.get() > last
    })
}
