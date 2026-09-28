import { describe, expect, it } from 'vitest';

import type { PreviewCutaway, PreviewPlan } from '../src/daemon/client.js';
import {
  type Cutaway,
  cutawayAt,
  footageCutaway,
  footageSecondsAt,
  pictureCutaway,
  pushScale,
  recordingTicks,
  roomAt,
  savedCutaway,
  setCutaways,
  withCutaway,
} from '../src/editor/cutaways.js';
import { plan } from './support/clips.js';
import { mapping } from './support/plan.js';

const SECOND = 90_000;
const PICTURE = `sha256:${'3'.repeat(64)}`;
const RECORDING = `sha256:${'2'.repeat(64)}`;

/** A ten-second clip at 30 frames a second. */
function clip(cutaways: PreviewPlan['cutaways'] = []): PreviewPlan {
  const rate = { frameCount: 300, rateNum: 30, rateDen: 1 };
  return { ...plan(), ...mapping(rate, 0), ...rate, cutaways };
}

function footage(id: string, start: number, end: number, inTicks = 0): Cutaway {
  return {
    cutaway_id: id,
    start_ticks: start * SECOND,
    end_ticks: end * SECOND,
    content: { kind: 'footage', source_fingerprint: RECORDING, in_ticks: inTicks },
  };
}

describe('b-roll over the clip', () => {
  it('goes in at the playhead, into the room before the next one', () => {
    const list = [footage('cut_1', 2, 4), footage('cut_2', 5, 7)];
    // Two and a half seconds where there is room.
    expect(roomAt(list, 7.5 * SECOND, 10 * SECOND)).toEqual({
      start: 7.5 * SECOND,
      end: 10 * SECOND,
    });
    // Up to the next one, and no further.
    expect(roomAt(list, 4 * SECOND, 10 * SECOND)).toEqual({ start: 4 * SECOND, end: 5 * SECOND });
    // Never over another, nor for less than a fifth of a second.
    expect(roomAt(list, 3 * SECOND, 10 * SECOND)).toBeNull();
    expect(roomAt(list, 4.9 * SECOND, 10 * SECOND)).toBeNull();
    expect(roomAt(list, 9.9 * SECOND, 10 * SECOND)).toBeNull();
    // Moving one, its own span is room.
    expect(roomAt(list, 3 * SECOND, 10 * SECOND, SECOND, 'cut_1')).toEqual({
      start: 3 * SECOND,
      end: 4 * SECOND,
    });
  });

  it('makes a picture that moves closer, and footage from its recording', () => {
    const shown = clip();
    expect(pictureCutaway(shown, [], PICTURE, SECOND)).toEqual({
      cutaway_id: 'cut_1',
      start_ticks: SECOND,
      end_ticks: 3.5 * SECOND,
      content: { kind: 'picture', asset: PICTURE, push_in: true },
    });
    const list = [footage('cut_1', 2, 4)];
    expect(footageCutaway(shown, list, RECORDING, 30 * SECOND, 0)).toEqual({
      cutaway_id: 'cut_2',
      start_ticks: 0,
      end_ticks: 2 * SECOND,
      content: { kind: 'footage', source_fingerprint: RECORDING, in_ticks: 30 * SECOND },
    });
    expect(footageCutaway(shown, list, RECORDING, 0, 3 * SECOND)).toBeNull();
  });

  it('keeps the list in program order, and replaces it whole', () => {
    const list = withCutaway([footage('cut_2', 5, 7)], footage('cut_1', 1, 2));
    expect(list.map((cutaway) => cutaway.cutaway_id)).toEqual(['cut_1', 'cut_2']);
    expect(withCutaway(list, footage('cut_2', 0.2, 0.8)).map((item) => item.cutaway_id)).toEqual([
      'cut_2',
      'cut_1',
    ]);
    expect(setCutaways(list, [{ hash: PICTURE, license: 'own_content' }])).toEqual({
      op: 'set_cutaways',
      cutaways: list,
      assets: [{ hash: PICTURE, license: 'own_content' }],
    });
  });

  it('is the document’s own form again from the plan', () => {
    const shown: PreviewCutaway = {
      cutawayId: 'cut_1',
      startTicks: SECOND,
      endTicks: 2 * SECOND,
      firstFrame: 30,
      endFrame: 60,
      fit: 'fit',
      kind: 'picture',
      asset: PICTURE,
      pushIn: false,
      inTicks: 0,
    };
    expect(savedCutaway(shown)).toEqual({
      cutaway_id: 'cut_1',
      start_ticks: SECOND,
      end_ticks: 2 * SECOND,
      fit: 'fit',
      content: { kind: 'picture', asset: PICTURE },
    });
    expect(cutawayAt(clip([shown]), 45)).toBe(shown);
    expect(cutawayAt(clip([shown]), 60)).toBeUndefined();
  });

  it('moves a picture closer as the render does, eight per cent by its last frame', () => {
    const moving = { kind: 'picture' as const, firstFrame: 30, endFrame: 61, pushIn: true };
    expect(pushScale(moving, 30)).toBe(1);
    expect(pushScale(moving, 45)).toBeCloseTo(1.04, 6);
    expect(pushScale(moving, 60)).toBeCloseTo(1.08, 6);
    expect(pushScale({ ...moving, pushIn: false }, 60)).toBe(1);
  });

  it('plays footage from its recording on the clip’s clock', () => {
    const shown = {
      ...clip(),
      proxies: [
        {
          sourceFingerprint: RECORDING,
          artifactId: 'sha256:proxy',
          file: 'proxy.mp4',
          coverageStartTicks: 10 * SECOND,
          coverageEndTicks: 70 * SECOND,
          width: 1280,
          height: 720,
          rateNum: 30,
          rateDen: 1,
        },
      ],
    };
    const cutaway = {
      firstFrame: 60,
      startTicks: 2 * SECOND,
      inTicks: 40 * SECOND,
      sourceFingerprint: RECORDING,
    };
    // Half a second into it is 40.5 s into the recording, 30.5 s into its proxy.
    expect(footageSecondsAt(shown, cutaway, 75)).toBeCloseTo(30.5, 6);
    expect(footageSecondsAt(clip(), cutaway, 75)).toBeNull();
    expect(recordingTicks(shown, RECORDING)).toBe(60 * SECOND);
  });
});
