/**
 * The studio tour: a guided walk through the app, one step at a time, with
 * the control a step is about lit and the rest of the window dimmed. It moves
 * between screens but starts nothing, and the app behind it takes no clicks
 * or keys while it runs. Arrow keys move through it, Escape ends it.
 */
import { ArrowLeft, ArrowRight, X } from 'lucide-react';
import { type JSX, useCallback, useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

import { BrandMark } from '../shell/BrandMark.js';
import { TourPictureOf, WelcomeBanner } from './TourPictures.js';
import { TOUR_STEPS, type TourPlacement, type TourStep } from './tourSteps.js';
import './tour.css';

/** How the tour ended: going to start a project, or anything else. */
export type TourOutcome = 'start' | 'done';

export interface CardPlace {
  readonly top: number;
  readonly left: number;
  readonly side: TourPlacement;
}

const GAP = 16;
const EDGE = 12;
/** How long a screen may take to put a step's control on screen. */
const SETTLE_MS = 2500;
/** Room left around a lit control. */
const HALO = 6;

const clamp = (value: number, low: number, high: number) =>
  Math.min(Math.max(value, low), Math.max(low, high));

/**
 * Where the card goes: beside the lit control on the asked side, kept inside
 * the window; else the opposite side, then either side across, wherever it
 * first covers none of the control; else wherever it covers least. A step
 * with nothing lit sits in the middle.
 */
export function placeCard(
  ring: DOMRect | null,
  card: { readonly width: number; readonly height: number },
  asked: TourPlacement,
  view: { readonly width: number; readonly height: number },
): CardPlace {
  if (ring === null || asked === 'center') {
    return {
      top: Math.max(EDGE, (view.height - card.height) / 2),
      left: Math.max(EDGE, (view.width - card.width) / 2),
      side: 'center',
    };
  }
  const x = (left: number) => clamp(left, EDGE, view.width - card.width - EDGE);
  const y = (top: number) => clamp(top, EDGE, view.height - card.height - EDGE);
  const alongY = y(ring.top + ring.height / 2 - card.height / 2);
  const alongX = x(ring.left + ring.width / 2 - card.width / 2);
  const at = {
    right: { top: alongY, left: x(ring.right + GAP) },
    left: { top: alongY, left: x(ring.left - GAP - card.width) },
    bottom: { top: y(ring.bottom + GAP), left: alongX },
    top: { top: y(ring.top - GAP - card.height), left: alongX },
  } as const;
  const covered = ({ top, left }: { readonly top: number; readonly left: number }) =>
    Math.max(0, Math.min(left + card.width, ring.right) - Math.max(left, ring.left)) *
    Math.max(0, Math.min(top + card.height, ring.bottom) - Math.max(top, ring.top));
  // The opposite side first, then across.
  const order = {
    right: ['right', 'left', 'bottom', 'top'],
    left: ['left', 'right', 'bottom', 'top'],
    bottom: ['bottom', 'top', 'right', 'left'],
    top: ['top', 'bottom', 'right', 'left'],
  }[asked] as readonly (keyof typeof at)[];
  const clear = order.find((side) => covered(at[side]) === 0);
  const side =
    clear ?? order.reduce((best, next) => (covered(at[next]) < covered(at[best]) ? next : best));
  return { ...at[side], side };
}

/** The lit window around a control, kept inside the app's window. */
function spotlight(ring: DOMRect): {
  readonly top: number;
  readonly left: number;
  readonly width: number;
  readonly height: number;
} {
  const top = Math.max(2, ring.top - HALO);
  const left = Math.max(2, ring.left - HALO);
  const bottom = Math.min(window.innerHeight - 2, ring.bottom + HALO);
  const right = Math.min(window.innerWidth - 2, ring.right + HALO);
  return { top, left, width: Math.max(0, right - left), height: Math.max(0, bottom - top) };
}

function onScreen(element: Element | null): element is HTMLElement {
  if (!(element instanceof HTMLElement)) return false;
  const rect = element.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
}

/** The first of a step's controls that is on screen, if any is. */
function findTarget(step: TourStep): HTMLElement | null {
  for (const name of step.targets ?? []) {
    const element = document.querySelector(`[data-tour="${name}"]`);
    if (onScreen(element)) return element;
  }
  return null;
}

export function StudioTour({
  onNavigate,
  onEnd,
  steps = TOUR_STEPS,
}: {
  /** Show a section of the app, by its rail id. */
  readonly onNavigate: (sectionId: string) => void;
  readonly onEnd: (outcome: TourOutcome) => void;
  readonly steps?: readonly TourStep[];
}): JSX.Element {
  const [at, setAt] = useState(0);
  // The control a step lights, once its screen has put it there; null for a
  // step with none, or whose control never came. Absent while still looking.
  const [found, setFound] = useState<{
    readonly step: string;
    readonly element: HTMLElement | null;
  } | null>(null);
  const [ring, setRing] = useState<DOMRect | null>(null);
  const [place, setPlace] = useState<CardPlace | null>(null);
  const card = useRef<HTMLElement>(null);
  const primary = useRef<HTMLButtonElement>(null);
  const navigate = useRef(onNavigate);
  const end = useRef(onEnd);
  useEffect(() => {
    navigate.current = onNavigate;
    end.current = onEnd;
  });
  const id = useId();
  const step = steps[at]!;
  const last = at === steps.length - 1;
  const settled = found?.step === step.id ? found : null;

  // Show the step's screen, then wait for its control to be on screen.
  useEffect(() => {
    if (step.section) navigate.current(step.section);
    if (!step.targets?.length) {
      setFound({ step: step.id, element: null });
      return undefined;
    }
    let frame = 0;
    const until = performance.now() + SETTLE_MS;
    const seek = () => {
      const element = findTarget(step);
      if (element) {
        const box = element.getBoundingClientRect();
        if (box.top < 0 || box.bottom > window.innerHeight) {
          element.scrollIntoView({ block: 'center', inline: 'nearest' });
        }
        setFound({ step: step.id, element });
      } else if (performance.now() < until) {
        frame = requestAnimationFrame(seek);
      } else {
        setFound({ step: step.id, element: null });
      }
    };
    frame = requestAnimationFrame(seek);
    return () => cancelAnimationFrame(frame);
  }, [step]);

  // Light the control, and place the card beside it once its size is known.
  const measure = useCallback(() => {
    if (settled === null) return;
    const element = settled.element;
    const box = element && onScreen(element) ? element.getBoundingClientRect() : null;
    setRing(box);
    const size = card.current?.getBoundingClientRect() ?? { width: 420, height: 320 };
    setPlace(
      placeCard(box, size, step.placement ?? 'center', {
        width: window.innerWidth,
        height: window.innerHeight,
      }),
    );
  }, [settled, step]);

  useLayoutEffect(() => {
    measure();
  }, [measure]);

  useEffect(() => {
    window.addEventListener('resize', measure);
    window.addEventListener('scroll', measure, true);
    return () => {
      window.removeEventListener('resize', measure);
      window.removeEventListener('scroll', measure, true);
    };
  }, [measure]);

  // The app behind the tour takes nothing while it runs.
  useEffect(() => {
    const shell = document.querySelector<HTMLElement>('.studio-shell');
    if (!shell) return undefined;
    shell.inert = true;
    return () => {
      shell.inert = false;
    };
  }, []);

  const forward = useCallback(() => {
    if (last) end.current('done');
    else setAt((current) => current + 1);
  }, [last]);
  const back = useCallback(() => setAt((current) => Math.max(0, current - 1)), []);

  useEffect(() => {
    const keys = (event: KeyboardEvent) => {
      if (event.key === 'Escape') end.current('done');
      else if (event.key === 'ArrowRight') forward();
      else if (event.key === 'ArrowLeft') back();
      else return;
      event.preventDefault();
      event.stopPropagation();
    };
    document.addEventListener('keydown', keys, true);
    return () => document.removeEventListener('keydown', keys, true);
  }, [back, forward]);

  // The primary button takes the keyboard once the step is showing.
  const shown = settled !== null && place !== null;
  useEffect(() => {
    if (shown) primary.current?.focus({ preventScroll: true });
  }, [shown, at]);

  const welcome = step.picture === 'welcome';
  return createPortal(
    <div className="studio-tour" data-tour-step={step.id}>
      {settled && ring ? (
        <div className="studio-tour-spotlight" aria-hidden="true" style={spotlight(ring)} />
      ) : (
        <div className="studio-tour-dim" aria-hidden="true" />
      )}
      <section
        ref={card}
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-body`}
        className={`studio-tour-card${welcome ? ' studio-tour-card--welcome' : ''}`}
        data-side={place?.side}
        style={shown && place ? { top: place.top, left: place.left } : { visibility: 'hidden' }}
      >
        {welcome ? (
          <WelcomeBanner />
        ) : (
          <div className="studio-tour-topline">
            <span className="studio-tour-mark" aria-hidden="true">
              <BrandMark size={14} />
            </span>
            <span>ClipMill tour</span>
          </div>
        )}
        <button
          type="button"
          className="studio-tour-close"
          aria-label="End the tour"
          onClick={() => end.current('done')}
        >
          <X aria-hidden="true" />
        </button>
        <div className="studio-tour-inner" key={step.id}>
          <div className="studio-tour-kicker">
            <span className="studio-tour-where">
              <span className="studio-tour-chapter">{step.chapter}</span>
              <span className="studio-tour-divider" aria-hidden="true" />
              <span className="studio-tour-place">{step.place}</span>
            </span>
            <span className="studio-tour-count" aria-label={`Step ${at + 1} of ${steps.length}`}>
              {String(at + 1).padStart(2, '0')} <span aria-hidden="true">/</span>{' '}
              {String(steps.length).padStart(2, '0')}
            </span>
          </div>
          <div className="studio-tour-progress" aria-hidden="true">
            {steps.map((item, index) => (
              <span key={item.id} className={index <= at ? 'is-done' : undefined} />
            ))}
          </div>
          <h2 id={`${id}-title`} className="studio-tour-title">
            {step.title}
          </h2>
          <div id={`${id}-body`} className="studio-tour-body">
            {step.body}
          </div>
          {step.picture && !welcome && (
            <div aria-hidden="true">
              <TourPictureOf kind={step.picture} />
            </div>
          )}
        </div>
        <footer className="studio-tour-footer">
          {last ? (
            <span />
          ) : (
            <button type="button" className="studio-tour-skip" onClick={() => end.current('done')}>
              Skip tour
            </button>
          )}
          <div className="studio-tour-actions">
            {at > 0 && (
              <button type="button" className="studio-tour-back" onClick={back}>
                <ArrowLeft aria-hidden="true" />
                Back
              </button>
            )}
            {last ? (
              <>
                <button
                  type="button"
                  className="studio-tour-back"
                  onClick={() => end.current('done')}
                >
                  Finish
                </button>
                <button
                  ref={primary}
                  type="button"
                  className="studio-tour-next"
                  onClick={() => end.current('start')}
                >
                  Start a project
                  <ArrowRight aria-hidden="true" />
                </button>
              </>
            ) : (
              <button ref={primary} type="button" className="studio-tour-next" onClick={forward}>
                {at === 0 ? 'Show me' : 'Next'}
                <ArrowRight aria-hidden="true" />
              </button>
            )}
          </div>
        </footer>
      </section>
    </div>,
    document.body,
  );
}
