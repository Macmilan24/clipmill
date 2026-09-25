/**
 * The repairs offered for a short subtitle, and the order "Fix all" plans them
 * in.
 *
 * The numbers are the reading profile's: a floor of five sixths of a second
 * and a gap of two frames at 24. What matters is that a repair clears the
 * finding without trading it for the next — a cue held to the floor that then
 * crowds its neighbour is refused by the same strip.
 */
import { describe, expect, it } from 'vitest';

import type { PreviewCue, PreviewPlan } from '../src/daemon/client.js';
import {
  isSilenceMarker,
  isUnreadable,
  repairAll,
  shortCues,
} from '../src/editor/captionRepairs.js';
import { mapping } from './support/plan.js';

const FLOOR = 75_000;
const GAP = 7_500;

function cue(id: string, startTicks: number, endTicks: number, text = 'Words.'): PreviewCue {
  return {
    cueId: id,
    startTicks,
    endTicks,
    firstFrame: Math.ceil(startTicks / 3_000),
    endFrame: Math.ceil(endTicks / 3_000),
    region: 'lower_safe',
    karaoke: false,
    leadInCentis: 0,
    lines: [
      text
        .split(' ')
        .map((word, index) => ({ text: word, holdCentis: 0, wordId: `${id}_${index}` })),
    ],
  };
}

/** A ten-second program at 30 fps with the given subtitle cues. */
function plan(readingCues: readonly PreviewCue[]): PreviewPlan {
  const program = { frameCount: 300, rateNum: 30, rateDen: 1 };
  return {
    ...mapping(program),
    revision: 1,
    rateNum: 30,
    rateDen: 1,
    frameCount: 300,
    crops: Array.from({ length: 300 }, () => null),
    cues: [],
    readingCues,
    readingMinDurationTicks: FLOOR,
    readingMinGapTicks: GAP,
    gain: [],
    width: 1080,
    height: 1920,
  };
}

describe('the silence marker', () => {
  it('is the one thing whisper writes for nothing, and not a sound description', () => {
    expect(isSilenceMarker('[BLANK_AUDIO]')).toBe(true);
    expect(isSilenceMarker(' [blank_audio] ')).toBe(true);
    expect(isSilenceMarker('[music]')).toBe(false);
    expect(isSilenceMarker('[laughter]')).toBe(false);
    expect(isSilenceMarker('blank')).toBe(false);
  });

  it('is one of the things nobody can read, beside bare punctuation', () => {
    expect(isUnreadable('[BLANK_AUDIO]')).toBe(true);
    expect(isUnreadable('-')).toBe(true);
    expect(isUnreadable('·')).toBe(true);
    expect(isUnreadable('…')).toBe(true);
    expect(isUnreadable('[music]')).toBe(false);
    expect(isUnreadable('a')).toBe(false);
    expect(isUnreadable('7')).toBe(false);
  });
});

describe('what a short subtitle is offered', () => {
  it('nothing, when every cue is held for the floor or longer', () => {
    expect(shortCues(plan([cue('a', 0, FLOOR), cue('b', 200_000, 400_000)]))).toEqual([]);
    expect(repairAll(plan([cue('a', 0, FLOOR)]))).toBeNull();
  });

  it('nothing, from a plan an older daemon sent without the subtitle track', () => {
    const { readingCues: _cues, readingMinDurationTicks: _floor, ...older } = plan([]);
    expect(shortCues(older as PreviewPlan)).toEqual([]);
    expect(repairAll(older as PreviewPlan)).toBeNull();
  });

  it('removal, when its only word is the silence marker', () => {
    const [found] = shortCues(plan([cue('a', 0, 57_432, '[BLANK_AUDIO]')]));
    expect(found?.repair).toMatchObject({ kind: 'remove', label: 'Remove silence marker' });
    expect(found?.repair?.command).toEqual({
      op: 'remove_caption_word',
      cue_id: 'a',
      word_index: 0,
    });
  });

  it('removal, when a placeholder is all an earlier edit left of it', () => {
    const [found] = shortCues(plan([cue('a', 0, 57_432, '-')]));
    expect(found?.repair).toMatchObject({ kind: 'remove', label: 'Remove empty caption' });
  });

  it('a longer hold into the blank after it, before anything else', () => {
    const [found] = shortCues(plan([cue('a', 90_000, 147_432), cue('b', 400_000, 500_000)]));
    expect(found?.repair).toMatchObject({
      kind: 'extend',
      label: 'Hold for 0.83s',
      startTicks: 90_000,
      endTicks: 165_000,
    });
    expect(found?.repair?.command).toEqual({
      op: 'set_cue_timing',
      cue_id: 'a',
      start_ticks: 90_000,
      end_ticks: 165_000,
    });
  });

  it('reaches backwards for what the blank ahead cannot give, and never crowds either neighbour', () => {
    // 50,000 ahead of b, less the gap: 42,500 of room after; the rest comes
    // from before, and stays a gap clear of a.
    const [found] = shortCues(
      plan([cue('a', 0, 100_000), cue('b', 150_000, 170_000), cue('c', 220_000, 400_000)]),
    );
    expect(found?.repair).toMatchObject({
      kind: 'extend',
      startTicks: 137_500,
      endTicks: 212_500,
    });
  });

  it('a merge with the next cue when there is no blank to grow into', () => {
    const [found] = shortCues(
      plan([cue('a', 0, 100_000), cue('b', 107_500, 127_500), cue('c', 135_000, 400_000)]),
    );
    expect(found?.repair).toMatchObject({
      kind: 'merge',
      label: 'Merge with next caption',
      withCueId: 'c',
    });
    expect(found?.repair?.command).toEqual({
      op: 'merge_cues',
      first_cue_id: 'b',
      second_cue_id: 'c',
    });
  });

  it('a merge with the previous cue when it is the last and the clip ends on it', () => {
    const program = 300 * 3_000;
    const [found] = shortCues(
      plan([cue('a', 0, program - 27_500), cue('b', program - 20_000, program)]),
    );
    expect(found?.repair).toMatchObject({ kind: 'merge', withCueId: 'a' });
    expect(found?.repair?.command).toEqual({
      op: 'merge_cues',
      first_cue_id: 'a',
      second_cue_id: 'b',
    });
  });

  it('is offered as if it were the only one, so the fixes can be taken in any order', () => {
    // Removing the marker would open the blank before c; offered alone, c's
    // repair must not assume that happened.
    const found = shortCues(
      plan([
        cue('a', 0, 80_000),
        cue('b', 87_500, 110_000, '[BLANK_AUDIO]'),
        cue('c', 165_000, 185_000),
        cue('d', 192_500, 400_000),
      ]),
    );
    expect(found.map((problem) => problem.repair?.kind)).toEqual(['remove', 'merge']);
  });
});

describe('fixing everything at once', () => {
  it('plans each repair against what the ones before it left, as one undoable batch', () => {
    // Once the marker is gone, c can be held from where the marker started.
    const everything = repairAll(
      plan([
        cue('a', 0, 80_000),
        cue('b', 87_500, 110_000, '[BLANK_AUDIO]'),
        cue('c', 165_000, 185_000),
        cue('d', 192_500, 400_000),
      ]),
    );
    expect(everything).toEqual({
      op: 'batch',
      commands: [
        { op: 'remove_caption_word', cue_id: 'b', word_index: 0 },
        { op: 'set_cue_timing', cue_id: 'c', start_ticks: 110_000, end_ticks: 185_000 },
      ],
    });
  });

  it('is the single command when only one cue needs one', () => {
    expect(repairAll(plan([cue('a', 0, 57_432, '[BLANK_AUDIO]')]))).toEqual({
      op: 'remove_caption_word',
      cue_id: 'a',
      word_index: 0,
    });
  });

  it('removes every word of a marker cue that somehow has several', () => {
    expect(repairAll(plan([cue('a', 0, 57_432, '[BLANK_AUDIO] [blank_audio]')]))).toEqual({
      op: 'batch',
      commands: [
        { op: 'remove_caption_word', cue_id: 'a', word_index: 0 },
        { op: 'remove_caption_word', cue_id: 'a', word_index: 0 },
      ],
    });
  });
});
