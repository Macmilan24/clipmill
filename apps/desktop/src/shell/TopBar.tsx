import type { JSX } from 'react';

import type { DeviceProfile } from '@clipmill/contracts';
import type { Theme } from '@clipmill/tokens';

import { formatBytes } from '../deviceProfile.js';
import type { ConnectionState } from '../daemon/client.js';
import { MoonIcon, SunIcon } from './icons.js';

interface TopBarProps {
  readonly breadcrumb: string;
  readonly theme: Theme;
  readonly onToggleTheme: () => void;
  readonly state: ConnectionState;
  readonly profile: DeviceProfile | null;
}

function statusLabel(state: ConnectionState): { text: string; className: string } {
  switch (state.status) {
    case 'connected':
      return { text: `daemon ${state.daemonVersion}`, className: 'success' };
    case 'connecting':
      return { text: 'connecting', className: 'warning' };
    default:
      return { text: 'daemon offline', className: 'danger' };
  }
}

/**
 * The design's top-right cluster shows live GPU load and temperature. Phase 0
 * measures memory but samples nothing continuously, so this renders the memory
 * it genuinely knows and leaves the rest out rather than animating a fiction.
 */
export function TopBar({
  breadcrumb,
  theme,
  onToggleTheme,
  state,
  profile,
}: TopBarProps): JSX.Element {
  const status = statusLabel(state);
  const total = profile?.memory.total_bytes;
  const available = profile?.phase0?.available_memory_bytes;
  const used = total !== undefined && available !== undefined ? total - available : undefined;
  const ratio = used !== undefined && total !== undefined && total > 0 ? used / total : 0;

  return (
    <header className="topbar glass">
      <span className="breadcrumb">{breadcrumb}</span>

      <div className="system-cluster">
        {used === undefined ? null : (
          <span className="meter">
            <span className="mono">
              RAM {formatBytes(used)}/{formatBytes(total)}
            </span>
            <span className="meter-track">
              <span className="meter-fill" style={{ width: `${Math.round(ratio * 100)}%` }} />
            </span>
          </span>
        )}

        <span className={`mono ${status.className}`}>{status.text}</span>

        <button
          type="button"
          className="icon-button"
          onClick={onToggleTheme}
          aria-label={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
          title={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
        >
          {theme === 'dark' ? <SunIcon /> : <MoonIcon />}
        </button>

        <span className="avatar" aria-hidden="true">
          S
        </span>
      </div>
    </header>
  );
}
