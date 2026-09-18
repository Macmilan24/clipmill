/**
 * Two projects, and the older one is the one with clips.
 *
 * This is the shape every "which clip" bug hid in. With one project the newest
 * project is the right project, the newest document is the right document,
 * and every screen that guessed guessed right. So the world here holds a newer
 * project that has been analyzed too, with its own proxy and its own document,
 * and the assertions are about the older one: if a screen falls back to
 * "newest", it opens the wrong recording and the test sees it.
 */
import type { DiscoveryCandidates, RankingSet } from '@clipmill/contracts';
import { JobState, TaskState } from '@clipmill/contracts';

import type { EditDocSummary, PreviewPlan } from '../../src/daemon/client.js';
import { NOW, emptyWorld, project, source, task, job, type FakeWorld } from './library.js';
import { FINGERPRINT, TICKS, mapping } from './plan.js';

export const OLD = 'p_old';
export const NEW = 'p_new';
export const OLD_SOURCE = `src_${OLD}`;
export const NEW_SOURCE = `src_${NEW}`;
export const OLD_JOB = `job-${OLD}`;
export const NEW_JOB = `job-${NEW}`;
export const OLD_PROXY = 'sha256:proxy-old';
export const NEW_PROXY = 'sha256:proxy-new';
export const OLD_DOC = 'edt_0000000000000000000000OLD1';
export const NEW_DOC = 'edt_0000000000000000000000NEW1';
export const CANDIDATE = 'cand_0000000000000001';
export const OTHER_CANDIDATE = 'cand_0000000000000002';

const OLD_FINGERPRINT = `sha256:${'aa'.repeat(32)}`;
const NEW_FINGERPRINT = `sha256:${'bb'.repeat(32)}`;

function ranking(fingerprint: string): RankingSet {
  return {
    schema_version: 'clipmill.ranking.set.v1',
    source_fingerprint: fingerprint,
    inputs: {
      candidates_artifact_id: fingerprint,
      index_artifact_id: fingerprint,
      transcript_artifact_id: fingerprint,
    },
    producer: { stage: 'rank-candidates', implementation: 'test@1' },
    rubric: { scorer: 'test', boundary: 'test', selector: 'test' },
    requested: { count: 2, diversity: 0.3 },
    cohort: [
      {
        candidate_id: CANDIDATE,
        rank: 1,
        display_score: 92,
        score: 0.92,
        factors: [{ name: 'hook', available: true, value: 0.9, weight: 0.2, evidence: [] }],
        penalties: [],
        uncertainty: { value: 0.1, band: 'strong', warnings: [] },
        boundary: {
          // Several minutes in: a preview that ignored the interval would
          // start at zero, and a test at zero could not tell.
          chosen: { start_ticks: 600 * 90_000, end_ticks: 630 * 90_000 },
          score: 0.9,
          terms: [{ name: 'hook_weight', value: 0.4, weight: 1 }],
          alternative: {
            interval: { start_ticks: 601 * 90_000, end_ticks: 630 * 90_000 },
            score: 0.8,
          },
        },
        cluster_id: 'clus_0000000000000001',
      },
      {
        candidate_id: OTHER_CANDIDATE,
        rank: 2,
        display_score: 70,
        score: 0.7,
        factors: [{ name: 'hook', available: true, value: 0.7, weight: 0.2, evidence: [] }],
        penalties: [],
        uncertainty: { value: 0.3, band: 'promising', warnings: [] },
        boundary: {
          chosen: { start_ticks: 900 * 90_000, end_ticks: 930 * 90_000 },
          score: 0.7,
          terms: [{ name: 'hook_weight', value: 0.3, weight: 1 }],
        },
        cluster_id: 'clus_0000000000000002',
      },
    ],
    selected: [CANDIDATE, OTHER_CANDIDATE],
    shortfall: [],
    filtered: [],
  } as unknown as RankingSet;
}

function candidates(): DiscoveryCandidates {
  return {
    candidates: [
      {
        id: CANDIDATE,
        boundary_lattice: {
          starts: [600 * 90_000, 601 * 90_000],
          ends: [630 * 90_000],
          phi_rejects: [],
        },
      },
      {
        id: OTHER_CANDIDATE,
        boundary_lattice: { starts: [900 * 90_000], ends: [930 * 90_000], phi_rejects: [] },
      },
    ],
  } as unknown as DiscoveryCandidates;
}

/**
 * A plan any document can answer with: one second at 30 fps, one cue, cut
 * from ten minutes into the older recording — far enough in that a player
 * seeking to zero is visibly wrong.
 */
export function plan(): PreviewPlan {
  const program = { frameCount: 30, rateNum: 30, rateDen: 1 };
  return {
    ...mapping(program, 600, {
      sources: [
        {
          sourceFingerprint: FINGERPRINT,
          sourceId: OLD_SOURCE,
          displayWidth: 1920,
          displayHeight: 1080,
        },
      ],
      proxies: [
        {
          sourceFingerprint: FINGERPRINT,
          artifactId: OLD_PROXY,
          file: 'proxy.mp4',
          coverageStartTicks: 0,
          coverageEndTicks: 3600 * TICKS,
          width: 1280,
          height: 720,
          rateNum: 30,
          rateDen: 1,
        },
      ],
    }),
    revision: 0,
    rateNum: 30,
    rateDen: 1,
    frameCount: 30,
    crops: Array.from({ length: 30 }, () => null),
    cues: [
      {
        cueId: 'cue_1',
        firstFrame: 0,
        endFrame: 30,
        region: 'lower_safe',
        karaoke: true,
        leadInCentis: 0,
        lines: [
          [
            { text: 'Charging', holdCentis: 50, wordId: 'w1' },
            { text: 'less', holdCentis: 50, wordId: 'w2' },
          ],
        ],
      },
    ],
    gain: [],
    width: 1080,
    height: 1920,
  };
}

export function document(
  projectId: string,
  docId: string,
  candidateId: string,
  overrides: Partial<EditDocSummary> = {},
): EditDocSummary {
  return {
    docId,
    projectId,
    sourceId: `src_${projectId}`,
    candidateId,
    jobId: `job-${projectId}`,
    revision: 0,
    createdUnixMillis: NOW - 1_000,
    updatedUnixMillis: NOW - 1_000,
    ...overrides,
  };
}

function analyzed(projectId: string, fingerprint: string, proxy: string) {
  const rankingId = `sha256:ranking-${projectId}`;
  const candidatesId = `sha256:candidates-${projectId}`;
  return {
    job: job(projectId, JobState.SUCCEEDED, [
      task('media.proxy.v1', TaskState.SUCCEEDED, { outputArtifactId: proxy }),
      task('discovery.candidates.v1', TaskState.SUCCEEDED, { outputArtifactId: candidatesId }),
      task('ranking.set.v1', TaskState.SUCCEEDED, { outputArtifactId: rankingId }),
    ]),
    documents: {
      [rankingId]: {
        artifactId: rankingId,
        kind: 'ranking.set.v1',
        json: JSON.stringify(ranking(fingerprint)),
      },
      [candidatesId]: {
        artifactId: candidatesId,
        kind: 'discovery.candidates.v1',
        json: JSON.stringify(candidates()),
      },
    },
  };
}

/**
 * The world: both projects analyzed, both with a proxy, only the newer one
 * with a document — unless a test adds one to the older.
 */
export function twoProjects(overrides: Partial<FakeWorld> = {}): FakeWorld {
  const old = analyzed(OLD, OLD_FINGERPRINT, OLD_PROXY);
  const fresh = analyzed(NEW, NEW_FINGERPRINT, NEW_PROXY);
  return {
    ...emptyWorld(),
    // Newest first, as the daemon lists them.
    projects: [project(NEW, 'Dogfood episode'), project(OLD, 'CUDA kernels', 3_600_000)],
    sources: {
      [OLD]: [source(OLD, { sourceFingerprint: OLD_FINGERPRINT })],
      [NEW]: [source(NEW, { sourceFingerprint: NEW_FINGERPRINT })],
    },
    jobs: { [OLD]: [old.job], [NEW]: [fresh.job] },
    documents: { ...old.documents, ...fresh.documents },
    editDocs: [document(NEW, NEW_DOC, CANDIDATE)],
    plan: plan(),
    ...overrides,
  };
}
