/**
 * The review queue: every candidate in ranking order with its opening frame,
 * length and decision, plus review progress and the auto-advance preference.
 */
import { Bookmark, Check, Film, X } from 'lucide-react';
import { useEffect, useRef } from 'react';

import { Switch } from '../components/ui/switch.js';
import { type ClipRow, duration } from '../results/model.js';
import { thumbnailPosition } from '../results/thumbnails.js';
import { overlapsOf, progress } from './review.js';

export type QueueFilter = 'all' | 'undecided';

export interface QueueProps {
  readonly rows: readonly ClipRow[];
  readonly candidateId: string;
  readonly filter: QueueFilter;
  readonly onFilter: (filter: QueueFilter) => void;
  readonly tileUrl: (atTicks: number) => string | null;
  /** Where each clip's camera would point, by candidate, for its thumbnail. */
  readonly framing?: ReadonlyMap<string, number> | undefined;
  readonly busy: boolean;
  readonly onSelect: (candidateId: string) => void;
  readonly autoAdvance: boolean;
  readonly onAutoAdvance: (on: boolean) => void;
}

const DECISION = {
  approved: { label: 'Approved', icon: Check, tone: 'success' },
  kept: { label: 'Kept for later', icon: Bookmark, tone: 'warning' },
  rejected: { label: 'Rejected', icon: X, tone: 'danger' },
} as const;

export function Queue({
  rows,
  candidateId,
  filter,
  onFilter,
  tileUrl,
  framing,
  busy,
  onSelect,
  autoAdvance,
  onAutoAdvance,
}: QueueProps) {
  const counts = progress(rows);
  const shown = rows.filter(
    (row) => filter === 'all' || row.decision === null || row.candidateId === candidateId,
  );
  const active = useRef<HTMLButtonElement>(null);
  // Keep the open clip in view as the review moves through a long list.
  useEffect(() => {
    active.current?.scrollIntoView?.({ block: 'nearest' });
  }, [candidateId]);

  return (
    <nav className="review-queue" aria-label="Clips in this review">
      <header className="review-queue-head">
        <div className="review-queue-title">
          <span>Clips</span>
          <span className="mono">{rows.length}</span>
        </div>
        <div
          className="review-segmented review-segmented-quiet"
          role="group"
          aria-label="Which clips to list"
        >
          <button type="button" aria-pressed={filter === 'all'} onClick={() => onFilter('all')}>
            All
          </button>
          <button
            type="button"
            aria-pressed={filter === 'undecided'}
            onClick={() => onFilter('undecided')}
          >
            To review
            <span className="mono">{rows.length - counts.decided}</span>
          </button>
        </div>
        <div
          className="review-progress"
          aria-label={`${counts.decided} of ${counts.total} reviewed`}
        >
          <span
            className="review-progress-bar"
            style={{ width: `${counts.total === 0 ? 0 : (counts.decided / counts.total) * 100}%` }}
          />
        </div>
        <p className="review-progress-text">
          {counts.decided} of {counts.total} reviewed
          {counts.approved > 0 && ` · ${counts.approved} approved`}
        </p>
      </header>

      <ol className="review-queue-list">
        {shown.map((row) => {
          const current = row.candidateId === candidateId;
          const mark = row.decision ? DECISION[row.decision] : null;
          const still = tileUrl(row.startTicks);
          const repeated = overlapsOf(rows, row).find((other) => other.rank < row.rank);
          return (
            <li key={row.candidateId}>
              <button
                ref={current ? active : undefined}
                type="button"
                className="review-queue-item"
                aria-current={current ? 'true' : undefined}
                disabled={busy && !current}
                onClick={() => onSelect(row.candidateId)}
              >
                <span className="review-queue-still">
                  {still ? (
                    <img
                      src={still}
                      alt=""
                      loading="lazy"
                      style={{
                        objectPosition: thumbnailPosition(framing?.get(row.candidateId) ?? 0.5),
                      }}
                    />
                  ) : (
                    <Film aria-hidden="true" />
                  )}
                  <span className="mono">{duration(row.durationSeconds)}</span>
                </span>
                <span className="review-queue-text">
                  <span className="review-queue-headline">{row.headline || 'Untitled clip'}</span>
                  <span className="review-queue-meta">
                    <span className="mono">{String(row.rank).padStart(2, '0')}</span>
                    {mark ? (
                      <span className="review-queue-mark" data-tone={mark.tone}>
                        <mark.icon aria-hidden="true" />
                        {mark.label}
                      </span>
                    ) : (
                      <span>{row.bandLabel}</span>
                    )}
                    {repeated && (
                      <span
                        className="review-queue-repeat"
                        title={`Covers the same ground as clip ${String(repeated.rank).padStart(2, '0')}`}
                      >
                        covers {String(repeated.rank).padStart(2, '0')}
                      </span>
                    )}
                  </span>
                </span>
              </button>
            </li>
          );
        })}
        {shown.length === 0 && <li className="review-queue-empty">Every clip has a decision.</li>}
      </ol>

      <footer className="review-queue-foot">
        <label className="review-switch" htmlFor="review-auto-advance">
          <Switch id="review-auto-advance" checked={autoAdvance} onCheckedChange={onAutoAdvance} />
          Next clip after deciding
        </label>
      </footer>
    </nav>
  );
}
