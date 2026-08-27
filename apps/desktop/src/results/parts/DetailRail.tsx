/**
 * The selected clip, without leaving the board.
 *
 * The still is the project's own proxy seeked to the clip's first frame — a real
 * frame of the real recording, served over the media protocol. There is no
 * placeholder image anywhere in here: a card that cannot show a frame says the
 * project published no proxy, because a stock thumbnail would be a picture of
 * something that is not this clip.
 *
 * "Why it ranked here" lists the axes that most moved the total, each with the
 * sentence it was read from. That ordering is by weighted contribution, so the
 * reasons given are the reasons the number is what it is rather than simply the
 * highest-scoring axes.
 */
import { ArrowRight, FileVideo, Quote } from 'lucide-react';

import { Button } from '../../components/ui/button.js';
import { type ClipRow, clock, duration, topFactors } from '../model.js';
import { ScoreRing } from './ScoreRing.js';

export interface DetailRailProps {
  readonly row: ClipRow | null;
  /** The proxy's URL, or null when this project published none. */
  readonly proxyUrl: string | null;
  readonly approvedCount: number;
  readonly onOpen: (candidateId: string) => void;
}

const SECONDS = 90_000;

export function DetailRail({ row, proxyUrl, approvedCount, onOpen }: DetailRailProps) {
  if (!row) {
    return (
      <aside className="glass hidden min-h-0 flex-col items-center justify-center gap-2 rounded-[var(--cm-radius-card)] p-8 text-center xl:flex">
        <FileVideo className="size-6 text-[var(--cm-text-muted)]" aria-hidden />
        <p className="text-[13px] text-[var(--cm-text-secondary)]">
          Select a clip to see why it ranked where it did.
        </p>
      </aside>
    );
  }

  const factors = topFactors(row);
  const startSeconds = Math.max(0, row.startTicks / SECONDS);

  return (
    <aside
      className="hidden min-h-0 flex-col gap-4 overflow-y-auto xl:flex"
      aria-label="Selected clip"
    >
      <div className="glass shrink-0 overflow-hidden rounded-[var(--cm-radius-card)]">
        <div className="relative aspect-video w-full bg-[var(--cm-recessed)]">
          {proxyUrl ? (
            <video
              // The fragment seeks the proxy to this clip's first frame, so the
              // still is this clip rather than the top of the recording.
              key={`${proxyUrl}#${startSeconds}`}
              src={`${proxyUrl}#t=${startSeconds.toFixed(2)}`}
              preload="metadata"
              muted
              playsInline
              className="size-full object-cover"
            />
          ) : (
            <div className="grid size-full place-items-center">
              <p className="px-4 text-center text-[11px] text-[var(--cm-text-muted)]">
                This project published no proxy, so there is no frame to show.
              </p>
            </div>
          )}
          {/* The scrim exists to keep the duration legible over a frame. With no
              frame under it there is nothing to darken, and in light mode it
              reads as a black slab dropped on the card. */}
          {proxyUrl && (
            <>
              <div
                aria-hidden
                className="pointer-events-none absolute inset-0"
                style={{ background: 'linear-gradient(to top, rgba(0,0,0,0.72), transparent 62%)' }}
              />
              <span className="mono absolute right-3 bottom-3 rounded bg-black/60 px-2 py-1 text-[11px] text-white backdrop-blur-sm">
                {duration(row.durationSeconds)}
              </span>
            </>
          )}
          {!proxyUrl && (
            <span className="mono absolute right-3 bottom-3 rounded bg-[var(--cm-glass-elevated)] px-2 py-1 text-[11px] text-[var(--cm-text-secondary)]">
              {duration(row.durationSeconds)}
            </span>
          )}
        </div>

        <div className="flex flex-col gap-4 p-4">
          <div className="flex items-start gap-3">
            <ScoreRing score={row.displayScore} band={row.band} size="md" />
            <div className="flex min-w-0 flex-col gap-1">
              <h3 className="text-[13px] leading-snug font-semibold text-[var(--cm-text-primary)]">
                {row.headline || 'No opening line indexed'}
              </h3>
              <span className="text-[11px] text-[var(--cm-text-secondary)]">{row.bandLabel}</span>
            </div>
          </div>

          <dl className="grid grid-cols-2 gap-x-3 gap-y-3 border-t border-[var(--cm-glass-border)] pt-4">
            {[
              ['In', clock(row.startTicks)],
              ['Out', clock(row.endTicks)],
              ['Rank', `${row.rank}`],
              ['Format', '9:16'],
            ].map(([label, value]) => (
              <div key={label} className="flex flex-col gap-1">
                <dt className="text-[10px] tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
                  {label}
                </dt>
                <dd className="mono text-[12px] text-[var(--cm-text-primary)]">{value}</dd>
              </div>
            ))}
          </dl>
        </div>
      </div>

      {factors.length > 0 && (
        <div className="glass flex shrink-0 flex-col gap-3 rounded-[var(--cm-radius-card)] p-4">
          <h4 className="text-[10px] font-medium tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
            Why it ranked here
          </h4>
          {factors.map((factor, index) => (
            <div
              key={factor.axis}
              className={`flex flex-col gap-1.5 ${index > 0 ? 'border-t border-[var(--cm-glass-border)] pt-3' : ''}`}
            >
              <div className="flex items-baseline justify-between gap-2">
                <span className="text-[12px] font-medium text-[var(--cm-text-primary)]">
                  {factor.label}
                </span>
                <span className="mono text-[11px] text-[var(--cm-text-secondary)]">
                  {Math.round((factor.value ?? 0) * 100)}
                </span>
              </div>
              {factor.evidence[0] && (
                <p className="flex gap-1.5 text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
                  <Quote className="mt-0.5 size-3 shrink-0 opacity-50" aria-hidden />
                  <span className="line-clamp-3 italic">{factor.evidence[0]}</span>
                </p>
              )}
            </div>
          ))}
        </div>
      )}

      <div className="glass-elevated mt-auto flex shrink-0 flex-col gap-3 rounded-[var(--cm-radius-card)] p-4">
        <div className="flex items-center justify-between text-[12px]">
          <span className="text-[var(--cm-text-secondary)]">
            Approved{' '}
            <span className="mono ml-1 text-[var(--cm-text-primary)]">{approvedCount}</span>
          </span>
        </div>
        <Button className="w-full justify-center gap-2" onClick={() => onOpen(row.candidateId)}>
          Open in the inspector
          <ArrowRight className="size-4" aria-hidden />
        </Button>
      </div>
    </aside>
  );
}
