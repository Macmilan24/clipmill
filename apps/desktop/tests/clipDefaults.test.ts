import { beforeEach, describe, expect, it, vi } from 'vitest';

const sent: [string, Record<string, unknown>][] = [];
vi.mock('../src/daemon/client.js', () => ({
  setClipDefaults: (projectId: string, defaults: Record<string, unknown>) => {
    sent.push([projectId, defaults]);
    return Promise.resolve();
  },
}));

const { shareKit, shareLook } = await import('../src/results/clipDefaults.js');
const { rememberKit } = await import('../src/editor/brand.js');
const { rememberHighlight, rememberLook, rememberOptions, setStartingLook } =
  await import('../src/results/captionLook.js');

describe('the look clips start with, as the engine keeps it', () => {
  beforeEach(() => {
    sent.length = 0;
    localStorage.clear();
  });

  it('sends the kit with the look new projects start with', () => {
    setStartingLook('clipmill.captions.boxed.v1');
    rememberKit({
      brand: { progress: { colour: '#FFD65C', edge: 'bottom', thickness: 10 } },
      shape: 'square',
    });
    shareKit();
    expect(sent).toEqual([
      [
        '',
        {
          styleRef: 'clipmill.captions.boxed.v1',
          brandJson: '{"progress":{"colour":"#FFD65C","edge":"bottom","thickness":10}}',
          shape: 'square',
        },
      ],
    ]);
  });

  it('sends a plain kit as nothing but the starting look', () => {
    shareKit();
    expect(sent).toEqual([['', { styleRef: 'clipmill.captions.clean.v1' }]]);
  });

  it('sends one project’s caption look, options and highlight', () => {
    rememberLook('prj_a', 'clipmill.captions.minimal.v1');
    rememberHighlight('prj_a', false);
    rememberOptions('prj_a', { font_size: 64 });
    shareLook('prj_a');
    expect(sent).toEqual([
      [
        'prj_a',
        {
          styleRef: 'clipmill.captions.minimal.v1',
          highlightSpokenWord: false,
          captionOptionsJson: '{"font_size":64}',
        },
      ],
    ]);
  });
});
