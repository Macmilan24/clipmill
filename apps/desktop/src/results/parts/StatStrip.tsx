/**
 * Run counts from ranking results and recorded decisions, with shortfall reasons
 * when the recommended set is smaller than requested.
 */
import { AlertTriangle } from 'lucide-react';

import type { Summary, Tallies } from '../model.js';

export interface StatStripProps {
  readonly summary: Summary;
  readonly tallies: Tallies;
  readonly bestScore: number | null;
}

export function StatStrip({ summary, tallies, bestScore }: StatStripProps) {
  return (
    <section className="results-summary" aria-label="Run summary">
      <details className="results-summary-details">
        <summary>
          Analysis details{' '}
          <span>
            {tallies.recommended} recommended · {tallies.approved} approved
          </span>
        </summary>
        <div className="results-summary-content">
          <dl>
            <div>
              <dt>Candidates</dt>
              <dd>{summary.cohort}</dd>
            </div>
            <div>
              <dt>Recommended</dt>
              <dd>{tallies.recommended}</dd>
            </div>
            <div>
              <dt>Approved</dt>
              <dd>{tallies.approved}</dd>
            </div>
            <div>
              <dt>Flagged</dt>
              <dd>{tallies.flagged}</dd>
            </div>
            {(summary.declined ?? 0) > 0 && (
              <div>
                <dt>Declined</dt>
                <dd>{summary.declined}</dd>
              </div>
            )}
            {bestScore !== null && (
              <div>
                <dt>Best score</dt>
                <dd>{bestScore}</dd>
              </div>
            )}
          </dl>
        </div>
      </details>
      {(summary.selected < summary.requested || summary.shortfall.length > 0) && (
        <div role="status" aria-label="Clip selection" className="results-caution">
          <AlertTriangle size={15} aria-hidden />
          <p>
            {summary.requested} asked for, {summary.selected} recommended
            {summary.shortfall.length > 0 ? `: ${summary.shortfall.join('; ')}` : '.'}
          </p>
        </div>
      )}
      {(summary.warnings?.length ?? 0) > 0 && (
        <div role="status" aria-label="Incomplete analysis" className="results-caution">
          <AlertTriangle size={15} aria-hidden />
          <div>
            <p className="font-medium">Some analysis is incomplete</p>
            {summary.warnings?.map((warning) => (
              <p key={warning}>{warning}</p>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
