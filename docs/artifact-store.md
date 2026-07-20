# Artifact content-addressed store (W3)

`clipmill-artifacts` is the daemon-owned immutable evidence store. Filesystem
objects hold reproducible outputs; SQLite holds mutable project roots. There is
no third source of truth and no public artifact RPC in W3.

## Identity and manifest

An artifact ID is `sha256:<64 lowercase hex>`. Only the daemon computes it. The
validated recipe is serialized as an RFC 8785/JCS canonical JSON preimage with:

- key version `clipmill.artifact.key.v1`;
- kind, source fingerprint, rational timebase, and producer stage;
- producer implementation and optional model digest;
- ordered input artifact IDs;
- canonical configuration object, network policy, and semantic version.

The ID is SHA-256 over the byte prefix
`clipmill.artifact.key.v1\0` followed by that canonical preimage. JSON object key
ordering therefore cannot alter the ID, while input ordering and every semantic
component can. Quality signals, payload sizes, and payload hashes are results,
not key inputs. Producing different manifest bytes for an existing recipe key
is `NonDeterministicOutput`; the candidate is quarantined and the committed
object is never overwritten.

Artifact-manifest v1 keeps `recipe` optional so existing fixtures remain
readable. New W3 writes always include it. A legacy manifest can be opened by
its explicit ID but is marked unverifiable and is never a computed cache hit.

## Private layout and paths

```text
<data-dir>/artifacts/
  objects/sha256/<first-two-hex>/<full-digest>/
    manifest.json
    <declared payload files>
  staging/stg_<ulid>/
  quarantine/<reason>-<timestamp>-<id>/
```

Directories are `0700`; writable staging files are `0600`; committed manifests
and payloads are `0400`. Artifact paths are portable UTF-8 relative paths.
Absolute paths, empty or dot components, `..`, backslashes, NUL, duplicates,
symlinks, non-regular files, and the reserved `manifest.json` name are rejected.

## Publication acknowledgement

Only one staging area may exist for a missing recipe key. Commit validates the
staging token and exact declared file set, hashes and sizes each payload, sorts
manifest entries, and then performs this durability sequence:

1. flush and `fsync` payload files, the manifest, and staging directory;
2. atomically rename the complete directory to its digest path;
3. `fsync` the digest parent directory;
4. attach `(project_id, artifact_id)` in SQLite schema v2;
5. return the durability acknowledgement.

A process death before the rename leaves private staging. A death after rename
but before the SQLite commit leaves a complete unreferenced object protected by
the GC grace period. A returned success means both the immutable object and its
project root are durable.

## Recovery, reads, and garbage collection

Startup occurs while the exclusive daemon lock is held. It quarantines all
leftover staging entries, structurally validates committed directories and
manifests, recomputes recipe keys, and rebuilds the in-memory catalog. Invalid
objects, symlinks, undeclared files, and path/key mismatches are quarantined.
Payload hashes are verified lazily on read using an already-open file handle;
the resulting lease pins the object until dropped.

Garbage collection marks project roots, W4 active-task roots, W5 source-map
roots, W7 active device-profile system roots, transitive manifest inputs, and
active reader pins. A missing or corrupt reachable node fails closed and deletes
nothing. Unreachable objects remain protected for `--artifact-gc-grace`,
`CLIPMILL_ARTIFACT_GC_GRACE`, or the seven-day default. A collectible object is
atomically moved to quarantine before recursive deletion. Quarantine cleanup
uses the same grace window. One pass runs after daemon startup and then every
six hours; shutdown cancels maintenance before closing the artifact and
database actors.

The cache drill hard-kills real daemon-backed publisher processes and verifies
all acknowledged roots and visible payloads after restart. W4 adds active task
output roots and transitive task inputs to GC reachability, then proves local
task/job recovery with a separate job kill drill. W6 extends that proof through
authenticated external workers: the daemon prepares lease-scoped staging,
validates the exact declaration, publishes and roots outputs itself, and
durably deduplicates both successful and failed completion acknowledgements.
Worker or daemon death abandons uncommitted staging without exposing a partial
object. W7 uses the same publication ordering for signed device profiles and
retains the active generation through a system root. The broader Phase 0 claim
still requires W8.
