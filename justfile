# ClipMill development entry points. `just --list` shows everything.

default:
    @just --list

# One-time setup: pinned FFmpeg, Python venvs, Node workspace.
setup:
    ./tools/fetch-ffmpeg.sh
    cd workers/sdk && uv sync
    cd workers/echo && uv sync
    cd eval/harness && uv sync
    pnpm install

# Regenerate all contract code from contracts/ (protobuf + JSON Schema).
codegen:
    ./tools/codegen/generate.sh

# Every linter, matching CI.
lint:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
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
    cd workers/sdk && uv run pytest tests/test_contracts.py
    pnpm --filter @clipmill/contracts test

# Exit gate: jobs survive forced termination (N=50 locally, 25 in CI).
gate-kill:
    ./tools/drills/kill-drill.sh 50

# Exit gate: deterministic cache round-trips. Lands with W3 (CAS store).
gate-cache:
    @echo "gate-cache: not yet implemented - lands with W3 (content-addressed store)" && exit 1

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

# All Phase 0 gates in sequence.
gate-phase0: gate-contracts gate-kill gate-cache gate-lock

# Launch the desktop shell. Lands with W9/W10.
app:
    @echo "app: not yet implemented - the Tauri shell lands in W9/W10" && exit 1
