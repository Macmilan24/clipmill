/**
 * The candidate contract, TypeScript leg.
 *
 * The shell shows candidates before ranking has run: the strip of proposed
 * clips, the cluster's alternatives behind a "why was this a duplicate?"
 * affordance, and the evidence a user clicks through to. It never authors one,
 * so byte-identity is asserted in Rust; what matters here is that the published
 * schema accepts what the daemon writes and that the type the renderer codes
 * against can actually answer those questions.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import type { DiscoveryCandidates } from '../src/gen/schemas/discovery-candidates.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const fixtures = join(repo, 'contracts', 'fixtures', 'discovery.candidates');
const ajv = new Ajv2020({ allErrors: true });
const validate = ajv.compile(
  JSON.parse(
    readFileSync(
      join(repo, 'contracts', 'schemas', 'clipmill.discovery.candidates.v1.json'),
      'utf8',
    ),
  ) as Record<string, unknown>,
);

describe('discovery.candidates', () => {
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

describe('candidates, as the shell shows them', () => {
  const load = (name: string): DiscoveryCandidates =>
    JSON.parse(readFileSync(join(fixtures, 'valid', name), 'utf8')) as DiscoveryCandidates;

  it('gives the strip intervals it can lay out directly', () => {
    const found = load('interview.json');
    expect(found.candidates.length).toBeGreaterThan(0);
    for (const candidate of found.candidates) {
      for (const interval of candidate.intervals) {
        expect(Number.isInteger(interval.start_ticks)).toBe(true);
        expect(interval.end_ticks).toBeGreaterThan(interval.start_ticks);
      }
    }
  });

  it('can answer why something was considered a duplicate', () => {
    const found = load('interview.json');
    const byId = new Map(found.clusters.map((cluster) => [cluster.id, cluster]));
    for (const candidate of found.candidates) {
      const cluster = byId.get(candidate.cluster_id);
      expect(cluster, 'a candidate names an unpublished cluster').toBeDefined();
      expect(cluster?.members).toContain(candidate.id);
    }
    // The alternatives are still there to offer, not dropped.
    const grouped = found.clusters.filter((cluster) => cluster.members.length > 1);
    for (const cluster of grouped) {
      expect(cluster.members).toContain(cluster.representative);
      expect(cluster.similarity).toBeLessThanOrEqual(1);
    }
  });

  it('lets a nomination be traced back to what it rests on', () => {
    const found = load('interview.json');
    for (const candidate of found.candidates) {
      expect(candidate.evidence.length).toBeGreaterThan(0);
      for (const reference of candidate.evidence) {
        expect(['utterance', 'sentence', 'topic']).toContain(reference.kind);
        expect(reference.index).toBeGreaterThanOrEqual(0);
      }
    }
  });

  it('says what each proposer actually measured', () => {
    const found = load('interview.json');
    for (const run of found.proposers) {
      expect(run.proposer.rubric).not.toBe(run.proposer.name);
      expect(run.candidates).toBeLessThanOrEqual(run.seeds);
    }
  });
});
