/**
 * The ranked set, joined to the things that explain it.
 *
 * Three published documents describe one clip between them: the ranking holds
 * the score card, the candidate set holds the lattice the boundary was chosen
 * from, and the evidence index holds the sentences a factor was read from. None
 * of them is useful alone and none of them is the interface's to reinterpret,
 * so everything here is a join rather than a calculation.
 *
 * Pure on purpose. A screen that computed while it rendered would be a screen
 * whose numbers could only be checked by looking at it; these functions are
 * checked by tests that never mount anything.
 */
import type {
  CaptionCues,
  DiscoveryCandidates,
  EditIr,
  IndexTranscript,
  RankingSet,
} from '@clipmill/contracts';

import type { ClipDecision, ClipDecisionRecord } from '../daemon/client.js';

/** Ticks per second, the daemon's timebase throughout. */
export const TICKS_PER_SECOND = 90_000;

/**
 * The eight axes, in the order the book decomposes them. Fixed here rather than
 * read from the document so a card that lost a factor renders a gap where it
 * used to be instead of quietly becoming a shorter list.
 */
export const AXES = [
  'hook',
  'flow',
  'value',
  'prompt_relevance',
  'novelty',
  'evidence',
  'craft',
  'feasibility',
] as const;

export type Axis = (typeof AXES)[number];

/** What each axis is called where a person reads it. */
export const AXIS_LABELS: Readonly<Record<Axis, string>> = {
  hook: 'Hook',
  flow: 'Flow',
  value: 'Value',
  prompt_relevance: 'Prompt fit',
  novelty: 'Novelty',
  evidence: 'Evidence',
  craft: 'Craft',
  feasibility: 'Feasibility',
};

/**
 * How much a card should be trusted, in the three words the book asks for.
 * Never a shading of the number: a user is told, not hinted at.
 */
export const BAND_LABELS: Readonly<Record<string, string>> = {
  strong: 'Strong',
  promising: 'Promising',
  needs_review: 'Needs review',
};

export interface AxisReading {
  readonly axis: Axis;
  readonly label: string;
  /** Absent when nothing measured this axis. Never zero standing in for it. */
  readonly value: number | null;
  readonly weight: number | null;
  /** Why nothing measured it, when nothing did. */
  readonly unavailableReason: string | null;
  /** The sentences it was read from, already resolved to text. */
  readonly evidence: readonly string[];
}

export interface BoundaryReading {
  readonly startTicks: number;
  readonly endTicks: number;
  readonly score: number;
  readonly terms: readonly { readonly name: string; readonly value: number }[];
  /** The runner-up, absent when the lattice offered one legal pair. */
  readonly alternative: { readonly startTicks: number; readonly endTicks: number } | null;
}

export interface ClipRow {
  readonly candidateId: string;
  readonly rank: number;
  readonly displayScore: number;
  readonly band: string;
  readonly bandLabel: string;
  readonly warnings: readonly string[];
  readonly startTicks: number;
  readonly endTicks: number;
  readonly durationSeconds: number;
  /** The clip's own first sentence, which is what a person recognises it by. */
  readonly headline: string;
  readonly axes: readonly AxisReading[];
  readonly penalties: readonly { readonly reason: string; readonly value: number }[];
  readonly boundary: BoundaryReading | null;
  readonly decision: ClipDecision | null;
  /** Lattice edges, for the boundary strip. */
  readonly latticeStarts: readonly number[];
  readonly latticeEnds: readonly number[];
}

/** Sentences and utterances by kind and position, as text. */
function evidenceText(index: IndexTranscript | null): Map<string, string> {
  const found = new Map<string, string>();
  if (!index) {
    return found;
  }
  for (const sentence of index.sentences ?? []) {
    found.set(`sentence:${sentence.index}`, sentence.text);
  }
  for (const utterance of index.utterances ?? []) {
    found.set(`utterance:${utterance.index}`, utterance.text);
  }
  for (const topic of index.topics ?? []) {
    const terms = (topic.keywords ?? []).map((keyword) => keyword.term).join(', ');
    if (terms) {
      found.set(`topic:${topic.index}`, terms);
    }
  }
  return found;
}

/**
 * The first sentence inside a span.
 *
 * A clip is recognised by what it opens with, so the headline is the opening
 * line rather than a summary — which nothing here is entitled to write.
 */
function headlineFor(index: IndexTranscript | null, startTicks: number, endTicks: number): string {
  const sentences = index?.sentences ?? [];
  const inside = sentences.find(
    (sentence) => sentence.start_ticks >= startTicks && sentence.start_ticks < endTicks,
  );
  return inside?.text ?? '';
}

/**
 * Join the three documents into the rows a board renders.
 *
 * Ordered by the ranking's own rank rather than re-sorted here: the cohort's
 * order is a decision the ranking made and an interface that re-made it would
 * be a second ranking nobody can see.
 */
export function clipRows(
  ranking: RankingSet,
  candidates: DiscoveryCandidates,
  index: IndexTranscript | null,
  decisions: readonly ClipDecisionRecord[],
): readonly ClipRow[] {
  const text = evidenceText(index);
  const byId = new Map(candidates.candidates.map((candidate) => [candidate.id, candidate]));
  const decided = new Map(
    decisions
      .filter((record) => record.decision !== 'unspecified')
      .map((record) => [record.candidateId, record.decision as ClipDecision]),
  );

  return [...ranking.cohort]
    .sort((left, right) => left.rank - right.rank)
    .map((ranked) => {
      const candidate = byId.get(ranked.candidate_id);
      const chosen = ranked.boundary.chosen;
      const factors = new Map(ranked.factors.map((factor) => [factor.name, factor]));
      return {
        candidateId: ranked.candidate_id,
        rank: ranked.rank,
        displayScore: ranked.display_score,
        band: ranked.uncertainty.band,
        bandLabel: BAND_LABELS[ranked.uncertainty.band] ?? ranked.uncertainty.band,
        warnings: ranked.uncertainty.warnings ?? [],
        startTicks: chosen.start_ticks,
        endTicks: chosen.end_ticks,
        durationSeconds: (chosen.end_ticks - chosen.start_ticks) / TICKS_PER_SECOND,
        headline: headlineFor(index, chosen.start_ticks, chosen.end_ticks),
        axes: AXES.map((axis) => {
          const factor = factors.get(axis);
          return {
            axis,
            label: AXIS_LABELS[axis],
            value: factor?.available ? (factor.value ?? null) : null,
            weight: factor?.available ? (factor.weight ?? null) : null,
            unavailableReason:
              factor && !factor.available ? (factor.unavailable_reason ?? null) : null,
            evidence: (factor?.evidence ?? [])
              .map((reference) => text.get(`${reference.kind}:${reference.index}`) ?? '')
              .filter((sentence) => sentence.length > 0),
          } satisfies AxisReading;
        }),
        penalties: (ranked.penalties ?? []).map((penalty) => ({
          reason: penalty.reason,
          value: penalty.value,
        })),
        boundary: {
          startTicks: chosen.start_ticks,
          endTicks: chosen.end_ticks,
          score: ranked.boundary.score,
          terms: ranked.boundary.terms.map((term) => ({ name: term.name, value: term.value })),
          alternative: ranked.boundary.alternative
            ? {
                startTicks: ranked.boundary.alternative.interval.start_ticks,
                endTicks: ranked.boundary.alternative.interval.end_ticks,
              }
            : null,
        },
        decision: decided.get(ranked.candidate_id) ?? null,
        latticeStarts: candidate?.boundary_lattice.starts ?? [],
        latticeEnds: candidate?.boundary_lattice.ends ?? [],
      } satisfies ClipRow;
    });
}

/** What a board shows above the rows: counts, not adjectives. */
export interface Summary {
  readonly selected: number;
  readonly cohort: number;
  readonly requested: number;
  /** Why fewer clips came back than were asked for. Never padded away. */
  readonly shortfall: readonly string[];
  readonly filtered: number;
}

export function summarize(ranking: RankingSet): Summary {
  return {
    selected: ranking.selected.length,
    cohort: ranking.cohort.length,
    requested: ranking.requested.count,
    shortfall: (ranking.shortfall ?? []).map(
      (reason) => reason.detail ?? `${reason.count} ${reason.reason.replaceAll('_', ' ')}`,
    ),
    filtered: (ranking.filtered ?? []).length,
  };
}

/** Which rows a filter leaves. Client-side, because the answer is already here. */
export interface Filters {
  readonly band: string | 'any';
  readonly decision: ClipDecision | 'any' | 'undecided';
  readonly minimumScore: number;
}

export const NO_FILTERS: Filters = { band: 'any', decision: 'any', minimumScore: 0 };

export function applyFilters(rows: readonly ClipRow[], filters: Filters): readonly ClipRow[] {
  return rows.filter((row) => {
    if (filters.band !== 'any' && row.band !== filters.band) {
      return false;
    }
    if (filters.decision === 'undecided' && row.decision !== null) {
      return false;
    }
    if (
      filters.decision !== 'any' &&
      filters.decision !== 'undecided' &&
      row.decision !== filters.decision
    ) {
      return false;
    }
    return row.displayScore >= filters.minimumScore;
  });
}

/** A tick position as `m:ss`, which is how a person reads a timeline. */
export function clock(ticks: number): string {
  const total = Math.max(0, Math.round(ticks / TICKS_PER_SECOND));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${String(seconds).padStart(2, '0')}`;
}

/**
 * The caption cues that fall inside a window, in window-relative seconds.
 *
 * Used by the preview overlay. The cues come from the directed document rather
 * than being re-derived, so what a viewer sees over the proxy is what the render
 * will burn in.
 */
/**
 * The burned-in cues of a directed edit document, in window-relative seconds.
 *
 * Read from the document the director produced rather than re-derived, so the
 * preview draws the words the encoder will draw. A document with no kinetic
 * grouping falls back to its reading cues, exactly as the renderer does.
 */
export function overlayCuesFromEdit(
  documentJson: string,
  startTicks: number,
): readonly { readonly from: number; readonly to: number; readonly text: string }[] {
  let document: EditIr;
  try {
    document = JSON.parse(documentJson) as EditIr;
  } catch {
    return [];
  }
  const captions = document.captions;
  const burned = captions?.burn_in?.length ? captions.burn_in : (captions?.cues ?? []);
  return burned.map((cue) => ({
    from: (cue.start_ticks - startTicks) / TICKS_PER_SECOND,
    to: (cue.end_ticks - startTicks) / TICKS_PER_SECOND,
    text: cue.lines.map((line) => line.words.map((word) => word.text).join(' ')).join('\n'),
  }));
}

export function overlayCues(
  document: CaptionCues | null,
  startTicks: number,
): readonly { readonly from: number; readonly to: number; readonly text: string }[] {
  if (!document) {
    return [];
  }
  return document.intents.burn_in.cues.map((cue) => ({
    from: (cue.start_ticks - startTicks) / TICKS_PER_SECOND,
    to: (cue.end_ticks - startTicks) / TICKS_PER_SECOND,
    text: cue.lines
      .map((line) =>
        document.tokens
          .slice(line.first_token, line.first_token + line.token_count)
          .map((token) => token.text)
          .join(' '),
      )
      .join('\n'),
  }));
}
