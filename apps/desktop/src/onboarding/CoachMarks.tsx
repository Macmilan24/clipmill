/**
 * A few tips the first time a workspace opens, each pointing at the real
 * control it is about. They do not block anything: the workspace stays live
 * behind them, a tip whose control is not on screen is passed over, and
 * Escape or "Skip tips" ends them for good.
 */
import { type JSX, useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { type CoachPlace, coachSeen, rememberCoach, useCoachEnabled } from './state.js';
import './onboarding.css';

export interface CoachMark {
  /** The `data-coach` value of the control it points at. */
  readonly target: string;
  readonly title: string;
  readonly body: string;
}

interface Placed {
  readonly index: number;
  readonly ring: DOMRect;
  readonly top: number;
  readonly left: number;
  readonly above: boolean;
}

const CARD_WIDTH = 300;
const GAP = 12;

export function CoachMarks({
  place,
  marks,
}: {
  readonly place: CoachPlace;
  readonly marks: readonly CoachMark[];
}): JSX.Element | null {
  const enabled = useCoachEnabled();
  const [index, setIndex] = useState<number | null>(null);
  const [placed, setPlaced] = useState<Placed | null>(null);
  const card = useRef<HTMLDivElement>(null);

  // A moment after the workspace opens, so its controls are there to point at.
  useEffect(() => {
    if (!enabled || coachSeen(place)) return undefined;
    const timer = setTimeout(() => setIndex(0), 700);
    return () => clearTimeout(timer);
  }, [enabled, place]);

  const finish = useCallback(() => {
    rememberCoach(place);
    setIndex(null);
    setPlaced(null);
  }, [place]);

  // Find the tip's control, passing over any not on screen, and place the
  // card below it — or above, where below would leave the window.
  const place_ = useCallback(() => {
    if (index === null) return;
    for (let at = index; at < marks.length; at += 1) {
      const element = document.querySelector<HTMLElement>(`[data-coach="${marks[at]!.target}"]`);
      const ring = element?.getBoundingClientRect();
      if (!ring || ring.width === 0 || ring.height === 0) continue;
      const height = card.current?.offsetHeight ?? 150;
      const above = ring.bottom + GAP + height > window.innerHeight;
      const top = above ? Math.max(GAP, ring.top - GAP - height) : ring.bottom + GAP;
      const left = Math.min(
        Math.max(GAP, ring.left + ring.width / 2 - CARD_WIDTH / 2),
        window.innerWidth - CARD_WIDTH - GAP,
      );
      if (at !== index) setIndex(at);
      setPlaced({ index: at, ring, top, left, above });
      return;
    }
    finish();
  }, [index, marks, finish]);

  useLayoutEffect(() => {
    place_();
  }, [place_]);

  useEffect(() => {
    if (index === null) return undefined;
    const again = () => place_();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') finish();
    };
    window.addEventListener('resize', again);
    window.addEventListener('scroll', again, true);
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('resize', again);
      window.removeEventListener('scroll', again, true);
      window.removeEventListener('keydown', onKey);
    };
  }, [index, place_, finish]);

  if (index === null || !placed) return null;
  const mark = marks[placed.index]!;
  const last = placed.index === marks.length - 1;
  return (
    <>
      <div
        className="coach-ring"
        aria-hidden="true"
        style={{
          top: placed.ring.top - 4,
          left: placed.ring.left - 4,
          width: placed.ring.width + 8,
          height: placed.ring.height + 8,
        }}
      />
      <div
        ref={card}
        className="coach-card"
        role="dialog"
        aria-modal="false"
        aria-labelledby="coach-title"
        data-above={placed.above ? 'true' : undefined}
        style={{ top: placed.top, left: placed.left, width: CARD_WIDTH }}
      >
        <p className="coach-count mono">
          {placed.index + 1} of {marks.length}
        </p>
        <h2 id="coach-title" className="coach-title">
          {mark.title}
        </h2>
        <p className="coach-body">{mark.body}</p>
        <div className="coach-actions">
          <Button size="sm" variant="ghost" onClick={finish}>
            Skip tips
          </Button>
          <Button size="sm" onClick={() => (last ? finish() : setIndex(placed.index + 1))}>
            {last ? 'Done' : 'Next'}
          </Button>
        </div>
      </div>
    </>
  );
}
