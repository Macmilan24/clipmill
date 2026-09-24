import { Film } from 'lucide-react';
import { type JSX, useState } from 'react';

import { StatusBadge } from '@/components/StatusBadge';
import { cn } from '@/lib/utils';

import { ProjectActions, type ProjectActionsProps } from './ProjectActions.js';
import {
  EM_DASH,
  type LibraryProject,
  describeActivity,
  describeStatus,
  formatDuration,
  formatRelative,
} from './model.js';

/**
 * A frame from the recording, or an honest blank.
 *
 * The image is served by the media protocol, which means the daemon has already
 * decided this project may see it. A load failure is still possible — an object
 * collected between the resolve and the fetch — and it falls back to the same
 * placeholder an un-ingested project gets, because "no frame yet" and "the frame
 * would not load" look the same to someone scanning a grid.
 */
function Thumbnail({
  src,
  duration,
}: {
  readonly src: string | null;
  readonly duration: string;
}): JSX.Element {
  const [broken, setBroken] = useState(false);
  const usable = src !== null && !broken;

  return (
    <div className="project-film-frame relative aspect-video w-full overflow-hidden rounded-[4px] bg-[var(--cm-viewer)] text-[var(--cm-viewer-ink)]">
      {usable ? (
        <img
          src={src}
          alt=""
          loading="lazy"
          className="size-full object-cover"
          onError={() => {
            setBroken(true);
          }}
        />
      ) : (
        <div className="flex size-full items-center justify-center">
          <Film className="size-6 opacity-60" />
        </div>
      )}
      {duration === EM_DASH ? null : (
        <span className="mono absolute right-2 bottom-2 rounded-[3px] bg-[var(--cm-viewer)] px-1.5 py-0.5 text-technical text-[var(--cm-viewer-ink)]">
          {duration}
        </span>
      )}
    </div>
  );
}

export function ProjectCard({
  entry,
  onOpen,
  actions,
}: {
  readonly entry: LibraryProject;
  readonly onOpen: (entry: LibraryProject) => void;
  /** The project's menu; absent where the card is only a way in. */
  readonly actions?: Omit<ProjectActionsProps, 'entry' | 'onOpen'>;
}): JSX.Element {
  const status = describeStatus(entry.status);
  const activity = describeActivity(entry.status);

  return (
    <div className="project-film-card group relative min-w-0 rounded-[6px] p-1">
      <button
        type="button"
        onClick={() => {
          onOpen(entry);
        }}
        className="block w-full min-w-0 rounded-[5px] text-left focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
      >
        <div className="relative">
          <Thumbnail src={entry.thumbnail} duration={formatDuration(entry.sourceMap)} />
        </div>

        <div className="border-b border-[var(--cm-glass-border)] px-1 pt-3 pb-3">
          <h2
            className="line-clamp-2 pr-7 text-card-title font-(--cm-weight-heading) leading-snug"
            title={entry.project.name}
          >
            {entry.project.name}
          </h2>
          {activity !== null && (
            <p
              className={cn(
                'mono mt-1 truncate text-meta',
                activity === null
                  ? 'text-[var(--cm-text-secondary)]'
                  : 'text-[var(--color-primary)]',
              )}
            >
              {activity}
            </p>
          )}

          <div className="mt-3 flex items-center justify-between gap-2">
            <StatusBadge tone={status.tone}>{status.label}</StatusBadge>
            <span className="text-meta text-[var(--cm-text-muted)]">
              {formatRelative(entry.project.createdUnixMillis)}
            </span>
          </div>
        </div>
      </button>
      {actions && (
        <div className="project-card-actions">
          <ProjectActions entry={entry} onOpen={onOpen} {...actions} />
        </div>
      )}
    </div>
  );
}
