/**
 * The studio tour: a card beside the lit control without covering it, a
 * walk that shows each step's screen and starts nothing, and an app behind
 * it that takes no input until it ends.
 */
import { act, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { StudioTour, placeCard } from '../src/onboarding/StudioTour.js';
import { TOUR_STEPS, type TourStep } from '../src/onboarding/tourSteps.js';

const window1280 = { width: 1280, height: 800 };
const card = { width: 420, height: 300 };
const ring = (left: number, top: number, width: number, height: number) =>
  new DOMRect(left, top, width, height);

describe('where the tour card goes', () => {
  it('sits on the asked side, and in the middle when nothing is lit', () => {
    expect(placeCard(ring(100, 200, 200, 100), card, 'right', window1280)).toEqual({
      top: 100,
      left: 316,
      side: 'right',
    });
    expect(placeCard(null, card, 'right', window1280).side).toBe('center');
  });

  it('moves to a side that leaves the control uncovered', () => {
    // No room on the right of a control at the window's right edge.
    expect(placeCard(ring(1000, 300, 250, 80), card, 'right', window1280).side).toBe('left');
  });

  it('covers as little as it can when no side is clear', () => {
    const wide = ring(20, 20, 1240, 700);
    const place = placeCard(wide, card, 'right', window1280);
    expect(place.top).toBeGreaterThanOrEqual(12);
    expect(place.left + card.width).toBeLessThanOrEqual(window1280.width - 12);
  });
});

describe('the studio tour', () => {
  const steps: readonly TourStep[] = [
    { id: 'one', chapter: '01 · Start', place: 'Welcome', title: 'First', body: 'Hello' },
    {
      id: 'two',
      chapter: '02 · Prepare',
      place: 'Models',
      title: 'Second',
      body: 'Models',
      section: 'models',
      targets: ['nowhere'],
    },
    { id: 'three', chapter: '03 · Begin', place: 'End', title: 'Third', body: 'Bye' },
  ];

  function tour(onNavigate = vi.fn(), onEnd = vi.fn()) {
    const shell = document.createElement('div');
    shell.className = 'studio-shell';
    document.body.append(shell);
    const rendered = render(<StudioTour steps={steps} onNavigate={onNavigate} onEnd={onEnd} />);
    return { shell, view: rendered, onNavigate, onEnd };
  }

  it('walks forward and back, showing each step’s screen, with the app inert', async () => {
    vi.useFakeTimers();
    const { shell, view, onNavigate } = tour();
    expect(shell.inert).toBe(true);
    expect(screen.getByRole('dialog', { name: 'First' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /Show me/ }));
    expect(onNavigate).toHaveBeenCalledWith('models');
    // Its control never comes, so the card waits, then sits in the middle.
    await act(async () => {
      vi.advanceTimersByTime(3000);
    });
    expect(screen.getByRole('dialog', { name: 'Second' })).toBeTruthy();
    fireEvent.keyDown(document, { key: 'ArrowLeft' });
    expect(screen.getByRole('dialog', { name: 'First' })).toBeTruthy();
    view.unmount();
    expect(shell.inert).toBe(false);
    shell.remove();
    vi.useRealTimers();
  });

  it('ends on Escape, and says so when the person goes to start a project', () => {
    const escaped = tour();
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(escaped.onEnd).toHaveBeenCalledWith('done');
    escaped.view.unmount();
    escaped.shell.remove();

    const started = tour();
    fireEvent.keyDown(document, { key: 'ArrowRight' });
    fireEvent.keyDown(document, { key: 'ArrowRight' });
    fireEvent.click(screen.getByRole('button', { name: /Start a project/ }));
    expect(started.onEnd).toHaveBeenCalledWith('start');
    started.view.unmount();
    started.shell.remove();
  });

  it('keeps every step short, and each picture step pointing at the rail', () => {
    for (const step of TOUR_STEPS) {
      expect(step.title.length).toBeLessThanOrEqual(52);
      if (step.picture && step.picture !== 'welcome') {
        expect(step.targets?.[0]).toMatch(/^nav-/);
      }
    }
  });
});
