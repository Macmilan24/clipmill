/**
 * What a row is, and what its dots mean — decided once, for every view.
 *
 * A signal is a fact the documents hold about a clip, given a colour. The
 * qualities are axes the ranker measured above a display threshold; the
 * cautions are its warnings and penalties. Nothing here is a grade the
 * interface invented: every dot names the axis and the value behind it, so a
 * green dot reads as "Hook 93" on hover and not as an opinion.
 *
 * The thresholds are display cut-offs and are written here as such. They decide
 * whether a dot is drawn, never what the number is.
 */
import type { ClipDecision } from '../../daemon/client.js';
import type { ClipRow } from '../model.js';

export type Tone = 'success' | 'accent' | 'warning' | 'danger' | 'muted';

export const TONE_INK: Readonly<Record<Tone, string>> = {
  success: 'var(--cm-success-ink)',
  accent: 'var(--cm-accent-ink)',
  warning: 'var(--cm-warning-ink)',
  danger: 'var(--cm-danger-ink)',
  muted: 'var(--cm-text-muted)',
};

export interface Signal {
  readonly key: string;
  readonly label: string;
  readonly tone: Tone;
}

/** Above this, an axis is worth a dot. A display cut-off, not a judgement. */
const NOTABLE = 0.75;

export function signalsFor(row: ClipRow): readonly Signal[] {
  if (row.review) {
    return row.review.reasons.map((reason, index) => ({
      key: `review:${index}`,
      label: reason,
      tone: row.review?.status === 'accepted' ? 'success' : 'warning',
    }));
  }
  const found: Signal[] = [];
  for (const axis of row.axes) {
    if (axis.value !== null && axis.value >= NOTABLE && axis.axis !== 'feasibility') {
      found.push({
        key: `axis:${axis.axis}`,
        label: `${axis.label} ${Math.round(axis.value * 100)}`,
        tone: axis.axis === 'hook' ? 'success' : 'accent',
      });
    }
  }
  for (const warning of row.warnings) {
    found.push({ key: `warn:${warning}`, label: warning, tone: 'warning' });
  }
  for (const penalty of row.penalties) {
    found.push({
      key: `pen:${penalty.reason}`,
      label: `${penalty.reason.replaceAll('_', ' ')} −${penalty.value}`,
      tone: 'danger',
    });
  }
  return found;
}

export interface RowState {
  readonly label: string;
  readonly tone: Tone;
}

const DECIDED: Readonly<Record<ClipDecision, RowState>> = {
  approved: { label: 'Approved', tone: 'success' },
  kept: { label: 'Kept', tone: 'muted' },
  rejected: { label: 'Rejected', tone: 'danger' },
};

/**
 * The badge a row wears.
 *
 * A person's decision outranks the ranker's state, because once somebody has
 * approved or rejected a clip the ranker's opinion of it is history. Below
 * that, the ranker's recommendation outranks a flag: a clip it stands behind
 * while warning about is still one it stands behind.
 */
export function stateOf(row: ClipRow): RowState {
  if (row.decision) {
    return DECIDED[row.decision];
  }
  if (row.recommended) {
    return { label: 'Recommended', tone: 'accent' };
  }
  if (row.flagged) {
    return { label: 'Flagged', tone: 'warning' };
  }
  return { label: 'New', tone: 'muted' };
}

/** A translucent wash of a tone, for badge backgrounds. */
export function wash(tone: Tone): string {
  return tone === 'muted'
    ? 'var(--cm-recessed)'
    : `color-mix(in srgb, ${TONE_INK[tone]} 14%, transparent)`;
}
