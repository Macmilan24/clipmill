/**
 * Which document the editor opens, and whose proxy it plays.
 *
 * The editor used to open the newest document of the newest project — right
 * for one project with one approval, and wrong for everything after. Every
 * test here has a newer project with an analysis and an edit of its own, and
 * asks about the older one: the plan fetched must be the older document's,
 * the proxy must be the older recording's, and the name on screen must be the
 * clip's own.
 */
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { TooltipProvider } from '../src/components/ui/tooltip.js';
import { EditorScreen } from '../src/screens/EditorScreen.js';
import type { ClipRef } from '../src/shell/route.js';
import {
  CANDIDATE,
  NEW,
  NEW_DOC,
  OLD,
  OLD_DOC,
  OLD_JOB,
  OLD_PROXY,
  OLD_SOURCE,
  document,
  twoProjects,
} from './support/clips.js';
import { type FakeWorld, fakeApi } from './support/library.js';

const OLDER_CLIP: ClipRef = {
  projectId: OLD,
  docId: OLD_DOC,
  sourceId: OLD_SOURCE,
  candidateId: CANDIDATE,
  jobId: OLD_JOB,
  labels: { project: 'CUDA kernels', clip: 'Clip 01' },
};

function show(clip: ClipRef | null, world: FakeWorld = twoProjects()) {
  const onOpen = vi.fn<(clip: ClipRef) => void>();
  const onExport = vi.fn<(clip: ClipRef) => void>();
  render(
    <TooltipProvider>
      <EditorScreen
        clip={clip}
        onOpenResults={() => {}}
        onOpen={onOpen}
        onExport={onExport}
        api={fakeApi(world)}
      />
    </TooltipProvider>,
  );
  return { world, onOpen, onExport };
}

describe('the editor, handed a clip in an older project', () => {
  it('asks for that document and no other', async () => {
    const { world } = show(OLDER_CLIP);
    await screen.findByTestId('stage');
    expect(world.planned).toEqual([[OLD, OLD_DOC]]);
  });

  it('plays the older recording, not the proxy the project published last', async () => {
    const { world } = show(OLDER_CLIP);
    const stage = await screen.findByTestId('stage');
    const video = stage.querySelector('video');
    expect(video?.getAttribute('src')).toBe(
      `clipmill-media://localhost/${OLD}/${OLD_PROXY}/proxy.mp4`,
    );
    // Nothing about the newer project was touched.
    expect(world.planned.some(([projectId]) => projectId === NEW)).toBe(false);
  });

  it('names the clip it is editing', async () => {
    show(OLDER_CLIP);
    expect((await screen.findByTestId('clip-name')).textContent).toBe('Clip 01');
    expect(screen.getByText(/CUDA kernels/)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Export this clip' })).toBeTruthy();
  });

  it('takes the same clip, unchanged, to the export screen', async () => {
    const { onExport } = show(OLDER_CLIP);
    fireEvent.click(await screen.findByRole('button', { name: /export this clip/i }));
    expect(onExport).toHaveBeenCalledWith(OLDER_CLIP);
  });

  it('reads the proxy from the newest run of the clip’s source when no run was named', async () => {
    const { jobId: _unnamed, ...unnamed } = OLDER_CLIP;
    show(unnamed);
    const stage = await screen.findByTestId('stage');
    expect(stage.querySelector('video')?.getAttribute('src')).toContain(`/${OLD}/${OLD_PROXY}/`);
  });

  it('says which clip could not be opened when the daemon refuses it', async () => {
    const { plan: _none, ...refusing } = twoProjects();
    show(OLDER_CLIP, refusing);
    expect(await screen.findByText(/this clip could not be opened/i)).toBeTruthy();
    expect(screen.getByText(/no such document/i)).toBeTruthy();
  });
});

describe('the editor, reached with no clip named', () => {
  it('lists every edit there is, newest first, rather than opening the newest', async () => {
    const world = twoProjects({
      editDocs: [
        document(NEW, NEW_DOC, CANDIDATE, { updatedUnixMillis: 5_000 }),
        document(OLD, OLD_DOC, CANDIDATE, { updatedUnixMillis: 9_000, revision: 4 }),
      ],
    });
    const { onOpen } = show(null, world);
    const list = await screen.findByRole('list', { name: /edits/i });
    const items = list.querySelectorAll('li');
    expect(items).toHaveLength(2);
    // The older project's edit was touched more recently, so it leads.
    expect(items[0]?.textContent).toContain('CUDA kernels');
    expect(items[0]?.textContent).toContain('r4');
    expect(items[1]?.textContent).toContain('Dogfood episode');
    // Nothing was opened on anyone's behalf.
    expect(world.planned).toEqual([]);

    fireEvent.click(items[0]!.querySelector('button')!);
    expect(onOpen).toHaveBeenCalledTimes(1);
    expect(onOpen.mock.calls[0]?.[0]).toMatchObject({
      projectId: OLD,
      docId: OLD_DOC,
      sourceId: OLD_SOURCE,
      candidateId: CANDIDATE,
      labels: { project: 'CUDA kernels' },
    });
  });

  it('says so when there are no edits at all', async () => {
    show(null, twoProjects({ editDocs: [] }));
    expect(await screen.findByText(/no clip is open/i)).toBeTruthy();
    await waitFor(() => {
      expect(screen.queryByText(/looking for edits/i)).toBeNull();
    });
    expect(screen.queryByRole('list', { name: /edits/i })).toBeNull();
  });
});
