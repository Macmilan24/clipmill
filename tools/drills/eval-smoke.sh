#!/usr/bin/env bash
# W7 signed public-corpus evaluation against a real daemon and CAS.
set -euo pipefail
cd "$(dirname "$0")/../.."

iterations="${1:-1}"
if ! [[ "$iterations" =~ ^[1-9][0-9]*$ ]]; then
  echo "eval-smoke: iterations must be a positive integer" >&2
  exit 2
fi
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "eval-smoke: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done

uv sync --offline --frozen --project eval/harness --quiet
cargo build --quiet -p clipmilld --bin clipmilld

drill_root=""
daemon_pid=""
cleanup() {
  if [ -n "$daemon_pid" ] && kill -0 "$daemon_pid" 2>/dev/null; then
    kill -TERM "$daemon_pid" 2>/dev/null || true
    wait "$daemon_pid" 2>/dev/null || true
  fi
  if [ -n "$drill_root" ]; then
    case "$drill_root" in
      /tmp/clipmill-eval.*) rm -rf -- "$drill_root" ;;
      *) echo "eval-smoke: refusing to remove unexpected path $drill_root" >&2 ;;
    esac
  fi
}
trap cleanup EXIT INT TERM

echo "==> signed public evaluation smoke ($iterations iterations)"
for iteration in $(seq 1 "$iterations"); do
  echo "eval-smoke: iteration $iteration/$iterations"
  drill_root="$(mktemp -d /tmp/clipmill-eval.XXXXXX)"
  data_dir="$drill_root/data"
  socket_path="$drill_root/daemon.sock"
  output_path="$drill_root/run-manifest.json"
  log_path="$drill_root/daemon.log"

  RUST_LOG=warn target/debug/clipmilld \
    --data-dir "$data_dir" \
    --socket "$socket_path" \
    --ffprobe "$PWD/.cache/bin/ffprobe" \
    >"$log_path" 2>&1 &
  daemon_pid=$!

  ready=false
  for _attempt in $(seq 1 300); do
    if [ -S "$socket_path" ]; then
      ready=true
      break
    fi
    if ! kill -0 "$daemon_pid" 2>/dev/null; then
      echo "eval-smoke: daemon exited before becoming ready" >&2
      sed -n '1,160p' "$log_path" >&2
      exit 1
    fi
    sleep 0.05
  done
  if [ "$ready" != true ]; then
    echo "eval-smoke: daemon did not become ready" >&2
    sed -n '1,160p' "$log_path" >&2
    exit 1
  fi

  uv run --offline --frozen --project eval/harness clipmill-eval smoke \
    --socket "$socket_path" \
    --data-dir "$data_dir" \
    --ffmpeg "$PWD/.cache/bin/ffmpeg" \
    --work-dir "$drill_root/corpus" \
    --output "$output_path"

  python3 - "$output_path" "$drill_root" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
private_root = sys.argv[2]
raw = manifest_path.read_text(encoding="utf-8")
manifest = json.loads(raw)
if manifest.get("schema_version") != "clipmill.eval.run.v1":
    raise SystemExit("eval-smoke: run manifest schema is invalid")
items = manifest.get("items")
if not isinstance(items, list) or len(items) != 5:
    raise SystemExit("eval-smoke: public corpus did not contain exactly five items")
successful = [item for item in items if item.get("observed_result") == "success"]
hostile = [item for item in items if item.get("observed_result") == "structured_failure"]
if len(successful) != 4 or len(hostile) != 1:
    raise SystemExit("eval-smoke: public corpus outcomes are incomplete")
if any(item.get("warm_cache_hit") is not True for item in successful):
    raise SystemExit("eval-smoke: a warm source-map run was not a cache hit")
if any(
    item.get("source_map_artifact_id") != item.get("cold_source_map_artifact_id")
    or item.get("source_map_artifact_id") != item.get("warm_source_map_artifact_id")
    for item in successful
):
    raise SystemExit("eval-smoke: a warm source-map artifact ID changed")
if any(item.get("warm_observed_result") != "structured_failure" for item in hostile):
    raise SystemExit("eval-smoke: hostile media did not repeat its structured failure")
if private_root in raw or '"absolute_path"' in raw:
    raise SystemExit("eval-smoke: run manifest leaked a private media path")
PY

  kill -TERM "$daemon_pid"
  wait "$daemon_pid"
  daemon_pid=""
  if [ -e "$socket_path" ]; then
    echo "eval-smoke: clean shutdown left the control socket behind" >&2
    exit 1
  fi
  rm -rf -- "$drill_root"
  drill_root=""
done
echo "eval-smoke: OK ($iterations iterations; signed corpus/profile, cold/warm CAS, hostile input)"
