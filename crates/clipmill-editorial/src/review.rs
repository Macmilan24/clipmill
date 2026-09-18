//! Apply typed semantic judgments without presenting unreviewed clips as ready.
use crate::validate::Error;
use clipmill_contracts::schemas::{
    editorial_judgments::{
        EditorialJudgments, Judgment, JudgmentOutcome, JudgmentStatus, ReasonCode,
    },
    editorial_looks::{CheckOutcome, EditorialLooks},
    ranking_set::{self as ranked, RankingSet},
};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

#[allow(
    clippy::too_many_lines,
    reason = "validate each review and publish its explicit outcome"
)]
pub fn apply(
    mut ranking: RankingSet,
    judgments: &EditorialJudgments,
    artifact_id: &str,
    sentence_count: usize,
    uncertain_regions: &[(u64, u64)],
) -> Result<RankingSet, Error> {
    if ranking.source_fingerprint.as_str() != judgments.source_fingerprint.as_str()
        || ranking.inputs.candidates_artifact_id.as_str()
            != judgments.inputs.candidates_artifact_id.as_str()
    {
        return Err(Error(
            "review belongs to different candidates or source".into(),
        ));
    }
    let all: BTreeSet<_> = ranking
        .cohort
        .iter()
        .map(|r| r.candidate_id.as_str().to_owned())
        .chain(
            ranking
                .filtered
                .iter()
                .map(|r| r.candidate_id.as_str().to_owned()),
        )
        .collect();
    let mut by_id = BTreeMap::new();
    let mut failures = BTreeMap::new();
    for answer in &judgments.candidates {
        let id = answer.candidate_id.as_str();
        if !all.contains(id) {
            return Err(Error("review invented candidates".into()));
        }
        if by_id.insert(id, answer).is_some() {
            return Err(Error("duplicate judgment".into()));
        }
        if let Some(reason) = unavailable(answer, sentence_count) {
            failures.insert(id, reason);
        }
    }
    for id in &all {
        if !by_id.contains_key(id.as_str()) {
            failures.insert(
                id.as_str(),
                "The model did not review this candidate.".to_owned(),
            );
        }
    }
    if !all.is_empty() && failures.len() == all.len() {
        return Err(Error(format!(
            "No candidate received a usable review: {}",
            failures
                .values()
                .next()
                .map_or("missing reviews", String::as_str)
        )));
    }
    // Record the incomplete pass even when enough other candidates were found.
    if !failures.is_empty() {
        ranking
            .editorial
            .get_or_insert_with(|| ranked::EditorialCoverage {
                window_count: 0,
                answered_windows: 0,
                failed_windows: Vec::new(),
                failed_reviews: 0,
                failed_visual_checks: 0,
            })
            .failed_reviews = u64::try_from(failures.len()).map_err(err)?;
    }
    let mut cohort = Vec::new();
    for mut row in std::mem::take(&mut ranking.cohort) {
        let id = row.candidate_id.as_str();
        if let Some(detail) = failures.get(id) {
            ranking.filtered.push(ranked::FilteredCandidate {
                candidate_id: row.candidate_id.clone(),
                reason: ranked::FilteredCandidateReason::ExcludedByDiscovery,
                detail: Some(parsed(&format!("Editorial review unavailable: {detail}"))?),
            });
            continue;
        }
        let answer = by_id
            .get(id)
            .ok_or_else(|| Error("missing review classification".into()))?;
        let mut reasons: Vec<String> = answer
            .reasons
            .iter()
            .map(|r| r.detail.to_string())
            .collect();
        if answer.status == Some(JudgmentStatus::Rejected) {
            ranking.filtered.push(ranked::FilteredCandidate {
                candidate_id: row.candidate_id.clone(),
                reason: ranked::FilteredCandidateReason::ExcludedByDiscovery,
                detail: Some(parsed(&format!(
                    "Editorial review: {}",
                    reasons.join("; ")
                ))?),
            });
            continue;
        }
        let mut status = if answer.status == Some(JudgmentStatus::Accepted) {
            ranked::RankedReviewStatus::Accepted
        } else {
            ranked::RankedReviewStatus::NeedsReview
        };
        if answer
            .reasons
            .iter()
            .any(|reason| reason.code != ReasonCode::Other)
        {
            status = ranked::RankedReviewStatus::NeedsReview;
        }
        if visually_dependent(answer) {
            status = ranked::RankedReviewStatus::NeedsReview;
            reasons.push("Understanding this clip depends on the picture; check the visual reference before export.".into());
        }
        let chosen = &row.boundary.chosen;
        if uncertain_regions
            .iter()
            .any(|(a, b)| *a < chosen.end_ticks && *b > chosen.start_ticks)
        {
            status = ranked::RankedReviewStatus::NeedsReview;
            reasons.push("Some caption word timings were interpolated or could not be aligned. Review caption timing before export; the clip boundaries are aligned.".into());
        }
        row.review = Some(ranked::RankedReview {
            status,
            reasons,
            route: parsed(&judgments.producer.route.to_string())?,
            summary: answer
                .summary
                .as_ref()
                .map(|summary| summary.as_str().to_owned()),
        });
        cohort.push(row);
    }
    cohort.sort_by_key(|r| {
        r.review
            .as_ref()
            .is_none_or(|v| v.status != ranked::RankedReviewStatus::Accepted)
    });
    let requested = usize::try_from(ranking.requested.count.get()).map_err(err)?;
    let mut selected = Vec::new();
    let mut intervals: Vec<(u64, u64)> = Vec::new();
    for (i, row) in cohort.iter_mut().enumerate() {
        row.rank = std::num::NonZeroU64::new(u64::try_from(i + 1).map_err(err)?)
            .ok_or_else(|| Error("invalid rank".into()))?;
        if selected.len() >= requested {
            continue;
        }
        let a = row.boundary.chosen.start_ticks;
        let b = row.boundary.chosen.end_ticks;
        if intervals.iter().any(|(s, e)| {
            u128::from(b.min(*e).saturating_sub(a.max(*s))) * 2
                >= u128::from(b.max(*e).saturating_sub(a.min(*s)))
        }) {
            continue;
        }
        intervals.push((a, b));
        selected.push(parsed(row.candidate_id.as_str())?);
    }
    ranking.shortfall = if selected.len() < requested {
        vec![ranked::ShortfallReason {
            reason: ranked::ShortfallReasonReason::CohortExhausted,
            count: std::num::NonZeroU64::new(
                u64::try_from(requested - selected.len()).map_err(err)?,
            )
            .ok_or_else(|| Error("invalid shortfall".into()))?,
            detail: Some(parsed(if failures.is_empty() {
                "Editorial review found fewer distinct suitable moments. No results were added to fill the requested count."
            } else {
                "Some candidates could not be reviewed. Only assessed moments are included; retry analysis to complete the review."
            })?),
        }]
    } else {
        Vec::new()
    };
    ranking.cohort = cohort;
    ranking.selected = selected;
    ranking.inputs.judgments_artifact_id = Some(parsed(artifact_id)?);
    Ok(ranking)
}

fn visually_dependent(answer: &Judgment) -> bool {
    answer.visual_dependency == Some(true)
        || answer
            .reasons
            .iter()
            .any(|reason| reason.code == ReasonCode::VisualDependency)
}

fn unavailable(answer: &Judgment, sentence_count: usize) -> Option<String> {
    if answer.outcome != JudgmentOutcome::Answered {
        return Some(
            answer
                .failure
                .as_ref()
                .map_or("Model did not answer.", |f| f.detail.as_str())
                .to_owned(),
        );
    }
    if answer.failure.is_some() || answer.status.is_none() || answer.visual_dependency.is_none() {
        return Some("The answer omitted a verdict or visual-dependency check.".into());
    }
    if answer
        .context
        .first_sentence_index
        .checked_add(answer.context.sentence_count.get())
        .is_none_or(|end| usize::try_from(end).map_or(true, |end| end > sentence_count))
    {
        return Some("The answer cites invalid transcript context.".into());
    }
    None
}

/// Attach bounded visual evidence. Missing or failed checks stay visible, and a
/// sparse frame answer never upgrades a visually dependent clip to ready.
pub fn apply_looks(
    mut ranking: RankingSet,
    looks: &EditorialLooks,
    judgments: &EditorialJudgments,
) -> Result<RankingSet, Error> {
    if looks.source_fingerprint.as_str() != ranking.source_fingerprint.as_str()
        || ranking
            .inputs
            .judgments_artifact_id
            .as_ref()
            .map(|id| id.as_str())
            != Some(looks.inputs.judgments_artifact_id.as_str())
    {
        return Err(Error("visual checks belong to a different review".into()));
    }
    let expected: BTreeSet<_> = judgments
        .candidates
        .iter()
        .filter(|j| {
            j.outcome == JudgmentOutcome::Answered
                && visually_dependent(j)
                && j.status != Some(JudgmentStatus::Rejected)
        })
        .map(|j| j.candidate_id.as_str())
        .collect();
    let mut checks = BTreeMap::new();
    for check in &looks.checks {
        let id = check.candidate_id.as_str();
        if !expected.contains(id) || checks.insert(id, check).is_some() {
            return Err(Error("unexpected or duplicate visual check".into()));
        }
    }
    let mut failed = 0;
    for id in expected {
        let check = checks.get(id);
        let complete = check.is_some_and(|c| {
            c.outcome == CheckOutcome::Answered
                && c.answer.is_some()
                && c.detail.is_some()
                && c.failure.is_none()
        });
        if !complete {
            failed += 1;
        }
        if let Some(review) = ranking
            .cohort
            .iter_mut()
            .find(|r| r.candidate_id.as_str() == id)
            .and_then(|r| r.review.as_mut())
        {
            review.status = ranked::RankedReviewStatus::NeedsReview;
            let detail = if complete {
                check
                    .and_then(|c| c.detail.as_ref())
                    .map_or("Inspect the visual reference.", |d| d.as_str())
            } else {
                check.and_then(|c| c.failure.as_ref()).map_or(
                    "No usable frame check was available; inspect the visual reference.",
                    |f| f.detail.as_str(),
                )
            };
            review.reasons.push(format!("Visual check: {detail}"));
        }
    }
    if failed > 0 {
        ranking
            .editorial
            .get_or_insert_with(|| ranked::EditorialCoverage {
                window_count: 0,
                answered_windows: 0,
                failed_windows: Vec::new(),
                failed_reviews: 0,
                failed_visual_checks: 0,
            })
            .failed_visual_checks = failed;
    }
    Ok(ranking)
}
fn parsed<T: FromStr>(value: &str) -> Result<T, Error>
where
    T::Err: std::fmt::Display,
{
    value.parse().map_err(err)
}
fn err(e: impl std::fmt::Display) -> Error {
    Error(e.to_string())
}
