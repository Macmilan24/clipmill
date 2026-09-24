/**
 * Load Results and Inspector artifacts from the completed analysis job's tasks.
 * Successful tasks supply published artifact addresses. Assemble documents here;
 * derive presentation values in `model.ts`.
 */
import type {
  DiscoveryCandidates,
  IndexTranscript,
  MediaAudioPeaks,
  MediaFilmstrip,
  RankingSet,
  SourceMap,
  SpeechTranscript,
} from '@clipmill/contracts';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { newest } from '../daemon/ordering.js';
import type { ClipDecisionRecord, Job, Source } from '../daemon/client.js';
import { publishedArtifact } from '../library/model.js';
import { type ClipRow, type Summary, clipRows, summarize } from './model.js';
import { type Transcript, readTranscript } from './transcript.js';

export const RANKING_KIND = 'ranking.set.v1';
export const CANDIDATES_KIND = 'discovery.candidates.v1';
export const INDEX_KIND = 'index.transcript.v1';
export const PROXY_KIND = 'media.proxy.v1';
export const FACES_KIND = 'vision.face_track.v1';
export const FILMSTRIP_KIND = 'media.filmstrip.v1';
export const PEAKS_KIND = 'media.audio_peaks.v1';
export const TRANSCRIPT_KIND = 'speech.transcript.v1';

/** Why a board has nothing to show, in words a person can act on. */
export type ResultsProblem =
  | { readonly kind: 'no-source' }
  | { readonly kind: 'not-analyzed' }
  | { readonly kind: 'unreadable'; readonly detail: string };

/** The analysis these clips came out of, by the facts the job records. */
export interface RunInfo {
  readonly jobId: string;
  readonly state: Job['state'];
  readonly completedUnixMillis: number;
}

/**
 * The filmstrip, ready to be asked for the frame nearest a moment.
 *
 * Tiles carry their own position, so the still for a clip is the tile the
 * ingest actually cut closest to its first frame — a real frame of the real
 * recording, and never a placeholder standing in for one.
 */
export interface Filmstrip {
  readonly artifactId: string;
  readonly tiles: readonly { readonly file: string; readonly tTicks: number }[];
}

/** A clip as a list outside Results names it. */
export interface ClipName {
  readonly title: string;
  readonly thumbnail: string | null;
}

/** The loudness contour, one min/max pair per bucket, for drawing a waveform. */
export interface Peaks {
  readonly bucketTicks: number;
  readonly values: readonly (readonly [number, number])[];
}

export interface ResultsSnapshot {
  readonly source: Source | null;
  readonly rows: readonly ClipRow[];
  readonly summary: Summary | null;
  readonly sourceDurationTicks?: number | null;
  /** The proxy this source's clips are previewed from, when it has one. */
  readonly proxyArtifactId: string | null;
  /** The face tracks a crop path is solved from. Null when nobody looked. */
  readonly faceTrackArtifactId: string | null;
  readonly run: RunInfo | null;
  readonly filmstrip: Filmstrip | null;
  readonly peaks: Peaks | null;
  /**
   * How long the search was asked to make a clip. A reviewer may cut outside
   * it (R63); the Inspector says so rather than stopping them.
   */
  readonly durationTarget?: { readonly minTicks: number; readonly maxTicks: number } | null;
  /**
   * The transcript the index was read from, which the Inspector loads on its
   * own: the board never draws a word, and a two-hour transcript is too much
   * to read for a grid of cards.
   */
  readonly transcriptArtifactId?: string | null;
  /** The index the rows were joined from, kept for the transcript's sentences. */
  readonly index?: IndexTranscript | null;
  readonly problem: ResultsProblem | null;
}

export const EMPTY_SNAPSHOT: ResultsSnapshot = {
  source: null,
  rows: [],
  summary: null,
  sourceDurationTicks: null,
  proxyArtifactId: null,
  faceTrackArtifactId: null,
  run: null,
  filmstrip: null,
  peaks: null,
  durationTarget: null,
  transcriptArtifactId: null,
  index: null,
  problem: { kind: 'no-source' },
};

/**
 * Prefer the route's run so re-analysis cannot change an open candidate's identity.
 * Otherwise use the newest ranking-producing job for this source. Legacy jobs
 * without source IDs remain eligible; the caller checks the ranking fingerprint.
 */
function analyzed(jobs: readonly Job[], sourceId: string, jobId: string | null): Job | null {
  if (jobId) {
    return jobs.find((job) => job.jobId === jobId) ?? null;
  }
  return newest(
    jobs.filter(
      (job) =>
        publishedArtifact(job, RANKING_KIND) !== null &&
        (job.sourceId === '' || job.sourceId === sourceId),
    ),
  );
}

export class ResultsLoader {
  constructor(private readonly api: ShellApi = daemonApi) {}

  async load(
    projectId: string,
    sourceId: string | null,
    jobId: string | null = null,
  ): Promise<ResultsSnapshot> {
    const [jobs, sources] = await Promise.all([
      this.api.listJobs(projectId),
      this.api.listSources(projectId),
    ]);
    const source = sourceId
      ? (sources.find((candidate) => candidate.sourceId === sourceId) ?? null)
      : newest(sources);
    if (!source) {
      return EMPTY_SNAPSHOT;
    }

    const job = analyzed(jobs, source.sourceId, jobId);
    const ranking = publishedArtifact(job, RANKING_KIND);
    const candidates = publishedArtifact(job, CANDIDATES_KIND);
    if (!job || !ranking || !candidates) {
      return { ...EMPTY_SNAPSHOT, source, problem: { kind: 'not-analyzed' } };
    }

    try {
      const filmstripId = publishedArtifact(job, FILMSTRIP_KIND);
      const [
        rankingDoc,
        candidateDoc,
        indexDoc,
        decisions,
        filmstripDoc,
        peaksDoc,
        documents,
        sourceMapDoc,
      ] = await Promise.all([
        this.api.readDocument(projectId, ranking),
        this.api.readDocument(projectId, candidates),
        this.readOptional(projectId, publishedArtifact(job, INDEX_KIND)),
        this.api.listClipDecisions(projectId, source.sourceId).catch(() => []),
        this.readOptional(projectId, filmstripId),
        this.readOptional(projectId, publishedArtifact(job, PEAKS_KIND)),
        // Which clips already have an edit. A failure here loses a badge,
        // not the board, so it is read like the optional documents.
        this.api.listEditDocs(projectId).catch(() => []),
        this.readOptional(
          projectId,
          publishedArtifact(job, 'evidence.source_map.v1') ?? (source.sourceMapArtifactId || null),
        ),
      ]);
      const rankingSet = JSON.parse(rankingDoc.json) as RankingSet;
      // The ranking names the recording it ranked. Showing another source's
      // clips under this one's name would be worse than showing none.
      if (rankingSet.source_fingerprint !== source.sourceFingerprint) {
        return { ...EMPTY_SNAPSHOT, source, problem: { kind: 'not-analyzed' } };
      }
      const candidateSet = JSON.parse(candidateDoc.json) as DiscoveryCandidates;
      const index = indexDoc ? (JSON.parse(indexDoc) as IndexTranscript) : null;
      const rows = clipRows(
        rankingSet,
        candidateSet,
        index,
        decisions as readonly ClipDecisionRecord[],
        documents.filter((document) => document.sourceId === source.sourceId),
      );
      const filmstrip: MediaFilmstrip | null = filmstripDoc ? JSON.parse(filmstripDoc) : null;
      const peaks: MediaAudioPeaks | null = peaksDoc ? JSON.parse(peaksDoc) : null;
      let sourceDurationTicks: number | null = null;
      if (sourceMapDoc) {
        try {
          const map = JSON.parse(sourceMapDoc) as SourceMap;
          if (
            map.schema_version === 'clipmill.source_map.v1' &&
            map.source_fingerprint === source.sourceFingerprint &&
            Number.isSafeInteger(map.container.duration_ticks) &&
            map.container.duration_ticks > 0
          )
            sourceDurationTicks = map.container.duration_ticks;
        } catch {
          /* Timing is unavailable; never infer the recording length from candidate spans. */
        }
      }
      return {
        source,
        rows,
        sourceDurationTicks,
        summary: summarize(rankingSet),
        proxyArtifactId: publishedArtifact(job, PROXY_KIND),
        faceTrackArtifactId: publishedArtifact(job, FACES_KIND),
        run: { jobId: job.jobId, state: job.state, completedUnixMillis: job.updatedUnixMillis },
        filmstrip:
          filmstrip &&
          filmstripId &&
          filmstrip.schema_version === 'clipmill.media.filmstrip.v1' &&
          filmstrip.source_fingerprint === source.sourceFingerprint
            ? {
                artifactId: filmstripId,
                tiles: filmstrip.tiles.map((tile) => ({ file: tile.file, tTicks: tile.t_ticks })),
              }
            : null,
        peaks: peaks
          ? {
              bucketTicks: peaks.bucket_ticks,
              values: peaks.peaks.map((bucket) => [bucket.min, bucket.max] as const),
            }
          : null,
        durationTarget: candidateSet.duration_target
          ? {
              minTicks: candidateSet.duration_target.min_ticks,
              maxTicks: candidateSet.duration_target.max_ticks,
            }
          : null,
        // The index names the transcript its word positions refer to, which
        // is the one to read; the job's own is the fallback for a run that
        // published no index.
        transcriptArtifactId:
          index?.inputs?.transcript_artifact_id ?? publishedArtifact(job, TRANSCRIPT_KIND),
        index,
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
   * The words of the recording a snapshot describes, for the Inspector.
   *
   * Null when the run published no transcript, or published one for a
   * different recording — words from another source would be read aloud
   * against this one's picture, and no transcript is the honest answer.
   */
  /**
   * What each clip of a run is called, and a frame of it: for the lists that
   * name clips outside Results, which would otherwise have only a file name.
   * Reads the ranking, the candidates, the index and the filmstrip, nothing
   * else, and answers with an empty map for a run that ranked nothing.
   */
  async clipNames(
    projectId: string,
    sourceId: string,
    jobId: string | null,
  ): Promise<ReadonlyMap<string, ClipName>> {
    const job = analyzed(await this.api.listJobs(projectId), sourceId, jobId);
    const ranking = publishedArtifact(job, RANKING_KIND);
    const candidates = publishedArtifact(job, CANDIDATES_KIND);
    if (!job || !ranking || !candidates) return new Map();
    const filmstripId = publishedArtifact(job, FILMSTRIP_KIND);
    const [rankingDoc, candidateDoc, indexDoc, filmstripDoc] = await Promise.all([
      this.api.readDocument(projectId, ranking),
      this.api.readDocument(projectId, candidates),
      this.readOptional(projectId, publishedArtifact(job, INDEX_KIND)),
      this.readOptional(projectId, filmstripId),
    ]);
    const rows = clipRows(
      JSON.parse(rankingDoc.json) as RankingSet,
      JSON.parse(candidateDoc.json) as DiscoveryCandidates,
      indexDoc ? (JSON.parse(indexDoc) as IndexTranscript) : null,
      [],
    );
    const strip = filmstripDoc ? (JSON.parse(filmstripDoc) as MediaFilmstrip) : null;
    const still = (ticks: number): string | null => {
      if (!strip || !filmstripId || strip.tiles.length === 0) return null;
      const nearest = strip.tiles.reduce((best, tile) =>
        Math.abs(tile.t_ticks - ticks) < Math.abs(best.t_ticks - ticks) ? tile : best,
      );
      return this.api.mediaUrl(projectId, filmstripId, nearest.file);
    };
    return new Map(
      rows.map((row) => [
        row.candidateId,
        { title: row.headline, thumbnail: still(row.startTicks) },
      ]),
    );
  }

  async loadTranscript(projectId: string, snapshot: ResultsSnapshot): Promise<Transcript | null> {
    const json = await this.readOptional(projectId, snapshot.transcriptArtifactId ?? null);
    if (!json || !snapshot.source) {
      return null;
    }
    try {
      const speech = JSON.parse(json) as SpeechTranscript;
      if (
        speech.schema_version !== 'clipmill.speech.transcript.v1' ||
        speech.source_fingerprint !== snapshot.source.sourceFingerprint
      ) {
        return null;
      }
      return readTranscript(speech, snapshot.index ?? null);
    } catch {
      return null;
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
