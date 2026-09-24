/**
 * Document picker for Editor and Export routes without a clip.
 * Rows show the clip's title and a frame of it, its project and when it was
 * last edited, newest first. Opening a row passes the full clip identity.
 */
import { Film, Scissors } from 'lucide-react';

import { Button } from '../components/ui/button.js';
import { Spinner } from '../components/ui/spinner.js';
import type { ClipRef } from '../shell/route.js';
import type { DocumentList } from './documents.js';

export interface DocumentPickerProps {
  readonly documents: DocumentList;
  /** What opening a row does, in the verb the screen uses. */
  readonly verb: string;
  readonly onOpen: (clip: ClipRef) => void;
}

function edited(millis: number): string {
  return new Date(millis).toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function DocumentPicker({ documents, verb, onOpen }: DocumentPickerProps) {
  if (documents.loading) {
    return (
      <p className="flex items-center gap-2 text-xs text-[var(--cm-ink-2)]">
        <Spinner className="size-3" /> Looking for edits…
      </p>
    );
  }
  if (documents.problem) {
    return <p className="text-xs text-[var(--cm-danger-ink)]">{documents.problem}</p>;
  }
  if (documents.entries.length === 0) {
    return null;
  }
  return (
    <ul className="flex w-full max-w-xl flex-col gap-2" aria-label="Edits">
      {documents.entries.map((entry) => (
        <li
          key={entry.clip.docId}
          className="flex items-center gap-3 rounded-lg border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] px-3 py-2 text-left"
        >
          <span className="grid h-10 w-[72px] shrink-0 place-items-center overflow-hidden rounded-[4px] bg-[var(--cm-viewer)] text-[var(--cm-viewer-ink)]">
            {entry.thumbnail ? (
              <img src={entry.thumbnail} alt="" className="size-full object-cover" />
            ) : (
              <Film className="size-4 opacity-60" aria-hidden />
            )}
          </span>
          <span className="flex min-w-0 flex-1 flex-col">
            <span className="truncate text-sm text-[var(--cm-ink-1)]">
              {entry.title ?? entry.sourceName ?? 'Untitled clip'}
            </span>
            <span className="truncate text-xs text-[var(--cm-ink-3)]">
              {entry.projectName} · edited {edited(entry.updatedUnixMillis)}
            </span>
          </span>
          <Button size="sm" variant="outline" onClick={() => onOpen(entry.clip)}>
            <Scissors className="size-3.5" aria-hidden />
            {verb}
          </Button>
        </li>
      ))}
    </ul>
  );
}
