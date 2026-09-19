import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ShellApi } from '../src/daemon/api.js';
import type { SourceDetails, YoutubeImport } from '../src/daemon/client.js';
import { ImportLoader } from '../src/import/loader.js';
import { YouTubeImport } from '../src/import/YouTubeImport.js';
import { recallYoutube, rememberYoutube } from '../src/import/youtube.js';
import { NewProject } from '../src/screens/NewProject.js';
import { emptyWorld, fakeApi, source, sourceMapDocument } from './support/library.js';

const URL = 'https://youtu.be/NYFGCESmikA?si=XwSGgLl0qT1Wz0tB';
const ID = 'NYFGCESmikA';
function record(overrides: Partial<YoutubeImport> = {}): YoutubeImport {
  return {
    importId: 'imp_1',
    projectId: 'prj_1',
    videoId: ID,
    canonicalUrl: `https://www.youtube.com/watch?v=${ID}`,
    title: '',
    channel: '',
    state: 'downloading',
    attempt: 0,
    downloadedBytes: 1024,
    sourceId: '',
    errorCode: '',
    error: '',
    createdUnixMillis: 1,
    updatedUnixMillis: 2,
    ...overrides,
  };
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
function show(overrides: Partial<ShellApi> = {}) {
  const api: ShellApi = { ...fakeApi(emptyWorld()), ...overrides };
  const onChosen = vi.fn();
  const view = render(
    <YouTubeImport
      importer={new ImportLoader(api)}
      connected
      disabled={false}
      onChosen={onChosen}
    />,
  );
  return { api, onChosen, ...view };
}
function authorize() {
  fireEvent.change(screen.getByLabelText('Video link'), { target: { value: URL } });
  fireEvent.click(screen.getByRole('checkbox', { name: /creator’s permission/ }));
}
const scrollDescriptor = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'scrollIntoView');
beforeEach(() => {
  Object.defineProperty(HTMLElement.prototype, 'scrollIntoView', {
    configurable: true,
    value: vi.fn(),
  });
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
});
afterEach(() => {
  localStorage.clear();
  vi.unstubAllGlobals();
  if (scrollDescriptor)
    Object.defineProperty(HTMLElement.prototype, 'scrollIntoView', scrollDescriptor);
  else Reflect.deleteProperty(HTMLElement.prototype, 'scrollIntoView');
});

describe('YouTube source import', () => {
  it('requires a valid single video and explicit permission before any network action', async () => {
    const start = vi.fn<ShellApi['startYoutubeImport']>().mockResolvedValue(record());
    show({ startYoutubeImport: start, getYoutubeImport: () => Promise.resolve(record()) });
    const button = screen.getByRole('button', { name: 'Import video' });
    fireEvent.change(screen.getByLabelText('Video link'), { target: { value: URL } });
    expect(button).toHaveProperty('disabled', true);
    expect(start).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('checkbox'));
    fireEvent.click(button);
    await waitFor(() =>
      expect(start).toHaveBeenCalledExactlyOnceWith(
        `prj_YouTube · ${ID}`,
        `https://www.youtube.com/watch?v=${ID}`,
        true,
        1080,
      ),
    );
    expect(screen.getByText(/does not connect your account or enable cloud AI/)).toBeTruthy();
    expect(screen.queryByRole('progressbar')).toBeNull();
    expect(screen.getByText(/1 KB downloaded.*total size not yet known/)).toBeTruthy();
  });

  it('opens the exact completed source and real title without starting analysis', async () => {
    const status = deferred<YoutubeImport>();
    const start = vi.fn<ShellApi['submitAnalyze']>();
    const getSource = vi.fn<ShellApi['getSource']>().mockResolvedValue({
      source: source('prj_1', { sourceId: 'src_imported' }),
      sourceMapJson: sourceMapDocument('map').json,
    });
    const { onChosen } = show({
      startYoutubeImport: () => Promise.resolve(record()),
      getYoutubeImport: () => status.promise,
      getSource,
      submitAnalyze: start,
    });
    authorize();
    fireEvent.click(screen.getByRole('button', { name: 'Import video' }));
    await screen.findByRole('region', { name: 'YouTube import status' });
    expect(onChosen).not.toHaveBeenCalledWith(
      expect.objectContaining({ source: expect.anything() }),
    );
    await act(async () =>
      status.resolve(
        record({
          state: 'completed',
          sourceId: 'src_imported',
          title: 'The creator’s actual video title',
          updatedUnixMillis: 3,
        }),
      ),
    );
    await waitFor(() =>
      expect(onChosen).toHaveBeenCalledWith(
        expect.objectContaining({
          projectId: 'prj_1',
          title: 'The creator’s actual video title',
          source: expect.objectContaining({ sourceId: 'src_imported' }),
        }),
      ),
    );
    expect(getSource).toHaveBeenCalledExactlyOnceWith('src_imported');
    expect(start).not.toHaveBeenCalled();
  });

  it('restores the explicitly selected older import, not the newest one, and never automatically retries', async () => {
    const old = record({
      importId: 'imp_old',
      state: 'interrupted',
      title: 'Older selected video',
    });
    const other = record({
      importId: 'imp_new',
      projectId: 'prj_new',
      title: 'Newest unrelated video',
      updatedUnixMillis: 99,
    });
    rememberYoutube({ importId: old.importId, projectId: old.projectId, videoId: old.videoId });
    const update = vi
      .fn<ShellApi['updateYoutubeImport']>()
      .mockResolvedValue({ ...old, attempt: 2, state: 'downloading' });
    const start = vi.fn<ShellApi['startYoutubeImport']>();
    show({
      listYoutubeImports: () => Promise.resolve([other, old]),
      getYoutubeImport: () => Promise.resolve(old),
      updateYoutubeImport: update,
      startYoutubeImport: start,
    });
    const status = await screen.findByRole('region', { name: 'YouTube import status' });
    expect(within(status).getByText('Older selected video')).toBeTruthy();
    expect(within(status).queryByText('Newest unrelated video')).toBeNull();
    expect(start).not.toHaveBeenCalled();
    expect(update).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Retry download' }));
    await waitFor(() => expect(update).toHaveBeenCalledExactlyOnceWith('imp_old', 'retry'));
  });

  it('cancels through the daemon and displays only the confirmed terminal state', async () => {
    const cancellation = deferred<YoutubeImport>();
    const update = vi
      .fn<ShellApi['updateYoutubeImport']>()
      .mockImplementation(() => cancellation.promise);
    const first = record();
    rememberYoutube({
      importId: first.importId,
      projectId: first.projectId,
      videoId: first.videoId,
    });
    let current = first;
    show({
      listYoutubeImports: () => Promise.resolve([current]),
      getYoutubeImport: () => Promise.resolve(current),
      updateYoutubeImport: update,
    });
    fireEvent.click(await screen.findByRole('button', { name: 'Cancel import' }));
    expect(screen.queryByText('Import cancelled')).toBeNull();
    expect(screen.getByRole('button', { name: 'Cancel import' })).toHaveProperty('disabled', true);
    current = { ...first, state: 'cancelled', updatedUnixMillis: 3 };
    await act(async () => cancellation.resolve(current));
    await screen.findByText('Import cancelled');
    expect(update).toHaveBeenCalledExactlyOnceWith(first.importId, 'cancel');
  });

  it('drops a completed source read after another import is selected', async () => {
    const old = record({ state: 'completed', sourceId: 'src_old', title: 'Old video' });
    const next = record({
      importId: 'imp_next',
      projectId: 'prj_next',
      state: 'interrupted',
      title: 'Next video',
    });
    const detail = deferred<SourceDetails>();
    rememberYoutube({ importId: old.importId, projectId: old.projectId, videoId: old.videoId });
    const { onChosen } = show({
      listYoutubeImports: () => Promise.resolve([old, next]),
      getYoutubeImport: (id) => Promise.resolve(id === old.importId ? old : next),
      getSource: () => detail.promise,
    });
    await screen.findByText('Opening the imported video…');
    fireEvent.click(screen.getByRole('button', { name: /Next video/ }));
    onChosen.mockClear();
    await act(async () =>
      detail.resolve({
        source: source(old.projectId, { sourceId: old.sourceId }),
        sourceMapJson: '',
      }),
    );
    expect(onChosen).not.toHaveBeenCalled();
    expect(screen.getByText('Download interrupted')).toBeTruthy();
    expect(screen.queryByText('Opening the imported video…')).toBeNull();
  });

  it('does not clear an opened source when the slower initial list restores the same import', async () => {
    const complete = record({
      state: 'completed',
      sourceId: 'src_finished',
      title: 'Finished video',
    });
    const list = deferred<readonly YoutubeImport[]>();
    rememberYoutube({
      importId: complete.importId,
      projectId: complete.projectId,
      videoId: complete.videoId,
    });
    const { onChosen } = show({
      listYoutubeImports: () => list.promise,
      getYoutubeImport: () => Promise.resolve(complete),
      getSource: () =>
        Promise.resolve({
          source: source(complete.projectId, { sourceId: complete.sourceId }),
          sourceMapJson: '',
        }),
    });
    await waitFor(() =>
      expect(onChosen).toHaveBeenCalledWith(
        expect.objectContaining({ projectId: complete.projectId }),
      ),
    );
    onChosen.mockClear();
    await act(async () => list.resolve([complete]));
    expect(onChosen).not.toHaveBeenCalledWith(null);
  });

  it('clears the completed source when the input changes and restores it only when explicitly selected', async () => {
    const complete = record({
      state: 'completed',
      sourceId: 'src_finished',
      title: 'Finished video',
    });
    rememberYoutube({
      importId: complete.importId,
      projectId: complete.projectId,
      videoId: complete.videoId,
    });
    const { onChosen } = show({
      listYoutubeImports: () => Promise.resolve([complete]),
      getYoutubeImport: () => Promise.resolve(complete),
      getSource: () =>
        Promise.resolve({
          source: source(complete.projectId, { sourceId: complete.sourceId }),
          sourceMapJson: '',
        }),
    });
    await waitFor(() =>
      expect(onChosen).toHaveBeenCalledWith(
        expect.objectContaining({ projectId: complete.projectId }),
      ),
    );
    expect(screen.queryByText(/total size not yet known/)).toBeNull();
    expect(screen.queryByText(/Attempt 0/)).toBeNull();
    onChosen.mockClear();
    fireEvent.change(screen.getByLabelText('Video link'), {
      target: { value: 'https://youtu.be/abcdefghijk' },
    });
    expect(onChosen).toHaveBeenLastCalledWith(null);
    expect(screen.queryByRole('region', { name: 'YouTube import status' })).toBeNull();
    expect(onChosen).not.toHaveBeenCalledWith(
      expect.objectContaining({ source: expect.anything() }),
    );
    fireEvent.click(screen.getByRole('button', { name: /Finished video/ }));
    await waitFor(() =>
      expect(onChosen).toHaveBeenLastCalledWith(
        expect.objectContaining({ projectId: complete.projectId }),
      ),
    );
  });

  it.each(['URL', 'quality'])(
    'drops a completed source read when its %s changes before hydration finishes',
    async (changed) => {
      const complete = record({ state: 'completed', sourceId: 'src_finished' });
      const details = deferred<SourceDetails>();
      rememberYoutube({
        importId: complete.importId,
        projectId: complete.projectId,
        videoId: complete.videoId,
      });
      const { onChosen } = show({
        listYoutubeImports: () => Promise.resolve([complete]),
        getYoutubeImport: () => Promise.resolve(complete),
        getSource: () => details.promise,
      });
      await screen.findByText('Opening the imported video…');
      if (changed === 'URL') {
        fireEvent.change(screen.getByLabelText('Video link'), {
          target: { value: 'https://youtu.be/abcdefghijk' },
        });
      } else {
        fireEvent.keyDown(screen.getByLabelText('Import quality'), { key: 'Enter' });
        fireEvent.click(await screen.findByRole('option', { name: /360p/ }));
      }
      onChosen.mockClear();
      await act(async () =>
        details.resolve({
          source: source(complete.projectId, { sourceId: complete.sourceId }),
          sourceMapJson: '',
        }),
      );
      expect(onChosen).not.toHaveBeenCalled();
      expect(screen.queryByText('Opening the imported video…')).toBeNull();
    },
  );

  it('recovers an admitted import after its start reply was lost without starting another download', async () => {
    const admitted = record({ projectId: `prj_YouTube · ${ID}`, title: 'Already downloading' });
    let listed: readonly YoutubeImport[] = [];
    const start = vi.fn<ShellApi['startYoutubeImport']>().mockImplementation(() => {
      listed = [admitted];
      return Promise.reject('daemon did not answer within 10s');
    });
    show({
      listYoutubeImports: () => Promise.resolve(listed),
      startYoutubeImport: start,
      getYoutubeImport: () => Promise.resolve(admitted),
    });
    authorize();
    fireEvent.click(screen.getByRole('button', { name: 'Import video' }));
    await screen.findByText('daemon did not answer within 10s');
    fireEvent.click(screen.getByRole('button', { name: 'Refresh import status' }));
    await screen.findByText('Already downloading');
    expect(start).toHaveBeenCalledOnce();
  });

  it('shows measured byte progress while processing remains cancellable', async () => {
    const processing = record({
      state: 'processing',
      title: 'Public video',
      channel: 'Actual creator',
      downloadedBytes: 1024,
      totalBytes: 2048,
    });
    rememberYoutube({
      importId: processing.importId,
      projectId: processing.projectId,
      videoId: processing.videoId,
    });
    show({
      listYoutubeImports: () => Promise.resolve([processing]),
      getYoutubeImport: () => Promise.resolve(processing),
    });
    await screen.findByText('Preparing video');
    expect(screen.getByRole('progressbar').getAttribute('aria-valuenow')).toBe('50');
    expect(screen.getByText('Actual creator · 1080p max')).toBeTruthy();
    expect(screen.queryByText(/Attempt/)).toBeNull();
    expect(screen.getByRole('button', { name: 'Cancel import' })).toHaveProperty('disabled', false);
  });

  it('runs the existing analysis flow for the exact imported source only after the separate rights confirmation', async () => {
    const complete = record({
      state: 'completed',
      sourceId: 'src_imported',
      title: 'Creator title',
    });
    rememberYoutube({
      importId: complete.importId,
      projectId: complete.projectId,
      videoId: complete.videoId,
    });
    const base = fakeApi(emptyWorld());
    const analyze = vi.fn<ShellApi['submitAnalyze']>(base.submitAnalyze);
    const onStarted = vi.fn();
    const api: ShellApi = {
      ...base,
      listYoutubeImports: () => Promise.resolve([complete]),
      getYoutubeImport: () => Promise.resolve(complete),
      getSource: () =>
        Promise.resolve({
          source: source(complete.projectId, { sourceId: complete.sourceId }),
          sourceMapJson: sourceMapDocument('map').json,
        }),
      submitAnalyze: analyze,
    };
    render(
      <NewProject
        state={{
          status: 'connected',
          daemonVersion: '0.1',
          localLock: false,
          startedUnixMillis: 0,
        }}
        loader={new ImportLoader(api)}
        onStarted={onStarted}
      />,
    );
    await screen.findByText(/1:42:07 · 1920×1080 · 29.97 fps/);
    const start = screen.getByRole('button', { name: /Analyze video/ });
    expect(start).toHaveProperty('disabled', true);
    expect(analyze).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('checkbox', { name: /rights to clip and publish/ }));
    await waitFor(() => expect(start).toHaveProperty('disabled', false));
    fireEvent.change(screen.getByLabelText('Video link'), {
      target: { value: 'https://youtu.be/abcdefghijk' },
    });
    expect(start).toHaveProperty('disabled', true);
    fireEvent.click(screen.getByRole('button', { name: /Creator title/ }));
    await waitFor(() => expect(start).toHaveProperty('disabled', false));
    fireEvent.click(screen.getByLabelText(/Extended/));
    fireEvent.click(start);
    await waitFor(() => expect(analyze).toHaveBeenCalledOnce());
    expect(analyze).toHaveBeenCalledWith(
      complete.projectId,
      expect.objectContaining({
        sourceId: complete.sourceId,
        localEditorial: true,
        minTicks: 60 * 90_000,
        maxTicks: 180 * 90_000,
      }),
    );
    await waitFor(() =>
      expect(onStarted).toHaveBeenCalledWith(complete.projectId, expect.any(String)),
    );
  });

  it('creates a distinct project when the same completed video is requested at 360p', async () => {
    const complete = record({
      state: 'completed',
      sourceId: 'src_hd',
      title: 'HD copy',
      maxHeight: 1080,
    });
    const smaller = record({ importId: 'imp_360', projectId: 'prj_360', maxHeight: 360 });
    rememberYoutube({
      importId: complete.importId,
      projectId: complete.projectId,
      videoId: complete.videoId,
      maxHeight: 1080,
    });
    const create = vi.fn<ShellApi['createProject']>().mockResolvedValue('prj_360');
    const start = vi.fn<ShellApi['startYoutubeImport']>().mockResolvedValue(smaller);
    const { onChosen } = show({
      listYoutubeImports: () => Promise.resolve([complete]),
      getYoutubeImport: (id) => Promise.resolve(id === complete.importId ? complete : smaller),
      getSource: () =>
        Promise.resolve({
          source: source(complete.projectId, { sourceId: complete.sourceId }),
          sourceMapJson: '',
        }),
      createProject: create,
      startYoutubeImport: start,
    });
    await waitFor(() =>
      expect(onChosen).toHaveBeenCalledWith(
        expect.objectContaining({ projectId: complete.projectId }),
      ),
    );
    onChosen.mockClear();
    fireEvent.keyDown(screen.getByLabelText('Import quality'), { key: 'Enter' });
    fireEvent.click(await screen.findByRole('option', { name: /360p/ }));
    expect(onChosen).toHaveBeenLastCalledWith(null);
    expect(screen.queryByRole('region', { name: 'YouTube import status' })).toBeNull();
    expect(screen.getByText(/Changing quality creates a separate project/)).toBeTruthy();
    fireEvent.click(screen.getByRole('checkbox', { name: /creator’s permission/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Import video' }));
    await waitFor(() =>
      expect(start).toHaveBeenCalledExactlyOnceWith('prj_360', complete.canonicalUrl, true, 360),
    );
    expect(create).toHaveBeenCalledOnce();
    await waitFor(() =>
      expect(recallYoutube()).toMatchObject({
        importId: 'imp_360',
        projectId: 'prj_360',
        maxHeight: 360,
      }),
    );
    expect(screen.getByLabelText('Import quality')).toHaveProperty('disabled', true);
    expect(onChosen).not.toHaveBeenCalledWith(
      expect.objectContaining({ projectId: complete.projectId }),
    );
  });

  it('restores the saved 360p copy after relaunch without changing its quality or starting a download', async () => {
    const complete = record({ state: 'completed', sourceId: 'src_360', maxHeight: 360 });
    rememberYoutube({
      importId: complete.importId,
      projectId: complete.projectId,
      videoId: complete.videoId,
      maxHeight: 360,
    });
    const start = vi.fn<ShellApi['startYoutubeImport']>();
    const { onChosen } = show({
      listYoutubeImports: () => Promise.resolve([complete]),
      getYoutubeImport: () => Promise.resolve(complete),
      getSource: () =>
        Promise.resolve({
          source: source(complete.projectId, { sourceId: complete.sourceId }),
          sourceMapJson: '',
        }),
      startYoutubeImport: start,
    });
    await waitFor(() =>
      expect(onChosen).toHaveBeenCalledWith(
        expect.objectContaining({ source: expect.objectContaining({ sourceId: 'src_360' }) }),
      ),
    );
    expect(screen.getByLabelText('Import quality').textContent).toContain('360p');
    expect(screen.getByText('360p max')).toBeTruthy();
    expect(start).not.toHaveBeenCalled();
  });
});
