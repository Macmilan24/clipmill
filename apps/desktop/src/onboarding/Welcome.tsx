/**
 * The welcome a new installation opens to: four short steps from a long
 * recording to an exported clip, then straight to New Project, or the studio
 * tour, which shows where each of those things is. The tips in the Inspector
 * and the Editor point at the real controls when they first open.
 */
import { Clapperboard, Download, ListChecks, ShieldCheck } from 'lucide-react';
import { type JSX, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '../components/ui/dialog.js';
import { type WelcomeOutcome, openTour, rememberWelcome } from './state.js';
import './onboarding.css';

const COMMAND = typeof navigator !== 'undefined' && /Mac/.test(navigator.platform) ? '⌘' : 'Ctrl';

const STEPS = [
  {
    icon: ShieldCheck,
    title: 'Clips from your long recordings, made here',
    body: 'ClipMill finds the moments in a podcast or interview worth posting, frames them for vertical video and captions them. It runs on this computer: your recordings and words stay on it.',
  },
  {
    icon: Download,
    title: 'Bring a recording',
    body: 'Start a project from a video file — drop it on New Project — or from YouTube. Choose how long the clips should be and how their captions look; analysis tells you how long it will take here.',
  },
  {
    icon: ListChecks,
    title: 'Review what it found',
    body: 'Results lists the clips worth a look, each with why. Open one to judge it as it will be built: approve, keep for later or reject, move its start and end, and hear past the edges.',
  },
  {
    icon: Clapperboard,
    title: 'Edit and export',
    body: `Cut from the transcript, fix a word, restyle the captions and reframe in the Editor, then export a 9:16 file with its subtitles. ${COMMAND}/ lists every key.`,
  },
] as const;

export function Welcome({
  open,
  onClose,
  onStart,
}: {
  readonly open: boolean;
  /** Closed, with how. */
  readonly onClose: () => void;
  /** Go and start a project. */
  readonly onStart: () => void;
}): JSX.Element {
  const [at, setAt] = useState(0);
  const step = STEPS[at]!;
  const last = at === STEPS.length - 1;
  const end = (outcome: WelcomeOutcome) => {
    rememberWelcome(outcome);
    setAt(0);
    if (outcome === 'started') onStart();
    onClose();
    if (outcome === 'toured') openTour();
  };
  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next) end('closed');
      }}
    >
      <DialogContent className="welcome w-[min(520px,calc(100vw-32px))]">
        <div className="welcome-icon" aria-hidden="true">
          <step.icon className="size-5" />
        </div>
        <DialogTitle className="welcome-title">{step.title}</DialogTitle>
        <DialogDescription className="welcome-body">{step.body}</DialogDescription>
        <div className="welcome-foot">
          <ol className="welcome-dots" aria-label={`Step ${at + 1} of ${STEPS.length}`}>
            {STEPS.map((item, index) => (
              <li key={item.title} data-current={index === at ? 'true' : undefined} />
            ))}
          </ol>
          <div className="welcome-actions">
            {at > 0 && (
              <Button variant="ghost" size="sm" onClick={() => setAt(at - 1)}>
                Back
              </Button>
            )}
            {last ? (
              <>
                <Button variant="outline" size="sm" onClick={() => end('toured')}>
                  Take the tour
                </Button>
                <Button size="sm" onClick={() => end('started')}>
                  Start a project
                </Button>
              </>
            ) : (
              <Button size="sm" onClick={() => setAt(at + 1)}>
                Next
              </Button>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
