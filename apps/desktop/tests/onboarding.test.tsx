import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { CoachMarks } from '../src/onboarding/CoachMarks.js';
import {
  CoachEnabled,
  coachSeen,
  forgetOnboarding,
  OPEN_TOUR_EVENT,
  rememberWelcome,
  shouldWelcome,
  WELCOME_KEY,
} from '../src/onboarding/state.js';
import { Welcome } from '../src/onboarding/Welcome.js';
import { EditingPreferences } from '../src/screens/EditingPreferences.js';
import { MEMORY_KEY } from '../src/shell/memory.js';

beforeEach(() => localStorage.clear());

/** Controls laid out as a browser would, which jsdom does not do. */
function sized() {
  for (const element of document.querySelectorAll<HTMLElement>('[data-coach]')) {
    element.getBoundingClientRect = () =>
      ({ top: 10, left: 10, bottom: 40, right: 110, width: 100, height: 30 }) as DOMRect;
  }
}

describe('who is welcomed', () => {
  it('greets a new installation once, and nobody who has used the app before', () => {
    expect(shouldWelcome()).toBe(true);
    rememberWelcome('closed');
    expect(shouldWelcome()).toBe(false);
    localStorage.clear();
    localStorage.setItem(MEMORY_KEY, '{}');
    expect(shouldWelcome()).toBe(false);
    forgetOnboarding();
    localStorage.removeItem(MEMORY_KEY);
    expect(shouldWelcome()).toBe(true);
  });
});

describe('the welcome', () => {
  it('walks four steps and ends in New Project', () => {
    const onStart = vi.fn();
    const onClose = vi.fn();
    render(<Welcome open onClose={onClose} onStart={onStart} />);
    expect(screen.getByRole('heading', { name: /made here/ })).toBeTruthy();
    for (let step = 0; step < 3; step += 1) {
      fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    }
    expect(screen.getByRole('heading', { name: 'Edit and export' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Start a project' }));
    expect(onStart).toHaveBeenCalledOnce();
    expect(onClose).toHaveBeenCalledOnce();
    expect(localStorage.getItem(WELCOME_KEY)).toBe('started');
  });

  it('offers the studio tour at its end, from Settings too', () => {
    const toured = vi.fn();
    window.addEventListener(OPEN_TOUR_EVENT, toured);
    const onClose = vi.fn();
    const welcome = render(<Welcome open onClose={onClose} onStart={vi.fn()} />);
    for (let step = 0; step < 3; step += 1) {
      fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    }
    fireEvent.click(screen.getByRole('button', { name: 'Take the tour' }));
    expect(onClose).toHaveBeenCalledOnce();
    expect(localStorage.getItem(WELCOME_KEY)).toBe('toured');
    expect(toured).toHaveBeenCalledOnce();
    welcome.unmount();
    render(<EditingPreferences />);
    fireEvent.click(screen.getByRole('button', { name: 'Take the tour' }));
    expect(toured).toHaveBeenCalledTimes(2);
    window.removeEventListener(OPEN_TOUR_EVENT, toured);
  });
});

describe('tips on real controls', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());
  const marks = [
    { target: 'first', title: 'First tip', body: 'About the first control.' },
    { target: 'missing', title: 'Missing tip', body: 'Its control is not on screen.' },
    { target: 'last', title: 'Last tip', body: 'About the last control.' },
  ];
  const workspace = (enabled: boolean) => (
    <CoachEnabled.Provider value={enabled}>
      <button type="button" data-coach="first">
        One
      </button>
      <button type="button" data-coach="last">
        Two
      </button>
      <CoachMarks place="inspector" marks={marks} />
    </CoachEnabled.Provider>
  );

  it('points at each control in turn, passes over one not on screen, and is remembered', () => {
    render(workspace(true));
    sized();
    act(() => {
      vi.advanceTimersByTime(800);
    });
    expect(screen.getByRole('dialog', { name: 'First tip' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(screen.getByRole('dialog', { name: 'Last tip' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(coachSeen('inspector')).toBe(true);
  });

  it('shows nothing where the app has not turned tips on', () => {
    render(workspace(false));
    sized();
    act(() => {
      vi.advanceTimersByTime(800);
    });
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});
