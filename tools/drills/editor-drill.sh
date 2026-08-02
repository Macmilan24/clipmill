#!/usr/bin/env bash
# W24 editor gate: preview parity.
#
# The player tells a creator what the export will look like. If it is wrong they
# find out after publishing, which is the worst possible moment and the reason
# this gate exists.
#
# What it checks is not that the preview matches a stored expectation — that
# would catch a change. It checks that the preview matches *the renderer*, frame
# by frame, because the only failure worth blocking a release for is the two of
# them disagreeing: a different word visible at a frame, a different crop
# rectangle, a cue arriving on a different frame, a different word carrying the
# highlight.
#
# The tolerances that are allowed — proxy resolution, antialiasing, a couple of
# pixels of text position — are written down in docs/preview-parity.md rather
# than left to whoever is looking. A tolerance nobody wrote down is a divergence
# nobody can argue about.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "editor-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "editor-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> the preview plan against the renderer that produces the file"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "editor-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmill-render preview::
done

echo "==> the subtitle writers, whose sweep the plan reads"
cargo test -p clipmill-render subtitles::
cargo test -p clipmill-render --test subtitles_round_trip

echo "==> the RPC the player fetches its plan from"
cargo test -p clipmilld --lib service::

echo "==> every edit is a command the daemon can replay and undo"
echo "==> the player applies the plan and computes nothing"
pnpm --filter @clipmill/desktop test

echo "editor-drill: OK ($ITERATIONS iterations; per-frame crops, cue windows, visible words, karaoke holds — all against the renderer)"
