import { Boxes, Check, ChevronDown, FolderOpen, Trash2, TriangleAlert } from 'lucide-react';
import { type JSX, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';

import type { StorageCategory, StorageStats } from '../daemon/client.js';
import type { CleanAction, CleanResult } from '../daemon/models.js';
import { formatBytes } from '../deviceProfile.js';

export const CATEGORY_LABELS: Readonly<Record<string, string>> = {
  artifacts: 'Generated media',
  models: 'Model weights',
  state: 'Project state',
  imports: 'Imported originals',
  backups: 'Database backups',
  temporary: 'Temporary files',
};
const CATEGORY_NOTES: Readonly<Record<string, string>> = {
  artifacts:
    'Proxies, transcripts, analysis and rendered media. Files no project uses can be cleaned up; the rest are kept.',
  models:
    'Model files, downloaded once and reused across projects. Download and remove them in Models.',
  state: 'Projects, saved edits and decisions. Keep these files to preserve your work.',
  imports:
    'Downloaded originals are kept with their project. Deleting the project removes its managed copies.',
  backups:
    'Copies of the project database taken before each ClipMill update. The newest is always kept.',
  temporary:
    'Scratch left behind by stopped work and interrupted model downloads. Work in progress is not touched.',
};
export const CATEGORY_COLORS: Readonly<Record<string, string>> = {
  artifacts: 'var(--cm-text-secondary)',
  models: 'var(--cm-text-muted)',
  state: 'var(--cm-text-primary)',
  imports: 'var(--color-primary)',
  backups: 'color-mix(in srgb, var(--cm-text-muted) 60%, transparent)',
  temporary: 'color-mix(in srgb, var(--cm-text-secondary) 45%, transparent)',
};

/** What each category's clean-up is called, and what it would free now. */
function cleanUpFor(
  category: StorageCategory,
  storage: StorageStats,
): { action: CleanAction; label: string; question: string; freeable: boolean } | null {
  switch (category.key) {
    case 'artifacts': {
      const bytes = storage.reclaimableBytes;
      const items = storage.reclaimableItems ?? 0;
      return {
        action: 'unused_files',
        label: bytes !== undefined && bytes > 0 ? `Free ${formatBytes(bytes)}` : 'Clean up unused',
        question:
          bytes !== undefined
            ? `Remove ${items} generated ${items === 1 ? 'file' : 'files'} no project uses (${formatBytes(bytes)})? Files your projects use are kept, and anything removed can be made again by analysing.`
            : 'Remove generated files no project uses? Files your projects use are kept, and anything removed can be made again by analysing.',
        freeable: bytes === undefined || bytes > 0,
      };
    }
    case 'backups':
      return {
        action: 'backups',
        label: 'Delete old backups',
        question: `Delete ${Math.max(0, category.items - 1)} older database ${category.items - 1 === 1 ? 'backup' : 'backups'}? The newest is kept.`,
        freeable: category.items > 1,
      };
    case 'temporary':
      return {
        action: 'temporary',
        label: 'Clear',
        question:
          'Clear temporary files? Scratch that is still being written and downloads in progress are kept.',
        freeable: category.bytes > 0,
      };
    default:
      return null;
  }
}

/**
 * The storage categories, each with what can be done about it.
 *
 * Clean-ups confirm in place, saying what goes and what stays, and report what
 * they freed in the daemon's figures. Model weights are removed in Models,
 * where a removal can say which job loses its model.
 */
export function StorageCategories({
  storage,
  onClean,
  onOpen,
  onOpenModels,
}: {
  readonly storage: StorageStats;
  readonly onClean?: ((action: CleanAction) => Promise<CleanResult>) | undefined;
  readonly onOpen?: ((key: string) => Promise<void>) | undefined;
  readonly onOpenModels?: (() => void) | undefined;
}): JSX.Element {
  const [confirming, setConfirming] = useState<CleanAction | null>(null);
  const [pending, setPending] = useState<CleanAction | null>(null);
  const [outcome, setOutcome] = useState<{
    readonly key: string;
    readonly text: string;
    readonly failed: boolean;
  } | null>(null);

  const clean = async (key: string, action: CleanAction) => {
    if (!onClean) return;
    setConfirming(null);
    setPending(action);
    setOutcome(null);
    try {
      const result = await onClean(action);
      setOutcome({
        key,
        failed: false,
        text:
          result.freedBytes > 0
            ? `Freed ${formatBytes(result.freedBytes)} (${result.removedItems} ${result.removedItems === 1 ? 'item' : 'items'}).`
            : 'Nothing needed cleaning up.',
      });
    } catch (cause) {
      setOutcome({
        key,
        failed: true,
        text: cause instanceof Error ? cause.message : String(cause),
      });
    } finally {
      setPending(null);
    }
  };

  const open = async (key: string) => {
    if (!onOpen) return;
    try {
      await onOpen(key);
    } catch (cause) {
      setOutcome({
        key,
        failed: true,
        text: cause instanceof Error ? cause.message : String(cause),
      });
    }
  };

  return (
    <ul className="mt-4 divide-y divide-[var(--cm-glass-border)]">
      {storage.categories.map((category) => {
        const label = CATEGORY_LABELS[category.key] ?? category.key;
        const cleanUp = cleanUpFor(category, storage);
        return (
          <li key={category.key} className="py-4 first:pt-0" aria-label={label}>
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div className="min-w-0">
                <h3 className="flex items-center gap-2 text-xs font-semibold">
                  <span
                    aria-hidden="true"
                    className="size-1.5 rounded-full"
                    style={{
                      backgroundColor: CATEGORY_COLORS[category.key] ?? 'var(--cm-text-muted)',
                    }}
                  />
                  {label}
                </h3>
                <p className="mt-1 font-mono text-[11px] text-[var(--cm-text-muted)]">
                  {formatBytes(category.bytes)} · {category.items}{' '}
                  {category.items === 1 ? 'item' : 'items'}
                </p>
              </div>
              <div className="flex flex-wrap items-center justify-end gap-1.5">
                {category.key === 'models' && onOpenModels && (
                  <Button size="sm" variant="outline" onClick={onOpenModels}>
                    <Boxes />
                    Manage in Models
                  </Button>
                )}
                {cleanUp !== null && onClean && (
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={!cleanUp.freeable || pending !== null || confirming !== null}
                    onClick={() => setConfirming(cleanUp.action)}
                    aria-label={`${cleanUp.label} in ${label}`}
                  >
                    {pending === cleanUp.action ? <Spinner /> : <Trash2 />}
                    {cleanUp.label}
                  </Button>
                )}
                {onOpen && (
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => void open(category.key)}
                    aria-label={`Open the ${label.toLowerCase()} folder`}
                  >
                    <FolderOpen />
                    Open folder
                  </Button>
                )}
              </div>
            </div>
            {cleanUp !== null && confirming === cleanUp.action && (
              <div
                className="storage-confirm"
                role="group"
                aria-label={`Confirm: ${cleanUp.label}`}
              >
                <p>{cleanUp.question}</p>
                <div className="flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={() => void clean(category.key, cleanUp.action)}
                  >
                    <Trash2 />
                    {cleanUp.action === 'temporary' ? 'Clear' : 'Delete'}
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => setConfirming(null)}>
                    Cancel
                  </Button>
                </div>
              </div>
            )}
            {outcome !== null && outcome.key === category.key && (
              <p
                role="status"
                className={
                  outcome.failed
                    ? 'storage-outcome text-[var(--cm-danger-ink)]'
                    : 'storage-outcome text-[var(--cm-success-ink)]'
                }
              >
                {outcome.failed ? (
                  <TriangleAlert aria-hidden="true" />
                ) : (
                  <Check aria-hidden="true" />
                )}
                {outcome.text}
              </p>
            )}
            <details className="preference-disclosure mt-2">
              <summary>
                <ChevronDown className="size-3" />
                Location and details
                <span className="sr-only"> for {label}</span>
              </summary>
              <p className="mt-3 max-w-[530px] text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
                {CATEGORY_NOTES[category.key] ?? 'Files managed by the local engine.'}
              </p>
              <p className="mt-2 select-all break-all rounded-md bg-[var(--cm-recessed)] px-2.5 py-2 font-mono text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
                {category.path}
              </p>
            </details>
          </li>
        );
      })}
    </ul>
  );
}
