/**
 * Turning an editor gesture into a command the daemon can replay.
 *
 * Every edit is a command, and every command comes back with its inverse. That
 * is why there is no mutation anywhere in the editor: dragging a keyframe does
 * not change a document, it *describes* a change, and what makes it real is the
 * daemon applying it and handing back the way out.
 *
 * The builders here are deliberately thin. The authority on what a command
 * means is the Rust crate that applies it; this file only has to name the
 * operation and put the fields where the deserializer expects them.
 */
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
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
export type LayoutMode = 'speaker_fill' | 'fit' | 'two_up';

export function setLayout(mode: LayoutMode, segmentId = SEGMENT): EditCommandJson {
  return { op: 'set_layout', segment_id: segmentId, state: mode };
}

export function setCropKeyframe(
  tTicks: number,
  rect: { readonly x: number; readonly y: number; readonly width: number; readonly height: number },
  segmentId = SEGMENT,
  secondary = false,
): EditCommandJson {
  return {
    op: secondary ? 'set_secondary_crop_keyframe' : 'set_crop_keyframe',
    segment_id: segmentId,
    t_ticks: tTicks,
    rect,
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
  if (!segment || ticks === null || ticks >= segment.outTicks) {
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
 * A solved keyframe as the crop keyframe the document stores.
 *
 * The solver answers in shares of the source frame at source ticks; the
 * document holds source pixels at segment-local ticks. This is the director's
 * own conversion, repeated here so a path the editor re-solves is the path the
 * director would have written: the height is what the solver decided, the
 * width follows the output aspect, both are even for the encoder's chroma
 * planes, and the rectangle is kept inside the frame. Converted against the
 * *output's* dimensions instead, as it was, a landscape source got a crop
 * measured in a frame it is not in.
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
 * A One-Euro filter, for the value under a live drag.
 *
 * A pointer is noisy and a crop that jitters while being dragged reads as a
 * broken control, so the value shown while the hand is moving is smoothed. What
 * gets **committed** is the smoothed value too — the alternative is a preview
 * that disagrees with the command it produced, which is the divergence this
 * whole workstream exists to prevent.
 *
 * The filter is the book's own choice (ch. 18) and is display-side by design:
 * nothing upstream of the drag ever sees it.
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
