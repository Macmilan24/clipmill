/**
 * Tell a person who looked away that a long run is over: the window asks for
 * attention (the Dock icon bounces once) and its title says what finished
 * until they come back to it. Each run is told once, whichever screen
 * notices first.
 */
import { requestAttention } from '../daemon/client.js';

const told = new Set<string>();
/** The window's own title while an announcement stands in for it. */
let ownTitle: string | null = null;

export function announceFinished(runId: string, title: string): void {
  if (told.has(runId)) return;
  told.add(runId);
  if (typeof document === 'undefined' || (!document.hidden && document.hasFocus())) return;
  if (ownTitle === null) {
    ownTitle = document.title;
    const restore = () => {
      if (ownTitle !== null) document.title = ownTitle;
      ownTitle = null;
      window.removeEventListener('focus', restore);
    };
    window.addEventListener('focus', restore);
  }
  document.title = title;
  void requestAttention().catch(() => undefined);
}
