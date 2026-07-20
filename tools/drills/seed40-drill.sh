#!/usr/bin/env bash
# W8 private Seed-40 evaluation. Media, full manifests, and the private key stay
# outside Git; only the path-free signed proof is written to the public output.
set -euo pipefail
cd "$(dirname "$0")/../.."

usage() {
  echo "usage: $0 CORPUS_DIR MANIFEST LICENSE_ATTESTATION SIGNING_KEY OUTPUT_DIR [CORPUS_PUBLIC_KEY]" >&2
  exit 2
}

[ "$#" -ge 5 ] && [ "$#" -le 6 ] || usage
corpus_dir="$1"
manifest_path="$2"
license_path="$3"
signing_key="$4"
output_dir="$5"
corpus_public_key="${6:-}"

[ -d "$corpus_dir" ] || { echo "seed40: corpus directory is missing" >&2; exit 2; }
[ -f "$manifest_path" ] || { echo "seed40: signed manifest is missing" >&2; exit 2; }
[ -f "$license_path" ] || { echo "seed40: license attestation is missing" >&2; exit 2; }
[ -f "$signing_key" ] || { echo "seed40: private signing key is missing" >&2; exit 2; }
if [ -n "$corpus_public_key" ] && [ ! -f "$corpus_public_key" ]; then
  echo "seed40: corpus verification key is missing" >&2
  exit 2
fi
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "seed40: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done

uv sync --frozen --project eval/harness --quiet
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
      /tmp/clipmill-seed40.*) rm -rf -- "$drill_root" ;;
      *) echo "seed40: refusing to remove unexpected path $drill_root" >&2 ;;
    esac
  fi
}
trap cleanup EXIT INT TERM

drill_root="$(mktemp -d /tmp/clipmill-seed40.XXXXXX)"
data_dir="$drill_root/data"
socket_path="$drill_root/daemon.sock"
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
    echo "seed40: daemon exited before becoming ready" >&2
    sed -n '1,160p' "$log_path" >&2
    exit 1
  fi
  sleep 0.05
done
if [ "$ready" != true ]; then
  echo "seed40: daemon did not become ready" >&2
  sed -n '1,160p' "$log_path" >&2
  exit 1
fi

command=(
  uv run --frozen --project eval/harness clipmill-eval seed40
  --socket "$socket_path"
  --data-dir "$data_dir"
  --corpus-dir "$corpus_dir"
  --manifest "$manifest_path"
  --license-attestation "$license_path"
  --signing-key "$signing_key"
  --output-dir "$output_dir"
)
if [ -n "$corpus_public_key" ]; then
  command+=(--public-key "$corpus_public_key")
fi
"${command[@]}"

kill -TERM "$daemon_pid"
wait "$daemon_pid"
daemon_pid=""
if [ -e "$socket_path" ]; then
  echo "seed40: clean shutdown left the control socket behind" >&2
  exit 1
fi
uv run --frozen --project eval/harness clipmill-eval verify-attestation \
  --attestation-dir "$output_dir"
echo "seed40: OK (40 verified items; cold/warm evidence; signed path-free public proof)"
