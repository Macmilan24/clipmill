/**
 * The Reframe tab, and the control that used to lie.
 *
 * Re-solving needs a face track. Nothing schedules a face pass during an
 * analysis yet, so today there is never one — and the button was enabled
 * anyway, with a handler that returned early and said nothing. A click did
 * nothing, twice, and the editor was left deciding whether they had misread
 * the control or the product was broken.
 *
 * These hold the refusal to both halves of R25: the control is shut, and the
 * screen says why. A disabled button with no sentence is the same failure
 * wearing a different face.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { PreviewPlan } from '../src/daemon/client.js';
import { Reframe } from '../src/editor/Reframe.js';
import { mapping } from './support/plan.js';

const NO_FACES = 'Nothing has detected faces in this recording, so there is no track to follow.';

function plan(crops: PreviewPlan['crops']): PreviewPlan {
  const program = { frameCount: 30, rateNum: 30_000, rateDen: 1_001 };
  return {
    ...mapping(program),
    revision: 1,
    rateNum: 30_000,
    rateDen: 1_001,
    frameCount: 30,
    crops,
    cues: [],
    gain: [],
    width: 1080,
    height: 1920,
  };
}

/** A solved path, so the tab has a crop to talk about. */
function solved(): PreviewPlan {
  return plan(Array.from({ length: 30 }, () => [200, 0, 608, 1080] as const));
}

/** A fitted clip: the state every clip is in today. */
function fitted(): PreviewPlan {
  return plan(Array.from({ length: 30 }, () => null));
}

function show(overrides: Partial<Parameters<typeof Reframe>[0]> = {}) {
  const props: Parameters<typeof Reframe>[0] = {
    plan: solved(),
    frame: 0,
    busy: false,
    onApply: () => {},
    onResolve: () => {},
    resolving: false,
    resolveRefusal: null,
    ...overrides,
  };
  const view = render(<Reframe {...props} />);
  return { replan: (next: PreviewPlan) => view.rerender(<Reframe {...props} plan={next} />) };
}

function resolveButton() {
  return screen.getByRole('button', { name: /re-solve the path/i });
}

describe('soft cuts', () => {
  it('keeps older documents off and enables the whole clip with a saved 120 ms command', () => {
    const onApply = vi.fn();
    const view = show({ onApply });
    const toggle = screen.getByRole('switch', { name: 'Soft cuts' });
    expect(toggle.getAttribute('aria-checked')).toBe('false');
    expect(screen.queryByRole('spinbutton', { name: 'Blend duration' })).toBeNull();
    expect(screen.getByText('Whole clip')).toBeTruthy();
    fireEvent.click(toggle);
    expect(onApply).toHaveBeenCalledExactlyOnceWith({
      op: 'set_transition',
      duration_ticks: 10_800,
    });
    // Only the saved plan changes the displayed state; a pending/failed save is not On.
    expect(toggle.getAttribute('aria-checked')).toBe('false');
    view.replan({ ...solved(), transitionTicks: 10_800, revision: 2 });
    expect(toggle.getAttribute('aria-checked')).toBe('true');
    expect(screen.getByRole('spinbutton', { name: 'Blend duration' })).toHaveProperty(
      'value',
      '120',
    );
  });

  it('submits one duration edit after typing and reflects the saved value on undo', () => {
    const onApply = vi.fn();
    const initial = { ...solved(), transitionTicks: 10_800 };
    const view = show({ plan: initial, onApply });
    const duration = screen.getByRole('spinbutton', { name: 'Blend duration' });
    fireEvent.change(duration, { target: { value: '1' } });
    fireEvent.change(duration, { target: { value: '18' } });
    fireEvent.change(duration, { target: { value: '180' } });
    expect(onApply).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Apply' }));
    expect(onApply).toHaveBeenCalledExactlyOnceWith({
      op: 'set_transition',
      duration_ticks: 16_200,
    });
    view.replan({ ...initial, transitionTicks: 16_200, revision: 2 });
    expect(screen.getByRole('spinbutton', { name: 'Blend duration' })).toHaveProperty(
      'value',
      '180',
    );
    expect(screen.getByRole('button', { name: 'Apply' })).toHaveProperty('disabled', true);
    view.replan({ ...initial, revision: 3 });
    expect(screen.getByRole('spinbutton', { name: 'Blend duration' })).toHaveProperty(
      'value',
      '120',
    );
    fireEvent.click(screen.getByRole('switch', { name: 'Soft cuts' }));
    expect(onApply).toHaveBeenLastCalledWith({ op: 'set_transition', duration_ticks: 0 });
  });

  it('supports form submission and Escape without committing every keyboard change', () => {
    const onApply = vi.fn();
    show({ plan: { ...solved(), transitionTicks: 18_000 }, onApply });
    const duration = screen.getByRole('spinbutton', { name: 'Blend duration' });
    expect(duration).toHaveProperty('value', '200');
    fireEvent.change(duration, { target: { value: '160' } });
    fireEvent.keyDown(duration, { key: 'Escape' });
    expect(duration).toHaveProperty('value', '200');
    expect(onApply).not.toHaveBeenCalled();
    fireEvent.change(duration, { target: { value: '40' } });
    fireEvent.submit(duration.closest('form')!);
    expect(onApply).toHaveBeenCalledExactlyOnceWith({
      op: 'set_transition',
      duration_ticks: 3_600,
    });
  });

  it.each(['', '39', '251', '120.5'])('refuses an invalid duration %s without an edit', (value) => {
    const onApply = vi.fn();
    show({ plan: { ...solved(), transitionTicks: 10_800 }, onApply });
    const duration = screen.getByRole('spinbutton', { name: 'Blend duration' });
    fireEvent.change(duration, { target: { value } });
    expect(duration.getAttribute('aria-invalid')).toBe('true');
    expect(screen.getByRole('button', { name: 'Apply' })).toHaveProperty('disabled', true);
    fireEvent.submit(duration.closest('form')!);
    expect(onApply).not.toHaveBeenCalled();
  });

  it('disables controls during a save and explains its scope and short-shot behavior', () => {
    const onApply = vi.fn();
    show({ plan: { ...solved(), transitionTicks: 10_800 }, busy: true, onApply });
    const toggle = screen.getByRole('switch', { name: 'Soft cuts' });
    expect(toggle).toHaveProperty('disabled', true);
    expect(screen.getByRole('spinbutton', { name: 'Blend duration' })).toHaveProperty(
      'disabled',
      true,
    );
    expect(screen.getByRole('button', { name: 'Apply' })).toHaveProperty('disabled', true);
    fireEvent.click(toggle);
    expect(onApply).not.toHaveBeenCalled();
    expect(screen.getByText(/short shots use shorter blends/i)).toBeTruthy();
    expect(screen.getByText(/this clip has no cuts to blend/i)).toBeTruthy();
  });

  it('explains when the saved plan cannot blend any cuts without guessing for older hosts', () => {
    const base = solved();
    const first = base.segments[0]!;
    const initial = {
      ...base,
      segments: [first, { ...first, segmentId: 'seg_2' }],
      transitionTicks: 3_600,
    };
    const view = show({ plan: initial });
    expect(screen.queryByText(/no cuts can be softened/i)).toBeNull();
    view.replan({ ...initial, transitions: [] });
    expect(screen.getByRole('note').textContent).toBe(
      'No cuts can be softened at this duration and framing.',
    );
    view.replan({ ...initial, transitionTicks: 0, transitions: [] });
    expect(screen.queryByText(/no cuts can be softened/i)).toBeNull();
    view.replan({
      ...initial,
      transitions: [
        { incomingSegmentId: 'seg_2', outgoingFrame: 14, firstFrame: 15, endFrame: 17 },
      ],
    });
    expect(screen.queryByText(/no cuts can be softened/i)).toBeNull();
  });
});

describe('asking the solver again', () => {
  it('offers the button when there is a track to solve from', () => {
    show();
    expect(resolveButton()).not.toHaveProperty('disabled', true);
  });

  it('shuts the button when there is nothing to solve from', () => {
    show({ resolveRefusal: NO_FACES });
    expect(resolveButton()).toHaveProperty('disabled', true);
  });

  it('says why, rather than leaving a dead control to be guessed at', () => {
    show({ resolveRefusal: NO_FACES });
    expect(screen.getByText(new RegExp(NO_FACES, 'i'))).toBeTruthy();
  });

  it('does not claim faces are absent while it is still looking', () => {
    show({ resolveRefusal: 'Looking for a face track…' });
    expect(resolveButton()).toHaveProperty('disabled', true);
    expect(screen.queryByText(/nothing has detected faces/i)).toBeNull();
  });

  it('drops the usual explanation when it would describe something impossible', () => {
    // "Tracking weights are the solver's defaults" is a sentence about a solve
    // that can happen. Under a refusal it is describing nothing.
    show({ resolveRefusal: NO_FACES });
    expect(screen.queryByText(/tracking weights/i)).toBeNull();
  });

  it('still shuts the button while a solve is already running', () => {
    show({ resolving: true });
    expect(screen.getByRole('button', { name: /solving/i })).toHaveProperty('disabled', true);
  });
});

describe('a fitted clip', () => {
  it('sends an editor to the solver when one can be asked', () => {
    show({ plan: fitted() });
    expect(screen.getByText(/choose face crop to calculate/i)).toBeTruthy();
  });

  it('does not send them to a solver that cannot be asked', () => {
    show({ plan: fitted(), resolveRefusal: NO_FACES });
    expect(screen.queryByText(/choose face crop to calculate/i)).toBeNull();
    expect(screen.getByText(/face crop is unavailable/i)).toBeTruthy();
  });
});

it('calculates a path before switching a fitted clip into face crop', () => {
  const onResolve = vi.fn();
  const onApply = vi.fn();
  show({ plan: fitted(), onResolve, onApply });
  fireEvent.click(screen.getByRole('button', { name: /^face crop$/i }));
  expect(onResolve).toHaveBeenCalledOnce();
  expect(onApply).not.toHaveBeenCalled();
});

it('can restore stored portraits after fitting the frame without another solve', () => {
  const fittedPlan = fitted();
  const onApply = vi.fn();
  const onResolve = vi.fn();
  show({
    plan: {
      ...fittedPlan,
      segments: fittedPlan.segments.map((segment) => ({ ...segment, hasTwoUpPaths: true })),
    },
    onApply,
    onResolve,
  });
  fireEvent.click(screen.getByRole('button', { name: 'Restore two portraits' }));
  expect(onApply).toHaveBeenCalledWith({
    op: 'set_layout',
    segment_id: fittedPlan.segments[0]!.segmentId,
    state: 'two_up',
  });
  expect(onResolve).not.toHaveBeenCalled();
});

it('checks a two-person crop against its half-height viewport', () => {
  const base = solved();
  show({
    plan: {
      ...base,
      crops: base.crops.map(() => [0, 140, 900, 800] as const),
      secondaryCrops: base.crops.map(() => [1000, 140, 900, 800] as const),
    },
  });
  expect(screen.queryByText('check')).toBeNull();
  expect(screen.getAllByText('ok')).toHaveLength(2);
});

it('writes Fit to repair a legacy missing crop instead of treating the draft fallback as saved', () => {
  const base = fitted();
  const onApply = vi.fn();
  show({
    plan: {
      ...base,
      segments: base.segments.map((segment) => ({
        ...segment,
        framingWarning: 'Missing crop path. Choose Fit.',
      })),
    },
    onApply,
  });
  const fit = screen.getByRole('button', { name: 'Fit' });
  expect(fit.getAttribute('aria-pressed')).toBe('false');
  fireEvent.click(fit);
  expect(onApply).toHaveBeenCalledWith({
    op: 'set_layout',
    segment_id: base.segments[0]!.segmentId,
    state: 'fit',
  });
});
