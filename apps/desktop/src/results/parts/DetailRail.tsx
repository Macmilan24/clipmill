/**
 * Selected clip details with the nearest source filmstrip tile and ranking evidence.
 * Missing filmstrips are reported explicitly. Explanations are ordered by weighted
 * contribution and retain source sentences and timestamps.
 */
import { ArrowRight, CheckCheck, Scissors, TriangleAlert } from 'lucide-react';

import { MediaStill } from '../../components/MediaStill.js';
import { Button } from '../../components/ui/button.js';
import { type ClipRow, clock, duration, topFactors } from '../model.js';
import { TONE_INK, stateOf } from './state.js';

export interface DetailRailProps {
  readonly row: ClipRow | null;
  readonly tileUrl: (atTicks: number) => string | null;
  readonly approvedCount: number;
  readonly checkedCount: number;
  readonly busy: boolean;
  readonly onApproveChecked: () => void;
  readonly onOpen: (candidateId: string) => void;
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
  if (!row) return null;
  const state = stateOf(row);
  const factors = row.review ? [] : topFactors(row);

  return (
    <aside className="results-detail" aria-label="Selected clip">
      <div className="results-detail-body">
        <div className="results-detail-heading">
          <span>Selected clip</span>
          <span className="mono">{String(row.rank).padStart(2, '0')}</span>
        </div>
        <div className="results-detail-picture">
          <MediaStill src={tileUrl(row.startTicks)} />
          <span className="results-card-duration mono">{duration(row.durationSeconds)}</span>
        </div>
        <h2 className="results-detail-title">{row.headline || 'Untitled clip'}</h2>
        <div className="results-detail-meta">
          <span style={{ color: TONE_INK[state.tone] }}>{state.label}</span>
          <span className="mono">
            {clock(row.startTicks)} – {clock(row.endTicks)}
          </span>
        </div>
        {row.warnings.length > 0 && (
          <p className="results-caution">
            <TriangleAlert size={14} aria-hidden />
            <span>{row.warnings[0]}</span>
          </p>
        )}
        <details className="results-disclosure" key={row.candidateId}>
          <summary>{row.review ? 'Why this moment' : 'Ranking details'}</summary>
          <div className="results-disclosure-content">
            <p className="results-detail-rating">
              {row.bandLabel}
              {!row.review && ` · ${row.displayScore}`}
            </p>
            {row.review ? (
              <>
                {row.review.summary && <p>{row.review.summary}</p>}
                {row.review.reasons.length > 0 && (
                  <ul>
                    {row.review.reasons.map((reason, index) => (
                      <li key={index}>{reason}</li>
                    ))}
                  </ul>
                )}
              </>
            ) : (
              factors.map((factor) => (
                <div className="results-factor" key={factor.axis}>
                  <div>
                    <span>{factor.label}</span>
                    <span className="mono">{Math.round((factor.value ?? 0) * 100)}</span>
                  </div>
                  {factor.evidence[0] && <p>“{factor.evidence[0].text}”</p>}
                </div>
              ))
            )}
            {row.penalties.length > 0 && (
              <ul>
                {row.penalties.map((penalty) => (
                  <li key={penalty.reason}>{penalty.reason.replaceAll('_', ' ')}</li>
                ))}
              </ul>
            )}
          </div>
        </details>
      </div>
      <div className="results-detail-actions">
        {checkedCount > 0 ? (
          <>
            <p>
              {checkedCount} selected <span>{approvedCount} approved</span>
            </p>
            <Button className="w-full justify-center" disabled={busy} onClick={onApproveChecked}>
              <CheckCheck />
              {busy ? 'Approving…' : `Approve ${checkedCount} selected`}
            </Button>
          </>
        ) : (
          <>
            {row.docId !== null && (
              <Button
                className="w-full justify-center"
                disabled={busy}
                onClick={() => onEdit(row.candidateId)}
              >
                <Scissors />
                Open in the editor
              </Button>
            )}
            <Button
              variant={row.docId !== null ? 'outline' : 'default'}
              className="w-full justify-center"
              disabled={busy}
              onClick={() => onOpen(row.candidateId)}
            >
              Open in the inspector <ArrowRight />
            </Button>
          </>
        )}
      </div>
    </aside>
  );
}
