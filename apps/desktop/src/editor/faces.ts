/**
 * The faces the detector saw, placed on the whole frame, for pointing at the
 * one the camera should follow.
 *
 * The daemon samples faces a few times a second; the preview shows every
 * frame. A face is drawn where its two nearest samples put it, or at the
 * nearest one when only one is close, and not at all when neither is: a box
 * held over a cut away from somebody would point at a person who is not there.
 */
import type { FaceSighting } from '../daemon/client.js';

/** How far from a sample a face is still drawn at it: a quarter second. */
const REACH_TICKS = 22_500;

/** One face at one moment, with the track it belongs to. */
export type FaceNow = FaceSighting;

/** Each track's samples in time order, grouped once per fetch. */
export function byTrack(sightings: readonly FaceSighting[]): ReadonlyMap<number, FaceSighting[]> {
  const tracks = new Map<number, FaceSighting[]>();
  for (const seen of sightings) {
    const track = tracks.get(seen.trackId);
    if (track) track.push(seen);
    else tracks.set(seen.trackId, [seen]);
  }
  const ordered = new Map<number, FaceSighting[]>();
  for (const [id, track] of tracks)
    ordered.set(
      id,
      track.toSorted((a, b) => a.tTicks - b.tTicks),
    );
  return ordered;
}

/** Where each face is at `ticks` of the source, when a sample is near enough. */
export function facesAt(
  tracks: ReadonlyMap<number, readonly FaceSighting[]>,
  ticks: number,
): readonly FaceNow[] {
  const found: FaceNow[] = [];
  for (const samples of tracks.values()) {
    const next = firstAtOrAfter(samples, ticks);
    const after = samples[next];
    const before = next > 0 ? samples[next - 1] : undefined;
    const nearBefore = before !== undefined && ticks - before.tTicks <= REACH_TICKS;
    const nearAfter = after !== undefined && after.tTicks - ticks <= REACH_TICKS;
    // `before` is strictly earlier than `ticks` and `after` is not, so two
    // near samples always have time between them to share.
    if (before && after && nearBefore && nearAfter) {
      const share = (ticks - before.tTicks) / (after.tTicks - before.tTicks);
      const mix = (from: number, to: number) => from + (to - from) * share;
      found.push({
        trackId: before.trackId,
        tTicks: ticks,
        x: mix(before.x, after.x),
        y: mix(before.y, after.y),
        width: mix(before.width, after.width),
        height: mix(before.height, after.height),
      });
    } else if (after && nearAfter) {
      found.push(after);
    } else if (before && nearBefore) {
      found.push(before);
    }
  }
  return found.toSorted((a, b) => a.x - b.x);
}

/** The index of the first sample at or after `ticks`; the length when none is. */
function firstAtOrAfter(samples: readonly FaceSighting[], ticks: number): number {
  let low = 0;
  let high = samples.length;
  while (low < high) {
    const middle = (low + high) >> 1;
    if (samples[middle]!.tTicks < ticks) low = middle + 1;
    else high = middle;
  }
  return low;
}

/**
 * Where the whole source frame sits on a stage that fits it without cropping,
 * as shares of the stage: the same rectangle the composition draws the
 * Original view into.
 */
export function fittedFrame(
  source: { readonly displayWidth: number; readonly displayHeight: number },
  stage: { readonly width: number; readonly height: number },
): {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
} {
  const frame = source.displayWidth / Math.max(1, source.displayHeight);
  const shape = stage.width / Math.max(1, stage.height);
  const width = frame > shape ? 1 : frame / shape;
  const height = frame > shape ? shape / frame : 1;
  return { left: (1 - width) / 2, top: (1 - height) / 2, width, height };
}
