import type { JSX } from 'react';

import { StatusBadge } from '@/components/StatusBadge';
import { TableCell, TableRow } from '@/components/ui/table';

import { formatBytes } from '../deviceProfile.js';
import {
  EM_DASH,
  type LibraryProject,
  describeActivity,
  describeStatus,
  formatDuration,
  formatRelative,
  formatVideoSpec,
} from './model.js';

/**
 * The same project, as a row.
 *
 * It shows what the card shows and one thing the card cannot: every project's
 * numbers line up in a column, which is the only reason to have a list view of a
 * grid. No thumbnail — at row height a frame is a smudge, and fetching one per
 * row to render it 24px tall would be work spent on nothing.
 */
export function ProjectRow({
  entry,
  onOpen,
}: {
  readonly entry: LibraryProject;
  readonly onOpen: (entry: LibraryProject) => void;
}): JSX.Element {
  const status = describeStatus(entry.status);
  const activity = describeActivity(entry.status);

  return (
    <TableRow
      tabIndex={0}
      role="button"
      className="cursor-pointer"
      onClick={() => {
        onOpen(entry);
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          onOpen(entry);
        }
      }}
    >
      <TableCell className="max-w-0">
        <div className="truncate font-(--cm-weight-label)">{entry.project.name}</div>
        {activity === null ? null : (
          <div className="mono truncate text-technical text-[var(--color-primary)]">{activity}</div>
        )}
      </TableCell>
      <TableCell className="mono text-[var(--cm-text-secondary)]">
        {formatDuration(entry.sourceMap)}
      </TableCell>
      <TableCell className="mono text-[var(--cm-text-secondary)]">
        {formatVideoSpec(entry.sourceMap)}
      </TableCell>
      <TableCell className="mono text-[var(--cm-text-secondary)]">
        {entry.source === null ? EM_DASH : formatBytes(entry.source.byteSize)}
      </TableCell>
      <TableCell className="mono text-[var(--cm-text-muted)]">
        {formatRelative(entry.project.createdUnixMillis)}
      </TableCell>
      <TableCell className="text-right">
        <StatusBadge tone={status.tone}>{status.label}</StatusBadge>
      </TableCell>
    </TableRow>
  );
}
