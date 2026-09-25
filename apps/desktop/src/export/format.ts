/**
 * The delivered picture's frame rate and size, as a creator chooses them.
 *
 * The default keeps the recording's own frame rate: converting 23.976 or 25
 * frames a second to 29.97 repeats a frame every few, and pans judder. Thirty
 * and sixty are there for a platform that asks for them. The size is the
 * 9:16 frame's height; larger sizes only help a recording that has the pixels,
 * and the export checks say so when it does not.
 */
import type { OutputFormat } from '../daemon/client.js';

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

export function heightLabel(height: HeightChoice): string {
  switch (height) {
    case 2560:
      return '1440p · 1440 × 2560';
    case 3840:
      return '4K · 2160 × 3840';
    default:
      return '1080p · 1080 × 1920';
  }
}

/** `1080 × 1920 · 23.98 fps · MP4`. */
export function formatSummary(choice: FormatChoice, sourceFps: number | null): string {
  const width = (choice.height * 9) / 16;
  const fps =
    choice.rate === 'source' ? (sourceFps === null ? null : fpsText(sourceFps)) : choice.rate;
  return [`${width} × ${choice.height}`, fps === null ? null : `${fps} fps`, 'MP4']
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
