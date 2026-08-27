/**
 * The board as a person meets it.
 *
 * The pure joins are covered in `results.test.ts`; what these hold is the part
 * only a rendered tree can be wrong about — that a chip's number matches the
 * rows behind it, that search narrows what is on screen, and that a clip with
 * nothing recorded against it shows no signal dots rather than a reassuring
 * green one.
 */
import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Results } from '../src/screens/Results.js';
import type { ClipRow, Summary } from '../src/results/model.js';

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
    axes: [],
    penalties: [],
    boundary: null,
    decision: null,
    latticeStarts: [],
    latticeEnds: [],
    ...overrides,
  };
}

const SUMMARY: Summary = {
  selected: 2,
  cohort: 5,
  requested: 4,
  shortfall: [],
  filtered: 1,
};

function board(overrides: Partial<Parameters<typeof Results>[0]> = {}) {
  render(
    <Results
      loading={false}
      rows={[
        row(),
        row({
          candidateId: 'cand_2',
          rank: 2,
          displayScore: 61,
          band: 'needs_review',
          bandLabel: 'Needs review',
          headline: 'The freemium trap for B2B SaaS',
          warnings: ['opens on an unresolved pronoun'],
          decision: 'approved',
        }),
      ]}
      summary={SUMMARY}
      problem={null}
      sourceName="Episode 41"
      proxyUrl={null}
      projects={[]}
      activeProjectId={null}
      onChooseProject={() => {}}
      onInspect={() => {}}
      onReload={() => {}}
      {...overrides}
    />,
  );
}

function rowsOnScreen() {
  return within(screen.getByRole('listbox', { name: /clip candidates/i })).getAllByRole('option');
}

describe('the board', () => {
  it('names the recording and counts the candidates', () => {
    board();
    expect(screen.getByText('Episode 41')).toBeTruthy();
    expect(screen.getByRole('heading', { name: /2 clip candidates/i })).toBeTruthy();
  });

  it('shows the shortfall rather than padding it away', () => {
    board({ summary: { ...SUMMARY, selected: 3, shortfall: ['2 duplicate cuts refused'] } });
    expect(screen.getByText(/2 duplicate cuts refused/)).toBeTruthy();
  });

  it('narrows to what was searched for', () => {
    board();
    expect(rowsOnScreen()).toHaveLength(2);
    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'freemium' } });
    const shown = rowsOnScreen();
    expect(shown).toHaveLength(1);
    expect(within(shown[0]!).getByText(/freemium trap/i)).toBeTruthy();
  });

  it('says so rather than showing an empty table when nothing matches', () => {
    board();
    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'zzzz' } });
    expect(screen.getByText(/no clip matches those filters/i)).toBeTruthy();
  });

  it('gives each chip the count that pressing it would leave', () => {
    board();
    const approved = screen.getByRole('button', { name: /approved/i });
    expect(within(approved).getByText('1')).toBeTruthy();
    fireEvent.click(approved);
    expect(rowsOnScreen()).toHaveLength(1);
  });

  it('disables a chip that would leave nothing', () => {
    board();
    // No row is 'promising' in this fixture, so the chip must not invite a
    // click that can only produce an empty table.
    expect(screen.getByRole('button', { name: /promising/i })).toHaveProperty('disabled', true);
  });

  it('draws a signal dot only where the ranker recorded something', () => {
    board();
    const [first, second] = rowsOnScreen();
    expect(within(first!).getByRole('group', { name: /no signals recorded/i })).toBeTruthy();
    expect(within(second!).getByTitle('opens on an unresolved pronoun')).toBeTruthy();
  });

  it('opens a clip on double click, and only selects on a single one', () => {
    let opened: string | null = null;
    board({ onInspect: (id) => (opened = id) });
    const [first] = rowsOnScreen();
    fireEvent.click(first!);
    expect(opened).toBeNull();
    fireEvent.doubleClick(first!);
    expect(opened).toBe('cand_1');
  });

  it('moves the selection with the keyboard and opens it with Enter', () => {
    let opened: string | null = null;
    board({ onInspect: (id) => (opened = id) });
    const list = screen.getByRole('listbox', { name: /clip candidates/i });
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    fireEvent.keyDown(list, { key: 'Enter' });
    expect(opened).toBe('cand_2');
  });

  it('reports why there is nothing, when there is nothing', () => {
    board({ problem: { kind: 'not-analyzed' }, rows: [], summary: null });
    expect(screen.getByText(/has not been analyzed/i)).toBeTruthy();
  });
});

describe('reaching the inspector', () => {
  it('gives every row its own way in, for the widths where the rail is hidden', () => {
    let opened: string | null = null;
    board({ onInspect: (id) => (opened = id) });
    fireEvent.click(screen.getByRole('button', { name: /open your first pricing model/i }));
    expect(opened).toBe('cand_1');
  });

  it('does not let opening a row be mistaken for selecting it', () => {
    // The row click selects; the chevron opens. If the chevron's click bubbled,
    // opening would also move the rail, which is the wrong clip on the way out.
    let opened: string | null = null;
    board({ onInspect: (id) => (opened = id) });
    fireEvent.click(screen.getByRole('button', { name: /open the freemium trap/i }));
    expect(opened).toBe('cand_2');
  });
});
