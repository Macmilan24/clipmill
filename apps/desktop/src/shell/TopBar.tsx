import { Moon, Sun } from 'lucide-react';
import { Fragment, type JSX } from 'react';

import type { DeviceProfile } from '@clipmill/contracts';
import type { Theme } from '@clipmill/tokens';

import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';

import type { ConnectionState } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';

interface TopBarProps {
  /** Outermost first. One part for a section, two for a screen inside one. */
  readonly trail: readonly string[];
  readonly theme: Theme;
  readonly onToggleTheme: () => void;
  readonly state: ConnectionState;
  readonly profile: DeviceProfile | null;
}

function statusLabel(state: ConnectionState): { text: string; tone: string } {
  switch (state.status) {
    case 'connected':
      return { text: `daemon ${state.daemonVersion}`, tone: 'text-[var(--color-success)]' };
    case 'connecting':
      return { text: 'connecting', tone: 'text-[var(--color-warning)]' };
    default:
      return { text: 'daemon offline', tone: 'text-[var(--color-destructive)]' };
  }
}

/**
 * The design's top-right cluster shows live GPU load and temperature. Phase 0
 * measures memory but samples nothing continuously, so this renders the memory
 * it genuinely knows and leaves the rest out rather than animating a fiction.
 */
export function TopBar({
  trail,
  theme,
  onToggleTheme,
  state,
  profile,
}: TopBarProps): JSX.Element {
  const status = statusLabel(state);
  const total = profile?.memory.total_bytes;
  const available = profile?.phase0?.available_memory_bytes;
  const used = total !== undefined && available !== undefined ? total - available : undefined;
  const ratio = used !== undefined && total !== undefined && total > 0 ? (used / total) * 100 : 0;
  const nextTheme = theme === 'dark' ? 'light' : 'dark';

  return (
    <header className="glass flex h-13 flex-none items-center justify-between rounded-none border-x-0 border-t-0 px-6 shadow-none">
      <Breadcrumb>
        <BreadcrumbList>
          {trail.map((part, index) => (
            <Fragment key={part}>
              {index === 0 ? null : <BreadcrumbSeparator />}
              <BreadcrumbItem>
                <BreadcrumbPage className="text-body text-[var(--cm-text-secondary)]">
                  {part}
                </BreadcrumbPage>
              </BreadcrumbItem>
            </Fragment>
          ))}
        </BreadcrumbList>
      </Breadcrumb>

      <div className="flex items-center gap-4 text-[var(--cm-text-secondary)]">
        {used === undefined ? null : (
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="flex items-center gap-1.5">
                <span className="mono text-technical">
                  RAM {formatBytes(used)}/{formatBytes(total)}
                </span>
                <Progress value={ratio} className="h-1 w-10" />
              </span>
            </TooltipTrigger>
            <TooltipContent>Measured at the last device profile, not sampled live</TooltipContent>
          </Tooltip>
        )}

        <span className={cn('mono text-technical', status.tone)}>{status.text}</span>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={onToggleTheme}
              aria-label={`Switch to ${nextTheme} theme`}
            >
              {theme === 'dark' ? <Sun /> : <Moon />}
            </Button>
          </TooltipTrigger>
          <TooltipContent>Switch to {nextTheme} theme</TooltipContent>
        </Tooltip>

        <Avatar className="size-7 border border-[var(--cm-glass-border)]">
          <AvatarFallback className="glass-elevated text-meta font-(--cm-weight-heading)">
            S
          </AvatarFallback>
        </Avatar>
      </div>
    </header>
  );
}
