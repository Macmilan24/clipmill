/**
 * A board's thumbnails as the vertical frames their clips will have.
 *
 * The filmstrip stores wide stills of the recording. A 9:16 window of one,
 * placed where the camera would point, is what the clip will look like; the
 * daemon says where that is for every clip at once, and this turns its answer
 * into the CSS that places a still inside a vertical box.
 */

/** Width over height of a vertical clip. */
const VERTICAL = 9 / 16;

/**
 * The `object-position` that puts the point `centre` of the way across a
 * wide still in the middle of a vertical box, as far as the still allows.
 *
 * With `object-fit: cover`, a horizontal position of p puts p of the overflow
 * to the left; the overflow is the still's width less the box's, so centring
 * a point means solving for p and keeping it within the still.
 */
export function thumbnailPosition(centre: number, stillAspect = 16 / 9): string {
  const ratio = stillAspect / VERTICAL;
  if (ratio <= 1) return '50% 50%';
  const share = (centre * ratio - 0.5) / (ratio - 1);
  const clamped = Math.min(1, Math.max(0, share));
  return `${(clamped * 100).toFixed(1)}% 50%`;
}

/** Where in a clip a pointer across its thumbnail is, from 0 to 1. */
export function scrubShare(clientX: number, box: { left: number; width: number }): number {
  return Math.min(1, Math.max(0, (clientX - box.left) / Math.max(1, box.width)));
}
