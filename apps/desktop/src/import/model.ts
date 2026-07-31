/**
 * What a new run is asked for, and what it may be started with.
 *
 * The design's setup card offers rather more than this: creative direction
 * compiled into scoring parameters, audience and tone, speaker-aware reframing,
 * burnt-in captions. None of those has anything behind it — there is no brief
 * compiler in the pipeline, reframing is a later workstream, and captions are
 * another. What is here maps one-to-one onto fields the analyze payload actually
 * carries, so every control changes what the daemon does.
 */
const TICKS_PER_SECOND = 90_000;

export function secondsToTicks(seconds: number): number {
  return Math.round(seconds * TICKS_PER_SECOND);
}

export interface Preset {
  readonly id: string;
  readonly label: string;
  readonly detail: string;
  readonly minSeconds: number;
  readonly maxSeconds: number;
}

/**
 * The two ranges the design names, and a third the user sets.
 *
 * A range rather than a fixed length, because the contract takes a range: a
 * clip's right length is a property of the moment, and the search expands
 * against the bounds rather than cutting to a number.
 */
export const PRESETS: readonly Preset[] = [
  {
    id: 'short',
    label: 'Short',
    detail: 'Fast hooks with one clear payoff',
    minSeconds: 15,
    maxSeconds: 60,
  },
  {
    id: 'extended',
    label: 'Extended',
    detail: 'More context and narrative room',
    minSeconds: 60,
    maxSeconds: 180,
  },
  { id: 'custom', label: 'Custom', detail: 'Set the bounds yourself', minSeconds: 15, maxSeconds: 90 },
];

export const CUSTOM_PRESET_ID = 'custom';

/** The widest range the daemon will plan against, and the narrowest. */
export const DURATION_BOUNDS = { min: 5, max: 600 } as const;
export const COUNT_BOUNDS = { min: 1, max: 20 } as const;

/**
 * The sentinel for "let the recognizer decide".
 *
 * The payload wants an empty string for that, but a select cannot carry one —
 * an empty value is how a select says "nothing is chosen", so the option would
 * render blank. It is converted back at the edge, in `languageSubtag`.
 */
export const AUTO_LANGUAGE = 'auto';

/**
 * Languages offered for the recognizer, plus letting it decide.
 *
 * Primary subtags only, which is what the payload takes. The list is the speech
 * family's own coverage rather than every language with a code — an option that
 * produced a worse transcript than auto-detection would be a setting that makes
 * the result worse for looking thorough.
 */
export const LANGUAGES: readonly { readonly value: string; readonly label: string }[] = [
  { value: AUTO_LANGUAGE, label: 'Auto-detect' },
  { value: 'en', label: 'English' },
  { value: 'es', label: 'Spanish' },
  { value: 'fr', label: 'French' },
  { value: 'de', label: 'German' },
  { value: 'pt', label: 'Portuguese' },
  { value: 'it', label: 'Italian' },
  { value: 'nl', label: 'Dutch' },
  { value: 'ja', label: 'Japanese' },
  { value: 'zh', label: 'Chinese' },
  { value: 'hi', label: 'Hindi' },
  { value: 'ar', label: 'Arabic' },
];

export interface ImportSettings {
  readonly presetId: string;
  readonly minSeconds: number;
  readonly maxSeconds: number;
  readonly count: number;
  readonly language: string;
  readonly rightsAttested: boolean;
}

export const DEFAULT_SETTINGS: ImportSettings = {
  presetId: 'short',
  minSeconds: 15,
  maxSeconds: 60,
  count: 5,
  language: AUTO_LANGUAGE,
  rightsAttested: false,
};

/** What the payload carries: a BCP 47 primary subtag, or empty for auto. */
export function languageSubtag(settings: ImportSettings): string {
  return settings.language === AUTO_LANGUAGE ? '' : settings.language;
}

/** Selecting a preset moves the bounds with it; Custom keeps what is there. */
export function applyPreset(settings: ImportSettings, presetId: string): ImportSettings {
  const preset = PRESETS.find((candidate) => candidate.id === presetId);
  if (preset === undefined) {
    return settings;
  }
  return presetId === CUSTOM_PRESET_ID
    ? { ...settings, presetId }
    : { ...settings, presetId, minSeconds: preset.minSeconds, maxSeconds: preset.maxSeconds };
}

export function clamp(value: number, low: number, high: number): number {
  return Math.min(high, Math.max(low, value));
}

/** `15–60 sec`, which is what the summary row and the preset row both show. */
export function describeRange(settings: ImportSettings): string {
  return `${settings.minSeconds}–${settings.maxSeconds} sec`;
}

/**
 * Why the run cannot start yet, or nothing.
 *
 * One reason at a time and in the order a person would hit them, because a
 * button that is merely disabled teaches nobody what to do about it.
 */
export function blockingReason(
  settings: ImportSettings,
  hasSource: boolean,
  busy: boolean,
): string | null {
  if (busy) {
    return 'Working…';
  }
  if (!hasSource) {
    return 'Choose a video first';
  }
  if (settings.minSeconds >= settings.maxSeconds) {
    return 'The shortest clip must be shorter than the longest';
  }
  if (settings.minSeconds < DURATION_BOUNDS.min || settings.maxSeconds > DURATION_BOUNDS.max) {
    return `Clip length must be between ${DURATION_BOUNDS.min} and ${DURATION_BOUNDS.max} seconds`;
  }
  if (settings.count < COUNT_BOUNDS.min || settings.count > COUNT_BOUNDS.max) {
    return `Ask for between ${COUNT_BOUNDS.min} and ${COUNT_BOUNDS.max} clips`;
  }
  return settings.rightsAttested ? null : 'Confirm you hold the rights to this footage';
}

/** `pricing-mistakes-episode-41.mp4` becomes `pricing-mistakes-episode-41`. */
export function projectNameFor(absolutePath: string): string {
  const name = absolutePath.split(/[/\\]/).pop() ?? absolutePath;
  const stem = name.replace(/\.[^.]+$/, '');
  return stem === '' ? name : stem;
}
