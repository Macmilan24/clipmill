import { ArrowRight, HardDrive } from 'lucide-react';
import type { JSX } from 'react';

import { Button } from '@/components/ui/button';

import type { StorageStats } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';

/**
 * The three categories, in the daemon's order, with the wording chosen here.
 *
 * The daemon sends stable keys and no labels, which is the right split: what a
 * category is called is a screen's business, and what it measures is not.
 */
const LABELS: Readonly<Record<string, string>> = {
  artifacts: 'Artifacts',
  models: 'Models',
  state: 'State',
};

export function StorageStrip({
  stats,
  onManage,
}: {
  readonly stats: StorageStats | null;
  readonly onManage: () => void;
}): JSX.Element {
  const summary =
    stats === null
      ? 'Storage not measured'
      : stats.categories
          .map(
            (category) => `${LABELS[category.key] ?? category.key} ${formatBytes(category.bytes)}`,
          )
          .join(' · ');

  return (
    <div className="glass mt-4 flex h-10 items-center justify-between rounded-xl px-3">
      <div className="flex min-w-0 items-center gap-2">
        <HardDrive className="size-4 shrink-0 text-[var(--cm-text-muted)]" />
        <span className="mono truncate text-technical text-[var(--cm-text-secondary)]">
          {summary}
        </span>
      </div>
      <div className="flex items-center gap-4">
        {/* Absent is not zero. A disk whose free space could not be read says
            nothing rather than claiming to be full. */}
        {stats?.availableBytes === undefined ? null : (
          <span className="mono text-technical text-[var(--cm-text-muted)]">
            {formatBytes(stats.availableBytes)} free
          </span>
        )}
        <Button variant="link" size="sm" className="h-auto p-0" onClick={onManage}>
          Manage storage
          <ArrowRight />
        </Button>
      </div>
    </div>
  );
}
