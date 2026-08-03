import { type CSSProperties, type JSX, useCallback, useEffect, useMemo, useState } from 'react';

import type { DeviceProfile } from '@clipmill/contracts';
import { type Theme, ThemeController } from '@clipmill/tokens';

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
import { TopBar } from './shell/TopBar.js';
import {
  DEFAULT_ROUTE,
  type Route,
  inspectorRoute,
  placementOf,
  sectionRoute,
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

  const [route, setRoute] = useState<Route>(DEFAULT_ROUTE);
  const [state, setState] = useState<ConnectionState>({ status: 'connecting' });
  const [profile, setProfile] = useState<DeviceProfile | null>(null);
  const [artifactId, setArtifactId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // Apply the resolved theme before the user touches anything.
  useEffect(() => {
    controller.apply(theme);
  }, [controller, theme]);

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

  const navigate = useCallback((sectionId: string) => {
    setRoute(sectionRoute(sectionId));
  }, []);

  // The run a screen opened, and the section it was opened from — which is the
  // row the sidebar keeps lit while it is on screen.
  const openAnalysis = useCallback((projectId: string, jobId: string, from: string) => {
    setRoute({ kind: 'analysis', projectId, jobId, from });
  }, []);

  const { section, trail } = placementOf(route);

  return (
    <TooltipProvider delayDuration={300}>
      <div className="ambient" aria-hidden="true" />
      <SidebarProvider
        // The shell is fixed at the design's width and never collapses; the
        // provider is here for the menu primitives, not for responsiveness.
        style={{ '--sidebar-width': 'var(--cm-shell-sidebar-width)' } as CSSProperties}
        className="relative z-1 h-full min-h-0"
      >
        <AppSidebar activeId={section.id} onSelect={navigate} state={state} />
        <SidebarInset className="min-w-0 bg-transparent">
          <TopBar
            trail={trail}
            theme={theme}
            onToggleTheme={toggleTheme}
            state={state}
            profile={profile}
          />
          <main className="min-h-0 flex-1 overflow-y-auto p-6">
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
                  openAnalysis(projectId, jobId, 'new-project');
                },
              },
              analysis: {
                profile,
                onBack: () => {
                  navigate(route.kind === 'analysis' ? route.from : 'library');
                },
                onNavigate: navigate,
              },
              results: {
                onInspect: (projectId, sourceId, candidateId) => {
                  setRoute(inspectorRoute(projectId, sourceId, candidateId));
                },
                onBack: () => {
                  navigate('results');
                },
              },
              editor: {
                onOpenResults: () => {
                  navigate('results');
                },
              },
              // Both read the daemon directly and take nothing from the shell,
              // so the entry exists to satisfy the registry rather than to
              // carry anything.
              export: {},
              settings: {},
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
  );
}
