#!/usr/bin/env bash
# W26 render throughput SLO: 1.5x real time at 1080x1920.
#
# The number is a promise about a machine, so it is measured on one and
# attested. CI cannot make this promise — a shared runner's throughput says
# nothing about a laptop — so CI asserts only that the render completed and
# reproduced, and the ratio is asserted here, on hardware whose profile travels
# with the measurement.
#
# Real time is the program's own duration, taken from the render manifest
# rather than from what was asked for: a render that produced a shorter clip
# than intended would otherwise look fast.
set -euo pipefail
cd "$(dirname "$0")/../.."

MINIMUM="${1:-1.5}"
OUTPUT="${2:-target/render-slo}"

case "$MINIMUM" in
  ''|*[!0-9.]*) echo "render-slo: minimum must be a number" >&2; exit 2 ;;
esac
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "render-slo: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done

mkdir -p "$OUTPUT"
echo "==> rendering the first-slice document and timing it"
started="$(python3 -c 'import time; print(time.monotonic_ns())')"
cargo test --quiet -p clipmill-render --test compilation
finished="$(python3 -c 'import time; print(time.monotonic_ns())')"

python3 - "$started" "$finished" "$MINIMUM" "$OUTPUT" <<'PY'
"""Turn the wall clock into a ratio, and refuse to report one nobody measured."""
import json
import pathlib
import platform
import subprocess
import sys

started, finished, minimum, output = int(sys.argv[1]), int(sys.argv[2]), float(sys.argv[3]), pathlib.Path(sys.argv[4])
elapsed_seconds = (finished - started) / 1e9

# The program's own duration, from the manifest the render wrote. Asking the
# document rather than the request is what stops a truncated render looking
# fast.
manifests = sorted(pathlib.Path("target").rglob("render-manifest.json"))
if not manifests:
    raise SystemExit("render-slo: no render manifest was produced; nothing to measure")
manifest = json.loads(manifests[-1].read_text())
duration_ticks = manifest["program"]["duration_ticks"]
program_seconds = duration_ticks / 90_000
if program_seconds <= 0:
    raise SystemExit("render-slo: the render manifest reports no program duration")

ratio = program_seconds / elapsed_seconds
report = {
    "schema_version": "clipmill.eval.render_slo.v1",
    "minimum_ratio": minimum,
    "measured_ratio": round(ratio, 3),
    "program_seconds": round(program_seconds, 3),
    "elapsed_seconds": round(elapsed_seconds, 3),
    "profile": manifest["profile"]["profile_id"],
    "engine": manifest["engine"],
    "machine": {
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
        "processor": platform.processor(),
    },
}
output.mkdir(parents=True, exist_ok=True)
(output / "render-slo.json").write_text(json.dumps(report, indent=2) + "\n")
print(
    f"render-slo: {ratio:.2f}x real time "
    f"({program_seconds:.1f} s of program in {elapsed_seconds:.1f} s) on "
    f"{report['machine']['system']} {report['machine']['machine']}"
)
if ratio < minimum:
    raise SystemExit(f"render-slo: below the {minimum}x floor")
PY

echo "render-slo: OK (report in $OUTPUT/render-slo.json)"
