/**
 * Which document the export delivers.
 *
 * The export screen used to deliver the newest document of the newest project,
 * whatever had just been approved. Here a newer project exists with an edit of
 * its own, and the screen is handed a clip in the older one: every request it
 * sends must name that document, and the archive must be of that project.
 */
import { JobState, TaskState } from '@clipmill/contracts';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { ExportPlan, Job } from '../src/daemon/client.js';
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
import { type FakeWorld, fakeApi, job, task } from './support/library.js';

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
          revision: 0,
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
          revision: 0,
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

    fireEvent.click(await screen.findByRole('button', { name: /^export revision/i }));
    await waitFor(() => {
      expect(screen.getByTestId('delivery')).toBeTruthy();
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

function passing(revision = 0): ExportPlan {
  return {
    passes: true,
    findings: [],
    stem: '01-charging-less',
    fileNames: ['01-charging-less.mp4', '01-charging-less.srt'],
    estimatedBytes: 1_000,
    revision,
  };
}

/** Choose a folder and wait for the plan the daemon answers with. */
async function planned(world: FakeWorld) {
  await screen.findByTestId('export-clip');
  fireEvent.change(screen.getByLabelText(/folder/i), {
    target: { value: '/Users/sami/Movies/clips' },
  });
  await waitFor(() => {
    expect(world.exported.length).toBeGreaterThanOrEqual(1);
  });
}

/** The export job in a state, with the package artifact when delivered. */
function exportJob(state: JobState, tasks: readonly ReturnType<typeof task>[]): Job {
  return {
    ...job(OLD, state, tasks),
    jobId: 'job_export',
    kind: 'export-clip',
    // What the daemon's job says it is delivering: how the export is found
    // again by a screen that did not queue it.
    export: {
      docId: OLD_DOC,
      revision: 0,
      irArtifactId: 'sha256:ir-snapshot',
      destinationDir: '/Users/sami/Movies/clips',
    },
  };
}

const PACKAGE = {
  artifactId: 'sha256:package',
  kind: 'export.package.v1',
  json: JSON.stringify({
    schema_version: 'clipmill.export.package.v1',
    doc_id: OLD_DOC,
    title: '',
    render_artifact_id: 'sha256:render',
    video: {},
    audio: {},
    disclosure: {},
    files: [
      { name: '01-charging-less.mp4', role: 'clip', sha256: 'a'.repeat(64), bytes: 900 },
      { name: '01-charging-less.srt', role: 'subtitles_srt', sha256: 'b'.repeat(64), bytes: 90 },
    ],
  }),
};

describe('the export screen and the revision that was reviewed', () => {
  it('sends the revision the plan checked, and says which it is exporting', async () => {
    const world = twoProjects({
      exportPlan: passing(4),
      plan: { ...twoProjects().plan!, revision: 4 },
    });
    show(OLDER_CLIP, world);
    await planned(world);
    const button = await screen.findByRole('button', { name: /export revision r4/i });
    fireEvent.click(button);
    await waitFor(() => {
      expect(world.exported.some((request) => request.expectedRevision === 4)).toBe(true);
    });
  });

  it('re-plans and says so when the document moved since the review', async () => {
    // The plan checked revision 4; by the time the click lands the document is
    // at revision 5, and the daemon refuses. The screen shows the refusal and
    // plans again over what the document is now rather than exporting blind.
    const world = twoProjects({
      exportPlan: passing(4),
      plan: { ...twoProjects().plan!, revision: 5 },
    });
    show(OLDER_CLIP, world);
    await planned(world);
    const plansBefore = world.exported.length;
    fireEvent.click(await screen.findByRole('button', { name: /export revision r4/i }));
    expect(await screen.findByText(/moved since it was reviewed/i)).toBeTruthy();
    await waitFor(() => {
      expect(world.exported.length).toBeGreaterThan(plansBefore);
    });
    expect(screen.queryByTestId('delivery')).toBeNull();
  });
});

describe('following a queued export', () => {
  it('shows the render and the delivery as they happen', async () => {
    const base = twoProjects();
    const world = twoProjects({
      exportPlan: passing(0),
      jobs: {
        ...base.jobs,
        [OLD]: [
          ...base.jobs[OLD]!,
          exportJob(JobState.RUNNING, [
            task('render.clip.v1', TaskState.RUNNING, {
              progress: { unit: 'frames', done: 120, total: 900 },
            }),
            task('export.package.v1', TaskState.PLANNED),
          ]),
        ],
      },
    });
    show(OLDER_CLIP, world);
    await planned(world);
    fireEvent.click(await screen.findByRole('button', { name: /export revision r0/i }));
    const card = await screen.findByTestId('delivery');
    expect(card.textContent).toContain('Delivering revision r0');
    expect(card.textContent).toContain('/Users/sami/Movies/clips');
    await waitFor(() => {
      expect(screen.getByTestId('stage-render').textContent).toBe('120 of 900 frames');
    });
    expect(screen.getByTestId('stage-deliver').textContent).toBe('waiting');
    // Nothing can be exported twice while one is in flight.
    expect(screen.getByRole('button', { name: /export revision/i })).toHaveProperty(
      'disabled',
      true,
    );
  });

  it('lists the files the delivery wrote, at the folder it wrote them to, and reveals one', async () => {
    const base = twoProjects();
    const world = twoProjects({
      exportPlan: passing(0),
      jobs: {
        ...base.jobs,
        [OLD]: [
          ...base.jobs[OLD]!,
          exportJob(JobState.SUCCEEDED, [
            task('render.clip.v1', TaskState.SUCCEEDED),
            task('export.package.v1', TaskState.SUCCEEDED, {
              outputArtifactId: PACKAGE.artifactId,
            }),
          ]),
        ],
      },
      documents: { ...base.documents, [PACKAGE.artifactId]: PACKAGE },
    });
    show(OLDER_CLIP, world);
    await planned(world);
    fireEvent.click(await screen.findByRole('button', { name: /export revision r0/i }));
    const files = await screen.findByRole('list', { name: /delivered files/i });
    expect(files.textContent).toContain('/Users/sami/Movies/clips/01-charging-less.mp4');
    expect(files.textContent).toContain('/Users/sami/Movies/clips/01-charging-less.srt');
    expect(screen.getByTestId('delivery').textContent).toContain('Delivered revision r0');
    fireEvent.click(screen.getByRole('button', { name: /reveal 01-charging-less\.mp4/i }));
    expect(world.revealed).toEqual(['/Users/sami/Movies/clips/01-charging-less.mp4']);
  });

  /**
   * The reproduction: an export was queued, the screen was left and reopened
   * on the same clip, and the delivery panel was gone — the job id lived only
   * in the screen's state. The export is the daemon's; a fresh mount finds it
   * there.
   */
  it('finds an in-flight export again when the clip is reopened', async () => {
    const base = twoProjects();
    const world = twoProjects({
      exportPlan: passing(0),
      jobs: {
        ...base.jobs,
        [OLD]: [
          ...base.jobs[OLD]!,
          exportJob(JobState.RUNNING, [
            task('render.clip.v1', TaskState.RUNNING, {
              progress: { unit: 'frames', done: 12, total: 90 },
            }),
            task('export.package.v1', TaskState.PLANNED),
          ]),
        ],
      },
    });
    const api = fakeApi(world);
    const onOpen = vi.fn<(clip: ClipRef) => void>();
    const first = render(<ExportScreen clip={OLDER_CLIP} onOpen={onOpen} api={api} />);
    await planned(world);
    fireEvent.click(await screen.findByRole('button', { name: /export revision r0/i }));
    await screen.findByTestId('delivery');
    await waitFor(() => {
      expect(screen.getByTestId('stage-render').textContent).toBe('12 of 90 frames');
    });
    first.unmount();

    render(<ExportScreen clip={OLDER_CLIP} onOpen={onOpen} api={api} />);
    await screen.findByTestId('export-clip');
    const card = await screen.findByTestId('delivery');
    expect(card.textContent).toContain('Delivering revision r0');
    await waitFor(() => {
      expect(screen.getByTestId('stage-render').textContent).toBe('12 of 90 frames');
    });
    // And nothing can be exported on top of it while it runs.
    expect(screen.getByRole('button', { name: /^export$/i })).toHaveProperty('disabled', true);
  });

  /** After a relaunch nothing was ever queued from this screen; the daemon still knows. */
  it('shows a delivered export on a fresh start, from the daemon alone', async () => {
    const base = twoProjects();
    const world = twoProjects({
      exportPlan: passing(0),
      jobs: {
        ...base.jobs,
        [OLD]: [
          ...base.jobs[OLD]!,
          exportJob(JobState.SUCCEEDED, [
            task('render.clip.v1', TaskState.SUCCEEDED),
            task('export.package.v1', TaskState.SUCCEEDED, {
              outputArtifactId: PACKAGE.artifactId,
            }),
          ]),
        ],
      },
      documents: { ...base.documents, [PACKAGE.artifactId]: PACKAGE },
    });
    show(OLDER_CLIP, world);
    const files = await screen.findByRole('list', { name: /delivered files/i });
    expect(files.textContent).toContain('/Users/sami/Movies/clips/01-charging-less.mp4');
    expect(screen.getByTestId('delivery').textContent).toContain('Delivered revision r0');
    // Nothing was asked of the daemon that it had not already done.
    expect(world.exported.filter((request) => 'expectedRevision' in request)).toHaveLength(0);
  });

  it('follows the newest export of the document, not another document\u2019s', async () => {
    const base = twoProjects();
    const other = {
      ...exportJob(JobState.SUCCEEDED, [task('render.clip.v1', TaskState.SUCCEEDED)]),
      jobId: 'job_export_other',
      createdUnixMillis: 9_999_999_999_999,
      export: {
        docId: NEW_DOC,
        revision: 4,
        irArtifactId: 'sha256:other',
        destinationDir: '/elsewhere',
      },
    };
    const world = twoProjects({
      exportPlan: passing(0),
      jobs: {
        ...base.jobs,
        [OLD]: [
          ...base.jobs[OLD]!,
          other,
          exportJob(JobState.RUNNING, [
            task('render.clip.v1', TaskState.RUNNING),
            task('export.package.v1', TaskState.PLANNED),
          ]),
        ],
      },
    });
    show(OLDER_CLIP, world);
    const card = await screen.findByTestId('delivery');
    expect(card.textContent).toContain('Delivering revision r0');
    expect(card.textContent).toContain('/Users/sami/Movies/clips');
    expect(card.textContent).not.toContain('/elsewhere');
  });

  /** A read that fails is not an outcome; the job goes on and so does the asking. */
  it('keeps following an export across a lost read rather than calling it settled', async () => {
    const base = twoProjects();
    const running = exportJob(JobState.RUNNING, [
      task('render.clip.v1', TaskState.RUNNING, {
        progress: { unit: 'frames', done: 30, total: 90 },
      }),
      task('export.package.v1', TaskState.PLANNED),
    ]);
    const world = twoProjects({
      exportPlan: passing(0),
      jobs: { ...base.jobs, [OLD]: [...base.jobs[OLD]!, running] },
    });
    const api = fakeApi(world);
    let reads = 0;
    const flaky = {
      ...api,
      fetchJob: (jobId: string) => {
        reads += 1;
        return reads === 2
          ? Promise.reject(new Error('the daemon is restarting'))
          : api.fetchJob(jobId);
      },
    };
    render(<ExportScreen clip={OLDER_CLIP} onOpen={vi.fn()} api={flaky} />);
    await planned(world);
    fireEvent.click(await screen.findByRole('button', { name: /export revision r0/i }));
    const card = await screen.findByTestId('delivery');
    expect(await screen.findByTestId('delivery-interruption')).toBeTruthy();
    expect(card.textContent).toContain('the daemon is restarting');
    expect(card.textContent).not.toContain('was not delivered');
    // The next read gets through, and the interruption is gone with it.
    await waitFor(
      () => {
        expect(reads).toBeGreaterThanOrEqual(3);
        expect(screen.queryByTestId('delivery-interruption')).toBeNull();
      },
      { timeout: 5_000 },
    );
    expect(screen.getByTestId('stage-render').textContent).toBe('30 of 90 frames');
  });

  it('says why an export failed, in the daemon\u2019s words', async () => {
    const base = twoProjects();
    const world = twoProjects({
      exportPlan: passing(0),
      jobs: {
        ...base.jobs,
        [OLD]: [
          ...base.jobs[OLD]!,
          {
            ...exportJob(JobState.FAILED, [
              task('render.clip.v1', TaskState.FAILED),
              task('export.package.v1', TaskState.CANCELLED),
            ]),
            failureDetail: 'the encoder ran out of disk at frame 411',
          },
        ],
      },
    });
    show(OLDER_CLIP, world);
    await planned(world);
    fireEvent.click(await screen.findByRole('button', { name: /export revision r0/i }));
    expect(await screen.findByText(/ran out of disk at frame 411/i)).toBeTruthy();
    expect(screen.getByTestId('stage-render').textContent).toBe('failed');
    expect(screen.queryByRole('list', { name: /delivered files/i })).toBeNull();
    // The failure is over; another export can be asked for.
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /export revision/i })).toHaveProperty(
        'disabled',
        false,
      );
    });
  });
});
