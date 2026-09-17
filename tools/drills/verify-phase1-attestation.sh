#!/usr/bin/env bash
# Verify the committed evidence for every Phase 1 gate a runner cannot run.
#
# Three of them need something no hosted machine has. Recall needs an annotated
# corpus whose media may not enter Git. The render SLO needs the reference Mac,
# because a ratio measured on a shared runner is a number about the runner. The
# accelerated speech selection needs an Apple GPU. So what CI checks is their
# evidence, exactly as it checks Seed-40's — the claim travels, the media does
# not.
#
# The two signed proofs verify themselves through their own scripts. The two
# reports this adds are checked here: committed, the schema they claim, and the
# number they were held to. A report that exists but misses its bar is a failed
# gate, not a present file — and the comparison is delegated to the harness that
# produced the report rather than restated, because a gate holding its own
# opinion about what a bar means is a second implementation of the metric.
#
# What this refuses is as important as what it accepts. A missing report is
# named, with the command that produces it, because "Phase 1 is done" said over
# an absent measurement is the one claim this whole register exists to prevent.
# A bar that constrains nothing is refused for the same reason: a check that
# cannot fail is indistinguishable from a check nobody wrote.
set -euo pipefail
cd "$(dirname "$0")/../.."

RECALL_REPORT="${1:-eval/recall/attestation/recall.json}"
SLO_REPORT="${2:-eval/render-slo/render-slo.json}"
# The bar the *corpus* is held to, which is not the one the synthetic smoke is
# held to. `planted-bar.json` requires perfect recall because its moments were
# planted, and holding a real recording to it would be demanding that a
# model-free proposer never miss anything a person thought was worth keeping.
RECALL_BAR="${3:-eval/recall/corpus-bar.json}"

missing=0

# A committed file, and genuinely a file: a symlink here could point at
# something outside the repository that no reviewer ever saw.
require_committed() {
  local path=$1 what=$2 how=$3
  if [ ! -f "$path" ] || [ -L "$path" ]; then
    echo "phase1-attestation: no $what at $path" >&2
    echo "    produce it with: $how" >&2
    missing=1
    return 1
  fi
  if ! git ls-files --error-unmatch "$path" >/dev/null 2>&1; then
    echo "phase1-attestation: $path is not committed to Git" >&2
    echo "    evidence nobody can review is not evidence" >&2
    missing=1
    return 1
  fi
  return 0
}

echo "==> the signed proofs, through their own verifiers"
./tools/drills/verify-phase0-attestation.sh
./tools/drills/verify-mlx-attestation.sh

echo "==> recall, against the bar it was held to"
# Both the measurement and the bar are committed files, so ratcheting is a
# reviewed change to one of them and never an edit to a gate. Gates over dates.
#
# The bar is required to exist rather than defaulted to something lenient. It
# cannot be written before the corpus is annotated and measured once — that
# first number *is* the bar — so its absence is the honest state, and a gate
# that quietly substituted the synthetic bar would be answering a question
# about real footage with a number about planted moments.
recall_ready=0
require_committed "$RECALL_REPORT" "recall report" \
  "just gate-recall <corpus> <manifest> <licences> <annotations> <socket> $RECALL_REPORT" \
  || recall_ready=1
require_committed "$RECALL_BAR" "corpus recall bar" \
  "measure the corpus once, then commit that result as the bar it ratchets from" \
  || recall_ready=1
if [ "$recall_ready" -eq 0 ]; then
  # Through the harness that measured it, so the exit gate and the run apply
  # one implementation of "does this report meet this bar".
  uv run --offline --frozen --project eval/harness clipmill-eval check-recall \
    --report "$RECALL_REPORT" --bar "$RECALL_BAR" || missing=1
fi

echo "==> the render SLO, on the machine that could measure it"
if require_committed "$SLO_REPORT" "render SLO report" \
  "just gate-render-slo   (on the reference Mac, then commit $SLO_REPORT)"; then
  python3 - "$SLO_REPORT" <<'PY'
import json
import sys

report = json.loads(open(sys.argv[1], encoding="utf-8").read())
if report.get("schema_version") != "clipmill.eval.render_slo.v1":
    raise SystemExit("phase1-attestation: the SLO report is not an SLO report")
measured, minimum = report.get("measured_ratio"), report.get("minimum_ratio")
if measured is None or minimum is None:
    raise SystemExit("phase1-attestation: the SLO report states no ratio")
if measured < minimum:
    raise SystemExit(
        f"phase1-attestation: {measured}x real time, and the SLO is {minimum}x"
    )
# The machine is printed because the number means nothing without it.
machine = report.get("machine", {})
print(
    f"    {measured}x real time (SLO {minimum}x) on "
    f"{machine.get('system', '?')} {machine.get('machine', '?')}"
)
PY
fi

if [ "$missing" -ne 0 ]; then
  echo "phase1-attestation: Phase 1 cannot be declared over evidence that is absent" >&2
  echo "    or that falls short of the bar it was held to" >&2
  exit 1
fi

echo "phase1-attestation: OK (Seed-40, MLX selection, recall and the render SLO are all committed and meet their bars)"
