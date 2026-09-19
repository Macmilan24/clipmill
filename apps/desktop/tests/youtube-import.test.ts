import { afterEach, describe, expect, it } from 'vitest';
import {
  importHeight,
  newestImport,
  recallYoutube,
  rememberYoutube,
  youtubeVideo,
} from '../src/import/youtube.js';
import type { YoutubeImport } from '../src/daemon/client.js';

afterEach(() => localStorage.clear());

describe('single-video YouTube links', () => {
  it.each([
    'https://youtu.be/NYFGCESmikA?si=XwSGgLl0qT1Wz0tB',
    'https://www.youtube.com/watch?v=NYFGCESmikA&t=30',
    'https://www.youtube.com/watch?v=NYFGCESmikA&list=PL123',
    'https://m.youtube.com/shorts/NYFGCESmikA',
    'https://youtube.com/live/NYFGCESmikA',
    'https://youtube.com/embed/NYFGCESmikA',
  ])('canonicalizes %s without sending tracking parameters', (url) => {
    expect(youtubeVideo(url)).toEqual({
      videoId: 'NYFGCESmikA',
      url: 'https://www.youtube.com/watch?v=NYFGCESmikA',
    });
  });

  it.each([
    'http://youtu.be/NYFGCESmikA',
    'https://youtube.com.example.org/watch?v=NYFGCESmikA',
    'https://youtube.com@evil.example/watch?v=NYFGCESmikA',
    'https://youtube.com:8443/watch?v=NYFGCESmikA',
    'https://www.youtube.com/playlist?list=PL123',
    'https://www.youtube.com/watch?v=NYFGCESmikA&v=abcdefghijk',
    'https://www.youtube.com/watch?v=NYFGCESmikA\n',
    `https://www.youtube.com/watch?v=NYFGCESmikA&extra=${'x'.repeat(2048)}`,
    'https://youtu.be/NYFGCESmikA/another',
    'https://www.youtube.com/watch?v=short',
    'file:///tmp/video.mp4',
    'https://www.youtube.com/@channel',
  ])('refuses %s', (url) => expect(youtubeVideo(url)).toBeNull());
});

it('persists an explicit import identity and rejects malformed storage', () => {
  const selection = {
    importId: 'imp_old',
    projectId: 'prj_old',
    videoId: 'NYFGCESmikA',
    maxHeight: 360,
  };
  rememberYoutube(selection);
  expect(recallYoutube()).toEqual(selection);
  rememberYoutube(null);
  expect(recallYoutube()).toBeNull();
});

it('defaults only legacy quality to 1080p and preserves explicit lower-quality identities', () => {
  expect(importHeight(null)).toBe(1080);
  expect(importHeight({})).toBe(1080);
  expect(importHeight({ maxHeight: 0 })).toBe(1080);
  expect(importHeight({ maxHeight: 360 })).toBe(360);
  expect(importHeight({ maxHeight: 720 })).toBe(720);
});

it('does not replace a retry with a late status from its previous attempt', () => {
  const retried = { importId: 'imp_1', attempt: 2, updatedUnixMillis: 200 } as YoutubeImport;
  expect(newestImport(retried, { ...retried, attempt: 1, updatedUnixMillis: 300 })).toBe(retried);
  expect(newestImport(retried, { ...retried, updatedUnixMillis: 100 })).toBe(retried);
});
