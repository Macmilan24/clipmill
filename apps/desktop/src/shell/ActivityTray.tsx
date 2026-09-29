/**
 * The activity tray: a button in the top bar that counts the long work
 * running now and opens onto each piece of it, with how far it has come.
 * A row leads to where that work is shown in full.
 */
import { Activity, CloudUpload, Download, FileDown, Film, Package, Sparkles } from 'lucide-react';
import type { JSX } from 'react';

import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Progress } from '@/components/ui/progress';

import type { ActivityItem, ActivityKind, ActivityTarget } from './activity.js';

const ICONS: Readonly<Record<ActivityKind, JSX.Element>> = {
  analysis: <Sparkles aria-hidden="true" />,
  export: <Film aria-hidden="true" />,
  download: <Download aria-hidden="true" />,
  install: <Package aria-hidden="true" />,
  import: <FileDown aria-hidden="true" />,
  upload: <CloudUpload aria-hidden="true" />,
};

export interface ActivityTrayProps {
  readonly items: readonly ActivityItem[];
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
  readonly onOpen: (target: ActivityTarget) => void;
}

export function ActivityTray({
  items,
  open,
  onOpenChange,
  onOpen,
}: ActivityTrayProps): JSX.Element {
  const running = items.length;
  return (
    <Popover open={open} onOpenChange={onOpenChange}>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="activity-trigger"
          aria-label={running === 0 ? 'Activity: nothing running' : `Activity: ${running} running`}
        >
          <Activity aria-hidden="true" />
          {running > 0 && <span className="activity-count mono">{running}</span>}
        </Button>
      </PopoverTrigger>
      <PopoverContent aria-label="Activity">
        <div className="border-b border-[var(--cm-glass-border)] px-4 py-3">
          <h2 className="text-xs font-semibold">Activity</h2>
          <p className="mt-0.5 text-[11px] text-[var(--cm-text-secondary)]">
            {running === 0
              ? 'Nothing is running.'
              : `${running} running. Each keeps going when you move to another screen.`}
          </p>
        </div>
        {running === 0 ? (
          <p className="px-4 py-4 text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
            Analyses, exports, model downloads, installs, imports and uploads show here while they
            run.
          </p>
        ) : (
          <ul className="max-h-80 divide-y divide-[var(--cm-glass-border)] overflow-y-auto">
            {items.map((item) => (
              <li key={item.id}>
                <button
                  type="button"
                  className="activity-row"
                  onClick={() => {
                    onOpenChange(false);
                    onOpen(item.target);
                  }}
                >
                  <span className="activity-icon">{ICONS[item.kind]}</span>
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-xs font-medium">{item.title}</span>
                    <span className="block truncate text-[11px] text-[var(--cm-text-secondary)]">
                      {item.detail}
                    </span>
                    {item.fraction !== null && (
                      <Progress
                        value={Math.round(item.fraction * 100)}
                        className="mt-1.5 h-1"
                        aria-label={`${item.title}: ${Math.round(item.fraction * 100)}%`}
                      />
                    )}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </PopoverContent>
    </Popover>
  );
}
