#![allow(clippy::unwrap_used)]
use clipmill_contracts::schemas::{
    editorial_judgments::EditorialJudgments, ranking_set::RankingSet,
};
use clipmill_editorial::review;
use serde_json::{Value, json};
const ID: &str = "sha256:e01a000000000000000000000000000000000000000000000000000000000031";
fn fixture() -> (RankingSet, Value) {
    let r: Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/ranking.set/valid/interview.json"
    ))
    .unwrap();
    let mut j: Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/editorial.judgments/valid/talk.json"
    ))
    .unwrap();
    j["source_fingerprint"] = r["source_fingerprint"].clone();
    j["inputs"]["candidates_artifact_id"] = r["inputs"]["candidates_artifact_id"].clone();
    j["candidates"]=json!(r["cohort"].as_array().unwrap().iter().chain(r["filtered"].as_array().into_iter().flatten()).map(|row|json!({
 "candidate_id":row["candidate_id"],"outcome":"answered","status":"accepted","context":{"first_sentence_index":0,"sentence_count":14},"reasons":[],"visual_dependency":false,"summary":"A complete moment"})).collect::<Vec<_>>());
    (serde_json::from_value(r).unwrap(), j)
}
#[test]
fn rejected_candidates_cannot_be_selected() {
    let (r, mut j) = fixture();
    for row in j["candidates"].as_array_mut().unwrap() {
        row["status"] = json!("rejected");
        row["reasons"] = json!([{"code":"incomplete_payoff","detail":"The answer never finishes"}]);
    }
    let j: EditorialJudgments = serde_json::from_value(j).unwrap();
    let result = review::apply(r, &j, ID, 14, &[]).unwrap();
    assert!(result.selected.is_empty());
    assert!(result.cohort.is_empty());
    assert!(!result.shortfall.is_empty());
}
#[test]
fn missing_one_review_keeps_other_candidates_and_reports_partial_coverage() {
    let (r, mut j) = fixture();
    let missing = j["candidates"].as_array_mut().unwrap().pop().unwrap()["candidate_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let result = review::apply(r, &serde_json::from_value(j).unwrap(), ID, 14, &[]).unwrap();
    assert_eq!(result.editorial.as_ref().unwrap().failed_reviews, 1);
    assert!(!result.selected.is_empty());
    assert!(
        !result
            .cohort
            .iter()
            .any(|r| r.candidate_id.as_str() == missing)
    );
    assert!(
        result
            .filtered
            .iter()
            .any(|r| r.candidate_id.as_str() == missing)
    );
}
#[test]
fn invented_and_duplicate_reviews_remain_hard_failures() {
    let (r, mut j) = fixture();
    j["candidates"][0]["candidate_id"] = json!("cand_ffffffffffffffff");
    assert!(review::apply(r, &serde_json::from_value(j).unwrap(), ID, 14, &[]).is_err());
    let (r, mut j) = fixture();
    let duplicate = j["candidates"][0].clone();
    j["candidates"].as_array_mut().unwrap().push(duplicate);
    assert!(review::apply(r, &serde_json::from_value(j).unwrap(), ID, 14, &[]).is_err());
}
#[test]
fn visual_dependency_never_silently_becomes_ready() {
    let (r, mut j) = fixture();
    for row in j["candidates"].as_array_mut().unwrap() {
        row["visual_dependency"] = json!(true);
    }
    let result = review::apply(r, &serde_json::from_value(j).unwrap(), ID, 14, &[]).unwrap();
    assert!(!result.cohort.is_empty());
    for row in result.cohort {
        let value = serde_json::to_value(row).unwrap();
        assert_eq!(value["review"]["status"], "needs_review");
    }
}
#[test]
fn bad_context_or_failed_review_is_excluded_without_discarding_other_moments() {
    let (r, mut j) = fixture();
    j["candidates"][0]["context"]["first_sentence_index"] = json!(9999);
    j["candidates"][1]["outcome"] = json!("failed");
    j["candidates"][1]["failure"] = json!({"failure_class":"transient","detail":"unavailable"});
    let result = review::apply(r, &serde_json::from_value(j).unwrap(), ID, 14, &[]).unwrap();
    assert_eq!(result.editorial.as_ref().unwrap().failed_reviews, 2);
    assert_eq!(result.cohort.len(), 8);
    assert_eq!(result.filtered.len(), 2);
    assert!(!result.selected.is_empty());
}
#[test]
fn no_usable_reviews_is_a_failure_instead_of_an_empty_success() {
    let (r, mut j) = fixture();
    j["candidates"] = json!([]);
    assert!(review::apply(r, &serde_json::from_value(j).unwrap(), ID, 14, &[]).is_err());
    let (r, mut j) = fixture();
    for answer in j["candidates"].as_array_mut().unwrap() {
        answer["outcome"] = json!("failed");
        answer["failure"] = json!({"failure_class":"transient","detail":"unavailable"});
    }
    assert!(
        review::apply(r, &serde_json::from_value(j).unwrap(), ID, 14, &[])
            .unwrap_err()
            .to_string()
            .contains("unavailable")
    );
}

#[test]
fn zero_candidates_is_a_valid_successful_review() {
    let (ranking, mut judgments) = fixture();
    let mut value = serde_json::to_value(ranking).unwrap();
    for key in ["cohort", "filtered", "selected", "shortfall"] {
        value[key] = json!([]);
    }
    judgments["candidates"] = json!([]);
    let result = review::apply(
        serde_json::from_value(value).unwrap(),
        &serde_json::from_value(judgments).unwrap(),
        ID,
        14,
        &[],
    )
    .unwrap();
    assert!(result.cohort.is_empty());
    assert!(result.selected.is_empty());
    assert!(!result.shortfall.is_empty());
}

#[test]
fn interpolated_interior_caption_timing_stays_visible_after_semantic_acceptance() {
    let (r, j) = fixture();
    let result = review::apply(
        r,
        &serde_json::from_value(j).unwrap(),
        ID,
        14,
        &[(0, u64::MAX)],
    )
    .unwrap();
    assert!(!result.cohort.is_empty());
    for row in result.cohort {
        let value = serde_json::to_value(row).unwrap();
        assert_eq!(value["review"]["status"], "needs_review");
        assert!(
            value["review"]["reasons"]
                .to_string()
                .contains("interpolated")
        );
    }
}

#[test]
fn missing_visual_checks_are_counted_and_never_upgrade_a_clip() {
    let (r, mut j) = fixture();
    for row in j["candidates"].as_array_mut().unwrap() {
        row["visual_dependency"] = json!(true);
    }
    let judgments: EditorialJudgments = serde_json::from_value(j).unwrap();
    let ranking = review::apply(r, &judgments, ID, 14, &[]).unwrap();
    let mut looks: Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/editorial.looks/valid/talk.json"
    ))
    .unwrap();
    looks["source_fingerprint"] = json!(ranking.source_fingerprint.as_str());
    looks["inputs"]["judgments_artifact_id"] = json!(ID);
    looks["checks"] = json!([]);
    let result =
        review::apply_looks(ranking, &serde_json::from_value(looks).unwrap(), &judgments).unwrap();
    assert_eq!(result.editorial.as_ref().unwrap().failed_visual_checks, 10);
    assert!(
        result
            .cohort
            .iter()
            .all(|r| r.review.as_ref().unwrap().status
                == clipmill_contracts::schemas::ranking_set::RankedReviewStatus::NeedsReview)
    );
}

#[test]
fn contradictory_accepted_verdicts_stay_needs_review() {
    let (r, mut j) = fixture();
    j["candidates"][0]["reasons"] =
        json!([{"code":"missing_question","detail":"The question is absent."}]);
    j["candidates"][1]["reasons"] =
        json!([{"code":"visual_dependency","detail":"Requires seeing the diagram."}]);
    let flagged: Vec<_> = j["candidates"].as_array().unwrap()[..2]
        .iter()
        .map(|r| r["candidate_id"].as_str().unwrap().to_owned())
        .collect();
    let judgments: EditorialJudgments = serde_json::from_value(j).unwrap();
    let result = review::apply(r, &judgments, ID, 14, &[]).unwrap();
    for row in result
        .cohort
        .iter()
        .filter(|r| flagged.iter().any(|id| r.candidate_id.as_str() == id))
    {
        assert_eq!(
            row.review.as_ref().unwrap().status,
            clipmill_contracts::schemas::ranking_set::RankedReviewStatus::NeedsReview
        );
    }
    let mut looks: Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/editorial.looks/valid/talk.json"
    ))
    .unwrap();
    looks["source_fingerprint"] = json!(result.source_fingerprint.as_str());
    looks["inputs"]["judgments_artifact_id"] = json!(ID);
    looks["checks"] = json!([]);
    let result =
        review::apply_looks(result, &serde_json::from_value(looks).unwrap(), &judgments).unwrap();
    assert_eq!(result.editorial.as_ref().unwrap().failed_visual_checks, 1);
}

#[test]
fn declines_preserve_the_exact_span_and_review_for_human_inspection() {
    let (mut ranking, mut judgments) = fixture();
    let original = ranking.cohort[0].clone();
    ranking.cohort[0].title = Some("The unfinished explanation".parse().unwrap());
    judgments["candidates"][0]["status"] = json!("rejected");
    judgments["candidates"][0]["reasons"] =
        json!([{"code":"incomplete_payoff","detail":"The next sentence contains the answer"}]);
    let result = review::apply(
        ranking,
        &serde_json::from_value(judgments).unwrap(),
        ID,
        14,
        &[],
    )
    .unwrap();
    assert_eq!(result.declined.len(), 1);
    let declined = &result.declined[0];
    assert_eq!(declined.candidate_id, original.candidate_id);
    assert_eq!(
        serde_json::to_value(&declined.boundary).unwrap(),
        serde_json::to_value(&original.boundary).unwrap()
    );
    assert_eq!(
        declined.title.as_ref().unwrap().as_str(),
        "The unfinished explanation"
    );
    assert_eq!(
        declined.review.as_ref().unwrap().status,
        clipmill_contracts::schemas::ranking_set::RankedReviewStatus::Rejected
    );
    assert!(declined.review.as_ref().unwrap().reasons[0].contains("next sentence"));
    assert!(
        !result
            .cohort
            .iter()
            .any(|row| row.candidate_id == original.candidate_id)
    );
    assert!(
        !result
            .selected
            .iter()
            .any(|id| id.as_str() == original.candidate_id.as_str())
    );
}

#[test]
fn uncertainty_alone_is_not_a_semantic_rejection() {
    for code in ["visual_dependency", "transcript_uncertain"] {
        let (ranking, mut judgments) = fixture();
        judgments["candidates"][0]["status"] = json!("rejected");
        judgments["candidates"][0]["reasons"] =
            json!([{"code":code,"detail":"Evidence is not clear enough"}]);
        let result = review::apply(
            ranking,
            &serde_json::from_value(judgments).unwrap(),
            ID,
            14,
            &[],
        )
        .unwrap();
        assert!(result.declined.is_empty());
        assert_eq!(result.cohort.len(), 10);
        assert!(
            result
                .cohort
                .iter()
                .any(|r| r.review.as_ref().unwrap().status
                    == clipmill_contracts::schemas::ranking_set::RankedReviewStatus::NeedsReview)
        );
    }
}

#[test]
fn a_review_for_another_content_profile_is_refused() {
    let (ranking, mut judgments) = fixture();
    judgments["content_profile"] = json!("scripted");
    let error = review::apply(
        ranking,
        &serde_json::from_value(judgments).unwrap(),
        ID,
        14,
        &[],
    )
    .unwrap_err();
    assert!(error.to_string().contains("content profile"));
}

#[test]
fn a_visual_answer_attaches_to_a_decline_without_overriding_missing_payoff() {
    let (ranking, mut judgments) = fixture();
    let candidate_id = ranking.cohort[0].candidate_id.to_string();
    judgments["candidates"][0]["status"] = json!("rejected");
    judgments["candidates"][0]["visual_dependency"] = json!(true);
    judgments["candidates"][0]["reasons"] =
        json!([{"code":"incomplete_payoff","detail":"The answer was cut off"}]);
    let judgments: EditorialJudgments = serde_json::from_value(judgments).unwrap();
    let ranking = review::apply(ranking, &judgments, ID, 14, &[]).unwrap();
    let mut looks: Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/editorial.looks/valid/talk.json"
    ))
    .unwrap();
    looks["source_fingerprint"] = json!(ranking.source_fingerprint.as_str());
    looks["inputs"]["judgments_artifact_id"] = json!(ID);
    looks["checks"] = json!([{"candidate_id":candidate_id,"outcome":"answered","answer":"yes","detail":"The diagram is visible","frames":[{"t_ticks":0}],"question":"Is the diagram visible?"}]);
    let result =
        review::apply_looks(ranking, &serde_json::from_value(looks).unwrap(), &judgments).unwrap();
    assert_eq!(result.declined.len(), 1);
    let review = result.declined[0].review.as_ref().unwrap();
    assert_eq!(
        review.status,
        clipmill_contracts::schemas::ranking_set::RankedReviewStatus::Rejected
    );
    assert!(
        review
            .reasons
            .iter()
            .any(|reason| reason.contains("diagram is visible"))
    );
}
