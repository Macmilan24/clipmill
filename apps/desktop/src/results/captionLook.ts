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
