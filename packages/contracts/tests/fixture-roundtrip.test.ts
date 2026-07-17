/** The Phase 0 contracts exit gate, TypeScript leg. */
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { fromBinary, toBinary } from '@bufbuild/protobuf';
import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import { canonicalJson } from '../src/canonical.js';
import type { ArtifactManifest } from '../src/gen/schemas/artifact-manifest.js';
import { PingRequestSchema } from '../src/index.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const fixtures = join(repo, 'contracts', 'fixtures');
const schema = JSON.parse(
  readFileSync(join(repo, 'contracts', 'schemas', 'clipmill.artifact.manifest.v1.json'), 'utf8'),
) as Record<string, unknown>;

const ajv = new Ajv2020({ allErrors: true });
const validate = ajv.compile(schema);

describe('artifact manifest', () => {
  it('valid fixture passes schema validation and round-trips canonically', () => {
    const raw = readFileSync(join(fixtures, 'artifact.manifest', 'valid', 'minimal.json'), 'utf8');
    const parsed = JSON.parse(raw) as ArtifactManifest;
    expect(validate(parsed), ajv.errorsText(validate.errors)).toBe(true);
    expect(parsed.policy).toBe('local-lock');
    expect(canonicalJson(parsed)).toBe(raw);
  });

  it.each(['float-seconds.json', 'missing-policy.json'])('invalid fixture %s fails', (name) => {
    const raw = readFileSync(join(fixtures, 'artifact.manifest', 'invalid', name), 'utf8');
    expect(validate(JSON.parse(raw))).toBe(false);
  });
});

describe('ping proto', () => {
  it('binpb fixture decodes, matches its JSON twin, and re-encodes identically', () => {
    const bytes = new Uint8Array(
      readFileSync(join(fixtures, 'proto', 'ping', 'ping_request.binpb')),
    );
    const twin = JSON.parse(
      readFileSync(join(fixtures, 'proto', 'ping', 'ping_request.json'), 'utf8'),
    ) as { echo: string };
    const message = fromBinary(PingRequestSchema, bytes);
    expect(message.echo).toBe(twin.echo);
    expect(toBinary(PingRequestSchema, message)).toEqual(bytes);
  });
});
