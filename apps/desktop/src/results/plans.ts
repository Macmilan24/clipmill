/**
 * The mind's clips (`mind.candidates.v1`): plans of parts, read for the board.
 *
 * A plan is not one span of the recording but parts taken from wherever the
 * moment is, in the order a viewer should hear them. What the board needs is
 * what a person needs to judge it before opening it: its form, where its parts
 * sit in the recording, what each part is for, and what was cut out.
 */
import type { MindClipPlan } from '@clipmill/contracts';

import { clock } from './model.js';

export const MIND_CANDIDATES_KIND = 'mind.candidates.v1';

const TICKS_PER_SECOND = 90_000;

export type PlanForm = MindClipPlan['form'];
type Plan = MindClipPlan;
type PlanPartDoc = Plan['parts'][number];

export const FORM_LABELS: Readonly<Record<PlanForm, string>> = {
  straight: 'Straight',
  tightened: 'Tightened',
  stitch: 'Stitched',
  cold_open: 'Cold open',
  theme_montage: 'Montage',
};

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

export interface PlanPart {
  readonly id: string;
  readonly role: string;
  readonly roleLabel: string;
  readonly startTicks: number;
  readonly endTicks: number;
  /** What plays: the part less what was cut out of it. */
  readonly keptTicks: number;
  /** How much was cut out of it, and how many cuts. */
  readonly cutTicks: number;
  readonly cuts: number;
}

export interface PlannedClip {
  readonly planId: string;
  readonly form: PlanForm;
  readonly formLabel: string;
  readonly title: string;
  readonly hookText: string | null;
  readonly durationSeconds: number;
  readonly parts: readonly PlanPart[];
  /** One sentence on how the clip is built, in the order it plays. */
  readonly summary: string;
  /** The plan as the daemon directs it. */
  readonly planJson: string;
}

function keptOf(part: PlanPartDoc): { kept: number; cut: number; cuts: number } {
  const span = part.end_ticks - part.start_ticks;
  const removed = part.removed ?? [];
  const cut = removed.reduce((sum, range) => sum + (range.end_ticks - range.start_ticks), 0);
  return { kept: Math.max(0, span - cut), cut, cuts: removed.length };
}

function seconds(ticks: number): string {
  return `${Math.round(ticks / TICKS_PER_SECOND)} s`;
}

/** How a plan is put together, said plainly. */
export function describe(form: PlanForm, parts: readonly PlanPart[]): string {
  const cut = parts.reduce((sum, part) => sum + part.cutTicks, 0);
  const trimmed = cut > 0 ? `, with ${seconds(cut)} cut out` : '';
  const [first, second] = parts;
  switch (form) {
    case 'stitch':
      return first && second
        ? `Joins the ${first.roleLabel.toLowerCase()} at ${clock(first.startTicks)} to the ${second.roleLabel.toLowerCase()} at ${clock(second.startTicks)}${trimmed}.`
        : `Parts from different places in the recording${trimmed}.`;
    case 'tightened':
      return `One moment at ${first ? clock(first.startTicks) : '0:00'} with ${seconds(cut)} cut out.`;
    case 'cold_open':
      return first && second
        ? `Opens on the line at ${clock(first.startTicks)}, then plays the moment from ${clock(second.startTicks)}${trimmed}.`
        : `Opens on its strongest line${trimmed}.`;
    case 'theme_montage':
      return `${parts.length} moments on one theme, in the order they were said${trimmed}.`;
    case 'straight':
      return `One continuous moment at ${first ? clock(first.startTicks) : '0:00'}.`;
  }
}

function plannedClip(plan: Plan): PlannedClip | null {
  if (
    plan.schema_version !== 'clipmill.mind.clip_plan.v1' ||
    !(plan.form in FORM_LABELS) ||
    // A document from disk, whatever its type says.
    !Array.isArray(plan.parts) ||
    (plan.parts as readonly unknown[]).length === 0
  ) {
    return null;
  }
  const parts = plan.parts.map((part) => {
    const { kept, cut, cuts } = keptOf(part);
    return {
      id: part.part_id,
      role: part.role,
      roleLabel: ROLE_LABELS[part.role] ?? part.role,
      startTicks: part.start_ticks,
      endTicks: part.end_ticks,
      keptTicks: kept,
      cutTicks: cut,
      cuts,
    };
  });
  return {
    planId: plan.plan_id,
    form: plan.form,
    formLabel: FORM_LABELS[plan.form],
    title: plan.title ?? plan.hook_text ?? 'Untitled clip',
    hookText: plan.hook_text ?? null,
    durationSeconds: plan.duration_ticks / TICKS_PER_SECOND,
    parts,
    summary: describe(plan.form, parts),
    planJson: JSON.stringify(plan),
  };
}

/**
 * The plans in a candidates document, best first, or none.
 *
 * None for a document about another recording: parts timed against a
 * different source would cut the wrong words out of this one.
 */
export function readPlans(document: unknown, sourceFingerprint: string): readonly PlannedClip[] {
  if (!document || typeof document !== 'object') return [];
  const doc = document as {
    schema_version?: unknown;
    source_fingerprint?: unknown;
    candidates?: unknown;
  };
  if (
    doc.schema_version !== 'clipmill.mind.candidates.v1' ||
    doc.source_fingerprint !== sourceFingerprint ||
    !Array.isArray(doc.candidates)
  ) {
    return [];
  }
  const plans: PlannedClip[] = [];
  for (const candidate of doc.candidates as readonly { plan?: unknown }[]) {
    const plan = candidate?.plan as Plan | undefined;
    if (!plan || plan.source_fingerprint !== sourceFingerprint) continue;
    const read = plannedClip(plan);
    if (read) plans.push(read);
  }
  return plans;
}
