/**
 * What an editor gesture turns into.
 *
 * The claim these protect: nothing the editor does is local. A nudge, a split,
 * a gain step — each becomes a command the daemon can replay and undo, in the
 * shape the Edit IR deserializes. A gesture that changed the screen without
 * producing one of these would look right in the player and be absent from the
 * file.
 */
import { describe, expect, it } from 'vitest';

import type { PreviewPlan } from '../src/daemon/client.js';
import {
  OneEuro,
  batch,
  editCaptionText,
  mergeCues,
  removeGainPoint,
  setCropKeyframe,
  setCueLines,
  setGain,
  setLayout,
  snapToWord,
  splitCue,
  ticksAt,
  trim,
} from '../src/editor/commands.js';

function plan(): PreviewPlan {
  return {
    revision: 1,
    rateNum: 30_000,
    rateDen: 1_001,
    frameCount: 60,
    crops: Array.from({ length: 60 }, () => [0, 0, 608, 1080] as const),
    cues: [
      {
        cueId: 'hot_1',
        firstFrame: 10,
        endFrame: 25,
        region: 'lower_safe',
        karaoke: true,
        leadInCentis: 0,
        lines: [[{ text: 'the', holdCentis: 20 }]],
      },
      {
        cueId: 'hot_2',
        firstFrame: 40,
        endFrame: 55,
        region: 'lower_safe',
        karaoke: true,
        leadInCentis: 0,
        lines: [[{ text: 'point', holdCentis: 20 }]],
      },
    ],
    gain: [],
    width: 1080,
    height: 1920,
  };
}

describe('the commands an editor produces', () => {
  it('carry the tag the Edit IR dispatches on', () => {
    expect(setLayout('fit').op).toBe('set_layout');
    expect(setCropKeyframe(0, { x: 1, y: 2, width: 3, height: 4 }).op).toBe('set_crop_keyframe');
    expect(trim(0, 10).op).toBe('trim');
    expect(splitCue('c', 1, 'c_b').op).toBe('split_cue');
    expect(mergeCues('a', 'b').op).toBe('merge_cues');
    expect(setCueLines('c', [2, 3]).op).toBe('set_cue_lines');
    expect(editCaptionText('c', 0, 'x').op).toBe('edit_caption_text');
    expect(setGain(0, -3).op).toBe('set_gain');
    expect(removeGainPoint(0).op).toBe('remove_gain_point');
  });

  it('name their fields the way the deserializer expects', () => {
    expect(setCropKeyframe(90_000, { x: 1, y: 2, width: 3, height: 4 }, 'seg_2')).toEqual({
      op: 'set_crop_keyframe',
      segment_id: 'seg_2',
      t_ticks: 90_000,
      rect: { x: 1, y: 2, width: 3, height: 4 },
    });
    expect(splitCue('hot_1', 2, 'hot_1_b')).toEqual({
      op: 'split_cue',
      cue_id: 'hot_1',
      at_word_index: 2,
      new_cue_id: 'hot_1_b',
    });
  });

  it('group several edits into one undoable step', () => {
    const grouped = batch([setLayout('fit'), setGain(0, -1)]);
    expect(grouped.op).toBe('batch');
    expect((grouped.commands as unknown[]).length).toBe(2);
  });

  it('speak in ticks derived from the plan rather than a constant', () => {
    const at = plan();
    expect(ticksAt(at, 0)).toBe(0);
    // One frame at 30000/1001 is 3003 ticks of a 90 kHz clock.
    expect(ticksAt(at, 1)).toBe(3_003);
  });
});

describe('word snapping', () => {
  it('lands a trim on a caption boundary rather than the pointer', () => {
    const at = plan();
    expect(snapToWord(at, 12)).toBe(10);
    expect(snapToWord(at, 23)).toBe(25);
    expect(snapToWord(at, 41)).toBe(40);
  });

  it('leaves a frame alone when there is nothing to snap to', () => {
    const bare = { ...plan(), cues: [] };
    expect(snapToWord(bare, 17)).toBe(17);
  });
});

describe('the One-Euro filter', () => {
  it('passes the first sample through untouched', () => {
    const filter = new OneEuro();
    expect(filter.filter(100, 0)).toBe(100);
  });

  it('damps jitter around a held value', () => {
    const filter = new OneEuro();
    filter.filter(100, 0);
    const noisy = [104, 96, 103, 97].map((value, index) => filter.filter(value, (index + 1) * 16));
    // Every smoothed sample is closer to where the hand was than the raw one.
    for (const value of noisy) {
      expect(Math.abs(value - 100)).toBeLessThan(4);
    }
  });

  it('still follows a real move rather than lagging forever', () => {
    const filter = new OneEuro();
    filter.filter(0, 0);
    let last = 0;
    for (let step = 1; step <= 30; step += 1) {
      last = filter.filter(step * 10, step * 16);
    }
    expect(last).toBeGreaterThan(250);
  });

  it('forgets its history when a new drag starts', () => {
    const filter = new OneEuro();
    filter.filter(500, 0);
    filter.reset();
    expect(filter.filter(10, 16)).toBe(10);
  });
});
