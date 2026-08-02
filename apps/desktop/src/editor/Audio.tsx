/**
 * Audio: the gain curve, and an honest meter beside it.
 *
 * The curve is editable because an editor knows things a normalizer does not —
 * a laugh that clips, a sentence that drops. What the meter must not do is
 * pretend to be the export's measurement: the render runs a real two-pass
 * loudnorm and this is a curve applied over a proxy. So the meter reads the
 * *target* and the curve's own offset, and says which is which.
 *
 * That distinction is written into `docs/preview-parity.md` as a tolerance
 * rather than hidden here, because an editor who thought this number was the
 * delivered loudness would trust it in exactly the situation where it is wrong.
 */
import { Button } from '../components/ui/button.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import { removeGainPoint, setGain, ticksAt } from './commands.js';
import { gainAt, lanePosition } from './player.js';

/** What the render normalizes to. Stated, not measured here. */
const TARGET_LUFS = -14;

export interface AudioProps {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
}

export function Audio({ plan, frame, busy, onApply }: AudioProps) {
  const here = gainAt(plan, frame);
  const atThisFrame = plan.gain.some((point) => point.frame === frame);

  return (
    <div className="flex flex-col gap-4 p-4 text-sm">
      <section>
        <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Gain curve</p>
        <div className="relative h-20 rounded-lg bg-[var(--cm-surface-2)]">
          {/* Unity, so an offset reads as a distance from something. */}
          <span className="absolute inset-x-0 top-1/2 h-px bg-[var(--cm-ink-3)]/40" />
          {plan.gain.map((point) => (
            <span
              key={point.frame}
              title={`${point.gainDb.toFixed(1)} dB at frame ${point.frame}`}
              className="absolute size-2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-[var(--cm-accent)]"
              style={{
                left: `${lanePosition(plan, point.frame)}%`,
                top: `${50 - clamp(point.gainDb, -12, 12) * 4}%`,
              }}
            />
          ))}
          <span
            className="absolute inset-y-0 w-px bg-[var(--cm-accent)]"
            style={{ left: `${lanePosition(plan, frame)}%` }}
          />
        </div>
      </section>

      <section>
        <p className="mb-2 text-xs text-[var(--cm-ink-2)]">At this frame</p>
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() => onApply(setGain(ticksAt(plan, frame), round(here - 1)))}
          >
            −1 dB
          </Button>
          <span className="w-16 text-center font-mono text-xs text-[var(--cm-ink-1)]">
            {here.toFixed(1)} dB
          </span>
          <Button
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() => onApply(setGain(ticksAt(plan, frame), round(here + 1)))}
          >
            +1 dB
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={busy || !atThisFrame}
            onClick={() => onApply(removeGainPoint(ticksAt(plan, frame)))}
          >
            Remove point
          </Button>
        </div>
      </section>

      <section>
        <p className="mb-2 text-xs text-[var(--cm-ink-2)]">Loudness</p>
        <dl className="space-y-1 text-xs">
          <Row label="Delivery target" value={`${TARGET_LUFS.toFixed(1)} LUFS`} />
          <Row label="Curve offset here" value={`${here >= 0 ? '+' : ''}${here.toFixed(1)} dB`} />
        </dl>
        <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
          This is the curve, not a measurement. The export runs a two-pass loudness normalization to
          the target above; what an editor changes here is the shape going into it.
        </p>
      </section>
    </div>
  );
}

function Row({ label, value }: { readonly label: string; readonly value: string }) {
  return (
    <div className="flex justify-between">
      <dt className="text-[var(--cm-ink-2)]">{label}</dt>
      <dd className="font-mono text-[var(--cm-ink-1)]">{value}</dd>
    </div>
  );
}

function clamp(value: number, low: number, high: number): number {
  return Math.max(low, Math.min(high, value));
}

/** Decibels to one place, so a command carries a value a person chose. */
function round(value: number): number {
  return Math.round(value * 10) / 10;
}
