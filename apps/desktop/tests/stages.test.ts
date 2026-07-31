/**
 * The pipeline list a screen renders, held to the contract that produced it.
 *
 * The ten stages are named and ordered here in TypeScript, which is a copy of a
 * decision the daemon owns. A copy drifts silently: a stage added to the DAG
 * would simply never appear on screen, and one removed would sit at "waiting"
 * forever, and neither shows up as a failure anywhere.
 *
 * The published analysis-manifest schema is the shared statement of what an
 * analysis produces — the daemon validates against it, so it cannot drift from
 * the daemon without the daemon noticing. Reading it here makes this list
 * derived from the same authority rather than merely agreeing with it today.
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import { ANALYSIS_STAGES, MANIFEST_KIND, stageFor } from '../src/pipeline/stages.js';

const schema = JSON.parse(
  readFileSync(
    resolve(process.cwd(), '../../contracts/schemas/clipmill.analysis.manifest.v1.json'),
    'utf8',
  ),
) as { $defs: { stage: { properties: { kind: { enum: string[] } } } } };

const PUBLISHED = schema.$defs.stage.properties.kind.enum;

describe('the stages a run is shown as', () => {
  it('names exactly the stages an analysis publishes, in the order it runs them', () => {
    expect(ANALYSIS_STAGES.map((stage) => stage.kind)).toEqual(PUBLISHED);
  });

  it('gives every stage a label and a line of detail', () => {
    for (const stage of ANALYSIS_STAGES) {
      expect(stage.label.length).toBeGreaterThan(0);
      expect(stage.detail.length).toBeGreaterThan(0);
    }
  });

  /**
   * Ingest's derivatives are the one place a kind maps to a row it does not
   * name. Each has to resolve, or the row goes blank for the minutes that
   * matter most — the task actually called `media.ingest_manifest.v1` is the
   * last thing to run and does almost nothing.
   */
  it('resolves every ingest derivative to the ingest row', () => {
    const ingest = ANALYSIS_STAGES.find((stage) => stage.kind === 'media.ingest_manifest.v1');
    expect(ingest?.covers?.length).toBe(8);
    for (const kind of ingest?.covers ?? []) {
      expect(stageFor(kind)).toBe(ingest);
    }
  });

  it('gives the fan-in no row of its own', () => {
    expect(PUBLISHED).not.toContain(MANIFEST_KIND);
    expect(stageFor(MANIFEST_KIND)).toBeUndefined();
  });

  it('claims no stage the contract does not know about', () => {
    expect(stageFor('speech.diarization.v1')).toBeUndefined();
    expect(stageFor('')).toBeUndefined();
  });
});
