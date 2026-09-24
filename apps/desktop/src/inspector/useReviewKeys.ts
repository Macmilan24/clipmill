/**
 * Review station keyboard: space and J/K/L play, I/O mark, arrows step and move
 * between clips, one key per decision. Keys owned by a focused control — text
 * fields, a button's Space or Enter, a slider's arrows — are left to it.
 */
import { useEffect, useRef } from 'react';

export interface ReviewKeyActions {
  readonly toggle: () => void;
  readonly pause: () => void;
  readonly shuttle: (direction: 1 | -1) => void;
  readonly step: (frames: number) => void;
  readonly skip: (seconds: number) => void;
  readonly goToStart: () => void;
  readonly goToEnd: () => void;
  readonly playAroundStart: () => void;
  readonly playAroundEnd: () => void;
  readonly markStart: () => void;
  readonly markEnd: () => void;
  readonly previousClip: () => void;
  readonly nextClip: () => void;
  readonly approve: () => void;
  readonly approveAndEdit: () => void;
  readonly keep: () => void;
  readonly reject: () => void;
  readonly clear: () => void;
  readonly undo: () => void;
}

const TYPING = 'input, textarea, select, [contenteditable="true"], [role="textbox"]';
const OWNS_ACTIVATION = 'button, a[href], summary, [role="button"], [role="tab"], [role="switch"]';
const OWNS_ARROWS =
  '[role="slider"], [role="tab"], [role="radio"], [role="menuitem"], [role="option"]';

export function useReviewKeys(actions: ReviewKeyActions, enabled = true): void {
  const latest = useRef(actions);
  latest.current = actions;

  useEffect(() => {
    if (!enabled) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.altKey || event.isComposing) return;
      const target = event.target instanceof Element ? event.target : null;
      if (target?.closest(TYPING)) return;
      // An open menu or dialog is somewhere else's keyboard.
      if (document.querySelector('[role="dialog"][data-state="open"], [role="menu"]')) return;
      const act = latest.current;
      const command = event.metaKey || event.ctrlKey;
      const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;

      if (command) {
        if (key === 'z' && !event.shiftKey) {
          event.preventDefault();
          act.undo();
        }
        return;
      }
      if ((key === ' ' || key === 'Enter') && target?.closest(OWNS_ACTIVATION)) return;
      if (key.startsWith('Arrow') && target?.closest(OWNS_ARROWS)) return;

      const run = (action: () => void) => {
        event.preventDefault();
        action();
      };
      switch (key) {
        case ' ':
          return run(act.toggle);
        case 'k':
          return run(act.pause);
        case 'j':
          return run(() => act.shuttle(-1));
        case 'l':
          return run(() => act.shuttle(1));
        case 'ArrowLeft':
          return run(() => (event.shiftKey ? act.skip(-1) : act.step(-1)));
        case 'ArrowRight':
          return run(() => (event.shiftKey ? act.skip(1) : act.step(1)));
        case 'Home':
          return run(act.goToStart);
        case 'End':
          return run(act.goToEnd);
        case '[':
          return run(act.playAroundStart);
        case ']':
          return run(act.playAroundEnd);
        case 'i':
          return run(act.markStart);
        case 'o':
          return run(act.markEnd);
        case 'ArrowUp':
          return run(act.previousClip);
        case 'ArrowDown':
          return run(act.nextClip);
        case 'a':
          return run(act.approve);
        case 'Enter':
          return run(act.approveAndEdit);
        case 'h':
          return run(act.keep);
        case 'x':
          return run(act.reject);
        case 'u':
          return run(act.clear);
        default:
          return;
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [enabled]);
}
