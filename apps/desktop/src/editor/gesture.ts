/**
 * A press that becomes a drag only once it has moved, so a click on something
 * draggable selects it and never nudges it. Escape abandons a drag in flight.
 */
import type { PointerEvent as ReactPointerEvent } from 'react';

export interface PressHandlers {
  /** Called on every move once the press has become a drag. */
  readonly onDrag?: (dx: number, dy: number, event: PointerEvent) => void;
  /** The drag ended where the pointer was released. */
  readonly onDrop?: (dx: number, dy: number, event: PointerEvent) => void;
  /** The press ended without moving far enough to be a drag. */
  readonly onClick?: (event: PointerEvent) => void;
  /** Escape was pressed, or the pointer was taken away, mid-drag. */
  readonly onCancel?: () => void;
}

/** How far a press may wander, in CSS pixels, and still count as a click. */
export const DRAG_THRESHOLD = 4;

export function pressOrDrag(
  event: ReactPointerEvent,
  handlers: PressHandlers,
  threshold = DRAG_THRESHOLD,
): void {
  if (event.button > 0) return;
  event.preventDefault();
  event.stopPropagation();
  const startX = event.clientX;
  const startY = event.clientY;
  let dragging = false;

  const stop = () => {
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', up);
    window.removeEventListener('pointercancel', cancel);
    window.removeEventListener('keydown', key, true);
  };
  const move = (next: PointerEvent) => {
    const dx = next.clientX - startX;
    const dy = next.clientY - startY;
    if (!dragging && Math.hypot(dx, dy) < threshold) return;
    dragging = true;
    handlers.onDrag?.(dx, dy, next);
  };
  const up = (next: PointerEvent) => {
    stop();
    const dx = next.clientX - startX;
    const dy = next.clientY - startY;
    if (dragging) handlers.onDrop?.(dx, dy, next);
    else handlers.onClick?.(next);
  };
  const cancel = () => {
    stop();
    if (dragging) handlers.onCancel?.();
  };
  const key = (next: KeyboardEvent) => {
    if (next.key !== 'Escape' || !dragging) return;
    next.preventDefault();
    next.stopPropagation();
    cancel();
  };
  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', up);
  window.addEventListener('pointercancel', cancel);
  window.addEventListener('keydown', key, true);
}
