/**
 * Music under the voice, and the voice cleaned before it is mixed.
 *
 * The render drops the music wherever the clip's words are said, on an
 * envelope it computes once and hands to the preview as levels by frame; the
 * editor plays the same sound at those levels, on the program's clock, so the
 * mix heard while editing is the export's. Voice cleanup is heard in the
 * rendered preview and the export: the proxy plays the recording as it is.
 */
import type { EditIr } from '@clipmill/contracts';
import { type RefObject, useEffect, useRef } from 'react';

import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';

export type MusicBed = NonNullable<EditIr['audio']['music']>;
export type VoiceCleanup = NonNullable<EditIr['audio']['cleanup']>;
type DocumentAsset = NonNullable<EditIr['assets']>[number];

export const MUSIC_LEVELS = { min: -40, max: 0, default: -18 } as const;
export const MUSIC_DUCKS = { min: -30, max: 0, default: -12 } as const;

export function setMusic(
  music: MusicBed | null,
  assets?: readonly DocumentAsset[],
): EditCommandJson {
  return {
    op: 'set_music',
    ...(music ? { music } : {}),
    ...(assets ? { assets: [...assets] } : {}),
  };
}

export function setCleanup(cleanup: VoiceCleanup | null): EditCommandJson {
  return { op: 'set_cleanup', ...(cleanup ? { cleanup } : {}) };
}

/** The music's level at a frame, in decibels, straight between the render's points. */
export function musicLevelAt(
  levels: readonly { readonly frame: number; readonly gainDb: number }[],
  frame: number,
): number {
  const first = levels[0];
  if (!first) return 0;
  if (frame <= first.frame) return first.gainDb;
  for (let index = 1; index < levels.length; index += 1) {
    const before = levels[index - 1]!;
    const after = levels[index]!;
    if (frame <= after.frame) {
      const span = after.frame - before.frame;
      return span <= 0
        ? after.gainDb
        : before.gainDb + ((after.gainDb - before.gainDb) * (frame - before.frame)) / span;
    }
  }
  return levels.at(-1)!.gainDb;
}

/** Where in the sound a program frame falls, in seconds, looping as the render does. */
export function musicSecondsAt(
  plan: Pick<PreviewPlan, 'rateNum' | 'rateDen'>,
  offsetTicks: number,
  frame: number,
  duration: number,
): number {
  const seconds = offsetTicks / 90_000 + (frame * plan.rateDen) / Math.max(1, plan.rateNum);
  return Number.isFinite(duration) && duration > 0 ? seconds % duration : seconds;
}

/** How far the music may wander from the picture before it is put back. */
const DRIFT_SECONDS = 0.15;

/**
 * Play a clip's music with the picture: started, stopped and moved with the
 * program, at the render's level for every frame. The element is the
 * caller's, so it can sit in the page like the picture's own.
 */
export function useMusicBed(
  element: RefObject<HTMLAudioElement | null>,
  plan: Pick<PreviewPlan, 'rateNum' | 'rateDen' | 'music'> | null,
  frame: number,
  playing: boolean,
  muted: boolean,
) {
  const music = plan?.music ?? null;
  const last = useRef({ frame, playing });
  useEffect(() => {
    const audio = element.current;
    if (!audio || !music || !plan) return;
    const expected = musicSecondsAt(plan, music.offsetTicks, frame, audio.duration);
    const jumped = Math.abs(frame - last.current.frame) > 2;
    if (!playing || jumped || Math.abs(audio.currentTime - expected) > DRIFT_SECONDS) {
      try {
        if (Math.abs(audio.currentTime - expected) > 1 / 90) audio.currentTime = expected;
      } catch {
        // Not loaded yet: the next frame puts it in place.
      }
    }
    audio.muted = muted;
    audio.volume = Math.min(1, Math.max(0, 10 ** (musicLevelAt(music.levels, frame) / 20)));
    if (playing && audio.paused) {
      // Older webviews answer play() with nothing rather than a promise.
      const started = audio.play() as Promise<void> | undefined;
      if (started) void started.catch(() => undefined);
    }
    if (!playing && !audio.paused) audio.pause();
    last.current = { frame, playing };
  }, [element, plan, music, frame, playing, muted]);
}
