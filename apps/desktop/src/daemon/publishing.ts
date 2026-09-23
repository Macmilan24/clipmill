import { isTauri } from './client.js';

export interface YoutubeConnection {
  readonly connectionId: string;
  readonly channelId: string;
  readonly title: string;
  readonly state: 'connecting' | 'connected' | 'interrupted' | 'failed' | 'disconnected';
  readonly error: string;
  readonly createdUnixMillis: number;
  readonly updatedUnixMillis: number;
}

export interface YoutubePublishingStatus {
  readonly available: boolean;
  readonly configured: boolean;
  readonly connections: readonly YoutubeConnection[];
}

export interface YoutubeVideoMetadata {
  readonly title: string;
  readonly description: string;
  readonly tags: readonly string[];
  readonly madeForKids: boolean;
  readonly containsSyntheticMedia: boolean;
}

export type YoutubeUploadState =
  | 'queued'
  | 'verifying'
  | 'starting'
  | 'uploading'
  | 'paused'
  | 'auth_required'
  | 'reconciling'
  | 'completion_uncertain'
  | 'private'
  | 'publishing'
  | 'public'
  | 'failed';
export interface YoutubeUpload {
  readonly uploadId: string;
  readonly projectId: string;
  readonly docId: string;
  readonly revision: number;
  readonly exportJobId: string;
  readonly irArtifactId: string;
  readonly renderArtifactId: string;
  readonly connectionId: string;
  readonly channelId: string;
  readonly channelTitle: string;
  readonly metadata: YoutubeVideoMetadata;
  readonly state: YoutubeUploadState;
  readonly acknowledgedBytes: number;
  readonly totalBytes: number;
  readonly videoId: string;
  readonly visibility: string;
  readonly errorCode: string;
  readonly error: string;
  readonly createdUnixMillis: number;
  readonly updatedUnixMillis: number;
}
export interface StartYoutubeUpload {
  readonly exportJobId: string;
  readonly connectionId: string;
  readonly expectedRevision: number;
  readonly metadata: YoutubeVideoMetadata;
  readonly rightsConfirmed: boolean;
}

export interface YoutubeMetadataDraft {
  readonly metadata: YoutubeVideoMetadata;
  readonly renderArtifactId: string;
  readonly revision: number;
  readonly transcriptExcerpt: string;
}

export interface PublishingApi {
  fetchYoutubePublishingStatus(): Promise<YoutubePublishingStatus>;
  chooseYoutubeClientConfig(): Promise<YoutubePublishingStatus | null>;
  connectYoutubeChannel(): Promise<string>;
  updateYoutubeConnection(
    connectionId: string,
    action: 'cancel' | 'disconnect',
  ): Promise<YoutubePublishingStatus>;
  startYoutubeUpload(request: StartYoutubeUpload): Promise<YoutubeUpload>;
  draftYoutubeMetadata(
    exportJobId: string,
    expectedRevision: number,
  ): Promise<YoutubeMetadataDraft>;
  listYoutubeUploads(projectId?: string): Promise<readonly YoutubeUpload[]>;
  getYoutubeUpload(uploadId: string): Promise<YoutubeUpload>;
  updateYoutubeUpload(
    uploadId: string,
    action: 'pause' | 'resume' | 'reconcile',
  ): Promise<YoutubeUpload>;
  publishYoutubeUpload(uploadId: string): Promise<YoutubeUpload>;
  openYoutubePage(
    page: 'setup' | 'console' | 'clients' | 'studio',
    uploadId?: string,
  ): Promise<void>;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error('Channel publishing is available in the desktop app.');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export const publishingApi: PublishingApi = {
  fetchYoutubePublishingStatus: () => call('youtube_publishing_status'),
  chooseYoutubeClientConfig: () => call('choose_youtube_client_config'),
  connectYoutubeChannel: () => call('connect_youtube_channel'),
  updateYoutubeConnection: (connectionId, action) =>
    call('update_youtube_connection', { connectionId, action }),
  startYoutubeUpload: (request) => call('start_youtube_upload', { request }),
  draftYoutubeMetadata: (exportJobId, expectedRevision) =>
    call('draft_youtube_metadata', { exportJobId, expectedRevision }),
  listYoutubeUploads: (projectId = '') => call('list_youtube_uploads', { projectId }),
  getYoutubeUpload: (uploadId) => call('get_youtube_upload', { uploadId }),
  updateYoutubeUpload: (uploadId, action) => call('update_youtube_upload', { uploadId, action }),
  publishYoutubeUpload: (uploadId) => call('publish_youtube_upload', { uploadId }),
  openYoutubePage: (page, uploadId) =>
    call('open_youtube_page', { page, ...(uploadId ? { uploadId } : {}) }),
};
