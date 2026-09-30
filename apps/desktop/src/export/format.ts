/**
 * The delivered picture's frame rate and size, as a creator chooses them.
 *
 * The default keeps the recording's own frame rate: converting 23.976 or 25
 * frames a second to 29.97 repeats a frame every few, and pans judder. Thirty
 * and sixty are there for a platform that asks for them. The size is named
 * by the 9:16 frame's height at it, and a clip in another shape keeps that
 * size's short side; larger sizes only help a recording that has the pixels,
 * and the export checks say so when it does not.
 */
import type { OutputFormat } from '../daemon/client.js';
import { type FrameShape, frameOfShape } from '../editor/layouts.js';

export type RateChoice = 'source' | '30' | '60';
export type HeightChoice = 1920 | 2560 | 3840;

export interface FormatChoice {
  readonly rate: RateChoice;
  readonly height: HeightChoice;
}

export const DEFAULT_FORMAT: FormatChoice = { rate: 'source', height: 1920 };

export const RATE_CHOICES: readonly RateChoice[] = ['source', '30', '60'];
export const HEIGHT_CHOICES: readonly HeightChoice[] = [1920, 2560, 3840];

const KEY = 'clipmill.export.format';

function isFormat(value: unknown): value is FormatChoice {
  if (typeof value !== 'object' || value === null) return false;
  const candidate = value as FormatChoice;
  return RATE_CHOICES.includes(candidate.rate) && HEIGHT_CHOICES.includes(candidate.height);
}

/** The format the last export used, which is what the next one starts from. */
export function recallFormat(): FormatChoice {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(KEY) ?? 'null');
    return isFormat(parsed) ? parsed : DEFAULT_FORMAT;
  } catch {
    return DEFAULT_FORMAT;
  }
}

export function rememberFormat(choice: FormatChoice): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(choice));
  } catch {
    // The default applies next time.
  }
}

/** What the daemon is asked for. */
export function outputFormat(choice: FormatChoice): OutputFormat {
  switch (choice.rate) {
    case '30':
      return { frameRateNum: 30, frameRateDen: 1, height: choice.height };
    case '60':
      return { frameRateNum: 60, frameRateDen: 1, height: choice.height };
    default:
      return { frameRateNum: 0, frameRateDen: 0, height: choice.height };
  }
}

/** `23.98` or `25`, as a person reads a frame rate. */
export function fpsText(fps: number): string {
  return Number.isInteger(fps) ? String(fps) : fps.toFixed(2);
}

export function rateLabel(rate: RateChoice, sourceFps: number | null): string {
  if (rate === 'source') {
    return sourceFps === null ? 'Match recording' : `Match recording (${fpsText(sourceFps)} fps)`;
  }
  return `${rate} fps`;
}

/** The delivered frame at a size, in a clip's shape, as the render sizes it. */
export function frameAt(
  height: HeightChoice,
  shape: FrameShape = 'vertical',
): { readonly width: number; readonly height: number } {
  const short = (height * 9) / 16;
  const frame = frameOfShape(shape);
  return { width: (frame.width * short) / 1080, height: (frame.height * short) / 1080 };
}

/** `1080p`, `1440p` or `4K`. */
export function sizeName(height: HeightChoice): string {
  return height === 2560 ? '1440p' : height === 3840 ? '4K' : '1080p';
}

export function heightLabel(height: HeightChoice, shape: FrameShape = 'vertical'): string {
  const frame = frameAt(height, shape);
  return `${sizeName(height)} · ${frame.width} × ${frame.height}`;
}

/** `1080 × 1920 · 23.98 fps · MP4`. */
export function formatSummary(
  choice: FormatChoice,
  sourceFps: number | null,
  shape: FrameShape = 'vertical',
): string {
  const frame = frameAt(choice.height, shape);
  const fps =
    choice.rate === 'source' ? (sourceFps === null ? null : fpsText(sourceFps)) : choice.rate;
  return [`${frame.width} × ${frame.height}`, fps === null ? null : `${fps} fps`, 'MP4']
    .filter(Boolean)
    .join(' · ');
}

/** The first video stream's frame rate in a stored probe, or null. */
export function sourceFpsOf(sourceMapJson: string): number | null {
  try {
    const map = JSON.parse(sourceMapJson) as {
      streams?: { kind?: string; video?: { frame_rate?: { num?: number; den?: number } } }[];
    };
    const rate = map.streams?.find((stream) => stream.kind === 'video')?.video?.frame_rate;
    if (!rate?.num || !rate.den) return null;
    return rate.num / rate.den;
  } catch {
    return null;
  }
}

const FOLDER_KEY = 'clipmill.export.folder';

/** The folder the last export went to, which the next one starts in. */
export function recallFolder(): string {
  try {
    return localStorage.getItem(FOLDER_KEY) ?? '';
  } catch {
    return '';
  }
}

export function rememberFolder(folder: string): void {
  if (!folder.trim()) return;
  try {
    localStorage.setItem(FOLDER_KEY, folder);
  } catch {
    // The next export asks again.
  }
}

/** Forget the folder, so the next export asks for one. */
export function forgetFolder(): void {
  try {
    localStorage.removeItem(FOLDER_KEY);
  } catch {
    // Nothing was kept.
  }
}

/** What an export names its files when nobody says otherwise. */
export const DEFAULT_PATTERN = '{index}-{clip}';
const PATTERN_KEY = 'clipmill.export.pattern';

/** The name pattern a new export starts with, as set in Settings. */
export function recallPattern(): string {
  try {
    return localStorage.getItem(PATTERN_KEY)?.trim() || DEFAULT_PATTERN;
  } catch {
    return DEFAULT_PATTERN;
  }
}

/** Keep `pattern` for every export that follows; empty goes back to the default. */
export function rememberPattern(pattern: string): void {
  try {
    if (pattern.trim() === '' || pattern.trim() === DEFAULT_PATTERN) {
      localStorage.removeItem(PATTERN_KEY);
    } else {
      localStorage.setItem(PATTERN_KEY, pattern.trim());
    }
  } catch {
    // Exports start from the default instead.
  }
}
