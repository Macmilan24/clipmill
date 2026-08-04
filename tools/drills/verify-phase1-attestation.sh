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
# gate, not a present file.
#
# What this refuses is as important as what it accepts. A missing report is
# named, with the command that produces it, because "Phase 1 is done" said over
# an absent measurement is the one claim this whole register exists to prevent.
set -euo pipefail
cd "$(dirname "$0")/../.."

RECALL_REPORT="${1:-eval/recall/attestation/recall.json}"
SLO_REPORT="${2:-eval/render-slo/render-slo.json}"

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
if require_committed "$RECALL_REPORT" "recall report" \
  "just gate-recall <corpus> <manifest> <licences> <annotations> <socket> $RECALL_REPORT"; then
  python3 - "$RECALL_REPORT" eval/recall/planted-bar.json <<'PY'
import json
import sys

report = json.loads(open(sys.argv[1], encoding="utf-8").read())
bar = json.loads(open(sys.argv[2], encoding="utf-8").read())
if report.get("schema_version") != "clipmill.eval.recall.v1":
    raise SystemExit("phase1-attestation: the recall report is not a recall report")

# Compared against the committed bar rather than a number written here, so
# ratcheting the bar is a reviewed change to one file and not an edit to a
# gate. Gates over dates.
checks = [
    ("recall", report.get("recall"), bar.get("recall"), "at least"),
    ("multi_moment_recall", report.get("multi_moment_recall"),
     bar.get("multi_moment_recall"), "at least"),
    ("duplicate_rate", report.get("duplicate_rate"), bar.get("duplicate_rate"), "at most"),
]
for name, measured, required, direction in checks:
    if required is None:
        continue
    if measured is None:
        raise SystemExit(f"phase1-attestation: the recall report omits {name}")
    ok = measured >= required if direction == "at least" else measured <= required
    if not ok:
        raise SystemExit(
            f"phase1-attestation: {name} is {measured}, and the bar is {direction} {required}"
        )
print(
    f"    recall {report['recall']:.3f}, duplicate rate {report['duplicate_rate']:.3f}"
    f" over {report['moments_total']} moments"
)
PY
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
  exit 1
fi

echo "phase1-attestation: OK (Seed-40, MLX selection, recall and the render SLO are all committed and meet their bars)"
