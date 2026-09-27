/**
 * The emoji the editor offers are the renderer's, word for word: an emoji
 * offered here that the render does not know would fail the export, and a
 * word that calls for one here but not there would put emoji on new clips
 * differently from the button. Both are read from their sources.
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import type { PreviewPlan } from '../src/daemon/client.js';
import {
  EMOJI,
  emojiBox,
  emojiCharacter,
  emojiForWord,
  emojiOverlay,
  keywordEmoji,
} from '../src/editor/emoji.js';
import { savedOverlay, withContent } from '../src/editor/overlays.js';
import { plan } from './support/clips.js';
import { mapping } from './support/plan.js';

const SECOND = 90_000;

function rendererCatalogue() {
  const source = readFileSync(
    resolve(process.cwd(), '../../crates/clipmill-captions/src/emoji.rs'),
    'utf8',
  );
  const start = source.indexOf('pub const EMOJI');
  const table = source.slice(start, source.indexOf('];\n', start));
  return [...table.matchAll(/code: "([^"]+)",\s*label: "([^"]+)",\s*words: &\[([^\]]*)\]/g)].map(
    (match) => ({
      code: match[1]!,
      label: match[2]!,
      words: [...match[3]!.matchAll(/"([^"]+)"/g)].map((word) => word[1]!),
    }),
  );
}

function pinned(): string[] {
  const bom = readFileSync(resolve(process.cwd(), '../../bom.toml'), 'utf8');
  const section = bom.slice(bom.indexOf('[emoji]'));
  return /^ids = "([^"]+)"/m.exec(section)![1]!.split(',');
}

/** A 30-second vertical clip at 30 frames a second. */
function clip(overlays: PreviewPlan['overlays'] = []): PreviewPlan {
  const rate = { frameCount: 900, rateNum: 30, rateDen: 1 };
  return { ...plan(), ...mapping(rate, 0), ...rate, width: 1080, height: 1920, overlays };
}

describe('the emoji a clip can show', () => {
  it('are the renderer’s, and the pinned pictures', () => {
    const renderer = rendererCatalogue();
    expect(renderer).toHaveLength(40);
    expect(EMOJI).toEqual(renderer);
    expect(EMOJI.map((entry) => entry.code)).toEqual(pinned());
  });

  it('are called for by words a speaker leans on, never everyday ones', () => {
    expect(emojiForWord('Money,')?.label).toBe('Money');
    expect(emojiForWord('“idea”')?.code).toBe('1f4a1');
    expect(emojiForWord('LAUNCHED!')?.code).toBe('1f680');
    for (const everyday of ['the', 'think', 'time', 'see', 'say', 'yes', 'best', 'here', '...']) {
      expect(emojiForWord(everyday), everyday).toBeUndefined();
    }
    const words = EMOJI.flatMap((entry) => entry.words);
    expect(new Set(words).size).toBe(words.length);
  });

  it('draw as the character where the picture cannot load, in colour', () => {
    expect(emojiCharacter('1f525')).toBe('🔥');
    // A symbol with a plain-text form asks for its colour form.
    expect(emojiCharacter('2764')).toBe('❤️');
  });

  it('are laid where the render lays them, an even side of the short edge', () => {
    expect(emojiBox({ x: 500, y: 640, size: 180 }, { width: 1080, height: 1920 })).toEqual({
      side: 194,
      cx: 540,
      cy: 1228,
    });
    // Across a landscape frame the short side is its height.
    expect(emojiBox({ x: 250, y: 500, size: 175 }, { width: 1920, height: 1080 })).toEqual({
      side: 188,
      cx: 480,
      cy: 540,
    });
  });

  it('go in at the playhead for a moment, and never past the clip’s end', () => {
    expect(emojiOverlay('ovl_1', '1f525', 3 * SECOND, clip())).toEqual({
      overlay_id: 'ovl_1',
      start_ticks: 3 * SECOND,
      end_ticks: 3 * SECOND + 108_000,
      content: { kind: 'emoji', emoji: '1f525', x: 500, y: 640, size: 180 },
    });
    const late = emojiOverlay('ovl_1', '1f525', 29.5 * SECOND, clip());
    expect(late.end_ticks).toBe(30 * SECOND);
  });

  it('go on key words at least three seconds apart, around any already there', () => {
    const shown = clip([
      {
        overlayId: 'ovl_1',
        kind: 'emoji',
        emoji: '1f4af',
        startTicks: 20 * SECOND,
        endTicks: 21 * SECOND,
        firstFrame: 600,
        endFrame: 630,
        text: '',
        role: 'label',
        x: 500,
        y: 640,
        size: 180,
        colour: '',
      },
    ]);
    const said = (text: string, seconds: number) => ({ text, startTicks: seconds * SECOND });
    const made = keywordEmoji(
      [
        said('Money', 1),
        said('ideas', 2),
        said('the', 5),
        said('launch.', 6),
        said('fire', 18),
        said('crazy', 25),
      ],
      shown,
    );
    expect(made.map((overlay) => [overlay.overlay_id, overlay.content.emoji])).toEqual([
      ['ovl_2', '1f4b0'],
      ['ovl_3', '1f680'],
      ['ovl_4', '1f92f'],
    ]);
    expect(made[0]!.start_ticks).toBe(SECOND);
  });

  it('keep what they are through the document and a change', () => {
    const shown = {
      overlayId: 'ovl_2',
      kind: 'emoji' as const,
      emoji: '1f525',
      startTicks: 0,
      endTicks: 108_000,
      firstFrame: 0,
      endFrame: 36,
      text: '',
      role: 'label' as const,
      x: 500,
      y: 640,
      size: 180,
      colour: '',
      plate: null,
    };
    const saved = savedOverlay(shown);
    expect(saved.content).toEqual({ kind: 'emoji', emoji: '1f525', x: 500, y: 640, size: 180 });
    expect(withContent(saved, { emoji: '1f4a1', size: 240 }).content).toEqual({
      kind: 'emoji',
      emoji: '1f4a1',
      x: 500,
      y: 640,
      size: 240,
    });
  });
});
