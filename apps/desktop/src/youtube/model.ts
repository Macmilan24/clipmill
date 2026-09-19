import type { YoutubeUpload, YoutubeUploadState } from '../daemon/publishing.js';

export function publishingError(error: unknown): string {
  const text = error instanceof Error ? error.message : typeof error === 'string' ? error : '';
  return text.trim() || 'YouTube could not be reached. Refresh the saved status and try again.';
}

export const UPLOAD_STATES: Readonly<
  Record<YoutubeUploadState, { label: string; detail: string }>
> = {
  queued: { label: 'Queued', detail: 'Waiting to upload this approved render privately.' },
  verifying: {
    label: 'Checking the render',
    detail: 'Verifying the saved file and its approved revision.',
  },
  starting: {
    label: 'Preparing upload',
    detail: 'Creating a private upload session with YouTube.',
  },
  uploading: {
    label: 'Uploading privately',
    detail: 'Progress shows the bytes YouTube has acknowledged.',
  },
  paused: {
    label: 'Paused',
    detail: 'Your progress is saved. Resume checks the existing upload session first.',
  },
  auth_required: {
    label: 'Reconnect your channel',
    detail: 'Sign in to the same channel again, then resume this upload.',
  },
  reconciling: {
    label: 'Checking with YouTube',
    detail: 'Finding out what YouTube already received before continuing.',
  },
  completion_uncertain: {
    label: 'Completion needs checking',
    detail:
      'YouTube may already have received the video. Check the existing upload; a second video will not be created automatically.',
  },
  private: {
    label: 'Private on YouTube',
    detail: 'Review the uploaded video. Publishing is a separate action.',
  },
  publishing: {
    label: 'Publishing',
    detail: 'Changing the existing video to public. No second upload is created.',
  },
  public: { label: 'Public on YouTube', detail: 'This uploaded video is public.' },
  failed: {
    label: 'Upload needs attention',
    detail: 'The saved upload has stopped. Check its message before continuing.',
  },
};

export function uploadActive(state: YoutubeUploadState): boolean {
  return ['queued', 'verifying', 'starting', 'uploading', 'reconciling', 'publishing'].includes(
    state,
  );
}

export function newerUpload(
  previous: YoutubeUpload | undefined,
  next: YoutubeUpload,
): YoutubeUpload {
  return previous && previous.updatedUnixMillis > next.updatedUnixMillis ? previous : next;
}

/** User-entered tags stay plain text; no model or route claims are fabricated. */
export function parseTags(value: string): string[] {
  return [
    ...new Set(
      value
        .split(',')
        .map((tag) => tag.trim().replace(/^#+/, ''))
        .filter(Boolean),
    ),
  ];
}

function hasControls(value: string): boolean {
  return Array.from(value).some((character) => {
    const point = character.codePointAt(0)!;
    return point < 32 || (point >= 127 && point <= 159);
  });
}

/** Mirrors the transport's Unicode title and UTF-8 description/tag bounds. */
export function metadataLimits(title: string, description: string, tags: readonly string[]) {
  const encoder = new TextEncoder();
  const titleCharacters = Array.from(title).length;
  const descriptionBytes = encoder.encode(description).length;
  // YouTube counts separator bytes and implicit quotes around tags containing spaces.
  const tagBytes = tags.reduce(
    (sum, tag) => sum + encoder.encode(tag).length + 1 + (tag.includes(' ') ? 2 : 0),
    0,
  );
  const problem =
    !title.trim() || titleCharacters > 100 || /[<>]/.test(title) || hasControls(title)
      ? 'Use a title of 1–100 characters without angle brackets or control characters.'
      : descriptionBytes > 5000 ||
          /[<>]/.test(description) ||
          hasControls(description.replace(/[\n\t\r]/g, ''))
        ? 'Keep the description within 5,000 UTF-8 bytes, without angle brackets or control characters.'
        : tagBytes > 500 || tags.some((tag) => !tag.trim() || hasControls(tag))
          ? 'Keep tags within 500 UTF-8 bytes, including separators and quotes around tags with spaces.'
          : null;
  return { titleCharacters, descriptionBytes, tagBytes, problem };
}
