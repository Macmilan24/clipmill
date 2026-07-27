/**
 * Shot detection's contract, TypeScript leg.
 *
 * The shell draws cut markers on the timeline and offers them as snap targets
 * when a user drags a clip boundary. It never authors a shots document, so
 * byte-identity is asserted in Rust and Python; what matters here is that the
 * published schema accepts everything those two write, and that the type the
 * renderer codes against actually describes it.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import type { EvidenceShots } from '../src/gen/schemas/evidence-shots.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const fixtures = join(repo, 'contracts', 'fixtures', 'evidence.shots');
const ajv = new Ajv2020({ allErrors: true });
const validate = ajv.compile(
  JSON.parse(
    readFileSync(join(repo, 'contracts', 'schemas', 'clipmill.evidence.shots.v1.json'), 'utf8'),
  ) as Record<string, unknown>,
);

describe('evidence.shots', () => {
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

describe('shots, as the timeline reads them', () => {
  const load = (name: string): EvidenceShots =>
    JSON.parse(readFileSync(join(fixtures, 'valid', name), 'utf8')) as EvidenceShots;

  it('gives the ruler snap targets it can place directly', () => {
    const shots = load('three_shots.json');
    expect(shots.cuts.length).toBeGreaterThan(0);
    for (const cut of shots.cuts) {
      expect(Number.isInteger(cut.t_ticks)).toBe(true);
      expect(cut.t_ticks).toBeGreaterThanOrEqual(shots.coverage.start_ticks);
      expect(cut.t_ticks).toBeLessThanOrEqual(shots.coverage.end_ticks);
    }
  });

  it('does not let an undecoded recording render as an unbroken take', () => {
    const unexamined = load('never_examined.json');
    const unbroken = load('one_unbroken_shot.json');
    expect(unexamined.cuts).toHaveLength(0);
    expect(unbroken.cuts).toHaveLength(0);
    // The distinguishing read is coverage, which is why it is required.
    expect(unexamined.coverage.analyzed).toBe(false);
    expect(unbroken.coverage.analyzed).toBe(true);
    expect(unexamined.invalid_regions.length).toBeGreaterThan(0);
  });
});
