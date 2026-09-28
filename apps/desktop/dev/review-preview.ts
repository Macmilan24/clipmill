/**
 * A stand-in for the dry run in the UI preview: the clip an approval of the
 * cut on screen would build — a followed speaker, then both people — with
 * captions from the recording's own words, three to a cue.
 */
import type { EditIr } from '@clipmill/contracts';
import type { PreviewPlan } from '../src/daemon/client.js';
import type { Cut } from '../src/inspector/review.js';
import type { Transcript } from '../src/results/transcript.js';

const SECOND = 90_000;
const RATE = 30;

export function reviewPlanOf(
  cut: Cut,
  transcript: Transcript,
): { readonly plan: PreviewPlan; readonly captions: EditIr['captions'] } {
  const frameOf = (ticks: number) => Math.round((ticks * RATE) / SECOND);
  const middle = cut.startTicks + Math.round((cut.endTicks - cut.startTicks) / 2);
  const spans = [
    [cut.startTicks, middle, 'speaker_fill'],
    [middle, cut.endTicks, 'two_up'],
  ] as const;
  let program = 0;
  const segments = spans.map(([from, to, layout], index) => {
    const segment = {
      segmentId: `seg_${index + 1}`,
      sourceFingerprint: 'review',
      inTicks: from,
      outTicks: to,
      programStartTicks: program,
      firstFrame: frameOf(program),
      endFrame: frameOf(program + to - from),
      layout,
      upperHeight: layout === 'two_up' ? 960 : 0,
      hasTwoUpPaths: layout === 'two_up',
    };
    program += to - from;
    return segment;
  });
  const frameCount = frameOf(program);
  const speaker = [458, 36, 364, 648] as const;
  const crops = Array.from({ length: frameCount }, (_, frame) =>
    frame < segments[1]!.firstFrame ? speaker : ([150, 90, 608, 540] as const),
  );
  const secondaryCrops = Array.from({ length: frameCount }, (_, frame) =>
    frame < segments[1]!.firstFrame ? null : ([560, 90, 608, 540] as const),
  );
  const spoken = transcript.words.filter(
    (word) => word.startTicks >= cut.startTicks && word.endTicks <= cut.endTicks,
  );
  type Cue = NonNullable<EditIr['captions']['cues']>[number];
  const cues: Cue[] = [];
  for (let at = 0; at < spoken.length; at += 3) {
    const group = spoken.slice(at, at + 3);
    const start = group[0]!.startTicks - cut.startTicks;
    const end = group.at(-1)!.endTicks - cut.startTicks;
    cues.push({
      cue_id: `cue_${at / 3 + 1}`,
      start_ticks: start,
      end_ticks: end,
      region: 'lower_safe',
      anim: 'karaoke',
      lines: [
        {
          words: group.map((word, index) => ({
            word_id: `w${at + index}`,
            text: word.text,
            start_ticks: word.startTicks - cut.startTicks,
            end_ticks: word.endTicks - cut.startTicks,
          })) as Cue['lines'][number]['words'],
        },
      ],
    });
  }
  const plan = {
    revision: 0,
    rateNum: RATE,
    rateDen: 1,
    frameCount,
    width: 1080,
    height: 1920,
    presentation: 'burn_in',
    crops,
    secondaryCrops,
    cues: [],
    gain: [],
    segments,
    sources: [
      { sourceFingerprint: 'review', sourceId: 'review', displayWidth: 1280, displayHeight: 720 },
    ],
    proxies: [],
    decisions: [
      'Following the single clear face with a steady-size crop.',
      'Two people remain visible in equal portraits.',
    ],
  } as unknown as PreviewPlan;
  return {
    plan,
    captions: { style_ref: 'clipmill.captions.clean.v1', cues, burn_in: cues },
  };
}
