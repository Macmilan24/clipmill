# Phase 0 threat model

## Scope and security claim

Phase 0 proves the local harness on macOS and Linux: one daemon owns durable
state and artifact publication; local workers authenticate over private Unix
sockets; media parsing is supervised; shared-memory handles are lease-bound;
and the full offline suite passes in a Linux network namespace whose egress
canary is denied. The protected repository also verifies dependency policy,
generated code, pinned FFmpeg provenance, and the signed Seed-40 result.

This is not the final desktop security boundary. Production OS firewall
enforcement, the signed worker/model registry, desktop WebView capabilities,
release signing, notarization, and packaging remain Phase 4 or the release
workstream. The Phase 0 Local Lock claim is exactly the CI namespace and egress
canary—not a promise that an arbitrary host process is contained.

## Assets and trust boundaries

The assets are original media, source paths, project metadata, immutable CAS
evidence, SQLite state, worker/device/evaluation private keys, worker trust
entries, signed corpus rights, and the truth of durability acknowledgements.

The boundaries are:

1. untrusted media into the pinned FFprobe subprocess and source normalizer;
2. local control clients into length-delimited Protobuf IPC;
3. externally launched workers into authenticated worker IPC;
4. worker-written staging into daemon-owned CAS validation and publication;
5. daemon-created shared memory into a read-only Python mapping;
6. repository dependencies and downloaded FFmpeg archives into the build;
7. private Seed-40 inputs into the path-free public run attestation;
8. every offline-core process against the denied-network namespace.

The local OS account and kernel are trusted in Phase 0. Other users, remote
services, media bytes, IPC frames, worker processes before authentication,
subprocess output, dependency metadata, and corpus inputs are not trusted.

## Threats, controls, and falsification gates

| Threat                                     | Phase 0 controls                                                                                                                                                                                                                                                      | Gate that must fail closed                                                                                                                                                       |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Hostile or malformed media                 | Regular-file and immutable-observation checks; URL/device/symlink rejection; pinned FFprobe; file/pipe-only protocols; sanitized environment; private working directory; bounded output; 15-second deadline; graceful then forced termination; rational normalization | Media conformance covers malformed/truncated metadata, gaps, resets, timeouts, mutation races, and URL rejection. Public evaluation includes an expected hostile input.          |
| Malformed control or worker IPC            | 4 MiB frame cap, bounded read/write deadlines, strict one-body validation, typed IDs/cursors, sequential requests per connection, 64-connection cap, stale lease rejection                                                                                            | Framing tests cover fragmented and malformed varints, truncation, zero bodies, oversize frames, and invalid fields. Worker recovery covers stale/mismatched completions.         |
| Worker impersonation or replay             | Fresh random challenge; Ed25519 signature over the complete descriptor; local public-key trust directory; worker-ID uniqueness; current/previous minor only; daemon remains scheduling truth                                                                          | Worker gate rejects unsigned, untrusted, replayed, duplicate, incompatible, and capability-mismatched registrations. Phase 4 replaces local trust with the signed registry.      |
| Shared-memory mutation, confusion, or leak | Linux sealed `memfd` plus `SCM_RIGHTS`; macOS read-only POSIX shared memory; random one-use token; lease/type/shape/length/timebase/hash validation; read-only zero-copy map; revocation on every terminal session path                                               | Worker gate proves seal/map/validate/acknowledge/cleanup and scans for handles after success, cancellation, disconnect, expiry, worker death, and daemon death.                  |
| Path traversal or unsafe filesystem object | Portable relative artifact paths; no absolute/dot/backslash/NUL/reserved components; exact declared files; regular files only; symlink rejection; private directories; daemon-owned hashing; atomic rename and parent `fsync`                                         | Artifact tests cover traversal, symlinks, missing/extra/duplicate files, modes, tampering, quarantine, and exact visibility. Repository scan rejects private media and keys.     |
| Partial publication or state corruption    | Single SQLite writer; WAL/FULL/foreign keys; startup integrity check; backed-up transactional migrations; staged/fsynced/atomic CAS; root only after object publication; completion acknowledgement only after one durable state transaction                          | Cache/job/worker hard-kill drills inject death at publication and lease boundaries, then verify every acknowledgement, root, payload, cursor, and recovery deadline.             |
| Subprocess escape or resource exhaustion   | No shell interpolation; explicit executable/arguments; sanitized minimal environment; private cwd; stdin closed; protocol allowlist; output/deadline bounds; termination escalation                                                                                   | Media and device gates execute the pinned binaries on both OSes; threat review is mandatory for subprocess changes. A production sandbox profile remains later work.             |
| Secret, path, or payload leakage           | Logs omit names, paths, payload bytes, canonical configuration, and private keys; private state is `0600`; private Seed-40 data remains outside Git; attestation recursively rejects path fields and absolute/file URIs                                               | Repository scan rejects private-key markers, credential patterns, media extensions, oversized blobs, and non-public Seed-40 files. Attestation tests inject paths and tampering. |
| Dependency or generated-code compromise    | Immutable Action commits with version comments; checkout credentials disabled; read-only workflow token; Rust/Python/Node license policies; Cargo, pip, and pnpm vulnerability audits; locked installs; generated-code drift; FFmpeg archive hashes and license flags | Contracts, supply-chain, and `phase0-security` jobs must all pass. Dependabot covers Cargo, npm, every uv project, and Actions.                                                  |
| Forbidden egress                           | No network operation in daemon, CAS, scheduler, probe, worker, shared-memory, profile, or evaluation protocols; CI runs the integrated suite after an outbound canary fails                                                                                           | `network-denial (Local Lock gate)` uses `unshare -rn`; any successful canary or any offline suite failure rejects the change. Production OS enforcement remains Phase 4.         |
| Corpus substitution or rights misstatement | Ed25519-signed corpus and license documents; byte-size/SHA-256 validation; exact license coverage; explicit redistributable or evaluation permission; exactly 40 items; dedicated private run key; canonical signed public evidence                                   | `gate-seed40` validates private inputs and cold/warm results. `gate-phase0` independently verifies the committed four-file proof and rejects private paths or extra files.       |

## Security review workflow

Every pull request uses the threat-review checklist. CI derives applicable
categories from the changed paths and rejects an unchecked relevant category.
Changes to parsers, subprocesses, IPC/authentication, filesystem publication,
credentials/logging, network policy, dependencies, licenses, or generated code
therefore require an explicit reviewer statement in the PR body. A checked box
means the author examined the relevant row above, added or updated a
falsification test, and documented any residual risk; it is not a claim that
the code is risk-free.

## Residual risks and later ownership

- Phase 4 must enforce Local Lock with OS facilities and replace local worker
  trust with the signed worker/model registry.
- Desktop WebView and capability isolation are outside Phase 0.
- Production model formats and parsers are not yet present; each joins this
  model and the no-network suite when introduced.
- Release archives, signing keys, notarization, update metadata, and rollback
  are owned by the desktop/release workstream.
- Phase 0 keeps Seed-40 media private. Its committed signature proves the run
  and aggregate rights metadata without publishing restricted bytes or paths.
