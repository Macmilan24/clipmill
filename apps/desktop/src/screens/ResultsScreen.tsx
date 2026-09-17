/**
 * The container both result screens share.
 *
 * Results and the Clip Inspector are one dataset seen two ways — the Inspector
 * is the board with a row opened — so one hook loads it and the route decides
 * which view renders. That keeps a decision made in the Inspector visible on the
 * board without either screen knowing about the other.
 *
 * Which recording is shown is the one the route named — the project clicked in
 * the Library, or the clip's own project, source and run when the Inspector is
 * open. Falling back to the newest project is what happens when nothing named
 * one, which is the sidebar entry; the picker in the header is how that stops
 * being a dead end, since a screen reachable from the navigation must be able
 * to reach every recording rather than whichever the daemon wrote last.
 *
 * Approving hands the clip on. The document the daemon answers with — built,
 * or reopened if the clip already had one — is named in full to the editor:
 * project, source, run, candidate, document. Nothing downstream has to find it.
 */
import { useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { newest } from '../daemon/ordering.js';
import type { ClipDecision, DirectedClip, Project } from '../daemon/client.js';
import type { ClipRow } from '../results/model.js';
import { useResults } from '../results/useResults.js';
import type { ClipRef } from '../shell/route.js';
import { ClipInspector } from './ClipInspector.js';
import { Results } from './Results.js';

export interface ResultsScreenProps {
  /** Set when the route is the Inspector, null on the board. */
  readonly candidateId: string | null;
  /** The project the route named, or null to fall back to the newest. */
  readonly projectId: string | null;
  /** The recording the route named, or null for the project's newest. */
  readonly sourceId: string | null;
  /** The analysis run the route named, or null for the source's newest. */
  readonly jobId: string | null;
  readonly onInspect: (
    projectId: string,
    sourceId: string,
    candidateId: string,
    labels?: { readonly project?: string; readonly clip?: string },
    jobId?: string,
  ) => void;
  /** Open a clip's edit document in the editor. */
  readonly onEdit: (clip: ClipRef) => void;
  readonly onBack: () => void;
  readonly api?: ShellApi;
}

export function ResultsScreen({
  candidateId,
  projectId,
  sourceId,
  jobId,
  onInspect,
  onEdit,
  onBack,
  api = daemonApi,
}: ResultsScreenProps) {
  const [projects, setProjects] = useState<readonly Project[]>([]);
  /** A pick made in the header, which outranks the route until the route moves. */
  const [picked, setPicked] = useState<string | null>(null);

  useEffect(() => {
    void api
      .listProjects()
      .then(setProjects)
      .catch(() => setProjects([]));
  }, [api]);

  // A new route is a new intent, so it clears a pick made under the old one.
  useEffect(() => {
    setPicked(null);
  }, [projectId]);

  const wanted = picked ?? projectId;
  const project = projects.find((candidate) => candidate.projectId === wanted) ?? newest(projects);
  // A pick from the header is a different recording, so the source and run the
  // route named belong to the project it named and not to the one picked.
  const routed = picked === null || picked === projectId;

  const results = useResults(
    project?.projectId ?? null,
    routed ? sourceId : null,
    routed ? jobId : null,
    api,
  );
  const { snapshot, solveFor } = results;

  /** The breadcrumb's words for a clip: the project's name and the clip's rank. */
  const labelsFor = (id: string) => {
    const row = snapshot.rows.find((candidate) => candidate.candidateId === id);
    return {
      ...(project ? { project: project.name } : {}),
      ...(row ? { clip: `Clip ${String(row.rank).padStart(2, '0')}` } : {}),
    };
  };

  const inspect = (next: string) => {
    if (project && snapshot.source) {
      onInspect(
        project.projectId,
        snapshot.source.sourceId,
        next,
        labelsFor(next),
        snapshot.run?.jobId,
      );
    }
  };

  /** Everything the editor needs to open a row's document, named in full. */
  const clipFor = (row: ClipRow, docId: string): ClipRef | null => {
    if (!project || !snapshot.source) {
      return null;
    }
    return {
      projectId: project.projectId,
      docId,
      sourceId: snapshot.source.sourceId,
      candidateId: row.candidateId,
      ...(snapshot.run ? { jobId: snapshot.run.jobId } : {}),
      labels: labelsFor(row.candidateId),
    };
  };

  const edit = (row: ClipRow, docId: string) => {
    const clip = clipFor(row, docId);
    if (clip) {
      onEdit(clip);
    }
  };

  /** Approving from the Inspector opens what it approved. */
  const approve = async (id: string) => {
    const directed: DirectedClip | null = await results.decide(id, 'approved');
    const row = snapshot.rows.find((candidate) => candidate.candidateId === id);
    if (directed && row) {
      edit(row, directed.docId);
    }
  };

  // Ask where the camera should point whenever the opened clip changes. The
  // solve writes nothing, so this is a question rather than a commitment.
  useEffect(() => {
    if (candidateId) {
      solveFor(candidateId);
    }
  }, [candidateId, solveFor]);

  if (candidateId) {
    const opened = snapshot.rows.find((row) => row.candidateId === candidateId);
    return (
      <ClipInspector
        rows={snapshot.rows}
        candidateId={candidateId}
        proxyUrl={results.proxyUrl}
        crop={results.crop}
        cues={results.cues}
        peaks={snapshot.peaks}
        busy={results.busy}
        notice={results.notice}
        onSelect={inspect}
        onBack={onBack}
        onDecide={(decision: ClipDecision) => {
          if (decision === 'approved') {
            void approve(candidateId);
          } else {
            void results.decide(candidateId, decision);
          }
        }}
        onUseAlternative={() => {
          void results.direct(candidateId, 'alternative');
        }}
        onTakeCut={(startTicks, endTicks) => {
          void results.direct(candidateId, 'exact', { startTicks, endTicks });
        }}
        onEdit={
          opened?.docId
            ? () => {
                edit(opened, opened.docId!);
              }
            : null
        }
      />
    );
  }

  // The recording is named on the board rather than in a line above it, so the
  // header says which run these numbers describe instead of leaving it implied.
  const sourceName = snapshot.source
    ? [project?.name, snapshot.source.absolutePath.split('/').at(-1)].filter(Boolean).join(' · ')
    : null;

  return (
    <Results
      loading={results.loading}
      rows={snapshot.rows}
      summary={snapshot.summary}
      problem={snapshot.problem}
      sourceName={sourceName}
      run={snapshot.run}
      tileUrl={results.tileUrl}
      projects={projects}
      activeProjectId={project?.projectId ?? null}
      busy={results.busy}
      onChooseProject={setPicked}
      onApproveMany={(ids) => {
        void results.approveMany(ids);
      }}
      onReload={results.reload}
      onInspect={inspect}
      onEdit={(id) => {
        const row = snapshot.rows.find((candidate) => candidate.candidateId === id);
        if (row?.docId) {
          edit(row, row.docId);
        }
      }}
    />
  );
}
