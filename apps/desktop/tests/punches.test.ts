import { describe, expect, it } from 'vitest';

import {
  autoPunches,
  clearPunches,
  punchHere,
  punchSummary,
  punchesFor,
  rezoomPunches,
  sentences,
} from '../src/editor/punches.js';
import type { ProgramWord } from '../src/editor/transcript.js';
import { plan } from './support/clips.js';

const SECOND = 90_000;

/** Words a third of a second each, from `at` seconds, one sentence per list. */
function said(...runs: readonly (readonly string[])[]): ProgramWord[] {
  const words: ProgramWord[] = [];
  let at = 0;
  runs.forEach((run) => {
    run.forEach((text) => {
      words.push({
        text,
        sourceIndex: words.length,
        segmentId: 'seg_1',
        startTicks: at,
        endTicks: at + SECOND / 3,
      });
      at += SECOND / 3;
    });
    at += SECOND / 10;
  });
  return words;
}

const long = ['One', 'two', 'three', 'four', 'five', 'six.'];

describe('punch-ins', () => {
  it('reads sentences from full stops and long pauses', () => {
    const words = said(['Hello', 'there.'], ['How', 'are', 'you?']);
    expect(sentences(words)).toEqual([
      { start: 0, end: (2 * SECOND) / 3 },
      { start: (2 * SECOND) / 3 + SECOND / 10, end: (5 * SECOND) / 3 + SECOND / 10 },
    ]);
  });

  it('punches a chosen sentence within its section, and skips a short one', () => {
    const part = { programStartTicks: SECOND, inTicks: 0, outTicks: 10 * SECOND };
    expect(
      punchesFor(
        part,
        [
          { start: 2 * SECOND, end: 4 * SECOND },
          { start: 5 * SECOND, end: 5.5 * SECOND },
          { start: 6 * SECOND, end: 20 * SECOND },
        ],
        125,
      ),
    ).toEqual([
      { start_ticks: SECOND, end_ticks: 3 * SECOND, zoom: 125 },
      // A long sentence is held close for five seconds at most.
      { start_ticks: 5 * SECOND, end_ticks: 10 * SECOND, zoom: 125 },
    ]);
  });

  it('moves in on every other sentence of the clip, only where it follows someone', () => {
    const base = plan();
    const part = { ...base.segments[0]!, programStartTicks: 0, inTicks: 0, outTicks: 12 * SECOND };
    const words = said(long, long, long, long);
    const document = {
      video: {
        segments: [
          {
            segment_id: part.segmentId,
            source_fingerprint: part.sourceFingerprint,
            in_ticks: 0,
            out_ticks: 12 * SECOND,
            layout: {
              state: 'speaker_fill' as const,
              crop_path: [{ t_ticks: 0, rect: { x: 656, y: 0, width: 608, height: 1080 } }],
            },
          },
        ],
      },
    };
    const command = autoPunches({ ...base, segments: [part] }, document, words) as {
      op: string;
      commands: { op: string; punches: { start_ticks: number }[] }[];
    };
    expect(command.op).toBe('batch');
    const punches = command.commands[0]!.punches;
    // The second and fourth sentences, each two seconds of six words.
    expect(punches.map((punch) => punch.start_ticks)).toEqual([
      2 * SECOND + SECOND / 10,
      6 * SECOND + (3 * SECOND) / 10,
    ]);
    const fitted = {
      video: {
        segments: [{ ...document.video.segments[0]!, layout: { state: 'fit' as const } }],
      },
    };
    expect(autoPunches({ ...base, segments: [part] }, fitted, words)).toBeNull();
  });

  it('removes, re-zooms and adds one at the playhead, apart from the others', () => {
    const segment = {
      segment_id: 'seg_1',
      source_fingerprint: 'sha256:x',
      in_ticks: 0,
      out_ticks: 10 * SECOND,
      layout: {
        state: 'speaker_fill' as const,
        crop_path: [{ t_ticks: 0, rect: { x: 0, y: 0, width: 608, height: 1080 } }],
        punches: [{ start_ticks: SECOND, end_ticks: 3 * SECOND, zoom: 125 }],
      },
    };
    const document = { video: { segments: [segment] } };
    expect(punchSummary(document)).toEqual({ count: 1, zoom: 125 });
    expect(clearPunches(document)).toEqual({
      op: 'batch',
      commands: [{ op: 'set_punches', segment_id: 'seg_1', punches: [] }],
    });
    expect(rezoomPunches(document, 150)).toEqual({
      op: 'batch',
      commands: [
        {
          op: 'set_punches',
          segment_id: 'seg_1',
          punches: [{ start_ticks: SECOND, end_ticks: 3 * SECOND, zoom: 150 }],
        },
      ],
    });
    // Inside the existing punch there is no room; after it there is.
    expect(punchHere(segment, 2 * SECOND, 125, 10 * SECOND)).toBeNull();
    expect(punchHere(segment, 5 * SECOND, 125, 10 * SECOND)).toEqual({
      op: 'set_punches',
      segment_id: 'seg_1',
      punches: [
        { start_ticks: SECOND, end_ticks: 3 * SECOND, zoom: 125 },
        { start_ticks: 5 * SECOND, end_ticks: 7 * SECOND, zoom: 125 },
      ],
    });
    expect(clearPunches({ video: { segments: [] } })).toBeNull();
  });
});
