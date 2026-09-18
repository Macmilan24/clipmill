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
  render(
    <Reframe
      plan={solved()}
      frame={0}
      busy={false}
      onApply={() => {}}
      onResolve={() => {}}
      resolving={false}
      resolveRefusal={null}
      {...overrides}
    />,
  );
}

function resolveButton() {
  return screen.getByRole('button', { name: /re-solve the path/i });
}

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
    expect(screen.getByText(/choose speaker-follow to calculate/i)).toBeTruthy();
  });

  it('does not send them to a solver that cannot be asked', () => {
    show({ plan: fitted(), resolveRefusal: NO_FACES });
    expect(screen.queryByText(/choose speaker-follow to calculate/i)).toBeNull();
    expect(screen.getByText(/speaker-follow is unavailable/i)).toBeTruthy();
  });
});

it('calculates a path before switching a fitted clip into speaker-follow', () => {
  const onResolve = vi.fn();
  const onApply = vi.fn();
  show({ plan: fitted(), onResolve, onApply });
  fireEvent.click(screen.getByRole('button', { name: /^speaker-follow$/i }));
  expect(onResolve).toHaveBeenCalledOnce();
  expect(onApply).not.toHaveBeenCalled();
});
