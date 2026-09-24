/**
 * A ten-second clip cut from a steady speaker: a word every half second, a
 * sentence every five words, and two words per caption. Small enough to
 * reason about by hand, which is what the Editor's gesture tests need.
 */
import type { EditIr } from '@clipmill/contracts';

import type { PreviewPlan } from '../../src/daemon/client.js';
import type { Transcript } from '../../src/results/transcript.js';
import { mapping, TICKS } from './plan.js';

type SavedCue = NonNullable<EditIr['captions']['cues']>[number];

/** Word `i` starts at 590 s + i/2 s and lasts 0.3 s. */
export const wordStart = (index: number) => 53_100_000 + 45_000 * index;
export const wordEnd = (index: number) => wordStart(index) + 27_000;

export function talk(fillers: readonly number[] = []): {
  plan: PreviewPlan;
  document: EditIr;
  transcript: Transcript;
} {
  const words = Array.from({ length: 60 }, (_, index) => ({
    text: fillers.includes(index) ? 'um' : `word${index}${index % 5 === 4 ? '.' : ''}`,
    startTicks: wordStart(index),
    endTicks: wordEnd(index),
  }));
  const transcript: Transcript = {
    words,
    sentences: Array.from({ length: 12 }, (_, sentence) => ({
      firstWord: sentence * 5,
      wordCount: 5,
      startTicks: wordStart(sentence * 5),
      endTicks: wordEnd(sentence * 5 + 4),
    })),
  };
  // The clip plays source 600–610 s: words 20 to 39.
  const cues = Array.from({ length: 10 }, (_, cue) => {
    const first = 20 + cue * 2;
    return {
      id: `cue_${cue + 1}`,
      start: cue * TICKS,
      end: cue * TICKS + 81_000,
      words: [first, first + 1].map((index) => ({
        word_id: `w${index}`,
        text: words[index]!.text,
        start_ticks: wordStart(index) - 600 * TICKS,
        end_ticks: wordEnd(index) - 600 * TICKS,
      })),
    };
  });
  const saved: SavedCue[] = cues.map((cue) => ({
    cue_id: cue.id,
    start_ticks: cue.start,
    end_ticks: cue.end,
    region: 'lower_safe',
    anim: 'karaoke',
    lines: [{ words: cue.words as SavedCue['lines'][number]['words'] }],
  }));
  const program = { frameCount: 300, rateNum: 30, rateDen: 1 };
  const plan: PreviewPlan = {
    ...mapping(program, 600),
    ...program,
    revision: 3,
    width: 1080,
    height: 1920,
    crops: Array.from({ length: 300 }, () => [656, 0, 608, 1080] as const),
    gain: [],
    cues: cues.map((cue) => ({
      cueId: cue.id,
      firstFrame: (cue.start * 30) / TICKS,
      endFrame: Math.floor((cue.end * 30) / TICKS),
      region: 'lower_safe',
      karaoke: true,
      leadInCentis: 0,
      lines: [cue.words.map((word) => ({ text: word.text, wordId: word.word_id, holdCentis: 30 }))],
    })),
  };
  const document: EditIr = {
    version: 'ir/1',
    timebase: { num: 1, den: TICKS },
    video: {
      segments: [
        {
          segment_id: 'seg_1',
          source_fingerprint: plan.segments[0]!.sourceFingerprint,
          in_ticks: 600 * TICKS,
          out_ticks: 610 * TICKS,
          layout: {
            state: 'speaker_fill',
            crop_path: [{ t_ticks: 0, rect: { x: 656, y: 0, width: 608, height: 1080 } }],
          },
        },
      ],
    },
    captions: { style_ref: 'clipmill.captions.clean.v1', cues: saved, burn_in: saved },
    audio: { target_lufs: -14, true_peak_dbtp: -1, gain_curve: [] },
  };
  return { plan, document, transcript };
}
