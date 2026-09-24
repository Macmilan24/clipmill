/** Synthetic Editor state for the opt-in browser preview. */
import type { EditIr } from '@clipmill/contracts';
import type { PreviewPlan } from '../src/daemon/client.js';
import type { Filmstrip } from '../src/results/loader.js';
import { plan as basePlan } from './fixtures.js';

const SECOND = 90_000;
type CaptionCue = NonNullable<EditIr['captions']['cues']>[number];

export const editorPlan: PreviewPlan = {
  ...basePlan,
  segments: basePlan.segments.map((segment) => ({
    ...segment,
    inTicks: 8 * SECOND,
    outTicks: 50 * SECOND,
  })),
};

export const editorDocument: EditIr = {
  version: 'ir/1',
  timebase: { num: 1, den: 90_000 },
  video: {
    segments: [
      {
        segment_id: 'segment',
        source_fingerprint: 'preview',
        in_ticks: 8 * SECOND,
        out_ticks: 50 * SECOND,
        layout: {
          state: 'speaker_fill',
          crop_path: [
            { t_ticks: 0, rect: { x: 656, y: 0, width: 608, height: 1080 } },
            { t_ticks: 20 * SECOND, rect: { x: 656, y: 0, width: 608, height: 1080 } },
          ],
        },
      },
    ],
  },
  captions: {
    style_ref: 'clipmill.captions.clean.v1',
    cues: editorPlan.cues.map((cue) => {
      const start = cue.firstFrame * 3_000;
      const end = cue.endFrame * 3_000;
      const words = cue.lines.flat();
      const mapped = words.map((word, index) => ({
        word_id: word.wordId,
        text: word.text,
        start_ticks: start + Math.round(((end - start) * index) / words.length),
        end_ticks: start + Math.round(((end - start) * (index + 1)) / words.length),
      }));
      const [first, ...rest] = mapped;
      if (!first) throw new Error(`Preview cue ${cue.cueId} has no words`);
      return {
        cue_id: cue.cueId,
        start_ticks: start,
        end_ticks: end,
        region: cue.region === 'upper_safe' || cue.region === 'center' ? cue.region : 'lower_safe',
        anim: 'karaoke',
        lines: [{ words: [first, ...rest] }] as CaptionCue['lines'],
      };
    }),
  },
  audio: {
    target_lufs: -14,
    true_peak_dbtp: -1,
    gain_curve: [
      { t_ticks: 0, gain_db: 0 },
      { t_ticks: 20 * SECOND, gain_db: -2 },
      { t_ticks: 42 * SECOND, gain_db: 0 },
    ],
  },
};

export const editorFilmstrip: Filmstrip = {
  artifactId: 'editor-preview',
  tiles: Array.from({ length: 15 }, (_, index) => ({
    file: `frame-${index}`,
    tTicks: (8 + index * 3) * SECOND,
  })),
};

export function editorFilmstripUrl(file: string): string {
  const index = Number(file.split('-')[1] ?? 0);
  const shift = (index % 5) * 3;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 90"><rect width="160" height="90" fill="#273440"/><rect x="0" y="56" width="160" height="34" fill="#17212c"/><circle cx="${48 + shift}" cy="34" r="16" fill="#b88464"/><path d="M14 90Q20 53 ${48 + shift} 53Q76 53 83 90" fill="#758f99"/><circle cx="118" cy="36" r="14" fill="#b58a70"/><path d="M88 90Q93 56 118 56Q144 56 156 90" fill="#7d707a"/><rect x="0" y="82" width="160" height="8" fill="#121b23"/></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}
