/**
 * The container both result screens share.
 *
 * Results and the Clip Inspector are one dataset seen two ways — the Inspector
 * is the board with a row opened — so one hook loads it and the route decides
 * which view renders. That keeps a decision made in the Inspector visible on the
 * board without either screen knowing about the other.
 *
 * Which recording is shown is the newest analyzed source of the newest project,
 * and the header says which rather than leaving it implied. Phase 1 has no
 * project picker; inventing one here would be a navigation surface the design
 * does not have.
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
  readonly onInspect: (projectId: string, sourceId: string, candidateId: string) => void;
  readonly onBack: () => void;
  readonly api?: ShellApi;
}

export function ResultsScreen({
  candidateId,
  onInspect,
  onBack,
  api = daemonApi,
}: ResultsScreenProps) {
  const [project, setProject] = useState<Project | null>(null);

  useEffect(() => {
    void api
      .listProjects()
      .then((projects) => setProject(newest(projects)))
      .catch(() => setProject(null));
  }, [api]);

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
      />
    );
  }

  return (
    <div className="flex min-h-0 flex-col">
      {snapshot.source && (
        <p className="px-8 pt-6 text-xs text-[var(--cm-ink-2)]">
          {project?.name} · {snapshot.source.absolutePath.split('/').at(-1)}
        </p>
      )}
      <Results
        loading={results.loading}
        rows={snapshot.rows}
        summary={snapshot.summary}
        problem={snapshot.problem}
        onReload={results.reload}
        onInspect={(next) => {
          if (project && snapshot.source) {
            onInspect(project.projectId, snapshot.source.sourceId, next);
          }
        }}
      />
    </div>
  );
}
