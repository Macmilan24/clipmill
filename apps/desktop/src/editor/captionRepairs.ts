/**
 * What to do about a subtitle the export strip would point at.
 *
 * The strip measures each sidecar cue against the reading profile's floor and
 * names the ones that fall short. What it cannot say is the remedy, because
 * that depends on the cue's neighbours: a cue with a blank after it can be
 * held longer; a cue hemmed in on both sides can only be joined to the one
 * beside it; and a cue whose only "word" is the silence marker the recognizer
 * leaked should not be there at all. This works that out from the same
 * numbers the strip used — the floor and the gap the plan carries — so what it
 * offers is what clears the finding rather than what trades it for the next.
 *
 * Every repair is an ordinary IR command. It is undoable, replayable, and
 * applied by the daemon under the same checks as a hand edit, so nothing here
 * can put a document into a state the editor could not. "Fix all" is one
 * batch, planned in order against the bounds the earlier repairs leave, so
 * two neighbours never extend into the same blank.
 */
import type { EditCommandJson, PreviewCue, PreviewPlan } from '../daemon/client.js';
import { batch, mergeCues, removeCaptionWord, setCueTiming } from './commands.js';

const TICKS_PER_SECOND = 90_000;

/**
 * The one thing whisper.cpp writes for a window it heard nothing in. It is
 * ordinary text to the model, so it reaches the transcript as a word; the
 * caption engine now drops it on the way in, and this is what clears it from
 * a document derived before that. Sound descriptions a deaf viewer wants —
 * `[music]`, `[laughter]` — are not this and are never removed for them.
 */
const SILENCE_MARKER = /^\[blank_audio\]$/i;

export function isSilenceMarker(text: string): boolean {
  return SILENCE_MARKER.test(text.trim());
}

/**
 * A word with nothing in it to read: the silence marker, or bare punctuation
 * such as the `-` and `·` an earlier "drop from caption" left in a word's
 * place. A cue made only of these is not a caption, whatever its length.
 */
export function isUnreadable(text: string): boolean {
  return isSilenceMarker(text) || !/[\p{L}\p{N}]/u.test(text);
}

/** A sidecar cue the reading profile would refuse, and what would clear it. */
export interface ShortCue {
  readonly cue: PreviewCue;
  readonly startTicks: number;
  readonly endTicks: number;
  /** What "Fix" applies. Null when nothing automatic clears it. */
  readonly repair: Repair | null;
}

export type Repair =
  | { readonly kind: 'remove'; readonly label: string; readonly command: EditCommandJson }
  | {
      readonly kind: 'extend';
      readonly label: string;
      readonly command: EditCommandJson;
      readonly startTicks: number;
      readonly endTicks: number;
    }
  | {
      readonly kind: 'merge';
      readonly label: string;
      readonly command: EditCommandJson;
      readonly withCueId: string;
    };

/** The reading profile's numbers, as the plan carries them. */
interface Floors {
  readonly minDurationTicks: number;
  readonly minGapTicks: number;
  readonly programTicks: number;
}

/** One cue's bounds as the planner tracks them while repairs accumulate. */
interface Bounds {
  readonly cueId: string;
  start: number;
  end: number;
  readonly wordCount: number;
  /** Nothing in the cue can be read: every word is a marker or bare punctuation. */
  readonly unreadable: boolean;
  /** At least one word is the recognizer's silence marker. */
  readonly marker: boolean;
}

function floorsOf(plan: PreviewPlan): Floors | null {
  if (!plan.readingMinDurationTicks || !plan.readingCues) return null;
  return {
    minDurationTicks: plan.readingMinDurationTicks,
    minGapTicks: plan.readingMinGapTicks ?? 0,
    programTicks: plan.segments.reduce(
      (sum, segment) => sum + segment.outTicks - segment.inTicks,
      0,
    ),
  };
}

function boundsOf(cues: readonly PreviewCue[]): Bounds[] {
  const bounds: Bounds[] = [];
  for (const cue of cues) {
    if (cue.startTicks === undefined || cue.endTicks === undefined) continue;
    const words = cue.lines.flat();
    bounds.push({
      cueId: cue.cueId,
      start: cue.startTicks,
      end: cue.endTicks,
      wordCount: words.length,
      unreadable: words.length > 0 && words.every((word) => isUnreadable(word.text)),
      marker: words.some((word) => isSilenceMarker(word.text)),
    });
  }
  return bounds;
}

function seconds(ticks: number): string {
  return `${(ticks / TICKS_PER_SECOND).toFixed(2)}s`;
}

/**
 * The repair for `bounds[index]`, and the bounds as they stand once it is
 * applied. Null when the cue is fine, or nothing automatic would clear it.
 */
function planRepair(bounds: Bounds[], index: number, floors: Floors): Repair | null {
  const here = bounds[index]!;
  if (here.end - here.start >= floors.minDurationTicks) return null;

  if (here.unreadable) {
    // Position zero every time: each removal shifts the rest down, and the
    // last one leaves the cue empty, which the daemon drops.
    const commands = Array.from({ length: here.wordCount }, () => removeCaptionWord(here.cueId, 0));
    bounds.splice(index, 1);
    return {
      kind: 'remove',
      label: here.marker ? 'Remove silence marker' : 'Remove empty caption',
      command: commands.length === 1 ? commands[0]! : batch(commands),
    };
  }

  // Extend forward into the blank first — a subtitle that lingers reads as
  // finishing the thought; one that arrives early reads as a spoiler — and
  // reach backwards only for what the blank ahead could not give.
  const previous = bounds[index - 1];
  const next = bounds[index + 1];
  const low = previous ? previous.end + floors.minGapTicks : 0;
  const high = next ? next.start - floors.minGapTicks : floors.programTicks;
  let end = Math.max(here.end, Math.min(high, here.start + floors.minDurationTicks));
  let start = here.start;
  if (end - start < floors.minDurationTicks) {
    start = Math.min(here.start, Math.max(low, end - floors.minDurationTicks));
  }
  if (end - start >= floors.minDurationTicks && (start !== here.start || end !== here.end)) {
    here.start = start;
    here.end = end;
    return {
      kind: 'extend',
      label: `Hold for ${seconds(end - start)}`,
      command: setCueTiming(here.cueId, start, end),
      startTicks: start,
      endTicks: end,
    };
  }

  // No blank to grow into: join the neighbour, the one after by preference so
  // the merged cue keeps this cue's start and reads on into what follows.
  if (next) {
    here.end = next.end;
    bounds.splice(index + 1, 1);
    return {
      kind: 'merge',
      label: 'Merge with next caption',
      command: mergeCues(here.cueId, next.cueId),
      withCueId: next.cueId,
    };
  }
  if (previous) {
    previous.end = here.end;
    bounds.splice(index, 1);
    return {
      kind: 'merge',
      label: 'Merge with previous caption',
      command: mergeCues(previous.cueId, here.cueId),
      withCueId: previous.cueId,
    };
  }
  return null;
}

/**
 * Every sidecar cue held for less than the floor, each with the repair that
 * would clear it on its own — planned against the document as it stands, so
 * a person can take them in any order.
 */
export function shortCues(plan: PreviewPlan): readonly ShortCue[] {
  const floors = floorsOf(plan);
  if (!floors || !plan.readingCues) return [];
  const cues = plan.readingCues;
  const found: ShortCue[] = [];
  for (const cue of cues) {
    if (cue.startTicks === undefined || cue.endTicks === undefined) continue;
    if (cue.endTicks - cue.startTicks >= floors.minDurationTicks) continue;
    // A fresh copy per cue: each repair is offered as if it were the only one.
    const bounds = boundsOf(cues);
    const index = bounds.findIndex((candidate) => candidate.cueId === cue.cueId);
    found.push({
      cue,
      startTicks: cue.startTicks,
      endTicks: cue.endTicks,
      repair: index < 0 ? null : planRepair(bounds, index, floors),
    });
  }
  return found;
}

/**
 * One undoable step that clears every short cue it can, planned in order so
 * each repair sees what the ones before it did. Null when there is nothing
 * automatic to do.
 */
export function repairAll(plan: PreviewPlan): EditCommandJson | null {
  const floors = floorsOf(plan);
  if (!floors || !plan.readingCues) return null;
  const bounds = boundsOf(plan.readingCues);
  const commands: EditCommandJson[] = [];
  let index = 0;
  while (index < bounds.length) {
    const before = bounds.length;
    const repair = planRepair(bounds, index, floors);
    if (repair) commands.push(repair.command);
    // A removal or a merge shortens the list in place of advancing past it.
    if (bounds.length === before) index += 1;
  }
  if (commands.length === 0) return null;
  return commands.length === 1 ? commands[0]! : batch(commands);
}
