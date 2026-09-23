/**
 * Display ranking axes with their weights and values.
 * Unmeasured axes show their reasons instead of zero-length bars. Weights explain
 * each measured axis's contribution to the total.
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
