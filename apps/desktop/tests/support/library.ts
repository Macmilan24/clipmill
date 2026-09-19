/**
 * A daemon that answers from memory.
 *
 * The Library's whole job is reading several daemon answers and turning them
 * into one view of a project, so the interesting failures are in the reading —
 * a job in a state nobody expected, an artifact that will not open, a probe that
 * reports no video. Those are cheap to stage here and expensive to stage against
 * a real daemon, which is the point of `ShellApi` being an interface.
 */
import type { SourceMap } from '@clipmill/contracts';
import { JobState, TaskState } from '@clipmill/contracts';

import type {
  ClipDecision,
  DirectClipInput,
  Document,
  Job,
  MediaArtifact,
  EditCommandJson,
  EditDocSummary,
  PreviewPlan,
  Progress,
  Project,
  Source,
  StorageStats,
  Task,
  ExportPlan,
  ExportRequest,
  LocalLock,
  Readiness,
  StageReadiness,
} from '../../src/daemon/client.js';
import type { ShellApi } from '../../src/daemon/api.js';

export const NOW = Date.UTC(2026, 6, 31, 12, 0, 0);

export function project(id: string, name: string, createdAgoMillis = 0): Project {
  return { projectId: id, name, createdUnixMillis: NOW - createdAgoMillis };
}

export function source(projectId: string, overrides: Partial<Source> = {}): Source {
  return {
    sourceId: `src_${projectId}`,
    projectId,
    absolutePath: `/Volumes/Media/${projectId}.mp4`,
    byteSize: 8_400_000_000,
    sourceFingerprint: 'sha256:aa',
    sourceMapArtifactId: `sha256:map-${projectId}`,
    createdUnixMillis: NOW,
    ...overrides,
  };
}

export function task(outputKind: string, state: TaskState, overrides: Partial<Task> = {}): Task {
  return {
    taskId: `task-${outputKind}`,
    kind: outputKind.replace(/\.v1$/, ''),
    outputKind,
    state,
    attempt: 1,
    maxAttempts: 3,
    waitReason: '',
    outputArtifactId: state === TaskState.SUCCEEDED ? `sha256:art-${outputKind}` : '',
    ...overrides,
  };
}

/** One stage of the daemon's readiness report, ready unless said otherwise. */
export function stageReadiness(
  stage: string,
  overrides: Partial<StageReadiness> = {},
): StageReadiness {
  const model = `${stage}-weights`;
  const base: StageReadiness = {
    stage,
    capability: stage,
    implementation: `${stage}-impl`,
    model,
    backend: 'mlx',
    modelPresent: true,
    missingFiles: [],
    workerPresent: true,
    ready: true,
    remedy: '',
  };
  const merged = { ...base, ...overrides };
  const ready = merged.modelPresent && merged.workerPresent;
  const remedy = merged.modelPresent
    ? merged.workerPresent
      ? ''
      : `No worker is connected that runs ${stage}: start the workers with \`just workers\`.`
    : `The model ${merged.model} is not installed: run \`tools/fetch-models.sh\` to fetch the pinned weights (${merged.missingFiles.length} file(s) missing).`;
  return { ...merged, ready, remedy: overrides.remedy ?? remedy };
}

/** A readiness report over the given stages, with the decoder in place. */
export function readiness(
  stages: readonly StageReadiness[],
  overrides: Partial<Readiness> = {},
): Readiness {
  const merged: Omit<Readiness, 'ready'> = {
    decoderPresent: true,
    decoderPath: '/Library/Application Support/dev.clipmill.ClipMill/tools/ffmpeg',
    stages,
    workers: stages.some((stage) => stage.workerPresent)
      ? [
          {
            workerId: 'worker-1',
            family: 'speech',
            capabilities: stages.filter((stage) => stage.workerPresent).map((stage) => stage.stage),
            backend: 'mlx',
            sinceUnixMillis: NOW - 30_000,
          },
        ]
      : [],
    ...overrides,
  };
  return {
    ...merged,
    ready:
      overrides.ready ?? (merged.decoderPresent && merged.stages.every((stage) => stage.ready)),
  };
}

export function job(projectId: string, state: JobState, tasks: readonly Task[] = []): Job {
  return {
    jobId: `job-${projectId}`,
    projectId,
    kind: 'analyze-source',
    state,
    createdUnixMillis: NOW - 3_600_000,
    updatedUnixMillis: NOW - 60_000,
    tasks,
    outputArtifactIds: [],
    failureClass: 0,
    failureDetail: '',
    sourceId: `src_${projectId}`,
  };
}

export function progress(unit: string, done: number, total: number): Progress {
  return { unit, done, total };
}

export function sourceMap(overrides: Partial<SourceMap> = {}): SourceMap {
  return {
    schema_version: 'clipmill.source_map.v1',
    source_fingerprint: 'sha256:aa',
    // 1:42:07 at 90000 ticks per second.
    container: { format: 'mov,mp4', duration_ticks: 6127 * 90_000 },
    streams: [
      {
        index: 0,
        kind: 'video',
        codec: 'h264',
        timebase: { num: 1, den: 90_000 },
        video: {
          coded_width: 1920,
          coded_height: 1080,
          display_width: 1920,
          display_height: 1080,
          frame_rate: { num: 30_000, den: 1001 },
        },
      },
    ],
    ...overrides,
  } as SourceMap;
}

export interface FakeWorld {
  readonly projects: readonly Project[];
  /** The path the fake file dialog returns, or null for "closed". */
  readonly chosenPath?: string | null;
  readonly jobs: Readonly<Record<string, readonly Job[]>>;
  readonly sources: Readonly<Record<string, readonly Source[]>>;
  readonly documents: Readonly<Record<string, Document>>;
  readonly media: Readonly<Record<string, MediaArtifact>>;
  readonly storage: StorageStats | null;
  /** What the Inspector asked the director for, in order. */
  readonly directed: DirectClipInput[];
  /** What was decided about each candidate. */
  readonly decisions: Map<string, ClipDecision>;
  /** The plan the player would apply, when the world has a document. */
  readonly plan?: PreviewPlan;
  readonly editDocs?: readonly EditDocSummary[];
  /** Every command the editor sent, in order. */
  readonly applied: EditCommandJson[];
  /** Every plan asked for, as (projectId, docId) pairs — which document opened. */
  readonly planned: Array<readonly [string, string]>;
  /** What the daemon answers when asked what an export would do. */
  readonly exportPlan?: ExportPlan;
  /** Every export request the screen sent, in order. */
  readonly exported: ExportRequest[];
  /** Every path the screen asked the host to reveal. */
  readonly revealed: string[];
  /** Every archive request, as (projectId, destination) pairs. */
  readonly archived: Array<readonly [string, string]>;
  readonly localLock?: LocalLock;
  /** What the daemon says an analysis would need, when the world says. */
  readonly readiness?: Readiness;
  /** The folder the fake dialog returns, or null for "closed". */
  readonly chosenFolder?: string | null;
}

export function emptyWorld(): FakeWorld {
  return {
    projects: [],
    jobs: {},
    sources: {},
    documents: {},
    media: {},
    storage: null,
    directed: [],
    decisions: new Map(),
    applied: [],
    planned: [],
    exported: [],
    revealed: [],
    archived: [],
  };
}

export function sourceMapDocument(artifactId: string, map = sourceMap()): Document {
  return { artifactId, kind: 'evidence.source_map.v1', json: JSON.stringify(map) };
}

export function filmstrip(artifactId: string, tiles: number): MediaArtifact {
  return {
    artifactId,
    kind: 'media.filmstrip.v1',
    files: Array.from({ length: tiles }, (_unused, index) => ({
      path: `strip_${String(index).padStart(5, '0')}.jpg`,
      bytes: 4096,
      mediaType: 'image/jpeg',
    })),
  };
}

export function fakeApi(world: FakeWorld): ShellApi {
  return {
    getSource: (sourceId) => {
      const foundSource = Object.values(world.sources)
        .flat()
        .find((item) => item.sourceId === sourceId);
      return foundSource
        ? Promise.resolve({
            source: foundSource,
            sourceMapJson: world.documents[foundSource.sourceMapArtifactId]?.json ?? '',
          })
        : Promise.reject(new Error('no such source'));
    },
    startYoutubeImport: () => Promise.reject(new Error('no YouTube import configured')),
    getYoutubeImport: () => Promise.reject(new Error('no such YouTube import')),
    listYoutubeImports: () => Promise.resolve([]),
    updateYoutubeImport: () => Promise.reject(new Error('no YouTube import configured')),
    submitExportBatch: () => Promise.reject(new Error('no batch response configured')),
    listExportBatches: () => Promise.resolve([]),
    updateExportBatchItem: () => Promise.reject(new Error('no batch response configured')),
    listProjects: () => Promise.resolve(world.projects),
    listJobs: (projectId) => Promise.resolve(world.jobs[projectId] ?? []),
    fetchJob: (jobId) => {
      const found = Object.values(world.jobs)
        .flat()
        .find((candidate) => candidate.jobId === jobId);
      return found === undefined
        ? Promise.reject(new Error('no such job'))
        : Promise.resolve(found);
    },
    listSources: (projectId) => Promise.resolve(world.sources[projectId] ?? []),
    readDocument: (_projectId, artifactId) => {
      const document = world.documents[artifactId];
      return document === undefined
        ? Promise.reject(new Error('this project published no such artifact'))
        : Promise.resolve(document);
    },
    resolveMedia: (_projectId, artifactId) => {
      const media = world.media[artifactId];
      return media === undefined
        ? Promise.reject(new Error('this project published no such artifact'))
        : Promise.resolve(media);
    },
    mediaUrl: (projectId, artifactId, file) =>
      `clipmill-media://localhost/${projectId}/${artifactId}/${file}`,
    fetchStorageStats: () =>
      world.storage === null
        ? Promise.reject(new Error('this daemon measures no storage'))
        : Promise.resolve(world.storage),
    createProject: (name) => Promise.resolve(`prj_${name}`),
    chooseSourceFile: () => Promise.resolve(world.chosenPath ?? null),
    registerSource: (projectId, absolutePath) =>
      Promise.resolve({
        source: source(projectId, { absolutePath }),
        observationCacheHit: false,
        sourceMapJson: JSON.stringify(sourceMap()),
      }),
    submitAnalyze: (projectId) => Promise.resolve(job(projectId, JobState.PLANNED, [])),
    // The director and the decisions are the daemon's, and a fake that
    // invented answers for them would let a screen test pass while the real
    // call was refused. These record what was asked and nothing else.
    directClip: (request) => {
      world.directed.push(request);
      // The one rule of the daemon's worth mirroring: a clip that already has
      // a document gets it back unless a variation was asked for. A screen
      // that navigated to a freshly minted id when the daemon would have
      // reopened an existing one would be tested against a daemon that does
      // not exist.
      const existing = request.variation
        ? undefined
        : (world.editDocs ?? []).find(
            (document) =>
              document.projectId === request.projectId &&
              document.sourceId === request.sourceId &&
              document.candidateId === request.candidateId,
          );
      return Promise.resolve({
        docId: existing?.docId ?? 'edt_00000000000000000000000000',
        projectId: request.projectId,
        sourceId: request.sourceId,
        candidateId: request.candidateId,
        jobId: existing?.jobId ?? request.jobId ?? '',
        revision: existing?.revision ?? 0,
        documentJson: '{}',
        startTicks: request.startTicks ?? 0,
        endTicks: request.endTicks ?? 0,
        decisions: [],
        reopened: existing !== undefined,
      });
    },
    solveCropPath: () =>
      Promise.resolve({
        keyframes: [],
        fit: true,
        fitReason: 'nothing looked for faces',
        containment: 0,
      }),
    listEditDocs: (projectId) =>
      Promise.resolve(
        (world.editDocs ?? []).filter((document) => document.projectId === projectId),
      ),
    applyEditCommand: (docId, expectedRevision, command) => {
      world.applied.push(command);
      // The inverse a real daemon computes depends on the document; a fake
      // that invented one would let a test pass while undo was broken, so this
      // returns the command itself and tests assert on what was sent.
      return Promise.resolve({
        docId,
        revision: expectedRevision + 1,
        inverseCommandJson: JSON.stringify(command),
      });
    },
    previewPlan: (projectId, docId) => {
      world.planned.push([projectId, docId]);
      return world.plan
        ? Promise.resolve(world.plan)
        : Promise.reject(new Error('this project has no such document'));
    },
    setClipDecision: (_projectId, _sourceId, candidateId, decision) => {
      world.decisions.set(candidateId, decision);
      return Promise.resolve({ candidateId, decision, decidedUnixMillis: 0 });
    },
    listClipDecisions: () =>
      Promise.resolve(
        [...world.decisions].map(([candidateId, decision]) => ({
          candidateId,
          decision,
          decidedUnixMillis: 0,
        })),
      ),
    // The strip is the daemon's and so is the naming. A fake that resolved a
    // pattern here would let a screen test pass while the real preview showed
    // something else, which is the exact divergence the daemon-side resolver
    // exists to prevent — so this answers only what the world was given.
    planExport: (request) => {
      world.exported.push(request);
      return world.exportPlan === undefined
        ? Promise.reject(new Error('this daemon plans no exports'))
        : Promise.resolve(world.exportPlan);
    },
    exportClip: (request) => {
      world.exported.push(request);
      // The daemon refuses to export a revision other than the one reviewed.
      if (
        request.expectedRevision !== undefined &&
        world.plan !== undefined &&
        request.expectedRevision !== world.plan.revision
      ) {
        return Promise.reject(
          new Error(
            `the document moved since it was reviewed: revision ${request.expectedRevision} was approved, it is now at revision ${world.plan.revision}`,
          ),
        );
      }
      return Promise.resolve({
        jobId: 'job_export',
        revision: request.expectedRevision ?? world.plan?.revision ?? 0,
        irArtifactId: 'sha256:ir-snapshot',
        destinationDir: request.destinationDir,
      });
    },
    revealPath: (path) => {
      world.revealed.push(path);
      return Promise.resolve();
    },
    exportArchive: (projectId, destinationDir) => {
      world.archived.push([projectId, destinationDir]);
      return Promise.resolve({
        path: `${destinationDir}/project.clipmill-archive.zip`,
        sha256: 'a'.repeat(64),
        bytes: 2048,
        entryCount: 3,
      });
    },
    fetchLocalLock: () =>
      world.localLock === undefined
        ? Promise.reject(new Error('this daemon reports no policy'))
        : Promise.resolve(world.localLock),
    chooseExportFolder: () => Promise.resolve(world.chosenFolder ?? null),
    fetchReadiness: () =>
      world.readiness === undefined
        ? Promise.reject(new Error('this daemon reports no readiness'))
        : Promise.resolve(world.readiness),
  };
}
