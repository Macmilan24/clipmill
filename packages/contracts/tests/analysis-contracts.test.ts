/**
 * The fan-in that closes an analysis, TypeScript leg.
 *
 * This is the document the shell reads to find out what a project has: one read
 * that yields the addresses of every observation, instead of nine reads guessing
 * which of them exist. The Library and the Analysis Progress view both render
 * from it, so what matters here is that a renderer can answer two questions
 * without opening a single artifact — which stages produced something, and for
 * the ones that did not, whether that is a property of the recording or work
 * nobody has done yet.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import type { AnalysisManifest } from '../src/gen/schemas/analysis-manifest.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const fixtures = join(repo, 'contracts', 'fixtures', 'analysis.manifest');
const ajv = new Ajv2020({ allErrors: true });
const validate = ajv.compile(
  JSON.parse(
    readFileSync(join(repo, 'contracts', 'schemas', 'clipmill.analysis.manifest.v1.json'), 'utf8'),
  ) as Record<string, unknown>,
);

describe('analysis.manifest', () => {
  it('accepts every valid fixture', () => {
    const names = readdirSync(join(fixtures, 'valid'));
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) {
      const raw = readFileSync(join(fixtures, 'valid', name), 'utf8');
      expect(validate(JSON.parse(raw)), `${name}: ${ajv.errorsText(validate.errors)}`).toBe(true);
    }
  });

  it('refuses every invalid fixture', () => {
    const names = readdirSync(join(fixtures, 'invalid'));
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) {
      const raw = readFileSync(join(fixtures, 'invalid', name), 'utf8');
      expect(validate(JSON.parse(raw)), `${name} was accepted`).toBe(false);
    }
  });
});

describe('the analysis, as the shell reads it', () => {
  const load = (name: string): AnalysisManifest =>
    JSON.parse(readFileSync(join(fixtures, 'valid', name), 'utf8')) as AnalysisManifest;

  it('hands a project view every artifact address in one read', () => {
    const analysis = load('interview.json');
    expect(analysis.stages.length).toBe(10);
    const byKind = new Map(analysis.stages.map((stage) => [stage.kind, stage.artifact_id]));
    // The two a results board opens first.
    expect(byKind.get('ranking.set.v1')).toMatch(/^sha256:[0-9a-f]{64}$/);
    expect(byKind.get('speech.transcript.v1')).toMatch(/^sha256:[0-9a-f]{64}$/);
    expect(byKind.size).toBe(analysis.stages.length);
  });

  it('lets a view distinguish "nothing to find" from "not looked for yet"', () => {
    const analysis = load('audio_only.json');
    const produced = new Set(analysis.stages.map((stage) => stage.kind));
    expect(produced.has('evidence.shots.v1')).toBe(false);

    // Absent *and* accounted for. Without the second half a view has to render
    // "no shot cuts" for a recording with no video and for one nobody has
    // analyzed, which are opposite things to tell somebody.
    const skipped = analysis.skipped ?? [];
    expect(skipped.map((stage) => stage.kind)).toContain('evidence.shots.v1');
    expect(skipped[0]?.reason).toBe('no_video');
  });

  it('reports a silent recording as a partial analysis rather than a failure', () => {
    const analysis = load('silent_footage.json');
    // Shot cuts were still found. A view that treated a missing ranked set as an
    // error would hide work that did succeed.
    expect(analysis.stages.map((stage) => stage.kind)).toContain('evidence.shots.v1');
    const skipped = analysis.skipped ?? [];
    expect(skipped.length).toBe(7);
    expect(new Set(skipped.map((stage) => stage.reason))).toEqual(new Set(['no_audio']));
  });

  it('gives a timeline a span to draw, and says whether it is the whole recording', () => {
    for (const name of readdirSync(join(fixtures, 'valid'))) {
      const analysis = load(name);
      expect(Number.isInteger(analysis.coverage.start_ticks)).toBe(true);
      expect(Number.isInteger(analysis.coverage.end_ticks)).toBe(true);
      expect(analysis.coverage.end_ticks).toBeGreaterThanOrEqual(analysis.coverage.start_ticks);
      expect(typeof analysis.coverage.analyzed).toBe('boolean');
    }
    // A partial analysis says so, so a timeline can shade what nobody examined
    // instead of implying the whole recording was read.
    expect(load('partial_coverage.json').coverage.analyzed).toBe(false);
    expect(load('interview.json').coverage.analyzed).toBe(true);
  });
});
