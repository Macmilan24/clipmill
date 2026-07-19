#!/usr/bin/env bash
# W3 hard-kill cache drill (Phase 0 recovery harness, book ch. 10).
#
# Each child process publishes real payload bytes into the filesystem CAS,
# attaches the resulting artifact root to a project in SQLite, and writes a
# durable acknowledgement only after both commits succeed. The parent races
# later publications against SIGKILL. A final daemon restart verifies every
# acknowledged root and payload, scans every visible object, and confirms that
# interrupted staging directories were quarantined.
#
# This proves W3 artifact publication and cache recovery. Task leases and
# interrupted-job recovery remain W4 work.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-25}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "cache-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "cache-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> artifact CAS hard-kill recovery ($ITERATIONS iterations)"
CLIPMILL_CACHE_ITERATIONS="$ITERATIONS" \
  cargo test -p clipmilld --test cache_recovery \
    acknowledged_artifacts_survive_random_hard_kills -- --ignored --exact --nocapture
echo "cache-drill: OK ($ITERATIONS iterations; acknowledged roots and payloads verified)"
