/**
 * The Clip Inspector: one clip, everything measured about it, and a decision.
 *
 * Three columns, and the middle one is the point. A score card can be read
 * anywhere; judging a clip means watching it, so the preview is the centre and
 * the numbers sit beside it rather than in front of it.
 *
 * Every number here was published by a stage and is being displayed, not
 * derived. An axis nothing measured shows the reason nothing did rather than a
 * zero, because a zero reads as a measurement of badness. Uncertainty is three
 * words rather than a shading of the score. And the boundary can be swapped for
 * the optimizer's runner-up in one click, because the runner-up is frequently
 * the editor's first choice and re-running the search to find it again would be
 * work the ranking already did.
 */
import { Check, ChevronLeft, Clock, X } from 'lucide-react';

import { Badge } from '../components/ui/badge.js';
import { Button } from '../components/ui/button.js';
import { ScrollArea } from '../components/ui/scroll-area.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import type { ClipDecision, CropPath } from '../daemon/client.js';
import { Preview, type OverlayCue } from '../inspector/Preview.js';
import { type ClipRow, clock } from '../results/model.js';

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

/** One axis, as a labelled bar or as the reason there is no bar. */
function AxisBar({ reading }: { readonly reading: ClipRow['axes'][number] }) {
  if (reading.value === null) {
    return (
      <div className="flex flex-col gap-1 py-1.5">
        <span className="text-xs text-[var(--cm-ink-2)]">{reading.label}</span>
        <span className="text-xs text-[var(--cm-ink-3)] italic">
          {reading.unavailableReason ?? 'not measured at this phase'}
        </span>
      </div>
    );
  }
  return (
    <div className="flex flex-col gap-1 py-1.5">
      <span className="flex items-baseline justify-between text-xs">
        <span className="text-[var(--cm-ink-2)]">{reading.label}</span>
        <span className="font-medium text-[var(--cm-ink-1)]">
          {Math.round(reading.value * 100)}
        </span>
      </span>
      <span className="h-1.5 w-full overflow-hidden rounded-full bg-[var(--cm-surface-2)]">
        <span
          className="block h-full rounded-full bg-[var(--cm-accent)]"
          style={{ width: `${Math.round(reading.value * 100)}%` }}
        />
      </span>
    </div>
  );
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
  const row = rows.find((candidate) => candidate.candidateId === candidateId) ?? rows[0];
  if (!row) {
    return <div className="p-8 text-sm text-[var(--cm-ink-2)]">This clip is no longer ranked.</div>;
  }
  const cited = row.axes.filter((axis) => axis.evidence.length > 0);

  return (
    <div className="grid h-full grid-cols-[minmax(200px,240px)_1fr_minmax(280px,340px)] gap-4 p-6">
      <ScrollArea className="rounded-xl border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)]">
        <div className="p-3">
          <Button variant="ghost" size="sm" className="mb-2 w-full justify-start" onClick={onBack}>
            <ChevronLeft className="size-4" /> Results
          </Button>
          <p className="px-2 pb-2 text-xs text-[var(--cm-ink-2)]">Candidates ({rows.length})</p>
          <ul className="flex flex-col gap-1">
            {rows.map((candidate) => (
              <li key={candidate.candidateId}>
                <button
                  type="button"
                  onClick={() => onSelect(candidate.candidateId)}
                  aria-current={candidate.candidateId === row.candidateId}
                  className="w-full rounded-lg px-3 py-2 text-left transition-colors aria-[current=true]:bg-[var(--cm-surface-2)] hover:bg-[var(--cm-surface-2)]"
                >
                  <span className="flex items-baseline justify-between gap-2">
                    <span className="truncate text-sm text-[var(--cm-ink-1)]">
                      {candidate.headline || `Clip ${candidate.rank}`}
                    </span>
                    <span className="text-xs text-[var(--cm-ink-2)]">{candidate.displayScore}</span>
                  </span>
                  <span className="text-xs text-[var(--cm-ink-3)]">
                    {clock(candidate.startTicks)} · {candidate.durationSeconds.toFixed(0)}s
                    {candidate.decision ? ` · ${candidate.decision}` : ''}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </div>
      </ScrollArea>

      <div className="flex flex-col items-center justify-start gap-4 rounded-xl border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] p-5">
        <Preview
          src={proxyUrl}
          startTicks={row.startTicks}
          endTicks={row.endTicks}
          crop={crop}
          cues={cues}
        />
        <section className="w-full max-w-[420px]" aria-label="Boundary">
          <div className="flex items-center justify-between text-xs text-[var(--cm-ink-2)]">
            <span>IN {clock(row.startTicks)}</span>
            <span className="font-medium text-[var(--cm-ink-1)]">
              {row.durationSeconds.toFixed(2)}s
            </span>
            <span>OUT {clock(row.endTicks)}</span>
          </div>
          {/* The lattice, drawn as the ticks a boundary may land on. Positions
              are relative to the span the edges themselves span, so the strip
              shows the choices the search actually had. */}
          <BoundaryStrip row={row} />
          {row.boundary?.alternative ? (
            <Button
              variant="outline"
              size="sm"
              className="mt-3 w-full"
              disabled={busy}
              onClick={onUseAlternative}
            >
              Use the runner-up: {clock(row.boundary.alternative.startTicks)}–
              {clock(row.boundary.alternative.endTicks)}
            </Button>
          ) : (
            <p className="mt-3 text-center text-xs text-[var(--cm-ink-3)]">
              This candidate&rsquo;s lattice offered one legal pair, so there is no alternative.
            </p>
          )}
        </section>
      </div>

      <div className="flex flex-col gap-3 rounded-xl border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] p-4">
        <Tabs defaultValue="score" className="min-h-0 flex-1">
          <TabsList className="w-full">
            <TabsTrigger value="score">Score</TabsTrigger>
            <TabsTrigger value="evidence">Evidence</TabsTrigger>
            <TabsTrigger value="boundary">Boundary</TabsTrigger>
            <TabsTrigger value="risk">Risk</TabsTrigger>
          </TabsList>

          <TabsContent value="score">
            <ScrollArea className="h-[420px] pr-3">
              <div className="flex items-center gap-3 py-3">
                <p className="text-3xl font-semibold text-[var(--cm-ink-1)]">{row.displayScore}</p>
                <Badge variant="outline">{row.bandLabel}</Badge>
              </div>
              {row.axes.map((axis) => (
                <AxisBar key={axis.axis} reading={axis} />
              ))}
            </ScrollArea>
          </TabsContent>

          <TabsContent value="evidence">
            <ScrollArea className="h-[420px] pr-3">
              {cited.length === 0 ? (
                <p className="py-3 text-sm text-[var(--cm-ink-2)]">
                  No factor cited a sentence. Without an evidence index there is nothing to quote.
                </p>
              ) : (
                cited.map((axis) => (
                  <div key={axis.axis} className="py-2">
                    <p className="text-xs text-[var(--cm-ink-2)]">{axis.label}</p>
                    {axis.evidence.map((sentence) => (
                      <p
                        key={sentence}
                        className="mt-1 rounded-lg bg-[var(--cm-surface-2)] px-3 py-2 text-sm text-[var(--cm-ink-1)]"
                      >
                        {sentence}
                      </p>
                    ))}
                  </div>
                ))
              )}
            </ScrollArea>
          </TabsContent>

          <TabsContent value="boundary">
            <ScrollArea className="h-[420px] pr-3">
              <p className="py-2 text-xs text-[var(--cm-ink-2)]">
                The weighted terms behind this cut, so a boundary you disagree with can be argued
                with.
              </p>
              {(row.boundary?.terms ?? []).map((term) => (
                <p key={term.name} className="flex justify-between py-1 text-sm">
                  <span className="text-[var(--cm-ink-2)]">{term.name.replaceAll('_', ' ')}</span>
                  <span className="text-[var(--cm-ink-1)]">{term.value.toFixed(3)}</span>
                </p>
              ))}
            </ScrollArea>
          </TabsContent>

          <TabsContent value="risk">
            <ScrollArea className="h-[420px] pr-3">
              {row.warnings.length === 0 && row.penalties.length === 0 ? (
                <p className="py-3 text-sm text-[var(--cm-ink-2)]">
                  Nothing was flagged about this clip.
                </p>
              ) : (
                <>
                  {row.warnings.map((warning) => (
                    <p key={warning} className="py-1 text-sm text-[var(--cm-warning-ink)]">
                      {warning}
                    </p>
                  ))}
                  {row.penalties.map((penalty) => (
                    <p key={penalty.reason} className="flex justify-between py-1 text-sm">
                      <span className="text-[var(--cm-ink-2)]">
                        {penalty.reason.replaceAll('_', ' ')}
                      </span>
                      <span className="text-[var(--cm-danger-ink)]">
                        −{penalty.value.toFixed(3)}
                      </span>
                    </p>
                  ))}
                </>
              )}
            </ScrollArea>
          </TabsContent>
        </Tabs>

        {notice && <p className="text-xs text-[var(--cm-ink-2)]">{notice}</p>}
        <div className="flex flex-col gap-2">
          <Button disabled={busy} onClick={() => onDecide('approved')}>
            <Check className="size-4" /> Approve for editor
          </Button>
          <div className="grid grid-cols-2 gap-2">
            <Button variant="outline" disabled={busy} onClick={() => onDecide('kept')}>
              <Clock className="size-4" /> Keep for later
            </Button>
            <Button variant="ghost" disabled={busy} onClick={() => onDecide('rejected')}>
              <X className="size-4" /> Reject
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

/** The lattice edges a boundary may land on, and where this one landed. */
function BoundaryStrip({ row }: { readonly row: ClipRow }) {
  const points = [...row.latticeStarts, ...row.latticeEnds];
  const from = Math.min(row.startTicks, ...points);
  const to = Math.max(row.endTicks, ...points);
  const span = Math.max(1, to - from);
  const position = (at: number) => `${((at - from) / span) * 100}%`;

  return (
    <div className="relative mt-2 h-10 rounded-lg bg-[var(--cm-surface-2)]">
      <div
        className="absolute inset-y-0 rounded-lg bg-[var(--cm-accent)]/25"
        style={{
          left: position(row.startTicks),
          right: `${100 - Number.parseFloat(position(row.endTicks))}%`,
        }}
      />
      {row.latticeStarts.map((at) => (
        <span
          key={`start-${at}`}
          title={`A legal start at ${clock(at)}`}
          className="absolute top-0 h-3 w-px bg-[var(--cm-ink-3)]"
          style={{ left: position(at) }}
        />
      ))}
      {row.latticeEnds.map((at) => (
        <span
          key={`end-${at}`}
          title={`A legal end at ${clock(at)}`}
          className="absolute bottom-0 h-3 w-px bg-[var(--cm-ink-3)]"
          style={{ left: position(at) }}
        />
      ))}
      <span
        className="absolute inset-y-0 w-0.5 bg-[var(--cm-accent)]"
        style={{ left: position(row.startTicks) }}
      />
      <span
        className="absolute inset-y-0 w-0.5 bg-[var(--cm-accent)]"
        style={{ left: position(row.endTicks) }}
      />
    </div>
  );
}
