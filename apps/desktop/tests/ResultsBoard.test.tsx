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
    docId: null,
    docJobId: null,
    latticeStarts: [],
    latticeEnds: [],
    recommended: false,
    proposer: null,
    clusterId: null,
    hook: null,
    payoff: null,
    flagged: false,
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
          flagged: true,
          decision: 'approved',
        }),
      ]}
      summary={SUMMARY}
      problem={null}
      sourceName="Episode 41"
      run={null}
      tileUrl={() => null}
      projects={[]}
      activeProjectId={null}
      busy={false}
      onChooseProject={() => {}}
      onApproveMany={() => {}}
      onInspect={() => {}}
      onEdit={() => {}}
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
    board({ summary: { ...SUMMARY, selected: 3, shortfall: ['2 duplicate cuts refused.'] } });
    expect(screen.getByText(/2 duplicate cuts refused/).textContent).toBe(
      '4 asked for, 3 recommended: 2 duplicate cuts refused.',
    );
  });

  it('shows missing coverage when the requested count was met', () => {
    board({
      summary: {
        ...SUMMARY,
        selected: 4,
        warnings: ['2 of 10 windows could not be assessed.', '1 candidate could not be reviewed.'],
      },
      run: { jobId: 'job_PARTIAL', state: 3, completedUnixMillis: 0 },
    });
    const notice = screen.getByRole('status', { name: 'Incomplete analysis' });
    expect(within(notice).getByText('2 of 10 windows could not be assessed.')).toBeTruthy();
    expect(within(notice).getByText('1 candidate could not be reviewed.')).toBeTruthy();
    expect(screen.getByText('Partial results')).toBeTruthy();
    expect(screen.queryByText('Analyzed')).toBeNull();
  });

  it('does not call an empty partial analysis evidence that no moments exist', () => {
    board({
      rows: [],
      summary: {
        ...SUMMARY,
        selected: 0,
        cohort: 0,
        warnings: ['Visual checks were unavailable for 1 candidate.'],
      },
      run: { jobId: 'job_PARTIAL', state: 3, completedUnixMillis: 0 },
    });
    expect(screen.getByRole('status', { name: 'Incomplete analysis' })).toBeTruthy();
    expect(
      screen.getByText(/unassessed sections may still contain worthwhile moments/i),
    ).toBeTruthy();
    expect(screen.queryByText(/no suitable complete moments were found/i)).toBeNull();
  });

  it('offers source recovery for no nominations without empty filters or a dead detail rail', () => {
    let opened = false;
    board({
      rows: [],
      summary: { ...SUMMARY, selected: 0, cohort: 0 },
      onManualClip: () => {
        opened = true;
      },
    });
    expect(screen.getByText('No clips are ready for review')).toBeTruthy();
    expect(screen.queryByRole('searchbox')).toBeNull();
    expect(screen.queryByText(/select a clip to see why/i)).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Choose a source interval' }));
    expect(opened).toBe(true);
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
    // No row is recommended in this fixture, so the chip must not invite a
    // click that can only produce an empty table.
    expect(screen.getByRole('button', { name: /^recommended/i })).toHaveProperty('disabled', true);
  });

  it("shows the ranker's recommendation as a state, since it is its one opinion", () => {
    board({ rows: [row({ recommended: true })] });
    // Scoped to the row: the word also names a chip and a stat, which is the
    // point — the same fact is counted, filterable and worn by the row.
    const [only] = rowsOnScreen();
    expect(within(only!).getByText('Recommended')).toBeTruthy();
  });

  it('lets a person decide about several clips at once, through the same path as one', () => {
    const approved: string[][] = [];
    board({ onApproveMany: (ids) => approved.push([...ids]) });
    fireEvent.click(screen.getByRole('checkbox', { name: /select your first pricing/i }));
    fireEvent.click(screen.getByRole('checkbox', { name: /select the freemium/i }));
    fireEvent.click(screen.getByRole('button', { name: /approve 2 selected/i }));
    expect(approved).toEqual([['cand_1', 'cand_2']]);
  });

  it('switches to cards without changing what any card says', () => {
    board();
    fireEvent.click(screen.getByRole('button', { name: /grid view/i }));
    // The same two clips, the same states — only the layout moved.
    const cards = screen.getAllByRole('option');
    expect(cards).toHaveLength(2);
    expect(within(cards[1]!).getByText('Approved')).toBeTruthy();
  });

  it('states the run it describes, from the job rather than a constant', () => {
    board({
      run: { jobId: 'job_01ABCDEFGH', state: 3, completedUnixMillis: 0 },
    });
    expect(screen.getByText('Analyzed')).toBeTruthy();
    expect(screen.getByText(/run_01ABCDEF/)).toBeTruthy();
  });

  it('draws a signal dot only where the ranker recorded something', () => {
    board();
    const [first, second] = rowsOnScreen();
    expect(within(first!).getByRole('group', { name: /no signals recorded/i })).toBeTruthy();
    expect(within(second!).getByTitle('opens on an unresolved pronoun')).toBeTruthy();
  });

  it('names editorial review without claiming that model-reviewed clips have no evidence', () => {
    board({
      rows: [
        row({ review: { status: 'needs_review', route: 'local', reasons: ['Visual context'] } }),
      ],
    });
    const [first] = rowsOnScreen();
    expect(within(first!).getByText('Editorial review')).toBeTruthy();
    expect(within(first!).queryByRole('group', { name: /no signals recorded/i })).toBeNull();
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

describe('the separate declined collection', () => {
  it('shows a rejected-only run as inspectable declines with no bulk approval or recommendations', () => {
    const inspected: string[] = [];
    board({
      rows: [
        row({
          candidateId: 'declined-1',
          band: 'declined',
          bandLabel: 'Declined by editorial review',
          review: { status: 'rejected', route: 'local', reasons: ['The answer never finishes.'] },
        }),
      ],
      summary: {
        selected: 0,
        cohort: 0,
        requested: 5,
        filtered: 1,
        declined: 1,
        contentProfile: 'scripted',
        shortfall: [],
      },
      onInspect: (id) => inspected.push(id),
    });
    expect(screen.getByText('Declined by the model')).toBeTruthy();
    expect(screen.getByText('The answer never finishes.')).toBeTruthy();
    expect(screen.getByText(/TV \/ movie scenes/)).toBeTruthy();
    expect(screen.queryByRole('listbox', { name: /clip candidates/i })).toBeNull();
    expect(screen.queryByRole('button', { name: /Approve/ })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: /Inspect declined moment/ }));
    expect(inspected).toEqual(['declined-1']);
  });

  it('keeps declines out of the suggested clip list and selection counts', () => {
    board({
      rows: [
        row({ recommended: true }),
        row({
          candidateId: 'declined-1',
          headline: 'A declined alternative',
          review: { status: 'rejected', route: 'local', reasons: ['Incomplete'] },
          band: 'declined',
          bandLabel: 'Declined',
        }),
      ],
    });
    expect(rowsOnScreen()).toHaveLength(1);
    expect(rowsOnScreen()[0]?.textContent).not.toContain('A declined alternative');
    expect(screen.getByRole('heading', { name: '1 clip candidate' })).toBeTruthy();
    expect(screen.getByRole('list', { name: 'Declined moments', hidden: true })).toBeTruthy();
  });
});
