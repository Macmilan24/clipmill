import { useEffect, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import type { EditCommandJson, PreviewCue, PreviewPlan } from '../daemon/client.js';
import { setCueTiming } from './commands.js';

const TICKS = 90_000;

export function CueTiming({
  plan,
  cue,
  busy,
  onApply,
}: {
  readonly plan: PreviewPlan;
  readonly cue: PreviewCue;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const [start, setStart] = useState('');
  const [end, setEnd] = useState('');
  useEffect(() => {
    setStart(cue.startTicks === undefined ? '' : String(cue.startTicks / TICKS));
    setEnd(cue.endTicks === undefined ? '' : String(cue.endTicks / TICKS));
  }, [cue.cueId, cue.startTicks, cue.endTicks, plan.presentation]);

  if (cue.startTicks === undefined || cue.endTicks === undefined || cue.endTicks <= cue.startTicks)
    return null;
  const from = Math.round(Number(start) * TICKS);
  const to = Math.round(Number(end) * TICKS);
  const duration = plan.segments.reduce(
    (sum, segment) => sum + segment.outTicks - segment.inTicks,
    0,
  );
  const index = plan.cues.findIndex((candidate) => candidate.cueId === cue.cueId);
  const before = plan.cues[index - 1]?.endTicks ?? 0;
  const after = plan.cues[index + 1]?.startTicks ?? duration;
  const valid =
    start.trim() !== '' &&
    end.trim() !== '' &&
    Number.isSafeInteger(from) &&
    Number.isSafeInteger(to) &&
    from >= 0 &&
    to > from &&
    to <= duration &&
    from >= before &&
    to <= after;
  const changed = from !== cue.startTicks || to !== cue.endTicks;
  const minimum = plan.presentation === 'reading' ? (plan.readingMinDurationTicks ?? 0) : 0;

  return (
    <details
      className="rounded-lg border p-3"
      open={minimum > 0 && cue.endTicks - cue.startTicks < minimum}
    >
      <summary className="cursor-pointer text-xs font-medium">
        Display timing{' '}
        <span className="float-right font-mono text-muted-foreground">
          {((cue.endTicks - cue.startTicks) / TICKS).toFixed(2)}s
        </span>
      </summary>
      <form
        className="mt-3 space-y-2"
        aria-label="Caption display timing"
        onSubmit={(event) => {
          event.preventDefault();
          if (!valid || !changed || busy) return;
          onApply(setCueTiming(cue.cueId, from, to, plan.presentation));
        }}
      >
        <div className="grid grid-cols-2 gap-2">
          <label className="space-y-1 text-xs">
            Start (seconds)
            <Input
              type="number"
              step="any"
              min="0"
              value={start}
              disabled={busy}
              onChange={(event) => setStart(event.target.value)}
            />
          </label>
          <label className="space-y-1 text-xs">
            End (seconds)
            <Input
              type="number"
              step="any"
              min="0"
              value={end}
              disabled={busy}
              onChange={(event) => setEnd(event.target.value)}
            />
          </label>
        </div>
        {minimum > 0 && (
          <p className="text-xs text-muted-foreground">
            Allow at least {(Math.ceil((minimum / TICKS) * 100) / 100).toFixed(2)}s. Extend into an
            available gap, or merge neighbouring captions.
          </p>
        )}
        {!valid && changed && (
          <p role="alert" className="text-xs text-destructive">
            Keep the caption inside the clip and clear of neighbouring captions.
          </p>
        )}
        <Button type="submit" size="sm" variant="outline" disabled={busy || !valid || !changed}>
          Save timing
        </Button>
        <p className="text-[11px] text-muted-foreground">
          Changes this caption track only. Spoken words and audio stay in place.
        </p>
      </form>
    </details>
  );
}
