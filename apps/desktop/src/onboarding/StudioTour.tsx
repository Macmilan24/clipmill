import { useCallback, useEffect, useMemo } from 'react';
import { EVENTS, Joyride, STATUS, type EventData } from 'react-joyride';

import { createTourSteps } from './steps.js';
import { StudioTourTooltip } from './TourTooltip.js';
import type { TourOutcome } from './state.js';

interface StudioTourProps {
  readonly onNavigate: (sectionId: string) => void;
  readonly onEnd: (outcome: TourOutcome) => void;
}

/** One mounted session is one journey. Replaying mounts a fresh tour at step one. */
export function StudioTour({ onNavigate, onEnd }: StudioTourProps) {
  const steps = useMemo(() => createTourSteps(onNavigate), [onNavigate]);
  const handleEvent = useCallback(
    (event: EventData) => {
      if (event.type !== EVENTS.TOUR_END) return;
      onEnd(event.status === STATUS.FINISHED ? 'finished' : 'skipped');
    },
    [onEnd],
  );

  useEffect(() => {
    // Joyride's tooltip is portaled to body, so the shell can be inert while
    // the guide explains controls without an accidental import or export.
    const shell = document.querySelector<HTMLElement>('.studio-shell');
    if (!shell) return;
    shell.inert = true;
    shell.setAttribute('aria-hidden', 'true');
    return () => {
      shell.inert = false;
      shell.removeAttribute('aria-hidden');
    };
  }, []);

  useEffect(() => {
    const dismiss = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return;
      event.preventDefault();
      event.stopPropagation();
      onEnd('skipped');
    };
    document.addEventListener('keydown', dismiss, true);
    return () => document.removeEventListener('keydown', dismiss, true);
  }, [onEnd]);

  return (
    <>
      <span data-tour="tour-stage" className="tour-stage-anchor" aria-hidden="true" />
      <Joyride
        continuous
        run
        steps={steps}
        onEvent={handleEvent}
        tooltipComponent={StudioTourTooltip}
        locale={{
          back: 'Back',
          close: 'Close tour',
          last: 'Finish tour',
          next: 'Continue',
          skip: 'Skip tour',
        }}
        options={{
          backgroundColor: 'var(--cm-glass-elevated)',
          arrowColor: 'var(--cm-glass-elevated)',
          textColor: 'var(--cm-text-primary)',
          primaryColor: 'var(--cm-accent)',
          overlayColor: 'rgba(3, 6, 14, 0.76)',
          blockTargetInteraction: true,
          buttons: ['back', 'close', 'primary', 'skip'],
          closeButtonAction: 'skip',
          dismissKeyAction: false,
          overlayClickAction: false,
          scrollDuration: 220,
          skipScroll: true,
          spotlightPadding: 8,
          spotlightRadius: 12,
          skipBeacon: true,
          targetWaitTimeout: 4000,
          width: 420,
          zIndex: 2000,
        }}
      />
    </>
  );
}
