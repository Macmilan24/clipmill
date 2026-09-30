/**
 * Small drawings of the screens a new installation has nothing on yet, so
 * the tour can show what Results, review, the Editor and Export will look
 * like instead of pointing at an empty page. Decoration only: each step's
 * words say what the picture shows.
 */
import type { JSX } from 'react';

import { BrandMark } from '../shell/BrandMark.js';
import type { TourPicture } from './tourSteps.js';

/** The first step's banner: a reel going from source to finished clip. */
export function WelcomeBanner(): JSX.Element {
  return (
    <div className="tour-banner" aria-hidden="true">
      <div className="tour-banner-brand">
        <span className="tour-banner-logo">
          <BrandMark size={24} />
        </span>
        <span>
          CLIPMILL <span className="tour-banner-muted">/ STUDIO TOUR</span>
        </span>
      </div>
      <div className="tour-banner-reel">
        <span className="tour-banner-frame" />
        <span className="tour-banner-frame tour-banner-frame--found" />
        <span className="tour-banner-frame" />
        <span className="tour-banner-playhead" />
      </div>
      <div className="tour-banner-captions">
        <span>Your recording</span>
        <span>The moment</span>
        <span>The clip</span>
      </div>
    </div>
  );
}

function ResultsPicture(): JSX.Element {
  return (
    <div className="tour-picture tour-results">
      <div className="tour-results-filters">
        <span className="is-on">All 12</span>
        <span>To decide 9</span>
        <span>Approved 3</span>
      </div>
      <div className="tour-results-cards">
        {[
          { score: 92, lines: [70, 45] },
          { score: 87, lines: [80, 55], hovered: true },
          { score: 81, lines: [60, 40] },
        ].map((card) => (
          <div key={card.score} className={`tour-results-card${card.hovered ? ' is-hovered' : ''}`}>
            <span className="tour-results-thumb">
              <span className="tour-person" />
              {card.hovered && <span className="tour-results-scrub" />}
              <span className="tour-results-score mono">{card.score}</span>
            </span>
            {card.lines.map((width) => (
              <span key={width} className="tour-line" style={{ width: `${width}%` }} />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

function ReviewPicture(): JSX.Element {
  return (
    <div className="tour-picture tour-review">
      <div className="tour-review-top">
        <span className="tour-frame">
          <span className="tour-person" />
          <span className="tour-caption">
            worth <em>keeping</em>
          </span>
        </span>
        <div className="tour-review-words">
          <span className="tour-line tour-line--dim" style={{ width: '64%' }} />
          <span className="tour-review-mark mono">IN</span>
          <span className="tour-line" style={{ width: '92%' }} />
          <span className="tour-line" style={{ width: '78%' }} />
          <span className="tour-line tour-line--live" style={{ width: '86%' }} />
          <span className="tour-review-mark mono">OUT</span>
          <span className="tour-line tour-line--dim" style={{ width: '52%' }} />
        </div>
      </div>
      <div className="tour-review-strip">
        <span className="tour-review-cut" />
      </div>
      <div className="tour-review-decide">
        <span>Reject</span>
        <span>Keep for later</span>
        <span className="is-primary">Approve</span>
      </div>
    </div>
  );
}

function EditorPicture(): JSX.Element {
  return (
    <div className="tour-picture tour-editor">
      <div className="tour-editor-top">
        <div className="tour-editor-words">
          <p>
            The best ideas start <s>um</s> with a <mark>question</mark>
          </p>
          <p className="tour-editor-dim">and most of us never ask it.</p>
        </div>
        <span className="tour-frame tour-frame--small">
          <span className="tour-person" />
          <span className="tour-caption">
            a <em>question</em>
          </span>
        </span>
      </div>
      <div className="tour-editor-timeline">
        <span className="tour-editor-track">
          <span style={{ left: '2%', width: '30%' }} />
          <span className="is-on" style={{ left: '34%', width: '26%' }} />
          <span style={{ left: '62%', width: '34%' }} />
        </span>
        <span className="tour-editor-track tour-editor-track--wave" />
        <span className="tour-editor-playhead" />
      </div>
    </div>
  );
}

function ExportPicture(): JSX.Element {
  return (
    <div className="tour-picture tour-export">
      <div className="tour-export-file">
        <span className="tour-frame tour-frame--tiny">
          <span className="tour-person" />
        </span>
        <span className="tour-export-name">
          <span className="mono">01-the-best-ideas.mp4</span>
          <span className="tour-export-facts mono">1080 × 1920 · 0:42</span>
        </span>
        <span className="tour-export-chips mono">
          <span>SRT</span>
          <span>VTT</span>
        </span>
      </div>
      <div className="tour-export-route">
        <span>Your folder</span>
        <span className="tour-export-arrow" />
        <span>YouTube · private</span>
        <span className="tour-export-arrow" />
        <span className="is-muted">Publish when ready</span>
      </div>
    </div>
  );
}

export function TourPictureOf({ kind }: { readonly kind: TourPicture }): JSX.Element | null {
  switch (kind) {
    case 'results':
      return <ResultsPicture />;
    case 'review':
      return <ReviewPicture />;
    case 'editor':
      return <EditorPicture />;
    case 'export':
      return <ExportPicture />;
    default:
      return null;
  }
}
