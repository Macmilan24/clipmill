import type { JSX } from 'react';

import type { ConnectionState } from '../daemon/client.js';
import { MarkIcon, ShieldCheckIcon } from './icons.js';
import { NAV_SECTIONS } from './navigation.js';

interface SidebarProps {
  readonly activeId: string;
  readonly onSelect: (id: string) => void;
  readonly state: ConnectionState;
}

interface LockView {
  readonly className: string;
  readonly headline: string;
  readonly caption: string;
}

/**
 * The badge reports the daemon's answer, including when there is no answer.
 * A Local Lock indicator that is hardcoded to "ON" would be worse than none:
 * it would claim a guarantee nobody checked.
 */
function describeLock(state: ConnectionState): LockView {
  if (state.status !== 'connected') {
    return {
      className: 'muted',
      headline: 'Local Lock · unknown',
      caption: 'daemon not connected',
    };
  }
  return state.localLock
    ? {
        className: 'success',
        headline: 'Local Lock · ON',
        caption: '0 bytes left this device',
      }
    : {
        className: 'warning',
        headline: 'Local Lock · OFF',
        caption: 'egress is permitted',
      };
}

export function Sidebar({ activeId, onSelect, state }: SidebarProps): JSX.Element {
  const lock = describeLock(state);

  return (
    <aside className="sidebar glass">
      <div className="wordmark">
        <MarkIcon />
        <span>CLIPMILL</span>
      </div>

      <nav className="nav" aria-label="Sections">
        {NAV_SECTIONS.map((section) => {
          const SectionIcon = section.icon;
          const active = section.id === activeId;
          return (
            <button
              key={section.id}
              type="button"
              className="nav-item"
              aria-current={active ? 'page' : undefined}
              onClick={() => {
                onSelect(section.id);
              }}
            >
              <SectionIcon className="nav-icon" />
              {section.label}
            </button>
          );
        })}
      </nav>

      <div className="local-lock">
        <div className={`local-lock-row ${lock.className}`}>
          <ShieldCheckIcon />
          {lock.headline}
        </div>
        <span className="local-lock-caption mono">{lock.caption}</span>
        <span className="local-lock-session">Local-first session</span>
      </div>
    </aside>
  );
}
