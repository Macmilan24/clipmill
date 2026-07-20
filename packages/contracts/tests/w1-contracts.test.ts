/** W1 contract fixtures, TypeScript leg. */
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { fromBinary, fromJson, toBinary } from '@bufbuild/protobuf';
import type { JsonValue } from '@bufbuild/protobuf';
import { Ajv2020 } from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import { canonicalJson } from '../src/canonical.js';
import {
  BufferDescriptorSchema,
  DataType,
  DemoDagPayloadV1Schema,
  DeviceProfilePayloadV1Schema,
  ProbeSourcePayloadV1Schema,
  CapabilityDescriptorSchema,
  TransportType,
} from '../src/index.js';

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

describe('W5 source-map mapping extension', () => {
  it.each(['minimal.json', 'with-mapping.json'])('accepts valid fixture %s', (name) => {
    const raw = readFileSync(join(fixtures, 'source_map', 'valid', name), 'utf8');
    const parsed: unknown = JSON.parse(raw);
    expect(validators.source_map!(parsed), ajv.errorsText(validators.source_map!.errors)).toBe(
      true,
    );
  });

  it.each(['float-ticks.json', 'bad-mapping-timebase.json'])(
    'rejects invalid fixture %s',
    (name) => {
      const raw = readFileSync(join(fixtures, 'source_map', 'invalid', name), 'utf8');
      expect(validators.source_map!(JSON.parse(raw))).toBe(false);
    },
  );
});

describe('W7 device-profile extension', () => {
  it('accepts the measured and attested Phase 0 fixture', () => {
    const raw = readFileSync(join(fixtures, 'device_profile', 'valid', 'phase0.json'), 'utf8');
    expect(validators.device_profile!(JSON.parse(raw))).toBe(true);
    expect(canonicalJson(JSON.parse(raw) as JsonValue)).toBe(raw);
  });

  it('rejects malformed attestation material', () => {
    const raw = readFileSync(
      join(fixtures, 'device_profile', 'invalid', 'bad-attestation.json'),
      'utf8',
    );
    expect(validators.device_profile!(JSON.parse(raw))).toBe(false);
  });

  it('enforces the device-profile job payload key version', () => {
    const valid = JSON.parse(
      readFileSync(join(fixtures, 'proto', 'device_profile', 'valid', 'payload.json'), 'utf8'),
    ) as JsonValue;
    const payload = fromJson(DeviceProfilePayloadV1Schema, valid);
    expect(payload.keyVersion).toBe('clipmill.device-profile.v1');
    expect(payload.measurementGeneration).toBe(1n);

    const invalid = JSON.parse(
      readFileSync(
        join(fixtures, 'proto', 'device_profile', 'invalid', 'wrong-version.json'),
        'utf8',
      ),
    ) as JsonValue;
    expect(fromJson(DeviceProfilePayloadV1Schema, invalid).keyVersion).not.toBe(
      'clipmill.device-profile.v1',
    );
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

describe('W5 probe-source payload', () => {
  it('enforces the source-evidence semantic key version fixtures', () => {
    const valid = JSON.parse(
      readFileSync(join(fixtures, 'proto', 'probe_source', 'valid', 'payload.json'), 'utf8'),
    ) as JsonValue;
    const message = fromJson(ProbeSourcePayloadV1Schema, valid);
    expect(message.keyVersion).toBe('clipmill.probe-source.v1');
    expect(message.sourceId.startsWith('src_')).toBe(true);

    const invalid = JSON.parse(
      readFileSync(
        join(fixtures, 'proto', 'probe_source', 'invalid', 'wrong-version.json'),
        'utf8',
      ),
    ) as JsonValue;
    expect(fromJson(ProbeSourcePayloadV1Schema, invalid).keyVersion).not.toBe(
      'clipmill.probe-source.v1',
    );
  });
});

describe('W6 worker protocol extension', () => {
  it('loads the signed capability fixture and exposes shared-memory transports', () => {
    const raw = JSON.parse(
      readFileSync(join(fixtures, 'proto', 'worker', 'valid', 'capability.json'), 'utf8'),
    ) as JsonValue;
    const descriptor = fromJson(CapabilityDescriptorSchema, raw);
    expect(descriptor.protocolVersion).toBe('1.1');
    expect(descriptor.capabilities).toContain('demo-join');
    expect(TransportType.SCM_RIGHTS_MEMFD).not.toBe(TransportType.UNSPECIFIED);
  });
});
