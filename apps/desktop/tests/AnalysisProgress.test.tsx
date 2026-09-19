/**
 * One run, on screen.
 *
 * The design's version of this page is full of things nobody measures: a 62%
 * completion figure, per-stage durations, live GPU load and temperature, and a
 * log of invented lines. These check that what replaced them is real — a counted
 * fraction of stages, elapsed time, the device profile labelled as measured
 * rather than sampled, and a log that says it begins when the screen opens.
 */
import { JobState, TaskState } from '@clipmill/contracts';
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { AnalysisLoader } from '../src/analysis/useAnalysis.js';
import { AnalysisProgress } from '../src/screens/AnalysisProgress.js';
import {
  type FakeWorld,
  emptyWorld,
  fakeApi,
  filmstrip,
  job,
  project,
  readiness,
  source,
  sourceMapDocument,
  stageReadiness,
  task,
} from './support/library.js';

const FILMSTRIP_ID = 'sha256:art-media.filmstrip.v1';

function running(): FakeWorld {
  return {
    ...emptyWorld(),
    projects: [project('p1', 'Episode 41')],
    jobs: {
      p1: [
        job('p1', JobState.RUNNING, [
          task('evidence.source_map.v1', TaskState.SUCCEEDED),
          task('media.proxy.v1', TaskState.SUCCEEDED),
          task('media.filmstrip.v1', TaskState.SUCCEEDED),
          task('media.frames.v1', TaskState.RUNNING),
          task('speech.vad.v1', TaskState.PLANNED),
          task('speech.asr.v1', TaskState.PLANNED),
          task('speech.alignment.v1', TaskState.PLANNED),
          task('speech.transcript.v1', TaskState.PLANNED),
          task('index.transcript.v1', TaskState.PLANNED),
          task('editorial.windows.v1', TaskState.PLANNED),
          task('discovery.candidates.v1', TaskState.PLANNED),
          task('ranking.set.v1', TaskState.PLANNED),
        ]),
      ],
    },
    sources: { p1: [source('p1')] },
    documents: { 'sha256:map-p1': sourceMapDocument('sha256:map-p1') },
    media: { [FILMSTRIP_ID]: filmstrip(FILMSTRIP_ID, 5) },
  };
}

function show(scene = running(), overrides: Record<string, unknown> = {}) {
  const onBack = vi.fn();
  const onNavigate = vi.fn();
  render(
    <AnalysisProgress
      projectId="p1"
      jobId="job-p1"
      profile={null}
      onBack={onBack}
      onNavigate={onNavigate}
      loader={new AnalysisLoader(fakeApi(scene))}
      {...overrides}
    />,
  );
  return { onBack, onNavigate };
}

describe('the Analysis Progress screen', () => {
  it('retries the selected run source locally even when another recording is listed first', async () => {
    const selected = source('p1');
    const scene = {
      ...running(),
      sources: {
        p1: [{ ...selected, sourceId: 'src_wrong', absolutePath: '/wrong.mp4' }, selected],
      },
      jobs: { p1: [job('p1', JobState.FAILED)] },
    };
    const api = fakeApi(scene);
    const submitAnalyze = vi.fn(api.submitAnalyze);
    const onRestarted = vi.fn();
    show(scene, { loader: new AnalysisLoader({ ...api, submitAnalyze }), onRestarted });
    fireEvent.click(await screen.findByRole('button', { name: 'Analyze again locally' }));
    await waitFor(() =>
      expect(submitAnalyze).toHaveBeenCalledWith('p1', {
        sourceId: selected.sourceId,
        language: 'en',
        minTicks: 20 * 90000,
        maxTicks: 90 * 90000,
        count: 5,
        localEditorial: true,
        contentProfile: 'interview',
      }),
    );
    await waitFor(() => expect(onRestarted).toHaveBeenCalledWith('p1', 'job-p1'));
  });

  it('shows every stage of the pipeline, named for a reader', async () => {
    show();
    const pipeline = within(await screen.findByRole('list', { name: 'Pipeline stages' }));
    expect(pipeline.getAllByRole('listitem')).toHaveLength(15);
    for (const label of [
      'Inspect source',
      'Ingest',
      'Find speech',
      'Recognise speech',
      'Align words',
      'Assemble transcript',
      'Detect shots',
      'Index transcript',
      'Cut windows',
      'Validate candidates',
      'Rank candidates',
    ]) {
      expect(pipeline.getByText(label)).toBeTruthy();
    }
  });

  /**
   * The design puts "62%" beside the title. Stages differ in cost by two orders
   * of magnitude, so eight of ten finished is not eighty percent of the wait —
   * the fraction is counted and labelled as stages.
   */
  it('counts stages rather than claiming a percentage', async () => {
    show();
    expect(await screen.findByText(/of \d+ stages ·/)).toBeTruthy();
    expect(screen.queryByText(/%/)).toBeNull();
  });

  it('shows ingest as one row that counts its derivatives', async () => {
    show();
    const pipeline = within(await screen.findByRole('list', { name: 'Pipeline stages' }));
    expect(pipeline.getByText('2 of 3 derivatives')).toBeTruthy();
    // The derivatives never appear as rows of their own.
    expect(pipeline.queryByText('media.proxy.v1')).toBeNull();
  });

  it('says a stage nobody planned is not needed rather than pending', async () => {
    const scene = {
      ...emptyWorld(),
      projects: [project('p1', 'Radio show')],
      jobs: {
        p1: [
          job('p1', JobState.RUNNING, [
            task('evidence.source_map.v1', TaskState.SUCCEEDED),
            task('speech.vad.v1', TaskState.RUNNING),
          ]),
        ],
      },
    };
    show(scene);
    const pipeline = within(await screen.findByRole('list', { name: 'Pipeline stages' }));
    expect(pipeline.getByText('Detect shots')).toBeTruthy();
    expect(pipeline.getAllByText('not needed').length).toBeGreaterThan(0);
    expect(pipeline.getAllByText('Not needed for this recording').length).toBeGreaterThan(0);
  });

  it('reports the source from its probe, not from its name', async () => {
    show();
    expect(await screen.findByText('1:42:07 · 1920×1080 · 29.97 fps')).toBeTruthy();
    expect(screen.getByText('7.8 GB')).toBeTruthy();
  });

  /** The design animates GPU load and temperature; nothing samples either. */
  it('says the device figures were measured, not sampled', async () => {
    show();
    expect(
      await screen.findByText('Measured at the last device profile. Nothing here is sampled live.'),
    ).toBeTruthy();
    expect(screen.queryByText(/°C/)).toBeNull();
  });

  it('says the log begins when the screen opens rather than implying silence', async () => {
    show();
    expect(
      await screen.findByText(
        'Waiting for the next transition. This log starts when the screen opens.',
      ),
    ).toBeTruthy();
  });

  it('keeps the results button disabled until the run has finished', async () => {
    show();
    const button = await screen.findByRole('button', { name: 'View results when ready' });
    expect(button.hasAttribute('disabled')).toBe(true);
  });

  it('offers results once the run succeeded', async () => {
    const scene = running();
    const finished = {
      ...scene,
      jobs: {
        p1: [
          job(
            'p1',
            JobState.SUCCEEDED,
            scene.jobs.p1![0]!.tasks.map((entry) => task(entry.outputKind, TaskState.SUCCEEDED)),
          ),
        ],
      },
    };
    show(finished);
    const button = await screen.findByRole('button', { name: 'View results' });
    expect(button.hasAttribute('disabled')).toBe(false);
  });

  it("says what a waiting stage is waiting for, in the daemon's words", async () => {
    // The task fixture names its kind after the artifact it publishes; the
    // readiness report is keyed by that same task kind.
    const scene = {
      ...running(),
      readiness: readiness([
        stageReadiness('speech.asr', { workerPresent: false }),
        stageReadiness('speech.vad'),
      ]),
    };
    show(scene);
    const reason = await screen.findByTestId('waiting-speech.asr.v1');
    expect(reason.textContent).toMatch(/No worker is connected that runs speech\.asr/);
    expect(reason.textContent).toMatch(/just workers/);
    // A stage whose worker is here waits on its dependencies, not on a remedy.
    expect(screen.queryByTestId('waiting-speech.vad.v1')).toBeNull();
    // The running stage is not waiting.
    expect(screen.queryByTestId('waiting-media.frames.v1')).toBeNull();
  });

  it('stops naming waits once the run has finished', async () => {
    const scene = running();
    const finished = {
      ...scene,
      readiness: readiness([stageReadiness('speech.asr', { workerPresent: false })]),
      jobs: {
        p1: [
          job(
            'p1',
            JobState.FAILED,
            scene.jobs.p1![0]!.tasks.map((entry) =>
              task(
                entry.outputKind,
                entry.state === TaskState.SUCCEEDED ? TaskState.SUCCEEDED : TaskState.CANCELLED,
              ),
            ),
          ),
        ],
      },
    };
    show(finished);
    await screen.findByRole('list', { name: 'Pipeline stages' });
    expect(screen.queryByTestId('waiting-speech.asr.v1')).toBeNull();
  });

  it('shows why a run stopped instead of only that it did', async () => {
    const scene = running();
    const failed = {
      ...scene,
      jobs: {
        p1: [
          {
            ...job('p1', JobState.FAILED, [task('speech.asr.v1', TaskState.FAILED)]),
            failureDetail: 'the speech model is not installed',
          },
        ],
      },
    };
    show(failed);
    expect(await screen.findByText('the speech model is not installed')).toBeTruthy();
  });

  it('goes back to where it was opened from', async () => {
    const { onBack } = show();
    (await screen.findByRole('button', { name: 'Back' })).click();
    expect(onBack).toHaveBeenCalled();
  });

  it('says so when the run is not in the store', async () => {
    show(emptyWorld());
    expect(await screen.findByText('no such job')).toBeTruthy();
  });
});
