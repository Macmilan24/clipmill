import type { EditIr } from '@clipmill/contracts';
import { ChevronDown, ChevronUp } from 'lucide-react';

import { Button } from '../components/ui/button.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import { clock } from '../results/model.js';

export function PlanParts({
  document,
  plan,
  busy,
  onSeek,
  onApply,
}: {
  readonly document: EditIr;
  readonly plan: PreviewPlan;
  readonly busy: boolean;
  readonly onSeek: (frame: number) => void;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const segments = document.video.segments ?? [];
  const parts = segments.filter(
    (seg, at) => seg.part && segments[at - 1]?.part?.part_id !== seg.part.part_id,
  );
  if (!document.plan || parts.length === 0) return null;
  return (
    <section className="space-y-3 p-3 text-xs" aria-label="Plan parts">
      <h3 className="font-semibold">Parts in playback order</h3>
      <ol className="space-y-2">
        {parts.map((first, at) => {
          const part = first.part!;
          const label = `${at + 1}. ${part.role.replaceAll('_', ' ')}`;
          const frame = plan.segments.find((seg) => seg.segmentId === first.segment_id)?.firstFrame;
          return (
            <li
              key={part.part_id}
              className="space-y-1 border-b border-[var(--cm-glass-border)] pb-2"
            >
              <button
                className="text-left font-medium"
                disabled={frame === undefined}
                onClick={() => frame !== undefined && onSeek(frame)}
              >
                {label} · {clock(first.in_ticks)}
              </button>
              {first.join && (
                <p className="text-[var(--cm-text-secondary)]">
                  {first.join.kind} join
                  {first.join.audio_crossfade_ticks
                    ? ` · ${Math.round(first.join.audio_crossfade_ticks / 90)} ms fade`
                    : ''}
                  {first.join.room_tone && first.join.room_tone_range ? ' · source room tone' : ''}
                </p>
              )}
              <div className="flex flex-wrap gap-1">
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy || at === 0}
                  aria-label={`Move ${label} earlier`}
                  onClick={() =>
                    onApply({ op: 'move_part', part_id: part.part_id, to_index: at - 1 })
                  }
                >
                  <ChevronUp />
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy || at === parts.length - 1}
                  aria-label={`Move ${label} later`}
                  onClick={() =>
                    onApply({ op: 'move_part', part_id: part.part_id, to_index: at + 1 })
                  }
                >
                  <ChevronDown />
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={busy || parts.length === 1}
                  aria-label={`Remove ${label}`}
                  onClick={() => onApply({ op: 'remove_part', part_id: part.part_id })}
                >
                  Remove
                </Button>
              </div>
            </li>
          );
        })}
      </ol>
      {(document.plan.removed?.length ?? 0) > 0 && (
        <>
          <h3 className="font-semibold">Left out of the clip</h3>
          <ul className="space-y-2">
            {document.plan.removed!.map((range, index) => (
              <li
                className="flex items-center justify-between gap-2"
                key={`${range.part_id ?? ''}-${range.in_ticks}-${index}`}
              >
                <span>
                  {clock(range.in_ticks)}–{clock(range.out_ticks)} ·{' '}
                  {range.reason.replaceAll('_', ' ')}
                </span>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy}
                  aria-label={`Restore ${clock(range.in_ticks)} to ${clock(range.out_ticks)}`}
                  onClick={() => onApply({ op: 'restore_plan_range', index })}
                >
                  Restore
                </Button>
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}
