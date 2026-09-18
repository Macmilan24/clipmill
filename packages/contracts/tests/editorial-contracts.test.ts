/**
 * The editorial contracts, TypeScript leg.
 *
 * The shell will read judgments (a status and reasons on the board) and
 * proposals (the "why" behind a candidate); it never authors any of these,
 * and it must never be able to read the trace. Byte-identity is asserted in
 * Rust; what matters here is that the published schemas accept what the
 * daemon and the worker write and refuse what they must never carry — a
 * timestamp in a proposal, a score in a judgment, a key in a trace.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import type { EditorialJudgments } from '../src/gen/schemas/editorial-judgments.js';
import type { EditorialWindows } from '../src/gen/schemas/editorial-windows.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const ajv = new Ajv2020({ allErrors: true });

const KINDS = ['windows', 'proposals', 'judgments', 'looks', 'trace'] as const;

for (const kind of KINDS) {
  const fixtures = join(repo, 'contracts', 'fixtures', `editorial.${kind}`);
  const validate = ajv.compile(
    JSON.parse(
      readFileSync(join(repo, 'contracts', 'schemas', `clipmill.editorial.${kind}.v1.json`), 'utf8'),
    ) as Record<string, unknown>,
  );

  describe(`editorial.${kind}`, () => {
    it('accepts every valid fixture', () => {
      const names = readdirSync(join(fixtures, 'valid'));
      expect(names.length).toBeGreaterThan(0);
      for (const name of names) {
        const raw = readFileSync(join(fixtures, 'valid', name), 'utf8');
        expect(validate(JSON.parse(raw)), `${name}: ${ajv.errorsText(validate.errors)}`).toBe(
          true,
        );
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
}

describe('what the shell will read', () => {
  it('finds every window a sentence is in, by ranges rather than copies', () => {
    const windows = JSON.parse(
      readFileSync(join(repo, 'contracts/fixtures/editorial.windows/valid/talk.json'), 'utf8'),
    ) as EditorialWindows;
    const sentence = windows.sentences[4]!;
    const holding = windows.windows.filter(
      (window) =>
        window.first_sentence_index <= sentence.index &&
        sentence.index < window.first_sentence_index + window.sentence_count,
    );
    expect(holding.map((window) => window.index)).toEqual([0, 1]);
    expect(windows.windows.every((window) => !('text' in window))).toBe(true);
  });

  it('shows a status and its reasons, and has no number to show', () => {
    const judgments = JSON.parse(
      readFileSync(join(repo, 'contracts/fixtures/editorial.judgments/valid/talk.json'), 'utf8'),
    ) as EditorialJudgments;
    const answered = judgments.candidates.filter((judgment) => judgment.outcome === 'answered');
    expect(answered.map((judgment) => judgment.status)).toEqual([
      'accepted',
      'needs_review',
      'rejected',
    ]);
    for (const judgment of answered) {
      if (judgment.status !== 'accepted') {
        expect(judgment.reasons.length).toBeGreaterThan(0);
        for (const reason of judgment.reasons) {
          expect(reason.detail.length).toBeGreaterThan(0);
        }
      }
      expect('score' in judgment).toBe(false);
      expect('display_score' in judgment).toBe(false);
    }
  });
});
