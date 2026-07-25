import { type JSX, useCallback, useEffect, useMemo, useState } from 'react';

import type { DeviceProfile } from '@clipmill/contracts';
import { type Theme, ThemeController } from '@clipmill/tokens';

import {
  type ConnectionState,
  fetchDaemonState,
  fetchDeviceProfile,
  reconnectDaemon,
  subscribeDaemonState,
} from './daemon/client.js';
import { ModelsDevice } from './screens/ModelsDevice.js';
import { PhasePlaceholder } from './screens/PhasePlaceholder.js';
import { Sidebar } from './shell/Sidebar.js';
import { TopBar } from './shell/TopBar.js';
import { DEFAULT_SECTION_ID, findSection } from './shell/navigation.js';

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

  const [theme, setTheme] = useState<Theme>(() => {
    const initial = ThemeController.resolveInitial(
      typeof localStorage === 'undefined' ? null : localStorage,
      typeof matchMedia === 'function' && matchMedia('(prefers-color-scheme: light)').matches,
    );
    return initial;
  });

  const [sectionId, setSectionId] = useState(DEFAULT_SECTION_ID);
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

  const section = findSection(sectionId);

  return (
    <>
      <div className="ambient" aria-hidden="true" />
      <div className="shell">
        <Sidebar activeId={sectionId} onSelect={setSectionId} state={state} />
        <div className="main">
          <TopBar
            breadcrumb={section.breadcrumb}
            theme={theme}
            onToggleTheme={toggleTheme}
            state={state}
            profile={profile}
          />
          <main className="content">
            {section.availability.kind === 'live' ? (
              <ModelsDevice
                state={state}
                profile={profile}
                artifactId={artifactId}
                error={error}
                busy={busy}
                onRescan={() => {
                  void loadProfile(true);
                }}
                onReconnect={handleReconnect}
              />
            ) : (
              <PhasePlaceholder section={section} />
            )}
          </main>
        </div>
      </div>
    </>
  );
}
