/**
 * Transcript words and sentences for the Clip Inspector, reduced to what it draws.
 * Words come from the speech transcript and sentences from the index; the helpers
 * place cut edges between words, never inside one (R63).
 */
import type { IndexTranscript, SpeechTranscript } from '@clipmill/contracts';

export interface TranscriptWord {
  readonly text: string;
  readonly startTicks: number;
  readonly endTicks: number;
}

export interface TranscriptSentence {
  readonly startTicks: number;
  readonly endTicks: number;
  /** Position of the sentence's first word in `Transcript.words`. */
  readonly firstWord: number;
  readonly wordCount: number;
}

export interface Transcript {
  /** Ordered by time, as the transcript publishes them. */
  readonly words: readonly TranscriptWord[];
  /** Ordered by time, covering the words the index grouped. */
  readonly sentences: readonly TranscriptSentence[];
}

/**
 * How far into the pause a cut goes before it reaches a word — a tenth of a
 * second, so the first consonant is heard whole — and never so far that it
 * takes in the end of the word before.
 */
export const EDGE_PAD_TICKS = 9_000;

/**
 * Keep what a screen draws.
 *
 * Sentences come from the index when there is one. Without it they are the
 * recognizer's own segments, which are coarser but still end where the speaker
 * did, and a transcript with neither is still a list of words to read.
 */
export function readTranscript(
  speech: SpeechTranscript,
  index: IndexTranscript | null,
): Transcript {
  const words = speech.words.map((word) => ({
    text: word.text,
    startTicks: word.start_ticks,
    endTicks: word.end_ticks,
  }));
  const position = new Map(speech.words.map((word, at) => [word.index, at]));
  const grouped = index?.sentences?.length
    ? index.sentences.map((sentence) => ({
        first: position.get(sentence.first_word_index),
        count: sentence.word_count,
      }))
    : speech.segments.map((segment) => ({
        first: position.get(segment.first_word_index),
        count: segment.word_count,
      }));
  const sentences: TranscriptSentence[] = [];
  for (const group of grouped) {
    if (group.first === undefined || group.count < 1) continue;
    const last = words[Math.min(words.length - 1, group.first + group.count - 1)];
    const first = words[group.first];
    if (!first || !last) continue;
    sentences.push({
      startTicks: first.startTicks,
      endTicks: last.endTicks,
      firstWord: group.first,
      wordCount: Math.min(group.count, words.length - group.first),
    });
  }
  return { words, sentences };
}

/** The position of the last word that starts at or before `ticks`, or −1. */
function lastStartingBy(words: readonly TranscriptWord[], ticks: number): number {
  let low = 0;
  let high = words.length - 1;
  let found = -1;
  while (low <= high) {
    const middle = (low + high) >> 1;
    if (words[middle]!.startTicks <= ticks) {
      found = middle;
      low = middle + 1;
    } else {
      high = middle - 1;
    }
  }
  return found;
}

/** The word being spoken at `ticks`, by position, or −1 in a pause. */
export function wordAt(transcript: Transcript, ticks: number): number {
  const at = lastStartingBy(transcript.words, ticks);
  return at >= 0 && ticks < transcript.words[at]!.endTicks ? at : -1;
}

/**
 * Whether a word is inside a cut.
 *
 * By its middle rather than its edges: a cut that starts a few frames into a
 * pause and a cut that starts exactly on the word both contain it, and only a
 * word more than half outside the cut reads as outside it.
 */
export function wordInside(word: TranscriptWord, startTicks: number, endTicks: number): boolean {
  const middle = (word.startTicks + word.endTicks) / 2;
  return middle >= startTicks && middle < endTicks;
}

/** Where a clip that begins with the word at `position` starts. */
export function startBefore(transcript: Transcript, position: number): number {
  const word = transcript.words[position];
  if (!word) return 0;
  const previousEnd = position > 0 ? transcript.words[position - 1]!.endTicks : 0;
  return Math.min(word.startTicks, Math.max(previousEnd, word.startTicks - EDGE_PAD_TICKS));
}

/** Where a clip that ends with the word at `position` ends. */
export function endAfter(transcript: Transcript, position: number): number {
  const word = transcript.words[position];
  if (!word) return 0;
  const next = transcript.words[position + 1];
  const padded = word.endTicks + EDGE_PAD_TICKS;
  return Math.max(word.endTicks, next ? Math.min(next.startTicks, padded) : padded);
}

/**
 * The start point nearest `ticks`: where a clip beginning with one of the two
 * words around it would start. A cut can land between any two words (R63), and
 * this is the between it means.
 */
export function snapStart(transcript: Transcript, ticks: number): number {
  const { words } = transcript;
  if (words.length === 0) return ticks;
  const at = Math.max(0, lastStartingBy(words, ticks));
  const options = [at, Math.min(words.length - 1, at + 1)].map((position) =>
    startBefore(transcript, position),
  );
  return options.reduce((best, option) =>
    Math.abs(option - ticks) < Math.abs(best - ticks) ? option : best,
  );
}

/** The end point nearest `ticks`, the same way. */
export function snapEnd(transcript: Transcript, ticks: number): number {
  const { words } = transcript;
  if (words.length === 0) return ticks;
  const at = Math.max(0, lastStartingBy(words, ticks));
  const options = [Math.max(0, at - 1), at].map((position) => endAfter(transcript, position));
  return options.reduce((best, option) =>
    Math.abs(option - ticks) < Math.abs(best - ticks) ? option : best,
  );
}

/**
 * The sentences to show for a cut: every one it touches, and `context` more
 * on each side, as a range of positions in `sentences`.
 */
export function sentencesAround(
  transcript: Transcript,
  startTicks: number,
  endTicks: number,
  context = 2,
): { readonly first: number; readonly end: number } {
  const { sentences } = transcript;
  let first = sentences.findIndex((sentence) => sentence.endTicks > startTicks);
  if (first < 0) first = sentences.length;
  let end = first;
  while (end < sentences.length && sentences[end]!.startTicks < endTicks) end += 1;
  return {
    first: Math.max(0, first - context),
    end: Math.min(sentences.length, end + context),
  };
}
