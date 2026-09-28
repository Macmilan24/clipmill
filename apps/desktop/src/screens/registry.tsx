/**
 * Map routes to screens, falling back to a placeholder for unimplemented sections.
 * Analysis Progress uses a run-specific route without a sidebar row. Editor and
 * Export open the named clip or a document picker when the route has no clip.
 */
import { lazy, Suspense, type JSX } from 'react';

import type { Route } from '../shell/route.js';
import { clipOf, focusOf, placementOf } from '../shell/route.js';
import { AnalysisProgress } from './AnalysisProgress.js';
import { Library } from './Library.js';
// Hardware charts are loaded only when their screen is opened.
const ModelsDevice = lazy(() =>
  import('./ModelsDevice.js').then((module) => ({ default: module.ModelsDevice })),
);
import { NewProject } from './NewProject.js';
import { PhasePlaceholder } from './PhasePlaceholder.js';
import { EditorScreen } from './EditorScreen.js';
import { ExportScreen } from './ExportScreen.js';
import { SettingsScreen } from './SettingsScreen.js';
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
  readonly results: Omit<
    Parameters<typeof ResultsScreen>[0],
    'candidateId' | 'projectId' | 'sourceId' | 'jobId'
  >;
  /** The clip comes from the route, so this is the rest. */
  readonly editor: Omit<Parameters<typeof EditorScreen>[0], 'clip'>;
  readonly export: Omit<Parameters<typeof ExportScreen>[0], 'clip'>;
  /** Settings reads the daemon directly; nothing routes into it. */
  readonly settings: Parameters<typeof SettingsScreen>[0];
}

type Screen = (context: ScreenContext) => JSX.Element;

const SCREENS: Readonly<Record<string, Screen>> = {
  library: ({ library }) => <Library {...library} />,
  'new-project': ({ newProject }) => <NewProject {...newProject} />,
  models: ({ models }) => (
    <Suspense
      fallback={
        <p role="status" className="text-xs text-[var(--cm-text-secondary)]">
          Loading Models &amp; Device…
        </p>
      }
    >
      <ModelsDevice {...models} />
    </Suspense>
  ),
  results: ({ results, route }) => (
    <ResultsScreen
      {...results}
      candidateId={null}
      projectId={route.kind === 'section' ? (route.projectId ?? null) : null}
      sourceId={route.kind === 'section' ? (route.sourceId ?? null) : null}
      jobId={route.kind === 'section' ? (route.jobId ?? null) : null}
    />
  ),
  editor: ({ editor, route }) => (
    <EditorScreen {...editor} clip={clipOf(route)} focus={focusOf(route)} />
  ),
  export: (context) => <ExportScreen {...context.export} clip={clipOf(context.route)} />,
  settings: ({ settings }) => <SettingsScreen {...settings} />,
};

/**
 * Render the current route or the registered section placeholder.
 */
export function renderScreen(context: ScreenContext): JSX.Element {
  const { route } = context;
  if (route.kind === 'analysis') {
    return (
      <AnalysisProgress {...context.analysis} projectId={route.projectId} jobId={route.jobId} />
    );
  }
  if (route.kind === 'inspector') {
    return (
      <ResultsScreen
        {...context.results}
        candidateId={route.candidateId}
        projectId={route.projectId}
        sourceId={route.sourceId}
        jobId={route.jobId ?? null}
      />
    );
  }
  const { section } = placementOf(route);
  const screen = section.availability.kind === 'live' ? SCREENS[section.id] : undefined;
  return screen ? screen(context) : <PhasePlaceholder section={section} />;
}
