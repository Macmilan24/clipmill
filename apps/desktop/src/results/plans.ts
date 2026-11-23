/**
 * The mind's clips, read from its published ranking (`ranking.set.v2`) and
 * turned into the rows Results and the Clip Inspector show for any clip.
 *
 * A plan is how the engine builds a clip: parts taken from wherever the moment
 * is, in the order a viewer should hear them. A person sees the clip, not the
 * plan, so each one becomes an ordinary row that also carries its parts, which
 * the Inspector plays in order.
 */
import type { MindClipPlan } from '@clipmill/contracts';

import type { ClipDecision, ClipDecisionRecord, EditDocSummary } from '../daemon/client.js';
import {
  type ClipPart,
  type ClipRow,
  type Summary,
  TICKS_PER_SECOND,
  newestDocumentPerCandidate,
} from './model.js';

export const MIND_RANKING_KIND = 'ranking.set.v2';

type Plan = MindClipPlan;
type PlanPartDoc = Plan['parts'][number];

const ROLE_LABELS: Readonly<Record<string, string>> = {
  hook: 'Hook',
  setup: 'Setup',
  question: 'Question',
  answer: 'Answer',
  body: 'Moment',
  payoff: 'Payoff',
  proof: 'Proof',
  reaction: 'Reaction',
  item: 'Moment',
};

const FORMS: ReadonlySet<string> = new Set([
  'straight',
  'tightened',
  'stitch',
  'cold_open',
  'theme_montage',
]);

export interface PlannedClip {
  readonly planId: string;
  readonly title: string;
  readonly hookText: string | null;
  readonly durationSeconds: number;
  readonly parts: readonly ClipPart[];
  readonly doubts: readonly string[];
  readonly assessment: {
    readonly keepScore: number;
    readonly reasons: readonly string[];
    readonly issues: readonly string[];
    readonly summary: string;
    /** What the review said about a join it did not pass. */
    readonly joinNotes: readonly string[];
  };
  /** The plan as the daemon directs it. */
  readonly planJson: string;
}

/** The published shortlist is authoritative, including a decision to return no clips. */
export interface PlanRanking {
  readonly clips: readonly PlannedClip[];
  readonly requestedCount: number;
  readonly cutoff: string | null;
}

/** What plays of a part: the part less its removed ranges, in order. */
function keptOf(part: PlanPartDoc): ClipPart['kept'] {
  const removed = (part.removed ?? []).toSorted(
    (left, right) => left.start_ticks - right.start_ticks,
  );
  const kept: { startTicks: number; endTicks: number }[] = [];
  let from = part.start_ticks;
  for (const range of removed) {
    if (range.start_ticks > from) kept.push({ startTicks: from, endTicks: range.start_ticks });
    from = Math.max(from, range.end_ticks);
  }
  if (part.end_ticks > from) kept.push({ startTicks: from, endTicks: part.end_ticks });
  return kept;
}

function record(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === 'object' ? (value as Record<string, unknown>) : {};
}

function strings(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((entry): entry is string => typeof entry === 'string' && entry.trim() !== '')
    : [];
}

/** A plan this Studio can play and direct, or null for one it cannot. */
function readable(plan: Plan | undefined, fingerprint: string): plan is Plan {
  return (
    plan !== undefined &&
    plan.schema_version === 'clipmill.mind.clip_plan.v1' &&
    plan.source_fingerprint === fingerprint &&
    FORMS.has(plan.form) &&
    // A document from disk, whatever its type says.
    Array.isArray(plan.parts) &&
    (plan.parts as readonly unknown[]).length > 0
  );
}

export function readPlanRanking(document: unknown, fingerprint: string): PlanRanking {
  const doc = record(document);
  if (
    doc.schema_version !== 'clipmill.ranking.set.v2' ||
    doc.source_fingerprint !== fingerprint ||
    !Array.isArray(doc.clips)
  ) {
    throw new Error('The published editorial ranking does not match this recording.');
  }
  const clips = doc.clips.map((value): PlannedClip => {
    const entry = record(value);
    const plan = entry.plan as Plan | undefined;
    const quality = record(entry.quality);
    const score = entry.keep_score;
    if (
      !readable(plan, fingerprint) ||
      quality.verdict !== 'accept' ||
      typeof score !== 'number' ||
      !Number.isFinite(score) ||
      score < 0 ||
      score > 100
    ) {
      throw new Error('The published editorial ranking contains an unreadable clip.');
    }
    const joinNotes = Array.isArray(quality.joins)
      ? quality.joins.flatMap((report) => {
          const item = record(report);
          return item.verdict !== 'pass' && typeof item.note === 'string' && item.note.trim()
            ? [item.note]
            : [];
        })
      : [];
    return {
      planId: plan.plan_id,
      title: plan.title ?? plan.hook_text ?? 'Untitled clip',
      hookText: plan.hook_text ?? null,
      durationSeconds: plan.duration_ticks / TICKS_PER_SECOND,
      parts: plan.parts.map((part) => ({
        label: ROLE_LABELS[part.role] ?? part.role,
        startTicks: part.start_ticks,
        endTicks: part.end_ticks,
        kept: keptOf(part),
      })),
      doubts: strings(plan.doubts),
      assessment: {
        keepScore: score,
        reasons: strings(entry.reasons),
        issues: strings(quality.issues),
        summary: typeof quality.summary === 'string' ? quality.summary : '',
        joinNotes,
      },
      planJson: JSON.stringify(plan),
    };
  });
  return {
    clips,
    requestedCount: typeof doc.requested_count === 'number' ? doc.requested_count : 0,
    cutoff:
      typeof doc.cutoff_reason === 'string' && doc.cutoff_reason.trim() ? doc.cutoff_reason : null,
  };
}

/**
 * The mind's clips as Results rows, in the ranking's order.
 *
 * `ids` names the id each plan's decisions and edit are stored under; a plan
 * missing from it is named by its plan id, which still plays and approves.
 */
export function planRows(
  ranking: PlanRanking,
  ids: ReadonlyMap<string, string>,
  decisions: readonly ClipDecisionRecord[],
  documents: readonly EditDocSummary[] = [],
): readonly ClipRow[] {
  const decided = new Map(
    decisions
      .filter((entry) => entry.decision !== 'unspecified')
      .map((entry) => [entry.candidateId, entry.decision as ClipDecision]),
  );
  const edited = newestDocumentPerCandidate(documents);
  return ranking.clips.map((clip, index): ClipRow => {
    const candidateId = ids.get(clip.planId) ?? clip.planId;
    const { assessment } = clip;
    const checks = [...clip.doubts, ...assessment.issues, ...assessment.joinNotes];
    const first = clip.parts[0]!;
    return {
      candidateId,
      rank: index + 1,
      displayScore: Math.round(assessment.keepScore),
      review: {
        status: 'accepted',
        reasons: assessment.reasons,
        route: 'local',
        ...(assessment.summary ? { summary: assessment.summary } : {}),
      },
      band: 'strong',
      bandLabel: 'Ready to review',
      warnings: [...assessment.reasons, ...checks],
      startTicks: Math.min(...clip.parts.map((part) => part.startTicks)),
      endTicks: Math.max(...clip.parts.map((part) => part.endTicks)),
      durationSeconds: clip.durationSeconds,
      headline: clip.title,
      axes: [],
      penalties: [],
      boundary: null,
      decision: decided.get(candidateId) ?? null,
      docId: edited.get(candidateId)?.docId ?? null,
      docJobId: edited.get(candidateId)?.jobId || null,
      latticeStarts: [],
      latticeEnds: [],
      recommended: true,
      proposer: null,
      clusterId: null,
      hook: clip.hookText ? { text: clip.hookText, atTicks: first.startTicks } : null,
      payoff: null,
      flagged: checks.length > 0,
      parts: clip.parts,
      planJson: clip.planJson,
    };
  });
}

/** What the board says above the mind's clips: how many, of how many asked for, and why. */
export function planSummary(
  ranking: PlanRanking,
  contentProfile: 'interview' | 'scripted',
): Summary {
  const count = ranking.clips.length;
  return {
    selected: count,
    cohort: count,
    requested: ranking.requestedCount || count,
    shortfall: ranking.cutoff && count < ranking.requestedCount ? [ranking.cutoff] : [],
    warnings: [],
    filtered: 0,
    declined: 0,
    contentProfile,
  };
}
