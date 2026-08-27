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
import { ArrowLeft, Check, Clock, RotateCcw, Scissors, TriangleAlert } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import type { ClipDecision, CropPath } from '../daemon/client.js';
import type { OverlayCue } from '../inspector/Preview.js';
import { AxisBars } from '../inspector/parts/AxisBars.js';
import { CandidateRail } from '../inspector/parts/CandidateRail.js';
import { FRAME_TICKS, Player, timecode } from '../inspector/parts/Player.js';
import { Timeline } from '../inspector/parts/Timeline.js';
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
  /**
   * Build the document from a boundary the editor moved.
   *
   * The daemon snaps whatever it is given to the lattice, so this is a proposal
   * rather than an instruction — which is why it is a named action and not
   * something a drag performs on its own.
   */
  readonly onTakeCut: (startTicks: number, endTicks: number) => void;
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
  onTakeCut,
}: ClipInspectorProps) {
  const row = rows.find((candidate) => candidate.candidateId === candidateId);

  /** The boundary as dragged, or null while it is still the ranker's. */
  const [draft, setDraft] = useState<{ startTicks: number; endTicks: number } | null>(null);
  const [positionTicks, setPositionTicks] = useState(row?.startTicks ?? 0);
  const [seekNonce, setSeekNonce] = useState(0);

  // A different clip is a different window: the draft belonged to the last one,
  // and leaving the playhead where it was would seek the proxy to a position
  // outside the clip now on screen.
  useEffect(() => {
    setDraft(null);
    setPositionTicks(row?.startTicks ?? 0);
    setSeekNonce((nonce) => nonce + 1);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- the clip is the signal
  }, [candidateId]);

  const cut = draft ?? { startTicks: row?.startTicks ?? 0, endTicks: row?.endTicks ?? 0 };
  const moved =
    row !== undefined && (cut.startTicks !== row.startTicks || cut.endTicks !== row.endTicks);

  const scrub = (ticks: number) => {
    setPositionTicks(Math.min(cut.endTicks, Math.max(cut.startTicks, ticks)));
    setSeekNonce((nonce) => nonce + 1);
  };

  /** Move one edge, keeping the window at least a frame wide. */
  const onDraft = (edge: 'in' | 'out', ticks: number) => {
    if (!row) {
      return;
    }
    const next =
      edge === 'in'
        ? { startTicks: Math.min(ticks, cut.endTicks - FRAME_TICKS), endTicks: cut.endTicks }
        : { startTicks: cut.startTicks, endTicks: Math.max(ticks, cut.startTicks + FRAME_TICKS) };
    setDraft(next);
    // Show the edge that moved, because a boundary is judged by what it lands on.
    setPositionTicks(edge === 'in' ? next.startTicks : next.endTicks);
    setSeekNonce((nonce) => nonce + 1);
  };

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
          <div className="flex min-h-0 flex-1 flex-col rounded-[var(--cm-radius-panel)] border border-[var(--cm-recessed-border)] bg-[var(--cm-recessed)] p-3">
            <Player
              src={proxyUrl}
              startTicks={cut.startTicks}
              endTicks={cut.endTicks}
              crop={crop}
              cues={cues}
              positionTicks={positionTicks}
              onPosition={setPositionTicks}
              seekNonce={seekNonce}
            />
          </div>

          <Timeline
            startTicks={cut.startTicks}
            endTicks={cut.endTicks}
            latticeStarts={row.latticeStarts}
            latticeEnds={row.latticeEnds}
            alternative={row.boundary?.alternative ?? null}
            positionTicks={positionTicks}
            onScrub={scrub}
            onDraft={onDraft}
          />

          {moved && (
            <div className="flex shrink-0 items-center gap-3 rounded-[var(--cm-radius-control)] border border-[var(--cm-glass-border)] bg-[var(--cm-glass-elevated)] px-3 py-2">
              <p className="min-w-0 flex-1 text-[11px] text-[var(--cm-text-secondary)]">
                Moved to{' '}
                <span className="mono text-[var(--cm-text-primary)]">
                  {timecode(cut.startTicks)} – {timecode(cut.endTicks)}
                </span>
                . The daemon snaps a cut to the lattice, so it may land nearby.
              </p>
              <Button
                size="sm"
                variant="ghost"
                disabled={busy}
                onClick={() => {
                  setDraft(null);
                  scrub(row.startTicks);
                }}
              >
                <RotateCcw className="size-3.5" aria-hidden />
                Reset
              </Button>
              <Button
                size="sm"
                disabled={busy}
                onClick={() => onTakeCut(cut.startTicks, cut.endTicks)}
              >
                <Scissors className="size-3.5" aria-hidden />
                Take this cut
              </Button>
            </div>
          )}
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
