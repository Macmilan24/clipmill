/**
 * Why a candidate was proposed: the editorial review or the scored factors, the
 * source lines behind them, and the checks recorded against the clip.
 */
import { Check, TriangleAlert } from 'lucide-react';

import { type ClipRow, type Quote, clock, topFactors } from '../results/model.js';
import { ScoreRing } from '../results/parts/ScoreRing.js';

export interface WhyPanelProps {
  readonly row: ClipRow;
  readonly onJump: (ticks: number) => void;
}

export function WhyPanel({ row, onJump }: WhyPanelProps) {
  const review = row.review;
  // Editorial reasons are joined into the warnings too, for the board's dots.
  // Here they are already read out above, so the checks list the rest.
  const checks = row.warnings.filter((warning) => !review?.reasons.includes(warning));
  const hero = review ? [] : topFactors(row, 3);
  const quotes = uniqueQuotes(row, hero);
  const measured = row.axes.filter((axis) => axis.value !== null);

  return (
    <div className="review-panel-body">
      {review?.status === 'rejected' && (
        <p className="review-callout" data-tone="warning">
          The editorial review did not recommend this moment. You can still approve it — the review
          is kept with the edit.
        </p>
      )}

      {review ? (
        <section className="review-section">
          <span className="review-verdict" data-band={row.band}>
            {row.band === 'strong' ? (
              <Check aria-hidden="true" />
            ) : (
              <TriangleAlert aria-hidden="true" />
            )}
            {row.bandLabel}
          </span>
          {review.summary && <p className="review-summary">{review.summary}</p>}
          {review.reasons.length > 0 && (
            <ul className="review-reasons">
              {review.reasons.map((reason, index) => (
                // eslint-disable-next-line react/no-array-index-key -- reasons are sentences in order
                <li key={index}>{reason}</li>
              ))}
            </ul>
          )}
          <p className="review-footnote">
            {review.route === 'cloud'
              ? 'Reviewed with cloud assistance'
              : 'Reviewed on this device'}
            {' · '}Your decision is final.
          </p>
        </section>
      ) : (
        <section className="review-section review-score">
          <ScoreRing score={row.displayScore} band={row.band} size="md" caption={row.bandLabel} />
          <div className="review-factors">
            {hero.map((axis) => (
              <div key={axis.axis} className="review-factor">
                <span>{axis.label}</span>
                <span className="mono">{Math.round((axis.value ?? 0) * 100)}</span>
                <span className="review-factor-bar">
                  <span style={{ width: `${Math.round((axis.value ?? 0) * 100)}%` }} />
                </span>
              </div>
            ))}
            {hero.length === 0 && <p className="review-footnote">No factor was measured.</p>}
          </div>
        </section>
      )}

      {quotes.length > 0 && (
        <section className="review-section">
          <h3 className="review-section-title">From the recording</h3>
          <ul className="review-quotes">
            {quotes.map(({ label, quote }) => (
              <li key={`${label}:${quote.text}`}>
                <span className="review-quote-label">{label}</span>
                <p>“{quote.text}”</p>
                {quote.atTicks !== null && (
                  <button
                    type="button"
                    className="review-jump mono"
                    onClick={() => onJump(quote.atTicks!)}
                    aria-label={`Play from ${clock(quote.atTicks)}`}
                  >
                    {clock(quote.atTicks)}
                  </button>
                )}
              </li>
            ))}
          </ul>
        </section>
      )}

      <section className="review-section">
        <h3 className="review-section-title">Checks</h3>
        {checks.length === 0 && row.penalties.length === 0 ? (
          <p className="review-check" data-tone="success">
            <Check aria-hidden="true" />
            Nothing was recorded against this clip.
          </p>
        ) : (
          <ul className="review-checks">
            {checks.map((warning) => (
              <li key={warning} className="review-check" data-tone="warning">
                <TriangleAlert aria-hidden="true" />
                {warning}
              </li>
            ))}
            {row.penalties.map((penalty) => (
              <li key={penalty.reason} className="review-check" data-tone="danger">
                <TriangleAlert aria-hidden="true" />
                <span>{penalty.reason.replaceAll('_', ' ')}</span>
                <span className="mono">−{penalty.value}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      {!review && row.axes.length > 0 && (
        <details className="review-disclosure">
          <summary>
            Every factor · {measured.length} of {row.axes.length} measured
          </summary>
          <dl className="review-axes">
            {row.axes.map((axis) => (
              <div key={axis.axis}>
                <dt>{axis.label}</dt>
                <dd className="mono">{axis.value === null ? '—' : Math.round(axis.value * 100)}</dd>
                {axis.value === null && (
                  <p className="review-footnote">{axis.unavailableReason ?? 'Not measured.'}</p>
                )}
              </div>
            ))}
          </dl>
        </details>
      )}
    </div>
  );
}

/** The opening, the payoff and each strong factor's line, never twice. */
function uniqueQuotes(
  row: ClipRow,
  hero: ReturnType<typeof topFactors>,
): { label: string; quote: Quote }[] {
  const seen = new Set<string>();
  const found: { label: string; quote: Quote }[] = [];
  const add = (label: string, quote: Quote | null | undefined) => {
    if (quote && quote.text && !seen.has(quote.text)) {
      seen.add(quote.text);
      found.push({ label, quote });
    }
  };
  add('Opens with', row.hook);
  add('Pays off with', row.payoff);
  for (const axis of hero) add(axis.label, axis.evidence[0]);
  for (const axis of row.axes) for (const quote of axis.evidence) add(axis.label, quote);
  return found;
}
