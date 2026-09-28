/**
 * Build replayable daemon commands from editor gestures.
 * The daemon applies each command and returns its inverse for undo. Builders
 * supply operation names and fields; the Rust implementation defines semantics.
 */
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import type { EditIr } from '@clipmill/contracts';
import { secondsAt, segmentAt, sourceTicksAt } from './player.js';

/**
 * The segment a document has when the plan does not say.
 *
 * The director cuts one segment and calls it this. Every builder takes the id
 * from the plan where it can, so a document with two segments is addressed
 * correctly; the constant is what remains for a caller with no plan in hand.
 */
export const SEGMENT = 'seg_1';

/** Which way the camera is framed. Three modes, exactly as the plan names them. */
export type LayoutMode = 'speaker_fill' | 'fit' | 'two_up' | 'picture_in_picture';

export function setLayout(mode: LayoutMode, segmentId = SEGMENT): EditCommandJson {
  return { op: 'set_layout', segment_id: segmentId, state: mode };
}

export function splitSegment(
  segmentId: string,
  atTicks: number,
  newSegmentId: string,
): EditCommandJson {
  return {
    op: 'split_segment',
    segment_id: segmentId,
    at_ticks: atTicks,
    new_segment_id: newSegmentId,
  };
}

/** The daemon derives captions for only the newly exposed source span. */
/**
 * Derive the clip's captions again from the newest transcript of its
 * recording. The daemon turns this into the cues themselves, so the saved
 * step replays and undoes without the transcript.
 */
export function refreshCaptions(): EditCommandJson {
  return { op: 'refresh_captions' };
}

export function extendWithCaptions(
  segmentId: string,
  inTicks: number,
  outTicks: number,
): EditCommandJson {
  return {
    op: 'extend_with_captions',
    segment_id: segmentId,
    in_ticks: inTicks,
    out_ticks: outTicks,
  };
}

export function swapPortraits(segmentId: string): EditCommandJson {
  return { op: 'swap_portraits', segment_id: segmentId };
}

export function setCaptionStyle(styleRef: string): EditCommandJson {
  return { op: 'set_caption_style', style_ref: styleRef };
}

export function setCaptionOptions(
  options: NonNullable<EditIr['captions']['options']>,
): EditCommandJson {
  return { op: 'set_caption_options', options };
}

export function setCueRegion(
  cueId: string,
  region: 'upper_safe' | 'center' | 'lower_safe',
  presentation: Presentation = 'reading',
): EditCommandJson {
  return { op: 'set_cue_region', cue_id: cueId, region, ...inList(presentation) };
}

/** The whole clip's soft-cut duration in document ticks; zero disables it. */
export function setTransition(durationTicks: number): EditCommandJson {
  return { op: 'set_transition', duration_ticks: durationTicks };
}

export function setCropKeyframe(
  tTicks: number,
  rect: { readonly x: number; readonly y: number; readonly width: number; readonly height: number },
  segmentId = SEGMENT,
  secondary = false,
  easing?: 'linear' | 'ease_in' | 'ease_out' | 'ease_in_out',
): EditCommandJson {
  return {
    op: secondary ? 'set_secondary_crop_keyframe' : 'set_crop_keyframe',
    segment_id: segmentId,
    t_ticks: tTicks,
    rect,
    ...(easing ? { easing } : {}),
  };
}

export function removeCropKeyframe(
  tTicks: number,
  segmentId = SEGMENT,
  secondary = false,
): EditCommandJson {
  return {
    op: secondary ? 'remove_secondary_crop_keyframe' : 'remove_crop_keyframe',
    segment_id: segmentId,
    t_ticks: tTicks,
  };
}

export function trim(inTicks: number, outTicks: number, segmentId = SEGMENT): EditCommandJson {
  return { op: 'trim', segment_id: segmentId, in_ticks: inTicks, out_ticks: outTicks };
}

/** Which of the document's two cue lists a cue-scoped command means. */
export type Presentation = 'reading' | 'burn_in';

/** A caption's centre on the frame, in thousandths of its width and height. */
export interface CaptionPosition {
  readonly x: number;
  readonly y: number;
}

/** Place one caption by hand, or hand it back to its region with null. */
export function setCuePosition(
  cueId: string,
  position: CaptionPosition | null,
  presentation: Presentation = 'reading',
): EditCommandJson {
  return {
    op: 'set_cue_position',
    cue_id: cueId,
    ...(position ? { position } : {}),
    ...inList(presentation),
  };
}

/** Mark or unmark a key word, in both caption presentations. */
export function setWordEmphasis(wordId: string, emphasis: boolean): EditCommandJson {
  return { op: 'set_word_emphasis', word_id: wordId, emphasis };
}

/** Show at most this many words on screen at once. The subtitle files keep theirs. */
export function regroupOnScreen(maxWords: number): EditCommandJson {
  return { op: 'regroup_on_screen', max_words: maxWords };
}

/** Remove recognizer dashes, silence markers and bracketed annotations. */
export function dropNonSpeechWords(): EditCommandJson {
  return { op: 'drop_non_speech_words' };
}

/** Name the clip, or clear the name with null. */
export function setTitle(title: string | null): EditCommandJson {
  return title === null ? { op: 'set_title' } : { op: 'set_title', title };
}

/** The presentation field, written only when it is not the default. */
function inList(presentation: Presentation): { readonly presentation?: Presentation } {
  return presentation === 'reading' ? {} : { presentation };
}

export function editCaptionText(
  cueId: string,
  wordIndex: number,
  text: string,
  presentation: Presentation = 'reading',
): EditCommandJson {
  return {
    op: 'edit_caption_text',
    cue_id: cueId,
    word_index: wordIndex,
    text,
    ...inList(presentation),
  };
}

/**
 * Correct one word, wherever it appears.
 *
 * Addressed to the word's identity rather than to a cue and an index, so the
 * correction lands in the burned-in cue on screen and in the reading cue the
 * sidecars are written from. The cue-and-index form above finds a word in one
 * grouping only, and the grouping it finds is not the one the player shows.
 */
export function setWordText(wordId: string, text: string): EditCommandJson {
  return { op: 'set_word_text', word_id: wordId, text };
}

/**
 * Begin the program at this frame, removing all earlier shots too.
 *
 * `Trim` speaks source ticks — the segment's own window into the recording —
 * so the frame is mapped through the plan's segments rather than sent as
 * program ticks, which read as a window at the recording's opening. Null when
 * the frame is off the program or the plan carries no segments.
 */
export function trimStartAt(plan: PreviewPlan, frame: number): EditCommandJson | null {
  const segment = segmentAt(plan, frame);
  const ticks = sourceTicksAt(plan, frame);
  if (!segment || ticks === null || ticks <= segment.inTicks || ticks >= segment.outTicks) {
    return null;
  }
  if (plan.segments.length > 1) {
    const end = segment.programStartTicks + ticks - segment.inTicks;
    return end > 0
      ? { op: 'ripple_delete', start_ticks: 0, end_ticks: end, reflow_edges: true }
      : null;
  }
  return trim(ticks, segment.outTicks, segment.segmentId);
}

/** End the program at this frame, removing all later shots too. */
export function trimEndAt(plan: PreviewPlan, frame: number): EditCommandJson | null {
  const segment = segmentAt(plan, frame);
  const ticks = sourceTicksAt(plan, frame);
  if (!segment || ticks === null) {
    return null;
  }
  if (plan.segments.length > 1) {
    const start = segment.programStartTicks + ticks - segment.inTicks;
    const end = plan.segments.reduce((total, item) => total + item.outTicks - item.inTicks, 0);
    return start > 0 && start < end
      ? { op: 'ripple_delete', start_ticks: start, end_ticks: end, reflow_edges: true }
      : null;
  }
  if (ticks <= segment.inTicks) return null;
  return trim(segment.inTicks, ticks, segment.segmentId);
}

/**
 * Convert normalized source coordinates and source ticks to source pixels and
 * segment-local ticks, matching the director. Preserve solver height, derive
 * width from output aspect, round dimensions to even values for chroma planes,
 * and clamp to the source frame. Use source dimensions, not output dimensions.
 */
export function solvedKeyframe(
  keyframe: {
    readonly tTicks: number;
    readonly centerX: number;
    readonly centerY: number;
    readonly scale: number;
  },
  segment: { readonly inTicks: number },
  source: { readonly displayWidth: number; readonly displayHeight: number },
  aspect: { readonly width: number; readonly height: number },
): { readonly tTicks: number; readonly rect: CropRect } {
  const frameWidth = source.displayWidth;
  const frameHeight = source.displayHeight;
  let height = clamp(Math.round(keyframe.scale * frameHeight), 2, frameHeight);
  height -= height % 2;
  let width = 2 * Math.round((height * aspect.width) / aspect.height / 2);
  if (width > frameWidth) {
    width = frameWidth - (frameWidth % 2);
    height = Math.round((width * aspect.height) / aspect.width);
  }
  width = Math.max(2, width);
  height = Math.max(2, height);
  const x = Math.round(keyframe.centerX * frameWidth) - Math.floor(width / 2);
  const y = Math.round(keyframe.centerY * frameHeight) - Math.floor(height / 2);
  return {
    tTicks: Number(keyframe.tTicks) - segment.inTicks,
    rect: {
      x: clamp(x, 0, Math.max(0, frameWidth - width)),
      y: clamp(y, 0, Math.max(0, frameHeight - height)),
      width,
      height,
    },
  };
}

interface CropRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

function clamp(value: number, low: number, high: number): number {
  return Math.max(low, Math.min(high, value));
}

/**
 * Where a frame sits inside its own segment, which is the clock crop keyframes
 * keep: segment-local ticks, so a trim of the window cannot silently re-time
 * the camera move.
 */
export function segmentTicksAt(
  plan: PreviewPlan,
  frame: number,
): { readonly segmentId: string; readonly tTicks: number } {
  const segment = segmentAt(plan, frame);
  if (!segment) {
    return { segmentId: SEGMENT, tTicks: ticksAt(plan, frame) };
  }
  return {
    segmentId: segment.segmentId,
    tTicks: Math.max(0, ticksAt(plan, frame) - segment.programStartTicks),
  };
}

export function setCueLines(
  cueId: string,
  lineWordCounts: readonly number[],
  presentation: Presentation = 'reading',
): EditCommandJson {
  return {
    op: 'set_cue_lines',
    cue_id: cueId,
    line_word_counts: [...lineWordCounts],
    ...inList(presentation),
  };
}

export function splitCue(
  cueId: string,
  atWordIndex: number,
  newCueId: string,
  presentation: Presentation = 'reading',
): EditCommandJson {
  return {
    op: 'split_cue',
    cue_id: cueId,
    at_word_index: atWordIndex,
    new_cue_id: newCueId,
    ...inList(presentation),
  };
}

export function mergeCues(
  firstCueId: string,
  secondCueId: string,
  presentation: Presentation = 'reading',
): EditCommandJson {
  return {
    op: 'merge_cues',
    first_cue_id: firstCueId,
    second_cue_id: secondCueId,
    ...inList(presentation),
  };
}

/**
 * Stop showing a word, in both caption tracks at once.
 *
 * A caption edit and not a media edit: the word was said and the audio is
 * untouched. Addressed by cue and position in the track on screen; the daemon
 * follows the word's identity into the other track, and drops a cue that is
 * left with nothing to show.
 */
export function removeCaptionWord(
  cueId: string,
  wordIndex: number,
  presentation: Presentation = 'reading',
): EditCommandJson {
  return {
    op: 'remove_caption_word',
    cue_id: cueId,
    word_index: wordIndex,
    ...inList(presentation),
  };
}

/** Hold a cue for a different window. Its spoken words do not move. */
export function setCueTiming(
  cueId: string,
  startTicks: number,
  endTicks: number,
  presentation: Presentation = 'reading',
): EditCommandJson {
  return {
    op: 'set_cue_timing',
    cue_id: cueId,
    start_ticks: startTicks,
    end_ticks: endTicks,
    ...inList(presentation),
  };
}

/**
 * The command that corrects one shown word, whatever the document knows it by.
 *
 * Addressed to the word's identity when it has one, so it lands in both
 * presentations. A document that predates ids is corrected by cue and index
 * in the presentation on screen — the one place the command can reach.
 */
export function correctWord(
  plan: PreviewPlan,
  cue: { readonly cueId: string },
  wordIndex: number,
  word: { readonly wordId: string },
  text: string,
): EditCommandJson {
  return word.wordId === ''
    ? editCaptionText(cue.cueId, wordIndex, text, plan.presentation)
    : setWordText(word.wordId, text);
}

export function setGain(tTicks: number, gainDb: number): EditCommandJson {
  return { op: 'set_gain', t_ticks: tTicks, gain_db: gainDb };
}

export function removeGainPoint(tTicks: number): EditCommandJson {
  return { op: 'remove_gain_point', t_ticks: tTicks };
}

/** Several edits as one undoable step. */
export function batch(commands: readonly EditCommandJson[]): EditCommandJson {
  return { op: 'batch', commands: [...commands] };
}

/**
 * The ticks a frame begins at, which is the unit every command speaks.
 *
 * Derived from the plan's own rate rather than a constant, so a document at a
 * different frame rate does not need this file to know about it.
 */
export function ticksAt(plan: PreviewPlan, frame: number): number {
  return Math.round(secondsAt(plan, frame) * 90_000);
}

/**
 * The word boundary nearest a tick position.
 *
 * Trimming to a word rather than to wherever the mouse landed is the same rule
 * the boundary optimizer follows upstream: a cut inside a word is a cut a
 * viewer hears. The candidates come from the plan's cues, which carry the words
 * that will actually be burned in.
 */
export function snapToWord(plan: PreviewPlan, frame: number): number {
  const edges: number[] = [];
  for (const cue of plan.cues) {
    edges.push(cue.firstFrame, cue.endFrame);
  }
  if (edges.length === 0) {
    return frame;
  }
  return edges.reduce(
    (best, edge) => (Math.abs(edge - frame) < Math.abs(best - frame) ? edge : best),
    edges[0]!,
  );
}

/**
 * Smooth live drag values with a One-Euro filter. Commit the smoothed value
 * so the resulting command matches the preview. Upstream crop solving is unaffected.
 */
export class OneEuro {
  private previous: number | null = null;
  private derivative = 0;
  private lastAt = 0;

  constructor(
    private readonly minimumCutoff = 1.2,
    private readonly beta = 0.02,
    private readonly derivativeCutoff = 1,
  ) {}

  reset(): void {
    this.previous = null;
    this.derivative = 0;
  }

  filter(value: number, atMillis: number): number {
    if (this.previous === null) {
      this.previous = value;
      this.lastAt = atMillis;
      return value;
    }
    const elapsed = Math.max(1, atMillis - this.lastAt) / 1000;
    this.lastAt = atMillis;

    const rate = (value - this.previous) / elapsed;
    this.derivative = smooth(alpha(this.derivativeCutoff, elapsed), rate, this.derivative);
    const cutoff = this.minimumCutoff + this.beta * Math.abs(this.derivative);
    this.previous = smooth(alpha(cutoff, elapsed), value, this.previous);
    return this.previous;
  }
}

function alpha(cutoff: number, elapsed: number): number {
  const tau = 1 / (2 * Math.PI * cutoff);
  return 1 / (1 + tau / elapsed);
}

function smooth(rate: number, value: number, previous: number): number {
  return rate * value + (1 - rate) * previous;
}
