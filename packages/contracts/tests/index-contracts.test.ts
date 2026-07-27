/**
 * The evidence index's contract, TypeScript leg.
 *
 * The shell reads this to draw the outline: utterances in the transcript pane,
 * topic bands over the timeline, and the edges a drag snaps to. It never
 * authors one, so byte-identity is asserted in Rust; what matters here is that
 * the published schema accepts what the daemon writes, and that the type the
 * renderer codes against can actually be walked back to words.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import type { IndexTranscript } from '../src/gen/schemas/index-transcript.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const fixtures = join(repo, 'contracts', 'fixtures', 'index.transcript');
const ajv = new Ajv2020({ allErrors: true });
const validate = ajv.compile(
  JSON.parse(
    readFileSync(join(repo, 'contracts', 'schemas', 'clipmill.index.transcript.v1.json'), 'utf8'),
  ) as Record<string, unknown>,
);

describe('index.transcript', () => {
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

describe('the index, as the shell reads it', () => {
  const load = (name: string): IndexTranscript =>
    JSON.parse(readFileSync(join(fixtures, 'valid', name), 'utf8')) as IndexTranscript;

  it('gives the transcript pane utterances it can lay out directly', () => {
    const index = load('ten_words.json');
    expect(index.utterances.length).toBeGreaterThan(0);
    for (const utterance of index.utterances) {
      expect(Number.isInteger(utterance.start_ticks)).toBe(true);
      expect(utterance.end_ticks).toBeGreaterThanOrEqual(utterance.start_ticks);
      expect(utterance.text.length).toBeGreaterThan(0);
    }
  });

  it('lets a quote be traced back to the words behind it', () => {
    const index = load('ten_words.json');
    for (const sentence of index.sentences) {
      expect(sentence.first_word_index).toBeGreaterThanOrEqual(0);
      expect(sentence.word_count).toBeGreaterThan(0);
      expect(sentence.utterance_index).toBeLessThan(index.utterances.length);
    }
  });

  it('marks how strong each sentence boundary is', () => {
    const index = load('ten_words.json');
    const kinds = new Set(index.sentences.map((sentence) => sentence.terminator));
    for (const kind of kinds) {
      expect(['punctuation', 'utterance_end', 'coverage_end']).toContain(kind);
    }
  });
});
