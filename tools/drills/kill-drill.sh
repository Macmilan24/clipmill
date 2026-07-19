#!/usr/bin/env bash
# W2 hard-kill drill (Phase 0 recovery harness, book ch. 10).
#
# The Rust integration test drives the real protobuf socket, commits at least
# one project mutation per iteration, races another mutation against SIGKILL,
# restarts clipmilld against the same WAL database, and checks that every
# acknowledged mutation remains present. Startup also runs quick_check and
# removes the stale socket left by SIGKILL.
#
# This proves W2 project-state durability. Task leases, partial artifact
# quarantine, and interrupted-job recovery are still W3/W4 work; the Phase 0
# recovery gate is not complete until those assertions join this drill.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-25}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "kill-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "kill-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> clipmilld hard-kill recovery ($ITERATIONS iterations)"
CLIPMILL_KILL_ITERATIONS="$ITERATIONS" \
  cargo test -p clipmilld --test kill_recovery \
    acknowledged_projects_survive_random_hard_kills -- --ignored --exact --nocapture
echo "kill-drill: OK ($ITERATIONS iterations; acknowledged projects survived)"
