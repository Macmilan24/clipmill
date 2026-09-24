/**
 * Editor keyboard, on CapCut and Premiere's defaults: space and J/K/L play,
 * I/O mark a range, ⌘B splits, Delete removes the selection, arrows step.
 * Keys a focused control owns — typing, a button's Space or Enter, a slider's
 * arrows — are left to it, so clicking a button never switches the keys off.
 */
import { useEffect, useRef } from 'react';

export interface EditorKeyActions {
  readonly toggle: () => void;
  readonly pause: () => void;
  readonly shuttle: (direction: 1 | -1) => void;
  readonly step: (frames: number) => void;
  readonly goToStart: () => void;
  readonly goToEnd: () => void;
  readonly previousEdit: () => void;
  readonly nextEdit: () => void;
  readonly markIn: () => void;
  readonly markOut: () => void;
  readonly clearMarks: () => void;
  readonly split: () => void;
  readonly remove: () => void;
  readonly selectTool: () => void;
  readonly bladeTool: () => void;
  readonly toggleSnap: () => void;
  readonly addMarker: () => void;
  readonly find: () => void;
  readonly zoomIn: () => void;
  readonly zoomOut: () => void;
  readonly zoomFit: () => void;
  readonly undo: () => void;
  readonly redo: () => void;
  readonly exportClip: () => void;
  readonly escape: () => void;
}

const TYPING = 'input, textarea, select, [contenteditable="true"], [role="textbox"]';
const OWNS_ACTIVATION = 'button, a[href], summary, [role="button"], [role="tab"], [role="switch"]';
const OWNS_ARROWS =
  '[role="slider"], [role="tab"], [role="radio"], [role="menuitem"], [role="option"], [role="listbox"]';

/**
 * Whether Space or Enter belongs to the focused control: only when it was
 * reached from the keyboard. A button someone clicked keeps focus, and Space
 * must still play the clip then, as it does in every editor.
 */
function ownsActivation(target: Element | null, clicked: Element | null): boolean {
  const control = target?.closest(OWNS_ACTIVATION);
  return control !== null && control !== undefined && !(clicked && control.contains(clicked));
}

export function useEditorKeys(actions: EditorKeyActions, enabled = true): void {
  const latest = useRef(actions);
  latest.current = actions;

  useEffect(() => {
    if (!enabled) return;
    // What the pointer last pressed; moving focus with Tab forgets it.
    let clicked: Element | null = null;
    const onPointer = (event: PointerEvent) => {
      clicked = event.target instanceof Element ? event.target : null;
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Tab') clicked = null;
      if (event.defaultPrevented || event.isComposing) return;
      const act = latest.current;
      const target = event.target instanceof Element ? event.target : null;
      if (event.key === 'Escape') {
        if (!target?.closest(TYPING)) act.escape();
        return;
      }
      if (target?.closest(TYPING)) return;
      // An open menu or dialog is somewhere else's keyboard.
      if (
        document.querySelector(
          '[role="dialog"][data-state="open"], [role="menu"], [role="listbox"]',
        )
      )
        return;
      const command = event.metaKey || event.ctrlKey;
      const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
      const run = (action: () => void) => {
        event.preventDefault();
        action();
      };

      if (command) {
        if (key === 'z') return run(event.shiftKey ? act.redo : act.undo);
        if (key === 'y') return run(act.redo);
        if (key === 'b') return run(act.split);
        if (key === 'f') return run(act.find);
        if (key === 'e') return run(act.exportClip);
        if (key === '=' || key === '+') return run(act.zoomIn);
        if (key === '-') return run(act.zoomOut);
        return;
      }
      if (event.altKey) {
        if (event.code === 'KeyX') return run(act.clearMarks);
        return;
      }
      if ((key === ' ' || key === 'Enter') && ownsActivation(target, clicked)) return;
      if (key.startsWith('Arrow') && target?.closest(OWNS_ARROWS)) return;

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
          return run(() => act.step(event.shiftKey ? -10 : -1));
        case 'ArrowRight':
          return run(() => act.step(event.shiftKey ? 10 : 1));
        case 'ArrowUp':
          return run(act.previousEdit);
        case 'ArrowDown':
          return run(act.nextEdit);
        case 'Home':
          return run(act.goToStart);
        case 'End':
          return run(act.goToEnd);
        case 'i':
          return run(act.markIn);
        case 'o':
          return run(act.markOut);
        case 'Backspace':
        case 'Delete':
          return run(act.remove);
        case 'v':
          return run(act.selectTool);
        case 'b':
          return run(act.bladeTool);
        case 's':
          return run(act.toggleSnap);
        case 'm':
          return run(act.addMarker);
        case 'z':
          if (event.shiftKey) return run(act.zoomFit);
          return;
        default:
          return;
      }
    };
    window.addEventListener('pointerdown', onPointer, true);
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('pointerdown', onPointer, true);
      window.removeEventListener('keydown', onKey);
    };
  }, [enabled]);
}
