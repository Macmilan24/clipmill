import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type {
  PublishingApi,
  YoutubeConnection,
  YoutubeMetadataDraft,
  YoutubePublishingStatus,
  YoutubeUpload,
} from '../src/daemon/publishing.js';
import { ConnectionCard } from '../src/youtube/ConnectionCard.js';
import { UploadPanel } from '../src/youtube/UploadPanel.js';
import { UploadHistory } from '../src/youtube/UploadHistory.js';
import { emptyWorld, fakeApi } from './support/library.js';

const channel: YoutubeConnection = {
  connectionId: 'con_1',
  channelId: 'UC_creator',
  title: 'Creator channel',
  state: 'connected',
  error: '',
  createdUnixMillis: 1,
  updatedUnixMillis: 1,
};
const status: YoutubePublishingStatus = {
  available: true,
  configured: true,
  connections: [channel],
};
const metadata = {
  title: 'A useful moment',
  description: 'What the clip says.',
  tags: ['interview'],
  madeForKids: false,
  containsSyntheticMedia: false,
};
const draft: YoutubeMetadataDraft = {
  metadata,
  renderArtifactId: 'render_1',
  revision: 7,
  transcriptExcerpt: 'The words spoken in this exact clip.',
};
const upload: YoutubeUpload = {
  uploadId: 'upl_1',
  projectId: 'prj_1',
  docId: 'doc_1',
  revision: 7,
  exportJobId: 'exp_1',
  irArtifactId: 'ir_1',
  renderArtifactId: 'render_1',
  connectionId: channel.connectionId,
  channelId: channel.channelId,
  channelTitle: channel.title,
  metadata,
  state: 'private',
  acknowledgedBytes: 1024,
  totalBytes: 1024,
  videoId: 'abcdefghijk',
  visibility: 'private',
  errorCode: '',
  error: '',
  createdUnixMillis: 1,
  updatedUnixMillis: 2,
};
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
function api(overrides: Partial<PublishingApi> = {}): PublishingApi {
  return {
    ...fakeApi(emptyWorld()),
    fetchYoutubePublishingStatus: () => Promise.resolve(status),
    draftYoutubeMetadata: () => Promise.resolve(draft),
    ...overrides,
  };
}
const props = {
  projectId: 'prj_1',
  docId: 'doc_1',
  exportJobId: 'exp_1',
  revision: 7,
  renderArtifactId: 'render_1',
  currentRevision: 7,
  delivered: true,
};
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
  vi.unstubAllGlobals();
  if (scrollDescriptor)
    Object.defineProperty(HTMLElement.prototype, 'scrollIntoView', scrollDescriptor);
  else Reflect.deleteProperty(HTMLElement.prototype, 'scrollIntoView');
});
async function choose(label: string, option: string) {
  fireEvent.keyDown(screen.getByLabelText(label), { key: 'Enter' });
  fireEvent.click(await screen.findByRole('option', { name: option }));
}
async function approve() {
  await choose('Audience', 'Not made for kids');
  await choose('Realistic altered or synthetic content', 'No');
  fireEvent.click(screen.getByRole('checkbox', { name: /I reviewed rendered r7/ }));
}

describe('channel setup', () => {
  it('keeps setup and sign-in explicit, and shows the actual channel only after confirmation', async () => {
    let current: YoutubePublishingStatus = { available: true, configured: false, connections: [] };
    const config = vi.fn<PublishingApi['chooseYoutubeClientConfig']>().mockImplementation(() => {
      current = { ...current, configured: true };
      return Promise.resolve(current);
    });
    const connect = vi.fn<PublishingApi['connectYoutubeChannel']>().mockImplementation(() => {
      current = {
        ...current,
        connections: [{ ...channel, title: '', channelId: '', state: 'connecting' }],
      };
      return Promise.resolve(channel.connectionId);
    });
    const cancel = vi.fn<PublishingApi['updateYoutubeConnection']>().mockImplementation(() => {
      current = { ...current, connections: [] };
      return Promise.resolve(current);
    });
    render(
      <ConnectionCard
        api={api({
          fetchYoutubePublishingStatus: () => Promise.resolve(current),
          chooseYoutubeClientConfig: config,
          connectYoutubeChannel: connect,
          updateYoutubeConnection: cancel,
        })}
      />,
    );
    await screen.findByText('Set up your Google desktop client');
    expect(config).not.toHaveBeenCalled();
    expect(connect).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Connect channel' })).toHaveProperty(
      'disabled',
      true,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Choose client JSON' }));
    await screen.findByText('Desktop client configured');
    fireEvent.click(screen.getByRole('button', { name: 'Connect channel' }));
    await screen.findByText('Complete sign-in in your browser');
    expect(screen.queryByText(channel.title)).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel sign-in' }));
    await waitFor(() =>
      expect(cancel).toHaveBeenCalledExactlyOnceWith(channel.connectionId, 'cancel'),
    );
    expect(connect).toHaveBeenCalledOnce();
  });

  it('keeps unsupported credential storage honest and offers no connection action', async () => {
    render(
      <ConnectionCard
        api={api({
          fetchYoutubePublishingStatus: () => Promise.resolve({ ...status, available: false }),
        })}
      />,
    );
    await screen.findByText(/Secure channel credentials are unavailable/);
    expect(screen.queryByRole('button', { name: /Connect channel/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /client JSON/ })).toBeNull();
  });
});

describe('private uploads and separate publishing', () => {
  it('requires a completed export and never starts remote work merely by opening the panel', async () => {
    const start = vi.fn<PublishingApi['startYoutubeUpload']>();
    const suggest = vi.fn<PublishingApi['draftYoutubeMetadata']>();
    render(
      <UploadPanel
        {...props}
        delivered={false}
        api={api({ startYoutubeUpload: start, draftYoutubeMetadata: suggest })}
      />,
    );
    await screen.findByText(/Export this clip first/);
    expect(start).not.toHaveBeenCalled();
    expect(suggest).not.toHaveBeenCalled();
    expect(screen.queryByRole('button', { name: /Upload r7 privately/ })).toBeNull();
  });

  it('binds upload to the approved render, preserves edits against a late draft, and blocks double submission', async () => {
    const suggested = deferred<YoutubeMetadataDraft>();
    const started = deferred<YoutubeUpload>();
    const start = vi
      .fn<PublishingApi['startYoutubeUpload']>()
      .mockImplementation(() => started.promise);
    render(
      <UploadPanel
        {...props}
        api={api({ startYoutubeUpload: start, draftYoutubeMetadata: () => suggested.promise })}
      />,
    );
    const title = await screen.findByLabelText('Video title');
    fireEvent.change(title, { target: { value: 'My reviewed title' } });
    fireEvent.change(screen.getByLabelText('Description & hashtags'), {
      target: { value: 'A checked description. #interview' },
    });
    await act(async () => suggested.resolve(draft));
    expect(title).toHaveProperty('value', 'My reviewed title');
    const button = screen.getByRole('button', { name: 'Upload r7 privately' });
    expect(button).toHaveProperty('disabled', true);
    await approve();
    fireEvent.click(button);
    fireEvent.click(button);
    expect(start).toHaveBeenCalledExactlyOnceWith({
      exportJobId: 'exp_1',
      connectionId: 'con_1',
      expectedRevision: 7,
      rightsConfirmed: true,
      metadata: {
        title: 'My reviewed title',
        description: 'A checked description. #interview',
        tags: [],
        madeForKids: false,
        containsSyntheticMedia: false,
      },
    });
    await act(async () => started.resolve({ ...upload, state: 'uploading' }));
  });

  it('refuses a metadata draft for a different render instead of treating it as reviewed', async () => {
    render(
      <UploadPanel
        {...props}
        api={api({
          draftYoutubeMetadata: () =>
            Promise.resolve({ ...draft, renderArtifactId: 'other_render' }),
        })}
      />,
    );
    await screen.findByText(/metadata draft belongs to a different rendered revision/);
    expect(screen.getByRole('button', { name: 'Upload r7 privately' })).toHaveProperty(
      'disabled',
      true,
    );
  });

  it('blocks a non-ASCII description that exceeds the transport byte limit', async () => {
    render(<UploadPanel {...props} api={api()} />);
    const description = await screen.findByLabelText('Description & hashtags');
    fireEvent.change(description, { target: { value: 'é'.repeat(2600) } });
    await approve();
    expect(screen.getByText(/Keep the description within 5,000 UTF-8 bytes/)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Upload r7 privately' })).toHaveProperty(
      'disabled',
      true,
    );
  });

  it('labels a remotely unlisted video accurately instead of claiming it is private', async () => {
    render(
      <UploadPanel
        {...props}
        api={api({
          listYoutubeUploads: () => Promise.resolve([{ ...upload, visibility: 'unlisted' }]),
        })}
      />,
    );
    await screen.findByText('Unlisted on YouTube');
    expect(screen.queryByText('Private on YouTube')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Publish…' }));
    expect(screen.getByRole('button', { name: 'Keep unlisted' })).toBeTruthy();
  });

  it('recovers a private upload and publishes only after a distinct explicit confirmation', async () => {
    let saved = upload;
    const publish = vi.fn<PublishingApi['publishYoutubeUpload']>().mockImplementation(() => {
      saved = { ...upload, state: 'public', visibility: 'public', updatedUnixMillis: 3 };
      return Promise.resolve(saved);
    });
    const start = vi.fn<PublishingApi['startYoutubeUpload']>();
    render(
      <UploadPanel
        {...props}
        api={api({
          listYoutubeUploads: () => Promise.resolve([saved]),
          publishYoutubeUpload: publish,
          startYoutubeUpload: start,
        })}
      />,
    );
    await screen.findByText('Private on YouTube');
    expect(start).not.toHaveBeenCalled();
    expect(publish).not.toHaveBeenCalled();
    expect(screen.queryByRole('button', { name: 'Upload r7 privately' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Publish…' }));
    expect(publish).not.toHaveBeenCalled();
    expect(screen.getByText(/Make “A useful moment” public on Creator channel/)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Publish publicly' }));
    await screen.findByText('Public on YouTube');
    expect(publish).toHaveBeenCalledExactlyOnceWith('upl_1');
  });

  it.each([
    { exportJobId: 'exp_again', renderArtifactId: 'render_1', irArtifactId: undefined },
    { exportJobId: 'exp_again', renderArtifactId: 'render_regenerated', irArtifactId: 'ir_1' },
  ])('recovers the same immutable revision across another export job: %j', async (identity) => {
    const start = vi.fn<PublishingApi['startYoutubeUpload']>();
    render(
      <UploadPanel
        {...props}
        exportJobId={identity.exportJobId}
        renderArtifactId={identity.renderArtifactId}
        {...(identity.irArtifactId ? { irArtifactId: identity.irArtifactId } : {})}
        api={api({
          listYoutubeUploads: () => Promise.resolve([upload]),
          startYoutubeUpload: start,
        })}
      />,
    );
    await screen.findByText(
      'This rendered revision already has an upload on this channel. Continue with its saved status below.',
    );
    expect(screen.getByText('Private on YouTube')).toBeTruthy();
    expect(screen.queryByLabelText('Video title')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Upload r7 privately' })).toBeNull();
    expect(start).not.toHaveBeenCalled();
  });

  it('keeps channel choice available when this revision already has an upload on another channel', async () => {
    const second = {
      ...channel,
      connectionId: 'con_2',
      channelId: 'UC_second',
      title: 'Second channel',
    };
    let records = [upload];
    const start = vi.fn<PublishingApi['startYoutubeUpload']>().mockImplementation((request) => {
      const next = {
        ...upload,
        uploadId: 'upl_2',
        connectionId: second.connectionId,
        channelId: second.channelId,
        channelTitle: second.title,
        metadata: request.metadata,
        state: 'uploading' as const,
      };
      records = [...records, next];
      return Promise.resolve(next);
    });
    render(
      <UploadPanel
        {...props}
        api={api({
          fetchYoutubePublishingStatus: () =>
            Promise.resolve({ ...status, connections: [channel, second] }),
          listYoutubeUploads: () => Promise.resolve(records),
          startYoutubeUpload: start,
        })}
      />,
    );
    await screen.findByLabelText('Destination channel');
    await choose('Destination channel', 'Creator channel');
    expect(screen.queryByLabelText('Video title')).toBeNull();
    await choose('Destination channel', 'Second channel');
    await screen.findByLabelText('Video title');
    await approve();
    fireEvent.click(screen.getByRole('button', { name: 'Upload r7 privately' }));
    await waitFor(() =>
      expect(start).toHaveBeenCalledWith(
        expect.objectContaining({
          exportJobId: 'exp_1',
          connectionId: 'con_2',
          expectedRevision: 7,
        }),
      ),
    );
    expect(start).toHaveBeenCalledOnce();
  });

  it('reconciles uncertain completion without offering a fresh upload or blind resume', async () => {
    const uncertain = {
      ...upload,
      state: 'completion_uncertain' as const,
      videoId: '',
      visibility: '',
      errorCode: 'session_expired',
    };
    const update = vi
      .fn<PublishingApi['updateYoutubeUpload']>()
      .mockResolvedValue({ ...uncertain, state: 'reconciling', updatedUnixMillis: 3 });
    render(
      <UploadPanel
        {...props}
        api={api({
          listYoutubeUploads: () => Promise.resolve([uncertain]),
          updateYoutubeUpload: update,
        })}
      />,
    );
    await screen.findByText('Completion needs checking');
    expect(screen.queryByRole('button', { name: /Resume/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /Restart/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /Upload r7 privately/ })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Check existing upload' }));
    await waitFor(() => expect(update).toHaveBeenCalledExactlyOnceWith('upl_1', 'reconcile'));
  });

  it('explicitly restarts only a safely incomplete expired session under its saved operation', async () => {
    const expired = {
      ...upload,
      state: 'failed' as const,
      videoId: '',
      visibility: '',
      errorCode: 'session_expired',
      acknowledgedBytes: 200,
    };
    const update = vi
      .fn<PublishingApi['updateYoutubeUpload']>()
      .mockResolvedValue({ ...expired, state: 'reconciling', updatedUnixMillis: 3 });
    render(
      <UploadPanel
        {...props}
        api={api({
          listYoutubeUploads: () => Promise.resolve([expired]),
          updateYoutubeUpload: update,
        })}
      />,
    );
    const restart = await screen.findByRole('button', { name: 'Restart incomplete upload' });
    expect(screen.getByText(/before the complete video was sent/)).toBeTruthy();
    expect(update).not.toHaveBeenCalled();
    expect(screen.queryByRole('button', { name: /Upload r7 privately/ })).toBeNull();
    fireEvent.click(restart);
    await waitFor(() => expect(update).toHaveBeenCalledExactlyOnceWith('upl_1', 'resume'));
  });

  it('resumes the saved older revision after relaunch without substituting a newer render', async () => {
    const older = { ...upload, exportJobId: 'exp_old', revision: 2, state: 'paused' as const };
    const update = vi
      .fn<PublishingApi['updateYoutubeUpload']>()
      .mockResolvedValue({ ...older, state: 'uploading', updatedUnixMillis: 3 });
    render(
      <UploadPanel
        {...props}
        api={api({
          listYoutubeUploads: () => Promise.resolve([older]),
          updateYoutubeUpload: update,
        })}
      />,
    );
    await screen.findByRole('region', { name: 'YouTube upload r2' });
    expect(update).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Resume existing upload' }));
    await waitFor(() => expect(update).toHaveBeenCalledExactlyOnceWith('upl_1', 'resume'));
  });

  it('drops an old project’s delayed upload list after changing clips', async () => {
    const old = deferred<readonly YoutubeUpload[]>();
    const client = api({
      listYoutubeUploads: (projectId) =>
        projectId === 'prj_1' ? old.promise : Promise.resolve([]),
    });
    const view = render(<UploadPanel {...props} api={client} />);
    view.rerender(
      <UploadPanel
        {...props}
        api={client}
        projectId="prj_2"
        docId="doc_2"
        exportJobId={null}
        delivered={false}
      />,
    );
    await act(async () => old.resolve([upload]));
    expect(screen.queryByRole('region', { name: 'YouTube upload r7' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Publish…' })).toBeNull();
  });
});

describe('upload receipts after project removal', () => {
  it('lists saved receipts globally and opens only the selected verified remote video', async () => {
    const list = vi.fn<PublishingApi['listYoutubeUploads']>().mockResolvedValue([upload]);
    const open = vi.fn<PublishingApi['openYoutubePage']>().mockResolvedValue();
    const start = vi.fn<PublishingApi['startYoutubeUpload']>();
    const publish = vi.fn<PublishingApi['publishYoutubeUpload']>();
    render(
      <UploadHistory
        api={{
          ...fakeApi(emptyWorld()),
          ...api({
            listYoutubeUploads: list,
            openYoutubePage: open,
            startYoutubeUpload: start,
            publishYoutubeUpload: publish,
          }),
        }}
      />,
    );
    await screen.findByText('Source project removed · upload receipt retained');
    expect(list).toHaveBeenCalledWith(undefined);
    expect(screen.getByText('Private on YouTube')).toBeTruthy();
    expect(start).not.toHaveBeenCalled();
    expect(publish).not.toHaveBeenCalled();
    expect(open).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Review in YouTube Studio' }));
    await waitFor(() => expect(open).toHaveBeenCalledExactlyOnceWith('studio', upload.uploadId));
  });

  it('keeps older uncertain receipts reachable beyond the first ten without starting a replacement', async () => {
    const records = Array.from({ length: 11 }, (_unused, index) => ({
      ...upload,
      uploadId: `upl_${index}`,
      updatedUnixMillis: index + 1,
      metadata: { ...metadata, title: `Saved clip ${index}` },
      ...(index === 0 ? { state: 'completion_uncertain' as const, videoId: '' } : {}),
    }));
    const update = vi
      .fn<PublishingApi['updateYoutubeUpload']>()
      .mockResolvedValue({ ...records[0]!, state: 'reconciling', updatedUnixMillis: 12 });
    render(
      <UploadHistory
        api={{
          ...fakeApi(emptyWorld()),
          ...api({
            listYoutubeUploads: () => Promise.resolve(records),
            updateYoutubeUpload: update,
          }),
        }}
      />,
    );
    await screen.findByRole('button', { name: 'Show all 11 uploads' });
    expect(screen.queryByText('Saved clip 0')).toBeNull();
    expect(update).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Show all 11 uploads' }));
    expect(screen.getByText('Saved clip 0')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Check existing upload' }));
    await waitFor(() => expect(update).toHaveBeenCalledExactlyOnceWith('upl_0', 'reconcile'));
  });

  it('shows a failed history action and keeps its receipt available for recovery', async () => {
    const paused = { ...upload, state: 'paused' as const };
    render(
      <UploadHistory
        api={{
          ...fakeApi(emptyWorld()),
          ...api({
            listYoutubeUploads: () => Promise.resolve([paused]),
            updateYoutubeUpload: () =>
              Promise.reject('Reconnect the original channel before resuming.'),
          }),
        }}
      />,
    );
    fireEvent.click(await screen.findByRole('button', { name: 'Resume existing upload' }));
    await screen.findByRole('alert');
    expect(screen.getByText('Reconnect the original channel before resuming.')).toBeTruthy();
    expect(screen.getByRole('region', { name: 'YouTube upload r7' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Refresh uploads' })).toHaveProperty(
      'disabled',
      false,
    );
  });
});

const generated = {
  ...metadata,
  title: 'Why keeping your files local matters',
  description: 'A grounded description of this clip. #LocalFiles',
  tags: ['local files', 'workflow'],
  madeForKids: true,
  containsSyntheticMedia: true,
};
const finishedWriting: YoutubeMetadataDraft = {
  ...draft,
  generationJobId: 'writing_1',
  generationState: 'succeeded',
  modelName: 'Qwen 3.5',
  generatedMetadata: generated,
};

describe('optional local metadata writing', () => {
  afterEach(() => vi.useRealTimers());

  it('starts only on request, preserves edits while writing, and applies only text with fresh approval', async () => {
    const response = deferred<YoutubeMetadataDraft>();
    const write = vi
      .fn<PublishingApi['draftYoutubeMetadata']>()
      .mockImplementation((_job, _revision, action) =>
        action === 'start' ? response.promise : Promise.resolve(draft),
      );
    const startUpload = vi.fn<PublishingApi['startYoutubeUpload']>();
    const publish = vi.fn<PublishingApi['publishYoutubeUpload']>();
    render(
      <UploadPanel
        {...props}
        api={api({
          draftYoutubeMetadata: write,
          startYoutubeUpload: startUpload,
          publishYoutubeUpload: publish,
        })}
      />,
    );
    const title = await screen.findByLabelText('Video title');
    await screen.findByRole('button', { name: 'Write with Qwen' });
    expect(write).toHaveBeenCalledExactlyOnceWith('exp_1', 7);
    await approve();
    const confirmation = screen.getByRole('checkbox', { name: /I reviewed rendered r7/ });
    expect(confirmation.getAttribute('aria-checked')).toBe('true');
    const button = screen.getByRole('button', { name: 'Write with Qwen' });
    fireEvent.click(button);
    fireEvent.click(button);
    expect(write.mock.calls.filter((call) => call[2] === 'start')).toHaveLength(1);
    fireEvent.change(title, { target: { value: 'My title while Qwen writes' } });
    await act(async () => response.resolve(finishedWriting));
    await screen.findByText(generated.title);
    expect(title).toHaveProperty('value', 'My title while Qwen writes');
    fireEvent.click(confirmation);
    expect(confirmation.getAttribute('aria-checked')).toBe('true');
    fireEvent.click(screen.getByRole('button', { name: 'Apply suggestion' }));
    expect(title).toHaveProperty('value', generated.title);
    expect(screen.getByLabelText('Description & hashtags')).toHaveProperty(
      'value',
      generated.description,
    );
    expect(screen.getByLabelText('Tags')).toHaveProperty('value', 'local files, workflow');
    expect(screen.getByLabelText('Audience').textContent).toBe('Not made for kids');
    expect(screen.getByLabelText('Realistic altered or synthetic content').textContent).toBe('No');
    expect(confirmation.getAttribute('aria-checked')).toBe('false');
    expect(startUpload).not.toHaveBeenCalled();
    expect(publish).not.toHaveBeenCalled();
  });

  it('restores a saved suggestion without running Qwen or applying it on open', async () => {
    const write = vi.fn<PublishingApi['draftYoutubeMetadata']>().mockResolvedValue(finishedWriting);
    render(<UploadPanel {...props} api={api({ draftYoutubeMetadata: write })} />);
    await screen.findByText(generated.title);
    expect(screen.getByLabelText('Video title')).toHaveProperty('value', metadata.title);
    expect(write).toHaveBeenCalledExactlyOnceWith('exp_1', 7);
  });

  it('cancels durably, ignores an older poll, and retries the identified failed job explicitly', async () => {
    const oldStatus = deferred<YoutubeMetadataDraft>();
    const write = vi
      .fn<PublishingApi['draftYoutubeMetadata']>()
      .mockImplementation((_job, _revision, action, jobId) => {
        if (action === 'start')
          return Promise.resolve({
            ...draft,
            generationJobId: 'writing_1',
            generationState: 'queued',
          });
        if (action === 'status' && jobId === 'writing_1') return oldStatus.promise;
        if (action === 'cancel')
          return Promise.resolve({
            ...draft,
            generationJobId: 'writing_1',
            generationState: 'cancelled',
          });
        if (action === 'retry')
          return Promise.resolve({
            ...draft,
            generationJobId: 'writing_2',
            generationState: 'running',
          });
        if (action === 'status' && jobId === 'writing_2')
          return Promise.resolve({
            ...draft,
            generationJobId: 'writing_2',
            generationState: 'failed',
            generationMessage: 'The local writer stopped. Your details are still editable.',
          });
        return Promise.resolve(draft);
      });
    render(<UploadPanel {...props} api={api({ draftYoutubeMetadata: write })} />);
    const writeButton = await screen.findByRole('button', { name: 'Write with Qwen' });
    vi.useFakeTimers();
    await act(async () => fireEvent.click(writeButton));
    expect(screen.getByText('Waiting for the local model')).toBeTruthy();
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    expect(write).toHaveBeenCalledWith('exp_1', 7, 'status', 'writing_1');
    await act(async () => fireEvent.click(screen.getByRole('button', { name: 'Cancel writing' })));
    expect(write).toHaveBeenCalledWith('exp_1', 7, 'cancel', 'writing_1');
    await act(async () => oldStatus.resolve(finishedWriting));
    expect(screen.queryByText(generated.title)).toBeNull();
    expect(screen.getByText('Writing cancelled. Your details are unchanged.')).toBeTruthy();
    await act(async () => fireEvent.click(screen.getByRole('button', { name: 'Retry writing' })));
    expect(write).toHaveBeenCalledWith('exp_1', 7, 'retry', 'writing_1');
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    expect(
      screen.getByText('The local writer stopped. Your details are still editable.'),
    ).toBeTruthy();
    fireEvent.change(screen.getByLabelText('Video title'), {
      target: { value: 'A manually written title' },
    });
    expect(screen.getByLabelText('Video title')).toHaveProperty(
      'value',
      'A manually written title',
    );
  });

  it('drops old export responses even when the same export job is shown at another revision', async () => {
    const late = deferred<YoutubeMetadataDraft>();
    const write = vi
      .fn<PublishingApi['draftYoutubeMetadata']>()
      .mockImplementation((_job, revision, action) =>
        action === 'start'
          ? late.promise
          : Promise.resolve({
              ...draft,
              revision,
              renderArtifactId: revision === 7 ? 'render_1' : 'render_2',
              metadata: { ...metadata, title: `Revision ${revision}` },
            }),
      );
    const currentApi = api({ draftYoutubeMetadata: write });
    const view = render(<UploadPanel {...props} api={currentApi} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Write with Qwen' }));
    view.rerender(
      <UploadPanel
        {...props}
        revision={8}
        currentRevision={8}
        renderArtifactId="render_2"
        api={currentApi}
      />,
    );
    await waitFor(() =>
      expect(screen.getByLabelText('Video title')).toHaveProperty('value', 'Revision 8'),
    );
    await act(async () => late.resolve(finishedWriting));
    expect(screen.queryByText(generated.title)).toBeNull();
    expect(screen.getByLabelText('Video title')).toHaveProperty('value', 'Revision 8');
  });

  it('keeps manual upload available when the local model is unavailable', async () => {
    const write = vi
      .fn<PublishingApi['draftYoutubeMetadata']>()
      .mockImplementation((_job, _revision, action) =>
        Promise.resolve(
          action
            ? {
                ...draft,
                generationState: 'unavailable',
                generationMessage:
                  'Install Qwen from Models, or keep writing these details yourself.',
              }
            : draft,
        ),
      );
    render(<UploadPanel {...props} api={api({ draftYoutubeMetadata: write })} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Write with Qwen' }));
    await screen.findByText('Install Qwen from Models, or keep writing these details yourself.');
    await approve();
    expect(screen.getByRole('button', { name: 'Upload r7 privately' })).toHaveProperty(
      'disabled',
      false,
    );
    expect(write.mock.calls.filter((call) => call[2] === 'start')).toHaveLength(1);
  });
});

it('offers manual editing instead of restarting an unavailable saved suggestion', async () => {
  const write = vi.fn<PublishingApi['draftYoutubeMetadata']>().mockResolvedValue({
    ...draft,
    generationState: 'unavailable',
    generationJobId: 'saved-writing',
    generationMessage:
      'This saved suggestion could not be verified. Keep editing these details yourself.',
    modelName: 'qwen3-5-editorial-mlx',
  });
  render(<UploadPanel {...props} api={api({ draftYoutubeMetadata: write })} />);
  await screen.findByText(/This saved suggestion could not be verified/);
  expect(screen.queryByRole('button', { name: 'Write with Qwen' })).toBeNull();
  expect(screen.queryByRole('button', { name: 'Retry writing' })).toBeNull();
  expect(screen.getByText('Qwen 3.5 · On this device · Optional')).toBeTruthy();
  expect(screen.queryByText(/qwen3-5-editorial-mlx/)).toBeNull();
  await approve();
  expect(screen.getByRole('button', { name: 'Upload r7 privately' })).toHaveProperty(
    'disabled',
    false,
  );
  expect(write).toHaveBeenCalledExactlyOnceWith('exp_1', 7);
});
