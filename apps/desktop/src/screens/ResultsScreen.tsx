/**
 * Shared data container for Results and Clip Inspector.
 * The route selects the project, source, and run; sidebar entry defaults to the
 * newest project and provides a picker. Approval passes the returned document's
 * full clip identity to the editor when opening it; review decisions stay on the
 * Inspector, advance through the queue, and can be undone most recent first.
 */
import { useEffect, useMemo, useRef, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { newest } from '../daemon/ordering.js';
import type { ClipDecision, Project } from '../daemon/client.js';
import { exactCaptionsOf } from '../editor/exactCaptions.js';
import { nextUndecided } from '../inspector/review.js';
import type { ClipRow } from '../results/model.js';
import { ManualClip } from '../results/ManualClip.js';
import { useResults } from '../results/useResults.js';
import type { ClipRef } from '../shell/route.js';
import { ClipInspector } from './ClipInspector.js';
import { Results } from './Results.js';
import { Skeleton } from '../components/ui/skeleton.js';

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
  const [manualOpen, setManualOpen] = useState(false);
  const [projects, setProjects] = useState<readonly Project[]>([]);
  /** A pick made in the header, which outranks the route until the route moves. */
  const [projectReload, setProjectReload] = useState(0);
  const [projectsLoading, setProjectsLoading] = useState(true);
  const [projectsProblem, setProjectsProblem] = useState<string | null>(null);
  const [picked, setPicked] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setProjectsLoading(true);
    void api
      .listProjects()
      .then((next) => {
        if (active) {
          setProjects(next);
          setProjectsProblem(null);
        }
      })
      .catch((cause: unknown) => {
        if (active) {
          setProjects([]);
          setProjectsProblem(cause instanceof Error ? cause.message : String(cause));
        }
      })
      .finally(() => {
        if (active) setProjectsLoading(false);
      });
    return () => {
      active = false;
    };
  }, [api, projectReload]);

  // A new route is a new intent, so it clears a pick made under the old one.
  useEffect(() => {
    setPicked(null);
  }, [projectId]);

  const intent = `${projectId ?? ''}/${sourceId ?? ''}/${jobId ?? ''}/${candidateId ?? ''}/${picked ?? ''}`;
  const intentRef = useRef(intent);
  intentRef.current = intent;
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const wanted = picked ?? projectId;
  const project = wanted
    ? projects.find((candidate) => candidate.projectId === wanted)
    : newest(projects);
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
  useEffect(() => setManualOpen(false), [project?.projectId, sourceId, jobId]);

  /** The breadcrumb's words for a clip: the project's name and the clip's rank. */
  const labelsFor = (id: string) => {
    const row = snapshot.rows.find((candidate) => candidate.candidateId === id);
    return {
      ...(project ? { project: project.name } : {}),
      // The clip's own title, as everywhere else it is named; the rank is a
      // position on this board and means nothing on the next screen.
      ...(row ? { clip: row.headline.trim() || `Clip ${String(row.rank).padStart(2, '0')}` } : {}),
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

  /**
   * Everything the editor needs to open a row's document, named in full.
   *
   * The run is the document's own when the store recorded one — a reopened
   * edit was cut from the run that minted its candidate, which may not be
   * the run the board is showing — and the board's otherwise.
   */
  const clipFor = (row: ClipRow, docId: string, documentJobId?: string): ClipRef | null => {
    if (!project || !snapshot.source) {
      return null;
    }
    const run = documentJobId || snapshot.run?.jobId;
    return {
      projectId: project.projectId,
      docId,
      sourceId: snapshot.source.sourceId,
      candidateId: row.candidateId,
      ...(run ? { jobId: run } : {}),
      labels: labelsFor(row.candidateId),
    };
  };

  const edit = (row: ClipRow, docId: string, documentJobId?: string) => {
    const clip = clipFor(row, docId, documentJobId);
    if (clip && mounted.current && intentRef.current === intent) {
      onEdit(clip);
    }
  };

  /** Whether a decision moves on to the next undecided clip; kept per machine. */
  const [autoAdvance, setAutoAdvance] = useState(() => {
    try {
      return localStorage.getItem('clipmill.review.advance') !== 'off';
    } catch {
      return true;
    }
  });
  const chooseAutoAdvance = (on: boolean) => {
    setAutoAdvance(on);
    try {
      localStorage.setItem('clipmill.review.advance', on ? 'on' : 'off');
    } catch {
      /* Still in effect for this session. */
    }
  };

  /**
   * What each Inspector decision replaced, newest last, so the last can be
   * taken back. Cleared with the recording: another board's decisions are not
   * on screen to undo.
   */
  const history = useRef<{ candidateId: string; previous: ClipDecision | null }[]>([]);
  const [undoable, setUndoable] = useState(0);
  const recording = `${project?.projectId ?? ''}/${snapshot.source?.sourceId ?? ''}`;
  useEffect(() => {
    history.current = [];
    setUndoable(0);
  }, [recording]);
  const recordDecision = (candidateId: string, next: ClipDecision | null) => {
    const previous = snapshot.rows.find((row) => row.candidateId === candidateId)?.decision ?? null;
    if (previous === next) return;
    history.current.push({ candidateId, previous });
    setUndoable(history.current.length);
  };

  const advanceFrom = (candidateId: string) => {
    if (!autoAdvance || !mounted.current || intentRef.current !== intent) return;
    const next = nextUndecided(snapshot.rows, candidateId);
    if (next) inspect(next);
  };

  const approveOnScreen = async (
    id: string,
    window: { readonly startTicks: number; readonly endTicks: number } | null,
    open: boolean,
  ) => {
    const row = snapshot.rows.find((candidate) => candidate.candidateId === id);
    const directed = await results.approve(id, window);
    if (!directed || !row) return;
    recordDecision(id, 'approved');
    if (open) edit(row, directed.docId, directed.jobId);
    else advanceFrom(id);
  };

  const decideOnScreen = async (id: string, decision: 'kept' | 'rejected' | null) => {
    const recorded = await results.decide(id, decision);
    if (!recorded) return;
    recordDecision(id, decision);
    if (decision !== null) advanceFrom(id);
  };

  /** Put the last decision back; an undone approval leaves its edit in Edits. */
  const undo = async () => {
    const last = history.current.pop();
    setUndoable(history.current.length);
    if (!last) return;
    const recorded =
      last.previous === 'approved'
        ? (await results.approve(last.candidateId, null)) !== null
        : await results.decide(last.candidateId, last.previous);
    if (!recorded) return;
    results.say('Decision undone.');
    if (last.candidateId !== candidateId) inspect(last.candidateId);
  };

  // Ask where the camera should point whenever the opened clip changes, and
  // read the recording's words for it. The solve writes nothing, so this is a
  // question rather than a commitment.
  const { requestTranscript } = results;
  useEffect(() => {
    if (candidateId) {
      solveFor(candidateId);
      requestTranscript();
    }
  }, [candidateId, solveFor, requestTranscript]);

  // The clip an approval would build, for the clip on screen only: another
  // clip's plan would frame this one by the wrong sections.
  const planned =
    candidateId && results.preview?.candidateId === candidateId ? results.preview.plan : null;
  const plannedCaptions = useMemo(
    () => exactCaptionsOf(planned, api.captionFontUrl ?? null),
    [planned, api],
  );
  const { previewFor } = results;
  // The manual clip's own preview: asked for by the sheet as its span moves.
  const manualPlanned =
    manualOpen && results.preview?.candidateId === '' ? results.preview.plan : null;
  const manualCaptions = useMemo(
    () => exactCaptionsOf(manualPlanned, api.captionFontUrl ?? null),
    [manualPlanned, api],
  );
  // The sheet snaps to words, so it needs them read.
  useEffect(() => {
    if (manualOpen) requestTranscript();
  }, [manualOpen, requestTranscript]);

  if (candidateId && (projectsLoading || results.loading))
    return (
      <div className="workspace-page" aria-label="Loading clip" aria-busy="true">
        <Skeleton className="h-10 w-2/3" />
        <Skeleton className="min-h-0 flex-1" />
      </div>
    );

  if (candidateId) {
    const opened = snapshot.rows.find((row) => row.candidateId === candidateId);
    return (
      <ClipInspector
        rows={snapshot.rows}
        candidateId={candidateId}
        proxyUrl={results.proxyUrl}
        crop={results.crop}
        preview={planned}
        previewCaptions={plannedCaptions}
        onPreview={(cut) => previewFor(candidateId, cut)}
        framing={results.framing}
        peaks={snapshot.peaks}
        tileUrl={results.tileUrl}
        transcript={results.transcript}
        sourceDurationTicks={snapshot.sourceDurationTicks ?? null}
        durationTarget={snapshot.durationTarget ?? null}
        busy={results.busy}
        notice={results.notice}
        autoAdvance={autoAdvance}
        onAutoAdvance={chooseAutoAdvance}
        onSelect={inspect}
        onBack={onBack}
        onApprove={(window, open) => {
          void approveOnScreen(candidateId, window, open);
        }}
        onDecide={(decision) => {
          void decideOnScreen(candidateId, decision);
        }}
        onUndo={undoable > 0 ? () => void undo() : null}
        onOpenEdit={
          opened?.docId
            ? () => {
                edit(opened, opened.docId!, opened.docJobId ?? undefined);
              }
            : null
        }
      />
    );
  }

  // The recording is named on the board rather than in a line above it, so the
  // header says which run these numbers describe instead of leaving it implied.
  const sourceName = snapshot.source
    ? [project?.name, snapshot.source.absolutePath.split(/[\\/]/).at(-1)]
        .filter(Boolean)
        .join(' · ')
    : null;

  return (
    <>
      <ManualClip
        key={`${project?.projectId ?? ''}/${snapshot.source?.sourceId ?? ''}/${snapshot.run?.jobId ?? ''}`}
        open={manualOpen}
        onOpenChange={setManualOpen}
        sourceName={sourceName ?? 'Recording'}
        sourceDurationTicks={snapshot.sourceDurationTicks ?? null}
        proxyUrl={results.proxyUrl}
        busy={results.busy}
        notice={results.notice}
        transcript={results.transcript.status === 'ready' ? results.transcript.transcript : null}
        peaks={snapshot.peaks}
        tileUrl={results.tileUrl}
        preview={manualPlanned}
        previewCaptions={manualCaptions}
        onPreview={(cut) => previewFor('', cut)}
        onCreate={async (startTicks, endTicks) => {
          const directed = await results.manual(startTicks, endTicks);
          if (!directed || !mounted.current || intentRef.current !== intent) return false;
          onEdit({
            projectId: directed.projectId,
            sourceId: directed.sourceId,
            candidateId: directed.candidateId,
            docId: directed.docId,
            ...(directed.jobId ? { jobId: directed.jobId } : {}),
            labels: { ...(project ? { project: project.name } : {}), clip: 'Manual clip' },
          });
          return true;
        }}
      />
      <Results
        loading={results.loading || projectsLoading}
        notice={results.notice}
        {...(snapshot.source && snapshot.run ? { onManualClip: () => setManualOpen(true) } : {})}
        rows={snapshot.rows}
        summary={snapshot.summary}
        problem={
          projectsProblem
            ? { kind: 'unreadable', detail: projectsProblem }
            : wanted && !project
              ? {
                  kind: 'unreadable',
                  detail:
                    'This project is no longer available. Choose another project to continue.',
                }
              : snapshot.problem
        }
        sourceName={sourceName}
        run={snapshot.run}
        tileUrl={results.tileUrl}
        framing={results.framing}
        projects={projects}
        activeProjectId={project?.projectId ?? null}
        busy={results.busy}
        onChooseProject={setPicked}
        onApproveMany={(ids) => {
          void results.approveMany(ids);
        }}
        onReload={() => {
          if (projectsProblem || (wanted && !project)) setProjectReload((value) => value + 1);
          else results.reload();
        }}
        onInspect={inspect}
        onEdit={(id) => {
          const row = snapshot.rows.find((candidate) => candidate.candidateId === id);
          if (row?.docId) {
            edit(row, row.docId, row.docJobId ?? undefined);
          }
        }}
      />
    </>
  );
}
