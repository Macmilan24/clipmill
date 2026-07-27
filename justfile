# ClipMill development entry points. `just --list` shows everything.

default:
    @just --list

# One-time setup: pinned FFmpeg, Python venvs, Node workspace.
setup:
    ./tools/fetch-ffmpeg.sh
    cd workers/sdk && uv sync
    cd workers/echo && uv sync
    cd workers/shots && uv sync
    cd eval/harness && uv sync
    pnpm install

# Regenerate all contract code from contracts/ (protobuf + JSON Schema).
codegen:
    ./tools/codegen/generate.sh

# Every linter, matching CI.
lint:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
    uvx ruff check .
    uvx ruff format --check .
    pnpm lint
    pnpm format
    python3 tools/schema-lint/check.py contracts/schemas/*.json

# Every test suite, matching CI.
test:
    cargo test --workspace
    cd workers/sdk && uv run pytest
    cd workers/echo && uv run pytest
    cd workers/vad && uv run pytest
    cd workers/asr-whispercpp && uv run pytest
    cd workers/align && uv run pytest
    cd workers/speech-mlx && uv run pytest
    cd workers/shots && uv run pytest
    cd eval/harness && uv run pytest
    pnpm typecheck
    pnpm test

# ---- Phase 0 exit gates (the book's ch. 24 harness gates) ----

# Exit gate: contracts compile to Rust/Python/TS and fixtures round-trip.
gate-contracts:
    ./tools/codegen/generate.sh
    git diff --exit-code -- crates/clipmill-contracts/src/gen packages/contracts/src/gen workers/sdk/src/clipmill workers/sdk/src/clipmill_worker_sdk/gen
    python3 tools/schema-lint/check.py contracts/schemas/*.json
    cargo test -p clipmill-contracts
    cd workers/sdk && uv run pytest tests/test_contracts.py tests/test_speech_contracts.py tests/test_shots_contracts.py tests/test_index_contracts.py tests/test_discovery_contracts.py tests/test_ranking_contracts.py
    pnpm --filter @clipmill/contracts test

# W2 coverage: acknowledged project mutations survive forced termination.
# W4 extends this drill with task leases and interrupted-job recovery.
gate-kill:
    ./tools/drills/kill-drill.sh 50

# W3 coverage: acknowledged filesystem + SQLite artifact publications survive
# forced termination and every visible cache object verifies after recovery.
gate-cache:
    ./tools/drills/cache-drill.sh 50

# W5 coverage: pinned FFprobe, immutable source observations, source maps,
# deterministic warm hits, hostile input rejection, and mutation detection.
gate-media:
    ./tools/drills/media-drill.sh 1

# W6 coverage: authenticated external workers, durable completion replay,
# shared-memory validation/cleanup, cancellation, and worker/daemon recovery.
gate-workers:
    ./tools/drills/worker-drill.sh 50

# W12 coverage: every edit command inverts exactly, a command log replays onto
# the live document byte for byte, acknowledged edits survive a killed daemon,
# and render snapshots are content-addressed without their rationale.
gate-ir:
    ./tools/drills/ir-drill.sh 1

# W11 coverage: the ingest fan-out derives every media derivative through
# sandboxed FFmpeg, all artifacts verify, warm re-ingest is a cache identity,
# mutated sources refuse deterministically, and kill recovery meets the SLO.
gate-ingest:
    ./tools/drills/ingest-drill.sh 1

# W14 coverage: a stage exists only if it is registered; a worker is admitted
# against the device the machine actually has rather than its own claim; a
# worker reads the daemon's store without trusting it; and weights that forbid
# what users do with the output are refused by policy.
gate-worker2:
    ./tools/drills/worker2-drill.sh 1

# W15 coverage: the speech chain. Stage algorithms against hand-written inputs
# — the cases no real recording contains — then the pinned models over a
# fixture whose word timing is known by construction rather than by
# annotation: voice activity finds the utterances the fixture was built from,
# recognition returns its words, alignment places them within 120 ms, and a
# second run produces byte-identical output.
gate-speech iterations="1":
    ./tools/drills/speech-drill.sh {{iterations}}

# W15 coverage on hardware CI does not have: the accelerated speech path.
# Measures every implementation the daemon can plan, asserts this device binds
# recognition and alignment to MLX *by measurement* rather than by default,
# holds the accelerated aligner to the same 120 ms bar, and signs the result.
# The private key never enters Git; only the public half and the signed
# document do, and CI verifies those without pretending to have measured
# anything (R18's pattern, applied to D19).
gate-asr-mlx signing_key output_dir="models/attestations/mlx-selection":
    ./tools/drills/asr-mlx-drill.sh --signing-key "{{signing_key}}" --output-dir "{{output_dir}}"

# W16 coverage: shot detection. Stage arithmetic against frames written by hand
# — a cut on the first frame, a flash, a recording that never changes — then the
# detector over a fixture whose cuts are known by construction and encoded
# exactly as the ingest proxy is: every cut found and none invented, a pan
# faster than a screen width per second not mistaken for one, a second pass
# byte-identical, and a proxy that is not video refused with a reason.
gate-shots iterations="1":
    ./tools/drills/shots-drill.sh {{iterations}}

# W17 coverage: the structure read out of a transcript. The levels against
# transcripts written by hand — a speaker who never pauses, a recognizer that
# punctuates nothing, a segment whose text and word count disagree — then every
# committed transcript indexed against a reviewed golden, the invariants
# discovery never rechecks (units tile the words, topics tile the sentences,
# every unit resolves to words somebody measured), and a second pass producing
# the same bytes.
gate-evidence iterations="1":
    ./tools/drills/evidence-drill.sh {{iterations}}

# W18 coverage: the proposer mesh and the lattice under it. The three
# proposers against recordings written by hand — a topic of one sentence, a
# question nobody answered, a shot cut inside a word — then every committed
# index searched against a reviewed golden, every lattice point paired with
# something legal, every candidate explicable and grouped, and a second pass
# producing the same bytes.
gate-discovery iterations="1":
    ./tools/drills/discovery-drill.sh {{iterations}}

# W19 coverage: what a clip is worth, where it is cut, and which to show. The
# score card against cards written by hand — an axis nobody measured must be
# distinguishable from one measured at zero — then J verified against brute
# force over the whole lattice, a set that comes back short rather than padded,
# and every committed cohort ranked against a reviewed golden.
gate-ranking iterations="1":
    ./tools/drills/ranking-drill.sh {{iterations}}

# W13 coverage — the first-slice milestone: the published Edit IR renders to a
# 1080x1920 clip with burned karaoke captions, matching sidecars, normalised
# loudness, and a manifest whose digests match its files; the same document
# renders to the same bytes in a fresh store; a repeat is a cache identity; an
# unattested render is refused; a killed daemon recovers.
gate-render:
    ./tools/drills/render-drill.sh 1

# W7 coverage: measured runtime/capacity, bounded codec/shared-memory probes,
# Ed25519 attestation, durable caching, and fingerprint generations.
gate-device:
    ./tools/drills/device-drill.sh 1

# W7 coverage: signed public corpus, real daemon IPC, cold/warm source-map
# cache identity, device-profile verification, CAS verification, hostile media.
gate-eval-smoke:
    ./tools/drills/eval-smoke.sh 1

# W8 rights-holder gate. The media, full signed manifest, license records, and
# private Ed25519 key remain outside Git. Only OUTPUT_DIR is safe to commit.
gate-seed40 corpus_dir manifest license_attestation signing_key output_dir="eval/seed40" corpus_public_key="":
    ./tools/drills/seed40-drill.sh "{{corpus_dir}}" "{{manifest}}" "{{license_attestation}}" "{{signing_key}}" "{{output_dir}}" "{{corpus_public_key}}"

# Exit gate: Local Lock. Replays the CI namespace job in a no-network
# container (Docker/OrbStack); the egress canary must be blocked.
gate-lock:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v docker >/dev/null 2>&1; then
        docker run --rm --network=none -v "$PWD":/w -w /w rust:1.96 ./tools/drills/network-denial.sh
    else
        echo "gate-lock: docker not found - install Docker/OrbStack, or rely on the"
        echo "network-denial CI job (the authoritative Linux-namespace gate)."
        exit 1
    fi

# W8 public security and supply-chain policy checks.
gate-security:
    ./tools/security/security-gate.sh

# W9 coverage: design tokens generate their CSS reproducibly, and the shell
# renderer typechecks, tests, and builds.
gate-tokens:
    pnpm --filter @clipmill/tokens check-drift
    pnpm --filter @clipmill/tokens test
    pnpm --filter @clipmill/desktop typecheck
    pnpm --filter @clipmill/desktop test
    pnpm --filter @clipmill/desktop build

# W10 coverage: the shell reads real measured hardware over the real socket,
# then reports the loss when the daemon is killed underneath it.
gate-shell:
    cargo build -p clipmilld
    cargo test -p clipmill-shell -- --ignored --nocapture

# All reproducible Phase 0 gates plus the committed private-run attestation.
# Running Seed-40 itself requires private rights-holder inputs via gate-seed40.
gate-phase0: gate-contracts gate-kill gate-cache gate-media gate-workers gate-device gate-eval-smoke gate-tokens gate-shell gate-security gate-lock
    ./tools/drills/verify-phase0-attestation.sh

# Launch the desktop shell against a live daemon.
app:
    pnpm --filter @clipmill/desktop tauri dev
