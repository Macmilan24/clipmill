#!/usr/bin/env bash
# W26 recall gate, synthetic half.
#
# Every other test in this repository checks that a candidate set is well
# formed. None of them checks that it found anything worth finding, because on
# a real recording nobody here can say what the right answer is.
#
# So the right answer is planted. `clipmill-discovery` builds a recording whose
# three moments are known because they were put there — a question with its
# answer, a claim carrying numbers, a topic that opens and closes — runs the
# real mesh, lattice, scorer, boundary optimizer and selector over it, and
# leaves the documents on disk. This scores them with the harness, which owns
# the only implementation of the recall arithmetic.
#
# The split matters. The engine is Rust and the metric is Python, and neither
# knows the other's answer: a bug that made discovery return nothing would fail
# here, and so would a metric that reported recall for moments nobody found.
#
# What this cannot tell you is whether the ranking is any good on a real
# recording. That needs an editor's annotations and the private corpus, which
# is `gate-recall`. This is the floor, and the floor is that a moment placed in
# front of the engine on purpose comes back out.
set -euo pipefail
cd "$(dirname "$0")/../.."

WORK="target/recall-smoke"
BAR="eval/recall/planted-bar.json"

echo "==> the metric itself, against numbers worked out by hand"
uv sync --offline --frozen --project eval/harness --quiet
(cd eval/harness && uv run pytest tests/test_recall.py tests/test_annotation.py tests/test_fetch.py -q)

echo "==> the engine over a recording whose answers were written down first"
rm -rf "$WORK"
cargo test --quiet -p clipmill-discovery planted

for name in candidates.json ranking.json annotation.json; do
  if [ ! -f "$WORK/$name" ]; then
    echo "recall-drill: the planted run left no $name to score" >&2
    exit 1
  fi
done

echo "==> scoring, by the code that owns the arithmetic"
(cd eval/harness && uv run clipmill-eval score-recall \
  --annotation "../../$WORK/annotation.json" \
  --candidates "../../$WORK/candidates.json" \
  --ranking "../../$WORK/ranking.json" \
  --output "../../$WORK/report.json" \
  --bar "../../$BAR")

echo "==> the same recording scores the same twice"
cargo test --quiet -p clipmill-discovery planted
(cd eval/harness && uv run clipmill-eval score-recall \
  --annotation "../../$WORK/annotation.json" \
  --candidates "../../$WORK/candidates.json" \
  --ranking "../../$WORK/ranking.json" \
  --output "../../$WORK/report-again.json" \
  --bar "../../$BAR" >/dev/null)
if ! cmp -s "$WORK/report.json" "$WORK/report-again.json"; then
  echo "recall-drill: two runs over one recording reported different numbers" >&2
  exit 1
fi

echo "recall-drill: OK (metric arithmetic, planted moments recalled end to end, report reproducible)"
