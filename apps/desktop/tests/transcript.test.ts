/**
 * The words a reviewer reads and cuts on: where a cut may land between them,
 * which one is being spoken, and which sentences frame a cut.
 */
import type { IndexTranscript, SpeechTranscript } from '@clipmill/contracts';
import { describe, expect, it } from 'vitest';

import {
  EDGE_PAD_TICKS,
  type Transcript,
  endAfter,
  readTranscript,
  sentencesAround,
  snapEnd,
  snapStart,
  startBefore,
  wordAt,
  wordInside,
} from '../src/results/transcript.js';

const SECOND = 90_000;
/** Seconds as whole ticks, the way every published position is. */
const t = (seconds: number) => Math.round(seconds * SECOND);

/** Two sentences: "One two three." at 1–2.6 s, then "Four five." at 4–5.1 s. */
function speech(): SpeechTranscript {
  const timed = [
    ['One', 1.0, 1.4],
    ['two', 1.5, 1.9],
    ['three.', 2.0, 2.6],
    ['Four', 4.0, 4.5],
    ['five.', 4.6, 5.1],
  ] as const;
  return {
    schema_version: 'clipmill.speech.transcript.v1',
    source_fingerprint: `sha256:${'aa'.repeat(32)}`,
    words: timed.map(([text, start, end], index) => ({
      // Transcript indices need not be positions; the index refers to them.
      index: index + 100,
      segment_index: index < 3 ? 0 : 1,
      text,
      start_ticks: t(start),
      end_ticks: t(end),
      confidence: { p50: 0.9, p10: 0.8 },
      timing: 'aligned',
    })),
    segments: [{ index: 0, first_word_index: 100, word_count: 5 }],
  } as unknown as SpeechTranscript;
}

function index(): IndexTranscript {
  return {
    sentences: [
      { index: 0, first_word_index: 100, word_count: 3, start_ticks: t(1), end_ticks: t(2.6) },
      { index: 1, first_word_index: 103, word_count: 2, start_ticks: t(4), end_ticks: t(5.1) },
    ],
  } as unknown as IndexTranscript;
}

function words(): Transcript {
  return readTranscript(speech(), index());
}

describe('reading a transcript', () => {
  it('groups words into the index sentences by transcript index, not by position', () => {
    const read = words();
    expect(read.words.map((word) => word.text)).toEqual(['One', 'two', 'three.', 'Four', 'five.']);
    expect(read.sentences).toEqual([
      { startTicks: SECOND, endTicks: t(2.6), firstWord: 0, wordCount: 3 },
      { startTicks: t(4), endTicks: t(5.1), firstWord: 3, wordCount: 2 },
    ]);
  });

  it('falls back to the recognizer segments when there is no index', () => {
    const read = readTranscript(speech(), null);
    expect(read.sentences).toEqual([
      { startTicks: SECOND, endTicks: t(5.1), firstWord: 0, wordCount: 5 },
    ]);
  });
});

describe('the word being spoken', () => {
  it('is the word under the playhead, and none in a pause', () => {
    const read = words();
    expect(wordAt(read, t(1.2))).toBe(0);
    expect(wordAt(read, t(4.7))).toBe(4);
    expect(wordAt(read, t(3))).toBe(-1);
    expect(wordAt(read, 0)).toBe(-1);
  });

  it('counts a word as inside a cut by its middle', () => {
    const [one] = words().words;
    // "One" runs 1.0–1.4 s; its middle is 1.2 s.
    expect(wordInside(one!, t(1.1), t(2))).toBe(true);
    expect(wordInside(one!, t(1.3), t(2))).toBe(false);
  });
});

describe('where a cut lands', () => {
  it('starts a little into the pause before a word, never into the word before', () => {
    const read = words();
    // A full pause before "Four": the start pads by the edge allowance.
    expect(startBefore(read, 3)).toBe(t(4) - EDGE_PAD_TICKS);
    // Only a tenth of a second between "One" and "two": the start stops at
    // the end of "One" rather than taking its tail.
    expect(startBefore(read, 1)).toBe(t(1.4));
    expect(startBefore(read, 0)).toBe(SECOND - EDGE_PAD_TICKS);
  });

  it('ends a little into the pause after a word, never into the word after', () => {
    const read = words();
    expect(endAfter(read, 2)).toBe(t(2.6) + EDGE_PAD_TICKS);
    expect(endAfter(read, 0)).toBe(t(1.5));
    expect(endAfter(read, 4)).toBe(t(5.1) + EDGE_PAD_TICKS);
  });

  it('snaps a dragged edge to the nearest boundary between words', () => {
    const read = words();
    // Mid-way through "two": the nearest start is the one before "two".
    expect(snapStart(read, t(1.6))).toBe(startBefore(read, 1));
    // Late in "three.": the nearest end is the one after it.
    expect(snapEnd(read, t(2.5))).toBe(endAfter(read, 2));
    // Just after "two" starts: the nearest end is after "One".
    expect(snapEnd(read, t(1.55))).toBe(endAfter(read, 0));
  });

  it('leaves a position alone when there are no words to snap to', () => {
    const empty: Transcript = { words: [], sentences: [] };
    expect(snapStart(empty, 123)).toBe(123);
    expect(snapEnd(empty, 456)).toBe(456);
  });
});

describe('the sentences shown around a cut', () => {
  it('includes the sentences the cut touches and the context either side', () => {
    const read = words();
    expect(sentencesAround(read, t(4), t(5.2), 0)).toEqual({ first: 1, end: 2 });
    expect(sentencesAround(read, t(4), t(5.2), 1)).toEqual({ first: 0, end: 2 });
    expect(sentencesAround(read, SECOND, t(5.2), 3)).toEqual({ first: 0, end: 2 });
  });
});
