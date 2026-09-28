/**
 * Text laid over the clip: a hook title that opens it, labels anywhere else.
 *
 * An overlay is timed in program time, like a caption, and positioned by its
 * centre as a share of the frame, so it stays where it was put at every size
 * and in every shape. A text is set in the clip's caption font and drawn by
 * the same renderer as the captions; lines break only where its text says, so
 * a suggested title is broken here to fit the frame it will sit in. An emoji
 * is the other kind (`emoji.ts`).
 */
import type { EditIr } from '@clipmill/contracts';

import type { EditCommandJson, PreviewOverlay, PreviewPlan } from '../daemon/client.js';

export type Overlay = NonNullable<EditIr['overlays']>[number];
export type OverlayContent = Overlay['content'];
export type TextContent = Extract<OverlayContent, { kind: 'text' }>;
export type EmojiContent = Extract<OverlayContent, { kind: 'emoji' }>;
export type TextOverlay = Omit<Overlay, 'content'> & { content: TextContent };
export type EmojiOverlay = Omit<Overlay, 'content'> & { content: EmojiContent };

const SECOND = 90_000;
/** How long a new text stays up. */
const TEXT_SPAN = 3 * SECOND;
/** The design height every size is stated at. */
const DESIGN_HEIGHT = 1920;

export const TEXT_SIZES = { min: 24, max: 240, hook: 88, label: 56 } as const;

/** Looks a text can take in one click: words on a plate, or outlined. */
export const TEXT_LOOKS: readonly {
  readonly name: string;
  readonly colour: string;
  readonly plate?: string;
}[] = [
  { name: 'White plate', colour: '#111111', plate: '#FFFFFF' },
  { name: 'Black plate', colour: '#FFFFFF', plate: '#111111' },
  { name: 'Red plate', colour: '#FFFFFF', plate: '#E0245E' },
  { name: 'Yellow plate', colour: '#111111', plate: '#FFD65C' },
  { name: 'Outlined', colour: '#FFFFFF' },
];

/** Where a text sits, in one click: its centre, per mille of the frame. */
export const TEXT_PLACES = [
  { name: 'Top', y: 140 },
  { name: 'Middle', y: 500 },
  { name: 'Lower', y: 640 },
] as const;

export function addOverlay(overlay: Overlay, at?: number): EditCommandJson {
  return { op: 'add_overlay', overlay, ...(at === undefined ? {} : { at }) };
}

export function removeOverlay(overlayId: string): EditCommandJson {
  return { op: 'remove_overlay', overlay_id: overlayId };
}

export function setOverlay(overlay: Overlay): EditCommandJson {
  return { op: 'set_overlay', overlay };
}

/** An id no overlay in the document has. */
export function freshOverlayId(existing: readonly { readonly overlay_id: string }[]): string {
  const taken = new Set(existing.map((overlay) => overlay.overlay_id));
  let number = existing.length + 1;
  while (taken.has(`ovl_${number}`)) number += 1;
  return `ovl_${number}`;
}

/** The overlays showing at a frame, bottom first. */
export function overlaysAt(plan: PreviewPlan, frame: number): readonly PreviewOverlay[] {
  return (plan.overlays ?? []).filter(
    (overlay) => frame >= overlay.firstFrame && frame < overlay.endFrame,
  );
}

/**
 * How many characters of a text this size fit across a frame of this shape:
 * most of its width at the design height, at an average glyph's width.
 */
export function charactersPerLine(size: number, frame: { width: number; height: number }): number {
  const designWidth = (frame.width * DESIGN_HEIGHT) / Math.max(1, frame.height);
  return Math.max(8, Math.floor((0.86 * designWidth) / (0.56 * size)));
}

/**
 * A text broken into lines that fit, at word boundaries, keeping each line
 * as long as it can be. A word longer than a line keeps a line to itself.
 */
export function breakLines(text: string, perLine: number): string {
  const lines: string[] = [];
  for (const paragraph of text.split('\n')) {
    let line = '';
    for (const word of paragraph.split(/\s+/).filter(Boolean)) {
      if (line && line.length + 1 + word.length > perLine) {
        lines.push(line);
        line = word;
      } else {
        line = line ? `${line} ${word}` : word;
      }
    }
    if (line) lines.push(line);
  }
  return lines.join('\n');
}

/** Whether any line of a text is longer than the frame holds at its size. */
export function overflows(text: string, size: number, frame: { width: number; height: number }) {
  const limit = charactersPerLine(size, frame);
  return text.split('\n').some((line) => line.length > limit);
}

/** A title the clip opens on, from what it is called or the first thing said. */
export function suggestedHook(title: string | null | undefined, firstSentence: string | null) {
  const source = (title ?? '').trim() || (firstSentence ?? '').trim();
  if (!source) return 'Your hook here';
  // A sentence is cut at a word, well short of a paragraph.
  const words = source
    .replace(/[{}\\]/g, '')
    .split(/\s+/)
    .filter(Boolean);
  let text = '';
  for (const word of words) {
    if ((text + ' ' + word).trim().length > 72) break;
    text = `${text} ${word}`.trim();
  }
  return text || words[0]!.slice(0, 72);
}

/** The most lines a hook is set in before it is set smaller. */
const HOOK_LINES = 3;

/**
 * A title broken to fit the frame in at most three lines, at the largest of
 * a few sizes that allows it; past that, the smallest, and the person
 * shortens it.
 */
export function fitHook(
  text: string,
  frame: { width: number; height: number },
): { readonly text: string; readonly size: number } {
  const sizes = [TEXT_SIZES.hook, 76, 66, 58];
  for (const size of sizes) {
    const broken = breakLines(text, charactersPerLine(size, frame));
    if (broken.split('\n').length <= HOOK_LINES) return { text: broken, size };
  }
  const size = sizes.at(-1)!;
  return { text: breakLines(text, charactersPerLine(size, frame)), size };
}

/** A hook title over the clip's opening seconds, near the top, on a plate. */
export function hookOverlay(
  id: string,
  text: string,
  plan: Pick<PreviewPlan, 'width' | 'height' | 'frameCount' | 'rateNum' | 'rateDen'>,
): TextOverlay {
  const fitted = fitHook(text, plan);
  const look = TEXT_LOOKS[0]!;
  return {
    overlay_id: id,
    start_ticks: 0,
    end_ticks: Math.min(TEXT_SPAN, programTicks(plan)),
    content: {
      kind: 'text',
      text: fitted.text,
      role: 'hook',
      x: 500,
      y: plan.height > plan.width ? 140 : 120,
      size: fitted.size,
      colour: look.colour,
      ...(look.plate ? { plate: look.plate } : {}),
    },
  };
}

/** A label from the playhead, above where captions sit, outlined. */
export function labelOverlay(
  id: string,
  startTicks: number,
  plan: Pick<PreviewPlan, 'width' | 'height' | 'frameCount' | 'rateNum' | 'rateDen'>,
): TextOverlay {
  const end = programTicks(plan);
  const start = Math.max(0, Math.min(startTicks, end - SECOND / 10));
  return {
    overlay_id: id,
    start_ticks: start,
    end_ticks: Math.min(start + TEXT_SPAN, end),
    content: {
      kind: 'text',
      text: 'Your text',
      x: 500,
      y: 640,
      size: TEXT_SIZES.label,
      colour: '#FFFFFF',
    },
  };
}

/** The document's form of an overlay the plan shows. */
export function savedOverlay(overlay: PreviewOverlay): Overlay {
  const timing = {
    overlay_id: overlay.overlayId,
    start_ticks: overlay.startTicks,
    end_ticks: overlay.endTicks,
  };
  if (overlay.kind === 'emoji') {
    return {
      ...timing,
      content: {
        kind: 'emoji',
        emoji: overlay.emoji ?? '',
        x: overlay.x,
        y: overlay.y,
        size: overlay.size,
      },
    };
  }
  return {
    ...timing,
    content: {
      kind: 'text',
      text: overlay.text,
      ...(overlay.role === 'hook' ? { role: 'hook' as const } : {}),
      x: overlay.x,
      y: overlay.y,
      size: overlay.size,
      colour: overlay.colour,
      ...(overlay.plate ? { plate: overlay.plate } : {}),
    },
  };
}

type Change<Content> = { readonly [Key in keyof Content]?: Content[Key] | undefined };

/**
 * The same overlay with some of its content changed; `plate: undefined` takes
 * a text's plate away.
 */
export function withContent<Kind extends Overlay | TextOverlay | EmojiOverlay>(
  overlay: Kind,
  change: Change<TextContent> | Change<EmojiContent>,
): Kind {
  const merged: Record<string, unknown> = { ...overlay.content, ...change };
  // No plate is the absence of one, not an empty colour.
  if (!merged.plate) delete merged.plate;
  if (merged.role !== 'hook') delete merged.role;
  return { ...overlay, content: merged as OverlayContent } as Kind;
}

function programTicks(plan: Pick<PreviewPlan, 'frameCount' | 'rateNum' | 'rateDen'>): number {
  return Math.floor((plan.frameCount * plan.rateDen * SECOND) / Math.max(1, plan.rateNum));
}
