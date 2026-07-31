/**
 * Which component answers for which navigation section.
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
 */
import type { JSX } from 'react';

import type { NavSection } from '../shell/navigation.js';
import { Library } from './Library.js';
import { ModelsDevice } from './ModelsDevice.js';
import { PhasePlaceholder } from './PhasePlaceholder.js';

/**
 * Everything a screen may need from the shell.
 *
 * One record rather than per-screen props threaded through `App`: screens differ
 * in what they use, and a shell that knew which would have to change every time
 * one of them started using something else.
 */
export interface ScreenContext {
  readonly section: NavSection;
  readonly models: Parameters<typeof ModelsDevice>[0];
  readonly library: Parameters<typeof Library>[0];
}

type Screen = (context: ScreenContext) => JSX.Element;

const SCREENS: Readonly<Record<string, Screen>> = {
  library: ({ library }) => <Library {...library} />,
  models: ({ models }) => <ModelsDevice {...models} />,
};

/**
 * The screen for a section, or the placeholder that says which phase builds it.
 *
 * `availability` still decides. A section marked planned renders the placeholder
 * even if something is registered for it, so a half-finished screen cannot reach
 * a user by being wired up early — the section's own declaration is what opens
 * the door.
 */
export function renderScreen(context: ScreenContext): JSX.Element {
  const { section } = context;
  const screen = section.availability.kind === 'live' ? SCREENS[section.id] : undefined;
  return screen ? screen(context) : <PhasePlaceholder section={section} />;
}
