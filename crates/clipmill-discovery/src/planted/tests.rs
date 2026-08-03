//! Run the whole engine over the planted recording and leave the result where
//! the recall gate can score it.
//!
//! This test asserts only what it can assert without owning the metric: that
//! the pipeline ran, that the documents are well formed, and that they reached
//! disk. Whether the planted moments were *recalled* is decided by
//! `clipmill_eval.recall`, which is the one implementation of that arithmetic —
//! duplicating the `IoU` rule here would create a second one, and the two would
//! eventually disagree about the number the project quotes about itself.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{fs, path::PathBuf};

use crate::{
    Inputs, Parameters, discover,
    fixture::{IMPLEMENTATION, INDEX_ID, TRANSCRIPT_ID},
    planted,
    ranking::{self, Request},
};

/// Where the gate looks. Tests run with the crate root as the working
/// directory, so the workspace target is reached from the manifest.
fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/recall-smoke")
}

#[test]
fn the_planted_recording_runs_end_to_end_and_is_left_for_the_gate() {
    let recording = planted::recording();
    let candidates = discover(
        &recording.index,
        &recording.transcript,
        None,
        Inputs {
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
            loudness: None,
        },
        Parameters::DEFAULT,
        IMPLEMENTATION,
    )
    .expect("the search runs over a planted recording");
    assert!(
        !candidates.candidates.is_empty(),
        "a recording with three planted moments nominated nothing"
    );

    let ranked = ranking::rank(
        &candidates,
        &recording.index,
        &recording.transcript,
        ranking::Inputs {
            candidates: "sha256:c0000000000000000000000000000000000000000000000000000000000000ca",
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
        },
        Request::DEFAULT,
        IMPLEMENTATION,
    )
    .expect("the ranking runs");
    assert!(
        !ranked.selected.is_empty(),
        "a cohort of candidates selected nothing to show"
    );

    let directory = output_dir();
    fs::create_dir_all(&directory).expect("the smoke directory is creatable");
    write(&directory.join("candidates.json"), &candidates);
    write(&directory.join("ranking.json"), &ranked);

    // The answers, in the published annotation format, as though an editor had
    // written them down — which for this recording is exactly what happened.
    let moments: Vec<serde_json::Value> = recording
        .moments
        .iter()
        .map(|moment| {
            serde_json::json!({
                "moment_id": moment.moment_id,
                "span": {
                    "start_ticks": moment.start_ticks,
                    "end_ticks": moment.end_ticks,
                },
                "importance": moment.importance,
            })
        })
        .collect();
    let annotation = serde_json::json!({
        "schema_version": "clipmill.eval_annotation.v1",
        "source_fingerprint": *recording.index.source_fingerprint,
        "item_id": "planted",
        "annotator_id": "planted-fixture",
        "annotated_unix_millis": 0,
        "timebase": { "num": 1, "den": 90_000 },
        "moments": moments,
        "exclusions": [],
    });
    fs::write(
        directory.join("annotation.json"),
        serde_json::to_vec_pretty(&annotation).expect("the annotation serialises"),
    )
    .expect("the annotation writes");
}

fn write<T: serde::Serialize>(path: &std::path::Path, value: &T) {
    fs::write(path, serde_json::to_vec_pretty(value).expect("serialises")).expect("writes");
}
