# Phase 0 evaluation harness

The W7 harness evaluates source evidence through the same daemon IPC, durable
job engine, and CAS used by production stages. It accepts only a local corpus
whose manifest and license attestation carry valid Ed25519 signatures. Before a
request reaches the daemon, every media file is checked for a portable relative
path, regular-file type, exact byte size, and SHA-256 digest. License records
must cover exactly the signed item set.

The committed public smoke recipe generates five tiny synthetic fixtures with
the pinned FFmpeg sidecar: CFR, VFR, rotation metadata, offset multi-audio, and
one intentionally malformed input. No binary media or private signing key is
stored in Git. The gate starts a real daemon, verifies its signed device-profile
artifact, registers each source, waits for its durable probe job, verifies the
source-map CAS payload, and repeats valid sources warm. A warm run must reuse
the exact source-map artifact ID.

Run the reproducible public gate with:

```sh
./tools/fetch-ffmpeg.sh
just gate-device
just gate-eval-smoke
```

The canonical `clipmill.eval.run.v1` manifest records daemon/schema versions,
policy, hardware-profile identity and generation, source fingerprints,
artifact IDs, cold/warm timings, cache results, and expected structured
failures. It never records corpus directories or source paths. CI runs the gate
on macOS and Linux, and the Linux Local Lock job repeats it inside a namespace
where the egress canary proves networking is unavailable.

## Private Seed-40 exit

The private Seed-40 baseline is deliberately not a repository media artifact.
The rights holder keeps the media, full signed corpus manifest, item-level
license attestation, and private evaluation signing key outside Git. A license
record must explicitly grant either redistribution or evaluation; private
evaluation permission is not mislabeled as redistribution.

The evaluation key is a raw 32-byte Ed25519 seed encoded as hexadecimal in a
regular, non-symlink file with mode `0600`. With the pinned sidecars installed,
the rights holder runs:

```sh
just gate-seed40 \
  /absolute/private/corpus \
  /absolute/private/corpus-manifest.json \
  /absolute/private/license-attestation.json \
  /absolute/private/phase0-signing.key \
  eval/seed40 \
  /absolute/private/corpus-verification-key.hex
```

The gate refuses any count other than 40, verifies every signed byte and rights
record, runs every item cold and warm through a real daemon, requires every
valid source map to verify with an identical warm artifact ID, and requires
every declared hostile item to repeat its expected structured failure. It then
writes exactly four public files under `eval/seed40/`:

- `verification-key.hex` — the dedicated run-verification public key;
- `corpus-metadata.json` — corpus ID, signed-manifest digest, signing public
  key, and valid/hostile counts;
- `license-summary.json` — signed-license-document digest and aggregate,
  publishable rights counts;
- `run-attestation.json` — the canonical, Ed25519-signed, path-free cold/warm
  run and the exact copies of both public summaries.

`just gate-phase0` verifies that exact committed file set without access to the
private corpus. It fails if a signature, count, cache identity, outcome,
license total, canonical byte, or path-leak invariant changes. Phase 0 remains
incomplete until the real four-file proof is committed, all required checks are
green, and the W8 pull request is merged to protected `main`.
