/**
 * Gathering what the Results board and the Clip Inspector show.
 *
 * The artifact addresses come off the analyze job's own tasks rather than from
 * a lookup: a task that succeeded names what it published, so the job is the
 * index. That also means a board can only show a run that finished, which is
 * the honest state — a half-finished analysis has no ranking to rank.
 *
 * Nothing here interprets. It fetches and assembles; what any of it means is
 * `model.ts`, which is pure and tested without a window.
 */
import type { DiscoveryCandidates, IndexTranscript, RankingSet } from '@clipmill/contracts';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { ClipDecisionRecord, Job, Source } from '../daemon/client.js';
import { publishedArtifact } from '../library/model.js';
import { type ClipRow, type Summary, clipRows, summarize } from './model.js';

export const RANKING_KIND = 'ranking.set.v1';
export const CANDIDATES_KIND = 'discovery.candidates.v1';
export const INDEX_KIND = 'index.transcript.v1';
export const PROXY_KIND = 'media.proxy.v1';
export const FACES_KIND = 'vision.face_track.v1';

/** Why a board has nothing to show, in words a person can act on. */
export type ResultsProblem =
  | { readonly kind: 'no-source' }
  | { readonly kind: 'not-analyzed' }
  | { readonly kind: 'unreadable'; readonly detail: string };

export interface ResultsSnapshot {
  readonly source: Source | null;
  readonly rows: readonly ClipRow[];
  readonly summary: Summary | null;
  /** The proxy this source's clips are previewed from, when it has one. */
  readonly proxyArtifactId: string | null;
  /** The face tracks a crop path is solved from. Null when nobody looked. */
  readonly faceTrackArtifactId: string | null;
  readonly problem: ResultsProblem | null;
}

export const EMPTY_SNAPSHOT: ResultsSnapshot = {
  source: null,
  rows: [],
  summary: null,
  proxyArtifactId: null,
  faceTrackArtifactId: null,
  problem: { kind: 'no-source' },
};

/**
 * The newest job that actually published a ranking.
 *
 * A job does not carry which source it ran over, so this cannot be filtered by
 * one. What it can do is refuse to guess: a ranking is only shown when its own
 * document names the same recording, checked by the caller against the source's
 * fingerprint.
 */
function analyzed(jobs: readonly Job[]): Job | null {
  return jobs.filter((job) => publishedArtifact(job, RANKING_KIND) !== null).at(-1) ?? null;
}

export class ResultsLoader {
  constructor(private readonly api: ShellApi = daemonApi) {}

  async load(projectId: string, sourceId: string | null): Promise<ResultsSnapshot> {
    const [jobs, sources] = await Promise.all([
      this.api.listJobs(projectId),
      this.api.listSources(projectId),
    ]);
    const source = sourceId
      ? (sources.find((candidate) => candidate.sourceId === sourceId) ?? null)
      : (sources.at(-1) ?? null);
    if (!source) {
      return EMPTY_SNAPSHOT;
    }

    const job = analyzed(jobs);
    const ranking = publishedArtifact(job, RANKING_KIND);
    const candidates = publishedArtifact(job, CANDIDATES_KIND);
    if (!job || !ranking || !candidates) {
      return { ...EMPTY_SNAPSHOT, source, problem: { kind: 'not-analyzed' } };
    }

    try {
      const [rankingDoc, candidateDoc, indexDoc, decisions] = await Promise.all([
        this.api.readDocument(projectId, ranking),
        this.api.readDocument(projectId, candidates),
        this.readOptional(projectId, publishedArtifact(job, INDEX_KIND)),
        this.api.listClipDecisions(projectId, source.sourceId).catch(() => []),
      ]);
      const rankingSet = JSON.parse(rankingDoc.json) as RankingSet;
      // The job did not say which recording it ranked, so the document does.
      // Showing another source's clips under this one's name would be worse
      // than showing none.
      if (rankingSet.source_fingerprint !== source.sourceFingerprint) {
        return { ...EMPTY_SNAPSHOT, source, problem: { kind: 'not-analyzed' } };
      }
      const rows = clipRows(
        rankingSet,
        JSON.parse(candidateDoc.json) as DiscoveryCandidates,
        indexDoc ? (JSON.parse(indexDoc) as IndexTranscript) : null,
        decisions as readonly ClipDecisionRecord[],
      );
      return {
        source,
        rows,
        summary: summarize(rankingSet),
        proxyArtifactId: publishedArtifact(job, PROXY_KIND),
        faceTrackArtifactId: publishedArtifact(job, FACES_KIND),
        problem: null,
      };
    } catch (error) {
      return {
        ...EMPTY_SNAPSHOT,
        source,
        problem: { kind: 'unreadable', detail: (error as Error).message },
      };
    }
  }

  /**
   * A document the board is better with and works without.
   *
   * Swallowed rather than propagated: a board that showed nothing because the
   * evidence index failed to verify would be hiding a ranking that verified
   * perfectly well.
   */
  private async readOptional(projectId: string, artifactId: string | null): Promise<string | null> {
    if (!artifactId) {
      return null;
    }
    try {
      return (await this.api.readDocument(projectId, artifactId)).json;
    } catch {
      return null;
    }
  }
}
