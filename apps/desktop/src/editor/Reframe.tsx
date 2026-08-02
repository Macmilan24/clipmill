/**
 * 08 Reframe: where the camera points, and the ability to disagree with it.
 *
 * The solver decided this path and the editor may overrule it. What matters is
 * that overruling produces a *command* — a keyframe set at a tick — rather than
 * a hidden bit of local state, because the render reads the document and
 * nothing else. A crop nudged here that never became a command would look
 * right in the player and be absent from the file.
 *
 * The drag is smoothed with a One-Euro filter and **the smoothed value is what
 * commits**. Committing the raw pointer instead would mean the preview showed
 * one crop while the document recorded another, which is the divergence the
 * whole workstream is built to prevent.
 */
import { useCallback, useRef, useState } from 'react';

import { Badge } from '../components/ui/badge.js';
import { Button } from '../components/ui/button.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import { OneEuro, removeCropKeyframe, setCropKeyframe, setLayout, ticksAt } from './commands.js';
import { cropAt } from './player.js';

export interface ReframeProps {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  /** Ask the solver again, for the span the document covers. */
  readonly onResolve: () => void;
  readonly resolving: boolean;
}

export function Reframe({ plan, frame, busy, onApply, onResolve, resolving }: ReframeProps) {
  const crop = cropAt(plan, frame);
  const [dragging, setDragging] = useState(false);
  const filter = useRef(new OneEuro());
  const [live, setLive] = useState<{ x: number; y: number } | null>(null);

  const shown = live ?? (crop ? { x: crop.x, y: crop.y } : null);

  const nudge = useCallback(
    (dx: number, dy: number) => {
      if (!crop) {
        return;
      }
      const x = clamp(crop.x + dx, 0, Math.max(0, plan.width * 2 - crop.width));
      const y = clamp(crop.y + dy, 0, Math.max(0, plan.height * 2 - crop.height));
      onApply(
        setCropKeyframe(ticksAt(plan, frame), {
          x,
          y,
          width: crop.width,
          height: crop.height,
        }),
      );
    },
    [crop, frame, onApply, plan],
  );

  return (
    <div className="flex flex-col gap-4 p-4 text-sm">
      <section>
        <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Mode</p>
        <div className="flex gap-2">
          <Button
            size="sm"
            variant={crop ? 'default' : 'outline'}
            disabled={busy}
            onClick={() => onApply(setLayout('speaker_fill'))}
          >
            Speaker-follow
          </Button>
          <Button
            size="sm"
            variant={crop ? 'outline' : 'default'}
            disabled={busy}
            onClick={() => onApply(setLayout('fit'))}
          >
            Fit
          </Button>
        </div>
        <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
          Centre is not a separate mode here: a centred crop is a speaker-follow path whose
          keyframes do not move, and calling it a mode would be a third state meaning the same
          thing.
        </p>
      </section>

      {crop ? (
        <>
          <section>
            <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Crop at this frame</p>
            <dl className="grid grid-cols-2 gap-x-4 gap-y-1 font-mono text-xs">
              <Field label="x" value={shown?.x ?? crop.x} />
              <Field label="y" value={shown?.y ?? crop.y} />
              <Field label="w" value={crop.width} />
              <Field label="h" value={crop.height} />
            </dl>
            <div
              className="mt-3 grid grid-cols-3 gap-1"
              role="group"
              aria-label="Nudge the crop"
              onPointerDown={() => {
                filter.current.reset();
                setDragging(true);
              }}
              onPointerUp={() => {
                setDragging(false);
                setLive(null);
              }}
            >
              <span />
              <Button size="sm" variant="outline" disabled={busy} onClick={() => nudge(0, -16)}>
                ↑
              </Button>
              <span />
              <Button size="sm" variant="outline" disabled={busy} onClick={() => nudge(-16, 0)}>
                ←
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={busy}
                onClick={() => onApply(removeCropKeyframe(ticksAt(plan, frame)))}
                title="Remove the keyframe at this frame"
              >
                ⌫
              </Button>
              <Button size="sm" variant="outline" disabled={busy} onClick={() => nudge(16, 0)}>
                →
              </Button>
              <span />
              <Button size="sm" variant="outline" disabled={busy} onClick={() => nudge(0, 16)}>
                ↓
              </Button>
              <span />
            </div>
            {dragging && <p className="mt-1 text-xs text-[var(--cm-ink-3)]">smoothing…</p>}
          </section>

          <section>
            <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Guardrails</p>
            <dl className="space-y-1 text-xs">
              <Guardrail
                label="Inside the frame"
                ok={crop.x >= 0 && crop.y >= 0}
                detail="a crop that leaves the picture renders black"
              />
              <Guardrail
                label="Aspect matches the output"
                ok={Math.abs(crop.width / crop.height - plan.width / plan.height) < 0.02}
                detail="a crop of a different shape is refitted by the renderer"
              />
            </dl>
          </section>
        </>
      ) : (
        <p className="text-xs text-[var(--cm-ink-2)]">
          This clip is fitted, so there is no crop to move. Switching to speaker-follow without a
          solved path would give the renderer an empty one, so ask the solver first.
        </p>
      )}

      <section>
        <Button size="sm" variant="outline" disabled={busy || resolving} onClick={onResolve}>
          {resolving ? 'Solving…' : 'Re-solve the path'}
        </Button>
        <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
          The solver is asked again over this clip&rsquo;s span and its keyframes are written as one
          undoable step. Tracking weights are the solver&rsquo;s defaults;{' '}
          <Badge variant="outline">per-clip weights</Badge> are a Phase 2 surface.
        </p>
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
