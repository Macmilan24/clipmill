/**
 * The speech chain's contracts, TypeScript leg.
 *
 * The shell reads transcripts to draw the timeline, the caption lanes, and the
 * evidence quotes in the inspector. It never authors one, so byte-identity is
 * asserted in Rust and Python; what matters here is that the published schema
 * accepts every document those two write, and that the types the renderer
 * codes against actually describe it.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import type { SpeechTranscript } from '../src/gen/schemas/speech-transcript.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const fixtures = join(repo, 'contracts', 'fixtures');
const ajv = new Ajv2020({ allErrors: true });

const kinds = ['speech.vad', 'speech.asr', 'speech.alignment', 'speech.transcript'] as const;

describe.each(kinds)('%s', (kind) => {
  const validate = ajv.compile(
    JSON.parse(
      readFileSync(join(repo, 'contracts', 'schemas', `clipmill.${kind}.v1.json`), 'utf8'),
    ) as Record<string, unknown>,
  );

  it('accepts every valid fixture', () => {
    const names = readdirSync(join(fixtures, kind, 'valid'));
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) {
      const raw = readFileSync(join(fixtures, kind, 'valid', name), 'utf8');
      expect(validate(JSON.parse(raw)), `${name}: ${ajv.errorsText(validate.errors)}`).toBe(true);
    }
  });

  it('refuses every invalid fixture', () => {
    const names = readdirSync(join(fixtures, kind, 'invalid'));
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) {
      const raw = readFileSync(join(fixtures, kind, 'invalid', name), 'utf8');
      expect(validate(JSON.parse(raw)), `${name} was accepted`).toBe(false);
    }
  });
});

describe('transcript, as the shell reads it', () => {
  const load = (name: string): SpeechTranscript =>
    JSON.parse(
      readFileSync(join(fixtures, 'speech.transcript', 'valid', name), 'utf8'),
    ) as SpeechTranscript;

  it('gives the caption lane word intervals it can lay out directly', () => {
    const transcript = load('ten_words.json');
    expect(transcript.words.length).toBeGreaterThan(0);
    for (const word of transcript.words) {
      expect(word.end_ticks).toBeGreaterThan(word.start_ticks);
      expect(Number.isInteger(word.start_ticks)).toBe(true);
    }
  });

  it('lets the inspector say which timings were measured and which were spread', () => {
    const spread = load('interpolated_timing.json');
    const guessed = spread.words.filter((word) => word.timing === 'interpolated');
    expect(guessed.length).toBeGreaterThan(0);
    // The UI has to be able to shade those spans without recomputing anything.
    expect(spread.invalid_regions.some((region) => region.reason === 'timing_interpolated')).toBe(
      true,
    );
  });
});
