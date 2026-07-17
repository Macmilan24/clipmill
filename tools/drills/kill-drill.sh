#!/usr/bin/env bash
# Kill drill (Phase 0 exit gate 1, ch. 10 recovery contract).
#
# Drives the real clipmilld binary: start it, SIGKILL it at a random point,
# restart it, and assert (a) the restart always comes up healthy and (b) no
# stray files are left behind in the data directory.
#
# Today clipmilld is a stub that prints and exits, so the drill proves the
# harness itself: launch/kill/relaunch loops leave a clean world. As W2-W4
# land (SQLite project state, CAS staging, task leases), this script gains
# the 30-second consistency assertions the book's contract box demands —
# the CI wiring never changes.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-25}"
DATA_DIR="$(mktemp -d)"
trap 'rm -rf "$DATA_DIR"' EXIT

cargo build -p clipmilld --quiet
BIN=target/debug/clipmilld

for i in $(seq 1 "$ITERATIONS"); do
  CLIPMILL_DATA_DIR="$DATA_DIR" "$BIN" >/dev/null 2>&1 &
  pid=$!
  # Kill at a random early moment (0-50ms) to catch mid-startup states.
  sleep "0.0$((RANDOM % 5))"
  kill -9 "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true

  # Restart must always succeed after a hard kill.
  if ! CLIPMILL_DATA_DIR="$DATA_DIR" "$BIN" >/dev/null 2>&1; then
    echo "kill-drill: FAILED at iteration $i - restart after SIGKILL did not exit cleanly" >&2
    exit 1
  fi

  # No partial/stray files may survive (staging must be atomic or quarantined).
  strays=$(find "$DATA_DIR" -name '*.tmp' -o -name '*.partial' | wc -l | tr -d ' ')
  if [ "$strays" != "0" ]; then
    echo "kill-drill: FAILED at iteration $i - $strays stray staging file(s) left behind" >&2
    find "$DATA_DIR" -name '*.tmp' -o -name '*.partial' >&2
    exit 1
  fi
done

echo "kill-drill: OK ($ITERATIONS iterations, clean restarts, no stray files)"
