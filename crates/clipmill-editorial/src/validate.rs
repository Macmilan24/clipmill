//! Turn untrusted model references into bounded, word-backed clip candidates.
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use clipmill_contracts::schemas::{
    discovery_candidates::{self as candidate, DiscoveryCandidates},
    editorial_proposals::{EditorialProposals, Proposal, WindowAnswerStatus},
    editorial_windows::{EditorialWindows, InvalidRegionReason, Window},
    speech_transcript::{SpeechTranscript, WordTiming},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
#[error("editorial validation: {0}")]
pub struct Error(pub String);

#[derive(Debug)]
pub struct Validated {
    pub candidates: DiscoveryCandidates,
    /// Rejections and merged nominations remain inspectable beside candidates.json.
    pub report: Value,
}

#[derive(Clone, Copy, Debug)]
struct Span {
    first: u64,
    last: u64,
    start: u64,
    end: u64,
}

/// Keep semantic spans intact. Failures in one window do not discard the
/// windows the model did read; their incomplete coverage travels to Results.
#[allow(
    clippy::too_many_lines,
    reason = "validate references and record each outcome in one pass"
)]
pub fn proposals(
    windows: &EditorialWindows,
    transcript: &SpeechTranscript,
    proposed: &EditorialProposals,
    windows_id: &str,
    min_ticks: u64,
    max_ticks: u64,
) -> Result<Validated, Error> {
    if min_ticks == 0 || max_ticks < min_ticks {
        return Err(Error("invalid duration range".into()));
    }
    if windows.source_fingerprint.as_str() != transcript.source_fingerprint.as_str()
        || windows.source_fingerprint.as_str() != proposed.source_fingerprint.as_str()
        || proposed.inputs.windows_artifact_id.as_str() != windows_id
    {
        return Err(Error("source or windows artifact mismatch".into()));
    }
    let mut seen_windows = BTreeSet::new();
    let mut seen_ids = BTreeSet::new();
    let mut accepted: BTreeMap<(u64, u64), candidate::Candidate> = BTreeMap::new();
    let mut refused = Vec::new();
    let mut merged = Vec::new();
    let mut seeds = 0;
    let mut coverage = candidate::EditorialCoverage {
        window_count: u64::try_from(windows.windows.len()).map_err(err)?,
        answered_windows: 0,
        failed_windows: Vec::new(),
        failed_reviews: 0,
        failed_visual_checks: 0,
    };
    let provenance = candidate::Proposer {
        name: parsed("editorial")?,
        rubric: parsed(proposed.producer.prompt_version.as_str())?,
        version: parsed("2")?,
    };
    for answer in &proposed.windows {
        let wi = answer.window_index;
        let window = windows
            .windows
            .iter()
            .find(|w| w.index == wi)
            .ok_or_else(|| Error(format!("unknown window {wi}")))?;
        if !seen_windows.insert(wi) {
            return Err(Error(format!("duplicate window {wi}")));
        }
        match answer.status {
            WindowAnswerStatus::None if answer.proposals.is_empty() && answer.failure.is_none() => {
                coverage.answered_windows += 1;
                continue;
            }
            WindowAnswerStatus::Answered
                if !answer.proposals.is_empty() && answer.failure.is_none() =>
            {
                coverage.answered_windows += 1;
            }
            WindowAnswerStatus::Failed | WindowAnswerStatus::Malformed
                if answer.proposals.is_empty() =>
            {
                let detail = answer
                    .failure
                    .as_ref()
                    .map_or("Model did not answer this window.", |f| f.detail.as_str());
                coverage
                    .failed_windows
                    .push(candidate::EditorialCoverageFailedWindowsItem {
                        index: wi,
                        detail: parsed(detail)?,
                    });
                continue;
            }
            _ => return Err(Error(format!("window {wi}: status contradicts proposals"))),
        }
        for proposal in &answer.proposals {
            seeds += 1;
            if !seen_ids.insert(proposal.id.as_str()) {
                return Err(Error(format!(
                    "duplicate proposal id {}",
                    proposal.id.as_str()
                )));
            }
            let parent = span(
                proposal.first_sentence_index,
                proposal.sentence_count.get(),
                proposal.start_word_index,
                proposal.end_word_index,
                window,
                windows,
                transcript,
            );
            let spans = match parent {
                Ok(parent) if parent.end - parent.start > max_ticks && proposal.splittable => {
                    let mut parts: Vec<_> = proposal
                        .subspans
                        .iter()
                        .map(|sub| {
                            let part = span(
                                sub.first_sentence_index,
                                sub.sentence_count.get(),
                                None,
                                None,
                                window,
                                windows,
                                transcript,
                            )?;
                            if part.first < parent.first
                                || part.last > parent.last
                                || part.start < parent.start
                                || part.end > parent.end
                            {
                                Err(Error("subspan lies outside its parent".into()))
                            } else {
                                Ok(part)
                            }
                        })
                        .collect();
                    if parts.is_empty() {
                        parts.push(Err(Error("overlong proposal has no subspans".into())));
                    }
                    parts
                }
                other => vec![other],
            };
            for result in spans {
                let span =
                    match result.and_then(|s| cut(s, windows, transcript, min_ticks, max_ticks)) {
                        Ok(s) => s,
                        Err(e) => {
                            refused.push(json!({"proposal_id":proposal.id,"reason":e.0}));
                            continue;
                        }
                    };
                let duplicate = accepted
                    .keys()
                    .find(|(a, b)| {
                        let intersection = span.end.min(*b).saturating_sub(span.start.max(*a));
                        let union = span.end.max(*b) - span.start.min(*a);
                        u128::from(intersection) * 100 >= u128::from(union) * 85
                    })
                    .copied();
                if let Some(key) = duplicate {
                    if let Some(existing) =
                        accepted.get_mut(&key).and_then(|c| c.editorial.as_mut())
                    {
                        existing.proposal_ids.push(parsed(proposal.id.as_str())?);
                        for uncertainty in &proposal.uncertainties {
                            if !existing
                                .uncertainties
                                .iter()
                                .any(|v| v.as_str() == uncertainty.as_str())
                            {
                                existing.uncertainties.push(parsed(uncertainty.as_str())?);
                            }
                        }
                    }
                    merged.push(json!({"proposal_id":proposal.id,"reason":"duplicate across windows","start_ticks":key.0,"end_ticks":key.1}));
                    continue;
                }
                let digest = hex::encode(Sha256::digest(format!(
                    "editorial.v2:{}:{}:{}",
                    windows.source_fingerprint.as_str(),
                    span.start,
                    span.end
                )));
                let id = format!("cand_{}", &digest[..16]);
                let cluster = format!("cl_{}", &digest[..16]);
                accepted.insert(
                    (span.start, span.end),
                    candidate::Candidate {
                        id: parsed(&id)?,
                        cluster_id: parsed(&cluster)?,
                        intervals: vec![candidate::Interval {
                            start_ticks: span.start,
                            end_ticks: span.end,
                        }],
                        proposer: provenance.clone(),
                        evidence: (span.first..=span.last).map(reference).collect(),
                        roles: candidate::CandidateRoles {
                            hook: Some(reference(span.first)),
                            payoff: Some(reference(span.last)),
                        },
                        boundary_lattice: candidate::BoundaryLattice {
                            starts: vec![span.start],
                            ends: vec![span.end],
                            phi_rejects: Vec::new(),
                        },
                        layout_requirements: Vec::new(),
                        prelim_score: 0.5,
                        exclusions: Vec::new(),
                        editorial: Some(moment(proposal)?),
                    },
                );
            }
        }
    }
    for window in &windows.windows {
        if !seen_windows.contains(&window.index) {
            coverage
                .failed_windows
                .push(candidate::EditorialCoverageFailedWindowsItem {
                    index: window.index,
                    detail: parsed("Model omitted this window; it was not assessed.")?,
                });
        }
    }
    if coverage.window_count > 0 && coverage.answered_windows == 0 {
        return Err(Error(format!(
            "No editorial window was assessed: {}",
            coverage
                .failed_windows
                .first()
                .map_or("model omitted all windows", |w| w.detail.as_str())
        )));
    }
    let candidates: Vec<_> = accepted.into_values().collect();
    let clusters = candidates
        .iter()
        .map(|c| {
            Ok(candidate::Cluster {
                id: c.cluster_id.clone(),
                representative: parsed(c.id.as_str())?,
                members: vec![parsed(c.id.as_str())?],
                similarity: 1.0,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let report = json!({"rejected":refused,"merged":merged,"editorial":coverage});
    Ok(Validated {
        candidates: DiscoveryCandidates {
            schema_version: json!("clipmill.discovery.candidates.v1"),
            source_fingerprint: parsed(windows.source_fingerprint.as_str())?,
            inputs: candidate::DiscoveryCandidatesInputs {
                index_artifact_id: parsed(windows.inputs.index_artifact_id.as_str())?,
                transcript_artifact_id: parsed(windows.inputs.transcript_artifact_id.as_str())?,
                loudness_artifact_id: None,
            },
            producer: candidate::Producer {
                stage: parsed("editorial-validate")?,
                implementation: parsed("clipmill-editorial-validate@1.1.0")?,
            },
            coverage: candidate::Coverage {
                start_ticks: windows.coverage.start_ticks,
                end_ticks: windows.coverage.end_ticks,
                analyzed: windows.coverage.analyzed,
            },
            duration_target: candidate::DurationRange {
                min_ticks: std::num::NonZeroU64::new(min_ticks)
                    .ok_or_else(|| Error("zero minimum duration".into()))?,
                max_ticks: std::num::NonZeroU64::new(max_ticks)
                    .ok_or_else(|| Error("zero maximum duration".into()))?,
            },
            proposers: vec![candidate::ProposerRun {
                proposer: provenance,
                seeds,
                candidates: u64::try_from(candidates.len()).map_err(err)?,
                floor_applied: None,
            }],
            candidates,
            clusters,
            editorial: Some(coverage),
        },
        report,
    })
}

fn reference(index: u64) -> candidate::EvidenceReference {
    candidate::EvidenceReference {
        kind: candidate::EvidenceReferenceKind::Sentence,
        index,
    }
}
fn moment(p: &Proposal) -> Result<candidate::EditorialMoment, Error> {
    Ok(candidate::EditorialMoment {
        title: parsed(p.title.as_str())?,
        hook: parsed(p.hook.as_str())?,
        setup: parsed(p.setup.as_str())?,
        payoff: parsed(p.payoff.as_str())?,
        reason: parsed(p.reason.as_str())?,
        proposal_ids: vec![parsed(p.id.as_str())?],
        uncertainties: p
            .uncertainties
            .iter()
            .map(|v| parsed(v.as_str()))
            .collect::<Result<_, _>>()?,
    })
}

/// Resolve endpoint words only from the cited sentences, never model timestamps.
#[allow(
    clippy::too_many_arguments,
    reason = "the two optional endpoint refs share one validation path"
)]
fn span(
    first: u64,
    count: u64,
    start_word: Option<u64>,
    end_word: Option<u64>,
    window: &Window,
    windows: &EditorialWindows,
    transcript: &SpeechTranscript,
) -> Result<Span, Error> {
    let last = first
        .checked_add(count)
        .and_then(|v| v.checked_sub(1))
        .filter(|_| count > 0)
        .ok_or_else(|| Error("empty or overflowing span".into()))?;
    let core_end = window
        .first_sentence_index
        .checked_add(window.sentence_count.get())
        .ok_or_else(|| Error("window overflow".into()))?;
    if first < window.first_sentence_index || last >= core_end {
        return Err(Error("span lies outside window".into()));
    }
    let find = |index| {
        windows
            .sentences
            .iter()
            .find(|s| s.index == index)
            .ok_or_else(|| Error("unknown sentence".into()))
    };
    let a = find(first)?;
    let b = find(last)?;
    let ae = a
        .first_word_index
        .checked_add(a.word_count.get())
        .ok_or_else(|| Error("word range overflow".into()))?;
    let be = b
        .first_word_index
        .checked_add(b.word_count.get())
        .ok_or_else(|| Error("word range overflow".into()))?;
    let from = start_word.unwrap_or(a.first_word_index);
    let to = end_word.unwrap_or(be.saturating_sub(1));
    if from < a.first_word_index || from >= ae || to < b.first_word_index || to >= be || to < from {
        return Err(Error("edge word outside its sentence".into()));
    }
    let a = transcript
        .words
        .get(usize::try_from(from).map_err(err)?)
        .ok_or_else(|| Error("unknown word".into()))?;
    let b = transcript
        .words
        .get(usize::try_from(to).map_err(err)?)
        .ok_or_else(|| Error("unknown word".into()))?;
    if a.timing != WordTiming::Aligned || b.timing != WordTiming::Aligned {
        return Err(Error("clip boundary word has unverified timing".into()));
    }
    if b.end_ticks <= a.start_ticks {
        return Err(Error("empty or reversed timing".into()));
    }
    Ok(Span {
        first,
        last,
        start: a.start_ticks,
        end: b.end_ticks,
    })
}

fn uncertain(reason: InvalidRegionReason) -> bool {
    matches!(
        reason,
        InvalidRegionReason::TimingInterpolated | InvalidRegionReason::AlignmentUnavailable
    )
}

/// Unplaced words inside a clip need caption review, not a change of meaning or
/// a discarded moment. Missing recognition/audio, and unverified cut edges,
/// remain hard failures. Padding may never reach a new invalid region.
fn cut(
    mut span: Span,
    windows: &EditorialWindows,
    transcript: &SpeechTranscript,
    min_ticks: u64,
    max_ticks: u64,
) -> Result<Span, Error> {
    if span.end - span.start < min_ticks || span.end - span.start > max_ticks {
        return Err(Error("duration outside target".into()));
    }
    for region in &windows.invalid_regions {
        if region.start_ticks < span.end
            && region.end_ticks > span.start
            && (!uncertain(region.reason)
                || region.start_ticks <= span.start
                || region.end_ticks >= span.end)
        {
            return Err(Error(format!(
                "{} at {:.2}–{:.2}s overlaps the clip or its cut boundary",
                region.reason,
                ticks_seconds(region.start_ticks),
                ticks_seconds(region.end_ticks)
            )));
        }
    }
    let previous = transcript
        .words
        .iter()
        .map(|w| w.end_ticks)
        .filter(|v| *v <= span.start)
        .max()
        .unwrap_or(0);
    let next = transcript
        .words
        .iter()
        .map(|w| w.start_ticks)
        .filter(|v| *v >= span.end)
        .min()
        .unwrap_or(span.end);
    let mut padded_start = span
        .start
        .saturating_sub(((span.start - previous) / 2).min(13_500));
    let mut padded_end = span.end.saturating_add(((next - span.end) / 2).min(13_500));
    for region in &windows.invalid_regions {
        if region.end_ticks <= span.start {
            padded_start = padded_start.max(region.end_ticks.min(span.start));
        }
        if region.start_ticks >= span.end {
            padded_end = padded_end.min(region.start_ticks.max(span.end));
        }
    }
    if padded_end - padded_start <= max_ticks {
        span.start = padded_start;
        span.end = padded_end;
    }
    Ok(span)
}
#[allow(
    clippy::cast_precision_loss,
    reason = "diagnostic seconds, never used to derive boundaries"
)]
fn ticks_seconds(ticks: u64) -> f64 {
    ticks as f64 / 90_000.0
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
