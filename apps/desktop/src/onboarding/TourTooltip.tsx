import { useId } from 'react';
import { ArrowLeft, ArrowRight, X } from 'lucide-react';
import type { TooltipRenderProps } from 'react-joyride';
import { BrandMark } from '../shell/BrandMark';
import './tour.css';

type TourStepData = {
  phase?: unknown;
  chapter?: unknown;
  visual?: unknown;
};

function label(value: unknown, fallback: string): string {
  return typeof value === 'string' && value.trim() ? value : fallback;
}

/** ClipMill's visual layer for Joyride. Navigation and focus remain owned by Joyride. */
export function StudioTourTooltip({
  backProps,
  closeProps,
  continuous,
  index,
  isLastStep,
  primaryProps,
  size,
  skipProps,
  step,
  tooltipProps,
}: TooltipRenderProps) {
  const id = useId();
  const titleId = `${id}-title`;
  const contentId = `${id}-content`;
  const data = step.data as TourStepData | undefined;
  const phase = label(data?.phase, 'ClipMill guide');
  const chapter = label(data?.chapter, 'Your workflow').replaceAll('-', ' ');
  const welcome = index === 0;
  const current = index + 1;
  const total = Math.max(size, 1);

  const closeButton = (
    <button {...closeProps} type="button" className="studio-tour-close">
      <X size={16} strokeWidth={1.8} aria-hidden="true" />
    </button>
  );

  return (
    <section
      {...tooltipProps}
      aria-labelledby={titleId}
      aria-describedby={contentId}
      className={`studio-tour-tooltip${welcome ? ' studio-tour-tooltip--welcome' : ''}`}
    >
      {welcome ? (
        <div className="studio-tour-welcome" aria-hidden="true">
          <div className="studio-tour-welcome-brand">
            <span className="studio-tour-welcome-logo">
              <BrandMark size={25} />
            </span>
            <span>
              CLIPMILL <span className="studio-tour-welcome-brand-muted">/ STUDIO</span>
            </span>
          </div>
          <div className="studio-tour-welcome-reel">
            <span className="studio-tour-welcome-frame studio-tour-welcome-frame--source" />
            <span className="studio-tour-welcome-frame studio-tour-welcome-frame--select" />
            <span className="studio-tour-welcome-frame studio-tour-welcome-frame--finish" />
            <span className="studio-tour-welcome-playhead" />
          </div>
          <div className="studio-tour-welcome-captions">
            <span>Source</span>
            <span>Find</span>
            <span>Finish</span>
          </div>
        </div>
      ) : (
        <div className="studio-tour-topline">
          <span className="studio-tour-mini-mark" aria-hidden="true">
            <BrandMark size={15} />
          </span>
          <span className="studio-tour-wordmark">ClipMill guide</span>
        </div>
      )}

      {closeButton}

      <div className="studio-tour-inner" key={index}>
        <div className="studio-tour-kicker">
          <div className="studio-tour-location">
            <span className="studio-tour-phase">{phase}</span>
            <span className="studio-tour-kicker-divider" aria-hidden="true" />
            <span className="studio-tour-chapter">{chapter}</span>
          </div>
          <span className="studio-tour-count" aria-label={`Step ${current} of ${total}`}>
            {String(current).padStart(2, '0')} <span aria-hidden="true">/</span>{' '}
            {String(total).padStart(2, '0')}
          </span>
        </div>

        <div className="studio-tour-progress" aria-hidden="true">
          {Array.from({ length: total }, (_, position) => (
            <span
              key={position}
              className={`studio-tour-progress-segment${position < current ? ' is-active' : ''}`}
            />
          ))}
        </div>

        <h2 className="studio-tour-title" id={titleId}>
          {step.title || chapter}
        </h2>
        <div className="studio-tour-content" id={contentId}>
          {step.content}
          {data?.visual === 'model-actions' && (
            <div className="studio-tour-model-demo" aria-hidden="true">
              <div className="studio-tour-model-demo-kicker">
                <span>Example model row</span>
                <span className="studio-tour-model-demo-status">Installed</span>
              </div>
              <div className="studio-tour-model-demo-actions">
                <span>Check files</span>
                <span className="studio-tour-model-demo-remove">Remove</span>
              </div>
              <div className="studio-tour-model-demo-confirm">
                <span>Confirm removal</span>
                <span>Keep it</span>
                <span className="studio-tour-model-demo-remove">Remove</span>
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="studio-tour-footer">
        <button {...skipProps} type="button" className="studio-tour-skip">
          {skipProps.title || 'Skip tour'}
        </button>
        <div className="studio-tour-actions">
          {index > 0 && (
            <button {...backProps} type="button" className="studio-tour-back">
              <ArrowLeft size={14} strokeWidth={1.8} aria-hidden="true" />
              <span>{backProps.title || 'Back'}</span>
            </button>
          )}
          {continuous && (
            <button {...primaryProps} type="button" className="studio-tour-next">
              <span>{primaryProps.title || (isLastStep ? 'Finish tour' : 'Next')}</span>
              <ArrowRight size={15} strokeWidth={1.8} aria-hidden="true" />
            </button>
          )}
        </div>
      </div>
    </section>
  );
}
