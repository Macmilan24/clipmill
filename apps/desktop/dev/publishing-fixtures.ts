/** Synthetic display-only data. This module never opens a browser or contacts Google. */
import type {
  PublishingApi,
  YoutubePublishingStatus,
  YoutubeUpload,
} from '../src/daemon/publishing.js';

const unavailable = () =>
  Promise.reject(
    new Error('Synthetic preview only. No browser opens and no video leaves this page.'),
  );

export function publishingFixture(connected: boolean, history = false): PublishingApi {
  const connection = {
    connectionId: 'preview-connection',
    channelId: 'UC_preview_creative_studio',
    title: 'The Creative Studio',
    state: 'connected' as const,
    error: '',
    createdUnixMillis: 1,
    updatedUnixMillis: 1,
  };
  let status: YoutubePublishingStatus = {
    available: true,
    configured: connected,
    connections: connected ? [connection] : [],
  };
  let uploads: YoutubeUpload[] = [];
  let writing = false;
  const metadata = {
    title: 'The question that changed how I create',
    description:
      'A conversation about curiosity, creative habits, and asking a more useful question.\n\n#Creativity #Interview',
    tags: ['creativity', 'interview', 'creative process'],
    madeForKids: false,
    containsSyntheticMedia: false,
  };
  if (history) {
    const saved: YoutubeUpload = {
      uploadId: 'preview-saved-private',
      projectId: 'preview',
      docId: 'preview-saved-edit',
      revision: 7,
      exportJobId: 'preview-saved-export',
      irArtifactId: 'preview-saved-ir',
      renderArtifactId: 'preview-saved-render',
      connectionId: connection.connectionId,
      channelId: connection.channelId,
      channelTitle: connection.title,
      metadata,
      state: 'private',
      acknowledgedBytes: 42_000_000,
      totalBytes: 42_000_000,
      videoId: 'preview0000',
      visibility: 'private',
      errorCode: '',
      error: '',
      createdUnixMillis: 1,
      updatedUnixMillis: 4,
    };
    uploads = [
      saved,
      {
        ...saved,
        uploadId: 'preview-saved-uncertain',
        projectId: 'preview-removed-project',
        metadata: { ...metadata, title: 'Leave room for a different answer' },
        state: 'completion_uncertain',
        videoId: '',
        visibility: '',
        errorCode: 'completion_uncertain',
        error: 'The final response was interrupted. Check the saved upload before retrying.',
        updatedUnixMillis: 3,
      },
      {
        ...saved,
        uploadId: 'preview-saved-unlisted',
        metadata: { ...metadata, title: 'Why slowing down helps' },
        visibility: 'unlisted',
        updatedUnixMillis: 2,
      },
      {
        ...saved,
        uploadId: 'preview-saved-expired',
        metadata: { ...metadata, title: 'A useful creative habit' },
        state: 'failed',
        acknowledgedBytes: 8_000_000,
        videoId: '',
        visibility: '',
        errorCode: 'session_expired',
        error: 'The unfinished upload session expired.',
        updatedUnixMillis: 1,
      },
    ];
  }
  return {
    fetchYoutubePublishingStatus: () => Promise.resolve(status),
    chooseYoutubeClientConfig: () => {
      status = { ...status, configured: true };
      return Promise.resolve(status);
    },
    connectYoutubeChannel: () => {
      status = { ...status, connections: [connection] };
      return Promise.resolve(connection.connectionId);
    },
    updateYoutubeConnection: () => {
      status = { ...status, connections: [] };
      return Promise.resolve(status);
    },
    draftYoutubeMetadata: (_job, revision, action) => {
      if (action === 'start' || action === 'retry') writing = true;
      if (action === 'cancel') writing = false;
      return Promise.resolve({
        metadata,
        revision,
        renderArtifactId: 'preview-render',
        generationJobId: writing ? 'preview-writing' : '',
        generationState: writing ? 'succeeded' : 'idle',
        generationMessage: '',
        modelName: 'Qwen 3.5',
        generatedMetadata: writing
          ? { ...metadata, title: 'The question that helps you finish creative work' }
          : null,
        transcriptExcerpt:
          'I used to ask, “Is this good enough?” But that question stopped me from making anything. Now I ask, “What can I learn from finishing this?” That small change gave me permission to keep creating.',
      });
    },
    startYoutubeUpload: (request) => {
      const record: YoutubeUpload = {
        uploadId: 'preview-upload',
        projectId: 'preview',
        docId: 'preview-edit',
        revision: request.expectedRevision,
        exportJobId: 'preview-export',
        irArtifactId: 'preview-ir',
        renderArtifactId: 'preview-render',
        connectionId: connection.connectionId,
        channelId: connection.channelId,
        channelTitle: connection.title,
        metadata: request.metadata,
        state: 'private',
        acknowledgedBytes: 42_000_000,
        totalBytes: 42_000_000,
        videoId: 'preview0000',
        visibility: 'private',
        errorCode: '',
        error: '',
        createdUnixMillis: 1,
        updatedUnixMillis: 2,
      };
      uploads = [record];
      return Promise.resolve(record);
    },
    listYoutubeUploads: () => Promise.resolve(uploads),
    getYoutubeUpload: () =>
      uploads[0] ? Promise.resolve(uploads[0]) : Promise.reject(new Error('No preview upload')),
    updateYoutubeUpload: unavailable,
    publishYoutubeUpload: unavailable,
    openYoutubePage: unavailable,
  };
}
