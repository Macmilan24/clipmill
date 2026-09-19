/**
 * 08 Reframe: where the camera points, and the ability to disagree with it.
 *
 * The solver decided this path and the editor may overrule it. What matters is
 * that overruling produces a *command* — a keyframe set at a tick — rather than
 * a hidden bit of local state, because the render reads the document and
 * nothing else. A crop nudged here that never became a command would look
 * right in the player and be absent from the file.
 *
 * Nudges are explicit keyframes; every change goes through the edit command log.
 */
import { useCallback, useState } from 'react';
import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, RotateCcw } from 'lucide-react';

import { Button } from '../components/ui/button.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import { removeCropKeyframe, segmentTicksAt, setCropKeyframe, setLayout } from './commands.js';
import { cropAt, segmentAt, sourceOf } from './player.js';

export interface ReframeProps {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  /** Ask the solver again, for the span the document covers. */
  readonly onResolve: () => void;
  readonly resolving: boolean;
  /**
   * Why the solver cannot be asked, or `null` when it can.
   *
   * Shown rather than only obeyed. A button that is disabled without saying why
   * is the same failure as one that is enabled and does nothing — the editor is
   * left guessing whether they misread the control or the product is broken.
   */
  readonly resolveRefusal: string | null;
}

export function Reframe({
  plan,
  frame,
  busy,
  onApply,
  onResolve,
  resolving,
  resolveRefusal,
}: ReframeProps) {
  const [portrait, setPortrait] = useState<'upper' | 'lower'>('upper');
  const hasTwo = cropAt(plan, frame, true) !== null;
  const secondary = hasTwo && portrait === 'lower';
  const crop = cropAt(plan, frame, secondary);
  // The crop lives in the source frame, and so does the keyframe's clock: a
  // keyframe is at segment-local ticks, so the segment the playhead is in is
  // what a nudge is addressed to.
  const segment = segmentAt(plan, frame);
  const source = segment ? sourceOf(plan, segment) : null;
  const at = segmentTicksAt(plan, frame);

  const nudge = useCallback(
    (dx: number, dy: number) => {
      if (!crop) {
        return;
      }
      const frameWidth = source?.displayWidth ?? plan.width * 2;
      const frameHeight = source?.displayHeight ?? plan.height * 2;
      const x = clamp(crop.x + dx, 0, Math.max(0, frameWidth - crop.width));
      const y = clamp(crop.y + dy, 0, Math.max(0, frameHeight - crop.height));
      onApply(
        setCropKeyframe(
          at.tTicks,
          {
            x,
            y,
            width: crop.width,
            height: crop.height,
          },
          at.segmentId,
          secondary,
        ),
      );
    },
    [at.segmentId, at.tTicks, crop, onApply, plan, source, secondary],
  );

  return (
    <div className="flex flex-col gap-4 p-4 text-sm">
      <section>
        <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Mode</p>
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant={crop ? 'default' : 'outline'}
            disabled={busy || resolving || (!crop && resolveRefusal !== null)}
            aria-pressed={crop !== null}
            onClick={() => {
              if (!crop) onResolve();
            }}
          >
            {hasTwo ? 'Two portraits' : 'Face crop'}
          </Button>
          <Button
            size="sm"
            variant={crop || segment?.framingWarning ? 'outline' : 'default'}
            disabled={busy}
            aria-pressed={crop === null && !segment?.framingWarning}
            onClick={() => {
              if (crop || segment?.framingWarning) onApply(setLayout('fit', at.segmentId));
            }}
          >
            Fit
          </Button>
          {!hasTwo && segment?.hasTwoUpPaths && (
            <Button
              size="sm"
              variant="outline"
              disabled={busy}
              onClick={() => onApply(setLayout('two_up', at.segmentId))}
            >
              Restore two portraits
            </Button>
          )}
        </div>
        <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
          Frame a reliably visible face, show both people in equal portraits, or fit the entire
          picture.
        </p>
      </section>

      {crop ? (
        <>
          {hasTwo && (
            <section>
              <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Portrait to adjust</p>
              <div className="flex gap-2" role="group" aria-label="Choose portrait">
                <Button
                  size="sm"
                  variant={portrait === 'upper' ? 'default' : 'outline'}
                  aria-pressed={portrait === 'upper'}
                  onClick={() => setPortrait('upper')}
                >
                  Upper
                </Button>
                <Button
                  size="sm"
                  variant={portrait === 'lower' ? 'default' : 'outline'}
                  aria-pressed={portrait === 'lower'}
                  onClick={() => setPortrait('lower')}
                >
                  Lower
                </Button>
              </div>
              <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
                Both people stay visible. Framing does not guess who is speaking.
              </p>
            </section>
          )}
          <section>
            <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Crop at this frame</p>
            <dl className="grid grid-cols-2 gap-x-4 gap-y-1 font-mono text-xs">
              <Field label="x" value={crop.x} />
              <Field label="y" value={crop.y} />
              <Field label="w" value={crop.width} />
              <Field label="h" value={crop.height} />
            </dl>
            <div className="mt-3 grid grid-cols-3 gap-1" role="group" aria-label="Nudge the crop">
              <span />
              <Button
                size="sm"
                variant="outline"
                disabled={busy}
                aria-label="Move crop up"
                onClick={() => nudge(0, -16)}
              >
                <ArrowUp className="size-4" />
              </Button>
              <span />
              <Button
                size="sm"
                variant="outline"
                disabled={busy}
                aria-label="Move crop left"
                onClick={() => nudge(-16, 0)}
              >
                <ArrowLeft className="size-4" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={busy}
                onClick={() => onApply(removeCropKeyframe(at.tTicks, at.segmentId, secondary))}
                title="Remove the keyframe at this frame"
                aria-label="Remove crop keyframe"
              >
                <RotateCcw className="size-4" />
              </Button>
              <Button
                size="sm"
                variant="outline"
                disabled={busy}
                aria-label="Move crop right"
                onClick={() => nudge(16, 0)}
              >
                <ArrowRight className="size-4" />
              </Button>
              <span />
              <Button
                size="sm"
                variant="outline"
                disabled={busy}
                aria-label="Move crop down"
                onClick={() => nudge(0, 16)}
              >
                <ArrowDown className="size-4" />
              </Button>
              <span />
            </div>
          </section>

          <section>
            <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Guardrails</p>
            <dl className="space-y-1 text-xs">
              <Guardrail
                label="Inside the frame"
                ok={
                  source !== null &&
                  crop.x >= 0 &&
                  crop.y >= 0 &&
                  crop.x + crop.width <= source.displayWidth &&
                  crop.y + crop.height <= source.displayHeight
                }
                detail="a crop that leaves the picture renders black"
              />
              <Guardrail
                label="Aspect matches the output"
                ok={
                  Math.abs(
                    crop.width * (plan.height / (hasTwo ? 2 : 1)) - crop.height * plan.width,
                  ) <=
                  plan.height / (hasTwo ? 2 : 1)
                }
                detail="a crop with the wrong proportions must be corrected before export"
              />
            </dl>
          </section>
        </>
      ) : (
        <p className="text-xs text-[var(--cm-ink-2)]">
          {resolveRefusal
            ? 'The whole frame is shown. Face crop is unavailable until a face track is ready.'
            : 'Choose Face crop to calculate a crop for this clip.'}
        </p>
      )}

      <section>
        <Button
          size="sm"
          variant="outline"
          disabled={busy || resolving || resolveRefusal !== null || hasTwo}
          onClick={onResolve}
        >
          {resolving ? 'Solving…' : 'Re-solve the path'}
        </Button>
        {hasTwo ? (
          <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
            Adjust each portrait above. After switching to Fit, Restore two portraits brings this
            composition back.
          </p>
        ) : resolveRefusal ? (
          <p className="mt-2 text-xs text-[var(--cm-ink-2)]" role="note">
            {resolveRefusal}
          </p>
        ) : (
          <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
            Recalculate framing for this clip. You can undo this change.
          </p>
        )}
      </section>
    </div>
  );
}

function Field({ label, value }: { readonly label: string; readonly value: number }) {
  return (
    <div className="flex justify-between">
      <dt className="text-[var(--cm-ink-2)]">{label}</dt>
      <dd className="text-[var(--cm-ink-1)]">{Math.round(value)}</dd>
    </div>
  );
}

function Guardrail({
  label,
  ok,
  detail,
}: {
  readonly label: string;
  readonly ok: boolean;
  readonly detail: string;
}) {
  return (
    <div className="flex items-baseline justify-between gap-2">
      <dt className="text-[var(--cm-ink-2)]">{label}</dt>
      <dd
        className={ok ? 'text-[var(--cm-success-ink)]' : 'text-[var(--cm-warning-ink)]'}
        title={detail}
      >
        {ok ? 'ok' : 'check'}
      </dd>
    </div>
  );
}

function clamp(value: number, low: number, high: number): number {
  return Math.max(low, Math.min(high, value));
}
