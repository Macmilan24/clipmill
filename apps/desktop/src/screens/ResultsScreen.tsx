/**
 * The container both result screens share.
 *
 * Results and the Clip Inspector are one dataset seen two ways — the Inspector
 * is the board with a row opened — so one hook loads it and the route decides
 * which view renders. That keeps a decision made in the Inspector visible on the
 * board without either screen knowing about the other.
 *
 * Which recording is shown is the one the route named — the project clicked in
 * the Library, or the clip's own project when the Inspector is open. Falling
 * back to the newest project is what happens when nothing named one, which is
 * the sidebar entry; the picker in the header is how that stops being a dead
 * end, since a screen reachable from the navigation must be able to reach every
 * recording rather than whichever the daemon wrote last.
 */
import { useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { newest } from '../daemon/ordering.js';
import type { ClipDecision, Project } from '../daemon/client.js';
import { useResults } from '../results/useResults.js';
import { ClipInspector } from './ClipInspector.js';
import { Results } from './Results.js';

export interface ResultsScreenProps {
  /** Set when the route is the Inspector, null on the board. */
  readonly candidateId: string | null;
  /** The project the route named, or null to fall back to the newest. */
  readonly projectId: string | null;
  readonly onInspect: (projectId: string, sourceId: string, candidateId: string) => void;
  readonly onBack: () => void;
  readonly api?: ShellApi;
}

export function ResultsScreen({
  candidateId,
  projectId,
  onInspect,
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

  const results = useResults(project?.projectId ?? null, null, api);
  const { snapshot, solveFor } = results;

  // Ask where the camera should point whenever the opened clip changes. The
  // solve writes nothing, so this is a question rather than a commitment.
  useEffect(() => {
    if (candidateId) {
      solveFor(candidateId);
    }
  }, [candidateId, solveFor]);

  if (candidateId) {
    return (
      <ClipInspector
        rows={snapshot.rows}
        candidateId={candidateId}
        proxyUrl={results.proxyUrl}
        crop={results.crop}
        cues={results.cues}
        busy={results.busy}
        notice={results.notice}
        onSelect={(next) => {
          if (project && snapshot.source) {
            onInspect(project.projectId, snapshot.source.sourceId, next);
          }
        }}
        onBack={onBack}
        onDecide={(decision: ClipDecision) => {
          void results.decide(candidateId, decision);
        }}
        onUseAlternative={() => {
          void results.direct(candidateId, 'alternative');
        }}
        onTakeCut={(startTicks, endTicks) => {
          void results.direct(candidateId, 'exact', { startTicks, endTicks });
        }}
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
      proxyUrl={results.proxyUrl}
      projects={projects}
      activeProjectId={project?.projectId ?? null}
      onChooseProject={setPicked}
      onReload={results.reload}
      onInspect={(next) => {
        if (project && snapshot.source) {
          onInspect(project.projectId, snapshot.source.sourceId, next);
        }
      }}
    />
  );
}
