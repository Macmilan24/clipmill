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
      caption: 'Waiting for the local engine',
    };
  }
  return state.localLock
    ? {
        tone: 'text-[var(--color-success)]',
        headline: 'Local Lock · ON',
        caption: 'No cloud tasks this session',
      }
    : {
        tone: 'text-[var(--color-warning)]',
        headline: 'Local Lock · OFF',
        caption: 'Cloud used this session',
      };
}

export function AppSidebar({ activeId, onSelect, state }: AppSidebarProps): JSX.Element {
  const lock = describeLock(state);

  return (
    <Sidebar
      collapsible="none"
      // One continuous glass surface touching the viewport edges: square outer
      // corners, a single hairline on the right, and no floating-island shadow.
      className="studio-sidebar glass h-full rounded-none border-y-0 border-l-0 shadow-none"
    >
      <SidebarHeader className="h-16 flex-row items-center gap-2.5 px-5">
        <BrandMark />
        <span className="sidebar-wordmark text-[15px] font-semibold tracking-tight">ClipMill</span>
      </SidebarHeader>

      <SidebarContent className="px-3">
        <span className="sidebar-label px-2.5 pb-3 pt-4 text-[10px] font-medium tracking-widest text-[var(--cm-text-muted)] uppercase">
          Workspace
        </span>
        <SidebarMenu className="gap-1">
          {['library', 'new-project', 'results', 'editor', 'export', 'models', 'settings']
            .map((id) => NAV_SECTIONS.find((section) => section.id === id)!)
            .map((section) => {
              const SectionIcon = section.icon;
              const active = section.id === activeId;
              return (
                <SidebarMenuItem
                  key={section.id}
                  className={
                    section.id === 'models'
                      ? 'mt-7 border-t border-[var(--cm-glass-border)] pt-4'
                      : undefined
                  }
                >
                  <SidebarMenuButton
                    isActive={active}
                    aria-label={section.label}
                    aria-current={active ? 'page' : undefined}
                    title={section.label}
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
                    <span className="sidebar-label">{section.label}</span>
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
          <span className="sidebar-label">{lock.headline}</span>
        </div>
        <span className="sidebar-label mt-1 text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
          {lock.caption}
        </span>
      </SidebarFooter>
    </Sidebar>
  );
}
