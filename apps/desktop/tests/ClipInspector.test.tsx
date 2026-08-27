/**
 * The Inspector's honesty, held to.
 *
 * Every claim here is one the screen could quietly get wrong in a way that looks
 * fine: an axis nobody measured drawn as a zero, an alternative cut offered when
 * the lattice held only one legal pair, a clip with nothing recorded against it
 * shown as though it had been checked and cleared.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { ClipInspector } from '../src/screens/ClipInspector.js';
import type { ClipRow } from '../src/results/model.js';

const SECOND = 90_000;

function row(overrides: Partial<ClipRow> = {}): ClipRow {
  return {
    candidateId: 'cand_1',
    rank: 1,
    displayScore: 92,
    band: 'strong',
    bandLabel: 'Strong',
    warnings: [],
    startTicks: 10 * SECOND,
    endTicks: 40 * SECOND,
    durationSeconds: 30,
    headline: 'Your first pricing model is probably backwards',
    axes: [
      {
        axis: 'hook',
        label: 'Hook',
        value: 0.8,
        weight: 1.4,
        unavailableReason: null,
        evidence: ['Most founders price from fear.'],
      },
      {
        axis: 'prompt_relevance',
        label: 'Prompt fit',
        value: null,
        weight: null,
        unavailableReason: 'no prompt was given; prompt retrieval is not one of this phase',
        evidence: [],
      },
    ],
    penalties: [],
    boundary: null,
    decision: null,
    latticeStarts: [9 * SECOND, 10 * SECOND],
    latticeEnds: [40 * SECOND, 42 * SECOND],
    ...overrides,
  };
}

/**
 * Move to a tab the way a person does.
 *
 * Radix switches on pointer-down rather than on a synthesised `click`, so a
 * bare click leaves the panel where it was and the assertion that follows fails
 * for a reason that has nothing to do with the screen.
 */
function openTab(name: RegExp) {
  const tab = screen.getByRole('tab', { name });
  fireEvent.mouseDown(tab);
  return tab;
}

function show(overrides: Partial<Parameters<typeof ClipInspector>[0]> = {}) {
  const props = {
    rows: [row()],
    candidateId: 'cand_1',
    proxyUrl: null,
    crop: null,
    cues: [],
    busy: false,
    notice: null,
    onSelect: () => {},
    onBack: () => {},
    onDecide: () => {},
    onUseAlternative: () => {},
    ...overrides,
  };
  render(<ClipInspector {...props} />);
}

describe('the score panel', () => {
  it('says an axis was not measured instead of drawing it as a zero', () => {
    show();
    expect(screen.getByText(/not measured/i)).toBeTruthy();
    expect(screen.getByText(/no prompt was given/i)).toBeTruthy();
    // A zero-valued bar would be indistinguishable from a scored zero.
    expect(screen.queryByRole('img', { name: /prompt fit/i })).toBeNull();
  });

  it('counts how many axes were measured, so a thin card cannot look full', () => {
    show();
    expect(screen.getByText('1 of 2')).toBeTruthy();
  });
});

describe('the boundary panel', () => {
  it('offers the runner-up only when the lattice held one', () => {
    show({
      rows: [
        row({
          boundary: {
            startTicks: 10 * SECOND,
            endTicks: 40 * SECOND,
            score: 0.5,
            terms: [{ name: 'pronoun_open', value: -0.2 }],
            alternative: null,
          },
        }),
      ],
    });
    openTab(/boundary/i);
    expect(screen.getByText(/offered one legal pair/i)).toBeTruthy();
    expect(screen.queryByRole('button', { name: /use the alternative/i })).toBeNull();
  });

  it('says so when the candidate carries no boundary record at all', () => {
    show();
    openTab(/boundary/i);
    expect(screen.getByText(/carries no boundary record/i)).toBeTruthy();
  });

  it('rebuilds from the alternative when there is one to take', () => {
    let asked = false;
    show({
      rows: [
        row({
          boundary: {
            startTicks: 10 * SECOND,
            endTicks: 40 * SECOND,
            score: 0.5,
            terms: [{ name: 'pronoun_open', value: -0.2 }],
            alternative: { startTicks: 9 * SECOND, endTicks: 42 * SECOND },
          },
        }),
      ],
      onUseAlternative: () => {
        asked = true;
      },
    });
    openTab(/boundary/i);
    fireEvent.click(screen.getByRole('button', { name: /use the alternative/i }));
    expect(asked).toBe(true);
  });
});

describe('the risk panel', () => {
  it('says nothing was recorded rather than implying a clean bill', () => {
    show();
    openTab(/risk/i);
    expect(screen.getByText(/recorded nothing against this clip/i)).toBeTruthy();
  });

  it('shows each penalty with the score it cost', () => {
    show({ rows: [row({ penalties: [{ reason: 'repetition', value: 7 }] })] });
    openTab(/risk/i);
    expect(screen.getByText('repetition')).toBeTruthy();
    expect(screen.getByText('−7')).toBeTruthy();
  });
});

describe('deciding', () => {
  it('records each of the three decisions', () => {
    const seen: string[] = [];
    show({ onDecide: (decision) => seen.push(decision) });
    fireEvent.click(screen.getByRole('button', { name: /approve for the editor/i }));
    fireEvent.click(screen.getByRole('button', { name: /keep for later/i }));
    fireEvent.click(screen.getByRole('button', { name: /reject/i }));
    expect(seen).toEqual(['approved', 'kept', 'rejected']);
  });

  it('shuts every decision while one is in flight', () => {
    show({ busy: true });
    expect(screen.getByRole('button', { name: /working/i })).toHaveProperty('disabled', true);
    expect(screen.getByRole('button', { name: /keep for later/i })).toHaveProperty(
      'disabled',
      true,
    );
  });

  it('shows what the last action said', () => {
    show({ notice: 'Sent to the editor.' });
    expect(screen.getByRole('status')).toBeTruthy();
    expect(screen.getByText('Sent to the editor.')).toBeTruthy();
  });
});

describe('a candidate that is not in the ranking', () => {
  it('says so rather than rendering an empty inspector', () => {
    show({ candidateId: 'cand_missing' });
    expect(screen.getByText(/not in the current ranking/i)).toBeTruthy();
  });
});
