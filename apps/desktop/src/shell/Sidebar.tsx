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
        caption: 'No network operations this session',
      }
    : {
        tone: 'text-[var(--color-warning)]',
        headline: 'Local Lock · OFF',
        caption: 'Network used this session',
      };
}

export function AppSidebar({ activeId, onSelect, state }: AppSidebarProps): JSX.Element {
  const lock = describeLock(state);

  return (
    <Sidebar
      collapsible="none"
      className="studio-sidebar glass h-full rounded-none border-y-0 border-l-0 shadow-none"
    >
      <SidebarHeader className="h-20 flex-col items-center justify-center gap-1.5 px-2">
        <span className="sidebar-brand-mark">
          <BrandMark size={18} />
        </span>
        <span className="sidebar-wordmark text-[12px] font-medium tracking-tight">ClipMill</span>
      </SidebarHeader>

      <SidebarContent className="px-2">
        <SidebarMenu className="gap-1 pt-3">
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
                      ? 'mt-5 border-t border-[var(--cm-glass-border)] pt-3'
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
                    className={cn(
                      'nav-row h-14 flex-col justify-center gap-1.5 rounded-md px-1 text-[11px] font-normal',
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

      <SidebarFooter className="mx-2 mb-4 gap-0 border-t border-[var(--cm-glass-border)] px-0 pt-3">
        <div
          title={`${lock.headline}. ${lock.caption}`}
          className={cn(
            'flex flex-col items-center gap-1.5 text-center text-[10px] font-(--cm-weight-heading)',
            lock.tone,
          )}
        >
          <ShieldCheck className="size-3.5" />
          <span className="sidebar-label">{lock.headline}</span>
        </div>
        <span className="sr-only">{lock.caption}</span>
      </SidebarFooter>
    </Sidebar>
  );
}
