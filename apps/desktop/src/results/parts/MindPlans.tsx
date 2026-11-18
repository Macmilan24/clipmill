import { ChevronDown, Sparkles, SquarePen } from 'lucide-react';

import { Button } from '../../components/ui/button.js';
import { clock, duration } from '../model.js';
import type { PlannedClip, PlanPart } from '../plans.js';

/**
 * Where a plan's parts sit in the recording, drawn to scale: a stitch shows
 * as two marks minutes apart, a tightened moment as one mark with a gap.
 */
function PartStrip({
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
      {parts.map((part) => (
        <span
          key={part.id}
          className="absolute inset-y-0 rounded-full bg-[var(--cm-accent)]"
          style={{
            left: at(part.startTicks),
            width: `max(3px, calc(${at(part.endTicks)} - ${at(part.startTicks)}))`,
          }}
        />
      ))}
    </div>
  );
}

function partLine(part: PlanPart, index: number): string {
  const cut =
    part.cuts > 0
      ? ` · ${Math.round(part.cutTicks / 90_000)} s cut out${part.cuts > 1 ? ` in ${part.cuts} places` : ''}`
      : '';
  return `${index + 1}. ${part.roleLabel} ${clock(part.startTicks)}–${clock(part.endTicks)}${cut}`;
}

/**
 * The clips the mind built: parts from wherever the moment is, not one span.
 * Each opens as an edit; nothing here is approved or exported by itself.
 */
export function MindPlans({
  plans,
  sourceDurationTicks,
  busy,
  onEdit,
}: {
  readonly plans: readonly PlannedClip[];
  readonly sourceDurationTicks: number | null;
  readonly busy: boolean;
  readonly onEdit: (plan: PlannedClip) => void;
}) {
  return (
    <details open className="group workspace-panel shrink-0 overflow-hidden">
      <summary className="flex cursor-pointer list-none items-start gap-3 px-4 py-3 [&::-webkit-details-marker]:hidden">
        <Sparkles className="mt-0.5 size-4 shrink-0 text-[var(--cm-accent-ink)]" />
        <div className="min-w-0 flex-1">
          <h2 className="text-xs font-semibold">
            Built by the mind{' '}
            <span className="ml-1 font-mono text-[var(--cm-text-muted)]">{plans.length}</span>
          </h2>
          <p className="mt-1 text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
            Clips cut from wherever the moment is: a question joined to its answer, a story with its
            tangents removed, a strong line moved to the front. Open one to watch and adjust it.
          </p>
        </div>
        <ChevronDown className="mt-0.5 size-4 text-[var(--cm-text-muted)] transition-transform group-open:rotate-180" />
      </summary>
      <ul
        aria-label="Clips built by the mind"
        className="max-h-[360px] divide-y divide-[var(--cm-glass-border)] overflow-y-auto border-t border-[var(--cm-glass-border)]"
      >
        {plans.map((plan) => (
          <li key={plan.planId} className="flex items-start justify-between gap-4 px-4 py-4">
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2">
                <span className="rounded-full border border-[var(--cm-glass-border)] px-2 py-0.5 text-[10px] font-medium text-[var(--cm-accent-ink)]">
                  {plan.formLabel}
                </span>
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
                  {plan.parts.map((part, index) => (
                    <li key={part.id}>{partLine(part, index)}</li>
                  ))}
                </ol>
              ) : null}
              {plan.hookText && (
                <p className="mt-2 max-w-[820px] text-[11px] italic leading-relaxed text-[var(--cm-text-secondary)]">
                  “{plan.hookText}”
                </p>
              )}
            </div>
            <Button
              variant="outline"
              size="sm"
              className="shrink-0"
              disabled={busy}
              onClick={() => onEdit(plan)}
              aria-label={`Edit: ${plan.title}`}
            >
              Edit
              <SquarePen />
            </Button>
          </li>
        ))}
      </ul>
    </details>
  );
}
