/**
 * Driving the Editor's timeline in jsdom, which lays nothing out: give the
 * track a size, find where a moment is drawn, and seek the way a person does,
 * by pressing the ruler.
 */
import { fireEvent } from '@testing-library/react';
import { vi } from 'vitest';

import type { PreviewPlan } from '../../src/daemon/client.js';
import { extentOf, ticksOfFrame } from '../../src/editor/timeline.js';

export const TRACK_WIDTH = 1200;

export function box(width: number, height: number, left = 0, top = 0): DOMRect {
  return {
    left,
    top,
    width,
    height,
    right: left + width,
    bottom: top + height,
    x: left,
    y: top,
    toJSON: () => ({}),
  } as DOMRect;
}

/** Give an element a layout box for the rest of the test. */
export function measure(element: Element | null, rect: DOMRect) {
  if (!element) throw new Error('Nothing to measure');
  return vi.spyOn(element, 'getBoundingClientRect').mockReturnValue(rect);
}

export function measureTrack(width = TRACK_WIDTH) {
  return measure(document.querySelector('.edit-track'), box(width, 150));
}

/** Where a program tick is drawn while the timeline shows the whole clip. */
export function xOf(plan: PreviewPlan, ticks: number, width = TRACK_WIDTH): number {
  const extent = extentOf(plan);
  return ((ticks - extent.startTicks) / (extent.endTicks - extent.startTicks)) * width;
}

/** Press the ruler at a frame, as a person seeks. */
export function seekTo(plan: PreviewPlan, frame: number): void {
  const track = measureTrack();
  // Half a frame in, so rounding cannot land on the one before.
  const x = xOf(plan, ticksOfFrame(plan, frame) + (ticksOfFrame(plan, 1) >> 1));
  fireEvent.pointerDown(document.querySelector('.edit-ruler')!, { clientX: x, button: 0 });
  fireEvent.pointerUp(window, { clientX: x, button: 0 });
  track.mockRestore();
}

/** Press, move and release: a drag, in window events as the gesture hears them. */
export function drag(
  element: Element,
  from: { readonly x: number; readonly y?: number },
  to: { readonly x: number; readonly y?: number },
): void {
  fireEvent.pointerDown(element, { clientX: from.x, clientY: from.y ?? 0, button: 0 });
  fireEvent.pointerMove(window, { clientX: to.x, clientY: to.y ?? 0 });
  fireEvent.pointerUp(window, { clientX: to.x, clientY: to.y ?? 0 });
}
