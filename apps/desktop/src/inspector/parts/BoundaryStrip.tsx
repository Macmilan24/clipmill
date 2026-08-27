/**
 * Where the cut landed, against everywhere it could legally have landed.
 *
 * The ticks are the lattice: every start and end the optimizer was allowed to
 * choose between, drawn at their real positions. That is what makes the strip
 * worth showing rather than decorative — an editor who thinks the cut is late
 * can see whether an earlier edge even existed, and the answer is frequently
 * that it did not, because every boundary from the index is word-aligned by
 * construction.
 *
 * The runner-up is drawn when the lattice offered one, and it is a real second
 * interval rather than a nudge of the first. Taking it rebuilds the edit
 * document from that cut, which is why it is a button and not a preference.
 */
import type { ClipRow } from '../../results/model.js';
import { clock } from '../../results/model.js';

export interface BoundaryStripProps {
  readonly row: ClipRow;
  /** The window drawn, wide enough to show the edges either side of the cut. */
  readonly padTicks?: number;
}

export function BoundaryStrip({ row, padTicks }: BoundaryStripProps) {
  const span = row.endTicks - row.startTicks;
  const pad = padTicks ?? Math.max(span * 0.35, 1);
  const from = Math.max(0, row.startTicks - pad);
  const to = row.endTicks + pad;
  const width = Math.max(1, to - from);
  const at = (ticks: number) => `${((ticks - from) / width) * 100}%`;

  const alternative = row.boundary?.alternative ?? null;

  return (
    <figure className="flex flex-col gap-2" aria-label="Boundary against the lattice">
      <div className="relative h-14 overflow-hidden rounded-[var(--cm-radius-control)] border border-[var(--cm-recessed-border)] bg-[var(--cm-recessed)]">
        {/* Every legal start, then every legal end. Two rows so a dense lattice
            stays readable instead of becoming one grey band. */}
        {row.latticeStarts.map((tick) => (
          <span
            key={`start-${tick}`}
            title={`A legal start at ${clock(tick)}`}
            className="absolute top-1 h-4 w-px"
            style={{ left: at(tick), background: 'var(--cm-text-disabled)' }}
          />
        ))}
        {row.latticeEnds.map((tick) => (
          <span
            key={`end-${tick}`}
            title={`A legal end at ${clock(tick)}`}
            className="absolute bottom-1 h-4 w-px"
            style={{ left: at(tick), background: 'var(--cm-text-disabled)' }}
          />
        ))}

        {alternative && (
          <span
            className="absolute inset-y-5 rounded-sm border border-dashed"
            title={`The runner-up: ${clock(alternative.startTicks)} – ${clock(alternative.endTicks)}`}
            style={{
              left: at(alternative.startTicks),
              width: at(alternative.endTicks - alternative.startTicks + from),
              borderColor: 'var(--cm-text-muted)',
            }}
          />
        )}

        <span
          className="absolute inset-y-0 rounded-sm"
          style={{
            left: at(row.startTicks),
            width: at(span + from),
            background: 'var(--cm-accent-selected)',
            borderLeft: '2px solid var(--cm-accent)',
            borderRight: '2px solid var(--cm-accent)',
          }}
        />
      </div>

      <figcaption className="mono flex justify-between text-[10px] text-[var(--cm-text-muted)]">
        <span>IN {clock(row.startTicks)}</span>
        <span className="text-[var(--cm-text-secondary)]">
          {(span / 90_000).toFixed(2)}s
          {row.latticeStarts.length > 0 && (
            <span className="ml-2 opacity-70">
              {row.latticeStarts.length}×{row.latticeEnds.length} legal pairs
            </span>
          )}
        </span>
        <span>OUT {clock(row.endTicks)}</span>
      </figcaption>
    </figure>
  );
}
