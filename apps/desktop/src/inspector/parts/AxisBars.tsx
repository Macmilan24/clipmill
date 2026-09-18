/**
 * The eight axes, including the ones nobody measured.
 *
 * An unmeasured axis renders as its reason rather than as a zero-length bar.
 * That distinction is the whole point of the panel: a clip that scored nothing
 * on prompt relevance and a clip nobody could score on prompt relevance are
 * different facts, and a bar at zero states the first while meaning the second.
 *
 * The weight is shown beside the value because the two together are what moved
 * the total. A reader comparing two cards needs to know that an axis at 0.9
 * carrying weight 0.4 contributed less than one at 0.7 carrying 1.4, and the
 * panel would be misleading without it.
 */
import { Minus } from 'lucide-react';

import type { ClipRow } from '../../results/model.js';

export interface AxisBarsProps {
  readonly axes: ClipRow['axes'];
}

export function AxisBars({ axes }: AxisBarsProps) {
  return (
    <ul className="flex flex-col">
      {axes.map((reading) => (
        <li
          key={reading.axis}
          className="flex flex-col gap-1.5 border-b border-[var(--cm-glass-border)] py-2.5 last:border-b-0"
        >
          <div className="flex items-baseline justify-between gap-3">
            <span className="text-[12px] font-medium text-[var(--cm-text-primary)]">
              {reading.label}
            </span>
            {reading.value === null ? (
              <span className="flex items-center gap-1 text-[10px] text-[var(--cm-text-muted)]">
                <Minus className="size-3" aria-hidden />
                not measured
              </span>
            ) : (
              <span className="flex items-baseline gap-2">
                {reading.weight !== null && (
                  <span
                    className="mono text-[10px] text-[var(--cm-text-muted)]"
                    title="The weight this axis carries in the total"
                  >
                    ×{reading.weight.toFixed(1)}
                  </span>
                )}
                <span className="mono text-[12px] text-[var(--cm-text-primary)]">
                  {Math.round(reading.value * 100)}
                </span>
              </span>
            )}
          </div>

          {reading.value === null ? (
            <p className="text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
              {reading.unavailableReason ?? 'No reason was recorded.'}
            </p>
          ) : (
            <div
              className="h-1 overflow-hidden rounded-full bg-[var(--cm-recessed)]"
              role="img"
              aria-label={`${reading.label} ${Math.round(reading.value * 100)} of 100`}
            >
              <div
                className="h-full rounded-full bg-[var(--cm-accent)]"
                style={{
                  width: `${Math.round(reading.value * 100)}%`,
                  transition: 'width 640ms cubic-bezier(0.22, 1, 0.36, 1)',
                }}
              />
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}
