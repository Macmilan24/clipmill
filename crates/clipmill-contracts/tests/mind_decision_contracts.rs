//! The editorial decision artifacts stay readable with the original clip-plan contract.

use clipmill_contracts::schemas::{
    mind_clip_plan::MindClipPlan, mind_plans::MindPlans, mind_quality_report::MindQualityReport,
    ranking_set_v2::RankingSetV2,
};

#[test]
fn a_reviewed_plan_roundtrips_from_its_draft_to_the_results_board() -> Result<(), serde_json::Error>
{
    let drafts: MindPlans = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/mind.plans/valid/stitch.json"
    ))?;
    let report: MindQualityReport = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/mind.quality_report/valid/repaired_stitch.json"
    ))?;
    let ranking: RankingSetV2 = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/ranking.set.v2/valid/single_accepted.json"
    ))?;
    assert_eq!(drafts.plans[0].plan, report.reviews[0].plan);
    assert_eq!(report.reviews[0].plan, ranking.clips[0].plan);
    let plan: MindClipPlan = serde_json::from_value(serde_json::to_value(&ranking.clips[0].plan)?)?;
    assert_eq!(drafts.plans[0].candidate_id.as_str(), plan.plan_id.as_str());
    assert!(ranking.clips[0].quality.repaired);
    Ok(())
}

#[test]
fn a_valid_empty_ranking_preserves_the_cutoff_reason() -> Result<(), serde_json::Error> {
    let ranking: RankingSetV2 = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/ranking.set.v2/valid/empty.json"
    ))?;
    assert!(ranking.clips.is_empty());
    assert!(ranking.cutoff_reason.is_some());
    Ok(())
}

#[test]
fn judgments_keep_the_self_check_beside_the_verdicts_it_allows() -> Result<(), serde_json::Error> {
    use clipmill_contracts::schemas::mind_judgments::MindJudgments;
    let text = include_str!("../../../contracts/fixtures/mind.judgments/valid/minimal.json");
    let judgments: MindJudgments = serde_json::from_str(text)?;
    assert!(judgments.self_check.endings.passed);
    assert!(!judgments.self_check.removals.passed);
    assert_eq!(judgments.endings.len(), 2);
    let back: serde_json::Value = serde_json::to_value(&judgments)?;
    assert_eq!(back, serde_json::from_str::<serde_json::Value>(text)?);
    Ok(())
}
