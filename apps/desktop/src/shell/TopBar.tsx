import { CircleArrowUp, Moon, Sun, X } from 'lucide-react';
import { Fragment, type JSX } from 'react';

import type { DeviceProfile } from '@clipmill/contracts';
import type { Theme } from '@clipmill/tokens';

import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';

import type { ConnectionState } from '../daemon/client.js';
import type { UpdateNotice } from './updates.js';

interface TopBarProps {
  /** Outermost first. One part for a section, two for a screen inside one. */
  readonly trail: readonly string[];
  readonly theme: Theme;
  readonly onToggleTheme: () => void;
  readonly state: ConnectionState;
  readonly profile: DeviceProfile | null;
  /** A newer release to tell the person about, when there is one. */
  readonly update?: UpdateNotice | null;
}

function statusLabel(state: ConnectionState): { text: string; tone: string } {
  switch (state.status) {
    case 'connected':
      return { text: 'Engine ready', tone: 'text-[var(--color-success)]' };
    case 'connecting':
      return { text: 'Connecting', tone: 'text-[var(--color-warning)]' };
    default:
      return { text: 'Engine offline', tone: 'text-[var(--color-destructive)]' };
  }
}

/**
 * Breadcrumbs, daemon connection status, and theme controls.
 */
export function TopBar({
  trail,
  theme,
  onToggleTheme,
  state,
  update = null,
}: TopBarProps): JSX.Element {
  const status = statusLabel(state);
  const nextTheme = theme === 'dark' ? 'light' : 'dark';

  return (
    <header className="studio-topbar glass flex h-13 flex-none items-center justify-between rounded-none border-x-0 border-t-0 px-6 shadow-none">
      <Breadcrumb className="min-w-0 overflow-hidden">
        <BreadcrumbList className="flex-nowrap">
          {trail.map((part, index) => (
            <Fragment key={part}>
              {index === 0 ? null : <BreadcrumbSeparator />}
              <BreadcrumbItem>
                <BreadcrumbPage className="max-w-72 truncate text-[12px] text-[var(--cm-text-secondary)]">
                  {part}
                </BreadcrumbPage>
              </BreadcrumbItem>
            </Fragment>
          ))}
        </BreadcrumbList>
      </Breadcrumb>

      <div className="flex items-center gap-4 text-[var(--cm-text-secondary)]">
        {update && (
          <span className="flex items-center rounded-full border border-[var(--cm-glass-border)] pl-2.5 text-[11px]">
            <button
              type="button"
              className="flex items-center gap-1.5 text-[var(--color-primary)] hover:underline"
              title="Opens the release page on GitHub"
              onClick={update.open}
            >
              <CircleArrowUp className="size-3.5" aria-hidden />
              ClipMill {update.version} is out
            </button>
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label={`Not now: hide ClipMill ${update.version}`}
              title="Not now"
              onClick={update.dismiss}
            >
              <X />
            </Button>
          </span>
        )}
        <span
          className="flex items-center gap-2 text-[11px] text-[var(--cm-text-secondary)]"
          title={state.status === 'connected' ? `Local engine ${state.daemonVersion}` : status.text}
        >
          <span className={cn('size-1.5 rounded-full bg-current', status.tone)} aria-hidden />
          {status.text}
        </span>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={onToggleTheme}
              aria-label={`Switch to ${nextTheme} mode`}
            >
              {theme === 'dark' ? <Sun /> : <Moon />}
            </Button>
          </TooltipTrigger>
          <TooltipContent>Switch to {nextTheme} mode</TooltipContent>
        </Tooltip>
      </div>
    </header>
  );
}
