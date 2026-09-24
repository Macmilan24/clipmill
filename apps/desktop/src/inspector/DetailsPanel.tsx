/**
 * Clip facts, overlapping candidates, and the boundary search's diagnostics,
 * which stay folded until asked for.
 */
import type { ClipRow } from '../results/model.js';
import { type Cut, type Overlap, clockTenths, lengthLabel, sameCut } from './review.js';

export interface DetailsPanelProps {
  readonly row: ClipRow;
  readonly total: number;
  readonly cut: Cut;
  readonly overlaps: readonly Overlap[];
  readonly durationTarget: { readonly minTicks: number; readonly maxTicks: number } | null;
  readonly onSelect: (candidateId: string) => void;
}

export function DetailsPanel({
  row,
  total,
  cut,
  overlaps,
  durationTarget,
  onSelect,
}: DetailsPanelProps) {
  const chosen = { startTicks: row.startTicks, endTicks: row.endTicks };
  const moved = !sameCut(cut, chosen);
  const length = cut.endTicks - cut.startTicks;
  const outside =
    durationTarget && (length < durationTarget.minTicks || length > durationTarget.maxTicks)
      ? `${length < durationTarget.minTicks ? 'Shorter' : 'Longer'} than the ${lengthLabel(durationTarget.minTicks)}–${lengthLabel(durationTarget.maxTicks)} this analysis aimed for.`
      : null;

  return (
    <div className="review-panel-body">
      <dl className="review-facts">
        <div>
          <dt>Length</dt>
          <dd className="mono">{lengthLabel(length)}</dd>
        </div>
        <div>
          <dt>{moved ? 'Your cut' : 'Cut'}</dt>
          <dd className="mono">
            {clockTenths(cut.startTicks)} – {clockTenths(cut.endTicks)}
          </dd>
        </div>
        {moved && (
          <div>
            <dt>Suggested cut</dt>
            <dd className="mono">
              {clockTenths(chosen.startTicks)} – {clockTenths(chosen.endTicks)}
            </dd>
          </div>
        )}
        <div>
          <dt>Rank</dt>
          <dd>
            {row.rank} of {total}
            {row.recommended ? ' · recommended' : ''}
          </dd>
        </div>
        <div>
          <dt>Found by</dt>
          <dd>{row.review ? 'Editorial model' : (row.proposer ?? 'Analysis')}</dd>
        </div>
        <div>
          <dt>Edit</dt>
          <dd>{row.docId ? 'Has an edit' : 'None yet'}</dd>
        </div>
      </dl>
      {outside && (
        <p className="review-callout" data-tone="muted">
          {outside} That is fine — it is your cut.
        </p>
      )}

      <section className="review-section">
        <h3 className="review-section-title">Covers the same ground as</h3>
        {overlaps.length === 0 ? (
          <p className="review-footnote">No other clip in this review overlaps it.</p>
        ) : (
          <ul className="review-overlaps">
            {overlaps.map((overlap) => (
              <li key={overlap.candidateId}>
                <button type="button" onClick={() => onSelect(overlap.candidateId)}>
                  Clip {String(overlap.rank).padStart(2, '0')}
                </button>
                <span className="mono">{Math.round(overlap.share * 100)}% shared</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      <details className="review-disclosure">
        <summary>Diagnostics</summary>
        <dl className="review-facts review-facts-technical">
          <div>
            <dt>Suggested edges</dt>
            <dd className="mono">
              {row.latticeStarts.length} starts · {row.latticeEnds.length} ends
            </dd>
          </div>
          {row.boundary && (
            <div>
              <dt>Boundary score</dt>
              <dd className="mono">{row.boundary.score.toFixed(3)}</dd>
            </div>
          )}
          {(row.review ? [] : (row.boundary?.terms ?? [])).map((term) => (
            <div key={term.name}>
              <dt>{term.name.replaceAll('_', ' ')}</dt>
              <dd className="mono">{term.value.toFixed(3)}</dd>
            </div>
          ))}
          {row.clusterId && (
            <div>
              <dt>Cluster</dt>
              <dd className="mono">{row.clusterId}</dd>
            </div>
          )}
          <div>
            <dt>Candidate</dt>
            <dd className="mono">{row.candidateId}</dd>
          </div>
        </dl>
      </details>
    </div>
  );
}
