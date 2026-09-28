import { describe, expect, it } from 'vitest';

import type { PreviewPlan } from '../src/daemon/client.js';
import { type ProgramWord, cutWords, fillerRuns } from '../src/editor/transcript.js';

const S = 90_000;
const say = (texts: string, guessed: readonly number[] = []): ProgramWord[] =>
  texts
    .split(' ')
    .map((text, at) =>
      Object.assign(
        { text, sourceIndex: at, segmentId: 'seg', startTicks: at * S, endTicks: at * S + S / 2 },
        guessed.includes(at) ? { guessed: true } : {},
      ),
    );
const plan = {
  segments: [{ segmentId: 'seg', inTicks: 0, outTicks: 20 * S }],
} as unknown as PreviewPlan;

describe('finding fillers', () => {
  const words = say('So, um, it was, like, huge and you know, I like it. Do you know him?');

  it('always finds hesitations, and "like" and "you know" only when asked', () => {
    expect(fillerRuns(words, { like: false, youKnow: false })).toEqual([[1]]);
    const all = fillerRuns(words, { like: true, youKnow: true });
    expect(all.map((run) => run.map((at) => words[at]!.text).join(' '))).toEqual([
      'um,',
      'like,',
      'you know,',
    ]);
  });

  it('leaves the verb and the question alone', () => {
    const found = fillerRuns(words, { like: true, youKnow: true }).flat();
    const verb = words.findIndex((word) => word.text === 'like');
    const question = words.findIndex((word) => word.text === 'him?') - 1;
    expect(words[verb]!.text).toBe('like');
    expect(words[question]!.text).toBe('know');
    expect(found).not.toContain(verb);
    expect(found).not.toContain(question);
  });
});

describe('cutting a word whose timing was guessed', () => {
  it('lands the cut where the audio says speech starts', () => {
    // "um" was spread across a span; voice activity heard a pause that ends
    // a fifth of a second before its guessed start.
    const words = say('so um right', [1]);
    const silences = [{ startTicks: 0.6 * S, endTicks: 0.8 * S }];
    const snapped = cutWords(plan, words, [1], silences);
    expect(snapped).toMatchObject({ start_ticks: 0.8 * S, end_ticks: 2 * S });
    // Without a silence close by, the guessed edge stands.
    expect(cutWords(plan, words, [1])).toMatchObject({ start_ticks: S, end_ticks: 2 * S });
  });

  it('leaves a measured word’s edges where they were measured', () => {
    const words = say('so um right');
    const silences = [{ startTicks: 0.6 * S, endTicks: 0.8 * S }];
    expect(cutWords(plan, words, [1], silences)).toMatchObject({ start_ticks: S });
  });
});
