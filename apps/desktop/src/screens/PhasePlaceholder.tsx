import { Film } from 'lucide-react';
import type { JSX } from 'react';

import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty';

import type { NavSection } from '../shell/navigation.js';

/**
 * Placeholder for planned sections, using their registered availability metadata.
 */
export function PhasePlaceholder({ section }: { readonly section: NavSection }): JSX.Element {
  if (section.availability.kind === 'live') {
    throw new Error(`${section.id} is live and must render its own screen`);
  }
  const { phase, summary } = section.availability;

  return (
    <>
      <div className="mb-4 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-page-title font-(--cm-weight-heading) tracking-[-0.01em]">
            {section.breadcrumb}
          </h1>
          <p className="mt-1 text-meta text-[var(--cm-text-secondary)]">
            Not built yet — and deliberately not mocked.
          </p>
        </div>
      </div>

      <Empty className="glass rounded-xl" aria-label={`${section.label} is not available yet`}>
        <EmptyHeader>
          <EmptyMedia variant="icon" className="glass-elevated size-14 rounded-full">
            <Film className="size-6" />
          </EmptyMedia>
          <EmptyTitle className="text-card-title">Arrives in Phase {phase}</EmptyTitle>
          <EmptyDescription>{summary}</EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <p className="mono text-technical text-[var(--cm-text-muted)]">
            phase 0 · harness only · contracts, daemon, artifacts, workers
          </p>
        </EmptyContent>
      </Empty>
    </>
  );
}
