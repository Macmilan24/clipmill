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

import type { ClipDecision, ClipDecisionRecord, EditDocSummary } from '../daemon/client.js';

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

/**
 * A sentence the evidence index holds, and where in the recording it was said.
 *
 * The position travels with the text because a quote a person can jump to is
 * evidence and a quote they cannot is a caption. `atTicks` is null only for a
 * topic, which is a span of keywords rather than a thing anybody said.
 */
export interface Quote {
  readonly text: string;
  readonly atTicks: number | null;
}

export interface AxisReading {
  readonly axis: Axis;
  readonly label: string;
  /** Absent when nothing measured this axis. Never zero standing in for it. */
  readonly value: number | null;
  readonly weight: number | null;
  /** Why nothing measured it, when nothing did. */
  readonly unavailableReason: string | null;
  /** The sentences it was read from, resolved to text and position. */
  readonly evidence: readonly Quote[];
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
  readonly review?:
    | {
        readonly status: string;
        readonly reasons: readonly string[];
        readonly route: string;
        readonly summary?: string;
      }
    | undefined;
  readonly band: string;
  readonly bandLabel: string;
  readonly warnings: readonly string[];
  readonly startTicks: number;
  readonly endTicks: number;
  readonly durationSeconds: number;
  /** The proposal's concise title, falling back to the clip's own opening sentence. */
  readonly headline: string;
  readonly axes: readonly AxisReading[];
  readonly penalties: readonly { readonly reason: string; readonly value: number }[];
  readonly boundary: BoundaryReading | null;
  readonly decision: ClipDecision | null;
  /**
   * The edit document this clip has, when it has one — the newest, if it has
   * several. Null means no edit exists yet, which is a different fact from the
   * decision: a clip can be approved and then have its document creation fail,
   * and a board that inferred the document from the decision would send the
   * editor to open nothing.
   */
  readonly docId: string | null;
  /** The run that document was cut from, when the store recorded one. */
  readonly docJobId: string | null;
  /** Lattice edges, for the boundary strip. */
  readonly latticeStarts: readonly number[];
  readonly latticeEnds: readonly number[];
  /**
   * Whether the ranker put this clip in its selected set.
   *
   * The cohort is everything that was scored; `selected` is the diverse subset
   * the ranker actually recommends. A board that showed the cohort without
   * saying which rows the ranker stood behind would be hiding its one opinion.
   */
  readonly recommended: boolean;
  /** Which proposer nominated it, by the name it publishes under. */
  readonly proposer: string | null;
  readonly clusterId: string | null;
  /** What the nomination opens with and pays off with, where the proposer said. */
  readonly hook: Quote | null;
  readonly payoff: Quote | null;
  /** The ranker recorded a warning or a penalty against it. */
  readonly flagged: boolean;
}

/** Sentences, utterances and topics by kind and position, as quotes. */
function evidenceQuotes(index: IndexTranscript | null): Map<string, Quote> {
  const found = new Map<string, Quote>();
  if (!index) {
    return found;
  }
  for (const sentence of index.sentences ?? []) {
    found.set(`sentence:${sentence.index}`, {
      text: sentence.text,
      atTicks: sentence.start_ticks,
    });
  }
  for (const utterance of index.utterances ?? []) {
    found.set(`utterance:${utterance.index}`, {
      text: utterance.text,
      atTicks: utterance.start_ticks,
    });
  }
  for (const topic of index.topics ?? []) {
    const terms = (topic.keywords ?? []).map((keyword) => keyword.term).join(', ');
    if (terms) {
      found.set(`topic:${topic.index}`, { text: terms, atTicks: null });
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
  documents: readonly EditDocSummary[] = [],
): readonly ClipRow[] {
  const quotes = evidenceQuotes(index);
  const quote = (reference: { kind: string; index: number } | null | undefined): Quote | null =>
    reference ? (quotes.get(`${reference.kind}:${reference.index}`) ?? null) : null;
  const byId = new Map(candidates.candidates.map((candidate) => [candidate.id, candidate]));
  const recommended = new Set(ranking.selected);
  const decided = new Map(
    decisions
      .filter((record) => record.decision !== 'unspecified')
      .map((record) => [record.candidateId, record.decision as ClipDecision]),
  );
  const edited = newestDocumentPerCandidate(documents);

  return [...ranking.cohort, ...(ranking.declined ?? [])]
    .toSorted((left, right) => left.rank - right.rank)
    .map((ranked) => {
      const candidate = byId.get(ranked.candidate_id);
      const chosen = ranked.boundary.chosen;
      const factors = new Map(ranked.factors.map((factor) => [factor.name, factor]));
      return {
        candidateId: ranked.candidate_id,
        rank: ranked.rank,
        displayScore: ranked.display_score,
        review: ranked.review,
        band: ranked.review
          ? ranked.review.status === 'accepted'
            ? 'strong'
            : ranked.review.status === 'rejected'
              ? 'declined'
              : 'needs_review'
          : ranked.uncertainty.band,
        bandLabel: ranked.review
          ? ranked.review.status === 'accepted'
            ? 'Ready to review'
            : ranked.review.status === 'rejected'
              ? 'Declined by editorial review'
              : 'Needs review'
          : (BAND_LABELS[ranked.uncertainty.band] ?? ranked.uncertainty.band),
        warnings: [...(ranked.review?.reasons ?? []), ...(ranked.uncertainty.warnings ?? [])],
        startTicks: chosen.start_ticks,
        endTicks: chosen.end_ticks,
        durationSeconds: (chosen.end_ticks - chosen.start_ticks) / TICKS_PER_SECOND,
        headline: ranked.title?.trim() || headlineFor(index, chosen.start_ticks, chosen.end_ticks),
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
              .map((reference) => quote(reference))
              .filter((found): found is Quote => found !== null && found.text.length > 0),
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
        docId: edited.get(ranked.candidate_id)?.docId ?? null,
        docJobId: edited.get(ranked.candidate_id)?.jobId || null,
        latticeStarts: candidate?.boundary_lattice.starts ?? [],
        latticeEnds: candidate?.boundary_lattice.ends ?? [],
        recommended: ranked.review?.status !== 'rejected' && recommended.has(ranked.candidate_id),
        proposer: candidate?.proposer?.name ?? null,
        clusterId: candidate?.cluster_id ?? null,
        hook: quote(candidate?.roles?.hook),
        payoff: quote(candidate?.roles?.payoff),
        flagged:
          (ranked.uncertainty.warnings ?? []).length > 0 || (ranked.penalties ?? []).length > 0,
      } satisfies ClipRow;
    });
}

/**
 * The newest document each candidate has.
 *
 * Newest by creation, the same rule the daemon reopens by: when a clip has an
 * approval and a variation taken afterwards, the variation is what somebody
 * was last working on. The caller has already scoped the list to one source,
 * so the candidate id is the whole key.
 */
export function newestDocumentPerCandidate(
  documents: readonly EditDocSummary[],
): ReadonlyMap<string, EditDocSummary> {
  const newest = new Map<string, EditDocSummary>();
  for (const document of documents) {
    if (document.candidateId === '') {
      continue;
    }
    const held = newest.get(document.candidateId);
    if (
      !held ||
      document.createdUnixMillis > held.createdUnixMillis ||
      (document.createdUnixMillis === held.createdUnixMillis && document.docId > held.docId)
    ) {
      newest.set(document.candidateId, document);
    }
  }
  return newest;
}

/** What a board shows above the rows: counts, not adjectives. */
export interface Summary {
  readonly selected: number;
  readonly cohort: number;
  readonly requested: number;
  /** Why fewer clips came back than were asked for. Never padded away. */
  readonly shortfall: readonly string[];
  /** Missing analysis is independent of whether the requested clip count was met. */
  readonly warnings?: readonly string[];
  readonly filtered: number;
  readonly declined?: number;
  readonly contentProfile?: 'interview' | 'scripted';
}

export function summarize(ranking: RankingSet): Summary {
  const coverage = ranking.editorial;
  const warnings: string[] = [];
  if (coverage) {
    if (coverage.failed_windows.length > 0) {
      warnings.push(
        `${coverage.failed_windows.length} of ${coverage.window_count} windows could not be assessed.`,
      );
    }
    if (coverage.failed_reviews > 0) {
      warnings.push(
        `${coverage.failed_reviews} ${coverage.failed_reviews === 1 ? 'candidate' : 'candidates'} could not be reviewed.`,
      );
    }
    if (coverage.failed_visual_checks > 0) {
      warnings.push(
        `Visual checks were unavailable for ${coverage.failed_visual_checks} ${coverage.failed_visual_checks === 1 ? 'candidate' : 'candidates'}.`,
      );
    }
  }
  return {
    selected: ranking.selected.length,
    declined: ranking.declined?.length ?? 0,
    contentProfile: ranking.content_profile ?? 'interview',
    cohort: ranking.cohort.length,
    requested: ranking.requested.count,
    shortfall: (ranking.shortfall ?? []).map(
      (reason) => reason.detail ?? `${reason.count} ${reason.reason.replaceAll('_', ' ')}`,
    ),
    warnings,
    filtered: (ranking.filtered ?? []).length,
  };
}

/** Which rows a filter leaves. Client-side, because the answer is already here. */
export interface Filters {
  readonly band: string | 'any';
  /**
   * `recommended` and `flagged` are states the ranker assigns, not decisions a
   * person made, but they are what an editor filters by first — "show me what
   * it stands behind", "show me what it warned about" — so they live beside the
   * decisions rather than in a second control.
   */
  readonly decision: ClipDecision | 'any' | 'undecided' | 'recommended' | 'flagged';
  readonly minimumScore: number;
  /**
   * A free-text query over what a person can actually read on a row.
   *
   * Optional because a filter set that names no query is a filter set that does
   * not filter by one, and every caller that predates search should keep
   * meaning exactly what it meant.
   */
  readonly query?: string;
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
    if (filters.decision === 'recommended' && !row.recommended) {
      return false;
    }
    if (filters.decision === 'flagged' && !row.flagged) {
      return false;
    }
    if (
      filters.decision !== 'any' &&
      filters.decision !== 'undecided' &&
      filters.decision !== 'recommended' &&
      filters.decision !== 'flagged' &&
      row.decision !== filters.decision
    ) {
      return false;
    }
    if (!matchesQuery(row, filters.query)) {
      return false;
    }
    return row.review !== undefined || row.displayScore >= filters.minimumScore;
  });
}

/**
 * Whether a row answers a query, over the two fields a person reads.
 *
 * The headline and the timecode, and nothing else. Searching the evidence would
 * find rows whose visible text does not contain the term, which reads as the
 * filter being broken rather than as being thorough.
 */
function matchesQuery(row: ClipRow, query: string | undefined): boolean {
  const needle = (query ?? '').trim().toLowerCase();
  if (needle === '') {
    return true;
  }
  return (
    row.headline.toLowerCase().includes(needle) ||
    `${clock(row.startTicks)}-${clock(row.endTicks)}`.includes(needle)
  );
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

/** How a board may be ordered. Rank is the ranking's own answer. */
export type SortKey = 'rank' | 'score' | 'longest' | 'shortest' | 'earliest';

export const SORT_LABELS: Readonly<Record<SortKey, string>> = {
  rank: 'Rank',
  score: 'Score, high to low',
  longest: 'Longest first',
  shortest: 'Shortest first',
  earliest: 'Position in recording',
};

/**
 * The rows in a chosen order, as a new array.
 *
 * Sorting never drops or merges, so the count under the table is the count in
 * it whatever the order. `toSorted` rather than `sort` because the rows belong
 * to the snapshot and a board that reordered them in place would change what
 * every other reader of that snapshot sees.
 */
export function sortRows(rows: readonly ClipRow[], key: SortKey): readonly ClipRow[] {
  switch (key) {
    case 'score':
      return rows.toSorted((left, right) =>
        left.review || right.review
          ? left.rank - right.rank
          : right.displayScore - left.displayScore,
      );
    case 'longest':
      return rows.toSorted((left, right) => right.durationSeconds - left.durationSeconds);
    case 'shortest':
      return rows.toSorted((left, right) => left.durationSeconds - right.durationSeconds);
    case 'earliest':
      return rows.toSorted((left, right) => left.startTicks - right.startTicks);
    case 'rank':
    default:
      return rows.toSorted((left, right) => left.rank - right.rank);
  }
}

/** What the filter chips count, so a chip never claims a number nobody has. */
export interface Tallies {
  readonly all: number;
  readonly strong: number;
  readonly promising: number;
  readonly needsReview: number;
  readonly undecided: number;
  readonly approved: number;
  readonly kept: number;
  readonly rejected: number;
  /** Rows carrying at least one warning or penalty, which is what a dot means. */
  readonly flagged: number;
  /** Rows in the ranker's selected set. */
  readonly recommended: number;
}

export function tally(rows: readonly ClipRow[]): Tallies {
  const count = (predicate: (row: ClipRow) => boolean) => rows.filter(predicate).length;
  return {
    all: rows.length,
    strong: count((row) => row.band === 'strong'),
    promising: count((row) => row.band === 'promising'),
    needsReview: count((row) => row.band === 'needs_review'),
    undecided: count((row) => row.decision === null),
    approved: count((row) => row.decision === 'approved'),
    kept: count((row) => row.decision === 'kept'),
    rejected: count((row) => row.decision === 'rejected'),
    flagged: count((row) => row.flagged),
    recommended: count((row) => row.recommended),
  };
}

/**
 * The axes that most moved a card, with the evidence they were read from.
 *
 * Ordered by weighted contribution rather than by raw value: an axis scoring
 * 0.9 at weight 0.4 moved the total less than one scoring 0.7 at weight 1.4, and
 * "why this ranked here" is a question about the total. Unmeasured axes are
 * never candidates — an axis nobody scored explains nothing.
 */
export function topFactors(row: ClipRow, limit = 3): readonly AxisReading[] {
  return row.axes
    .filter((axis) => axis.value !== null && axis.weight !== null)
    .toSorted(
      (left, right) =>
        (right.value ?? 0) * (right.weight ?? 0) - (left.value ?? 0) * (left.weight ?? 0),
    )
    .slice(0, limit);
}

/** A duration as `m:ss`, for a column that reads as a length not a position. */
export function duration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`;
}
