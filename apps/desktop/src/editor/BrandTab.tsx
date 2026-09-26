/**
 * The Brand tab: a progress bar and a logo over every frame, and the kit
 * that starts new clips with them — and in the shape they are framed for.
 * What the clip is, in numbers, is kept at the bottom.
 */
import { ImagePlus, Stamp, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Button } from '../components/ui/button.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import type { Asset, AssetLicense, EditCommandJson, PreviewPlan } from '../daemon/client.js';
import type { EditIr } from '@clipmill/contracts';
import {
  type AssetAccess,
  BAR_THICKNESSES,
  type BrandKit,
  DEFAULT_BAR,
  LICENSES,
  LOGO_OPACITIES,
  LOGO_SIZES,
  type Logo,
  hasKit,
  recallKit,
  rememberKit,
  setBrand,
  withAsset,
  withBrand,
} from './brand.js';
import { CommitSlider, Field, Swatch } from './controls.js';
import { FRAME_SHAPES, shapeOfFrame } from './layouts.js';

const CORNERS = [
  ['top_left', 'Top left'],
  ['top_right', 'Top right'],
  ['bottom_left', 'Bottom left'],
  ['bottom_right', 'Bottom right'],
] as const;

export function BrandTab({
  plan,
  document,
  busy,
  onApply,
  assets,
}: {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly assets: AssetAccess | null;
}) {
  const brand = document?.brand ?? null;
  const bar = brand?.progress;
  const logo = brand?.logo;
  const [kit, setKit] = useState<BrandKit>(() => recallKit());
  const shape = shapeOfFrame(plan);
  const saved = hasKit(document, kit) && (kit.shape ?? 'vertical') === shape;
  const apply = (next: ReturnType<typeof withBrand>, list?: NonNullable<EditIr['assets']>) =>
    onApply(setBrand(next, list));

  return (
    <div className="review-panel-body">
      <section className="review-section">
        <h3 className="review-section-title">Progress bar</h3>
        <Field label="Bar">
          <div className="review-segmented" role="group" aria-label="Progress bar">
            <button
              type="button"
              aria-pressed={!bar}
              disabled={busy || !document}
              onClick={() => apply(withBrand(brand, { progress: null }))}
            >
              Off
            </button>
            <button
              type="button"
              aria-pressed={Boolean(bar)}
              disabled={busy || !document}
              onClick={() => apply(withBrand(brand, { progress: bar ?? DEFAULT_BAR }))}
            >
              On
            </button>
          </div>
        </Field>
        {bar && (
          <>
            <Field label="Colour">
              <Swatch
                label="Bar colour"
                value={bar.colour}
                disabled={busy}
                onCommit={(colour) => apply(withBrand(brand, { progress: { ...bar, colour } }))}
              />
            </Field>
            <Field label="Edge">
              <div className="review-segmented" role="group" aria-label="Bar edge">
                {(['top', 'bottom'] as const).map((edge) => (
                  <button
                    key={edge}
                    type="button"
                    aria-pressed={bar.edge === edge}
                    disabled={busy}
                    onClick={() => apply(withBrand(brand, { progress: { ...bar, edge } }))}
                  >
                    {edge === 'top' ? 'Top' : 'Bottom'}
                  </button>
                ))}
              </div>
            </Field>
            <Field label="Thickness">
              <CommitSlider
                label="Bar thickness"
                min={BAR_THICKNESSES.min}
                max={BAR_THICKNESSES.max}
                value={bar.thickness}
                disabled={busy}
                format={(value) => `${value} px`}
                onCommit={(thickness) =>
                  apply(withBrand(brand, { progress: { ...bar, thickness } }))
                }
              />
            </Field>
          </>
        )}
        <p className="review-footnote">
          It fills from the left as the clip plays, so a viewer sees how much is left.
        </p>
      </section>

      <LogoSection
        brand={brand}
        logo={logo}
        document={document}
        busy={busy}
        assets={assets}
        onApply={onApply}
      />

      <section className="review-section">
        <h3 className="review-section-title">Your brand kit</h3>
        <p className="review-footnote">
          {kit.brand || (kit.shape && kit.shape !== 'vertical')
            ? `New clips start with ${kitSummary(kit)}.`
            : 'Save this clip’s progress bar, logo and shape, and every new clip starts with them.'}
        </p>
        <div className="edit-inline">
          <Button
            size="sm"
            variant="outline"
            disabled={busy || !document || saved}
            onClick={() => {
              const license = document?.assets?.find((item) => item.hash === brand?.logo?.asset)
                ?.license as AssetLicense | undefined;
              const next: BrandKit = {
                ...(brand ? { brand } : {}),
                ...(shape !== 'vertical' ? { shape } : {}),
                ...(license ? { logoLicense: license } : {}),
              };
              rememberKit(next);
              setKit(next);
            }}
          >
            <Stamp className="size-4" aria-hidden="true" />
            {saved ? 'Saved as your kit' : 'Start new clips like this'}
          </Button>
          {kit.brand && !hasKit(document, kit) && (
            <Button
              size="sm"
              variant="ghost"
              disabled={busy || !document}
              onClick={() => {
                const logoAsset = kit.brand?.logo?.asset;
                const listed = document?.assets?.find((item) => item.hash === logoAsset);
                // A kit's logo comes with the licence it was saved with.
                apply(
                  kit.brand ?? null,
                  logoAsset && !listed
                    ? withAsset(document?.assets, {
                        hash: logoAsset,
                        license: kit.logoLicense ?? 'own_content',
                      })
                    : undefined,
                );
              }}
            >
              Use your kit on this clip
            </Button>
          )}
        </div>
      </section>
    </div>
  );
}

function LogoSection({
  brand,
  logo,
  document,
  busy,
  assets,
  onApply,
}: {
  readonly brand: EditIr['brand'] | null;
  readonly logo: Logo | undefined;
  readonly document: EditIr | null;
  readonly busy: boolean;
  readonly assets: AssetAccess | null;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const [pictures, setPictures] = useState<readonly Asset[]>([]);
  const [license, setLicense] = useState<AssetLicense>('own_content');
  const [problem, setProblem] = useState<string | null>(null);
  const [bringing, setBringing] = useState(false);
  useEffect(() => {
    if (!assets) return undefined;
    let current = true;
    void assets
      .list('image')
      .then((found) => {
        if (current) setPictures(found);
      })
      .catch(() => {});
    return () => {
      current = false;
    };
  }, [assets]);
  const choose = (picture: Asset) => {
    const next: Logo = logo
      ? { ...logo, asset: picture.hash }
      : {
          asset: picture.hash,
          corner: 'top_left',
          size: LOGO_SIZES.default,
          opacity: LOGO_OPACITIES.default,
        };
    onApply(setBrand(withBrand(brand, { logo: next }), withAsset(document?.assets, picture)));
  };
  const bring = async () => {
    if (!assets) return;
    setBringing(true);
    setProblem(null);
    try {
      const picture = await assets.bring('image', license);
      if (picture) {
        setPictures((current) => [
          picture,
          ...current.filter((item) => item.hash !== picture.hash),
        ]);
        choose(picture);
      }
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
    } finally {
      setBringing(false);
    }
  };
  const current = logo ? pictures.find((picture) => picture.hash === logo.asset) : undefined;

  return (
    <section className="review-section">
      <h3 className="review-section-title">Logo</h3>
      {!assets && (
        <p className="review-footnote">Pictures can be brought in in the ClipMill app.</p>
      )}
      {assets && (
        <>
          {pictures.length > 0 && (
            <div className="edit-brand-pictures" role="group" aria-label="Your pictures">
              {pictures.slice(0, 8).map((picture) => (
                <button
                  key={picture.hash}
                  type="button"
                  title={picture.name}
                  aria-label={`Use ${picture.name}`}
                  aria-pressed={logo?.asset === picture.hash}
                  disabled={busy || !document}
                  onClick={() => choose(picture)}
                >
                  <img src={assets.url(picture.hash)} alt="" />
                </button>
              ))}
            </div>
          )}
          <div className="flex flex-wrap items-center gap-2">
            <Select value={license} onValueChange={(value) => setLicense(value as AssetLicense)}>
              <SelectTrigger aria-label="Whose picture it is" className="h-8 w-[190px] text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {LICENSES.map((item) => (
                  <SelectItem key={item.value} value={item.value}>
                    {item.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              size="sm"
              variant="outline"
              disabled={busy || bringing || !document}
              onClick={() => void bring()}
            >
              <ImagePlus className="size-4" aria-hidden="true" />
              {bringing ? 'Bringing it in…' : 'Choose a picture'}
            </Button>
          </div>
          {problem && <p className="review-footnote text-[var(--cm-warning-ink)]">{problem}</p>}
          <p className="review-footnote">
            A PNG with a clear background sits best over a picture. Copies stay in ClipMill, so
            moving the file changes nothing.
          </p>
        </>
      )}
      {logo && (
        <>
          <Field label="Corner">
            <div className="review-segmented" role="group" aria-label="Logo corner">
              {CORNERS.map(([corner, label]) => (
                <button
                  key={corner}
                  type="button"
                  aria-pressed={logo.corner === corner}
                  disabled={busy}
                  onClick={() => onApply(setBrand(withBrand(brand, { logo: { ...logo, corner } })))}
                >
                  {label}
                </button>
              ))}
            </div>
          </Field>
          <Field label="Size">
            <CommitSlider
              label="Logo size"
              min={LOGO_SIZES.min / 10}
              max={LOGO_SIZES.max / 10}
              value={Math.round(logo.size / 10)}
              disabled={busy}
              format={(value) => `${value}%`}
              onCommit={(value) =>
                onApply(setBrand(withBrand(brand, { logo: { ...logo, size: value * 10 } })))
              }
            />
          </Field>
          <Field label="Opacity">
            <CommitSlider
              label="Logo opacity"
              min={LOGO_OPACITIES.min}
              max={LOGO_OPACITIES.max}
              value={logo.opacity}
              disabled={busy}
              format={(value) => `${value}%`}
              onCommit={(opacity) =>
                onApply(setBrand(withBrand(brand, { logo: { ...logo, opacity } })))
              }
            />
          </Field>
          <Button
            size="sm"
            variant="ghost"
            className="self-start"
            disabled={busy}
            onClick={() => onApply(setBrand(withBrand(brand, { logo: null })))}
          >
            <Trash2 className="size-4" aria-hidden="true" />
            Remove the logo{current ? ` (${current.name})` : ''}
          </Button>
        </>
      )}
    </section>
  );
}

function kitSummary(kit: BrandKit): string {
  const parts: string[] = [];
  if (kit.brand?.progress) parts.push('a progress bar');
  if (kit.brand?.logo) parts.push('your logo');
  const shape = FRAME_SHAPES.find((item) => item.shape === (kit.shape ?? 'vertical'));
  if (kit.shape && kit.shape !== 'vertical' && shape) parts.push(`the ${shape.ratio} shape`);
  if (parts.length === 0) return 'nothing yet';
  if (parts.length === 1) return parts[0]!;
  return `${parts.slice(0, -1).join(', ')} and ${parts.at(-1)}`;
}
