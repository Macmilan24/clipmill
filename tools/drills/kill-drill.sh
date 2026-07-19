#!/usr/bin/env bash
# W4 hard-kill drill (Phase 0 recovery harness, book ch. 10).
#
# The Rust integration test drives the real protobuf socket, commits at least
# one project and durable demo job per iteration, rotates termination across
# submission, running, intermediate-publication, acknowledged-output, and
# randomized request boundaries, then restarts against the same WAL/CAS state.
# Every acknowledged job must reach a verified terminal output within thirty
# seconds. Startup also runs quick_check and removes stale sockets.
#
# W3 cache-drill remains the exhaustive CAS publication drill. This test adds
# the W4 task-lease and interrupted-job recovery guarantee.
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
    acknowledged_jobs_and_projects_survive_random_hard_kills -- --ignored --exact --nocapture
echo "kill-drill: OK ($ITERATIONS iterations; acknowledged jobs, projects, and outputs recovered)"
