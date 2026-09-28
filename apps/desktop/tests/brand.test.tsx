import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Asset } from '../src/daemon/client.js';
import {
  DEFAULT_BAR,
  hasKit,
  kitRequest,
  recallKit,
  rememberKit,
  setBrand,
  withAsset,
  withBrand,
} from '../src/editor/brand.js';
import { BrandTab } from '../src/editor/BrandTab.js';
import { plan } from './support/clips.js';

const logoHash = `sha256:${'2'.repeat(64)}`;
const picture: Asset = {
  hash: logoHash,
  kind: 'image',
  name: 'mark.png',
  mediaType: 'image/png',
  bytes: 1_200,
  width: 400,
  height: 200,
  durationTicks: 0,
  license: 'own_content',
  addedUnixMillis: 1,
};

beforeEach(() => localStorage.clear());
afterEach(cleanup);

describe('a clip’s brand', () => {
  it('changes one part at a time and goes away when nothing is left', () => {
    const barred = withBrand(null, { progress: DEFAULT_BAR });
    expect(barred).toEqual({ progress: DEFAULT_BAR });
    const logo = { asset: logoHash, corner: 'top_left' as const, size: 160, opacity: 90 };
    expect(withBrand(barred, { logo })).toEqual({ progress: DEFAULT_BAR, logo });
    expect(withBrand(barred, { progress: null })).toBeNull();
    expect(setBrand(null)).toEqual({ op: 'set_brand' });
    expect(setBrand(barred, [{ hash: logoHash, license: 'own_content' }])).toEqual({
      op: 'set_brand',
      brand: { progress: DEFAULT_BAR },
      assets: [{ hash: logoHash, license: 'own_content' }],
    });
    expect(withAsset([{ hash: logoHash, license: 'licensed' }], picture)).toEqual([
      { hash: logoHash, license: 'licensed' },
    ]);
  });

  it('keeps a kit that new clips start with, shape and all', () => {
    expect(kitRequest()).toEqual({});
    rememberKit({ brand: { progress: DEFAULT_BAR }, shape: 'square' });
    expect(recallKit().shape).toBe('square');
    expect(kitRequest()).toEqual({
      brandJson: JSON.stringify({ progress: DEFAULT_BAR }),
      shape: 'square',
    });
    expect(hasKit({ brand: { progress: DEFAULT_BAR } }, recallKit())).toBe(true);
    rememberKit({});
    expect(localStorage.getItem('clipmill.brand.v1')).toBeNull();
  });
});

describe('the Brand tab', () => {
  const assets = {
    list: vi.fn().mockResolvedValue([picture]),
    bring: vi.fn().mockResolvedValue(picture),
    url: (hash: string) => `asset://${hash}`,
  };
  const document = {
    version: 'ir/1',
    timebase: { num: 1, den: 90_000 },
    video: { segments: [] },
    captions: { style_ref: 'clean' },
    audio: { target_lufs: -14, true_peak_dbtp: -1 },
  } as never;

  it('turns a progress bar on, and puts a picture in the corner with its licence', async () => {
    const onApply = vi.fn();
    render(
      <BrandTab plan={plan()} document={document} busy={false} onApply={onApply} assets={assets} />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'On' }));
    expect(onApply).toHaveBeenLastCalledWith({ op: 'set_brand', brand: { progress: DEFAULT_BAR } });
    fireEvent.click(await screen.findByRole('button', { name: 'Use mark.png' }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_brand',
      brand: { logo: { asset: logoHash, corner: 'top_left', size: 160, opacity: 90 } },
      assets: [{ hash: logoHash, license: 'own_content' }],
    });
    fireEvent.click(screen.getByRole('button', { name: /choose a picture/i }));
    await waitFor(() => expect(assets.bring).toHaveBeenCalledWith('image', 'own_content'));
  });

  it('saves the clip’s brand and shape as the kit for new clips', () => {
    const branded = { ...(document as object), brand: { progress: DEFAULT_BAR } } as never;
    render(
      <BrandTab
        plan={{ ...plan(), width: 1080, height: 1080 }}
        document={branded}
        busy={false}
        onApply={vi.fn()}
        assets={null}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: /start new clips like this/i }));
    expect(recallKit()).toEqual({ brand: { progress: DEFAULT_BAR }, shape: 'square' });
    expect(screen.getByText(/new clips start with a progress bar and the 1:1 shape/i)).toBeTruthy();
  });
});
