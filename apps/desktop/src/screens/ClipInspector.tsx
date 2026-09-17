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
 * resolved through the evidence index — with the position each was said at, so
 * a quote is a place the player can jump to — and the boundary strip is the
 * real lattice the optimizer chose between, over the recording's own waveform.
 * Where a value is missing the panel says which and why, because an axis nobody
 * measured is a different fact from an axis that scored nothing.
 */
import { ArrowLeft, Check, Clock, RotateCcw, Scissors, TriangleAlert } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import type { ClipDecision, CropPath } from '../daemon/client.js';
import type { OverlayCue } from '../inspector/Preview.js';
import { CandidateRail } from '../inspector/parts/CandidateRail.js';
import { FRAME_TICKS, Player, timecode } from '../inspector/parts/Player.js';
import { Timeline } from '../inspector/parts/Timeline.js';
import type { Peaks } from '../results/loader.js';
import { type ClipRow, type Quote, clock, duration, topFactors } from '../results/model.js';
import { ScoreRing } from '../results/parts/ScoreRing.js';
import { TONE_INK, stateOf, wash } from '../results/parts/state.js';

export interface ClipInspectorProps {
  readonly rows: readonly ClipRow[];
  readonly candidateId: string;
  readonly proxyUrl: string | null;
  readonly crop: CropPath | null;
  readonly cues: readonly OverlayCue[];
  readonly peaks: Peaks | null;
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
  /**
   * Open the clip's existing edit document in the editor.
   *
   * Null when the clip has none yet. Approving is what makes one — and opens
   * it — so this is the way back to an edit that already exists, offered
   * beside the approval rather than folded into it.
   */
  readonly onEdit: (() => void) | null;
}

/** A quote with its position, as a card whose timecode jumps the player. */
function QuoteCard({
  quote,
  label,
  onJump,
}: {
  readonly quote: Quote;
  readonly label: string;
  readonly onJump: (ticks: number) => void;
}) {
  return (
    <div className="flex items-start justify-between gap-3 rounded-[var(--cm-radius-control)] border border-[var(--cm-recessed-border)] bg-[var(--cm-recessed)] p-3">
      <div className="flex min-w-0 flex-col gap-1">
        <span className="text-[10px] tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
          {label}
        </span>
        <p className="text-[12px] leading-snug text-[var(--cm-text-primary)]">“{quote.text}”</p>
      </div>
      {quote.atTicks !== null && (
        <button
          type="button"
          onClick={() => onJump(quote.atTicks!)}
          title="Jump the player to this line"
          className="mono shrink-0 rounded px-1.5 py-0.5 text-[10px] transition-colors hover:bg-[var(--cm-accent-selected)]"
          style={{ color: 'var(--cm-accent)', background: 'var(--cm-accent-selected)' }}
        >
          {clock(quote.atTicks)}
        </button>
      )}
    </div>
  );
}

export function ClipInspector({
  rows,
  candidateId,
  proxyUrl,
  crop,
  cues,
  peaks,
  busy,
  notice,
  onSelect,
  onBack,
  onDecide,
  onUseAlternative,
  onTakeCut,
  onEdit,
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

  const measured = row.axes.filter((axis) => axis.value !== null);
  const hero = topFactors(row, 3);
  const heroKeys = new Set(hero.map((axis) => axis.axis));
  const detail = row.axes.filter((axis) => !heroKeys.has(axis.axis));
  const state = stateOf(row);

  // The reasons: what the proposer said opens and pays off the clip, then the
  // sentences the strongest factors were read from, without repeating one.
  const seen = new Set<string>();
  const reasons: { label: string; quote: Quote }[] = [];
  const add = (label: string, quote: Quote | null | undefined) => {
    if (quote && !seen.has(quote.text)) {
      seen.add(quote.text);
      reasons.push({ label, quote });
    }
  };
  add('Opens with', row.hook);
  add('Pays off with', row.payoff);
  for (const axis of hero) {
    add(axis.label, axis.evidence[0]);
  }

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
        <span
          className="shrink-0 rounded px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase"
          style={{ color: TONE_INK[state.tone], background: wash(state.tone) }}
        >
          {state.label}
        </span>
        <span className="mono ml-auto shrink-0 text-[11px] text-[var(--cm-text-muted)]">
          Rank {row.rank} of {rows.length}
          {row.proposer && <span className="font-sans"> · {row.proposer}</span>}
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
            peaks={peaks}
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
          className="glass flex w-[372px] shrink-0 flex-col overflow-hidden rounded-[var(--cm-radius-card)]"
          aria-label="Why this clip"
        >
          <Tabs defaultValue="score" className="flex min-h-0 flex-1 flex-col">
            <TabsList className="mx-3 mt-3 shrink-0">
              <TabsTrigger value="score">Score</TabsTrigger>
              <TabsTrigger value="evidence">Evidence</TabsTrigger>
              <TabsTrigger value="boundary">Boundary</TabsTrigger>
              <TabsTrigger value="risk">Risk</TabsTrigger>
            </TabsList>

            <div className="min-h-0 flex-1 overflow-y-auto p-4">
              <TabsContent value="score" className="mt-0 flex flex-col gap-6">
                <div className="flex items-center gap-5">
                  <ScoreRing
                    score={row.displayScore}
                    band={row.band}
                    size="lg"
                    caption={row.bandLabel}
                  />
                  <div className="flex min-w-0 flex-1 flex-col gap-2.5">
                    {hero.map((axis, index) => (
                      <div key={axis.axis} className="flex flex-col gap-1">
                        <div className="flex items-baseline justify-between">
                          <span className="text-[10px] tracking-[0.06em] text-[var(--cm-text-secondary)] uppercase">
                            {axis.label}
                          </span>
                          <span
                            className="mono text-[11px]"
                            style={{
                              color: index === 0 ? 'var(--cm-accent)' : 'var(--cm-text-primary)',
                            }}
                          >
                            {Math.round((axis.value ?? 0) * 100)}
                          </span>
                        </div>
                        <div className="h-1 overflow-hidden rounded-full bg-[var(--cm-recessed)]">
                          <div
                            className="h-full rounded-full"
                            style={{
                              width: `${Math.round((axis.value ?? 0) * 100)}%`,
                              background:
                                index === 0 ? 'var(--cm-accent)' : 'var(--cm-text-secondary)',
                              boxShadow:
                                index === 0
                                  ? '0 0 6px color-mix(in srgb, var(--cm-accent) 60%, transparent)'
                                  : undefined,
                              transition: 'width 640ms cubic-bezier(0.22, 1, 0.36, 1)',
                            }}
                          />
                        </div>
                      </div>
                    ))}
                    {hero.length === 0 && (
                      <p className="text-[11px] text-[var(--cm-text-muted)]">
                        No axis was measured for this clip.
                      </p>
                    )}
                  </div>
                </div>

                <div className="flex flex-col gap-3">
                  <h3 className="border-b border-[var(--cm-glass-border)] pb-1 text-[10px] tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
                    Detailed axis scores · {measured.length} of {row.axes.length} measured
                  </h3>
                  <div className="grid grid-cols-2 gap-x-4 gap-y-3">
                    {detail.map((axis) => (
                      <div key={axis.axis} className="flex flex-col gap-1">
                        <div className="flex items-baseline justify-between">
                          <span className="mono text-[11px] text-[var(--cm-text-secondary)]">
                            {axis.label}
                          </span>
                          <span className="mono text-[11px] text-[var(--cm-text-primary)]">
                            {axis.value === null ? '—' : Math.round(axis.value * 100)}
                          </span>
                        </div>
                        {axis.value === null ? (
                          <p
                            className="truncate text-[10px] text-[var(--cm-text-muted)]"
                            title={axis.unavailableReason ?? 'not measured'}
                          >
                            {axis.unavailableReason ?? 'not measured'}
                          </p>
                        ) : (
                          <div className="h-0.5 bg-[var(--cm-recessed)]">
                            <div
                              className="h-full bg-[var(--cm-text-secondary)]"
                              style={{ width: `${Math.round(axis.value * 100)}%` }}
                            />
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>

                {reasons.length > 0 && (
                  <div className="flex flex-col gap-3">
                    <h3 className="border-b border-[var(--cm-glass-border)] pb-1 text-[10px] tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
                      Why selected
                    </h3>
                    {reasons.map((reason) => (
                      <QuoteCard
                        key={`${reason.label}-${reason.quote.text}`}
                        label={reason.label}
                        quote={reason.quote}
                        onJump={scrub}
                      />
                    ))}
                  </div>
                )}
              </TabsContent>

              <TabsContent value="evidence" className="mt-0 flex flex-col gap-3">
                {row.axes.every((axis) => axis.evidence.length === 0) ? (
                  <p className="text-[12px] text-[var(--cm-text-muted)]">
                    No evidence index was published for this analysis, so the factors carry
                    positions but no text.
                  </p>
                ) : (
                  row.axes.flatMap((axis) =>
                    axis.evidence.map((quote, index) => (
                      <QuoteCard
                        key={`${axis.axis}-${index}`}
                        label={axis.label}
                        quote={quote}
                        onJump={scrub}
                      />
                    )),
                  )
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
                    <dl className="grid grid-cols-2 gap-x-3 gap-y-2 border-t border-[var(--cm-glass-border)] pt-3 text-[11px]">
                      <dt className="text-[var(--cm-text-muted)]">Length</dt>
                      <dd className="mono text-right text-[var(--cm-text-primary)]">
                        {duration(row.durationSeconds)}
                      </dd>
                      <dt className="text-[var(--cm-text-muted)]">Legal pairs</dt>
                      <dd className="mono text-right text-[var(--cm-text-primary)]">
                        {row.latticeStarts.length}×{row.latticeEnds.length}
                      </dd>
                      {row.clusterId && (
                        <>
                          <dt className="text-[var(--cm-text-muted)]">Cluster</dt>
                          <dd className="mono truncate text-right text-[var(--cm-text-primary)]">
                            {row.clusterId}
                          </dd>
                        </>
                      )}
                    </dl>
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
              {busy ? 'Working…' : onEdit ? 'Approve and open the edit' : 'Approve for the editor'}
            </Button>
            {onEdit && (
              <Button variant="outline" className="w-full justify-center gap-2" onClick={onEdit}>
                <Scissors className="size-4" aria-hidden />
                Open the existing edit
              </Button>
            )}
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
