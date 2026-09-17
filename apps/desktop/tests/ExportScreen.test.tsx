/**
 * Which document the export delivers.
 *
 * The export screen used to deliver the newest document of the newest project,
 * whatever had just been approved. Here a newer project exists with an edit of
 * its own, and the screen is handed a clip in the older one: every request it
 * sends must name that document, and the archive must be of that project.
 */
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { ExportScreen } from '../src/screens/ExportScreen.js';
import type { ClipRef } from '../src/shell/route.js';
import {
  CANDIDATE,
  NEW,
  NEW_DOC,
  OLD,
  OLD_DOC,
  OLD_JOB,
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
  render(<ExportScreen clip={clip} onOpen={onOpen} api={fakeApi(world)} />);
  return { world, onOpen };
}

describe('the export screen, handed a clip in an older project', () => {
  it('reads that document’s plan and names the clip', async () => {
    const { world } = show(
      OLDER_CLIP,
      twoProjects({
        exportPlan: {
          passes: true,
          findings: [],
          stem: '01-charging-less',
          fileNames: ['01-charging-less.mp4'],
          estimatedBytes: 1_000,
        },
      }),
    );
    expect((await screen.findByTestId('export-clip')).textContent).toContain(
      'CUDA kernels · Clip 01',
    );
    expect(world.planned).toEqual([[OLD, OLD_DOC]]);
    expect(screen.getByText(OLD_DOC)).toBeTruthy();
  });

  it('plans and queues the export of that document, not the newest', async () => {
    const { world } = show(
      OLDER_CLIP,
      twoProjects({
        exportPlan: {
          passes: true,
          findings: [],
          stem: '01-charging-less',
          fileNames: ['01-charging-less.mp4'],
          estimatedBytes: 1_000,
        },
      }),
    );
    await screen.findByTestId('export-clip');
    fireEvent.change(screen.getByLabelText(/folder/i), {
      target: { value: '/Users/sami/Movies/clips' },
    });
    await waitFor(() => {
      expect(world.exported.length).toBeGreaterThanOrEqual(1);
    });
    expect(world.exported.every((request) => request.docId === OLD_DOC)).toBe(true);

    fireEvent.click(await screen.findByRole('button', { name: /^export$/i }));
    await waitFor(() => {
      expect(screen.getByText(/queued as/i)).toBeTruthy();
    });
    expect(world.exported.at(-1)).toMatchObject({
      docId: OLD_DOC,
      destinationDir: '/Users/sami/Movies/clips',
    });
    expect(world.exported.some((request) => request.docId === NEW_DOC)).toBe(false);
  });

  it('archives the clip’s project', async () => {
    const { world } = show(OLDER_CLIP);
    await screen.findByTestId('export-clip');
    fireEvent.change(screen.getByLabelText(/folder/i), {
      target: { value: '/Users/sami/Movies/clips' },
    });
    fireEvent.click(screen.getByRole('button', { name: /archive this project/i }));
    await waitFor(() => {
      expect(world.archived).toHaveLength(1);
    });
    expect(world.archived[0]).toEqual([OLD, '/Users/sami/Movies/clips']);
  });
});

describe('the export screen, reached with no clip named', () => {
  it('lists the edits to choose from rather than delivering the newest', async () => {
    const world = twoProjects({
      editDocs: [document(NEW, NEW_DOC, CANDIDATE), document(OLD, OLD_DOC, CANDIDATE)],
    });
    const { onOpen } = show(null, world);
    expect(await screen.findByText(/no clip is chosen/i)).toBeTruthy();
    const list = await screen.findByRole('list', { name: /edits/i });
    expect(list.querySelectorAll('li')).toHaveLength(2);
    expect(world.planned).toEqual([]);
    expect(world.exported).toEqual([]);

    const older = [...list.querySelectorAll('li')].find((item) =>
      item.textContent?.includes('CUDA kernels'),
    );
    fireEvent.click(older!.querySelector('button')!);
    expect(onOpen.mock.calls[0]?.[0]).toMatchObject({ projectId: OLD, docId: OLD_DOC });
  });
});
