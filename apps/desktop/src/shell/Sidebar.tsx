import { ShieldCheck } from 'lucide-react';
import type { JSX } from 'react';

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar';
import { cn } from '@/lib/utils';

import type { ConnectionState } from '../daemon/client.js';
import { BrandMark } from './BrandMark.js';
import { NAV_SECTIONS } from './navigation.js';

interface AppSidebarProps {
  readonly activeId: string;
  readonly onSelect: (id: string) => void;
  readonly state: ConnectionState;
}

interface LockView {
  readonly tone: string;
  readonly headline: string;
  readonly caption: string;
}

/**
 * The badge reports the daemon's answer, including when there is no answer.
 * A Local Lock indicator hardcoded to "ON" would be worse than none: it would
 * claim a guarantee nobody checked.
 */
function describeLock(state: ConnectionState): LockView {
  if (state.status !== 'connected') {
    return {
      tone: 'text-[var(--cm-text-muted)]',
      headline: 'Local Lock · unknown',
      caption: 'daemon not connected',
    };
  }
  return state.localLock
    ? {
        tone: 'text-[var(--color-success)]',
        headline: 'Local Lock · ON',
        caption: '0 bytes left this device',
      }
    : {
        tone: 'text-[var(--color-warning)]',
        headline: 'Local Lock · OFF',
        caption: 'egress is permitted',
      };
}

export function AppSidebar({ activeId, onSelect, state }: AppSidebarProps): JSX.Element {
  const lock = describeLock(state);

  return (
    <Sidebar
      collapsible="none"
      // One continuous glass surface touching the viewport edges: square outer
      // corners, a single hairline on the right, and no floating-island shadow.
      className="glass h-full rounded-none border-y-0 border-l-0 shadow-none"
    >
      <SidebarHeader className="h-16 flex-row items-center gap-2 px-5 pt-[18px]">
        <BrandMark />
        <span className="text-body font-(--cm-weight-wordmark) tracking-[0.2em]">CLIPMILL</span>
      </SidebarHeader>

      <SidebarContent className="px-3">
        <SidebarMenu className="gap-1">
          {NAV_SECTIONS.map((section) => {
            const SectionIcon = section.icon;
            const active = section.id === activeId;
            return (
              <SidebarMenuItem key={section.id}>
                <SidebarMenuButton
                  isActive={active}
                  onClick={() => {
                    onSelect(section.id);
                  }}
                  // 36px rows with a 2px indigo indicator inset on the left.
                  className={cn(
                    'nav-row h-9 gap-2.5 px-2.5 text-body font-(--cm-weight-label)',
                    active ? 'text-[var(--cm-text-primary)]' : 'text-[var(--cm-text-secondary)]',
                  )}
                >
                  <SectionIcon
                    className={cn('size-[18px]', active && 'text-[var(--color-primary)]')}
                  />
                  <span>{section.label}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            );
          })}
        </SidebarMenu>
      </SidebarContent>

      <SidebarFooter className="mx-4 mb-4 gap-0 border-t border-[var(--cm-glass-border)] px-0 pt-3">
        <div
          className={cn(
            'flex items-center gap-1.5 text-meta font-(--cm-weight-heading)',
            lock.tone,
          )}
        >
          <ShieldCheck className="size-3.5" />
          {lock.headline}
        </div>
        <span className="mono mt-[3px] text-technical text-[var(--cm-text-muted)]">
          {lock.caption}
        </span>
        <span className="mt-1.5 text-[9px] uppercase tracking-[0.08em] text-[var(--cm-text-muted)]">
          Local-first session
        </span>
      </SidebarFooter>
    </Sidebar>
  );
}
