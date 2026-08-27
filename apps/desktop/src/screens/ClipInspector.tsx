/**
 * One clip, and everything the system can say about why it is here.
 *
 * Three panes, and the order is the argument: the other candidates, the clip
 * itself, and the reasons. A decision made without the first pane is a decision
 * made without comparison; a decision made without the third is a decision made
 * on a number. Both are the failure this screen exists to prevent.
 *
 * Nothing here is a summary of a summary. The bars are the ranking document's
 * own factors, the quotes are the sentences those factors were read from
 * resolved through the evidence index, and the boundary strip is the real
 * lattice the optimizer chose between. Where a value is missing the panel says
 * which and why, because an axis nobody measured is a different fact from an
 * axis that scored nothing.
 */
import { ArrowLeft, Check, Clock, TriangleAlert } from 'lucide-react';

import { Button } from '../components/ui/button.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import type { ClipDecision, CropPath } from '../daemon/client.js';
import { Preview, type OverlayCue } from '../inspector/Preview.js';
import { AxisBars } from '../inspector/parts/AxisBars.js';
import { BoundaryStrip } from '../inspector/parts/BoundaryStrip.js';
import { CandidateRail } from '../inspector/parts/CandidateRail.js';
import { type ClipRow, clock, duration } from '../results/model.js';
import { ScoreRing } from '../results/parts/ScoreRing.js';

export interface ClipInspectorProps {
  readonly rows: readonly ClipRow[];
  readonly candidateId: string;
  readonly proxyUrl: string | null;
  readonly crop: CropPath | null;
  readonly cues: readonly OverlayCue[];
  /** True while a decision or a direct is in flight. */
  readonly busy: boolean;
  /** What the last action said, when it said something. */
  readonly notice: string | null;
  readonly onSelect: (candidateId: string) => void;
  readonly onBack: () => void;
  readonly onDecide: (decision: ClipDecision) => void;
  readonly onUseAlternative: () => void;
}

export function ClipInspector({
  rows,
  candidateId,
  proxyUrl,
  crop,
  cues,
  busy,
  notice,
  onSelect,
  onBack,
  onDecide,
  onUseAlternative,
}: ClipInspectorProps) {
  const row = rows.find((candidate) => candidate.candidateId === candidateId);

  if (!row) {
    return (
      <div className="grid flex-1 place-items-center p-8">
        <div className="flex flex-col items-center gap-3">
          <p className="text-[13px] text-[var(--cm-text-secondary)]">
            That clip is not in the current ranking.
          </p>
          <Button variant="outline" onClick={onBack}>
            Back to the board
          </Button>
        </div>
      </div>
    );
  }

  const measured = row.axes.filter((axis) => axis.value !== null).length;
  const quotes = row.axes.flatMap((axis) =>
    axis.evidence.map((text) => ({ axis: axis.label, text })),
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 p-6">
      <header className="flex shrink-0 items-center gap-3">
        <Button variant="ghost" size="sm" onClick={onBack} className="gap-1.5">
          <ArrowLeft className="size-4" aria-hidden />
          Board
        </Button>
        <h1 className="truncate text-[length:var(--cm-type-card-title)] font-semibold text-[var(--cm-text-primary)]">
          {row.headline || 'Untitled clip'}
        </h1>
        <span className="mono ml-auto shrink-0 text-[11px] text-[var(--cm-text-muted)]">
          Rank {row.rank} of {rows.length}
        </span>
      </header>

      <div className="flex min-h-0 flex-1 gap-3">
        <CandidateRail rows={rows} candidateId={candidateId} onSelect={onSelect} />

        <section
          className="glass flex min-h-0 min-w-0 flex-1 flex-col gap-3 rounded-[var(--cm-radius-card)] p-4"
          aria-label="The clip"
        >
          <div className="grid min-h-0 flex-1 place-items-center rounded-[var(--cm-radius-panel)] border border-[var(--cm-recessed-border)] bg-[var(--cm-recessed)] p-3">
            <Preview
              src={proxyUrl}
              startTicks={row.startTicks}
              endTicks={row.endTicks}
              crop={crop}
              cues={cues}
            />
          </div>
          <BoundaryStrip row={row} />
        </section>

        <section
          className="glass flex w-[360px] shrink-0 flex-col overflow-hidden rounded-[var(--cm-radius-card)]"
          aria-label="Why this clip"
        >
          <div className="flex items-center gap-4 border-b border-[var(--cm-glass-border)] p-4">
            <ScoreRing score={row.displayScore} band={row.band} size="lg" caption={row.bandLabel} />
            <dl className="flex min-w-0 flex-1 flex-col gap-2 text-[11px]">
              {[
                ['Length', duration(row.durationSeconds)],
                ['Window', `${clock(row.startTicks)} – ${clock(row.endTicks)}`],
                ['Axes measured', `${measured} of ${row.axes.length}`],
              ].map(([label, value]) => (
                <div key={label} className="flex items-baseline justify-between gap-2">
                  <dt className="text-[var(--cm-text-muted)]">{label}</dt>
                  <dd className="mono text-[var(--cm-text-primary)]">{value}</dd>
                </div>
              ))}
            </dl>
          </div>

          <Tabs defaultValue="score" className="flex min-h-0 flex-1 flex-col">
            <TabsList className="mx-3 mt-3 shrink-0">
              <TabsTrigger value="score">Score</TabsTrigger>
              <TabsTrigger value="evidence">Evidence</TabsTrigger>
              <TabsTrigger value="boundary">Boundary</TabsTrigger>
              <TabsTrigger value="risk">Risk</TabsTrigger>
            </TabsList>

            <div className="min-h-0 flex-1 overflow-y-auto p-4">
              <TabsContent value="score" className="mt-0">
                <AxisBars axes={row.axes} />
              </TabsContent>

              <TabsContent value="evidence" className="mt-0 flex flex-col gap-3">
                {quotes.length === 0 ? (
                  <p className="text-[12px] text-[var(--cm-text-muted)]">
                    No evidence index was published for this analysis, so the factors carry
                    positions but no text.
                  </p>
                ) : (
                  quotes.map((quote, index) => (
                    <blockquote
                      key={`${quote.axis}-${index}`}
                      className="rounded-[var(--cm-radius-control)] border border-[var(--cm-recessed-border)] bg-[var(--cm-recessed)] p-3"
                    >
                      <p className="text-[12px] leading-relaxed text-[var(--cm-text-primary)] italic">
                        “{quote.text}”
                      </p>
                      <footer className="mt-2 text-[10px] tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
                        {quote.axis}
                      </footer>
                    </blockquote>
                  ))
                )}
              </TabsContent>

              <TabsContent value="boundary" className="mt-0 flex flex-col gap-4">
                {row.boundary ? (
                  <>
                    <ul className="flex flex-col gap-2">
                      {row.boundary.terms.map((term) => (
                        <li key={term.name} className="flex items-baseline justify-between gap-3">
                          <span className="text-[12px] text-[var(--cm-text-secondary)]">
                            {term.name.replaceAll('_', ' ')}
                          </span>
                          <span className="mono text-[11px] text-[var(--cm-text-primary)]">
                            {term.value.toFixed(3)}
                          </span>
                        </li>
                      ))}
                    </ul>
                    {row.boundary.alternative ? (
                      <div className="flex flex-col gap-2 border-t border-[var(--cm-glass-border)] pt-3">
                        <p className="text-[11px] text-[var(--cm-text-secondary)]">
                          The runner-up cut ran{' '}
                          <span className="mono text-[var(--cm-text-primary)]">
                            {clock(row.boundary.alternative.startTicks)} –{' '}
                            {clock(row.boundary.alternative.endTicks)}
                          </span>
                          . Taking it rebuilds the edit document from that cut.
                        </p>
                        <Button
                          variant="outline"
                          size="sm"
                          disabled={busy}
                          onClick={onUseAlternative}
                        >
                          Use the alternative cut
                        </Button>
                      </div>
                    ) : (
                      <p className="border-t border-[var(--cm-glass-border)] pt-3 text-[11px] text-[var(--cm-text-muted)]">
                        The lattice offered one legal pair, so there is no alternative to swap to.
                      </p>
                    )}
                  </>
                ) : (
                  <p className="text-[12px] text-[var(--cm-text-muted)]">
                    This candidate carries no boundary record.
                  </p>
                )}
              </TabsContent>

              <TabsContent value="risk" className="mt-0 flex flex-col gap-3">
                {row.warnings.length === 0 && row.penalties.length === 0 ? (
                  <p className="flex items-center gap-2 text-[12px] text-[var(--cm-text-secondary)]">
                    <Check className="size-4 text-[var(--cm-success-ink)]" aria-hidden />
                    The ranker recorded nothing against this clip.
                  </p>
                ) : (
                  <>
                    {row.warnings.map((warning) => (
                      <p
                        key={warning}
                        className="flex items-start gap-2 text-[12px] text-[var(--cm-warning-ink)]"
                      >
                        <TriangleAlert className="mt-px size-3.5 shrink-0" aria-hidden />
                        {warning}
                      </p>
                    ))}
                    {row.penalties.map((penalty) => (
                      <p
                        key={penalty.reason}
                        className="flex items-start justify-between gap-2 text-[12px] text-[var(--cm-text-secondary)]"
                      >
                        <span>{penalty.reason.replaceAll('_', ' ')}</span>
                        <span className="mono text-[var(--cm-danger-ink)]">−{penalty.value}</span>
                      </p>
                    ))}
                  </>
                )}
              </TabsContent>
            </div>
          </Tabs>

          <footer className="flex shrink-0 flex-col gap-2 border-t border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-4">
            {notice && (
              <p
                className="flex items-start gap-2 text-[11px] text-[var(--cm-text-secondary)]"
                role="status"
              >
                <Clock className="mt-px size-3 shrink-0" aria-hidden />
                {notice}
              </p>
            )}
            <Button
              className="w-full justify-center"
              disabled={busy}
              onClick={() => onDecide('approved')}
            >
              {busy ? 'Working…' : 'Approve for the editor'}
            </Button>
            <div className="flex gap-2">
              <Button
                variant="outline"
                className="flex-1"
                disabled={busy}
                onClick={() => onDecide('kept')}
              >
                Keep for later
              </Button>
              <Button
                variant="ghost"
                className="flex-1 text-[var(--cm-danger-ink)]"
                disabled={busy}
                onClick={() => onDecide('rejected')}
              >
                Reject
              </Button>
            </div>
          </footer>
        </section>
      </div>
    </div>
  );
}
