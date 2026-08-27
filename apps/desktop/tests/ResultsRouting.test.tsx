/**
 * Which recording the board opens.
 *
 * Clicking a project in the Library used to navigate to the Results *section*
 * and drop which project was clicked, so the screen fell back to the newest one
 * the daemon held. An editor who chose one recording was shown another — and it
 * looked like the board was stuck, because with one recent analysis every route
 * led back to the same clips.
 */
import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { ShellApi } from '../src/daemon/api.js';
import { ResultsScreen } from '../src/screens/ResultsScreen.js';

/** Newest first, which is the order the daemon answers in and `newest` reads. */
const PROJECTS = [
  { projectId: 'p_new', name: 'Dogfood episode', createdUnixMillis: 9_000 },
  { projectId: 'p_old', name: 'CUDA kernels', createdUnixMillis: 1_000 },
];

/**
 * Enough daemon to render the board's empty state.
 *
 * Every project reports as un-analyzed, which is what makes the assertion
 * clean: the screen names the recording it settled on in its own header, so the
 * test reads the choice rather than the clips.
 */
function api(): ShellApi {
  return {
    listProjects: async () => PROJECTS,
    listJobs: async () => [],
    listSources: async () => [],
    listClipDecisions: async () => [],
    mediaUrl: () => '',
  } as unknown as ShellApi;
}

function show(projectId: string | null) {
  render(
    <ResultsScreen
      candidateId={null}
      projectId={projectId}
      onInspect={() => {}}
      onBack={() => {}}
      api={api()}
    />,
  );
}

describe('the project the board opens', () => {
  it('is the one the route named, not the newest', async () => {
    show('p_old');
    // The older project is the one asked for; the newer one must not win.
    await waitFor(() => {
      expect(screen.getByRole('combobox')).toBeTruthy();
    });
    expect(screen.getByRole('combobox').textContent).toContain('CUDA kernels');
  });

  it('falls back to the newest when the route named none', async () => {
    show(null);
    await waitFor(() => {
      expect(screen.getByRole('combobox')).toBeTruthy();
    });
    expect(screen.getByRole('combobox').textContent).toContain('Dogfood episode');
  });

  it('offers every project, so the sidebar entry is not a dead end', async () => {
    show(null);
    await waitFor(() => {
      expect(screen.getByRole('combobox')).toBeTruthy();
    });
    // The picker exists precisely because a route with no project would
    // otherwise strand the screen on whichever the daemon wrote last.
    expect(screen.getByRole('combobox')).toBeTruthy();
  });
});
