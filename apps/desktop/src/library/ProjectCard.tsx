import { Film } from 'lucide-react';
import { type JSX, useState } from 'react';

import { StatusBadge } from '@/components/StatusBadge';
import { cn } from '@/lib/utils';

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
    <div className="relative aspect-[352/166] w-full overflow-hidden rounded-[10px] bg-[var(--cm-recessed)]">
      {usable ? (
        <img
          src={src}
          alt=""
          loading="lazy"
          // Desaturated in both themes, per the design: a wall of full-colour
          // stills competes with the interface for attention.
          className="size-full object-cover saturate-[0.72]"
          onError={() => {
            setBroken(true);
          }}
        />
      ) : (
        <div className="flex size-full items-center justify-center">
          <Film className="size-6 text-[var(--cm-text-disabled)]" />
        </div>
      )}
      {duration === EM_DASH ? null : (
        <span className="mono absolute bottom-2 left-2 rounded-[4px] bg-[color-mix(in_srgb,#000_62%,transparent)] px-1.5 py-0.5 text-technical text-white">
          {duration}
        </span>
      )}
    </div>
  );
}

export function ProjectCard({
  entry,
  onOpen,
}: {
  readonly entry: LibraryProject;
  readonly onOpen: (entry: LibraryProject) => void;
}): JSX.Element {
  const status = describeStatus(entry.status);
  const activity = describeActivity(entry.status);
  const spec = formatVideoSpec(entry.sourceMap);
  const size = entry.source === null ? EM_DASH : formatBytes(entry.source.byteSize);

  return (
    <button
      type="button"
      onClick={() => {
        onOpen(entry);
      }}
      // The design's restrained hover: lifted 2px, the surface a step brighter,
      // the shadow a step deeper. Every value comes from a token, so a change to
      // the glass scale moves the resting and hovered states together.
      className="glass block rounded-[var(--cm-radius-card)] p-3 text-left transition-[transform,background-color,box-shadow] hover:-translate-y-0.5 hover:bg-[var(--cm-glass-elevated)] hover:shadow-[0_18px_48px_rgba(0,0,0,0.28)] focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
    >
      <div className="relative">
        <Thumbnail src={entry.thumbnail} duration={formatDuration(entry.sourceMap)} />
        <StatusBadge tone={status.tone} className="glass absolute top-2 right-2">
          {status.label}
        </StatusBadge>
      </div>

      <div className="px-1 pt-3">
        <h2 className="truncate text-card-title font-(--cm-weight-heading)">
          {entry.project.name}
        </h2>
        <p
          className={cn(
            'mono mt-1 truncate text-meta',
            activity === null ? 'text-[var(--cm-text-secondary)]' : 'text-[var(--color-primary)]',
          )}
        >
          {activity ?? `${spec} · ${size}`}
        </p>

        <div className="mt-3 flex items-center justify-between border-t border-[var(--cm-glass-border)] pt-2.5">
          <span className="mono text-technical text-[var(--cm-text-muted)]">
            Created {formatRelative(entry.project.createdUnixMillis)}
          </span>
          {activity === null ? null : (
            <span className="mono text-technical text-[var(--cm-text-muted)]">{spec}</span>
          )}
        </div>
      </div>
    </button>
  );
}
