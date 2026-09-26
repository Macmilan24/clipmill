/** Map spoken source words onto the edited program, and turn selections into edits. */
import type { EditIr } from '@clipmill/contracts';

import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import type { Transcript } from '../results/transcript.js';
import { batch, editCaptionText, removeCaptionWord, setWordText } from './commands.js';

export const TICKS = 90_000;
const FILLERS = new Set(['ah', 'eh', 'er', 'erm', 'hmm', 'huh', 'mhm', 'uh', 'uhm', 'um', 'umm']);

export interface ProgramWord {
  readonly text: string;
  readonly sourceIndex: number;
  readonly segmentId: string;
  readonly startTicks: number;
  readonly endTicks: number;
  /** Said, but its timing spread across a span rather than measured. */
  readonly guessed?: boolean;
}

/** A stretch of the program, in program ticks. */
export interface ProgramSpan {
  readonly startTicks: number;
  readonly endTicks: number;
}

export function programWords(plan: PreviewPlan, transcript: Transcript | null): ProgramWord[] {
  if (!transcript) return [];
  return plan.segments.flatMap((segment) =>
    transcript.words.flatMap((word, sourceIndex) => {
      const middle = (word.startTicks + word.endTicks) / 2;
      if (middle < segment.inTicks || middle >= segment.outTicks) return [];
      return [
        {
          text: word.text,
          sourceIndex,
          segmentId: segment.segmentId,
          startTicks: segment.programStartTicks + Math.max(0, word.startTicks - segment.inTicks),
          endTicks:
            segment.programStartTicks +
            Math.min(segment.outTicks - segment.inTicks, word.endTicks - segment.inTicks),
          ...(word.guessed ? { guessed: true } : {}),
        },
      ];
    }),
  );
}

/** The measured silences the program plays, in program ticks. */
export function programSilences(plan: PreviewPlan, transcript: Transcript | null): ProgramSpan[] {
  const silences = transcript?.silences ?? [];
  return plan.segments.flatMap((segment) =>
    silences.flatMap((silence) => {
      const from = Math.max(silence.startTicks, segment.inTicks);
      const to = Math.min(silence.endTicks, segment.outTicks);
      if (to <= from) return [];
      return [
        {
          startTicks: segment.programStartTicks + from - segment.inTicks,
          endTicks: segment.programStartTicks + to - segment.inTicks,
        },
      ];
    }),
  );
}

/** How far a guessed word's edge may be from the silence it snaps to. */
const SNAP_REACH_TICKS = 27_000;

/** Where speech resumes near `ticks`: the end of a measured silence close by. */
function speechStartsNear(ticks: number, silences: readonly ProgramSpan[]): number | null {
  let best: number | null = null;
  for (const silence of silences) {
    const distance = Math.abs(silence.endTicks - ticks);
    if (distance <= SNAP_REACH_TICKS && (best === null || distance < Math.abs(best - ticks))) {
      best = silence.endTicks;
    }
  }
  return best;
}

/** Where speech stops near `ticks`: the start of a measured silence close by. */
function speechStopsNear(ticks: number, silences: readonly ProgramSpan[]): number | null {
  let best: number | null = null;
  for (const silence of silences) {
    const distance = Math.abs(silence.startTicks - ticks);
    if (distance <= SNAP_REACH_TICKS && (best === null || distance < Math.abs(best - ticks))) {
      best = silence.startTicks;
    }
  }
  return best;
}

/** The fillers a person may also ask to find, beside the hesitations. */
export interface FillerChoices {
  readonly like: boolean;
  readonly youKnow: boolean;
}

/**
 * The program's fillers, as runs of word positions.
 *
 * Hesitations always. "Like" and "you know" only when asked for, and then
 * only where the recognizer set them off with a comma — "it was, like, huge"
 * — which is how a filler is punctuated and a verb is not.
 */
export function fillerRuns(
  words: readonly ProgramWord[],
  choices: FillerChoices,
): (readonly number[])[] {
  const plain = (at: number) => (words[at]?.text ?? '').toLowerCase().replaceAll(/[^a-z']/g, '');
  const commaAfter = (at: number) => /,\s*$/.test(words[at]?.text ?? '');
  const runs: (readonly number[])[] = [];
  for (let at = 0; at < words.length; at += 1) {
    const word = plain(at);
    if (FILLERS.has(word)) {
      runs.push([at]);
    } else if (choices.like && word === 'like' && (commaAfter(at) || commaAfter(at - 1))) {
      runs.push([at]);
    } else if (
      choices.youKnow &&
      word === 'you' &&
      plain(at + 1) === 'know' &&
      (commaAfter(at + 1) || commaAfter(at - 1))
    ) {
      runs.push([at, at + 1]);
      at += 1;
    }
  }
  return runs;
}

export function isFiller(text: string): boolean {
  return FILLERS.has(text.toLowerCase().replaceAll(/[^a-z']/g, ''));
}

export function rippleRange(
  plan: PreviewPlan,
  startTicks: number,
  endTicks: number,
): EditCommandJson | null {
  const duration = plan.segments.reduce((sum, part) => sum + part.outTicks - part.inTicks, 0);
  const start = Math.max(0, Math.round(startTicks));
  const end = Math.min(duration, Math.round(endTicks));
  if (end <= start || end - start >= duration) return null;
  return { op: 'ripple_delete', start_ticks: start, end_ticks: end, reflow_edges: true };
}

/**
 * Cut words from the picture and the sound, as one undoable step.
 *
 * Each run of chosen words goes with the pause after it, so what is left
 * keeps the speaker's rhythm rather than gaining a doubled silence. A run at
 * the end of a section takes the pause before it instead. Cuts are sent last
 * first, so each one's program times are still true when it lands.
 */
export function cutWords(
  plan: PreviewPlan,
  words: readonly ProgramWord[],
  positions: readonly number[],
  silences: readonly ProgramSpan[] = [],
): EditCommandJson | null {
  // An edge taken from a word the aligner could not place moves to where the
  // audio says speech starts or stops, when a measured silence is close by:
  // a guessed edge would otherwise clip the word or leave half of it behind.
  const start = (word: ProgramWord) =>
    (word.guessed ? speechStartsNear(word.startTicks, silences) : null) ?? word.startTicks;
  const end = (word: ProgramWord) =>
    (word.guessed ? speechStopsNear(word.endTicks, silences) : null) ?? word.endTicks;
  const range = (from: number, to: number, fallback: [number, number]) =>
    to > from ? rippleRange(plan, from, to) : rippleRange(plan, fallback[0], fallback[1]);
  const chosen = [...new Set(positions)]
    .filter((at) => words[at] !== undefined)
    .toSorted((a, b) => a - b);
  if (chosen.length === 0) return null;
  const runs: { first: number; last: number }[] = [];
  for (const at of chosen) {
    const run = runs.at(-1);
    if (run && at === run.last + 1) run.last = at;
    else runs.push({ first: at, last: at });
  }
  const cuts = runs
    .map(({ first, last }) => {
      const head = words[first]!;
      const tail = words[last]!;
      const after = words[last + 1];
      const before = words[first - 1];
      if (after && after.segmentId === tail.segmentId) {
        return range(start(head), start(after), [head.startTicks, after.startTicks]);
      }
      if (before && before.segmentId === head.segmentId) {
        return range(end(before), end(tail), [before.endTicks, tail.endTicks]);
      }
      return range(start(head), end(tail), [head.startTicks, tail.endTicks]);
    })
    .filter((cut): cut is EditCommandJson => cut !== null)
    .toReversed();
  return cuts.length === 0 ? null : cuts.length === 1 ? cuts[0]! : batch(cuts);
}

export interface PauseCut {
  readonly after: number;
  readonly seconds: number;
  readonly startTicks: number;
  readonly endTicks: number;
}

export function longPauses(words: readonly ProgramWord[], thresholdSeconds = 0.8): PauseCut[] {
  const cuts: PauseCut[] = [];
  for (let at = 0; at + 1 < words.length; at += 1) {
    const current = words[at]!;
    const next = words[at + 1]!;
    if (current.segmentId !== next.segmentId) continue;
    const gap = next.startTicks - current.endTicks;
    if (gap <= thresholdSeconds * TICKS) continue;
    const hold = Math.min(gap, 0.25 * TICKS);
    cuts.push({
      after: at,
      seconds: gap / TICKS,
      startTicks: Math.round(current.endTicks + hold / 2),
      endTicks: Math.round(next.startTicks - hold / 2),
    });
  }
  return cuts;
}

export function cutPauses(plan: PreviewPlan, pauses: readonly PauseCut[]): EditCommandJson | null {
  const cuts = pauses
    .toSorted((a, b) => a.startTicks - b.startTicks)
    .toReversed()
    .map((pause) => rippleRange(plan, pause.startTicks, pause.endTicks))
    .filter((cut): cut is EditCommandJson => cut !== null);
  return cuts.length === 0 ? null : cuts.length === 1 ? cuts[0]! : batch(cuts);
}

type SavedCue = NonNullable<EditIr['captions']['cues']>[number];

/** The cue list the player shows, as the document stores it. */
export function shownCues(plan: PreviewPlan, document: EditIr | null): readonly SavedCue[] {
  if (!document) return [];
  return plan.presentation === 'burn_in' && document.captions.burn_in?.length
    ? document.captions.burn_in
    : (document.captions.cues ?? []);
}

/** A spoken word's place in the captions: which cue, which word, what it now reads. */
export interface CaptionWordRef {
  readonly cueId: string;
  readonly index: number;
  readonly wordId: string;
  readonly text: string;
}

/**
 * Each program word's caption, matched by time, or null where the word was
 * hidden from the captions. What a correction or a hide changed shows here.
 */
export function captionRefs(
  words: readonly ProgramWord[],
  cues: readonly SavedCue[],
): (CaptionWordRef | null)[] {
  const flat = cues.flatMap((cue) =>
    cue.lines.flatMap((line) => line.words).map((word, index) => ({ cue, word, index })),
  );
  let cursor = 0;
  return words.map((word) => {
    const middle = (word.startTicks + word.endTicks) / 2;
    while (cursor < flat.length && flat[cursor]!.word.end_ticks <= word.startTicks) cursor += 1;
    for (let at = cursor; at < flat.length; at += 1) {
      const candidate = flat[at]!;
      if (candidate.word.start_ticks > word.endTicks) break;
      if (candidate.word.start_ticks <= middle && middle < candidate.word.end_ticks) {
        return {
          cueId: candidate.cue.cue_id,
          index: candidate.index,
          wordId: candidate.word.word_id ?? '',
          text: candidate.word.text,
        };
      }
    }
    return null;
  });
}

/** Stop showing words in the captions while the sound keeps them. */
export function hideInCaptions(
  plan: PreviewPlan,
  refs: readonly (CaptionWordRef | null)[],
): EditCommandJson | null {
  const byCue = new Map<string, CaptionWordRef[]>();
  for (const ref of refs) {
    if (ref) byCue.set(ref.cueId, [...(byCue.get(ref.cueId) ?? []), ref]);
  }
  // Within a cue, the last word goes first so earlier indexes stay true.
  const commands = [...byCue.values()].flatMap((group) =>
    group
      .toSorted((a, b) => b.index - a.index)
      .map((ref) => removeCaptionWord(ref.cueId, ref.index, plan.presentation)),
  );
  return commands.length === 0 ? null : commands.length === 1 ? commands[0]! : batch(commands);
}

/** Correct one caption word, by its identity when it has one. */
export function correctCaptionWord(
  plan: PreviewPlan,
  ref: CaptionWordRef,
  text: string,
): EditCommandJson {
  return ref.wordId
    ? setWordText(ref.wordId, text)
    : editCaptionText(ref.cueId, ref.index, text, plan.presentation);
}

/** How many caption words read `find`, ignoring case and the punctuation around them. */
export function countMatches(plan: PreviewPlan, find: string): number {
  const needle = find.trim().toLowerCase();
  if (!needle) return 0;
  let count = 0;
  for (const cue of plan.cues) {
    for (const word of cue.lines.flat()) {
      if (bare(word.text) === needle) count += 1;
    }
  }
  return count;
}

function bare(text: string): string | undefined {
  return /^[^\p{L}\p{N}]*([\p{L}\p{N}'’]+)[^\p{L}\p{N}]*$/u.exec(text)?.[1]?.toLowerCase();
}

/** Correct every named caption word once, even when it occurs in both cue lists. */
export function replaceCaptionWords(
  plan: PreviewPlan,
  find: string,
  replace: string,
): EditCommandJson | null {
  const needle = find.trim().toLowerCase();
  if (!needle || !replace.trim()) return null;
  const seen = new Set<string>();
  const commands: EditCommandJson[] = [];
  for (const cue of plan.cues) {
    let wordIndex = 0;
    for (const word of cue.lines.flat()) {
      const match = /^([^\p{L}\p{N}]*)([\p{L}\p{N}'’]+)([^\p{L}\p{N}]*)$/u.exec(word.text);
      const key = word.wordId || `${cue.cueId}:${wordIndex}`;
      if (match?.[2]?.toLowerCase() === needle && !seen.has(key)) {
        const text = `${match[1]}${replace.trim()}${match[3]}`;
        if (text !== word.text) {
          commands.push(
            word.wordId
              ? setWordText(word.wordId, text)
              : editCaptionText(cue.cueId, wordIndex, text, plan.presentation),
          );
        }
        seen.add(key);
      }
      wordIndex += 1;
    }
  }
  return commands.length === 0 ? null : batch(commands);
}
