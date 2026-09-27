/**
 * B-roll: another picture over a moment of the clip while the voice carries
 * on — a still from the person's pictures, or footage from a recording of
 * the project.
 *
 * Cutaways are one list in program order, none overlapping, and every change
 * replaces the list, so undo is the list as it was. A new one goes in at the
 * playhead, into whatever room there is before the next.
 */
import type { EditIr } from '@clipmill/contracts';

import type { EditCommandJson, PreviewCutaway, PreviewPlan } from '../daemon/client.js';
import { programTicks } from './timeline.js';

export type Cutaway = NonNullable<EditIr['video']['cutaways']>[number];
type DocumentAsset = NonNullable<EditIr['assets']>[number];

const SECOND = 90_000;
/** How long a new cutaway covers, room allowing. */
const CUTAWAY_SPAN = (5 * SECOND) / 2;
/** The shortest a cutaway can be, as the document holds it. */
export const SHORTEST_CUTAWAY = SECOND / 5;
/** How much closer a picture that pushes in is by its last frame. */
export const PUSH_IN = 0.08;

export function setCutaways(
  cutaways: readonly Cutaway[],
  assets?: readonly DocumentAsset[],
): EditCommandJson {
  return {
    op: 'set_cutaways',
    cutaways: [...cutaways],
    ...(assets ? { assets: [...assets] } : {}),
  };
}

/** An id no cutaway in the list has. */
export function freshCutawayId(existing: readonly { readonly cutaway_id: string }[]): string {
  const taken = new Set(existing.map((cutaway) => cutaway.cutaway_id));
  let number = existing.length + 1;
  while (taken.has(`cut_${number}`)) number += 1;
  return `cut_${number}`;
}

/** The cutaway covering a frame, if one does. */
export function cutawayAt(
  plan: Pick<PreviewPlan, 'cutaways'>,
  frame: number,
): PreviewCutaway | undefined {
  return (plan.cutaways ?? []).find(
    (cutaway) => frame >= cutaway.firstFrame && frame < cutaway.endFrame,
  );
}

/** The document's form of a cutaway the plan shows. */
export function savedCutaway(cutaway: PreviewCutaway): Cutaway {
  return {
    cutaway_id: cutaway.cutawayId,
    start_ticks: cutaway.startTicks,
    end_ticks: cutaway.endTicks,
    ...(cutaway.fit === 'fit' ? { fit: 'fit' as const } : {}),
    content:
      cutaway.kind === 'footage'
        ? {
            kind: 'footage',
            source_fingerprint: cutaway.sourceFingerprint ?? '',
            in_ticks: cutaway.inTicks,
          }
        : {
            kind: 'picture',
            asset: cutaway.asset ?? '',
            ...(cutaway.pushIn ? { push_in: true } : {}),
          },
  };
}

/** The list the plan shows, as the document holds it. */
export function savedCutaways(plan: Pick<PreviewPlan, 'cutaways'>): Cutaway[] {
  return (plan.cutaways ?? []).map(savedCutaway);
}

/**
 * The room a cutaway starting at `startTicks` has: up to the next one, the
 * clip's end, or `want`, whichever comes first. `null` where another already
 * covers the moment or there is less than the shortest a cutaway can be.
 */
export function roomAt(
  list: readonly Cutaway[],
  startTicks: number,
  programEnd: number,
  want: number = CUTAWAY_SPAN,
  ignoring?: string,
): { readonly start: number; readonly end: number } | null {
  const others = list.filter((cutaway) => cutaway.cutaway_id !== ignoring);
  const start = Math.max(0, Math.round(startTicks));
  if (others.some((cutaway) => start >= cutaway.start_ticks && start < cutaway.end_ticks)) {
    return null;
  }
  const next = others
    .map((cutaway) => cutaway.start_ticks)
    .filter((ticks) => ticks > start)
    .reduce((first, ticks) => Math.min(first, ticks), programEnd);
  const end = Math.min(start + want, next, programEnd);
  return end - start >= SHORTEST_CUTAWAY ? { start, end } : null;
}

/** The list with one more, in its place in program order. */
export function withCutaway(list: readonly Cutaway[], cutaway: Cutaway): Cutaway[] {
  return [...list.filter((item) => item.cutaway_id !== cutaway.cutaway_id), cutaway].toSorted(
    (a, b) => a.start_ticks - b.start_ticks,
  );
}

/** A picture over the moment at the playhead, moving slowly closer. */
export function pictureCutaway(
  plan: PreviewPlan,
  list: readonly Cutaway[],
  asset: string,
  startTicks: number,
): Cutaway | null {
  const room = roomAt(list, startTicks, programTicks(plan));
  if (!room) return null;
  return {
    cutaway_id: freshCutawayId(list),
    start_ticks: room.start,
    end_ticks: room.end,
    content: { kind: 'picture', asset, push_in: true },
  };
}

/** Footage from a recording, from where `inTicks` says, over the moment at the playhead. */
export function footageCutaway(
  plan: PreviewPlan,
  list: readonly Cutaway[],
  sourceFingerprint: string,
  inTicks: number,
  startTicks: number,
): Cutaway | null {
  const room = roomAt(list, startTicks, programTicks(plan));
  if (!room) return null;
  return {
    cutaway_id: freshCutawayId(list),
    start_ticks: room.start,
    end_ticks: room.end,
    content: { kind: 'footage', source_fingerprint: sourceFingerprint, in_ticks: inTicks },
  };
}

/**
 * How much larger a picture that pushes in is drawn on a frame: from its
 * size at its first frame to eight per cent closer at its last, as the
 * render's zoom moves it.
 */
export function pushScale(
  cutaway: Pick<PreviewCutaway, 'firstFrame' | 'endFrame' | 'pushIn' | 'kind'>,
  frame: number,
): number {
  if (cutaway.kind !== 'picture' || !cutaway.pushIn) return 1;
  const span = Math.max(1, cutaway.endFrame - cutaway.firstFrame - 1);
  const into = Math.min(Math.max(frame - cutaway.firstFrame, 0), span);
  return 1 + (PUSH_IN * into) / span;
}

/** Where in its proxy footage is at a program frame, in seconds. */
export function footageSecondsAt(
  plan: Pick<PreviewPlan, 'rateNum' | 'rateDen' | 'proxies'>,
  cutaway: Pick<PreviewCutaway, 'firstFrame' | 'inTicks' | 'sourceFingerprint' | 'startTicks'>,
  frame: number,
): number | null {
  const proxy = plan.proxies.find((item) => item.sourceFingerprint === cutaway.sourceFingerprint);
  if (!proxy) return null;
  // Program time from the cutaway's start, on the program's frame grid.
  const into = Math.max(
    0,
    (frame * plan.rateDen * SECOND) / Math.max(1, plan.rateNum) - cutaway.startTicks,
  );
  return (cutaway.inTicks + into - proxy.coverageStartTicks) / SECOND;
}

/** How long a recording's proxy runs, in ticks, when the plan lists one. */
export function recordingTicks(
  plan: Pick<PreviewPlan, 'proxies'>,
  sourceFingerprint: string | null | undefined,
): number | null {
  const proxy = plan.proxies.find((item) => item.sourceFingerprint === sourceFingerprint);
  return proxy ? proxy.coverageEndTicks - proxy.coverageStartTicks : null;
}
