/**
 * Starting a run, from choosing a file to handing off a job id.
 *
 * The design's setup page offers a great deal this build cannot do — creative
 * direction compiled into scoring parameters, audience and tone, speaker-aware
 * reframing, burnt-in captions, a folder watcher and a YouTube importer. What is
 * checked here is that the controls which survived all change something the
 * daemon receives, and that the ones which state a fact — the cloud toggle, the
 * rights gate — behave as facts rather than as decoration.
 */
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { AnalyzeRequest, ConnectionState } from '../src/daemon/client.js';
import { ImportLoader } from '../src/import/loader.js';
import { NewProject } from '../src/screens/NewProject.js';
import { type FakeWorld, emptyWorld, fakeApi, sourceMapDocument } from './support/library.js';

const CONNECTED: ConnectionState = {
  status: 'connected',
  daemonVersion: '0.0.1',
  localLock: true,
  startedUnixMillis: 0,
};

const PATH = '/Volumes/Creator/Podcast/Episode 41/pricing-mistakes-episode-41.mp4';

function scene(overrides: Partial<FakeWorld> = {}): FakeWorld {
  return {
    ...emptyWorld(),
    chosenPath: PATH,
    // The register fake hands back a source whose map artifact id is derived
    // from the project id, which is derived from the file name.
    documents: {
      'sha256:map-prj_pricing-mistakes-episode-41': sourceMapDocument(
        'sha256:map-prj_pricing-mistakes-episode-41',
      ),
    },
    ...overrides,
  };
}

function show(world = scene()) {
  const onStarted = vi.fn();
  const api = fakeApi(world);
  const submitted: { projectId: string; request: AnalyzeRequest }[] = [];
  const spied = {
    ...api,
    submitAnalyze: (projectId: string, request: AnalyzeRequest) => {
      submitted.push({ projectId, request });
      return api.submitAnalyze(projectId, request);
    },
  };
  render(<NewProject state={CONNECTED} onStarted={onStarted} loader={new ImportLoader(spied)} />);
  return { onStarted, submitted };
}

async function chooseFile(): Promise<void> {
  fireEvent.click(screen.getByRole('button', { name: 'Browse files' }));
  await screen.findByText('pricing-mistakes-episode-41.mp4');
}

describe('the New Project screen', () => {
  it('will not start until a file has been chosen', () => {
    show();
    const start = screen.getByRole('button', { name: /Analyze video/ });
    expect(start.hasAttribute('disabled')).toBe(true);
    expect(screen.getByText('Choose a video first')).toBeTruthy();
  });

  it('shows what the daemon probed rather than what the file is called', async () => {
    show();
    await chooseFile();
    expect(screen.getByText(/1:42:07 · 1920×1080 · 29.97 fps/)).toBeTruthy();
  });

  it('shows the whole path, middle-truncated so the name survives', async () => {
    show();
    await chooseFile();
    const shown = screen.getByTitle(PATH);
    expect(shown.textContent?.endsWith('pricing-mistakes-episode-41.mp4')).toBe(true);
  });

  /**
   * A gate, not a decoration: it blocks the one primary action, and the copy
   * under the button says which of the several reasons is the current one.
   */
  it('requires the rights attestation and says so', async () => {
    show();
    await chooseFile();

    expect(screen.getByText('Confirm you hold the rights to this footage')).toBeTruthy();
    expect(screen.getByRole('button', { name: /Analyze video/ }).hasAttribute('disabled')).toBe(
      true,
    );

    fireEvent.click(screen.getByRole('checkbox'));
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Analyze video/ }).hasAttribute('disabled')).toBe(
        false,
      );
    });
  });

  /** The cloud toggle is a statement of what this build does, not a preference. */
  it('leaves the cloud switch off and unswitchable', () => {
    show();
    const cloud = screen.getByRole('switch', { name: 'Use cloud models' });
    expect(cloud.getAttribute('data-state')).toBe('unchecked');
    expect(cloud.hasAttribute('disabled')).toBe(true);
  });

  it('sends the preset, the count and the language the form was set to', async () => {
    const { submitted, onStarted } = show();
    await chooseFile();

    fireEvent.click(screen.getByLabelText(/Extended/));
    fireEvent.click(screen.getByRole('button', { name: 'More clips' }));
    fireEvent.click(screen.getByRole('checkbox'));
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Analyze video/ }).hasAttribute('disabled')).toBe(
        false,
      );
    });
    fireEvent.click(screen.getByRole('button', { name: /Analyze video/ }));

    await waitFor(() => {
      expect(submitted).toHaveLength(1);
    });
    // Ticks, at the contract's 1/90000, converted once on this side.
    expect(submitted[0]?.request).toEqual({
      sourceId: 'src_prj_pricing-mistakes-episode-41',
      language: '',
      minTicks: 60 * 90_000,
      maxTicks: 180 * 90_000,
      count: 6,
    });
    await waitFor(() => {
      expect(onStarted).toHaveBeenCalledWith(
        'prj_pricing-mistakes-episode-41',
        'job-prj_pricing-mistakes-episode-41',
      );
    });
  });

  it('lets the bounds be set by hand, and refuses a range that is backwards', async () => {
    show();
    await chooseFile();
    fireEvent.click(screen.getByRole('checkbox'));
    fireEvent.click(screen.getByLabelText(/Custom/));

    const shortest = await screen.findByLabelText('Shortest');
    fireEvent.change(shortest, { target: { value: '200' } });
    await waitFor(() => {
      expect(screen.getByText('The shortest clip must be shorter than the longest')).toBeTruthy();
    });
  });

  /** None of the four appear, because nothing behind them exists yet. */
  it('offers nothing the pipeline cannot act on', () => {
    show();
    for (const absent of [
      /Creative direction/,
      /YouTube/,
      /Folder watch/,
      /Speaker-aware/,
      /Burn captions/,
      /Estimated/,
    ]) {
      expect(screen.queryByText(absent)).toBeNull();
    }
  });

  it('says what closing the app actually does', async () => {
    show();
    await chooseFile();
    fireEvent.click(screen.getByRole('checkbox'));
    expect(
      await screen.findByText('Closing ClipMill pauses the run; it resumes when you reopen.'),
    ).toBeTruthy();
  });

  it('reports a refusal instead of leaving the button spinning', async () => {
    const world = scene();
    const api = fakeApi(world);
    render(
      <NewProject
        state={CONNECTED}
        onStarted={vi.fn()}
        loader={
          new ImportLoader({
            ...api,
            registerSource: () => Promise.reject(new Error('ffprobe found no streams')),
          })
        }
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Browse files' }));
    expect(await screen.findByText('ffprobe found no streams')).toBeTruthy();
  });
});
