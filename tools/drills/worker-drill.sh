#!/usr/bin/env bash
# Authenticated worker, shared-memory, response-loss, and hard-kill recovery.
set -euo pipefail
cd "$(dirname "$0")/../.."

iterations="${1:-50}"
if ! [[ "$iterations" =~ ^[1-9][0-9]*$ ]]; then
  echo "worker-drill: iterations must be a positive integer" >&2
  exit 2
fi

echo "==> prepare locked echo-worker environment"
uv sync --offline --frozen --project workers/echo --quiet

echo "==> authenticated shared memory and durable response-loss replay"
cargo test --quiet -p clipmilld --test worker_recovery \
  authenticated_echo_worker_completes_dag_and_replays_lost_ack \
  -- --ignored --nocapture --test-threads=1

for iteration in $(seq 1 "$iterations"); do
  echo "==> worker recovery hard-kill iteration $iteration/$iterations"
  cargo test --quiet -p clipmilld --test worker_recovery \
    worker_and_daemon_death_recover_without_partial_outputs \
    -- --ignored --nocapture --test-threads=1
done

echo "worker-drill: OK ($iterations hard-kill iterations)"
