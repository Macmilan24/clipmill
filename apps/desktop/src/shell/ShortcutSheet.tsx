/**
 * Every key the app answers, in one place, opened with ⌘/ or from Help.
 *
 * The buttons carry no key letters — the app reads like the tools creators
 * already use, which do not print them either — so this sheet is where the
 * keys are learned. It lists what `useEditorKeys` and `useReviewKeys` bind; a
 * key added there belongs here too.
 */
import { Keyboard } from 'lucide-react';
import { type JSX, useEffect, useState } from 'react';

import { Dialog, DialogContent, DialogDescription, DialogTitle } from '../components/ui/dialog.js';

interface Shortcut {
  readonly keys: readonly string[];
  readonly does: string;
}

interface Group {
  readonly title: string;
  readonly shortcuts: readonly Shortcut[];
}

const COMMAND = typeof navigator !== 'undefined' && /Mac/.test(navigator.platform) ? '⌘' : 'Ctrl';

export const SHORTCUTS: readonly Group[] = [
  {
    title: 'Playing',
    shortcuts: [
      { keys: ['Space'], does: 'Play or pause' },
      {
        keys: ['J', 'K', 'L'],
        does: 'Play backwards, stop, play forwards; press again to go faster',
      },
      { keys: ['←', '→'], does: 'One frame back or forward' },
      { keys: ['⇧ ←', '⇧ →'], does: 'Ten frames in the Editor, one second in review' },
      { keys: ['Home', 'End'], does: 'Go to the start or the end' },
    ],
  },
  {
    title: 'Editor',
    shortcuts: [
      { keys: ['I', 'O'], does: 'Mark the in and out of a range' },
      { keys: ['⌥ X'], does: 'Clear the marks' },
      { keys: ['Delete'], does: 'Remove the selection or the marked range' },
      { keys: [`${COMMAND} B`], does: 'Split at the playhead' },
      { keys: ['B', 'V'], does: 'Blade tool, select tool' },
      { keys: ['S'], does: 'Snapping on or off' },
      { keys: ['M'], does: 'Add a marker' },
      { keys: ['↑', '↓'], does: 'Previous or next edit point' },
      { keys: [`${COMMAND} =`, `${COMMAND} −`, '⇧ Z'], does: 'Zoom the timeline in, out, to fit' },
      { keys: [`${COMMAND} F`], does: 'Find and replace in the transcript' },
      { keys: [`${COMMAND} Z`, `${COMMAND} ⇧ Z`], does: 'Undo, redo' },
      { keys: [`${COMMAND} E`], does: 'Export the clip' },
      { keys: ['⌥ drag'], does: 'Drag one caption on its own on the preview' },
    ],
  },
  {
    title: 'Reviewing clips',
    shortcuts: [
      { keys: ['A'], does: 'Approve' },
      { keys: ['Enter'], does: 'Approve and open in the Editor' },
      { keys: ['H'], does: 'Keep for later' },
      { keys: ['X'], does: 'Reject' },
      { keys: ['U'], does: 'Clear the decision' },
      { keys: ['↑', '↓'], does: 'Previous or next clip' },
      { keys: ['I', 'O'], does: 'Move the start or the end to the playhead' },
      { keys: ['[', ']'], does: 'Hear around the start or the end' },
      { keys: [`${COMMAND} Z`], does: 'Undo the last decision' },
    ],
  },
  {
    title: 'Everywhere',
    shortcuts: [{ keys: [`${COMMAND} /`], does: 'Show these shortcuts' }],
  },
];

/** The sheet, and ⌘/ to open it from anywhere in the app. */
export function ShortcutSheet({
  open,
  onOpenChange,
}: {
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
}): JSX.Element {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="shortcut-sheet w-[min(760px,calc(100vw-32px))]">
        <div className="shortcut-head">
          <DialogTitle className="flex items-center gap-2">
            <Keyboard className="size-4" aria-hidden="true" />
            Keyboard shortcuts
          </DialogTitle>
          <DialogDescription>
            Every key works while nothing is being typed. Click a button with the mouse and the keys
            keep working.
          </DialogDescription>
        </div>
        <div className="shortcut-groups">
          {SHORTCUTS.map((group) => (
            <section key={group.title} aria-label={group.title}>
              <h3>{group.title}</h3>
              <dl>
                {group.shortcuts.map((shortcut) => (
                  <div key={shortcut.does}>
                    <dt>
                      {shortcut.keys.map((key) => (
                        <kbd key={key}>{key}</kbd>
                      ))}
                    </dt>
                    <dd>{shortcut.does}</dd>
                  </div>
                ))}
              </dl>
            </section>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}

/** ⌘/ (Ctrl+/ elsewhere) toggles the sheet wherever focus is. */
export function useShortcutSheet(): {
  readonly open: boolean;
  readonly setOpen: (open: boolean) => void;
} {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key === '/') {
        event.preventDefault();
        setOpen((current) => !current);
      }
    };
    const onAsk = () => setOpen(true);
    window.addEventListener('keydown', onKey);
    window.addEventListener(OPEN_SHORTCUTS_EVENT, onAsk);
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener(OPEN_SHORTCUTS_EVENT, onAsk);
    };
  }, []);
  return { open, setOpen };
}

/** Opens the sheet from a screen's own help menu or settings. */
export const OPEN_SHORTCUTS_EVENT = 'clipmill:shortcuts';

export function openShortcuts(): void {
  window.dispatchEvent(new Event(OPEN_SHORTCUTS_EVENT));
}
