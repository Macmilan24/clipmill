import type { YoutubeImport, YoutubeImportState } from '../daemon/client.js';

export const YOUTUBE_SELECTION_KEY = 'clipmill.youtube-import.v1';

export interface YoutubeSelection {
  readonly importId?: string;
  readonly projectId: string;
  readonly videoId: string;
  readonly maxHeight?: number;
}

/** Legacy imports predate this choice and used the 1080p download ceiling. */
export function importHeight(value: { readonly maxHeight?: number } | null): number {
  return value?.maxHeight || 1080;
}

/** Advisory UI validation; the daemon independently validates the URL again. */
export function youtubeVideo(input: string): { videoId: string; url: string } | null {
  if (input.length > 2048) return null;
  for (const character of input) {
    const code = character.charCodeAt(0);
    if (code < 32 || code === 127) return null;
  }
  try {
    const url = new URL(input.trim());
    if (
      url.protocol !== 'https:' ||
      url.username ||
      url.password ||
      url.port ||
      url.searchParams.getAll('v').length > 1
    )
      return null;
    const host = url.hostname.toLowerCase();
    let id: string | null = null;
    if (host === 'youtu.be') id = url.pathname.slice(1);
    else if (['youtube.com', 'www.youtube.com', 'm.youtube.com'].includes(host)) {
      if (url.pathname === '/watch') id = url.searchParams.get('v');
      else id = /^\/(?:shorts|live|embed)\/([^/]+)\/?$/.exec(url.pathname)?.[1] ?? null;
    }
    return id && /^[A-Za-z0-9_-]{11}$/.test(id)
      ? { videoId: id, url: `https://www.youtube.com/watch?v=${id}` }
      : null;
  } catch {
    return null;
  }
}

export function activeImport(state: YoutubeImportState): boolean {
  return (
    state === 'queued' ||
    state === 'downloading' ||
    state === 'processing' ||
    state === 'registering'
  );
}

export const IMPORT_STATES: Record<YoutubeImportState, { label: string; detail: string }> = {
  queued: {
    label: 'Preparing download',
    detail: 'Checking this video and preparing a local copy.',
  },
  downloading: { label: 'Downloading video', detail: 'Saving the video and audio on this device.' },
  processing: {
    label: 'Preparing video',
    detail: 'Combining the downloaded video and audio into a local file.',
  },
  registering: {
    label: 'Checking the file',
    detail: 'Reading the downloaded video’s streams and timing before analysis.',
  },
  completed: {
    label: 'Imported',
    detail: 'The local copy is ready. Choose your analysis settings and continue.',
  },
  failed: {
    label: 'Import failed',
    detail: 'The video could not be imported. Your analysis has not started.',
  },
  cancelled: {
    label: 'Import cancelled',
    detail: 'No analysis was started. You can retry when you are ready.',
  },
  interrupted: {
    label: 'Download interrupted',
    detail:
      'The app stopped before the import finished. Retry explicitly to resume network activity.',
  },
};

export function rememberYoutube(selection: YoutubeSelection | null): void {
  try {
    if (selection) localStorage.setItem(YOUTUBE_SELECTION_KEY, JSON.stringify(selection));
    else localStorage.removeItem(YOUTUBE_SELECTION_KEY);
  } catch {
    /* Storage is optional; durable import state remains in the daemon. */
  }
}

export function recallYoutube(): YoutubeSelection | null {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(YOUTUBE_SELECTION_KEY) ?? 'null');
    if (!value || typeof value !== 'object' || !('projectId' in value) || !('videoId' in value))
      return null;
    if (typeof value.projectId !== 'string' || typeof value.videoId !== 'string') return null;
    if (!/^[A-Za-z0-9_-]{11}$/.test(value.videoId)) return null;
    if ('maxHeight' in value && ![0, 360, 720, 1080].includes(value.maxHeight as number))
      return null;
    return {
      projectId: value.projectId,
      videoId: value.videoId,
      ...('maxHeight' in value ? { maxHeight: value.maxHeight as number } : {}),
      ...('importId' in value && typeof value.importId === 'string'
        ? { importId: value.importId }
        : {}),
    };
  } catch {
    return null;
  }
}

/** Late poll responses cannot undo a retry or a newer daemon observation. */
export function newestImport(previous: YoutubeImport | null, next: YoutubeImport): YoutubeImport {
  if (!previous || previous.importId !== next.importId) return next;
  return next.attempt < previous.attempt ||
    (next.attempt === previous.attempt && next.updatedUnixMillis < previous.updatedUnixMillis)
    ? previous
    : next;
}
