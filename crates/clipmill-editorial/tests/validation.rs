#![allow(clippy::unwrap_used, clippy::expect_used)]
use clipmill_contracts::schemas::{
    editorial_proposals::EditorialProposals, editorial_windows::EditorialWindows,
    speech_transcript::SpeechTranscript,
};
use clipmill_editorial::validate;
use serde_json::{Value, json};
const ID: &str = "sha256:e01a000000000000000000000000000000000000000000000000000000000031";
fn fixture() -> (EditorialWindows, SpeechTranscript, Value) {
    let w: EditorialWindows = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/editorial.windows/valid/interview.json"
    ))
    .unwrap();
    let t: SpeechTranscript = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/speech.transcript/valid/interview.json"
    ))
    .unwrap();
    let mut p: Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/editorial.proposals/valid/talk.json"
    ))
    .unwrap();
    p["windows"] = json!([{"window_index":0,"status":"answered","proposals":[{
 "id":"prop_0_0","first_sentence_index":0,"sentence_count":4,"title":"A complete thought","hook":"Question","setup":"Context","payoff":"Answer","reason":"Sentences zero to three","uncertainties":[],"splittable":false}]}]);
    (w, t, p)
}
fn run(w: &EditorialWindows, t: &SpeechTranscript, p: Value) -> validate::Validated {
    let p: EditorialProposals = serde_json::from_value(p).unwrap();
    validate::proposals(w, t, &p, ID, 90_000, 90 * 90_000).unwrap()
}
#[test]
fn a_valid_span_is_bound_to_words_and_not_changed_by_ranking() {
    let (w, t, p) = fixture();
    let out = run(&w, &t, p);
    assert_eq!(out.candidates.candidates.len(), 1);
    let c = &out.candidates.candidates[0];
    assert_eq!(c.boundary_lattice.starts.len(), 1);
    assert_eq!(c.boundary_lattice.ends.len(), 1);
    assert_eq!(c.intervals[0].start_ticks, 0);
    assert!(c.intervals[0].end_ticks <= w.sentences[3].end_ticks + 13_500);
}
#[test]
fn content_profile_is_bound_to_proposals_and_preserved_in_candidates() {
    let (mut windows, transcript, mut proposals) = fixture();
    windows.content_profile =
        clipmill_contracts::schemas::editorial_windows::EditorialWindowsContentProfile::Scripted;
    let mismatch: EditorialProposals = serde_json::from_value(proposals.clone()).unwrap();
    assert!(
        validate::proposals(&windows, &transcript, &mismatch, ID, 90_000, 90 * 90_000)
            .unwrap_err()
            .to_string()
            .contains("content profile")
    );
    proposals["content_profile"] = json!("scripted");
    let output = run(&windows, &transcript, proposals);
    assert_eq!(
        serde_json::to_value(output.candidates).unwrap()["content_profile"],
        "scripted"
    );
}
#[test]
fn unknown_sentences_and_outside_window_spans_are_refused() {
    let (w, t, mut p) = fixture();
    p["windows"][0]["proposals"][0]["first_sentence_index"] = json!(999);
    let out = run(&w, &t, p);
    assert!(out.candidates.candidates.is_empty());
    assert!(
        out.report["rejected"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("outside window")
    );
}
#[test]
fn an_edge_word_must_belong_to_its_sentence() {
    let (w, t, mut p) = fixture();
    p["windows"][0]["proposals"][0]["start_word_index"] = json!(100);
    let out = run(&w, &t, p);
    assert!(out.candidates.candidates.is_empty());
    assert!(
        out.report["rejected"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("edge word")
    );
}
#[test]
fn duplicate_nominations_do_not_become_duplicate_clips() {
    let (w, t, mut p) = fixture();
    let mut duplicate = p["windows"][0]["proposals"][0].clone();
    duplicate["id"] = json!("prop_0_1");
    p["windows"][0]["proposals"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    let out = run(&w, &t, p);
    assert_eq!(out.candidates.candidates.len(), 1);
    assert!(out.report["rejected"].as_array().unwrap().is_empty());
    assert_eq!(out.report["merged"].as_array().unwrap().len(), 1);
    assert_eq!(
        out.candidates.candidates[0]
            .editorial
            .as_ref()
            .unwrap()
            .proposal_ids
            .len(),
        2
    );
}
#[test]
fn no_proposals_is_valid_but_missing_or_failed_windows_are_not() {
    let (w, t, mut p) = fixture();
    p["windows"][0]["status"] = json!("none");
    p["windows"][0]["proposals"] = json!([]);
    assert!(run(&w, &t, p.clone()).candidates.candidates.is_empty());
    p["windows"] = json!([]);
    let p = serde_json::from_value(p).unwrap();
    assert!(validate::proposals(&w, &t, &p, ID, 90_000, 90 * 90_000).is_err());
    let (_, _, mut p) = fixture();
    p["windows"][0]["status"] = json!("failed");
    p["windows"][0]["proposals"] = json!([]);
    p["windows"][0]["failure"] = json!({"failure_class":"transient","detail":"model unavailable"});
    assert!(
        validate::proposals(
            &w,
            &t,
            &serde_json::from_value(p).unwrap(),
            ID,
            90_000,
            90 * 90_000
        )
        .unwrap_err()
        .to_string()
        .contains("model unavailable")
    );
}
#[test]
fn duration_and_parent_subspan_limits_are_enforced() {
    let (w, t, mut p) = fixture();
    let proposal = &mut p["windows"][0]["proposals"][0];
    proposal["splittable"] = json!(true);
    proposal["subspans"] = json!([{"first_sentence_index":10,"sentence_count":1}]);
    let p = serde_json::from_value(p).unwrap();
    let out = validate::proposals(&w, &t, &p, ID, 90_000, 2 * 90_000).unwrap();
    assert!(out.candidates.candidates.is_empty());
    assert!(
        out.report["rejected"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("outside its parent")
    );
}
#[test]
fn a_proposal_cannot_cross_sources_or_windows_artifacts() {
    let (w, t, p) = fixture();
    let p = serde_json::from_value(p).unwrap();
    assert!(validate::proposals(&w, &t, &p, "sha256:wrong", 90_000, 90 * 90_000).is_err());
}

#[test]
fn interior_interpolation_allows_review_but_bad_boundaries_and_missing_speech_do_not() {
    let (w, t, p) = fixture();
    let mut value = serde_json::to_value(&w).unwrap();
    value["invalid_regions"] = json!([{"start_ticks":90000,"end_ticks":99000,"reason":"timing_interpolated","detail":"interpolated word"}]);
    let interior = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(run(&interior, &t, p.clone()).candidates.candidates.len(), 1);
    value["invalid_regions"][0]["reason"] = json!("no_audio");
    assert!(
        run(
            &serde_json::from_value(value.clone()).unwrap(),
            &t,
            p.clone()
        )
        .candidates
        .candidates
        .is_empty()
    );
    value["invalid_regions"][0]["reason"] = json!("timing_interpolated");
    value["invalid_regions"][0]["start_ticks"] = json!(0);
    assert!(
        run(&serde_json::from_value(value).unwrap(), &t, p)
            .candidates
            .candidates
            .is_empty()
    );
}

#[test]
fn a_short_unaligned_interior_does_not_discard_an_otherwise_aligned_moment() {
    let (windows, mut transcript, proposed) = fixture();
    let mut value = serde_json::to_value(windows).unwrap();
    // A 220 ms alignment gap inside a sixteen-second clip, matching the shape
    // of the real failure. Both words defining the clip's edges remain aligned.
    value["invalid_regions"] = json!([{
        "start_ticks":1_149_300,
        "end_ticks":1_169_100,
        "reason":"alignment_unavailable",
        "detail":"One interior utterance could not be aligned"
    }]);
    transcript.words[23].timing =
        clipmill_contracts::schemas::speech_transcript::WordTiming::Interpolated;
    let windows = serde_json::from_value(value).unwrap();
    let result = run(&windows, &transcript, proposed);
    assert_eq!(result.candidates.candidates.len(), 1);
    assert_eq!(result.report["rejected"], json!([]));
    let candidate = &result.candidates.candidates[0];
    assert_eq!(candidate.intervals[0].start_ticks, 0);
    assert!(candidate.intervals[0].end_ticks > 1_169_100);
    assert_eq!(
        candidate.editorial.as_ref().unwrap().title.as_str(),
        "A complete thought"
    );
}

#[test]
fn unavailable_cut_edges_and_missing_audio_are_still_refused() {
    for (start, end, reason) in [
        (0, 19_800, "alignment_unavailable"),
        (1_438_200, 1_458_000, "alignment_unavailable"),
        (1_149_300, 1_169_100, "no_audio"),
        (1_149_300, 1_169_100, "decode_failed"),
    ] {
        let (windows, transcript, proposed) = fixture();
        let mut value = serde_json::to_value(windows).unwrap();
        value["invalid_regions"] = json!([{
            "start_ticks":start, "end_ticks":end, "reason":reason,
            "detail":"Unavailable region"
        }]);
        let result = run(
            &serde_json::from_value(value).unwrap(),
            &transcript,
            proposed,
        );
        assert!(
            result.candidates.candidates.is_empty(),
            "accepted {reason} at {start}"
        );
        assert_eq!(result.report["rejected"].as_array().unwrap().len(), 1);
        assert!(
            result.report["rejected"][0]["reason"]
                .as_str()
                .unwrap()
                .contains(reason)
        );
    }
}

#[test]
fn partial_windows_keep_usable_proposals_and_report_missing_coverage() {
    let (w, t, mut p) = fixture();
    let mut v = serde_json::to_value(&w).unwrap();
    let mut extra = v["windows"][0].clone();
    extra["index"] = json!(1);
    v["windows"].as_array_mut().unwrap().push(extra.clone());
    extra["index"] = json!(2);
    v["windows"].as_array_mut().unwrap().push(extra);
    p["windows"].as_array_mut().unwrap().push(json!({"window_index":1,"status":"failed","proposals":[],"failure":{"failure_class":"transient","detail":"model unavailable"}}));
    let out = run(&serde_json::from_value(v).unwrap(), &t, p);
    assert_eq!(out.candidates.candidates.len(), 1);
    let coverage = out.candidates.editorial.as_ref().unwrap();
    assert_eq!(coverage.window_count, 3);
    assert_eq!(coverage.answered_windows, 1);
    assert_eq!(coverage.failed_windows.len(), 2);
    let moment = out.candidates.candidates[0].editorial.as_ref().unwrap();
    assert_eq!(moment.title.as_str(), "A complete thought");
    assert_eq!(moment.hook.as_str(), "Question");
    assert_eq!(moment.payoff.as_str(), "Answer");
}

#[test]
fn an_interpolated_edge_is_refused_even_without_an_invalid_region() {
    let (w, t, p) = fixture();
    let mut t = serde_json::to_value(t).unwrap();
    t["words"][0]["timing"] = json!("interpolated");
    let out = run(&w, &serde_json::from_value(t).unwrap(), p);
    assert!(out.candidates.candidates.is_empty());
    assert!(
        out.report["rejected"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("boundary word")
    );
}

#[test]
fn silence_padding_stops_before_unavailable_regions_on_both_sides() {
    let (windows, transcript, mut proposals) = fixture();
    proposals["windows"][0]["proposals"][0]["first_sentence_index"] = json!(1);
    proposals["windows"][0]["proposals"][0]["sentence_count"] = json!(3);
    let mut windows = serde_json::to_value(windows).unwrap();
    windows["invalid_regions"] = json!([
        {"start_ticks":250_000,"end_ticks":255_000,"reason":"no_audio","detail":"Gap before speech"},
        {"start_ticks":1_460_000,"end_ticks":1_470_000,"reason":"no_audio","detail":"Gap after speech"}
    ]);
    let out = run(
        &serde_json::from_value(windows).unwrap(),
        &transcript,
        proposals,
    );
    assert_eq!(out.candidates.candidates.len(), 1);
    let cut = &out.candidates.candidates[0].intervals[0];
    assert_eq!(cut.start_ticks, 255_000);
    assert_eq!(cut.end_ticks, 1_460_000);
}
