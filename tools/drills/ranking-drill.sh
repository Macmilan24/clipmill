#!/usr/bin/env bash
# W19 ranking gate: what a clip is worth, where it is cut, which to show, and the
# one job that produces all of it.
#
#   The score card, against cards written by hand. An axis nobody measured must
#   be distinguishable from one measured at zero — the whole reason the score is
#   decomposed — so the unmeasured cases are written out rather than waited for.
#
#   J against brute force. The boundary optimizer is exhaustive by design, so
#   the property is checkable directly: the chosen pair is the argmax over every
#   legal pair, and the runner-up is the second, with nothing scoring between.
#
#   Selection, against the temptation to pad. Asked for more clips than a
#   recording holds, the set comes back short with the difference accounted for.
#
#   The published contract, over the fixtures every language reads, and the
#   goldens that say which clips this system would actually show.
#
#   The analyze DAG, against a real daemon and a real worker. Probe, ingest, shot
#   detection, and the fan-in that roots them: the plan is accepted, the addresses
#   the plan declared reach a worker's lease, the manifest names every stage that
#   ran and accounts for the ones it skipped, a warm re-submit resolves to the
#   same identities, and a killed daemon finishes inside the 30-second SLO.
#
#   What that last part does not cover: the speech half. A recording with audio
#   needs the three pinned speech models and a worker fleet no drill starts, which
#   is W26's harness. The chain itself is covered end to end by `gate-speech`, and
#   the stages that read a transcript are covered by the goldens above.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "ranking-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "ranking-drill: iterations must be at least 1" >&2
  exit 2
fi

# What the goldens looked like before this run. Compared afterwards rather than
# diffed against git, because the question is whether *this run* rewrote them —
# not whether the working tree happens to be dirty, which it legitimately is
# while somebody is changing what the goldens should say.
fingerprint() {
  find "$1" -type f -name '*.json' -print0 | sort -z | xargs -0 shasum -a 256 2>/dev/null | shasum -a 256
}
goldens_before="$(fingerprint contracts/fixtures/ranking.set)"

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

# A filter that matches nothing still exits zero.
ran() {
  if ! grep -qE "test result: ok\. [1-9]" "$log"; then
    echo "ranking-drill: a test filter selected nothing" >&2
    cat "$log" >&2
    exit 1
  fi
}

echo "==> the card, the optimizer, and the selector"
for filter in scorecard:: boundary:: ranking::; do
  cargo test -p clipmill-discovery --lib "$filter" -- --nocapture | tee "$log"
  ran
done

echo "==> registration, keying, and the shape of the DAG"
for filter in recipes:: ranking:: analyze_tests::; do
  cargo test -p clipmilld --lib "$filter" -- --nocapture | tee "$log"
  ran
done

echo "==> the published contract, in three languages"
for suite in ranking_contracts analysis_contracts; do
  cargo test -p clipmill-contracts --test "$suite" | tee "$log"
  ran
done
(cd workers/sdk && uv run pytest -q \
  tests/test_ranking_contracts.py tests/test_analysis_contracts.py tests/test_inputs.py)
pnpm --filter @clipmill/contracts test

echo "==> ranking over every published cohort ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "ranking-drill: iteration $iteration/$ITERATIONS"
  # Deliberately not blessed here. A gate that regenerated its own goldens
  # would pass for any behaviour at all.
  cargo test -p clipmill-discovery --test ranking_golden -- --nocapture | tee "$log"
  ran
done

# A gate that regenerated its own goldens would pass for any behaviour at all,
# so the one thing the tests above cannot assert about themselves is asserted
# here: nothing this run did changed the files it was checking against.
if [ "$(fingerprint contracts/fixtures/ranking.set)" != "$goldens_before" ]; then
  echo "ranking-drill: this run rewrote the committed ranking goldens" >&2
  git --no-pager diff --stat -- contracts/fixtures/ranking.set >&2
  exit 1
fi

# The end-to-end half needs a decoder and a worker environment. Verified rather
# than fetched: acquisition happens outside the Local Lock, so a machine without
# them is one this drill cannot speak for.
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "ranking-drill: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done
if [ ! -x workers/shots/.venv/bin/clipmill-worker-shots ]; then
  echo "ranking-drill: the shots worker is not built; run 'uv sync --project workers/shots'" >&2
  exit 2
fi

echo "==> the analyze DAG, over a real daemon ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "ranking-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmilld --test analyze_dag -- --ignored --nocapture --test-threads=1 | tee "$log"
  ran
done

echo "ranking-drill: OK ($ITERATIONS iterations; card, boundaries, selection, goldens, analyze DAG)"
