/**
 * Which component answers for where the shell is.
 *
 * The shell used to pick its screen with a ternary — one live section, and
 * everything else a placeholder. That stops scaling the moment a second screen
 * exists, and it hides the thing worth being explicit about: a section is either
 * backed by something real or it is honestly marked as not built yet.
 *
 * So the registry is the single place that answers, and the answer is the same
 * shape either way. A section with no entry here falls through to the placeholder
 * that names the phase which will build it — which means adding a screen is one
 * line, and forgetting to add one produces "coming soon" rather than a blank
 * pane or a crash.
 *
 * One screen is not a section at all. Analysis Progress is about a particular
 * run, has no navigation row by design, and is reached from the two screens that
 * can name a run — so the route decides, and only then does the section.
 */
import type { JSX } from 'react';

import type { Route } from '../shell/route.js';
import { placementOf } from '../shell/route.js';
import { AnalysisProgress } from './AnalysisProgress.js';
import { Library } from './Library.js';
import { ModelsDevice } from './ModelsDevice.js';
import { NewProject } from './NewProject.js';
import { PhasePlaceholder } from './PhasePlaceholder.js';
import { ResultsScreen } from './ResultsScreen.js';

/**
 * Everything a screen may need from the shell.
 *
 * One record rather than per-screen props threaded through `App`: screens differ
 * in what they use, and a shell that knew which would have to change every time
 * one of them started using something else.
 */
export interface ScreenContext {
  readonly route: Route;
  readonly models: Parameters<typeof ModelsDevice>[0];
  readonly library: Parameters<typeof Library>[0];
  readonly newProject: Parameters<typeof NewProject>[0];
  /** The run-specific arguments come from the route, so these are the rest. */
  readonly analysis: Omit<Parameters<typeof AnalysisProgress>[0], 'projectId' | 'jobId'>;
  /** The Inspector's own arguments come from the route, so these are the rest. */
  readonly results: Omit<Parameters<typeof ResultsScreen>[0], 'candidateId'>;
}

type Screen = (context: ScreenContext) => JSX.Element;

const SCREENS: Readonly<Record<string, Screen>> = {
  library: ({ library }) => <Library {...library} />,
  'new-project': ({ newProject }) => <NewProject {...newProject} />,
  models: ({ models }) => <ModelsDevice {...models} />,
  results: ({ results }) => <ResultsScreen {...results} candidateId={null} />,
};

/**
 * The screen for the current route, or the placeholder that names its phase.
 *
 * For a section, `availability` still decides: a section marked planned renders
 * the placeholder even if something is registered for it, so a half-finished
 * screen cannot reach a user by being wired up early — the section's own
 * declaration is what opens the door.
 */
export function renderScreen(context: ScreenContext): JSX.Element {
  const { route } = context;
  if (route.kind === 'analysis') {
    return (
      <AnalysisProgress {...context.analysis} projectId={route.projectId} jobId={route.jobId} />
    );
  }
  if (route.kind === 'inspector') {
    return <ResultsScreen {...context.results} candidateId={route.candidateId} />;
  }
  const { section } = placementOf(route);
  const screen = section.availability.kind === 'live' ? SCREENS[section.id] : undefined;
  return screen ? screen(context) : <PhasePlaceholder section={section} />;
}
