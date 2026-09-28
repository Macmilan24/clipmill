/**
 * A clip's brand — a progress bar that fills as it plays, a logo in a corner
 * — and the kit that starts every new clip with one.
 *
 * The brand lives in the document and is drawn by the render over every
 * frame. The kit is this installation's: what the person saved from a clip
 * they liked, sent with each new clip so it starts branded, together with
 * the shape clips are framed for.
 */
import type { EditIr } from '@clipmill/contracts';

import type { Asset, AssetLicense, EditCommandJson } from '../daemon/client.js';
import type { FrameShape } from './layouts.js';

export type Brand = NonNullable<EditIr['brand']>;
export type ProgressBar = NonNullable<Brand['progress']>;
export type Logo = NonNullable<Brand['logo']>;
type DocumentAsset = NonNullable<EditIr['assets']>[number];

export const BAR_THICKNESSES = { min: 4, max: 40, default: 10 } as const;
export const LOGO_SIZES = { min: 60, max: 300, default: 160 } as const;
export const LOGO_OPACITIES = { min: 20, max: 100, default: 90 } as const;

export const DEFAULT_BAR: ProgressBar = {
  colour: '#FFD65C',
  edge: 'bottom',
  thickness: BAR_THICKNESSES.default,
};

/**
 * The brand made so, as one step: `null` takes it away. A logo brings its
 * asset into the clip's list with the licence it was brought in with.
 */
export function setBrand(brand: Brand | null, assets?: readonly DocumentAsset[]): EditCommandJson {
  return {
    op: 'set_brand',
    ...(brand && (brand.progress || brand.logo) ? { brand } : {}),
    ...(assets ? { assets: [...assets] } : {}),
  };
}

/** The same brand with part of it changed; a part set to null is taken away. */
export function withBrand(
  brand: Brand | null | undefined,
  change: { readonly progress?: ProgressBar | null; readonly logo?: Logo | null },
): Brand | null {
  const next: { progress?: ProgressBar; logo?: Logo } = { ...brand };
  if (change.progress === null) delete next.progress;
  else if (change.progress) next.progress = change.progress;
  if (change.logo === null) delete next.logo;
  else if (change.logo) next.logo = change.logo;
  return next.progress || next.logo ? next : null;
}

/** A clip's asset list with this picture in it, licence and all. */
export function withAsset(
  assets: readonly DocumentAsset[] | undefined,
  asset: Pick<Asset, 'hash' | 'license'>,
): DocumentAsset[] {
  const list = [...(assets ?? [])];
  if (!list.some((item) => item.hash === asset.hash)) {
    list.push({ hash: asset.hash, license: asset.license });
  }
  return list;
}

/** What new clips start with. */
export interface BrandKit {
  readonly brand?: Brand;
  readonly shape?: FrameShape;
  /** The licence the logo was brought in with, for a clip that lists it. */
  readonly logoLicense?: AssetLicense;
}

const KIT_KEY = 'clipmill.brand.v1';

export function recallKit(): BrandKit {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(KIT_KEY) ?? 'null');
    return parsed && typeof parsed === 'object' ? (parsed as BrandKit) : {};
  } catch {
    return {};
  }
}

export function rememberKit(kit: BrandKit): void {
  try {
    if (!kit.brand && (!kit.shape || kit.shape === 'vertical')) localStorage.removeItem(KIT_KEY);
    else localStorage.setItem(KIT_KEY, JSON.stringify(kit));
  } catch {
    // New clips start plain next time, no worse.
  }
}

/** What a new clip's request carries from the kit. */
export function kitRequest(kit: BrandKit = recallKit()): {
  readonly brandJson?: string;
  readonly shape?: FrameShape;
} {
  return {
    ...(kit.brand ? { brandJson: JSON.stringify(kit.brand) } : {}),
    ...(kit.shape && kit.shape !== 'vertical' ? { shape: kit.shape } : {}),
  };
}

/** Whether a clip already has the kit's brand. */
export function hasKit(document: Pick<EditIr, 'brand'> | null, kit: BrandKit): boolean {
  return JSON.stringify(document?.brand ?? null) === JSON.stringify(kit.brand ?? null);
}

/** How the editor reaches the person's pictures and sounds. */
export interface AssetAccess {
  readonly list: (kind: Asset['kind']) => Promise<readonly Asset[]>;
  /** Bring one in through the host's picker; null when it was closed. */
  readonly bring: (kind: Asset['kind'], license: AssetLicense) => Promise<Asset | null>;
  readonly url: (hash: string) => string;
}

/** What a person can say about their right to use a picture or a sound. */
export const LICENSES: readonly { readonly value: AssetLicense; readonly label: string }[] = [
  { value: 'own_content', label: 'I made it' },
  { value: 'licensed', label: 'I have a licence for it' },
  { value: 'royalty_free', label: 'It is royalty-free' },
  { value: 'public_domain', label: 'It is in the public domain' },
];
