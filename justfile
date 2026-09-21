# ClipMill development entry points. `just --list` shows everything.

default:
    @just --list

# One-time setup: pinned FFmpeg, Python venvs, Node workspace.
setup:
    ./tools/fetch-ffmpeg.sh
    cd workers/sdk && uv sync
    cd workers/echo && uv sync
    cd workers/vad && uv sync
    cd workers/asr-whispercpp && uv sync
    cd workers/align && uv sync
    cd workers/speech-mlx && uv sync
    cd workers/shots && uv sync
    cd workers/faces && uv sync
    cd workers/editorial && uv sync
    cd integrations/youtube-import && uv sync --frozen
    cd eval/harness && uv sync
    pnpm install

# Verify the optional YouTube source importer and its runtime.
setup-youtube:
    cd integrations/youtube-import && uv sync --frozen
    ./tools/import-youtube.sh --check --ffmpeg "{{justfile_directory()}}/.cache/bin/ffmpeg"

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
    python3 -m unittest discover -s tools/security/tests
    cd workers/sdk && uv run pytest
    cd workers/echo && uv run pytest
    cd workers/vad && uv run pytest
    cd workers/asr-whispercpp && uv run pytest
    cd workers/align && uv run pytest
    cd workers/speech-mlx && uv run pytest
    cd workers/shots && uv run pytest
    cd workers/faces && uv run pytest
    cd workers/editorial && uv run pytest
    cd integrations/youtube-import && uv run --frozen pytest
    cd eval/harness && uv run pytest
    pnpm typecheck
    pnpm test

# Integration and recovery checks

# Regenerate contracts and verify fixture round-trips across languages.
gate-contracts:
    ./tools/codegen/generate.sh
    git diff --exit-code -- crates/clipmill-contracts/src/gen packages/contracts/src/gen workers/sdk/src/clipmill workers/sdk/src/clipmill_worker_sdk/gen
    python3 tools/schema-lint/check.py contracts/schemas/*.json
    cargo test -p clipmill-contracts
    cd workers/sdk && uv run pytest tests/test_contracts.py tests/test_speech_contracts.py tests/test_shots_contracts.py tests/test_index_contracts.py tests/test_discovery_contracts.py tests/test_ranking_contracts.py tests/test_editorial_contracts.py
    pnpm --filter @clipmill/contracts test

# Verify acknowledged project mutations and task recovery after forced termination.
gate-kill:
    ./tools/drills/kill-drill.sh 50

# Verify artifact publication and cache integrity after forced termination.
gate-cache:
    ./tools/drills/cache-drill.sh 50

# Check pinned probing, source timing, cache identity, and hostile input refusal.
gate-media:
    ./tools/drills/media-drill.sh 1

# Check worker authentication, durable completion, cancellation, and recovery.
gate-workers:
    ./tools/drills/worker-drill.sh 50

# Verify edit inversion, command replay, durable edits, and immutable snapshots.
gate-ir:
    ./tools/drills/ir-drill.sh 1

# Verify ingest derivatives, deterministic caching, and interrupted-job recovery.
gate-ingest:
    ./tools/drills/ingest-drill.sh 1

# Check registered stages, device admission, artifact validation, and model policy.
gate-worker2:
    ./tools/drills/worker2-drill.sh 1

# Check speech models against timed fixtures and byte-identical repeat output.
gate-speech iterations="1":
    ./tools/drills/speech-drill.sh {{iterations}}

# Measure accelerated speech selection and alignment, then sign the result.
# Keep the signing key private; only the public key and signed report are committed.
gate-asr-mlx signing_key output_dir="models/attestations/mlx-selection":
    ./tools/drills/asr-mlx-drill.sh --signing-key "{{signing_key}}" --output-dir "{{output_dir}}"

# Check shot detection against known cuts, motion, flashes, and invalid proxies.
gate-shots iterations="1":
    ./tools/drills/shots-drill.sh {{iterations}}

# Check transcript structure, word coverage, reviewed goldens, and determinism.
gate-evidence iterations="1":
    ./tools/drills/evidence-drill.sh {{iterations}}

# Check proposals, legal boundaries, deduplication, and deterministic discovery.
gate-discovery iterations="1":
    ./tools/drills/discovery-drill.sh {{iterations}}

# Editorial contracts and failure cases; needs no model weights.
gate-editorial-unit iterations="1":
    ./tools/drills/editorial-drill.sh {{iterations}}

# Real pinned Qwen, daemon, workers, review, and immutable MP4/SRT/VTT export.
# Uses the existing synthesized talk (created by gate-milestone-1), not a benchmark.
gate-editorial iterations="1":
    ./tools/drills/editorial-drill.sh {{iterations}}
    cargo build -p clipmilld --bin clipmilld
    workers/editorial/.venv/bin/python tools/drills/editorial-live.py

# Verify ranking, boundary optimization, analysis dependencies, and restart recovery.
# Requires pinned FFmpeg sidecars and the shots worker environment.
gate-ranking iterations="1":
    ./tools/drills/ranking-drill.sh {{iterations}}

# Verify rendered captions, sidecars, loudness, digests, cache identity, and recovery.
gate-render:
    ./tools/drills/render-drill.sh 1

# Check device measurements, bounded probes, attestation, and profile caching.
gate-device:
    ./tools/drills/device-drill.sh 1

# Check signed public fixtures, source maps, device profiles, and artifact integrity.
gate-eval-smoke:
    ./tools/drills/eval-smoke.sh 1

# Evaluate private rights-cleared media. Only the output report is safe to commit;
# keep source media, license records, the full manifest, and signing keys private.
gate-seed40 corpus_dir manifest license_attestation signing_key output_dir="eval/seed40" corpus_public_key="":
    ./tools/drills/seed40-drill.sh "{{corpus_dir}}" "{{manifest}}" "{{license_attestation}}" "{{signing_key}}" "{{output_dir}}" "{{corpus_public_key}}"

# Run offline tests in a denied-network container and verify the egress canary.
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

# Run security, dependency-license, and supply-chain policy checks.
gate-security:
    ./tools/security/security-gate.sh

# Check reproducible token CSS and the renderer types, tests, and build.
gate-tokens:
    pnpm --filter @clipmill/tokens check-drift
    pnpm --filter @clipmill/tokens test
    pnpm --filter @clipmill/desktop typecheck
    pnpm --filter @clipmill/desktop test
    pnpm --filter @clipmill/desktop build

# Verify native daemon queries and disconnect reporting after forced termination.
gate-shell:
    cargo build -p clipmilld
    cargo test -p clipmill-shell --test daemon_link -- --ignored --nocapture

# Check native import, probe, job events, documents, media ranges, and access refusals.
gate-shell-pipeline iterations="1":
    ./tools/drills/shell-drill.sh {{iterations}}

# Check crop solver goldens, projection constraints, and face detector determinism.
gate-reframe iterations="1":
    ./tools/drills/reframe-drill.sh {{iterations}}

# Check cue boundaries, reading speed, sidecars, and render-writer round-trips.
gate-captions iterations="1":
    ./tools/drills/captions-drill.sh {{iterations}}

# Check editorial decisions, boundary snapping, persistence, and result joins.
gate-inspector iterations="1":
    ./tools/drills/inspector-drill.sh {{iterations}}

# Compare preview crops and captions with the render presentation.
gate-editor iterations="1":
    ./tools/drills/editor-drill.sh {{iterations}}

# Check export validation, naming, and archive round-trips.
gate-export iterations="1":
    ./tools/drills/export-drill.sh {{iterations}}

# Exercise selected-clip edits, restart recovery, and decoded export through the host.
# Requires macOS speech synthesis, worker environments, and model weights.
gate-milestone-1:
    ./tools/drills/milestone-1-drill.sh

# Check recall metrics against known values and a recording with planted moments.
gate-recall-smoke:
    ./tools/drills/recall-drill.sh

# Evaluate private media and annotations and produce a signed recall report.
# The default corpus bar must already exist; use bar="" for the first measurement.
# Paths resolve from the caller's directory, not from eval/harness.
gate-recall corpus_dir manifest license_attestation annotations socket output bar="eval/recall/corpus-bar.json" public_key="":
    uv run --project eval/harness clipmill-eval recall \
      --corpus-dir {{corpus_dir}} --manifest {{manifest}} \
      --license-attestation {{license_attestation}} \
      --annotations {{annotations}} --socket {{socket}} \
      --output {{output}} \
      {{ if bar == "" { "" } else { "--bar " + bar } }} \
      {{ if public_key == "" { "" } else { "--public-key " + public_key } }}

# Measure render throughput on the reference device and attest the result.
gate-render-slo minimum="1.5":
    ./tools/drills/render-slo.sh {{minimum}}

# Run the stage golden tests together at the current revision.
gate-golden:
    ./tools/drills/gate-golden.sh

# Run the foundation checks and verify committed private-run evidence.
# Re-running gate-seed40 requires private rights-holder inputs.
gate-phase0: gate-contracts gate-kill gate-cache gate-media gate-workers gate-device gate-eval-smoke gate-tokens gate-shell gate-security gate-lock
    ./tools/drills/verify-phase0-attestation.sh

# Run analysis, clip direction, and rendering with workers in a denied namespace.
# Fetch weights and sidecars beforehand; this drill never downloads them.
gate-lock-phase1:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v docker >/dev/null 2>&1; then
        docker run --rm --network=none -v "$PWD":/w -w /w rust:1.96 ./tools/drills/lock-phase1.sh
    else
        echo "gate-lock-phase1: docker not found - install Docker/OrbStack, or rely on"
        echo "the lock-phase1 CI job (the authoritative Linux-namespace gate)."
        exit 1
    fi

# Run the full processing checks and verify committed private-run evidence.
# Recall, render throughput, and accelerated speech measurements require their
# original media or hardware and are verified here through committed reports.
gate-phase1: gate-phase0 gate-ingest gate-ir gate-render gate-worker2 gate-speech gate-shots gate-evidence gate-discovery gate-ranking gate-reframe gate-captions gate-inspector gate-editor gate-export gate-golden gate-recall-smoke gate-lock-phase1
    ./tools/drills/verify-phase1-attestation.sh

# Build the daemon, enroll workers, and launch the native application.
# Explicit sidecar and registry paths keep spawned processes independent of cwd.
# Enrollment precedes startup because the daemon reads its trust store once.
# Start processing with just workers in a second terminal.
app:
    # Build the daemon explicitly; tauri dev only rebuilds the shell.
    cargo build -p clipmilld --bin clipmilld
    ./tools/run-workers.sh --enrol-only
    CLIPMILL_FFPROBE="{{justfile_directory()}}/.cache/bin/ffprobe" \
      CLIPMILL_MODELS_DIR="{{justfile_directory()}}/models/registry" \
      pnpm --filter @clipmill/desktop tauri dev

# Start the model workers against a running daemon. Ctrl-C stops them all.
workers:
    ./tools/run-workers.sh
