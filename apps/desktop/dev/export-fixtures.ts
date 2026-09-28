/**
 * The Export screen's states, for the preview: `?export=` names one.
 *
 * Each is what the screen would be handed by the daemon in that state, so the
 * page can be looked at blocked, advised, exporting, exported and failed
 * without an export being run.
 */
import type { ExportFinding, ExportPlan } from '../src/daemon/client.js';
import type { Delivery } from '../src/export/delivery.js';

const FILES = [
  '01-a-better-question.mp4',
  '01-a-better-question.srt',
  '01-a-better-question.vtt',
  '01-a-better-question.jpg',
  '01-a-better-question.metadata.json',
  '01-a-better-question.render-manifest.json',
  '01-a-better-question.sha256',
];

export function exportPlanOf(
  revision: number,
  findings: readonly ExportFinding[] = [],
): ExportPlan {
  return {
    passes: findings.every((finding) => finding.severity !== 'blocking'),
    findings: [...findings],
    stem: '01-a-better-question',
    fileNames: FILES,
    revision,
    estimatedBytes: 42_000_000,
    availableBytes: 75_000_000_000,
  };
}

function delivery(state: 'exporting' | 'exported' | 'failed', revision: number): Delivery {
  const folder = '/Users/demo/Movies/ClipMill';
  return {
    revision,
    destinationDir: folder,
    settled: state !== 'exporting',
    files:
      state === 'exported'
        ? FILES.map((name, index) => ({
            name,
            path: `${folder}/${name}`,
            role: index === 0 ? 'video' : 'sidecar',
            bytes: index === 0 ? 41_200_000 : 2_400 * (index + 1),
          }))
        : null,
    failure:
      state === 'failed'
        ? 'render task failed: ffmpeg exited 1: ran out of disk at frame 411'
        : null,
    interruption: null,
    stages: [
      {
        kind: 'render',
        label: 'Render',
        state: state === 'exporting' ? 'running' : state === 'failed' ? 'failed' : 'done',
        progress: state === 'exporting' ? { unit: 'export.rendering', done: 42, total: 100 } : null,
        waitReason: '',
      },
      {
        kind: 'deliver',
        label: 'Deliver',
        state: state === 'exported' ? 'done' : 'waiting',
        progress: null,
        waitReason: 'waiting: dependencies',
      },
    ],
  };
}

const TOO_BRIEF: ExportFinding = {
  code: 'captions.too_brief',
  severity: 'blocking',
  detail: 'Subtitle 4 at 0:42.51 — “So” is on screen for 0.21s; the minimum is 0.33s.',
  cueId: 'cue_4',
};

const HOT: readonly ExportFinding[] = [
  {
    code: 'captions.reading_rate',
    severity: 'advisory',
    detail: 'Subtitle 7 at 0:18.20 asks for 24.6 characters a second — confirmed as read.',
    cueId: 'cue_7',
  },
  {
    code: 'captions.reading_rate',
    severity: 'advisory',
    detail: 'Subtitle 12 at 0:31.04 asks for 22.1 characters a second — confirmed as read.',
    cueId: 'cue_12',
  },
];

export interface ExportScenario {
  readonly plan: ExportPlan | null;
  readonly hotCaptions: readonly ExportFinding[];
  readonly delivery: Delivery | null;
  readonly rightsGateNeeded: boolean;
  readonly destination?: string;
  readonly auditioned: boolean;
}

export function exportScenario(name: string | null, revision: number): ExportScenario {
  const quiet = { hotCaptions: [], delivery: null, rightsGateNeeded: false, auditioned: false };
  switch (name) {
    case 'blocked':
      return {
        ...quiet,
        plan: exportPlanOf(revision, [
          TOO_BRIEF,
          {
            code: 'source.missing',
            severity: 'blocking',
            detail:
              'The recording creative-process-ep12.mp4 is no longer where it was imported from.',
          },
        ]),
      };
    case 'advice':
      return {
        ...quiet,
        plan: exportPlanOf(revision, [
          { ...TOO_BRIEF, severity: 'advisory' },
          {
            code: 'disk.low',
            severity: 'advisory',
            detail: 'This export leaves 3.1 GB free on the drive.',
          },
        ]),
      };
    case 'hot':
      return { ...quiet, plan: exportPlanOf(revision, HOT), hotCaptions: HOT };
    case 'gate':
      return {
        ...quiet,
        rightsGateNeeded: true,
        plan: exportPlanOf(revision, [
          {
            code: 'rights.gate_not_passed',
            severity: 'blocking',
            detail:
              'Clips longer than a minute need your confirmation that the permission covers them.',
          },
        ]),
      };
    case 'nofolder':
      return { ...quiet, plan: null, destination: '' };
    case 'exporting':
    case 'exported':
    case 'failed':
      return {
        ...quiet,
        plan: exportPlanOf(revision),
        delivery: delivery(name, revision),
        auditioned: name === 'exported',
      };
    default:
      return { ...quiet, plan: exportPlanOf(revision) };
  }
}
