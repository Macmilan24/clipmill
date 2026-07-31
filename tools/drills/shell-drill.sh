#!/usr/bin/env bash
# W22 shell data-plane gate.
#
# The screens sit on a data path that had never been exercised end to end. This
# runs it out of process against a real daemon: a real file is encoded by the
# pinned FFmpeg, registered and probed, submitted as an analysis, watched as it
# moves, read back through the document door, and streamed through the same
# protocol handler the WebView addresses — including a byte range, which is what
# a player seeking actually sends.
#
# It also asserts the refusals, because a door is only as good as what it will
# not open: a kind nobody put on the media list, an artifact belonging to another
# project, and a file the artifact's own descriptor never named.
#
# The run is not waited out. The stages after ingest need worker processes this
# drill does not start; what is proven is everything the shell reads before them.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "shell-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "shell-drill: iterations must be at least 1" >&2
  exit 2
fi
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "shell-drill: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done

echo "==> building the daemon the shell talks to"
cargo build -p clipmilld

echo "==> shell data plane ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "shell-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmill-shell -- --ignored --nocapture
done
echo "shell-drill: OK ($ITERATIONS iterations; import, probe, transitions, documents, ranged media, four refusals)"
