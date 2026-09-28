/**
 * The clip's history, and the clip's name.
 *
 * History lists every edit the clip has had, newest first, across sessions:
 * the daemon logged each with its inverse. Going back to a point is one edit,
 * so it can itself be undone.
 *
 * The title is the clip's own. Named here, it is an edit like any other; left
 * empty, the clip goes back to the headline its analysis gave it.
 */
import { History as HistoryIcon, RotateCcw } from 'lucide-react';
import { type JSX, useEffect, useRef, useState } from 'react';

import { Button } from '../components/ui/button.js';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '../components/ui/sheet.js';
import type { EditCommandJson } from '../daemon/client.js';
import { TipButton } from '../inspector/TipButton.js';
import { type HistoryStep, revertTo } from './history.js';
import { setTitle } from './commands.js';

export function HistoryButton({
  revision,
  busy,
  onLoad,
  onApply,
}: {
  /** The document's revision now, so the list is re-read after each edit. */
  readonly revision: number;
  readonly busy: boolean;
  readonly onLoad: () => Promise<readonly HistoryStep[]>;
  readonly onApply: (command: EditCommandJson) => void;
}): JSX.Element {
  const [open, setOpen] = useState(false);
  const [steps, setSteps] = useState<readonly HistoryStep[] | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  useEffect(() => {
    if (!open) return undefined;
    let live = true;
    setProblem(null);
    onLoad()
      .then((loaded) => {
        if (live) setSteps(loaded);
      })
      .catch(() => {
        if (live) setProblem('The history could not be read. Close this and try again.');
      });
    return () => {
      live = false;
    };
  }, [open, onLoad, revision]);
  const newest = steps?.at(-1)?.revision ?? 0;
  return (
    <>
      <TipButton label="History" pressed={open} onClick={() => setOpen(true)}>
        <HistoryIcon className="size-4" />
      </TipButton>
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetContent className="w-full gap-0 overflow-hidden bg-[var(--cm-glass)] sm:max-w-[380px]">
          <SheetHeader className="shrink-0 border-b border-[var(--cm-glass-border)] p-5 pr-10">
            <SheetTitle className="flex items-center gap-2">
              <HistoryIcon className="size-4" />
              History
            </SheetTitle>
            <SheetDescription className="text-xs leading-relaxed">
              Every edit this clip has had, newest first. Going back is one edit, so it can be
              undone too.
            </SheetDescription>
          </SheetHeader>
          <ol className="edit-history" aria-label="Edits, newest first">
            {problem && <li className="edit-history-note">{problem}</li>}
            {steps?.length === 0 && (
              <li className="edit-history-note">No edits yet. This is the clip as it was cut.</li>
            )}
            {steps && steps.length > 0 && (
              <>
                {steps.toReversed().map((step) => (
                  <li key={step.revision} data-current={step.revision === newest || undefined}>
                    <span className="edit-history-label">{step.label}</span>
                    <time
                      className="edit-history-time"
                      dateTime={new Date(step.appliedUnixMillis).toISOString()}
                    >
                      {whenText(step.appliedUnixMillis)}
                    </time>
                    {step.revision === newest ? (
                      <span className="edit-history-now">Now</span>
                    ) : (
                      <Button
                        size="xs"
                        variant="ghost"
                        disabled={busy}
                        onClick={() => {
                          const command = revertTo(steps, step.revision);
                          if (command) onApply(command);
                        }}
                      >
                        <RotateCcw className="size-3" aria-hidden="true" />
                        Back to here
                      </Button>
                    )}
                  </li>
                ))}
                <li>
                  <span className="edit-history-label">As it was cut</span>
                  <span className="edit-history-time" />
                  <Button
                    size="xs"
                    variant="ghost"
                    disabled={busy}
                    onClick={() => {
                      const command = revertTo(steps, 0);
                      if (command) onApply(command);
                    }}
                  >
                    <RotateCcw className="size-3" aria-hidden="true" />
                    Back to here
                  </Button>
                </li>
              </>
            )}
          </ol>
        </SheetContent>
      </Sheet>
    </>
  );
}

/** "Just now", "5 min ago", "Yesterday 18:04", or a date. */
function whenText(unixMillis: number): string {
  const seconds = Math.max(0, (Date.now() - unixMillis) / 1000);
  if (seconds < 60) return 'Just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)} min ago`;
  const at = new Date(unixMillis);
  const time = at.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  if (seconds < 86_400 && at.getDate() === new Date().getDate()) return time;
  if (seconds < 2 * 86_400) return `Yesterday ${time}`;
  return at.toLocaleDateString([], { day: 'numeric', month: 'short' });
}

/** The clip's name, renamed in place. */
export function ClipTitle({
  title,
  fallback,
  busy,
  onApply,
}: {
  /** The name somebody gave the clip, when they did. */
  readonly title: string | null;
  /** What it is called otherwise: its headline, or the route's label. */
  readonly fallback: string;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
}): JSX.Element {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState('');
  const input = useRef<HTMLInputElement | null>(null);
  const shown = title ?? fallback;
  const finish = (save: boolean) => {
    setEditing(false);
    if (!save) return;
    const next = draft.trim();
    if (next === (title ?? '')) return;
    if (next === '' || next === fallback) {
      if (title !== null) onApply(setTitle(null));
      return;
    }
    onApply(setTitle(next.slice(0, 120)));
  };
  if (editing) {
    return (
      <input
        ref={input}
        className="edit-title-input"
        aria-label="Clip title"
        value={draft}
        maxLength={120}
        autoFocus
        onChange={(event) => setDraft(event.target.value)}
        onBlur={() => finish(true)}
        onKeyDown={(event) => {
          if (event.key === 'Enter') finish(true);
          if (event.key === 'Escape') finish(false);
        }}
      />
    );
  }
  return (
    <h1 data-testid="clip-name">
      <button
        type="button"
        className="edit-title-button"
        title="Rename this clip"
        disabled={busy}
        onClick={() => {
          setDraft(shown);
          setEditing(true);
        }}
      >
        {shown}
      </button>
    </h1>
  );
}
