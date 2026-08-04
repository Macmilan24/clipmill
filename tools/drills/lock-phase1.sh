#!/usr/bin/env bash
# Phase 1 exit gate: the whole product, offline, with a real worker fleet.
#
# Every other Local Lock proof stops before the part that would actually phone
# home. `network-denial.sh` runs the suite and six drills with no network, but
# each of them stops at the worker plane: the analyze gate detects shots on a
# silent video and says so (`analyze_dag.rs` — "a full analysis of a recording
# with speech needs the three pinned speech models and a worker fleet no drill
# currently starts"), the shell gate stops at ingest for the same reason, and
# the speech conformance harness imports the model classes in-process, which
# proves the models work rather than that a lease delivers them.
#
# So the claim "zero hidden cloud dependencies" was true of everything anybody
# had run, and untested on the three stages that load half a gigabyte of
# weights and the one that writes the file a user keeps. This runs those.
#
# The shape is the same as every Lock proof and the order matters:
#
#   1. The egress canary must fail. If it succeeds, the denial is not active
#      and everything after it is theatre, so this aborts rather than reports.
#   2. A recording with real speech goes in. Analyze plans nineteen stages,
#      five of them leased to worker processes that load pinned weights; a clip
#      is directed from what they published; the render compiler writes it out.
#   3. The canary is checked again at the end, because a stage that opened a
#      socket and gave up is still a stage that tried.
#
# Everything acquired from the network is acquired *before* this runs and is
# named as a precondition: the pinned FFmpeg sidecars, the pinned weights, the
# Python environments, and the daemon binary. That split is the point — model
# acquisition is a deliberate act outside the Lock (`fetch-models.sh`), and the
# app is never the thing that downloads. This refuses rather than fetches.
set -euo pipefail
cd "$(dirname "$0")/../.."

WEIGHTS_DIR="${CLIPMILL_WEIGHTS_DIR:-$PWD/.cache/models}"
REGISTRY_DIR="$PWD/models/registry"

fail() { echo "lock-phase1: $*" >&2; exit 1; }
refuse() { echo "lock-phase1: $*" >&2; exit 2; }

# ---- preconditions, refused rather than fetched -----------------------------

for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  [ -x "$tool" ] || refuse "$tool is missing; run ./tools/fetch-ffmpeg.sh outside the namespace"
done
[ -d "$REGISTRY_DIR" ] || refuse "no model registry at $REGISTRY_DIR"
for model in silero-vad whisper-base wav2vec2-ctc-en; do
  [ -d "$WEIGHTS_DIR/$model" ] ||
    refuse "pinned weights for $model are missing; run ./tools/fetch-models.sh $model outside the namespace"
done

# ---- 1. the canary ----------------------------------------------------------

echo "==> egress canary (must fail)"
if curl --silent --max-time 5 --output /dev/null https://example.com 2>/dev/null; then
  echo "lock-phase1: FAILED - the egress canary reached the internet;" >&2
  fail "this drill is not running inside a denied namespace"
fi
echo "canary blocked - denial is active"

echo "==> building the daemon offline"
CARGO_NET_OFFLINE=true cargo build --quiet --offline -p clipmilld --bin clipmilld

# ---- lifecycle --------------------------------------------------------------

drill_root=""
daemon_pid=""
worker_pids=""
cleanup() {
  for pid in $worker_pids; do
    kill -TERM "$pid" 2>/dev/null || true
  done
  if [ -n "$daemon_pid" ] && kill -0 "$daemon_pid" 2>/dev/null; then
    kill -TERM "$daemon_pid" 2>/dev/null || true
    wait "$daemon_pid" 2>/dev/null || true
  fi
  wait 2>/dev/null || true
  if [ -n "$drill_root" ]; then
    case "$drill_root" in
      /tmp/clipmill-lock-phase1.*) rm -rf -- "$drill_root" ;;
      *) echo "lock-phase1: refusing to remove unexpected path $drill_root" >&2 ;;
    esac
  fi
}
trap cleanup EXIT INT TERM

drill_root="$(mktemp -d /tmp/clipmill-lock-phase1.XXXXXX)"
data_dir="$drill_root/data"
log_path="$drill_root/daemon.log"
media_path="$drill_root/spoken.mp4"

# The daemon's own layout, not paths of this drill's choosing. Three sockets
# live in the run directory and the workers find the shared-memory broker as a
# sibling of the worker socket they were given — so a drill that scattered them
# would test a topology no install has, and would fail with "shared memory
# unavailable" for a reason that is its own fault.
run_dir="$data_dir/run"
socket_path="$run_dir/clipmilld.sock"
worker_socket="$run_dir/clipmill-workers.sock"

# ---- 2. a recording with real speech ----------------------------------------

# Speech the recognizer can actually be asked about, and video with a cut in
# it. The generator refuses with a readable sentence when the platform has no
# synthesizer, so a silent fixture cannot quietly report a green Lock for a
# pipeline that never ran a recognizer.
echo "==> synthesizing a recording with speech and shots"
./tools/fixtures/make-spoken-video.sh "$media_path" --ffmpeg "$PWD/.cache/bin/ffmpeg"

# Enrolment first: the daemon reads its trust directory once, at startup, so a
# key created afterwards is a key it never sees.
echo "==> enrolling the worker fleet"
./tools/run-workers.sh --enrol-only --data-dir "$data_dir" >/dev/null

echo "==> starting the daemon"
RUST_LOG=warn \
CLIPMILL_MODELS_DIR="$REGISTRY_DIR" \
CLIPMILL_WEIGHTS_DIR="$WEIGHTS_DIR" \
  target/debug/clipmilld \
  --data-dir "$data_dir" \
  --ffprobe "$PWD/.cache/bin/ffprobe" \
  >"$log_path" 2>&1 &
daemon_pid=$!

ready=false
for _attempt in $(seq 1 600); do
  if [ -S "$socket_path" ] && [ -S "$worker_socket" ]; then
    ready=true
    break
  fi
  if ! kill -0 "$daemon_pid" 2>/dev/null; then
    echo "lock-phase1: daemon exited before becoming ready" >&2
    sed -n '1,160p' "$log_path" >&2
    exit 1
  fi
  sleep 0.05
done
[ "$ready" = true ] || { sed -n '1,160p' "$log_path" >&2; fail "daemon did not become ready"; }

# A registry that loaded nothing does not fail loudly: every stage falls back to
# a 512 MiB estimate, a worker that honestly declares less is never handed the
# task, and speech sits planned forever with nothing to say why. Asserted here
# so that failure is a sentence rather than a fifteen-minute timeout.
#
# Matched with the escape codes stripped: the daemon's logger colours its field
# names, so `models=0` never appears as those eight literal bytes and a plain
# grep for it silently never fires.
loaded="$(sed -e 's/\x1b\[[0-9;]*m//g' "$log_path" |
  sed -n 's/.*pinned model registry loaded models=\([0-9]*\).*/\1/p' | head -1)"
[ -n "$loaded" ] || fail "the daemon never reported loading a model registry"
[ "$loaded" -gt 0 ] || fail "the daemon loaded an empty model registry from $REGISTRY_DIR"
echo "    $loaded pinned models"

echo "==> launching the worker fleet"
identity_dir="$data_dir/state/worker-dev-identity"
for entry in vad:clipmill-worker-vad asr-whispercpp:clipmill-worker-asr \
             align:clipmill-worker-align shots:clipmill-worker-shots \
             faces:clipmill-worker-faces; do
  family="${entry%%:*}"
  command="${entry##*:}"
  (
    cd "workers/$family"
    exec uv run --offline --frozen "$command" \
      --identity "$identity_dir/$family.json" \
      --worker-socket "$worker_socket"
  ) >"$drill_root/worker-$family.log" 2>&1 &
  worker_pids="$worker_pids $!"
done

echo "==> analyze, direct and render with zero egress"
uv run --offline --frozen --project eval/harness python tools/drills/lock_phase1.py \
  --socket "$socket_path" \
  --data-dir "$data_dir" \
  --media "$media_path" \
  --daemon-log "$log_path"

# ---- 3. the canary again ----------------------------------------------------

echo "==> egress canary again (a stage that tried and gave up still tried)"
if curl --silent --max-time 5 --output /dev/null https://example.com 2>/dev/null; then
  fail "the namespace gained network access during the run"
fi

kill -TERM "$daemon_pid"
wait "$daemon_pid" 2>/dev/null || true
daemon_pid=""
if [ -e "$socket_path" ]; then
  fail "clean shutdown left the control socket behind"
fi

echo "lock-phase1: OK (canary blocked; analyze, direct and render green with a live worker fleet and zero egress)"
