/** Synthetic review data, exclusively for the opt-in browser development page. */
import type { ClipRow } from '../src/results/model.js';
import type { ConnectionState, PreviewPlan } from '../src/daemon/client.js';
export const connection: ConnectionState = {
  status: 'connected',
  daemonVersion: 'preview',
  localLock: true,
  startedUnixMillis: 1,
};
export const titles = [
  'The best ideas start with a better question',
  'Why doing less can make your work better',
  'A small habit that changes how you listen',
  'What we get wrong about creative confidence',
  'Learning to leave room for the unexpected',
  'The advice I wish I had heard sooner',
];
export const rows: ClipRow[] = titles.map((headline, i) => ({
  candidateId: `preview-${i}`,
  rank: i + 1,
  displayScore: 0,
  band: i === 2 ? 'needs_review' : 'strong',
  bandLabel: i === 2 ? 'Needs review' : 'Ready to review',
  review: {
    status: 'accepted',
    route: 'local',
    summary:
      'A complete thought with a clear opening and a useful takeaway. The conversation moves naturally from the question to a concrete example.',
    reasons: [
      'The opening makes sense without the preceding conversation.',
      'The example gives the viewer something practical to take away.',
    ],
  },
  warnings: i === 2 ? ['Check the opening caption: one word has uncertain timing.'] : [],
  startTicks: i * 6 * 90_000,
  endTicks: (i * 6 + 42) * 90_000,
  durationSeconds: 42,
  headline,
  axes: [],
  penalties: [],
  boundary: null,
  decision: i === 1 ? 'approved' : null,
  docId: i === 1 ? 'preview-edit' : null,
  docJobId: null,
  latticeStarts: [i * 6 * 90_000],
  latticeEnds: [(i * 6 + 42) * 90_000],
  recommended: i < 4,
  proposer: 'Qwen',
  clusterId: null,
  hook: { text: headline + '.', atTicks: i * 6 * 90_000 },
  payoff: {
    text: 'Start with one small thing you can try tomorrow.',
    atTicks: (i * 6 + 36) * 90_000,
  },
  flagged: i === 2,
}));
export const plan: PreviewPlan = {
  revision: 4,
  rateNum: 30,
  rateDen: 1,
  frameCount: 1260,
  width: 1080,
  height: 1920,
  presentation: 'burn_in',
  crops: Array.from({ length: 1260 }, () => [656, 0, 608, 1080] as const),
  cues: [
    'The best ideas start',
    'with a better question.',
    'You have to leave room',
    'for a different answer.',
    'And sometimes that means',
    'slowing the conversation down.',
    'Listen to what they say,',
    'not what you expect.',
    'That changes everything.',
    'Start with one small thing.',
  ].map((text, i) => ({
    cueId: `cue_${i}`,
    firstFrame: i * 120,
    endFrame: (i + 1) * 120,
    region: 'lower_safe',
    karaoke: true,
    leadInCentis: 0,
    lines: [
      text.split(' ').map((word, j) => ({ text: word, wordId: `word_${i}_${j}`, holdCentis: 70 })),
    ],
  })),
  gain: [
    { frame: 0, gainDb: 0 },
    { frame: 600, gainDb: -2 },
    { frame: 1259, gainDb: 0 },
  ],
  segments: [
    {
      segmentId: 'segment',
      sourceFingerprint: 'preview',
      inTicks: 0,
      outTicks: 42 * 90_000,
      firstFrame: 0,
      endFrame: 1260,
      programStartTicks: 0,
    },
  ],
  sources: [
    { sourceFingerprint: 'preview', sourceId: 'preview', displayWidth: 1920, displayHeight: 1080 },
  ],
  proxies: [
    {
      sourceFingerprint: 'preview',
      artifactId: 'preview',
      file: 'preview.mp4',
      coverageStartTicks: 0,
      coverageEndTicks: 81 * 90_000,
      width: 1920,
      height: 1080,
      rateNum: 30,
      rateDen: 1,
    },
  ],
};
