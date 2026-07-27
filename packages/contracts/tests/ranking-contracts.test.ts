/**
 * The score card and the ranked set, TypeScript leg.
 *
 * The shell renders this directly: rank rows on the results board, the eight
 * axis bars in the inspector, the boundary strip with its one-click
 * alternative, and the "why fewer than you asked for" line. It never authors
 * one, so byte-identity is asserted in Rust; what matters here is that the
 * type the renderer codes against can answer those questions.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import type { RankingSet } from '../src/gen/schemas/ranking-set.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const fixtures = join(repo, 'contracts', 'fixtures', 'ranking.set');
const ajv = new Ajv2020({ allErrors: true });
const validate = ajv.compile(
  JSON.parse(
    readFileSync(join(repo, 'contracts', 'schemas', 'clipmill.ranking.set.v1.json'), 'utf8'),
  ) as Record<string, unknown>,
);

describe('ranking.set', () => {
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

describe('the ranked set, as the shell renders it', () => {
  const load = (name: string): RankingSet =>
    JSON.parse(readFileSync(join(fixtures, 'valid', name), 'utf8')) as RankingSet;

  it('gives the inspector eight bars it can draw without inventing one', () => {
    const set = load('interview.json');
    for (const entry of set.cohort) {
      expect(entry.factors).toHaveLength(8);
      for (const factor of entry.factors) {
        if (factor.available) {
          expect(typeof factor.value).toBe('number');
        } else {
          // An unmeasured axis is drawn as absent, not as a bar at zero.
          expect(factor.value).toBeUndefined();
          expect(factor.unavailable_reason).toBeTruthy();
        }
      }
    }
  });

  it('gives the boundary strip an alternative to offer', () => {
    const set = load('interview.json');
    const withAlternative = set.cohort.filter((entry) => entry.boundary.alternative);
    expect(withAlternative.length).toBeGreaterThan(0);
    for (const entry of withAlternative) {
      expect(entry.boundary.alternative?.score).toBeLessThanOrEqual(entry.boundary.score);
    }
  });

  it('can say why the board is shorter than the user asked for', () => {
    const set = load('interview.json');
    const accounted = set.shortfall.reduce((total, reason) => total + reason.count, 0);
    expect(set.selected.length + accounted).toBe(set.requested.count);
  });

  it('lets every selected clip be looked up in the cohort', () => {
    const set = load('interview.json');
    const byId = new Map(set.cohort.map((entry) => [entry.candidate_id, entry]));
    for (const id of set.selected) {
      expect(byId.get(id), 'a selected clip is not in the cohort').toBeDefined();
    }
  });
});
