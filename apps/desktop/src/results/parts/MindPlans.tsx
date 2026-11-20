import { SquarePen } from 'lucide-react';

import { Button } from '../../components/ui/button.js';
import { clock, duration } from '../model.js';
import type { PlannedClip, PlanPart } from '../plans.js';

/**
 * Where a plan's parts sit in the recording, drawn to scale: a stitch shows
 * as two marks minutes apart, a tightened moment as one mark with a gap.
 */
export function PartStrip({
  parts,
  sourceDurationTicks,
}: {
  readonly parts: readonly PlanPart[];
  readonly sourceDurationTicks: number;
}) {
  const at = (ticks: number) => `${Math.min(100, (ticks / sourceDurationTicks) * 100)}%`;
  return (
    <div
      aria-hidden
      className="relative mt-2 h-1.5 w-full max-w-[420px] overflow-hidden rounded-full bg-[var(--cm-recessed)]"
    >
      {parts.flatMap((part) => {
        const edges = [
          part.startTicks,
          ...part.removed.flatMap((cut) => [cut.start_ticks, cut.end_ticks]),
          part.endTicks,
        ];
        return edges.flatMap((start, index) => {
          const end = edges[index + 1];
          if (index % 2 !== 0 || end === undefined) return [];
          return (
            <span
              key={`${part.id}-${start}`}
              className="absolute inset-y-0 rounded-full bg-[var(--cm-accent)]"
              style={{ left: at(start), width: `max(2px, calc(${at(end)} - ${at(start)}))` }}
            />
          );
        });
      })}
    </div>
  );
}

/** One part, as a line: its role, where it is, and what was cut out of it. */
export function partLine(part: PlanPart, index: number): string {
  const cut =
    part.cuts > 0
      ? ` · ${Math.round(part.cutTicks / 90_000)} s cut out${part.cuts > 1 ? ` in ${part.cuts} places` : ''}`
      : '';
  return `${index + 1}. ${part.roleLabel} ${clock(part.startTicks)}–${clock(part.endTicks)}${cut}`;
}

/**
 * The mind's clips as a list, one row each with its parts written out.
 * Selecting a row shows it in the rail; nothing here approves or exports a
 * clip by itself.
 */
export function PlanList({
  plans,
  sourceDurationTicks,
  focusedId,
  busy,
  onFocus,
  onEdit,
  onInspect,
}: {
  readonly plans: readonly PlannedClip[];
  readonly sourceDurationTicks: number | null;
  readonly focusedId: string | null;
  readonly busy: boolean;
  readonly onFocus: (plan: PlannedClip) => void;
  readonly onEdit: (plan: PlannedClip) => void;
  readonly onInspect?: ((plan: PlannedClip) => void) | undefined;
}) {
  return (
    <ul
      aria-label="Clips built by the mind"
      className="workspace-panel divide-y divide-[var(--cm-glass-border)] overflow-y-auto"
    >
      {plans.map((plan, index) => (
        <li
          key={plan.planId}
          aria-current={plan.planId === focusedId ? 'true' : undefined}
          onClick={() => onFocus(plan)}
          className="flex cursor-pointer items-start justify-between gap-4 px-4 py-4 aria-[current=true]:bg-[var(--cm-accent-selected)]"
        >
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-mono text-[10px] text-[var(--cm-text-muted)]">
                {String(index + 1).padStart(2, '0')}
              </span>
              <span className="rounded-full border border-[var(--cm-glass-border)] px-2 py-0.5 text-[10px] font-medium text-[var(--cm-accent-ink)]">
                {plan.formLabel}
              </span>
              {plan.assessment && (
                <span className="font-mono text-[11px] text-[var(--cm-text-secondary)]">
                  Keep score {plan.assessment.keepScore}/100
                </span>
              )}
              <h3 className="truncate text-xs font-medium">{plan.title}</h3>
              <span className="font-mono text-[10px] text-[var(--cm-text-muted)]">
                {duration(plan.durationSeconds)}
              </span>
            </div>
            <p className="mt-1.5 text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
              {plan.summary}
            </p>
            {sourceDurationTicks !== null && sourceDurationTicks > 0 && (
              <PartStrip parts={plan.parts} sourceDurationTicks={sourceDurationTicks} />
            )}
            {plan.parts.length > 1 || plan.parts.some((part) => part.cuts > 0) ? (
              <ol className="mt-2 space-y-0.5 font-mono text-[10px] text-[var(--cm-text-muted)]">
                {plan.parts.map((part, at) => (
                  <li key={part.id}>{partLine(part, at)}</li>
                ))}
              </ol>
            ) : null}
            {plan.hookText && (
              <p className="mt-2 max-w-[820px] text-[11px] italic leading-relaxed text-[var(--cm-text-secondary)]">
                “{plan.hookText}”
              </p>
            )}
          </div>
          <div className="flex flex-wrap items-center gap-2">
            {onInspect && (
              <Button
                variant="outline"
                size="sm"
                disabled={busy}
                onClick={(event) => {
                  event.stopPropagation();
                  onInspect(plan);
                }}
                aria-label={`Review ${plan.title}`}
              >
                Review
              </Button>
            )}
            <Button
              variant="outline"
              size="sm"
              className="shrink-0"
              disabled={busy}
              onClick={(event) => {
                event.stopPropagation();
                onEdit(plan);
              }}
              aria-label={`Edit: ${plan.title}`}
            >
              Edit
              <SquarePen />
            </Button>
          </div>
        </li>
      ))}
    </ul>
  );
}
