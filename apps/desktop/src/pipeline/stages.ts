/**
 * The stages an analyze job runs, named for a reader.
 *
 * The daemon's DAG is wider than this list: ingest alone fans out into eight
 * derivatives — proxy, two audio rates, loudness, reference index, filmstrip,
 * peaks, frames — and the run ends with a fan-in that publishes the manifest
 * naming everything. Showing all of that would be showing the plumbing.
 *
 * So this is the reading of the DAG, not the DAG. Ingest is one line because a
 * user who asked for one thing to happen should see one thing happening; the
 * fan-in is absent because a manifest over work already reported is bookkeeping.
 * The order is the order the daemon declares in `analysis.rs`, and the keys are
 * artifact kinds, so a task is matched by what it publishes rather than by what
 * the daemon calls the work.
 */
export interface AnalysisStage {
  /** The artifact kind this stage publishes. */
  readonly kind: string;
  readonly label: string;
  /** One line on what the stage actually does. */
  readonly detail: string;
  /**
   * Kinds that roll up into this stage rather than standing on their own.
   *
   * Only ingest has any. Its eight derivatives run as eight tasks and finish at
   * eight different times, and the row has to say "ingest is happening" while
   * any of them is — otherwise the screen would go blank for the minutes that
   * matter most, because the task named `media.ingest_manifest.v1` is the last
   * thing to run and does almost nothing.
   */
  readonly covers?: readonly string[];
}

export const ANALYSIS_STAGES: readonly AnalysisStage[] = [
  {
    kind: 'evidence.source_map.v1',
    label: 'Inspect source',
    detail: 'Container, streams, and timing read from the file',
  },
  {
    kind: 'media.ingest_manifest.v1',
    label: 'Ingest',
    detail: 'Proxy, audio, loudness, filmstrip, and frames',
    covers: [
      'media.proxy.v1',
      'media.audio_16k.v1',
      'media.audio_48k.v1',
      'media.loudness_envelope.v1',
      'media.reference_index.v1',
      'media.filmstrip.v1',
      'media.audio_peaks.v1',
      'media.frames.v1',
    ],
  },
  {
    kind: 'speech.vad.v1',
    label: 'Find speech',
    detail: 'Where in the recording anyone is talking',
  },
  {
    kind: 'speech.asr.v1',
    label: 'Recognise speech',
    detail: 'Words, with the model that heard them',
  },
  {
    kind: 'speech.alignment.v1',
    label: 'Align words',
    detail: 'Each word placed against the audio',
  },
  {
    kind: 'speech.transcript.v1',
    label: 'Assemble transcript',
    detail: 'The three speech passes fused into one document',
  },
  {
    kind: 'evidence.shots.v1',
    label: 'Detect shots',
    detail: 'Cuts, so no clip is allowed to straddle one',
  },
  {
    kind: 'index.transcript.v1',
    label: 'Index transcript',
    detail: 'Structure over the words, for search and selection',
  },
  {
    kind: 'discovery.candidates.v1',
    label: 'Propose candidates',
    detail: 'Moments nominated, each with evidence',
  },
  {
    kind: 'ranking.set.v1',
    label: 'Rank candidates',
    detail: 'Scored and ordered against the rubric',
  },
];

/** The fan-in. It publishes over work already reported, so nothing shows it. */
export const MANIFEST_KIND = 'analysis.manifest.v1';

const BY_KIND = new Map(
  ANALYSIS_STAGES.flatMap((stage) =>
    [stage.kind, ...(stage.covers ?? [])].map((kind) => [kind, stage] as const),
  ),
);

/**
 * The stage a task belongs to, by the kind it publishes.
 *
 * Nothing for the fan-in manifest, and nothing for a kind no analyze job
 * produces — both of which a caller should treat as "not a row", not as an
 * error.
 */
export function stageFor(outputKind: string): AnalysisStage | undefined {
  return BY_KIND.get(outputKind);
}
