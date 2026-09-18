//! The editorial contracts, Rust leg (plan, Milestone 2).
//!
//! Five documents around one model: the windows it reads, the proposals it
//! makes, the judgments it passes, the looks it takes, and the trace of every
//! call. What these assert is the shape a consumer codes against and the
//! things the documents may never lose: a proposal cites sentences and never
//! carries a timestamp; a judgment is a status with reasons and never a
//! score; every window is ranges into one sentence list and together they
//! cover it; a trace carries no credential.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::schemas::{
    editorial_judgments::{EditorialJudgments, JudgmentOutcome, JudgmentStatus},
    editorial_looks::EditorialLooks,
    editorial_proposals::{EditorialProposals, WindowAnswerStatus},
    editorial_trace::EditorialTrace,
    editorial_windows::EditorialWindows,
};
use serde::{Serialize, de::DeserializeOwned};

fn read(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => panic!("cannot read {}: {err}", path.display()),
    }
}

fn canonical(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_else(|err| panic!("{err}"));
    text.push('\n');
    text
}

fn roundtrip<T: DeserializeOwned + Serialize>(rel: &str) -> T {
    let raw = read(rel);
    let parsed: T = match serde_json::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(err) => panic!("valid fixture {rel} rejected: {err}"),
    };
    let reserialized =
        serde_json::to_value(&parsed).unwrap_or_else(|err| panic!("reserialize {rel}: {err}"));
    assert_eq!(
        canonical(&reserialized),
        raw,
        "canonical round-trip must be byte-identical for {rel}"
    );
    parsed
}

fn refused<T: DeserializeOwned>(rel: &str) {
    let raw = read(rel);
    assert!(
        serde_json::from_str::<T>(&raw).is_err(),
        "invalid fixture {rel} was accepted"
    );
}

#[test]
fn every_valid_fixture_roundtrips_canonically() {
    roundtrip::<EditorialWindows>("contracts/fixtures/editorial.windows/valid/ten_words.json");
    roundtrip::<EditorialWindows>("contracts/fixtures/editorial.windows/valid/talk.json");
    roundtrip::<EditorialProposals>("contracts/fixtures/editorial.proposals/valid/talk.json");
    roundtrip::<EditorialProposals>(
        "contracts/fixtures/editorial.proposals/valid/cloud_route.json",
    );
    roundtrip::<EditorialJudgments>("contracts/fixtures/editorial.judgments/valid/talk.json");
    roundtrip::<EditorialLooks>("contracts/fixtures/editorial.looks/valid/talk.json");
    roundtrip::<EditorialTrace>("contracts/fixtures/editorial.trace/valid/local.json");
    roundtrip::<EditorialTrace>("contracts/fixtures/editorial.trace/valid/cloud.json");
}

/// The refusals the generated types make on their own: an unknown enum
/// value, a missing required field, a field the schema never named — the
/// last one being how a timestamp, a score, or a key would get in.
#[test]
fn every_invalid_fixture_is_refused() {
    for name in ["unknown_terminator", "no_budget", "window_with_text"] {
        refused::<EditorialWindows>(&format!(
            "contracts/fixtures/editorial.windows/invalid/{name}.json"
        ));
    }
    for name in [
        "bad_id",
        "unknown_status",
        "proposal_with_timestamp",
        "no_route",
    ] {
        refused::<EditorialProposals>(&format!(
            "contracts/fixtures/editorial.proposals/invalid/{name}.json"
        ));
    }
    for name in ["unknown_status", "with_a_score", "unknown_reason"] {
        refused::<EditorialJudgments>(&format!(
            "contracts/fixtures/editorial.judgments/invalid/{name}.json"
        ));
    }
    refused::<EditorialLooks>("contracts/fixtures/editorial.looks/invalid/unknown_answer.json");
    for name in ["unknown_outcome", "credential_in_call"] {
        refused::<EditorialTrace>(&format!(
            "contracts/fixtures/editorial.trace/invalid/{name}.json"
        ));
    }
}

/// Windows are ranges into one sentence list, and together they cover it:
/// every sentence is inside at least one core, consecutive windows overlap,
/// and a window's ticks and word count are what its sentences say.
#[test]
fn windows_cover_every_sentence_and_overlap_on_the_seams() {
    let windows: EditorialWindows =
        roundtrip("contracts/fixtures/editorial.windows/valid/talk.json");
    let sentences = &windows.sentences;
    for (index, sentence) in sentences.iter().enumerate() {
        assert_eq!(
            sentence.index, index as u64,
            "sentences are listed in order, once"
        );
    }
    let mut covered = vec![false; sentences.len()];
    let mut previous_end: Option<u64> = None;
    for window in &windows.windows {
        let first = usize::try_from(window.first_sentence_index).unwrap();
        let count = usize::try_from(window.sentence_count.get()).unwrap();
        assert!(
            first + count <= sentences.len(),
            "a window past the sentences"
        );
        let core = &sentences[first..first + count];
        assert_eq!(window.start_ticks, core[0].start_ticks);
        assert_eq!(window.end_ticks, core[count - 1].end_ticks);
        assert_eq!(
            window.word_count.get(),
            core.iter()
                .map(|sentence| sentence.word_count.get())
                .sum::<u64>()
        );
        assert!(
            u64::try_from(first).unwrap() >= window.context_before_sentences,
            "context before the first sentence would name sentences that do not exist"
        );
        if let Some(end) = previous_end {
            assert!(
                window.first_sentence_index < end,
                "window {} begins after the previous one ended: no overlap",
                window.index
            );
        }
        previous_end = Some(window.first_sentence_index + window.sentence_count.get());
        for flag in &mut covered[first..first + count] {
            *flag = true;
        }
    }
    assert!(
        covered.iter().all(|flag| *flag),
        "a sentence is in no window"
    );
    for line in &windows.outline {
        let first = usize::try_from(line.first_sentence_index).unwrap();
        let count = usize::try_from(line.sentence_count.get()).unwrap();
        assert!(
            first + count <= sentences.len(),
            "an outline line past the sentences"
        );
        assert!(!line.keywords.is_empty());
    }
}

/// A proposal is sentences, never a timestamp, and a window the model could
/// not answer is a failure with a class — not a window with nothing in it.
#[test]
fn proposals_cite_sentences_and_a_failure_is_not_an_empty_answer() {
    let proposals: EditorialProposals =
        roundtrip("contracts/fixtures/editorial.proposals/valid/talk.json");
    let answered = proposals
        .windows
        .iter()
        .filter(|window| window.status == WindowAnswerStatus::Answered)
        .count();
    assert_eq!(answered, 2);
    for window in &proposals.windows {
        match window.status {
            WindowAnswerStatus::Answered => assert!(!window.proposals.is_empty()),
            WindowAnswerStatus::None => assert!(window.proposals.is_empty()),
            WindowAnswerStatus::Malformed | WindowAnswerStatus::Failed => {
                assert!(window.proposals.is_empty());
                assert!(window.failure.is_some(), "a failure says what happened");
            }
        }
        for proposal in &window.proposals {
            assert!(proposal.sentence_count.get() >= 1);
            assert_eq!(
                proposal.id.to_string(),
                format!(
                    "prop_{}_{}",
                    window.window_index,
                    proposal.id.to_string().rsplit('_').next().unwrap()
                ),
                "a proposal id names its window"
            );
            if proposal.splittable {
                assert!(
                    !proposal.subspans.is_empty(),
                    "a splittable proposal names its parts"
                );
            }
        }
    }
    // The cloud route names its provider and carries no digest; the local
    // route the other way round.
    let cloud: EditorialProposals =
        roundtrip("contracts/fixtures/editorial.proposals/valid/cloud_route.json");
    assert!(cloud.producer.model.provider.is_some());
    assert!(cloud.producer.model.digest.is_none());
    assert!(proposals.producer.model.digest.is_some());
    assert!(proposals.producer.model.provider.is_none());
}

/// A judgment is a status and reasons, and only an answered one has them.
#[test]
fn a_judgment_is_a_status_with_reasons_and_never_a_number() {
    let judgments: EditorialJudgments =
        roundtrip("contracts/fixtures/editorial.judgments/valid/talk.json");
    let mut statuses = Vec::new();
    for judgment in &judgments.candidates {
        if judgment.outcome == JudgmentOutcome::Answered {
            let status = judgment.status.expect("an answered judgment has a status");
            statuses.push(status);
            if status != JudgmentStatus::Accepted {
                assert!(
                    !judgment.reasons.is_empty(),
                    "a status short of accepted says why"
                );
            }
        } else {
            assert!(judgment.status.is_none());
            assert!(judgment.failure.is_some());
        }
    }
    assert_eq!(
        statuses,
        [
            JudgmentStatus::Accepted,
            JudgmentStatus::NeedsReview,
            JudgmentStatus::Rejected
        ]
    );
}

/// A look names the frames it was answered from, and a trace names what
/// every call cost in tokens — and on the cloud route, in money against the cap.
#[test]
fn looks_name_their_frames_and_traces_account_for_every_call() {
    let looks: EditorialLooks = roundtrip("contracts/fixtures/editorial.looks/valid/talk.json");
    for check in &looks.checks {
        assert!(!check.frames.is_empty());
        assert!(!check.question.is_empty());
    }
    let local: EditorialTrace = roundtrip("contracts/fixtures/editorial.trace/valid/local.json");
    assert!(local.budget.is_none(), "the local route spends nothing");
    let cloud: EditorialTrace = roundtrip("contracts/fixtures/editorial.trace/valid/cloud.json");
    let budget = cloud.budget.expect("the cloud route has a cap");
    let spent: u64 = cloud
        .calls
        .iter()
        .filter_map(|call| call.cost_micro_usd)
        .sum();
    assert!(spent <= budget.cap_micro_usd);
    assert_eq!(spent, budget.spent_micro_usd);
}
