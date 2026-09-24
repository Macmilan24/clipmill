/** Map spoken source words to the edited program clock. */
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import type { Transcript } from '../results/transcript.js';
import { batch, editCaptionText, setWordText } from './commands.js';

export const TICKS = 90_000;
const FILLERS = new Set(['ah', 'eh', 'er', 'erm', 'hmm', 'huh', 'mhm', 'uh', 'uhm', 'um', 'umm']);

export interface ProgramWord {
  readonly text: string;
  readonly sourceIndex: number;
  readonly segmentId: string;
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
        },
      ];
    }),
  );
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
  if (
    startTicks < 0 ||
    endTicks <= startTicks ||
    endTicks > duration ||
    endTicks - startTicks >= duration
  )
    return null;
  return { op: 'ripple_delete', start_ticks: startTicks, end_ticks: endTicks, reflow_edges: true };
}

/** Descending cuts keep the earlier program positions unchanged. */
export function cutWords(
  plan: PreviewPlan,
  words: readonly ProgramWord[],
  positions: readonly number[],
): EditCommandJson | null {
  const selected = [...new Set(positions)]
    .toSorted((a, b) => a - b)
    .map((at) => words[at])
    .filter((word): word is ProgramWord => word !== undefined);
  if (selected.length === 0) return null;
  const groups: { start: number; end: number }[] = [];
  for (const word of selected) {
    const last = groups.at(-1);
    if (last && word.startTicks <= last.end + 1) last.end = Math.max(last.end, word.endTicks);
    else groups.push({ start: word.startTicks, end: word.endTicks });
  }
  const cuts = groups
    .toReversed()
    .map(({ start, end }) => rippleRange(plan, start, end))
    .filter((cut): cut is EditCommandJson => cut !== null);
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
    .toReversed()
    .map((pause) => rippleRange(plan, pause.startTicks, pause.endTicks))
    .filter((cut): cut is EditCommandJson => cut !== null);
  return cuts.length === 0 ? null : cuts.length === 1 ? cuts[0]! : batch(cuts);
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
      if (
        match?.[2]?.toLowerCase() === needle &&
        !seen.has(word.wordId || `${cue.cueId}:${wordIndex}`)
      ) {
        const text = `${match[1]}${replace.trim()}${match[3]}`;
        commands.push(
          word.wordId
            ? setWordText(word.wordId, text)
            : editCaptionText(cue.cueId, wordIndex, text, plan.presentation),
        );
        seen.add(word.wordId || `${cue.cueId}:${wordIndex}`);
      }
      wordIndex += 1;
    }
  }
  return commands.length === 0 ? null : batch(commands);
}
