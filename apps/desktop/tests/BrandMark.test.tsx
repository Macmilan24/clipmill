import { fireEvent, render } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { BrandMark } from '../src/shell/BrandMark.js';

const originalAnimate = Object.getOwnPropertyDescriptor(Element.prototype, 'animate');
const motion = Object.assign(new EventTarget(), { matches: false });
const animation = { playbackRate: 0, play: vi.fn(), pause: vi.fn(), cancel: vi.fn() };
const animate = vi.fn(() => animation);

beforeEach(() => {
  vi.useFakeTimers();
  motion.matches = false;
  animation.playbackRate = 0;
  vi.clearAllMocks();
  vi.stubGlobal('matchMedia', () => motion);
  vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) =>
    window.setTimeout(() => callback(performance.now()), 16),
  );
  vi.stubGlobal('cancelAnimationFrame', (id: number) => window.clearTimeout(id));
  Object.defineProperty(Element.prototype, 'animate', { configurable: true, value: animate });
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  if (originalAnimate) Object.defineProperty(Element.prototype, 'animate', originalAnimate);
  else Reflect.deleteProperty(Element.prototype, 'animate');
});

function show(processing = false) {
  const view = render(
    <span data-testid="brand">
      <BrandMark animated processing={processing} />
    </span>,
  );
  return { ...view, target: view.getByTestId('brand') };
}

it('gives hover a brief burst, then returns to idle without moving the hub', () => {
  const { target, unmount, rerender } = show();
  expect(animate).toHaveBeenCalledWith(expect.any(Array), {
    duration: 30_000,
    iterations: Infinity,
  });
  expect(animate.mock.contexts[0]).toBe(target.querySelector('.brand-mark-blades'));
  expect(target.querySelector('circle')).not.toBe(animate.mock.contexts[0]);
  expect(animation.playbackRate).toBe(1);
  expect(vi.getTimerCount()).toBe(0);
  fireEvent.pointerEnter(target);
  vi.advanceTimersByTime(160);
  expect(animation.playbackRate).toBeGreaterThan(1);
  expect(animation.playbackRate).toBeLessThan(4);
  vi.advanceTimersByTime(1200);
  expect(animation.playbackRate).toBe(4);
  vi.advanceTimersByTime(1700);
  expect(animation.playbackRate).toBe(1);

  fireEvent.pointerLeave(target);
  fireEvent.pointerEnter(target);
  vi.advanceTimersByTime(160);
  expect(animation.playbackRate).toBeGreaterThan(1);
  expect(animation.playbackRate).toBeLessThan(4);
  fireEvent.pointerLeave(target);
  vi.advanceTimersByTime(1200);
  expect(animation.playbackRate).toBe(1);

  rerender(
    <span data-testid="brand">
      <BrandMark animated processing />
    </span>,
  );
  vi.advanceTimersByTime(1800);
  expect(animation.playbackRate).toBe(40);
  fireEvent.pointerEnter(target);
  vi.advanceTimersByTime(1800);
  expect(animation.playbackRate).toBe(40);
  rerender(
    <span data-testid="brand">
      <BrandMark animated />
    </span>,
  );
  vi.advanceTimersByTime(1800);
  expect(animation.playbackRate).toBe(1);
  expect(animation.pause).not.toHaveBeenCalled();
  expect(animation.cancel).not.toHaveBeenCalled();
  expect(vi.getTimerCount()).toBe(0);

  fireEvent.pointerLeave(target);
  fireEvent.pointerEnter(target);
  expect(animate).toHaveBeenCalledTimes(1);
  unmount();
  expect(animation.cancel).toHaveBeenCalledOnce();
  expect(vi.getTimerCount()).toBe(0);
});

it('honors reduced motion at startup and when it changes during a turn', () => {
  motion.matches = true;
  const { target, unmount } = show();
  fireEvent.pointerEnter(target);
  expect(animate).not.toHaveBeenCalled();
  motion.matches = false;
  motion.dispatchEvent(new Event('change'));
  vi.advanceTimersByTime(160);
  expect(animation.playbackRate).toBeGreaterThanOrEqual(1);
  motion.matches = true;
  motion.dispatchEvent(new Event('change'));
  expect(animation.pause).toHaveBeenCalled();
  expect(vi.getTimerCount()).toBe(0);
  unmount();
});
