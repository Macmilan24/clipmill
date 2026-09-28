/**
 * The brand over the preview: the progress bar and the logo, placed from the
 * numbers the render uses, so what is seen here is where the export puts it.
 * Neither takes the pointer.
 */
import type { PreviewPlan } from '../daemon/client.js';

const share = (pixels: number, of: number) => `${(pixels / Math.max(1, of)) * 100}%`;

export function BrandLayer({
  plan,
  frame,
  assetUrl,
}: {
  readonly plan: Pick<PreviewPlan, 'width' | 'height' | 'frameCount' | 'progress' | 'logo'>;
  readonly frame: number;
  /** Where a picture loads from; absent shows no logo. */
  readonly assetUrl?: ((hash: string) => string) | null | undefined;
}) {
  const bar = plan.progress;
  const logo = plan.logo;
  return (
    <>
      {bar && (
        <div
          className="edit-brand-bar"
          aria-hidden="true"
          data-testid="progress-bar"
          style={{
            [bar.edge]: 0,
            height: share(bar.thickness, plan.height),
            // As the render slides it in: full on the last frame.
            width: share(Math.min(frame + 1, plan.frameCount), plan.frameCount),
            background: bar.colour,
          }}
        />
      )}
      {logo && assetUrl && (
        <img
          className="edit-brand-logo"
          alt=""
          aria-hidden="true"
          data-testid="brand-logo"
          src={assetUrl(logo.asset)}
          style={{
            [logo.corner.endsWith('left') ? 'left' : 'right']: share(logo.insetX, plan.width),
            [logo.corner.startsWith('top') ? 'top' : 'bottom']: share(logo.insetY, plan.height),
            width: share(logo.side, plan.width),
            height: share(logo.side, plan.height),
            objectPosition: `${logo.corner.endsWith('left') ? 'left' : 'right'} ${
              logo.corner.startsWith('top') ? 'top' : 'bottom'
            }`,
            opacity: logo.opacity / 100,
          }}
        />
      )}
    </>
  );
}
