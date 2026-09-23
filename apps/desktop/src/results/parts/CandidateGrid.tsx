/**
 * Candidate cards using the filmstrip tile nearest each clip's first frame.
 * Report missing filmstrips explicitly. State, signal, and selection rules are
 * shared with the table so layout changes preserve meaning.
 */
import { ArrowUpRight, TriangleAlert } from 'lucide-react';
import { useEffect, useRef } from 'react';

import { MediaStill } from '../../components/MediaStill.js';
import { Checkbox } from '../../components/ui/checkbox.js';
import { type ClipRow, clock, duration } from '../model.js';
import { TONE_INK, signalsFor, stateOf } from './state.js';

export interface CandidateGridProps {
  readonly rows: readonly ClipRow[];
  readonly focusedId: string | null;
  readonly checked: ReadonlySet<string>;
  readonly tileUrl: (atTicks: number) => string | null;
  readonly onFocus: (candidateId: string) => void;
  readonly onToggle: (candidateId: string) => void;
  readonly onOpen: (candidateId: string) => void;
}

export function CandidateGrid({
  rows,
  focusedId,
  checked,
  tileUrl,
  onFocus,
  onToggle,
  onOpen,
}: CandidateGridProps) {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const active = listRef.current?.querySelector('[aria-selected="true"]');
    if (active instanceof HTMLElement && typeof active.scrollIntoView === 'function') {
      active.scrollIntoView({ block: 'nearest' });
    }
  }, [focusedId]);

  return (
    <div
      ref={listRef}
      role="listbox"
      aria-label="Clip candidates"
      aria-activedescendant={focusedId ? `candidate-${focusedId}` : undefined}
      tabIndex={0}
      className="results-grid"
      onKeyDown={(event) => {
        if (event.target !== event.currentTarget || rows.length === 0) return;
        const at = rows.findIndex((row) => row.candidateId === focusedId);
        if (['ArrowDown', 'ArrowRight', 'j'].includes(event.key)) {
          event.preventDefault();
          onFocus(rows[Math.min(rows.length - 1, at + 1)]!.candidateId);
        } else if (['ArrowUp', 'ArrowLeft', 'k'].includes(event.key)) {
          event.preventDefault();
          onFocus(rows[Math.max(0, at - 1)]!.candidateId);
        } else if (event.key === ' ' && focusedId) {
          event.preventDefault();
          onToggle(focusedId);
        } else if (event.key === 'Enter' && focusedId) {
          event.preventDefault();
          onOpen(focusedId);
        }
      }}
    >
      {rows.map((row) => {
        const focused = row.candidateId === focusedId;
        const ticked = checked.has(row.candidateId);
        const state = stateOf(row);
        const signals = row.review ? [] : signalsFor(row);
        return (
          <div
            key={row.candidateId}
            id={`candidate-${row.candidateId}`}
            role="option"
            aria-selected={focused}
            aria-checked={ticked}
            onClick={() => onFocus(row.candidateId)}
            onDoubleClick={() => onOpen(row.candidateId)}
            className="results-card"
          >
            <div className="results-card-picture">
              <MediaStill src={tileUrl(row.startTicks)} />
              <span className="results-card-select" onClick={(event) => event.stopPropagation()}>
                <Checkbox
                  checked={ticked}
                  onCheckedChange={() => onToggle(row.candidateId)}
                  aria-label={`Select ${row.headline || 'this clip'} for a batch action`}
                />
              </span>
              <span className="results-card-duration mono">{duration(row.durationSeconds)}</span>
            </div>
            <div className="results-card-copy">
              <div className="results-card-meta">
                <span className="mono">
                  {String(row.rank).padStart(2, '0')} · {clock(row.startTicks)}
                </span>
                <span style={{ color: TONE_INK[state.tone] }}>{state.label}</span>
              </div>
              <p className="results-card-title">{row.headline || 'Untitled clip'}</p>
              <div className="results-card-bottom">
                {row.review ? (
                  <span className="results-card-review">Editorial review</span>
                ) : (
                  <span
                    role="group"
                    aria-label={
                      signals.length === 0 ? 'No signals recorded' : `${signals.length} signals`
                    }
                    className="results-card-signals"
                  >
                    {signals
                      .filter((signal) => signal.tone === 'warning' || signal.tone === 'danger')
                      .slice(0, 2)
                      .map((signal) => (
                        <span
                          key={signal.key}
                          title={signal.label}
                          style={{ color: TONE_INK[signal.tone] }}
                        >
                          <TriangleAlert size={13} aria-hidden />
                        </span>
                      ))}
                  </span>
                )}
                <button
                  type="button"
                  className="results-card-open"
                  aria-label={`Open ${row.headline || 'this clip'} in the inspector`}
                  onClick={(event) => {
                    event.stopPropagation();
                    onOpen(row.candidateId);
                  }}
                >
                  Review <ArrowUpRight size={14} aria-hidden />
                </button>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
