/**
 * The engine's copy of the look clips start with. The Studio keeps its
 * choices on this machine; it also sends them to the engine whenever they
 * change, so a clip an agent or the command line makes starts with the same
 * captions, brand and frame as one made here.
 */
import { setClipDefaults } from '../daemon/client.js';
import { recallKit } from '../editor/brand.js';
import { highlightFor, lookFor, optionsFor, startingLook } from './captionLook.js';

/** The installation's kit and the look new projects start with. */
export function shareKit(): void {
  const kit = recallKit();
  setClipDefaults('', {
    styleRef: startingLook(),
    ...(kit.brand ? { brandJson: JSON.stringify(kit.brand) } : {}),
    ...(kit.shape && kit.shape !== 'vertical' ? { shape: kit.shape } : {}),
  }).catch(() => {
    // The engine keeps its earlier copy; the next change sends it again.
  });
}

/** The caption look one project's clips start with. */
export function shareLook(projectId: string): void {
  const options = optionsFor(projectId);
  setClipDefaults(projectId, {
    styleRef: lookFor(projectId) ?? '',
    highlightSpokenWord: highlightFor(projectId),
    ...(options ? { captionOptionsJson: options } : {}),
  }).catch(() => {
    // As above: kept until the next change.
  });
}
