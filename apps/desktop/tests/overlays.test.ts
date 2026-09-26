import { describe, expect, it } from 'vitest';

import {
  breakLines,
  charactersPerLine,
  freshOverlayId,
  hookOverlay,
  labelOverlay,
  overflows,
  overlaysAt,
  suggestedHook,
  withContent,
} from '../src/editor/overlays.js';
import { plan } from './support/clips.js';

const vertical = { width: 1080, height: 1920 };
const landscape = { width: 1920, height: 1080 };

describe('text over the clip', () => {
  it('fits more words on a line across a wider frame', () => {
    expect(charactersPerLine(88, vertical)).toBe(18);
    expect(charactersPerLine(88, landscape)).toBeGreaterThan(50);
    expect(breakLines('Why the second question always wins the room', 18)).toBe(
      'Why the second\nquestion always\nwins the room',
    );
    // A person's own break is kept; a word longer than a line keeps one.
    expect(breakLines('Short\nIncomprehensibilities', 10)).toBe('Short\nIncomprehensibilities');
    expect(overflows('A line far too long for the frame', 88, vertical)).toBe(true);
    expect(overflows('Fits\nfine', 88, vertical)).toBe(false);
  });

  it('suggests a hook from the clip’s name, or else what is said first', () => {
    expect(suggestedHook('Charging less', 'Anything else')).toBe('Charging less');
    expect(suggestedHook('  ', 'Attention is the real budget. And more.')).toBe(
      'Attention is the real budget. And more.',
    );
    const long = suggestedHook(
      'A title that goes on and on well past what anybody would read in a hook over a clip',
      null,
    );
    expect(long.length).toBeLessThanOrEqual(72);
    expect(long.endsWith(' ')).toBe(false);
    expect(suggestedHook('{\\b1}Marked up', null)).toBe('b1Marked up');
    expect(suggestedHook(null, null)).toBe('Your hook here');
  });

  it('opens the clip with a hook on a plate, and labels from the playhead', () => {
    const base = { ...plan(), width: 1080, height: 1920, frameCount: 60 };
    const hook = hookOverlay('ovl_1', 'Why the second question always wins', base);
    expect(hook).toMatchObject({
      overlay_id: 'ovl_1',
      start_ticks: 0,
      end_ticks: 180_000,
      content: { kind: 'text', role: 'hook', x: 500, y: 140, size: 88, plate: '#FFFFFF' },
    });
    expect(hook.content.text).toBe('Why the second\nquestion always\nwins');
    // Shorter than three seconds, the hook covers the clip.
    expect(hookOverlay('ovl_1', 'Hi', { ...base, frameCount: 30 }).end_ticks).toBe(90_000);
    const label = labelOverlay('ovl_2', 170_000, base);
    expect(label.start_ticks).toBe(170_000);
    expect(label.end_ticks).toBe(180_000);
    expect(label.content).not.toHaveProperty('role');
    expect(label.content).not.toHaveProperty('plate');
  });

  it('takes a plate away rather than leaving an empty colour', () => {
    const hook = hookOverlay('ovl_1', 'Hook', { ...plan(), width: 1080, height: 1920 });
    const outlined = withContent(hook, { plate: undefined, colour: '#ffffff' });
    expect(outlined.content).not.toHaveProperty('plate');
    expect(outlined.content.colour).toBe('#ffffff');
    expect(outlined.content.role).toBe('hook');
    expect(withContent(outlined, { role: 'label' }).content).not.toHaveProperty('role');
  });

  it('names a new overlay apart from the others and finds what shows at a frame', () => {
    expect(freshOverlayId([{ overlay_id: 'ovl_1' }, { overlay_id: 'ovl_3' }])).toBe('ovl_4');
    expect(freshOverlayId([{ overlay_id: 'ovl_2' }])).toBe('ovl_3');
    expect(freshOverlayId([])).toBe('ovl_1');
    const shown = {
      ...plan(),
      overlays: [
        {
          overlayId: 'a',
          startTicks: 0,
          endTicks: 45_000,
          firstFrame: 0,
          endFrame: 15,
          text: 'A',
          role: 'hook' as const,
          x: 500,
          y: 140,
          size: 88,
          colour: '#111111',
          plate: '#FFFFFF',
        },
      ],
    };
    expect(overlaysAt(shown, 14).map((item) => item.overlayId)).toEqual(['a']);
    expect(overlaysAt(shown, 15)).toEqual([]);
  });
});

describe('a long title as a hook', () => {
  it('is set smaller rather than run past three lines', async () => {
    const { fitHook } = await import('../src/editor/overlays.js');
    const fitted = fitHook('Pick the smallest version of the thing that scares you', vertical);
    expect(fitted.size).toBe(76);
    expect(fitted.text.split('\n')).toHaveLength(3);
    expect(fitHook('Charging less', vertical)).toEqual({ text: 'Charging less', size: 88 });
  });
});
