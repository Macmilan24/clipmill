/** W1 contract fixtures, TypeScript leg. */
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { fromBinary, fromJson, toBinary } from '@bufbuild/protobuf';
import type { JsonValue } from '@bufbuild/protobuf';
import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import { canonicalJson } from '../src/canonical.js';
import { BufferDescriptorSchema, DataType, DemoDagPayloadV1Schema } from '../src/index.js';

const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const fixtures = join(repo, 'contracts', 'fixtures');

const ajv = new Ajv2020({ allErrors: true });
const validators = Object.fromEntries(
  ['source_map', 'device_profile'].map((kind) => [
    kind,
    ajv.compile(
      JSON.parse(
        readFileSync(join(repo, 'contracts', 'schemas', `clipmill.${kind}.v1.json`), 'utf8'),
      ) as Record<string, unknown>,
    ),
  ]),
);

describe.each([
  ['source_map', 'float-ticks.json'],
  ['device_profile', 'missing-measured.json'],
])('%s schema', (kind, invalidName) => {
  it('valid fixture passes and round-trips canonically', () => {
    const raw = readFileSync(join(fixtures, kind, 'valid', 'minimal.json'), 'utf8');
    const parsed: unknown = JSON.parse(raw);
    const validate = validators[kind]!;
    expect(validate(parsed), ajv.errorsText(validate.errors)).toBe(true);
    expect(canonicalJson(parsed)).toBe(raw);
  });

  it(`invalid fixture ${invalidName} fails`, () => {
    const raw = readFileSync(join(fixtures, kind, 'invalid', invalidName), 'utf8');
    expect(validators[kind]!(JSON.parse(raw))).toBe(false);
  });
});

describe('shm buffer descriptor', () => {
  it('binpb round-trips with the cross-package timebase intact', () => {
    const bytes = new Uint8Array(
      readFileSync(join(fixtures, 'proto', 'shm', 'buffer_descriptor.binpb')),
    );
    const twin = JSON.parse(
      readFileSync(join(fixtures, 'proto', 'shm', 'buffer_descriptor.json'), 'utf8'),
    ) as {
      shm_name: string;
      byte_len: number;
      shape: number[];
      timebase: { num: number; den: number };
    };
    const descriptor = fromBinary(BufferDescriptorSchema, bytes);
    expect(descriptor.shmName).toBe(twin.shm_name);
    expect(Number(descriptor.byteLen)).toBe(twin.byte_len);
    expect(descriptor.dtype).toBe(DataType.U8);
    expect(descriptor.shape.map(Number)).toEqual(twin.shape);
    expect(Number(descriptor.timebase?.den)).toBe(twin.timebase.den);
    expect(toBinary(BufferDescriptorSchema, descriptor)).toEqual(bytes);
  });
});

describe('W4 demo DAG payload', () => {
  it('enforces the Phase 0 semantic key version fixtures', () => {
    const valid = JSON.parse(
      readFileSync(join(fixtures, 'proto', 'demo_dag', 'valid', 'payload.json'), 'utf8'),
    ) as JsonValue;
    const message = fromJson(DemoDagPayloadV1Schema, valid);
    expect(message.keyVersion).toBe('clipmill.demo-dag.v1');
    expect(new TextDecoder().decode(message.seed)).toBe('seed-40');

    const invalid = JSON.parse(
      readFileSync(join(fixtures, 'proto', 'demo_dag', 'invalid', 'wrong-version.json'), 'utf8'),
    ) as JsonValue;
    expect(fromJson(DemoDagPayloadV1Schema, invalid).keyVersion).not.toBe('clipmill.demo-dag.v1');
  });
});
