/**
 * Punch-ins: moments a followed shot moves in closer, then back.
 *
 * A section keeps its punches beside its crop path, which they leave as it
 * is: the render takes each crop tighter about its own centre for a punch's
 * span. Automatic punch-ins land on every other sentence — the clip opens
 * wide, the next thought comes in close — so the cut in feels like emphasis
 * rather than a camera looking for something.
 */
import type { EditIr } from '@clipmill/contracts';

import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import { batch } from './commands.js';
import type { ProgramWord } from './transcript.js';

type Segment = NonNullable<EditIr['video']['segments']>[number];
export type Punch = NonNullable<Segment['layout']['punches']>[number];

const SECOND = 90_000;
/** How long a move in or out takes, as the document states it. */
const MOVE = 6_000;
/** The shortest a punch may be, and the least gap between two. */
const SPAN = 2 * MOVE;
/** A sentence shorter than this is not worth moving in for. */
const SHORTEST = 1.5 * SECOND;
/** A punch holds no longer than this, however long the sentence. */
const LONGEST = 5 * SECOND;
/** A pause this long ends a sentence as surely as a full stop. */
const PAUSE = 0.6 * SECOND;

export const PUNCH_ZOOMS = { min: 105, max: 200, default: 125 } as const;

export function setPunches(segmentId: string, punches: readonly Punch[]): EditCommandJson {
  return { op: 'set_punches', segment_id: segmentId, punches: [...punches] };
}

/** The program's sentences, as spans of program ticks. */
export function sentences(words: readonly ProgramWord[]): { start: number; end: number }[] {
  const spans: { start: number; end: number }[] = [];
  let start: number | null = null;
  words.forEach((word, index) => {
    start ??= word.startTicks;
    const next = words[index + 1];
    const ends =
      /[.?!…]["”’)]*$/.test(word.text) || !next || next.startTicks - word.endTicks >= PAUSE;
    if (ends) {
      spans.push({ start, end: word.endTicks });
      start = null;
    }
  });
  return spans;
}

/**
 * Punches for a section, one for each of the sentences given that plays in
 * it: within the section, long enough to read as a move, no longer than a
 * held thought, and apart from the one before as the document requires.
 */
export function punchesFor(
  part: { readonly programStartTicks: number; readonly inTicks: number; readonly outTicks: number },
  chosen: readonly { start: number; end: number }[],
  zoom: number,
): Punch[] {
  const duration = part.outTicks - part.inTicks;
  const punches: Punch[] = [];
  let lastEnd = -Infinity;
  for (const span of chosen) {
    if (span.end - span.start < SHORTEST) continue;
    const start = Math.max(0, Math.round(span.start - part.programStartTicks));
    const end = Math.min(
      duration,
      Math.round(Math.min(span.end, span.start + LONGEST) - part.programStartTicks),
    );
    if (end - start < SPAN) continue;
    if (start !== lastEnd && start - lastEnd < SPAN) continue;
    punches.push({ start_ticks: start, end_ticks: end, zoom });
    lastEnd = end;
  }
  return punches;
}

/** Whether a section can punch in: it follows someone. */
function punchable(segment: Segment): boolean {
  return segment.layout.state === 'speaker_fill' && (segment.layout.crop_path?.length ?? 0) > 0;
}

/**
 * Punch-ins across the clip, as one step: every followed section, on every
 * other sentence of the clip — the second, the fourth — counted across the
 * whole clip so the rhythm carries over cuts. Null when nothing in the clip
 * follows anyone.
 */
export function autoPunches(
  plan: PreviewPlan,
  document: Pick<EditIr, 'video'>,
  words: readonly ProgramWord[],
  zoom: number = PUNCH_ZOOMS.default,
): EditCommandJson | null {
  const alternate = sentences(words).filter((_, index) => index % 2 === 1);
  const commands: EditCommandJson[] = [];
  for (const segment of document.video.segments ?? []) {
    if (!punchable(segment)) continue;
    const part = plan.segments.find((item) => item.segmentId === segment.segment_id);
    if (!part) continue;
    const from = part.programStartTicks;
    const to = from + part.outTicks - part.inTicks;
    const chosen = alternate.filter((span) => span.start >= from && span.start < to);
    commands.push(setPunches(segment.segment_id, punchesFor(part, chosen, zoom)));
  }
  return commands.length > 0 ? batch(commands) : null;
}

/** Every punch taken away, as one step; null when there are none. */
export function clearPunches(document: Pick<EditIr, 'video'>): EditCommandJson | null {
  const commands = (document.video.segments ?? [])
    .filter((segment) => (segment.layout.punches?.length ?? 0) > 0)
    .map((segment) => setPunches(segment.segment_id, []));
  return commands.length > 0 ? batch(commands) : null;
}

/** Every punch set to one closeness, as one step; null when there are none. */
export function rezoomPunches(
  document: Pick<EditIr, 'video'>,
  zoom: number,
): EditCommandJson | null {
  const commands = (document.video.segments ?? [])
    .filter((segment) => (segment.layout.punches?.length ?? 0) > 0)
    .map((segment) =>
      setPunches(
        segment.segment_id,
        (segment.layout.punches ?? []).map((punch) => ({ ...punch, zoom })),
      ),
    );
  return commands.length > 0 ? batch(commands) : null;
}

/** How many punches the clip has, and how close the first one goes. */
export function punchSummary(document: Pick<EditIr, 'video'> | null): {
  readonly count: number;
  readonly zoom: number;
} {
  const all = (document?.video.segments ?? []).flatMap((segment) => segment.layout.punches ?? []);
  return { count: all.length, zoom: all[0]?.zoom ?? PUNCH_ZOOMS.default };
}

/**
 * A punch at the playhead in the section it is in: two seconds, or to the
 * section's end, apart from any punch already there. Null where none fits.
 */
export function punchHere(
  segment: Segment,
  localTicks: number,
  zoom: number,
  duration: number,
): EditCommandJson | null {
  if (!punchable(segment)) return null;
  const start = Math.max(0, Math.round(localTicks));
  const end = Math.min(duration, start + 2 * SECOND);
  if (end - start < SPAN) return null;
  const existing = segment.layout.punches ?? [];
  const clear = existing.every(
    (punch) =>
      punch.end_ticks === start ||
      punch.start_ticks === end ||
      punch.end_ticks + SPAN <= start ||
      end + SPAN <= punch.start_ticks,
  );
  if (!clear) return null;
  const punches = [...existing, { start_ticks: start, end_ticks: end, zoom }].toSorted(
    (a, b) => a.start_ticks - b.start_ticks,
  );
  return setPunches(segment.segment_id, punches);
}
