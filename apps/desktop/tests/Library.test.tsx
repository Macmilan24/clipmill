/**
 * The Library screen, driven end to end through a daemon that answers from
 * memory.
 *
 * The screen is injected with a loader rather than reaching for the real one, so
 * these exercise the whole path — gather, derive, render — without a window or a
 * socket. What they check is the part of the design that was deliberately not
 * built to the letter: no invented scores, a search field that promises only
 * what it does, and empty states that say which of the several empties this is.
 */
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { JobState, TaskState } from '@clipmill/contracts';
import { describe, expect, it, vi } from 'vitest';

import type { ConnectionState } from '../src/daemon/client.js';
import { LibraryLoader } from '../src/library/loader.js';
import { Library } from '../src/screens/Library.js';
import {
  type FakeWorld,
  emptyWorld,
  fakeApi,
  filmstrip,
  job,
  progress,
  project,
  source,
  sourceMapDocument,
  task,
} from './support/library.js';

const CONNECTED: ConnectionState = {
  status: 'connected',
  daemonVersion: '0.0.1',
  localLock: true,
  startedUnixMillis: 0,
};

const FILMSTRIP_ID = 'sha256:art-media.filmstrip.v1';

function world(): FakeWorld {
  return {
    ...emptyWorld(),
    projects: [
      project('p1', 'Episode 41 — pricing mistakes', 3_600_000),
      project('p2', 'CUDA kernels, part 3', 7_200_000),
    ],
    jobs: {
      p1: [
        job('p1', JobState.SUCCEEDED, [
          task('media.filmstrip.v1', TaskState.SUCCEEDED),
          task('ranking.set.v1', TaskState.SUCCEEDED),
        ]),
      ],
      p2: [
        job('p2', JobState.RUNNING, [
          task('ranking.set.v1', TaskState.RUNNING, {
            progress: progress('candidates', 11, 18),
          }),
        ]),
      ],
    },
    sources: { p1: [source('p1')], p2: [source('p2')] },
    documents: {
      'sha256:map-p1': sourceMapDocument('sha256:map-p1'),
      'sha256:map-p2': sourceMapDocument('sha256:map-p2'),
    },
    media: { [FILMSTRIP_ID]: filmstrip(FILMSTRIP_ID, 5) },
    storage: {
      categories: [
        { key: 'artifacts', bytes: 20_078_895_104, items: 412 },
        { key: 'models', bytes: 24_588_615_680, items: 6 },
        { key: 'state', bytes: 43_008_000, items: 4 },
      ],
      availableBytes: 335_007_449_088,
    },
  };
}

function show(overrides: Partial<Parameters<typeof Library>[0]> = {}, scene = world()) {
  const onNavigate = vi.fn();
  render(
    <Library
      state={CONNECTED}
      onNavigate={onNavigate}
      onReconnect={vi.fn()}
      loader={new LibraryLoader(fakeApi(scene))}
      {...overrides}
    />,
  );
  return { onNavigate };
}

describe('the Library screen', () => {
  it('shows every project with what was actually measured about it', async () => {
    show();

    expect(await screen.findByText('Episode 41 — pricing mistakes')).toBeTruthy();
    expect(screen.getByText('CUDA kernels, part 3')).toBeTruthy();
    expect(screen.getByText('2 projects')).toBeTruthy();
    // Duration, resolution and size come from the probe and the source record.
    expect(screen.getAllByText('1:42:07').length).toBeGreaterThan(0);
    expect(screen.getByText('1920×1080 · 29.97 fps · 7.8 GB')).toBeTruthy();
  });

  /**
   * The design's card carries a score ring — 94, 91, 86. No such number exists:
   * the ranker scores candidates, not projects. Inventing one would be the whole
   * failure this screen was written to avoid.
   */
  it('shows no score, because no project has one', async () => {
    show();
    await screen.findByText('Episode 41 — pricing mistakes');
    expect(screen.queryByText(/^\d{2}$/)).toBeNull();
  });

  it('names the stage a running analysis is on, counted and never as a percent', async () => {
    show();
    expect(await screen.findByText('Rank candidates · 11 of 18 candidates')).toBeTruthy();
    expect(screen.queryByText(/%/)).toBeNull();
  });

  it('says the search covers titles, and searches titles', async () => {
    show();
    await screen.findByText('Episode 41 — pricing mistakes');

    const search = screen.getByLabelText('Search project titles');
    expect(search.getAttribute('placeholder')).toBe('Search project titles');

    fireEvent.change(search, { target: { value: 'cuda' } });
    await waitFor(() => {
      expect(screen.queryByText('Episode 41 — pricing mistakes')).toBeNull();
    });
    expect(screen.getByText('CUDA kernels, part 3')).toBeTruthy();
  });

  it('distinguishes an empty store from a search that matched nothing', async () => {
    show();
    await screen.findByText('Episode 41 — pricing mistakes');

    fireEvent.change(screen.getByLabelText('Search project titles'), {
      target: { value: 'zzzz' },
    });
    expect(await screen.findByText('Nothing matches')).toBeTruthy();
  });

  it('offers an honest empty state when the store has no projects', async () => {
    show({}, emptyWorld());
    expect(await screen.findByText('No projects yet')).toBeTruthy();
    expect(screen.queryByLabelText('Sort projects')).toBeNull();
  });

  it('says so rather than listing nothing when the daemon is not connected', () => {
    show({ state: { status: 'disconnected', reason: 'socket closed' } });
    expect(screen.getByText('Daemon not connected')).toBeTruthy();
  });

  it('filters by a status only when some project is in it', async () => {
    show();
    await screen.findByText('Episode 41 — pricing mistakes');

    const chips = within(screen.getByRole('group', { name: 'Filter by status' }));
    expect(chips.queryByRole('button', { name: /Failed/ })).toBeNull();
    fireEvent.click(chips.getByRole('button', { name: /Analyzing/ }));
    await waitFor(() => {
      expect(screen.queryByText('Episode 41 — pricing mistakes')).toBeNull();
    });
  });

  it('shows the same projects as rows, with their numbers in columns', async () => {
    show();
    await screen.findByText('Episode 41 — pricing mistakes');

    fireEvent.click(screen.getByLabelText('List view'));
    const table = await screen.findByRole('table');
    expect(within(table).getByText('Episode 41 — pricing mistakes')).toBeTruthy();
    expect(within(table).getAllByText('1:42:07')).toHaveLength(2);
  });

  it('reports measured storage and free space, both from the daemon', async () => {
    show();
    expect(await screen.findByText('Artifacts 18.7 GB · Models 22.9 GB · State 41 MB')).toBeTruthy();
    expect(screen.getByText('312 GB free')).toBeTruthy();
  });

  it('says storage was not measured rather than showing zeroes', async () => {
    show({}, { ...world(), storage: null });
    expect(await screen.findByText('Storage not measured')).toBeTruthy();
    expect(screen.queryByText(/free$/)).toBeNull();
  });

  it('sends the primary action to the screen that imports', async () => {
    const { onNavigate } = show();
    await screen.findByText('Episode 41 — pricing mistakes');

    fireEvent.click(screen.getByRole('button', { name: /Import video/ }));
    expect(onNavigate).toHaveBeenCalledWith('new-project');
  });

  it('opens a project at its results', async () => {
    const { onNavigate } = show();

    fireEvent.click(await screen.findByText('Episode 41 — pricing mistakes'));
    expect(onNavigate).toHaveBeenCalledWith('results');
  });
});
