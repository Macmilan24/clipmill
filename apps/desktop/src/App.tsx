import {
  type CSSProperties,
  type JSX,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useState,
} from 'react';

import type { DeviceProfile } from '@clipmill/contracts';
import { type Theme, type WorkspaceTheme, ThemeController } from '@clipmill/tokens';

import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar';
import { TooltipProvider } from '@/components/ui/tooltip';

import {
  type ConnectionState,
  fetchDaemonState,
  fetchDeviceProfile,
  reconnectDaemon,
  subscribeDaemonState,
} from './daemon/client.js';
import { renderScreen } from './screens/registry.js';
import { AppSidebar } from './shell/Sidebar.js';
import { useUpdateNotice } from './shell/updates.js';
import { TopBar } from './shell/TopBar.js';
import { useAnalysisActivity } from './shell/useAnalysisActivity.js';
import { ShortcutSheet, useShortcutSheet } from './shell/ShortcutSheet.js';
import { CoachEnabled, OPEN_WELCOME_EVENT, shouldWelcome } from './onboarding/state.js';
import { Welcome } from './onboarding/Welcome.js';
import { recall, remember } from './shell/memory.js';
import {
  type ClipRef,
  type EditorFocus,
  type Route,
  editorRoute,
  exportRoute,
  inspectorRoute,
  placementOf,
  sectionRoute,
  resultsRouteFor,
} from './shell/route.js';

/**
 * Identity of the *current* daemon process, not merely of being connected.
 * A daemon that was killed and respawned reports a new start time, so this key
 * changes and the profile is re-fetched — which is what makes the recovery
 * demo work without restarting the app.
 */
function connectionKey(state: ConnectionState): string | null {
  return state.status === 'connected' ? `${state.daemonVersion}:${state.startedUnixMillis}` : null;
}

export function App(): JSX.Element {
  const controller = useMemo(
    () =>
      new ThemeController(
        document.documentElement,
        typeof localStorage === 'undefined' ? null : localStorage,
      ),
    [],
  );

  const [theme, setTheme] = useState<Theme>(() =>
    ThemeController.resolveInitial(
      typeof localStorage === 'undefined' ? null : localStorage,
      typeof matchMedia === 'function' && matchMedia('(prefers-color-scheme: light)').matches,
    ),
  );
  const [workspaceTheme, setWorkspaceTheme] = useState<WorkspaceTheme>(() =>
    ThemeController.resolveWorkspace(typeof localStorage === 'undefined' ? null : localStorage),
  );

  // Where the shell was, and which clip it was on, put back from the last
  // launch. The clip is what the Editor and Export rows open when reached from
  // the sidebar with nothing named — a person's own last choice, rather than
  // whichever document the daemon wrote most recently.
  const [memory] = useState(() =>
    recall(typeof localStorage === 'undefined' ? null : localStorage),
  );
  const [route, setRoute] = useState<Route>(memory.route);
  const [clip, setClip] = useState<ClipRef | null>(memory.clip);
  const [state, setState] = useState<ConnectionState>({ status: 'connecting' });
  const [profile, setProfile] = useState<DeviceProfile | null>(null);
  const [artifactId, setArtifactId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const analysisActivity = useAnalysisActivity(state);

  // Restore both choices before the first React paint.
  useLayoutEffect(() => {
    controller.apply(theme);
    controller.applyWorkspace(workspaceTheme);
  }, [controller, theme, workspaceTheme]);

  const toggleTheme = useCallback(() => {
    setTheme(controller.toggle());
  }, [controller]);

  // Seed the connection state, then follow the host's transitions.
  useEffect(() => {
    let active = true;
    void fetchDaemonState().then((initial) => {
      if (active) {
        setState(initial);
      }
    });

    const pending = subscribeDaemonState((next) => {
      if (active) {
        setState(next);
      }
    });

    return () => {
      active = false;
      void pending.then((unlisten) => {
        unlisten();
      });
    };
  }, []);

  const loadProfile = useCallback(async (remeasure: boolean) => {
    setBusy(true);
    try {
      const result = await fetchDeviceProfile(remeasure);
      setProfile(result.profile);
      setArtifactId(result.artifactId);
      setError(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }, []);

  // Re-measure identity, not just connectivity: see connectionKey.
  const key = connectionKey(state);
  useEffect(() => {
    if (key !== null) {
      void loadProfile(false);
    }
  }, [key, loadProfile]);

  const handleReconnect = useCallback(() => {
    void reconnectDaemon().then(setState);
  }, []);

  useEffect(() => {
    remember(typeof localStorage === 'undefined' ? null : localStorage, { route, clip });
  }, [route, clip]);

  /** Open a clip in the editor or on the export screen, and remember it. */
  const openClip = useCallback(
    (next: ClipRef, screen: 'editor' | 'export', focus?: EditorFocus) => {
      setClip(next);
      setRoute(screen === 'editor' ? editorRoute(next, focus) : exportRoute(next));
    },
    [],
  );

  const navigate = useCallback(
    (sectionId: string, projectId?: string) => {
      // The two rows that are about a clip open the one last opened, when
      // there is one; the plain section is the screen saying "choose".
      if ((sectionId === 'editor' || sectionId === 'export') && clip && projectId === undefined) {
        setRoute(sectionId === 'editor' ? editorRoute(clip) : exportRoute(clip));
        return;
      }
      setRoute(sectionRoute(sectionId, projectId));
    },
    [clip],
  );

  // The run a screen opened, and the section it was opened from — which is the
  // row the sidebar keeps lit while it is on screen.
  const openAnalysis = useCallback((projectId: string, jobId: string, from: string) => {
    setRoute({ kind: 'analysis', projectId, jobId, from });
  }, []);

  const { section, trail } = placementOf(route);
  const shortcuts = useShortcutSheet();
  const update = useUpdateNotice(state.status === 'connected' ? state.daemonVersion : null);
  // The welcome, once for a new installation, and again when Settings asks.
  const [welcoming, setWelcoming] = useState(() => shouldWelcome());
  useEffect(() => {
    const again = () => setWelcoming(true);
    window.addEventListener(OPEN_WELCOME_EVENT, again);
    return () => window.removeEventListener(OPEN_WELCOME_EVENT, again);
  }, []);
  // The two workspaces give the picture the height: the trail and engine
  // line fold away — the workspace's own heading names the clip and leads
  // back — unless the engine is not ready, which is then the first thing on
  // screen. The sidebar stays as it is on every other page.
  const workspace = route.kind === 'inspector' || route.kind === 'editor';
  const folded = workspace && state.status === 'connected';

  return (
    <CoachEnabled.Provider value={true}>
      <TooltipProvider delayDuration={300}>
        <ShortcutSheet open={shortcuts.open} onOpenChange={shortcuts.setOpen} />
        <Welcome
          open={welcoming}
          onClose={() => setWelcoming(false)}
          onStart={() => navigate('new-project')}
        />
        <SidebarProvider
          // The sidebar becomes an icon rail in compact desktop windows.
          style={{ '--sidebar-width': 'var(--cm-shell-sidebar-width)' } as CSSProperties}
          className="studio-shell relative h-full min-h-0"
        >
          <AppSidebar
            activeId={section.id}
            onSelect={navigate}
            state={state}
            analysisBusy={analysisActivity.active}
          />
          <SidebarInset className="min-h-0 min-w-0 bg-transparent">
            {!folded && (
              <TopBar
                trail={trail}
                theme={theme}
                onToggleTheme={toggleTheme}
                state={state}
                profile={profile}
                update={update}
              />
            )}
            <main
              className={`studio-main ${['results', 'editor'].includes(section.id) ? 'studio-main-workspace' : 'studio-main-page'}`}
            >
              {renderScreen({
                route,
                library: {
                  state,
                  onNavigate: navigate,
                  onOpenAnalysis: (projectId, jobId) => {
                    openAnalysis(projectId, jobId, 'library');
                  },
                  onReconnect: handleReconnect,
                },
                newProject: {
                  state,
                  onStarted: (projectId, jobId) => {
                    analysisActivity.markStarted(jobId);
                    openAnalysis(projectId, jobId, 'new-project');
                  },
                  onOpenModels: () => {
                    navigate('models');
                  },
                },
                analysis: {
                  profile,
                  onRestarted: (projectId, jobId) => {
                    analysisActivity.markStarted(jobId);
                    openAnalysis(projectId, jobId, 'library');
                  },
                  onBack: () => {
                    navigate(route.kind === 'analysis' ? route.from : 'library');
                  },
                  onNavigate: navigate,
                },
                results: {
                  onInspect: (projectId, sourceId, candidateId, labels, jobId) => {
                    setRoute(inspectorRoute(projectId, sourceId, candidateId, labels, jobId));
                  },
                  onEdit: (next) => {
                    openClip(next, 'editor');
                  },
                  onBack: () => {
                    setRoute(resultsRouteFor(route));
                  },
                },
                editor: {
                  onOpenResults: () => {
                    setRoute(resultsRouteFor(route));
                  },
                  onOpen: (next) => {
                    openClip(next, 'editor');
                  },
                  onExport: (next) => {
                    openClip(next, 'export');
                  },
                },
                export: {
                  onOpenChannelSettings: () => setRoute({ kind: 'section', sectionId: 'settings' }),
                  onEdit: (next, focus) => openClip(next, 'editor', focus),
                  onOpen: (next) => {
                    openClip(next, 'export');
                  },
                },
                settings: {
                  engineVersion: state.status === 'connected' ? state.daemonVersion : null,
                  theme,
                  onThemeChange: setTheme,
                  workspaceTheme,
                  onWorkspaceThemeChange: setWorkspaceTheme,
                  onOpenModels: () => {
                    navigate('models');
                  },
                },
                models: {
                  state,
                  profile,
                  artifactId,
                  error,
                  busy,
                  onRescan: () => {
                    void loadProfile(true);
                  },
                  onReconnect: handleReconnect,
                },
              })}
            </main>
          </SidebarInset>
        </SidebarProvider>
      </TooltipProvider>
    </CoachEnabled.Provider>
  );
}
