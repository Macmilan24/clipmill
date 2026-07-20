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

The private Seed-40 baseline is deliberately not a W7 repository artifact.
W8 requires exactly 40 rights-cleared items, an externally provisioned signing
key, publishable verification material, and a signed path-free run attestation.
Until that human-owned input exists and its gate passes on protected `main`,
Phase 0 is not complete.
