import { ArrowUpRight, ChevronDown, MessageSquareWarning } from 'lucide-react';
import { Button } from '../../components/ui/button.js';
import { type ClipRow, clock, duration } from '../model.js';

/** An inspectable model opinion, kept outside recommended clips and bulk approval. */
export function DeclinedMoments({
  rows,
  onlyDeclines,
  busy,
  onInspect,
}: {
  readonly rows: readonly ClipRow[];
  readonly onlyDeclines: boolean;
  readonly busy: boolean;
  readonly onInspect: (id: string) => void;
}) {
  return (
    <details
      open={onlyDeclines}
      className={`group workspace-panel overflow-hidden ${onlyDeclines ? 'min-h-0 overflow-y-auto' : 'shrink-0'}`}
    >
      <summary className="flex cursor-pointer list-none items-start gap-3 px-4 py-3 [&::-webkit-details-marker]:hidden">
        <MessageSquareWarning className="mt-0.5 size-4 shrink-0 text-[var(--cm-warning-ink)]" />
        <div className="min-w-0 flex-1">
          <h2 className="text-xs font-semibold">
            Declined by the model{' '}
            <span className="ml-1 font-mono text-[var(--cm-text-muted)]">{rows.length}</span>
          </h2>
          <p className="mt-1 text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
            {onlyDeclines
              ? 'No clips were recommended. Inspect the proposed moments and their reasons, then decide whether to make your own edit.'
              : 'These proposed moments were not recommended. You can still inspect them and choose to edit.'}
          </p>
        </div>
        <ChevronDown className="mt-0.5 size-4 text-[var(--cm-text-muted)] transition-transform group-open:rotate-180" />
      </summary>
      <ul
        aria-label="Declined moments"
        className={`divide-y divide-[var(--cm-glass-border)] border-t border-[var(--cm-glass-border)] ${onlyDeclines ? '' : 'max-h-[240px] overflow-y-auto'}`}
      >
        {rows.map((row) => (
          <li key={row.candidateId} className="flex items-start justify-between gap-4 px-4 py-4">
            <div className="min-w-0">
              <h3 className="text-xs font-medium">{row.headline || 'Untitled moment'}</h3>
              <p className="mt-1 font-mono text-[10px] text-[var(--cm-text-muted)]">
                {clock(row.startTicks)} – {clock(row.endTicks)} · {duration(row.durationSeconds)}
              </p>
              <p className="mt-2 max-w-[820px] text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
                {row.review?.reasons[0] ||
                  row.review?.summary ||
                  'The editorial review did not recommend this moment.'}
              </p>
              {row.docId && (
                <p className="mt-1.5 text-[10px] text-[var(--cm-text-muted)]">
                  You created an edit from this moment. The original model review is retained.
                </p>
              )}
            </div>
            <Button
              variant="outline"
              size="sm"
              className="shrink-0"
              disabled={busy}
              onClick={() => onInspect(row.candidateId)}
              aria-label={`Inspect declined moment: ${row.headline || 'Untitled moment'}`}
            >
              Inspect
              <ArrowUpRight />
            </Button>
          </li>
        ))}
      </ul>
    </details>
  );
}
