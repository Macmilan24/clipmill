//! The document an approved candidate produces, written down.
//!
//! The claim the director rests on is that assembling an edit is not a
//! judgement, and the way to hold a claim like that is a golden. If the same
//! candidate, the same boundary and the same evidence stop producing the same
//! bytes, something started deciding — and an editor who approves the same clip
//! twice and gets two different edits has been told the tool is guessing.
//!
//! The evidence is built here rather than read from the contract fixtures
//! because the three documents have to agree with each other about one
//! recording: the same fingerprint, the same candidate id, a lattice the
//! boundary is actually on, and words inside the span. Fixtures that were
//! written for three different schemas do not, and making them agree by hand
//! would be a fourth fixture nobody maintains.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use clipmill_contracts::schemas::{
    discovery_candidates::DiscoveryCandidates, ranking_set::RankingSet,
    speech_transcript::SpeechTranscript,
};
use clipmill_director::{Aspect, Boundary, Cut, Evidence, Frame, Request, direct};
use serde_json::json;

const FINGERPRINT: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const ADDRESS: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const CANDIDATE: &str = "cand_00000000000000a1";
const SECOND: u64 = 90_000;

fn transcript() -> SpeechTranscript {
    let words: Vec<_> = "The whole point of pricing is that it is a decision you make on purpose."
        .split_whitespace()
        .enumerate()
        .map(|(index, text)| {
            let start = 10 * SECOND + index as u64 * SECOND / 2;
            json!({
                "index": index,
                "segment_index": 0,
                "text": text,
                "start_ticks": start,
                "end_ticks": start + SECOND / 3,
                "confidence": { "p50": 0.95, "p10": 0.8 },
                "timing": "aligned",
            })
        })
        .collect();
    serde_json::from_value(json!({
        "schema_version": "clipmill.speech.transcript.v1",
        "source_fingerprint": FINGERPRINT,
        "inputs": { "vad_artifact_id": ADDRESS, "asr_artifact_id": ADDRESS, "alignment_artifact_id": ADDRESS },
        "producers": [{ "stage": "transcribe-source", "implementation": "test@1" }],
        "language": "en",
        "language_confidence": 0.99,
        "confidence": { "p50": 0.95, "p10": 0.8 },
        "coverage": {
            "start_ticks": 0, "end_ticks": 120 * SECOND, "analyzed": true,
            "speech_ticks": 30 * SECOND, "aligned_ticks": 30 * SECOND, "sampling_plan": "full",
        },
        "words": words,
        "segments": [{
            "index": 0, "start_ticks": 10 * SECOND, "end_ticks": 20 * SECOND,
            "text": "The whole point of pricing is that it is a decision you make on purpose.",
            "first_word_index": 0, "word_count": 15,
            "confidence": { "p50": 0.95, "p10": 0.8 },
        }],
        "silences": [],
        "invalid_regions": [],
    }))
    .expect("a transcript")
}

fn candidates() -> DiscoveryCandidates {
    serde_json::from_value(json!({
        "schema_version": "clipmill.discovery.candidates.v1",
        "source_fingerprint": FINGERPRINT,
        "inputs": { "index_artifact_id": ADDRESS, "transcript_artifact_id": ADDRESS },
        "producer": { "stage": "discover-candidates", "implementation": "test@1" },
        "coverage": { "start_ticks": 0, "end_ticks": 120 * SECOND, "analyzed": true },
        "duration_target": { "min_ticks": 15 * SECOND, "max_ticks": 90 * SECOND },
        "proposers": [{
            "proposer": { "name": "test", "rubric": "test", "version": "1" },
            "seeds": 1, "candidates": 1,
        }],
        "candidates": [{
            "id": CANDIDATE,
            "intervals": [{ "start_ticks": 10 * SECOND, "end_ticks": 40 * SECOND }],
            "proposer": { "name": "test", "rubric": "test", "version": "1" },
            "evidence": [{ "kind": "sentence", "index": 0 }],
            "roles": [],
            "boundary_lattice": {
                "starts": [8 * SECOND, 10 * SECOND, 12 * SECOND],
                "ends": [30 * SECOND, 40 * SECOND, 55 * SECOND],
                "phi_rejects": [],
            },
            "layout_requirements": [],
            "cluster_id": "cl_00000000000000a1",
            "prelim_score": 0.7,
            "exclusions": [],
        }],
        "clusters": [{
            "id": "cl_00000000000000a1",
            "representative": CANDIDATE,
            "members": [CANDIDATE],
            "similarity": 1.0,
        }],
    }))
    .expect("a candidate set")
}

fn ranking() -> RankingSet {
    serde_json::from_value(json!({
        "schema_version": "clipmill.ranking.set.v1",
        "source_fingerprint": FINGERPRINT,
        "inputs": {
            "candidates_artifact_id": ADDRESS,
            "index_artifact_id": ADDRESS,
            "transcript_artifact_id": ADDRESS,
        },
        "producer": { "stage": "rank-candidates", "implementation": "test@1" },
        "rubric": { "scorer": "test", "boundary": "test", "selector": "test" },
        "requested": { "count": 1, "diversity": 0.3 },
        "cohort": [{
            "candidate_id": CANDIDATE,
            "rank": 1,
            "display_score": 88,
            "score": 0.88,
            "factors": [{ "name": "hook", "available": true, "value": 0.9, "weight": 0.3, "evidence": [] }],
            "penalties": [],
            "uncertainty": { "value": 0.2, "band": "strong", "warnings": [] },
            "boundary": {
                "chosen": { "start_ticks": 10 * SECOND, "end_ticks": 40 * SECOND },
                "score": 0.8,
                "terms": [{ "name": "hook", "value": 0.4, "weight": 1.0 }],
                "alternative": {
                    "interval": { "start_ticks": 12 * SECOND, "end_ticks": 40 * SECOND },
                    "score": 0.7,
                },
                "considered": 9,
            },
            "cluster_id": "cl_00000000000000a1",
        }],
        "selected": [CANDIDATE],
        "shortfall": [],
        "filtered": [],
    }))
    .expect("a ranking")
}

fn request(cut: Cut) -> Request {
    Request {
        candidate_id: CANDIDATE.to_owned(),
        cut,
        style_ref: clipmill_captions::DEFAULT_STYLE_REF.to_owned(),
        frame: Frame {
            width: 1_920,
            height: 1_080,
        },
        aspect: Aspect::default(),
    }
}

fn evidence<'a>(
    candidates: &'a DiscoveryCandidates,
    ranking: &'a RankingSet,
    transcript: &'a SpeechTranscript,
) -> Evidence<'a> {
    Evidence {
        candidates,
        ranking,
        transcript,
        index: None,
        shots: None,
        faces: None,
    }
}

#[test]
fn the_same_candidate_directs_to_the_same_bytes_every_time() {
    let (candidates, ranking, transcript) = (candidates(), ranking(), transcript());
    let at = evidence(&candidates, &ranking, &transcript);

    let first = serde_json::to_string(&direct(at, &request(Cut::Chosen)).expect("a document"))
        .expect("json");
    let second = serde_json::to_string(&direct(at, &request(Cut::Chosen)).expect("a document"))
        .expect("json");

    assert_eq!(first, second, "assembling an edit must not be a judgement");
}

#[test]
fn the_document_carries_the_span_the_ranking_chose() {
    let (candidates, ranking, transcript) = (candidates(), ranking(), transcript());
    let document = direct(
        evidence(&candidates, &ranking, &transcript),
        &request(Cut::Chosen),
    )
    .expect("a document");

    let segment = &document.video.segments[0];
    assert_eq!(segment.in_ticks, 10 * 90_000);
    assert_eq!(segment.out_ticks, 40 * 90_000);
    assert_eq!(segment.source_fingerprint, FINGERPRINT);
    assert_eq!(document.video.segments.len(), 1);
}

#[test]
fn taking_the_runner_up_produces_the_runner_ups_document() {
    let (candidates, ranking, transcript) = (candidates(), ranking(), transcript());
    let at = evidence(&candidates, &ranking, &transcript);

    let chosen = direct(at, &request(Cut::Chosen)).expect("a document");
    let alternative = direct(at, &request(Cut::Alternative)).expect("a document");

    assert_eq!(alternative.video.segments[0].in_ticks, 12 * 90_000);
    assert_ne!(
        serde_json::to_string(&chosen).unwrap(),
        serde_json::to_string(&alternative).unwrap(),
    );
    // The swap is a different cut of one clip, not a different clip.
    assert_eq!(
        chosen.rationale.as_ref().unwrap().candidate_id,
        alternative.rationale.as_ref().unwrap().candidate_id,
    );
}

#[test]
fn a_boundary_the_lattice_does_not_offer_is_refused_rather_than_rounded() {
    let (candidates, ranking, transcript) = (candidates(), ranking(), transcript());
    let at = evidence(&candidates, &ranking, &transcript);

    // Half a second off a legal start is exactly the mid-word cut the boundary
    // optimizer exists to avoid.
    let error = direct(
        at,
        &request(Cut::Exact(Boundary {
            start_ticks: 10 * 90_000 + 45_000,
            end_ticks: 40 * 90_000,
        })),
    )
    .expect_err("an illegal boundary");

    assert!(error.to_string().contains("lattice"), "{error}");
}

#[test]
fn a_clip_nobody_looked_for_faces_in_is_fitted_and_says_so() {
    let (candidates, ranking, transcript) = (candidates(), ranking(), transcript());
    let document = direct(
        evidence(&candidates, &ranking, &transcript),
        &request(Cut::Chosen),
    )
    .expect("a document");

    assert_eq!(
        document.video.segments[0].layout.state,
        clipmill_edit_ir::LayoutState::Fit
    );
    let decisions = &document.rationale.as_ref().expect("a rationale").decisions;
    assert!(
        decisions
            .iter()
            .any(|line| line.contains("nothing looked for faces")),
        "a fitted clip owes a reason: {decisions:?}"
    );
}

#[test]
fn both_caption_groupings_reach_the_document_and_hold_the_same_words() {
    let (candidates, ranking, transcript) = (candidates(), ranking(), transcript());
    let document = direct(
        evidence(&candidates, &ranking, &transcript),
        &request(Cut::Chosen),
    )
    .expect("a document");

    let words = |cues: &[clipmill_edit_ir::CaptionCue]| {
        cues.iter()
            .flat_map(|cue| cue.lines.iter())
            .flat_map(|line| line.words.iter())
            .map(|word| word.text.clone())
            .collect::<Vec<_>>()
    };
    assert!(
        !document.captions.cues.is_empty(),
        "the clip has speech in it"
    );
    assert!(!document.captions.burn_in.is_empty());
    assert_eq!(
        words(&document.captions.cues),
        words(&document.captions.burn_in),
        "the two groupings must never disagree about the words",
    );
    assert!(document.captions.burn_in.len() > document.captions.cues.len());
}

#[test]
fn the_document_the_director_produced_is_one_the_ir_accepts() {
    let (candidates, ranking, transcript) = (candidates(), ranking(), transcript());
    let document = direct(
        evidence(&candidates, &ranking, &transcript),
        &request(Cut::Chosen),
    )
    .expect("a document");

    document.validate().expect("a valid edit document");
}
