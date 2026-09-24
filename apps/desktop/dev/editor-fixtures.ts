/**
 * Synthetic Editor state for the opt-in browser preview: the approved review
 * clip, with captions grouped from the same synthetic transcript the Inspector
 * reads, a drifting crop, and a dip in the volume. No daemon is involved.
 */
import type { EditIr } from '@clipmill/contracts';
import type { PreviewCue, PreviewPlan } from '../src/daemon/client.js';
import type { Filmstrip } from '../src/results/loader.js';
import { reviewRows, reviewTranscript } from './review-fixtures.js';

const SECOND = 90_000;
const RATE = 30;
type SavedCue = NonNullable<EditIr['captions']['cues']>[number];

const clip = reviewRows[1]!;
export const editorLabels = { project: 'The creative process · Episode 12', clip: clip.headline };

const inTicks = clip.startTicks;
const outTicks = clip.endTicks;
const duration = outTicks - inTicks;
const frameCount = Math.floor((duration * RATE) / SECOND);
const frameOf = (programTicks: number) =>
  Math.max(0, Math.min(frameCount, Math.floor((programTicks * RATE) / SECOND)));

/** Burned-in cues of three or four words, never across a sentence. */
function group(): { cues: PreviewCue[]; saved: SavedCue[] } {
  const words = reviewTranscript.words
    .map((word, index) => ({ ...word, index }))
    .filter(
      (word) =>
        (word.startTicks + word.endTicks) / 2 >= inTicks &&
        (word.startTicks + word.endTicks) / 2 < outTicks,
    );
  const runs: (typeof words)[] = [];
  let run: typeof words = [];
  for (const word of words) {
    run.push(word);
    if (
      run.length >= 4 ||
      (run.length >= 3 && /[.,?!]$/.test(word.text)) ||
      /[.?!]$/.test(word.text)
    ) {
      runs.push(run);
      run = [];
    }
  }
  if (run.length > 0) runs.push(run);
  const cues: PreviewCue[] = [];
  const saved: SavedCue[] = [];
  runs.forEach((items, at) => {
    const start = items[0]!.startTicks - inTicks;
    const next = runs[at + 1]?.[0];
    const end = Math.min(
      items.at(-1)!.endTicks - inTicks + 0.25 * SECOND,
      next ? next.startTicks - inTicks : duration,
    );
    cues.push({
      cueId: `cue_${at + 1}`,
      firstFrame: frameOf(start),
      endFrame: frameOf(end),
      region: 'lower_safe',
      karaoke: true,
      leadInCentis: 0,
      lines: [
        items.map((word) => ({
          text: word.text,
          wordId: `w${word.index}`,
          holdCentis: Math.round(((word.endTicks - word.startTicks) / SECOND) * 100) + 7,
        })),
      ],
    });
    saved.push({
      cue_id: `cue_${at + 1}`,
      start_ticks: start,
      end_ticks: end,
      region: 'lower_safe',
      anim: 'karaoke',
      lines: [
        {
          words: items.map((word) => ({
            word_id: `w${word.index}`,
            text: word.text,
            start_ticks: word.startTicks - inTicks,
            end_ticks: word.endTicks - inTicks,
          })) as SavedCue['lines'][number]['words'],
        },
      ],
    });
  });
  return { cues, saved };
}

const captions = group();

/** A crop that drifts a little and leans in, in the proxy's own pixels. */
function cropAt(programTicks: number): [number, number, number, number] {
  const seconds = programTicks / SECOND;
  const scale = seconds > 12 && seconds < 18 ? 0.84 : 0.94;
  const height = Math.round((720 * scale) / 2) * 2;
  const width = Math.round((height * 9) / 16 / 2) * 2;
  const centre = 0.5 + 0.03 * Math.sin(seconds / 4);
  const x = Math.max(0, Math.min(1280 - width, Math.round(centre * 1280 - width / 2)));
  const y = Math.max(0, Math.min(720 - height, Math.round(0.47 * 720 - height / 2)));
  return [x, y, width, height];
}

export const editorPlan: PreviewPlan = {
  revision: 4,
  rateNum: RATE,
  rateDen: 1,
  frameCount,
  width: 1080,
  height: 1920,
  presentation: 'burn_in',
  crops: Array.from({ length: frameCount }, (_, frame) => cropAt((frame * SECOND) / RATE)),
  cues: captions.cues,
  gain: [
    { frame: frameOf(duration * 0.45), gainDb: 0 },
    { frame: frameOf(duration * 0.55), gainDb: -4 },
    { frame: frameOf(duration * 0.7), gainDb: 0 },
  ],
  segments: [
    {
      segmentId: 'seg_1',
      sourceFingerprint: 'preview',
      inTicks,
      outTicks,
      programStartTicks: 0,
      firstFrame: 0,
      endFrame: frameCount,
    },
  ],
  sources: [
    { sourceFingerprint: 'preview', sourceId: 'preview', displayWidth: 1280, displayHeight: 720 },
  ],
  proxies: [
    {
      sourceFingerprint: 'preview',
      artifactId: 'preview',
      file: 'preview.mp4',
      coverageStartTicks: 0,
      coverageEndTicks: 480 * SECOND,
      width: 1280,
      height: 720,
      rateNum: RATE,
      rateDen: 1,
    },
  ],
  captionStyle: {
    styleRef: 'clipmill.captions.clean.v1',
    fontFamily: 'Inter',
    fontSize: 84,
    spoken: '#ffd65c',
    unspoken: '#ffffff',
    outline: '#000000',
    shadow: '#000000',
    outlineWidth: 5,
    shadowDepth: 2,
    bold: true,
    boxed: false,
    marginHorizontal: 90,
    marginVertical: 300,
  },
};

export const editorDocument: EditIr = {
  version: 'ir/1',
  timebase: { num: 1, den: 90_000 },
  video: {
    segments: [
      {
        segment_id: 'seg_1',
        source_fingerprint: 'preview',
        in_ticks: inTicks,
        out_ticks: outTicks,
        layout: {
          state: 'speaker_fill',
          crop_path: [0, 0.3, 0.62].map((share) => {
            const t = Math.round(duration * share);
            const [x, y, width, height] = cropAt(t);
            return { t_ticks: t, rect: { x, y, width, height } };
          }),
        },
      },
    ],
  },
  captions: {
    style_ref: 'clipmill.captions.clean.v1',
    cues: captions.saved,
    burn_in: captions.saved,
  },
  audio: {
    target_lufs: -14,
    true_peak_dbtp: -1,
    gain_curve: editorPlan.gain.map((point) => ({
      t_ticks: Math.round((point.frame * SECOND) / RATE),
      gain_db: point.gainDb,
    })),
  },
};

/** Stills every two seconds across the clip and the source around it. */
export const editorFilmstrip: Filmstrip = {
  artifactId: 'editor-preview',
  tiles: Array.from({ length: Math.ceil((duration + 20 * SECOND) / (2 * SECOND)) }, (_, index) => ({
    file: `frame-${index}`,
    tTicks: inTicks - 10 * SECOND + index * 2 * SECOND,
  })),
};

export function editorFilmstripUrl(file: string): string {
  const index = Number(file.split('-')[1] ?? 0);
  const shift = (index % 5) * 3;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 90"><rect width="160" height="90" fill="#3a3f38"/><rect x="0" y="58" width="160" height="32" fill="#2c302a"/><circle cx="${78 + shift}" cy="34" r="15" fill="#caa58a"/><path d="M50 90Q56 52 ${78 + shift} 52Q102 52 110 90" fill="#8c8272"/></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}
