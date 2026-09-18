/**
 * Applying a preview plan. Nothing here derives anything.
 *
 * Every function in this file is a lookup. The crop at a frame is read out of
 * an array the daemon filled; the caption at a frame is the cue whose window
 * contains it; the highlighted word is found by walking holds that were already
 * measured in centiseconds. If any of this computed instead of looked up, it
 * would be a second implementation of the render's arithmetic — and the whole
 * point of the plan is that there is only one.
 *
 * The one thing this file *does* compute is the mapping between a media
 * element's `currentTime` and a frame index, and that is unavoidable: a
 * browser reports seconds, of the proxy, while the plan is in frames, of the
 * program. The two are not the same clock. A clip cut from ten minutes into a
 * recording starts at program frame zero and proxy second six hundred, and a
 * player that confused them seeked to the recording's opening — so the
 * conversion goes through the plan's own segment map, in one place, and every
 * seek, scrub and trim uses it.
 */
import type {
  PreviewCue,
  PreviewPlan,
  PreviewProxy,
  PreviewSegment,
  PreviewSource,
} from '../daemon/client.js';

/** Ticks per second, the daemon's timebase throughout. */
const TICKS_PER_SECOND = 90_000;

/** The frame a playhead in seconds is inside. */
export function frameAt(plan: PreviewPlan, seconds: number): number {
  if (plan.rateNum <= 0 || plan.rateDen <= 0) {
    return 0;
  }
  const frame = Math.floor((seconds * plan.rateNum) / plan.rateDen);
  return Math.max(0, Math.min(plan.frameCount - 1, frame));
}

/** Where a frame starts, in seconds. The inverse of `frameAt`. */
export function secondsAt(plan: PreviewPlan, frame: number): number {
  if (plan.rateNum <= 0) {
    return 0;
  }
  return (frame * plan.rateDen) / plan.rateNum;
}

/** The segment a program frame plays from, or null off the end. */
export function segmentAt(plan: PreviewPlan, frame: number): PreviewSegment | null {
  return (
    plan.segments.find((segment) => frame >= segment.firstFrame && frame < segment.endFrame) ?? null
  );
}

/** The source a segment names, when the plan knows it. */
export function sourceOf(plan: PreviewPlan, segment: PreviewSegment): PreviewSource | null {
  return (
    plan.sources.find((source) => source.sourceFingerprint === segment.sourceFingerprint) ?? null
  );
}

/** The proxy a segment plays from, when its source has one. */
export function proxyOf(plan: PreviewPlan, segment: PreviewSegment): PreviewProxy | null {
  return (
    plan.proxies.find((proxy) => proxy.sourceFingerprint === segment.sourceFingerprint) ?? null
  );
}

/**
 * The source tick a program frame plays, through the segment it is in.
 *
 * The document's own `program_to_source`, in frames: the frame's offset into
 * its segment, in ticks at the plan's rate, added to where the segment starts
 * in the source.
 */
export function sourceTicksAt(plan: PreviewPlan, frame: number): number | null {
  const segment = segmentAt(plan, frame);
  if (!segment) {
    return null;
  }
  return (
    segment.inTicks +
    Math.round(secondsAt(plan, frame) * TICKS_PER_SECOND) -
    segment.programStartTicks
  );
}

/**
 * Where the media element must be, in proxy seconds, to show a program frame.
 *
 * Null when the frame's source has no proxy — which is "nothing to play",
 * not "play from zero".
 */
export function proxySecondsAt(plan: PreviewPlan, frame: number): number | null {
  const segment = segmentAt(plan, frame);
  const proxy = segment ? proxyOf(plan, segment) : null;
  const ticks = sourceTicksAt(plan, frame);
  if (!proxy || ticks === null) {
    return null;
  }
  return (ticks - proxy.coverageStartTicks) / TICKS_PER_SECOND;
}

/**
 * The program frame a media element's time is showing, inside one segment.
 *
 * The inverse of `proxySecondsAt`, for the segment that is playing: a time
 * past the segment's end answers with the segment's last frame, and the
 * caller — which is what knows a segment ended — moves on to the next.
 */
export function frameAtProxySeconds(
  plan: PreviewPlan,
  segment: PreviewSegment,
  seconds: number,
): number {
  const proxy = proxyOf(plan, segment);
  if (!proxy || plan.rateNum <= 0 || plan.rateDen <= 0) {
    return segment.firstFrame;
  }
  const sourceTicks = seconds * TICKS_PER_SECOND + proxy.coverageStartTicks;
  const programSeconds =
    (sourceTicks - segment.inTicks + segment.programStartTicks) / TICKS_PER_SECOND;
  const frame = Math.floor((programSeconds * plan.rateNum) / plan.rateDen);
  return Math.max(segment.firstFrame, Math.min(segment.endFrame - 1, frame));
}

/**
 * The CSS transform that shows a crop in a stage the crop's aspect fills.
 *
 * The crop is a rectangle in the *source* frame, and the element on stage is
 * the source scaled to the stage's height. So the scale is how many times the
 * crop's height goes into the source's, and the translate moves the crop's
 * centre to the element's centre, as a share of the element's own size. Built
 * against the output's dimensions instead, as it was, this was right only for
 * a source that happened to share the output's aspect.
 */
export function stageTransform(
  crop: { readonly x: number; readonly y: number; readonly width: number; readonly height: number },
  source: { readonly displayWidth: number; readonly displayHeight: number },
): string | undefined {
  if (
    crop.width <= 0 ||
    crop.height <= 0 ||
    source.displayWidth <= 0 ||
    source.displayHeight <= 0
  ) {
    return undefined;
  }
  const scale = source.displayHeight / crop.height;
  const dx = (0.5 - (crop.x + crop.width / 2) / source.displayWidth) * 100;
  const dy = (0.5 - (crop.y + crop.height / 2) / source.displayHeight) * 100;
  return `scale(${scale}) translate(${dx}%, ${dy}%)`;
}

/** The crop the encoder will apply at a frame, or null where it fits. */
export function cropAt(
  plan: PreviewPlan,
  frame: number,
  secondary = false,
): {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
} | null {
  const crops = secondary ? (plan.secondaryCrops ?? []) : plan.crops;
  const found = crops[Math.max(0, Math.min(crops.length - 1, frame))];
  if (!found) {
    return null;
  }
  const [x, y, width, height] = found;
  return { x, y, width, height };
}

/** The cue on screen at a frame, or nothing. */
export function cueAt(plan: PreviewPlan, frame: number): PreviewCue | null {
  return plan.cues.find((cue) => frame >= cue.firstFrame && frame < cue.endFrame) ?? null;
}

/**
 * Which word of a cue carries the highlight at a frame.
 *
 * Walks the holds the plan already measured rather than re-deriving them from
 * word timings, which is the same reason the burned-in track and this agree:
 * both are reading one sweep. Returns -1 before the first word is sung.
 */
export function highlightedWord(plan: PreviewPlan, cue: PreviewCue, frame: number): number {
  if (!cue.karaoke) {
    return -1;
  }
  const elapsedCentis = (secondsAt(plan, frame) - secondsAt(plan, cue.firstFrame)) * 100;
  let at = cue.leadInCentis;
  if (elapsedCentis < at) {
    return -1;
  }
  let index = 0;
  for (const line of cue.lines) {
    for (const word of line) {
      at += word.holdCentis;
      if (elapsedCentis < at) {
        return index;
      }
      index += 1;
    }
  }
  return index - 1;
}

/** A cue's text as a reader sees it, lines kept apart. */
export function cueLines(cue: PreviewCue): readonly string[] {
  return cue.lines.map((line) => line.map((word) => word.text).join(' '));
}

/** Where a frame sits along a lane, as a percentage. */
export function lanePosition(plan: PreviewPlan, frame: number): number {
  if (plan.frameCount <= 1) {
    return 0;
  }
  return (frame / (plan.frameCount - 1)) * 100;
}

/** The same linear dB curve the renderer applies, held outside its endpoints. */
export function gainAt(plan: PreviewPlan, frame: number): number {
  const first = plan.gain[0];
  if (!first) return 0;
  if (frame <= first.frame) return first.gainDb;
  for (let index = 1; index < plan.gain.length; index++) {
    const before = plan.gain[index - 1]!;
    const after = plan.gain[index]!;
    if (frame <= after.frame) {
      const span = after.frame - before.frame;
      return span <= 0
        ? after.gainDb
        : before.gainDb + ((after.gainDb - before.gainDb) * (frame - before.frame)) / span;
    }
  }
  return plan.gain[plan.gain.length - 1]!.gainDb;
}

/** A frame as `m:ss.ff`, which is how an editor reads a transport. */
export function timecode(plan: PreviewPlan, frame: number): string {
  const seconds = secondsAt(plan, frame);
  const whole = Math.floor(seconds);
  const frames = Math.max(0, frame - Math.floor((whole * plan.rateNum) / plan.rateDen));
  return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}.${String(frames).padStart(2, '0')}`;
}
