/**
 * The selected clip, without leaving the board.
 *
 * The still is the filmstrip tile the ingest cut nearest this clip's first
 * frame — a real frame of the real recording, served over the media protocol.
 * There is no placeholder image anywhere in here: a card that cannot show a
 * frame says the run published no filmstrip, because a stock thumbnail would be
 * a picture of something that is not this clip.
 *
 * The four facts under the title are the design's four slots filled with what
 * this system actually publishes. "Confidence" is the ranker's band, which is
 * the word the book uses for it; "Risk" is what the ranker recorded against the
 * clip, or the fact that it recorded nothing. The design's "Format 9:16" was a
 * constant, and a constant is not a fact about a clip.
 *
 * "Why it ranked here" lists the axes that most moved the total, each with the
 * sentence it was read from and where in the recording that sentence is. That
 * ordering is by weighted contribution, so the reasons given are the reasons the
 * number is what it is.
 */
import { MediaStill } from '../../components/MediaStill.js';
import {
  ArrowRight,
  CheckCheck,
  Fingerprint,
  Gauge,
  Lightbulb,
  type LucideIcon,
  MessageSquareQuote,
  Scissors,
  Sparkles,
  Waves,
  Wrench,
} from 'lucide-react';

import { Button } from '../../components/ui/button.js';
import { type Axis, type ClipRow, clock, duration, topFactors } from '../model.js';
import { ScoreRing } from './ScoreRing.js';
import { TONE_INK, stateOf } from './state.js';

/** One icon per axis, so the reasons read at a glance the way the design draws them. */
const AXIS_ICON: Readonly<Record<Axis, LucideIcon>> = {
  hook: Sparkles,
  flow: Waves,
  value: Lightbulb,
  prompt_relevance: MessageSquareQuote,
  novelty: Fingerprint,
  evidence: MessageSquareQuote,
  craft: Wrench,
  feasibility: Gauge,
};

export interface DetailRailProps {
  readonly row: ClipRow | null;
  readonly tileUrl: (atTicks: number) => string | null;
  readonly approvedCount: number;
  /** How many rows are ticked for the batch action, and the action itself. */
  readonly checkedCount: number;
  readonly busy: boolean;
  readonly onApproveChecked: () => void;
  readonly onOpen: (candidateId: string) => void;
  /** Open the clip's edit document. Offered only for a row that has one. */
  readonly onEdit: (candidateId: string) => void;
}

export function DetailRail({
  row,
  tileUrl,
  approvedCount,
  checkedCount,
  busy,
  onApproveChecked,
  onOpen,
  onEdit,
}: DetailRailProps) {
  if (!row) {
    return (
      <aside className="glass flex min-h-0 flex-col items-center justify-center gap-2 rounded-[var(--cm-radius-card)] p-8 text-center">
        <p className="text-[13px] text-[var(--cm-text-secondary)]">
          Select a clip to see why it ranked where it did.
        </p>
      </aside>
    );
  }

  const factors = row.review ? [] : topFactors(row);
  const still = tileUrl(row.startTicks);
  const state = stateOf(row);

  return (
    <aside className="results-detail" aria-label="Selected clip">
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
        <div className="glass shrink-0 overflow-hidden rounded-[var(--cm-radius-card)]">
          <div className="relative aspect-video w-full bg-[var(--cm-recessed)]">
            <MediaStill src={still} />
            {still && (
              <div
                aria-hidden
                className="pointer-events-none absolute inset-0"
                style={{ background: 'linear-gradient(to top, rgba(0,0,0,0.72), transparent 60%)' }}
              />
            )}
            <span
              className="absolute top-3 left-3 rounded px-2 py-0.5 text-[10px] font-bold tracking-wide uppercase"
              style={{
                color: 'white',
                background: state.tone === 'accent' ? 'var(--cm-accent)' : TONE_INK[state.tone],
              }}
            >
              {state.label}
            </span>
            <span
              className="mono absolute right-3 bottom-3 rounded px-2 py-1 text-[11px]"
              style={
                still
                  ? { background: 'rgba(0,0,0,0.6)', color: 'white' }
                  : { background: 'var(--cm-glass-elevated)', color: 'var(--cm-text-secondary)' }
              }
            >
              {duration(row.durationSeconds)}
            </span>
          </div>

          <div className="flex flex-col gap-4 p-4">
            <div className="flex items-start gap-3">
              {!row.review && <ScoreRing score={row.displayScore} band={row.band} size="md" />}
              <div className="flex min-w-0 flex-col gap-1">
                <h3 className="line-clamp-3 text-[13px] leading-snug font-semibold text-[var(--cm-text-primary)]">
                  {row.headline || 'No opening line indexed'}
                </h3>
              </div>
            </div>

            <dl className="grid grid-cols-2 gap-x-3 gap-y-3 border-t border-[var(--cm-glass-border)] pt-4">
              <Fact label="Review" value={row.bandLabel} />
              <Fact label="Length" value={duration(row.durationSeconds)} />
              <Fact
                label="Risk"
                value={
                  row.warnings.length + row.penalties.length === 0
                    ? 'None recorded'
                    : `${row.warnings.length + row.penalties.length} noted`
                }
                ink={row.flagged ? TONE_INK.warning : undefined}
              />
              <Fact label="Window" value={`${clock(row.startTicks)} – ${clock(row.endTicks)}`} />
            </dl>
          </div>
        </div>

        {row.review && (
          <section className="px-1">
            <h4 className="mb-2 text-[12px] font-medium">Why this moment</h4>
            <p className="text-[12px] leading-relaxed text-[var(--cm-text-secondary)]">
              {row.review.summary ||
                row.review.reasons[0] ||
                'Watch this moment to decide whether it belongs in your edit.'}
            </p>
            {row.warnings.length > 0 && (
              <p className="mt-3 border-l-2 border-[var(--cm-warning-ink)] pl-3 text-[11px] leading-relaxed text-[var(--cm-warning-ink)]">
                {row.warnings[0]}
              </p>
            )}
          </section>
        )}

        {factors.length > 0 && (
          <div className="glass flex shrink-0 flex-col gap-3 rounded-[var(--cm-radius-card)] p-4">
            <h4 className="text-[10px] font-medium tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
              Why it ranked {row.rank === 1 ? 'first' : `#${row.rank}`}
            </h4>
            {factors.map((factor, index) => {
              const Icon = AXIS_ICON[factor.axis];
              const evidence = factor.evidence[0];
              return (
                <div
                  key={factor.axis}
                  className={`flex gap-3 ${index > 0 ? 'border-t border-[var(--cm-glass-border)] pt-3' : ''}`}
                >
                  <Icon
                    className="mt-0.5 size-4 shrink-0"
                    aria-hidden
                    style={{ color: index === 0 ? TONE_INK.success : TONE_INK.accent }}
                  />
                  <div className="flex min-w-0 flex-col gap-0.5">
                    <div className="flex items-baseline justify-between gap-2">
                      <span className="text-[12px] font-medium text-[var(--cm-text-primary)]">
                        {factor.label}
                      </span>
                      <span className="mono text-[11px] text-[var(--cm-text-secondary)]">
                        {Math.round((factor.value ?? 0) * 100)}
                      </span>
                    </div>
                    {evidence ? (
                      <p className="text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
                        <span className="line-clamp-2 italic">“{evidence.text}”</span>
                        {evidence.atTicks !== null && (
                          <span className="mono mt-0.5 block text-[10px] text-[var(--cm-accent-ink)]">
                            {clock(evidence.atTicks)}
                          </span>
                        )}
                      </p>
                    ) : (
                      <p className="text-[11px] text-[var(--cm-text-muted)]">
                        Measured over the whole clip.
                      </p>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
      <div className="glass-elevated flex shrink-0 flex-col gap-3 rounded-[var(--cm-radius-card)] p-4">
        <div className="flex items-center justify-between text-[12px]">
          <span className="text-[var(--cm-text-primary)]">
            Selected{' '}
            <span className="mono ml-1 rounded bg-[var(--cm-recessed)] px-1.5 py-0.5">
              {checkedCount}
            </span>
          </span>
          <span className="text-[var(--cm-text-secondary)]">
            Approved{' '}
            <span className="mono ml-1 text-[var(--cm-text-primary)]">{approvedCount}</span>
          </span>
        </div>
        {checkedCount > 0 ? (
          <Button
            className="w-full justify-center gap-2"
            disabled={busy}
            onClick={onApproveChecked}
          >
            <CheckCheck className="size-4" aria-hidden />
            {busy ? 'Approving…' : `Approve ${checkedCount} selected`}
          </Button>
        ) : (
          <>
            {row.docId !== null && (
              <Button
                className="w-full justify-center gap-2"
                disabled={busy}
                onClick={() => onEdit(row.candidateId)}
              >
                <Scissors className="size-4" aria-hidden />
                Open in the editor
              </Button>
            )}
            <Button
              variant="outline"
              className="w-full justify-center gap-2"
              disabled={busy}
              onClick={() => onOpen(row.candidateId)}
            >
              Open in the inspector
              <ArrowRight className="size-4" aria-hidden />
            </Button>
          </>
        )}
      </div>
    </aside>
  );
}

function Fact({
  label,
  value,
  ink,
}: {
  readonly label: string;
  readonly value: string;
  readonly ink?: string | undefined;
}) {
  return (
    <div className="flex flex-col gap-1">
      <dt className="text-[10px] tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
        {label}
      </dt>
      <dd
        className="mono truncate text-[12px]"
        style={{ color: ink ?? 'var(--cm-text-primary)' }}
        title={value}
      >
        {value}
      </dd>
    </div>
  );
}
