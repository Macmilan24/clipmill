/**
 * B-roll over the preview, as the render lays it: over the program's own
 * picture, under the brand, the emoji and the captions. A picture is drawn
 * from the person's assets and moves closer as the render's zoom moves it;
 * footage plays from its recording's proxy on the program's clock, without
 * its sound. Picking it selects it, so the picture under it is not reframed
 * by a drag that cannot be seen.
 */
import { type CSSProperties, useEffect, useRef } from 'react';

import type { PreviewCutaway, PreviewPlan } from '../daemon/client.js';
import { cutawayAt, footageSecondsAt, pushScale } from './cutaways.js';
import type { EditorSelection } from './selection.js';

/** How far footage may wander from the picture before it is put back. */
const DRIFT_SECONDS = 0.15;

export function CutawayLayer({
  plan,
  frame,
  playing,
  assetUrl,
  proxyUrls,
  selection,
  onSelect,
}: {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly playing: boolean;
  readonly assetUrl: ((hash: string) => string) | null;
  readonly proxyUrls: ReadonlyMap<string, string>;
  readonly selection: EditorSelection;
  readonly onSelect: (selection: EditorSelection) => void;
}) {
  const cutaway = cutawayAt(plan, frame);
  if (!cutaway) return null;
  const selected = selection.kind === 'cutaway' && selection.cutawayId === cutaway.cutawayId;
  // The render blurs by 40 pixels of a 1080-pixel short side.
  const blur = `${((40 * Math.min(plan.width, plan.height)) / 1080 / Math.max(1, plan.width)) * 100}cqw`;
  const scale: CSSProperties = { transform: `scale(${pushScale(cutaway, frame)})` };
  return (
    <div
      className="edit-cutaway"
      data-testid="cutaway"
      data-kind={cutaway.kind}
      data-fit={cutaway.fit}
      data-selected={selected ? 'true' : undefined}
      onPointerDown={(event) => {
        if (event.button > 0) return;
        event.stopPropagation();
        onSelect({ kind: 'cutaway', cutawayId: cutaway.cutawayId });
      }}
    >
      {cutaway.kind === 'picture' ? (
        assetUrl && cutaway.asset ? (
          <div className="edit-cutaway-frame" style={scale}>
            {cutaway.fit === 'fit' && (
              <img
                className="edit-cutaway-backdrop"
                src={assetUrl(cutaway.asset)}
                alt=""
                draggable={false}
                style={{ filter: `blur(${blur})` }}
              />
            )}
            <img
              className="edit-cutaway-picture"
              src={assetUrl(cutaway.asset)}
              alt=""
              draggable={false}
            />
          </div>
        ) : (
          <p className="edit-cutaway-missing">A picture from your folder shows here.</p>
        )
      ) : (
        <Footage
          key={cutaway.cutawayId}
          plan={plan}
          cutaway={cutaway}
          frame={frame}
          playing={playing}
          url={
            cutaway.sourceFingerprint ? (proxyUrls.get(cutaway.sourceFingerprint) ?? null) : null
          }
          blur={blur}
        />
      )}
    </div>
  );
}

function Footage({
  plan,
  cutaway,
  frame,
  playing,
  url,
  blur,
}: {
  readonly plan: PreviewPlan;
  readonly cutaway: PreviewCutaway;
  readonly frame: number;
  readonly playing: boolean;
  readonly url: string | null;
  readonly blur: string;
}) {
  const video = useRef<HTMLVideoElement>(null);
  const backdrop = useRef<HTMLCanvasElement>(null);
  const last = useRef(frame);
  useEffect(() => {
    const element = video.current;
    const expected = footageSecondsAt(plan, cutaway, frame);
    if (!element || expected === null) return;
    const jumped = Math.abs(frame - last.current) > 2;
    if (!playing || jumped || Math.abs(element.currentTime - expected) > DRIFT_SECONDS) {
      try {
        if (Math.abs(element.currentTime - expected) > 1 / 90) element.currentTime = expected;
      } catch {
        // Not loaded yet: the next frame puts it in place.
      }
    }
    if (playing && element.paused) {
      // Older webviews answer play() with nothing rather than a promise.
      const started = element.play() as Promise<void> | undefined;
      if (started) void started.catch(() => undefined);
    }
    if (!playing && !element.paused) element.pause();
    last.current = frame;
  }, [plan, cutaway, frame, playing]);
  // The whole picture over a blurred copy: the copy painted small from the
  // same video, which the page then blurs and stretches to cover.
  useEffect(() => {
    if (cutaway.fit !== 'fit') return undefined;
    let request = 0;
    const paint = () => {
      const element = video.current;
      const context = backdrop.current?.getContext('2d');
      if (element && context && element.readyState >= 2 && element.videoWidth > 0) {
        // Covering the canvas as the render's copy covers the frame.
        const { width, height } = context.canvas;
        const cover = Math.max(width / element.videoWidth, height / element.videoHeight);
        const [cropWidth, cropHeight] = [width / cover, height / cover];
        context.drawImage(
          element,
          (element.videoWidth - cropWidth) / 2,
          (element.videoHeight - cropHeight) / 2,
          cropWidth,
          cropHeight,
          0,
          0,
          width,
          height,
        );
      }
      request = requestAnimationFrame(paint);
    };
    request = requestAnimationFrame(paint);
    return () => cancelAnimationFrame(request);
  }, [cutaway.fit]);
  if (!url) {
    return <p className="edit-cutaway-missing">This recording has no preview copy yet.</p>;
  }
  return (
    <div className="edit-cutaway-frame">
      {cutaway.fit === 'fit' && (
        <canvas
          ref={backdrop}
          className="edit-cutaway-backdrop"
          width={64}
          height={Math.max(1, Math.round((64 * plan.height) / Math.max(1, plan.width)))}
          style={{ filter: `blur(${blur})` }}
        />
      )}
      {/* eslint-disable-next-line jsx-a11y/media-has-caption -- B-roll plays
          without its sound; the clip's captions are drawn above it. */}
      <video
        ref={video}
        className="edit-cutaway-picture"
        src={url}
        muted
        playsInline
        preload="auto"
        crossOrigin="anonymous"
        data-testid="cutaway-footage"
      />
    </div>
  );
}
