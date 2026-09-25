/**
 * What a right-click on the timeline offers, by what it landed on.
 *
 * A section, a caption, a framing keyframe and a volume point each carry the
 * edits a person reaches for there — the same edits the tabs and keys make,
 * gathered where the thing is. A right-click on empty lane opens nothing.
 */
import type { EditIr } from '@clipmill/contracts';
import {
  Combine,
  Crop,
  LayoutPanelTop,
  Maximize,
  Play,
  RotateCcw,
  Scissors,
  Split,
  Trash2,
  Volume2,
} from 'lucide-react';
import type { JSX, ReactNode } from 'react';

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '../components/ui/context-menu.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import { clockTenths } from '../inspector/review.js';
import {
  mergeCues,
  removeCropKeyframe,
  removeGainPoint,
  setCropKeyframe,
  setCuePosition,
  setGain,
  setLayout,
  splitCue,
} from './commands.js';
import { freshCueId, ticksOfFrame } from './timeline.js';
import { rippleRange } from './transcript.js';

type SavedCue = NonNullable<EditIr['captions']['cues']>[number];
type Keyframe = NonNullable<EditIr['video']['segments']>[number]['layout']['crop_path'];
type Rect = NonNullable<Keyframe>[number]['rect'];
type Easing = 'linear' | 'ease_in' | 'ease_out' | 'ease_in_out';

/** The thing a right-click landed on. */
export type MenuTarget =
  | { readonly kind: 'section'; readonly segmentId: string }
  | { readonly kind: 'cue'; readonly cueId: string }
  | {
      readonly kind: 'keyframe';
      readonly segmentId: string;
      readonly tTicks: number;
      readonly secondary: boolean;
      readonly rect: Rect;
      readonly easing: Easing;
    }
  | { readonly kind: 'gain'; readonly tTicks: number; readonly db: number };

const EASINGS: readonly (readonly [Easing, string])[] = [
  ['linear', 'Steady'],
  ['ease_in', 'Ease in'],
  ['ease_out', 'Ease out'],
  ['ease_in_out', 'Ease in and out'],
];

export function TimelineMenu({
  plan,
  cues,
  target,
  playhead,
  busy,
  children,
  onApply,
  onSeek,
  onSplit,
  onClose,
}: {
  readonly plan: PreviewPlan;
  /** The saved cues of the list on screen, for their words' own times. */
  readonly cues: readonly SavedCue[];
  readonly target: MenuTarget | null;
  /** The playhead's frame when the menu opened. */
  readonly playhead: number;
  readonly busy: boolean;
  readonly children: ReactNode;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSeek: (frame: number) => void;
  readonly onSplit: (frame: number) => void;
  readonly onClose: () => void;
}): JSX.Element {
  return (
    <ContextMenu
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <ContextMenuTrigger
        asChild
        onContextMenu={(event) => {
          // Decided by what was clicked, not by state: the item's own handler
          // has only just asked for its menu, and the render that carries it
          // has not happened yet.
          const on = event.target instanceof Element ? event.target.closest('[data-menu]') : null;
          if (!on) event.preventDefault();
        }}
      >
        {children}
      </ContextMenuTrigger>
      <ContextMenuContent>
        {target?.kind === 'section' && (
          <SectionItems
            plan={plan}
            segmentId={target.segmentId}
            playhead={playhead}
            busy={busy}
            onApply={onApply}
            onSeek={onSeek}
            onSplit={onSplit}
          />
        )}
        {target?.kind === 'cue' && (
          <CueItems
            plan={plan}
            cues={cues}
            cueId={target.cueId}
            playhead={playhead}
            busy={busy}
            onApply={onApply}
            onSeek={onSeek}
          />
        )}
        {target?.kind === 'keyframe' && (
          <>
            <ContextMenuLabel>
              {target.secondary ? 'Lower speaker' : 'Framing'} keyframe
            </ContextMenuLabel>
            {EASINGS.map(([easing, label]) => (
              <ContextMenuItem
                key={easing}
                disabled={busy || easing === target.easing}
                onSelect={() =>
                  onApply(
                    setCropKeyframe(
                      target.tTicks,
                      target.rect,
                      target.segmentId,
                      target.secondary,
                      easing,
                    ),
                  )
                }
              >
                <Crop aria-hidden="true" />
                {label}
                {easing === target.easing && ' ✓'}
              </ContextMenuItem>
            ))}
            <ContextMenuSeparator />
            <ContextMenuItem
              tone="danger"
              disabled={busy}
              onSelect={() =>
                onApply(removeCropKeyframe(target.tTicks, target.segmentId, target.secondary))
              }
            >
              <Trash2 aria-hidden="true" />
              Remove keyframe
            </ContextMenuItem>
          </>
        )}
        {target?.kind === 'gain' && (
          <>
            <ContextMenuLabel>
              Volume {target.db > 0 ? '+' : ''}
              {target.db.toFixed(1)} dB at {clockTenths(target.tTicks)}
            </ContextMenuLabel>
            <ContextMenuItem
              disabled={busy || target.db === 0}
              onSelect={() => onApply(setGain(target.tTicks, 0))}
            >
              <Volume2 aria-hidden="true" />
              Back to 0 dB
            </ContextMenuItem>
            <ContextMenuItem
              tone="danger"
              disabled={busy}
              onSelect={() => onApply(removeGainPoint(target.tTicks))}
            >
              <Trash2 aria-hidden="true" />
              Remove point
            </ContextMenuItem>
          </>
        )}
      </ContextMenuContent>
    </ContextMenu>
  );
}

function SectionItems({
  plan,
  segmentId,
  playhead,
  busy,
  onApply,
  onSeek,
  onSplit,
}: {
  readonly plan: PreviewPlan;
  readonly segmentId: string;
  readonly playhead: number;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSeek: (frame: number) => void;
  readonly onSplit: (frame: number) => void;
}): JSX.Element | null {
  const index = plan.segments.findIndex((part) => part.segmentId === segmentId);
  const part = plan.segments[index];
  if (!part) return null;
  const from = part.programStartTicks;
  const to = from + part.outTicks - part.inTicks;
  const inside = playhead > part.firstFrame && playhead < part.endFrame;
  const two = part.hasTwoUpPaths || (plan.secondaryCrops?.[part.firstFrame] ?? null) !== null;
  const fit = !two && plan.crops[part.firstFrame] === null;
  const cut = plan.segments.length > 1 ? rippleRange(plan, from, to) : null;
  return (
    <>
      <ContextMenuLabel>
        {plan.segments.length > 1 ? `Section ${index + 1}` : 'The clip'} · {clockTenths(to - from)}
      </ContextMenuLabel>
      <ContextMenuItem onSelect={() => onSeek(part.firstFrame)}>
        <Play aria-hidden="true" />
        Play from its start
      </ContextMenuItem>
      <ContextMenuItem disabled={busy || !inside} onSelect={() => onSplit(playhead)}>
        <Split aria-hidden="true" />
        Split at the playhead
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem disabled={busy || fit} onSelect={() => onApply(setLayout('fit', segmentId))}>
        <Maximize aria-hidden="true" />
        Whole frame
      </ContextMenuItem>
      <ContextMenuItem
        disabled={busy || (!fit && !two)}
        onSelect={() => onApply(setLayout('speaker_fill', segmentId))}
      >
        <Crop aria-hidden="true" />
        Follow speaker
      </ContextMenuItem>
      <ContextMenuItem
        disabled={busy || two || !part.hasTwoUpPaths}
        onSelect={() => onApply(setLayout('two_up', segmentId))}
      >
        <LayoutPanelTop aria-hidden="true" />
        Two speakers
      </ContextMenuItem>
      {cut && (
        <>
          <ContextMenuSeparator />
          <ContextMenuItem tone="danger" disabled={busy} onSelect={() => onApply(cut)}>
            <Scissors aria-hidden="true" />
            Cut this section
          </ContextMenuItem>
        </>
      )}
    </>
  );
}

function CueItems({
  plan,
  cues,
  cueId,
  playhead,
  busy,
  onApply,
  onSeek,
}: {
  readonly plan: PreviewPlan;
  readonly cues: readonly SavedCue[];
  readonly cueId: string;
  readonly playhead: number;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSeek: (frame: number) => void;
}): JSX.Element | null {
  const index = plan.cues.findIndex((cue) => cue.cueId === cueId);
  const cue = plan.cues[index];
  if (!cue) return null;
  const saved = cues.find((item) => item.cue_id === cueId);
  const words = saved?.lines.flatMap((line) => line.words) ?? [];
  const at = ticksOfFrame(plan, playhead);
  // Split before the first word that starts after the playhead.
  const splitAt = words.findIndex((word) => word.start_ticks >= at);
  const next = plan.cues[index + 1];
  const text = cue.lines
    .flat()
    .map((word) => word.text)
    .join(' ');
  return (
    <>
      <ContextMenuLabel>“{text.length > 36 ? `${text.slice(0, 35)}…` : text}”</ContextMenuLabel>
      <ContextMenuItem onSelect={() => onSeek(cue.firstFrame)}>
        <Play aria-hidden="true" />
        Play this caption
      </ContextMenuItem>
      <ContextMenuItem
        disabled={busy || splitAt <= 0}
        onSelect={() =>
          onApply(
            splitCue(
              cueId,
              splitAt,
              freshCueId(
                plan.cues.map((item) => item.cueId),
                cueId,
              ),
              plan.presentation,
            ),
          )
        }
      >
        <Split aria-hidden="true" />
        Split at the playhead
      </ContextMenuItem>
      <ContextMenuItem
        disabled={busy || !next}
        onSelect={() => next && onApply(mergeCues(cueId, next.cueId, plan.presentation))}
      >
        <Combine aria-hidden="true" />
        Merge with the next
      </ContextMenuItem>
      {saved?.position && (
        <ContextMenuItem
          disabled={busy}
          onSelect={() => onApply(setCuePosition(cueId, null, plan.presentation))}
        >
          <RotateCcw aria-hidden="true" />
          Line up with the others
        </ContextMenuItem>
      )}
    </>
  );
}
