import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { Asset } from '../src/daemon/client.js';
import { MusicSection } from '../src/editor/MusicSection.js';
import { musicLevelAt, musicSecondsAt, setCleanup, setMusic } from '../src/editor/music.js';

afterEach(cleanup);

const bedHash = `sha256:${'3'.repeat(64)}`;
const sound: Asset = {
  hash: bedHash,
  kind: 'audio',
  name: 'Warm bed.mp3',
  mediaType: 'audio/mpeg',
  bytes: 800_000,
  width: 0,
  height: 0,
  durationTicks: 30 * 90_000,
  license: 'royalty_free',
  addedUnixMillis: 1,
};
const document = {
  version: 'ir/1',
  timebase: { num: 1, den: 90_000 },
  video: { segments: [] },
  captions: { style_ref: 'clean' },
  audio: { target_lufs: -14, true_peak_dbtp: -1 },
} as never;

describe('music under the voice', () => {
  it('plays at the render’s level for each frame, looping as the render does', () => {
    const levels = [
      { frame: 0, gainDb: -60 },
      { frame: 15, gainDb: -18 },
      { frame: 30, gainDb: -18 },
      { frame: 36, gainDb: -30 },
    ];
    expect(musicLevelAt(levels, 0)).toBe(-60);
    expect(musicLevelAt(levels, 20)).toBe(-18);
    expect(musicLevelAt(levels, 33)).toBe(-24);
    expect(musicLevelAt(levels, 90)).toBe(-30);
    const rate = { rateNum: 30, rateDen: 1 };
    expect(musicSecondsAt(rate, 90_000, 30, 10)).toBe(2);
    expect(musicSecondsAt(rate, 0, 330, 10)).toBe(1);
  });

  it('writes one step each for music and for cleanup', () => {
    expect(setMusic(null)).toEqual({ op: 'set_music' });
    expect(setCleanup('light')).toEqual({ op: 'set_cleanup', cleanup: 'light' });
    expect(setCleanup(null)).toEqual({ op: 'set_cleanup' });
  });

  it('puts a sound under the clip with its licence, and cleans the voice', async () => {
    const onApply = vi.fn();
    const assets = {
      list: vi.fn().mockResolvedValue([sound]),
      bring: vi.fn().mockResolvedValue(null),
      url: (hash: string) => `asset://${hash}`,
    };
    render(<MusicSection document={document} busy={false} onApply={onApply} assets={assets} />);
    fireEvent.click(await screen.findByRole('button', { name: /warm bed/i }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_music',
      music: { asset: bedHash, level_db: -18, duck_db: -12 },
      assets: [{ hash: bedHash, license: 'royalty_free' }],
    });
    fireEvent.click(screen.getByRole('button', { name: 'Strong' }));
    expect(onApply).toHaveBeenLastCalledWith({ op: 'set_cleanup', cleanup: 'strong' });
  });
});
