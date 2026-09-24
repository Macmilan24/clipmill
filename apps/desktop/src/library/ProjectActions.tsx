/**
 * A project's menu in the Library: open it, rename it, show its recording on
 * disk, or delete it. Rename and delete confirm in a dialog; deleting leaves a
 * recording imported from disk where it is.
 */
import { FolderOpen, MoreHorizontal, Pencil, SquareArrowOutUpRight, Trash2 } from 'lucide-react';
import { type JSX, useState } from 'react';

import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';

import type { LibraryProject } from './model.js';

export interface ProjectActionsProps {
  readonly entry: LibraryProject;
  readonly onOpen: (entry: LibraryProject) => void;
  readonly onRename: (entry: LibraryProject, name: string) => Promise<void>;
  readonly onReveal: (path: string) => void;
  readonly onDelete: (entry: LibraryProject) => Promise<void>;
}

/** What the platform calls showing a file in its folder. */
export function revealLabel(): string {
  const platform = typeof navigator === 'undefined' ? '' : navigator.userAgent;
  if (/Mac/i.test(platform)) return 'Show in Finder';
  if (/Windows/i.test(platform)) return 'Show in File Explorer';
  return 'Show in folder';
}

export function ProjectActions({
  entry,
  onOpen,
  onRename,
  onReveal,
  onDelete,
}: ProjectActionsProps): JSX.Element {
  const [dialog, setDialog] = useState<'rename' | 'delete' | null>(null);
  const running = entry.status.kind === 'analyzing' || entry.status.kind === 'queued';
  const path = entry.source?.absolutePath ?? null;
  const name = entry.project.name;

  return (
    <>
      <DropdownMenu modal={false}>
        <DropdownMenuTrigger asChild>
          <button
            type="button"
            className="project-actions-trigger"
            aria-label={`Actions for ${name}`}
            onClick={(event) => event.stopPropagation()}
            onKeyDown={(event) => event.stopPropagation()}
          >
            <MoreHorizontal className="size-4" aria-hidden="true" />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" onClick={(event) => event.stopPropagation()}>
          <DropdownMenuItem onSelect={() => onOpen(entry)}>
            <SquareArrowOutUpRight aria-hidden="true" />
            Open
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => setDialog('rename')}>
            <Pencil aria-hidden="true" />
            Rename…
          </DropdownMenuItem>
          {path !== null && (
            <DropdownMenuItem onSelect={() => onReveal(path)}>
              <FolderOpen aria-hidden="true" />
              {revealLabel()}
            </DropdownMenuItem>
          )}
          <DropdownMenuSeparator />
          <DropdownMenuItem tone="danger" disabled={running} onSelect={() => setDialog('delete')}>
            <Trash2 aria-hidden="true" />
            {running ? 'Delete (stop the analysis first)' : 'Delete…'}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <RenameDialog
        open={dialog === 'rename'}
        name={name}
        onClose={() => setDialog(null)}
        onSave={(next) => onRename(entry, next)}
      />
      <DeleteDialog
        open={dialog === 'delete'}
        name={name}
        imported={path !== null}
        onClose={() => setDialog(null)}
        onDelete={() => onDelete(entry)}
      />
    </>
  );
}

function RenameDialog({
  open,
  name,
  onClose,
  onSave,
}: {
  readonly open: boolean;
  readonly name: string;
  readonly onClose: () => void;
  readonly onSave: (name: string) => Promise<void>;
}): JSX.Element {
  const [draft, setDraft] = useState(name);
  const [saving, setSaving] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const next = draft.trim();
  const changed = next !== '' && next !== name && next.length <= 200;
  return (
    <Dialog
      open={open}
      onOpenChange={(opened) => {
        if (opened) return;
        setDraft(name);
        setProblem(null);
        onClose();
      }}
    >
      <DialogContent onClick={(event) => event.stopPropagation()}>
        <form
          className="grid gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            if (!changed || saving) return;
            setSaving(true);
            setProblem(null);
            onSave(next)
              .then(onClose)
              .catch((error: unknown) =>
                setProblem(error instanceof Error ? error.message : String(error)),
              )
              .finally(() => setSaving(false));
          }}
        >
          <DialogTitle>Rename project</DialogTitle>
          <Input
            aria-label="Project name"
            value={draft}
            maxLength={200}
            autoFocus
            onChange={(event) => setDraft(event.target.value)}
          />
          {problem && (
            <p role="alert" className="text-label text-[var(--cm-danger-ink)]">
              {problem}
            </p>
          )}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={!changed || saving}>
              {saving ? 'Saving…' : 'Rename'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function DeleteDialog({
  open,
  name,
  imported,
  onClose,
  onDelete,
}: {
  readonly open: boolean;
  readonly name: string;
  readonly imported: boolean;
  readonly onClose: () => void;
  readonly onDelete: () => Promise<void>;
}): JSX.Element {
  const [deleting, setDeleting] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  return (
    <Dialog
      open={open}
      onOpenChange={(opened) => {
        if (opened || deleting) return;
        setProblem(null);
        onClose();
      }}
    >
      <DialogContent onClick={(event) => event.stopPropagation()}>
        <DialogTitle>Delete “{name}”?</DialogTitle>
        <DialogDescription>
          Its analysis, clips and edits are removed from ClipMill, and this cannot be undone.
          {imported ? ' The recording itself stays where it is on your disk.' : ''}
        </DialogDescription>
        {problem && (
          <p role="alert" className="text-label text-[var(--cm-danger-ink)]">
            {problem}
          </p>
        )}
        <DialogFooter>
          <Button type="button" variant="ghost" disabled={deleting} onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="button"
            variant="destructive"
            disabled={deleting}
            onClick={() => {
              setDeleting(true);
              setProblem(null);
              onDelete()
                .then(onClose)
                .catch((error: unknown) =>
                  setProblem(error instanceof Error ? error.message : String(error)),
                )
                .finally(() => setDeleting(false));
            }}
          >
            {deleting ? 'Deleting…' : 'Delete project'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
