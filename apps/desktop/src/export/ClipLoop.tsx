/**
 * The clip being exported, shown: a few seconds of it on a muted loop, framed
 * and captioned by the same drawing the Editor uses, so what the button makes
 * is what is on screen. A person who asks for less motion gets a still.
 */
import { type JSX, useEffect, useMemo, useRef, useState } from 'react';

import type { ShellApi } from '../daemon/api.js';
import type { PreviewPlan } from '../daemon/client.js';
import { CaptionCanvas } from '../editor/CaptionCanvas.js';
import { BrandLayer } from '../editor/BrandLayer.js';
import { CompositionCanvas } from '../editor/CompositionCanvas.js';
import { exactCaptionsOf } from '../editor/exactCaptions.js';
import { frameAtProxySeconds, proxySecondsAt } from '../editor/player.js';

/** How much of the clip the loop shows. */
const LOOP_SECONDS = 6;
/** A section shorter than this is a cut away, not what the clip looks like. */
const SHORTEST_SECTION_SECONDS = 2;

export function ClipLoop({
  api,
  projectId,
  plan,
}: {
  readonly api: ShellApi;
  readonly projectId: string;
  /** The clip's preview plan, as the screen already read it. */
  readonly plan: PreviewPlan | null;
}): JSX.Element | null {
  if (!plan) return null;
  return <Loop api={api} projectId={projectId} plan={plan} />;
}

function Loop({
  api,
  projectId,
  plan: full,
}: {
  readonly api: ShellApi;
  readonly projectId: string;
  readonly plan: PreviewPlan;
}) {
  const video = useRef<HTMLVideoElement>(null);
  // The first section long enough to show what the clip looks like, played
  // on its own: no blends, so no second decoder.
  const segment = useMemo(() => {
    const seconds = (count: number) => (count * full.rateDen) / Math.max(1, full.rateNum);
    return (
      full.segments.find(
        (part) => seconds(part.endFrame - part.firstFrame) >= SHORTEST_SECTION_SECONDS,
      ) ?? full.segments[0]
    );
  }, [full]);
  const plan = useMemo(() => ({ ...full, transitions: [] }), [full]);
  const proxy = segment
    ? plan.proxies.find((item) => item.sourceFingerprint === segment.sourceFingerprint)
    : undefined;
  const url = proxy ? api.mediaUrl(projectId, proxy.artifactId, proxy.file) : null;
  const first = segment?.firstFrame ?? 0;
  const last = segment
    ? Math.min(segment.endFrame, first + Math.round((LOOP_SECONDS * plan.rateNum) / plan.rateDen))
    : 0;
  const [frame, setFrame] = useState(first);
  const captions = useMemo(() => exactCaptionsOf(plan, api.captionFontUrl ?? null), [plan, api]);
  const still =
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  useEffect(() => {
    const media = video.current;
    const start = proxySecondsAt(plan, first);
    if (!media || start === null) return;
    const begin = () => {
      // A still is a second in, past the cut's first frame.
      media.currentTime = still
        ? (proxySecondsAt(plan, Math.min(last - 1, first + 30)) ?? start)
        : start;
      if (!still) void media.play().catch(() => undefined);
    };
    if (media.readyState >= 1) begin();
    else media.addEventListener('loadedmetadata', begin, { once: true });
    return () => media.removeEventListener('loadedmetadata', begin);
  }, [plan, first, last, still, url]);

  if (!segment || !url) return null;
  return (
    <div
      className="export-loop"
      style={{ aspectRatio: `${plan.width} / ${plan.height}`, containerType: 'inline-size' }}
      aria-label="The clip, as it will export"
      role="img"
    >
      {/* eslint-disable-next-line jsx-a11y/media-has-caption -- muted; the
          captions are drawn over it from the plan. */}
      <video
        ref={video}
        src={url}
        className="export-loop-decoder"
        crossOrigin="anonymous"
        muted
        playsInline
        preload="auto"
        aria-hidden="true"
      />
      <CompositionCanvas
        video={video}
        mediaKey={url}
        plan={plan}
        frame={frame}
        onFrame={(seconds) => {
          const at = frameAtProxySeconds(plan, segment, seconds);
          const media = video.current;
          if (!still && media && at >= last - 1) {
            const start = proxySecondsAt(plan, first);
            if (start !== null) media.currentTime = start;
          }
          setFrame(at);
          return at;
        }}
      />
      <BrandLayer plan={plan} frame={frame} assetUrl={api.assetUrl ?? null} />
      {captions && (
        <CaptionCanvas
          ass={captions.ass}
          faces={captions.faces}
          family={captions.family}
          seconds={(frame * plan.rateDen) / Math.max(1, plan.rateNum)}
          frameWidth={plan.width}
          frameHeight={plan.height}
        />
      )}
    </div>
  );
}
