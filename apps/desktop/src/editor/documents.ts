/**
 * Every edit document this installation holds, named well enough to pick one.
 *
 * The editor and the export screen each have a row in the sidebar, and a row
 * reached with no clip named has to lead somewhere. Falling back to the newest
 * document is what this replaces: it was right for one project with one
 * approval and silently wrong otherwise. So the fallback is a list, and the
 * person chooses.
 *
 * A document is named by the recording it was cut from and the project it is
 * in — the two facts the daemon records beside it — plus how far it has been
 * edited and when. Its rank on the Results board is not here, because that
 * lives in a ranking this list would have to fetch per project to know; the
 * board is where a clip is chosen by what it says, and this is where an edit
 * is found again by where it came from.
 */
import { useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { EditDocSummary, Project, Source } from '../daemon/client.js';
import type { ClipRef } from '../shell/route.js';

export interface DocumentEntry {
  /** Everything a screen needs to open it. */
  readonly clip: ClipRef;
  readonly projectName: string;
  /** The recording's file name, or null when the document names no source. */
  readonly sourceName: string | null;
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
      entries.push({
        clip: {
          projectId: project.projectId,
          docId: document.docId,
          sourceId: document.sourceId,
          ...(document.candidateId === '' ? {} : { candidateId: document.candidateId }),
          ...(document.jobId === '' ? {} : { jobId: document.jobId }),
          labels: { project: project.name, clip: clipLabel(document, source) },
        },
        projectName: project.name,
        sourceName: source ? fileName(source.absolutePath) : null,
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

/** What the breadcrumb calls a document opened from this list. */
function clipLabel(document: EditDocSummary, source: Source | undefined): string {
  const recording = source ? fileName(source.absolutePath) : 'Clip';
  return document.candidateId === ''
    ? recording
    : `${recording} · ${document.candidateId.replace(/^cand_/, '').slice(0, 6)}`;
}

function fileName(path: string): string {
  return path.split('/').at(-1) ?? path;
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
        if (live) {
          setEntries(documentEntries(projects, sources, documents));
          setLoading(false);
        }
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
