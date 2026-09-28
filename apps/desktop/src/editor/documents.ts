/**
 * Load edit documents for Editor and Export pickers.
 * Identify documents by source, project, revision, and edit time. Ranking data is
 * not loaded here; Results provides selection by clip content and rank.
 */
import { useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { EditDocSummary, Project, Source } from '../daemon/client.js';
import { type ClipName, ResultsLoader } from '../results/loader.js';
import type { ClipRef } from '../shell/route.js';

export interface DocumentEntry {
  /** Everything a screen needs to open it. */
  readonly clip: ClipRef;
  readonly projectName: string;
  /** The recording's file name, or null when the document names no source. */
  readonly sourceName: string | null;
  /** The clip's title: the one somebody gave it, or its analysis's headline. */
  readonly title: string | null;
  /** Whether somebody named the clip, so its analysis's headline stays out. */
  readonly named?: boolean;
  /** A frame of the clip, once it is known. */
  readonly thumbnail: string | null;
  readonly revision: number;
  readonly updatedUnixMillis: number;
}

export interface DocumentList {
  readonly loading: boolean;
  /** Newest edited first, because that is the one somebody is looking for. */
  readonly entries: readonly DocumentEntry[];
  readonly problem: string | null;
}

/** Pure: the join, so it is checked without a window. */
export function documentEntries(
  projects: readonly Project[],
  sources: ReadonlyMap<string, readonly Source[]>,
  documents: ReadonlyMap<string, readonly EditDocSummary[]>,
): readonly DocumentEntry[] {
  const entries: DocumentEntry[] = [];
  for (const project of projects) {
    const known = sources.get(project.projectId) ?? [];
    for (const document of documents.get(project.projectId) ?? []) {
      // A document that names no source has no proxy to open and no clip to
      // be; it is listed so it is not lost, but it cannot be edited here.
      if (document.sourceId === '') {
        continue;
      }
      const source = known.find((candidate) => candidate.sourceId === document.sourceId);
      // A name somebody gave the clip is its name everywhere, ahead of the
      // headline its analysis suggested.
      const named = document.title?.trim() || null;
      entries.push({
        clip: {
          projectId: project.projectId,
          docId: document.docId,
          sourceId: document.sourceId,
          ...(document.candidateId === '' ? {} : { candidateId: document.candidateId }),
          ...(document.jobId === '' ? {} : { jobId: document.jobId }),
          labels: { project: project.name, clip: named ?? clipLabel(document, source) },
        },
        projectName: project.name,
        sourceName: source ? fileName(source.absolutePath) : null,
        title: named,
        named: named !== null,
        thumbnail: null,
        revision: document.revision,
        updatedUnixMillis: document.updatedUnixMillis,
      });
    }
  }
  return entries.toSorted(
    (left, right) =>
      right.updatedUnixMillis - left.updatedUnixMillis ||
      left.clip.docId.localeCompare(right.clip.docId),
  );
}

/**
 * The entries with their clips' titles and frames, where the analysis that
 * found each clip still says what it is called. The run a document names is
 * read once for however many of its clips are listed.
 */
export async function withClipNames(
  entries: readonly DocumentEntry[],
  api: ShellApi,
): Promise<readonly DocumentEntry[]> {
  const loader = new ResultsLoader(api);
  const runs = new Map<string, Promise<ReadonlyMap<string, ClipName>>>();
  const named = await Promise.all(
    entries.map(async (entry) => {
      const { projectId, sourceId, jobId, candidateId } = entry.clip;
      if (!candidateId) return entry;
      const key = `${projectId}\u0000${sourceId}\u0000${jobId ?? ''}`;
      if (!runs.has(key)) {
        runs.set(
          key,
          loader.clipNames(projectId, sourceId, jobId ?? null).catch(() => new Map()),
        );
      }
      const name = (await runs.get(key)!).get(candidateId);
      if (!name || !name.title.trim()) return entry;
      if (entry.named) return { ...entry, thumbnail: name.thumbnail };
      return {
        ...entry,
        title: name.title,
        thumbnail: name.thumbnail,
        clip: { ...entry.clip, labels: { project: entry.projectName, clip: name.title } },
      };
    }),
  );
  return named;
}

/** What the breadcrumb calls a document opened from this list, until its title is known. */
function clipLabel(document: EditDocSummary, source: Source | undefined): string {
  const recording = source ? fileName(source.absolutePath) : 'Clip';
  return document.candidateId === ''
    ? recording
    : `${recording} · ${document.candidateId.replace(/^cand_/, '').slice(0, 6)}`;
}

/** The last part of a path, whichever separator the system writes. */
function fileName(path: string): string {
  return path.split(/[\\/]/).at(-1) ?? path;
}

export function useEditDocuments(api: ShellApi = daemonApi): DocumentList {
  const [loading, setLoading] = useState(true);
  const [entries, setEntries] = useState<readonly DocumentEntry[]>([]);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const projects = await api.listProjects();
        const sources = new Map<string, readonly Source[]>();
        const documents = new Map<string, readonly EditDocSummary[]>();
        await Promise.all(
          projects.map(async (project) => {
            const [found, held] = await Promise.all([
              api.listSources(project.projectId).catch(() => []),
              api.listEditDocs(project.projectId).catch(() => []),
            ]);
            sources.set(project.projectId, found);
            documents.set(project.projectId, held);
          }),
        );
        const listed = documentEntries(projects, sources, documents);
        if (live) {
          setEntries(listed);
          setLoading(false);
        }
        // Titles take a few more reads; the list is usable before they land.
        const named = await withClipNames(listed, api);
        if (live) setEntries(named);
      } catch (error) {
        if (live) {
          setProblem((error as Error).message);
          setLoading(false);
        }
      }
    })();
    return () => {
      live = false;
    };
  }, [api]);

  return { loading, entries, problem };
}
