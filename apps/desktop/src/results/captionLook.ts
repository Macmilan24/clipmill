/**
 * The caption look a project's clips start with, chosen when it was imported.
 * Kept on this machine per project; the Editor can change any clip's look later.
 */

export const CAPTION_LOOKS = [
  { label: 'Clean', ref: 'clipmill.captions.clean.v1' },
  { label: 'Minimal', ref: 'clipmill.captions.minimal.v1' },
  { label: 'Boxed', ref: 'clipmill.captions.boxed.v1' },
] as const;

export const DEFAULT_LOOK = CAPTION_LOOKS[0].ref;

const key = (projectId: string) => `clipmill.captionLook.${projectId}`;
const highlightKey = (projectId: string) => `clipmill.captionHighlight.${projectId}`;
const optionsKey = (projectId: string) => `clipmill.captionOptions.${projectId}`;

/**
 * The options a saved style starts a project's clips with, or none for a
 * plain look. Its look is remembered with `rememberLook` beside it.
 */
export function rememberOptions(
  projectId: string,
  options: Readonly<Record<string, unknown>> | null,
): void {
  try {
    if (options) localStorage.setItem(optionsKey(projectId), JSON.stringify(options));
    else localStorage.removeItem(optionsKey(projectId));
  } catch {
    /* Clips start with the look's own options instead. */
  }
}

/** The options a project's clips start with, as the daemon reads them. */
export function optionsFor(projectId: string): string | undefined {
  try {
    const stored = localStorage.getItem(optionsKey(projectId));
    if (!stored) return undefined;
    const parsed: unknown = JSON.parse(stored);
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? stored : undefined;
  } catch {
    return undefined;
  }
}

export function rememberHighlight(projectId: string, enabled: boolean): void {
  try {
    localStorage.setItem(highlightKey(projectId), enabled ? 'on' : 'off');
  } catch {
    /* default applies */
  }
}

export function highlightFor(projectId: string): boolean {
  try {
    return localStorage.getItem(highlightKey(projectId)) !== 'off';
  } catch {
    return true;
  }
}

export function rememberLook(projectId: string, styleRef: string): void {
  try {
    localStorage.setItem(key(projectId), styleRef);
  } catch {
    /* Clips start with the default look instead. */
  }
}

/** The look chosen for a project, when one was and it is still offered. */
export function lookFor(projectId: string): string | undefined {
  try {
    const stored = localStorage.getItem(key(projectId));
    return CAPTION_LOOKS.some((look) => look.ref === stored) ? (stored ?? undefined) : undefined;
  } catch {
    return undefined;
  }
}
