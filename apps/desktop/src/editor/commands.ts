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
import { secondsAt } from './player.js';

export const SEGMENT = 'seg_1';

/** Which way the camera is framed. Three modes, exactly as the plan names them. */
export type LayoutMode = 'speaker_fill' | 'fit';

export function setLayout(mode: LayoutMode, segmentId = SEGMENT): EditCommandJson {
  return { op: 'set_layout', segment_id: segmentId, state: mode };
}

export function setCropKeyframe(
  tTicks: number,
  rect: { readonly x: number; readonly y: number; readonly width: number; readonly height: number },
  segmentId = SEGMENT,
): EditCommandJson {
  return { op: 'set_crop_keyframe', segment_id: segmentId, t_ticks: tTicks, rect };
}

export function removeCropKeyframe(tTicks: number, segmentId = SEGMENT): EditCommandJson {
  return { op: 'remove_crop_keyframe', segment_id: segmentId, t_ticks: tTicks };
}

export function trim(inTicks: number, outTicks: number, segmentId = SEGMENT): EditCommandJson {
  return { op: 'trim', segment_id: segmentId, in_ticks: inTicks, out_ticks: outTicks };
}

export function editCaptionText(cueId: string, wordIndex: number, text: string): EditCommandJson {
  return { op: 'edit_caption_text', cue_id: cueId, word_index: wordIndex, text };
}

export function setCueLines(cueId: string, lineWordCounts: readonly number[]): EditCommandJson {
  return { op: 'set_cue_lines', cue_id: cueId, line_word_counts: [...lineWordCounts] };
}

export function splitCue(cueId: string, atWordIndex: number, newCueId: string): EditCommandJson {
  return { op: 'split_cue', cue_id: cueId, at_word_index: atWordIndex, new_cue_id: newCueId };
}

export function mergeCues(firstCueId: string, secondCueId: string): EditCommandJson {
  return { op: 'merge_cues', first_cue_id: firstCueId, second_cue_id: secondCueId };
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
