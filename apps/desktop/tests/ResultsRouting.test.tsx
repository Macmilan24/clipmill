/**
 * Which recording the board opens, and which clip approving hands on.
 *
 * Clicking a project in the Library used to navigate to the Results *section*
 * and drop which project was clicked, so the screen fell back to the newest one
 * the daemon held. An editor who chose one recording was shown another — and it
 * looked like the board was stuck, because with one recent analysis every route
 * led back to the same clips.
 *
 * Approving had the same failure one screen later: it wrote a decision, built
 * a document, and said "sent to the editor" without saying which — and the
 * editor then opened the newest document of the newest project. So the second
 * half of this file is the older-project scenario: a newer project exists, has
 * been analyzed, and has an edit of its own, and approving a clip in the older
 * one must name the older one's document, in full, to whatever opens next.
 */
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { TooltipProvider } from '../src/components/ui/tooltip.js';
import type { ShellApi } from '../src/daemon/api.js';
import { ResultsScreen } from '../src/screens/ResultsScreen.js';
import type { ClipRef } from '../src/shell/route.js';
import {
  CANDIDATE,
  OLD,
  OTHER_CANDIDATE,
  OLD_DOC,
  OLD_JOB,
  OLD_SOURCE,
  document,
  twoProjects,
} from './support/clips.js';
import { fakeApi, sourceMap, sourceMapDocument } from './support/library.js';

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
    listEditDocs: async () => [],
    mediaUrl: () => '',
  } as unknown as ShellApi;
}

function show(projectId: string | null) {
  render(
    <ResultsScreen
      candidateId={null}
      projectId={projectId}
      sourceId={null}
      jobId={null}
      onInspect={() => {}}
      onEdit={() => {}}
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

/** The Inspector, open on the older project's first clip. */
function inspect(world = twoProjects(), onEdit = vi.fn<(clip: ClipRef) => void>()) {
  const shell = fakeApi(world);
  const decided = vi.spyOn(shell, 'setClipDecision');
  render(
    <TooltipProvider>
      <ResultsScreen
        candidateId={CANDIDATE}
        projectId={OLD}
        sourceId={OLD_SOURCE}
        jobId={OLD_JOB}
        onInspect={() => {}}
        onEdit={onEdit}
        onBack={() => {}}
        api={shell}
      />
    </TooltipProvider>,
  );
  return { world, onEdit, decided };
}

describe('approving a clip in an older project', () => {
  it('asks the daemon for that project, source and candidate, approving in the same call', async () => {
    const { world, decided } = inspect();
    const approve = await screen.findByRole('button', { name: /approve for the editor/i });
    fireEvent.click(approve);
    await waitFor(() => {
      expect(world.directed).toHaveLength(1);
    });
    expect(world.directed[0]).toMatchObject({
      projectId: OLD,
      sourceId: OLD_SOURCE,
      candidateId: CANDIDATE,
      cut: 'chosen',
      approve: true,
      // The run the board is showing, so the director reads that run's
      // stages and not whichever a re-analysis published last.
      jobId: OLD_JOB,
    });
    // One write, not two. A decision recorded first and a document that then
    // failed to build was how a clip ended up approved with nothing to open.
    expect(decided).not.toHaveBeenCalled();
  });

  it('hands the editor the older project, its source, its run and the document it got back', async () => {
    const { onEdit } = inspect();
    fireEvent.click(await screen.findByRole('button', { name: /approve for the editor/i }));
    await waitFor(() => {
      expect(onEdit).toHaveBeenCalledTimes(1);
    });
    expect(onEdit.mock.calls[0]?.[0]).toEqual({
      projectId: OLD,
      docId: 'edt_00000000000000000000000000',
      sourceId: OLD_SOURCE,
      candidateId: CANDIDATE,
      jobId: OLD_JOB,
      labels: { project: 'CUDA kernels', clip: 'Clip 01' },
    });
  });

  it('reopens the edit the clip already has rather than opening a second one', async () => {
    // The older project has an edit of this clip from an earlier session, and
    // so does the newer project. Approving again must open the older one's.
    const world = twoProjects({
      editDocs: [
        document('p_new', 'edt_0000000000000000000000NEW1', CANDIDATE),
        // Cut from an earlier run of this recording than the board shows.
        document(OLD, OLD_DOC, CANDIDATE, { revision: 3, jobId: 'job-older-run' }),
      ],
    });
    const { onEdit } = inspect(world);
    fireEvent.click(await screen.findByRole('button', { name: /approve and open the edit/i }));
    await waitFor(() => {
      expect(onEdit).toHaveBeenCalledTimes(1);
    });
    // The editor is handed the document's own run, not the board's: its
    // captions and face tracks are that run's.
    expect(onEdit.mock.calls[0]?.[0]).toMatchObject({
      projectId: OLD,
      docId: OLD_DOC,
      jobId: 'job-older-run',
    });
    expect(screen.getByRole('status').textContent).toMatch(/already has an edit/i);
  });

  it('offers the existing edit beside the approval, and opens that one', async () => {
    const world = twoProjects({ editDocs: [document(OLD, OLD_DOC, CANDIDATE)] });
    const { onEdit, world: seen } = inspect(world);
    fireEvent.click(await screen.findByRole('button', { name: /open the existing edit/i }));
    expect(onEdit).toHaveBeenCalledTimes(1);
    expect(onEdit.mock.calls[0]?.[0]).toMatchObject({
      projectId: OLD,
      docId: OLD_DOC,
      sourceId: OLD_SOURCE,
      candidateId: CANDIDATE,
    });
    // Opening what exists asks the director for nothing.
    expect(seen.directed).toHaveLength(0);
  });

  it('asks for a different cut as a variation, so the existing edit is not handed back instead', async () => {
    const world = twoProjects({ editDocs: [document(OLD, OLD_DOC, CANDIDATE)] });
    const { world: seen } = inspect(world);
    // The runner-up cut lives on the boundary tab; Radix tabs switch on
    // pointer-down rather than click.
    fireEvent.mouseDown(await screen.findByRole('tab', { name: /boundary/i }));
    fireEvent.click(await screen.findByRole('button', { name: /use the alternative/i }));
    await waitFor(() => {
      expect(seen.directed).toHaveLength(1);
    });
    expect(seen.directed[0]).toMatchObject({ cut: 'alternative', variation: true });
    expect(seen.directed[0]?.approve).not.toBe(true);
  });
});

it('does not substitute the newest project when the requested project is missing', async () => {
  show('removed-project');
  expect(await screen.findByText(/this project is no longer available/i)).toBeTruthy();
  expect(screen.getByRole('combobox').textContent).toContain('Choose a project');
});

it('does not reopen a clip after the user leaves it during approval', async () => {
  const shell = fakeApi(twoProjects());
  const original = shell.directClip;
  let finish!: () => void;
  const pending = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const direct = vi.spyOn(shell, 'directClip').mockImplementation(async (request) => {
    await pending;
    return original(request);
  });
  const onEdit = vi.fn();
  const at = (candidateId: string) => (
    <TooltipProvider>
      <ResultsScreen
        candidateId={candidateId}
        projectId={OLD}
        sourceId={OLD_SOURCE}
        jobId={OLD_JOB}
        onInspect={() => {}}
        onEdit={onEdit}
        onBack={() => {}}
        api={shell}
      />
    </TooltipProvider>
  );
  const view = render(at(CANDIDATE));
  fireEvent.click(await screen.findByRole('button', { name: /approve for the editor/i }));
  await waitFor(() => expect(direct).toHaveBeenCalledOnce());
  view.rerender(at(OTHER_CANDIDATE));
  await act(async () => finish());
  await waitFor(() =>
    expect(screen.getByRole('button', { name: /approve for the editor/i })).toHaveProperty(
      'disabled',
      false,
    ),
  );
  expect(onEdit).not.toHaveBeenCalled();
});

it('opens the actual manual document identity returned for an older named run', async () => {
  const world = twoProjects();
  const selected = world.sources[OLD]![0]!;
  const map = sourceMapDocument(
    selected.sourceMapArtifactId,
    sourceMap({
      source_fingerprint: selected.sourceFingerprint,
    }),
  );
  const shell = fakeApi({ ...world, documents: { ...world.documents, [map.artifactId]: map } });
  const original = shell.directClip;
  const direct = vi.spyOn(shell, 'directClip').mockImplementation(async (input) => ({
    ...(await original(input)),
    candidateId: 'manual_600_630',
    docId: 'edt_manual_source_span',
  }));
  const onEdit = vi.fn();
  render(
    <TooltipProvider>
      <ResultsScreen
        candidateId={null}
        projectId={OLD}
        sourceId={OLD_SOURCE}
        jobId={OLD_JOB}
        onInspect={() => {}}
        onEdit={onEdit}
        onBack={() => {}}
        api={shell}
      />
    </TooltipProvider>,
  );
  fireEvent.click(await screen.findByRole('button', { name: 'Make a manual clip' }));
  fireEvent.change(screen.getByLabelText('Start time'), { target: { value: '10:00' } });
  fireEvent.change(screen.getByLabelText('End time'), { target: { value: '10:30' } });
  fireEvent.click(screen.getByRole('button', { name: 'Create manual edit' }));
  await waitFor(() => expect(onEdit).toHaveBeenCalledOnce());
  expect(direct).toHaveBeenCalledWith(
    expect.objectContaining({
      candidateId: '',
      manualSpan: true,
      approve: false,
      jobId: OLD_JOB,
      startTicks: 600 * 90_000,
      endTicks: 630 * 90_000,
    }),
  );
  expect(onEdit).toHaveBeenCalledWith({
    projectId: OLD,
    sourceId: OLD_SOURCE,
    jobId: OLD_JOB,
    candidateId: 'manual_600_630',
    docId: 'edt_manual_source_span',
    labels: { project: 'CUDA kernels', clip: 'Manual clip' },
  });
});
