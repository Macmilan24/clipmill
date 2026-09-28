/**
 * Display stages for analysis jobs, ordered to match the daemon's `analysis.rs`.
 * Ingest derivatives share one row; manifest publication is omitted. Artifact kinds
 * match tasks to rows independently of task names.
 */
export interface AnalysisStage {
  /** The artifact kind this stage publishes. */
  readonly kind: string;
  readonly label: string;
  /** One line on what the stage actually does. */
  readonly detail: string;
  /**
   * Artifact kinds grouped under this stage. Ingest remains active while any
   * derivative runs; its manifest task executes only after all derivatives finish.
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
    kind: 'speech.speakers.v1',
    label: 'Tell voices apart',
    detail: 'Who speaks when, so the transcript can say',
  },
  {
    kind: 'evidence.shots.v1',
    label: 'Detect shots',
    detail: 'Camera cuts, so framing stays stable within each shot',
  },
  {
    kind: 'vision.face_track.v1',
    label: 'Plan framing',
    detail: 'Find faces for stable single- and two-person compositions',
  },
  {
    kind: 'index.transcript.v1',
    label: 'Index transcript',
    detail: 'Structure over the words, for search and selection',
  },
  {
    kind: 'editorial.windows.v1',
    label: 'Cut windows',
    detail: 'The transcript in overlapping pieces an editorial model reads',
  },
  {
    kind: 'editorial.proposals.v1',
    label: 'Find complete moments',
    detail: 'Find hooks, setup, and payoff from the transcript',
  },
  {
    kind: 'discovery.candidates.v1',
    label: 'Validate candidates',
    detail: 'Check references, boundaries, and duplicate moments',
  },
  {
    kind: 'editorial.judgments.v1',
    label: 'Review meaning',
    detail: 'Check completeness, context, and misleading omissions',
  },
  {
    kind: 'editorial.looks.v1',
    label: 'Check visual references',
    detail: 'Inspect a few frames only when the meaning depends on the picture',
  },
  {
    kind: 'ranking.set.v1',
    label: 'Rank candidates',
    detail: 'Order distinct moments for your review',
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
