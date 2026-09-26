/**
 * A clip's edit history, as the daemon logged it.
 *
 * Every command is stored with the command that undoes it, so the history is
 * already durable: undo can outlive the window it was made in, and going back
 * to any earlier point is the inverses of everything since, applied newest
 * first, as one edit.
 */
import type { EditCommandJson, EditHistoryEntry } from '../daemon/client.js';
import { batch } from './commands.js';
import { FRAME_SHAPES } from './layouts.js';

const LAYOUTS: Record<string, string> = {
  fit: 'Whole frame',
  speaker_fill: 'Follow speaker',
  two_up: 'Two speakers',
  picture_in_picture: 'Picture in picture',
};

const LOOKS: Record<string, string> = {
  clean: 'Clean',
  minimal: 'Minimal',
  boxed: 'Boxed',
};

/** What an edit did, in the words the editor's controls use. */
export function describeCommand(command: EditCommandJson): string {
  switch (command.op) {
    case 'batch': {
      const inner = (command.commands as readonly EditCommandJson[] | undefined) ?? [];
      const first = inner[0];
      if (!first) return 'Edit';
      // A new shape carries its reshaped crops along; the shape is the edit.
      if (first.op === 'set_frame_shape') return describeCommand(first);
      const labels = new Set(inner.map(describeCommand));
      return labels.size === 1 ? describeCommand(first) : `${describeCommand(first)} and more`;
    }
    case 'set_caption_style': {
      const look = /\.(clean|minimal|boxed)\./.exec(String(command.style_ref))?.[1];
      return `Caption look: ${look ? LOOKS[look] : 'changed'}`;
    }
    case 'set_caption_options':
      return 'Caption style';
    case 'trim':
      return 'Trim';
    case 'extend_segment':
      return 'Extend the clip';
    case 'ripple_delete':
      return 'Cut';
    case 'restore_arrangement':
      return 'Restore a cut';
    case 'set_layout':
      return `Framing: ${LAYOUTS[String(command.state)] ?? 'changed'}`;
    case 'swap_portraits':
      return 'Swap speakers';
    case 'set_layout_style':
      return 'Layout style';
    case 'split_segment':
      return 'Split section';
    case 'set_crop_keyframe':
    case 'remove_crop_keyframe':
    case 'replace_crop_path':
    case 'set_secondary_crop_keyframe':
    case 'remove_secondary_crop_keyframe':
    case 'replace_secondary_crop_path':
      return 'Reframe';
    case 'edit_caption_text':
    case 'set_word_text':
      return `Correct “${String(command.text ?? '')}”`;
    case 'remove_caption_word':
      return 'Hide a word in captions';
    case 'set_cue_region':
    case 'set_cue_position':
      return 'Move a caption';
    case 'set_cue_timing':
      return 'Caption timing';
    case 'set_cue_lines':
      return 'Line break';
    case 'split_cue':
      return 'Split a caption';
    case 'merge_cues':
      return 'Merge captions';
    case 'set_gain':
    case 'remove_gain_point':
      return 'Volume';
    case 'set_transition':
      return Number(command.duration_ticks) > 0 ? 'Soft cuts on' : 'Soft cuts off';
    case 'set_word_emphasis':
      return command.emphasis ? 'Key word' : 'Plain word';
    case 'regroup_on_screen':
      return `Words on screen: ${String(command.max_words)}`;
    case 'drop_non_speech_words':
      return 'Remove non-speech marks';
    case 'replace_cues':
      return 'Replace captions';
    case 'refresh_captions':
      return 'Refresh captions';
    case 'set_title':
      return command.title ? `Rename: ${String(command.title)}` : 'Clear the title';
    case 'set_frame_shape':
      return `Shape: ${FRAME_SHAPES.find((item) => item.shape === command.shape)?.ratio ?? 'changed'}`;
    default:
      return 'Edit';
  }
}

/** One step of the history, ready to show. */
export interface HistoryStep {
  readonly revision: number;
  readonly label: string;
  readonly appliedUnixMillis: number;
  readonly inverse: EditCommandJson;
}

export function historySteps(entries: readonly EditHistoryEntry[]): readonly HistoryStep[] {
  const steps: HistoryStep[] = [];
  for (const entry of entries) {
    try {
      steps.push({
        revision: entry.revision,
        label: describeCommand(JSON.parse(entry.commandJson) as EditCommandJson),
        appliedUnixMillis: entry.appliedUnixMillis,
        inverse: JSON.parse(entry.inverseJson) as EditCommandJson,
      });
    } catch {
      // A step that does not parse cannot be undone from here; the rest can.
    }
  }
  return steps;
}

/**
 * The one edit that takes the document back to how it was at `revision`:
 * every later step's inverse, newest first. Null when nothing came after.
 */
export function revertTo(steps: readonly HistoryStep[], revision: number): EditCommandJson | null {
  const later = steps.filter((step) => step.revision > revision);
  if (later.length === 0) return null;
  const inverses = later.toReversed().map((step) => step.inverse);
  return inverses.length === 1 ? inverses[0]! : batch(inverses);
}
