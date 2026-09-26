/**
 * What libass needs to draw a plan's captions: the script the export burns
 * in, the installed faces it may use, and the family it asks for first.
 */
import type { PreviewPlan } from '../daemon/client.js';
import type { CaptionFace } from './CaptionCanvas.js';

export interface ExactCaptions {
  readonly ass: string;
  readonly faces: readonly CaptionFace[];
  readonly family: string;
}

/**
 * The plan's captions for libass, or null where they cannot be drawn exactly
 * — no script, or no installed face — and the CSS approximation stands in.
 */
export function exactCaptionsOf(
  plan: PreviewPlan | null,
  fontUrl: ((file: string) => string) | null,
  ass: string | undefined = plan?.ass,
): ExactCaptions | null {
  if (!plan || !ass || !fontUrl) return null;
  const faces = (plan.fonts ?? [])
    .filter((face) => face.installed)
    .map((face) => ({ family: face.family, url: fontUrl(face.file) }));
  if (faces.length === 0) return null;
  const family =
    /Style: lower_safe,([^,]+),/.exec(ass)?.[1] ?? plan.captionStyle?.fontFamily ?? 'Inter';
  return { ass, faces, family };
}
