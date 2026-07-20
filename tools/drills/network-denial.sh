#!/usr/bin/env bash
# Network denial (Phase 0 exit gate 3, decision D16 - zero revisits allowed).
#
# Runs INSIDE a no-network namespace (CI: `unshare -rn`, local: a
# --network=none container). Two proofs, in order:
#
#   1. The egress canary: an outbound connection attempt MUST fail. If it
#      succeeds, the denial harness itself is broken and everything after
#      it would be theater - abort loudly.
#   2. The test suite plus five-iteration cache, job, media, and worker drills
#      pass with zero network: no hidden cloud dependency in the workspace.
#      Build artifacts are prepared outside the namespace; Cargo is offline.
set -euo pipefail
cd "$(dirname "$0")/../.."

echo "==> egress canary (must fail)"
if curl --silent --max-time 5 --output /dev/null https://example.com 2>/dev/null; then
  echo "network-denial: FAILED - the egress canary reached the internet;" >&2
  echo "this script is not running inside a denied namespace" >&2
  exit 1
fi
echo "canary blocked - denial is active"

echo "==> rust test suite, offline"
CARGO_NET_OFFLINE=true cargo test --workspace --offline

echo "==> artifact cache hard-kill smoke test, offline"
CARGO_NET_OFFLINE=true ./tools/drills/cache-drill.sh 5

echo "==> durable job hard-kill smoke test, offline"
CARGO_NET_OFFLINE=true ./tools/drills/kill-drill.sh 5

echo "==> source evidence and media smoke test, offline"
CARGO_NET_OFFLINE=true ./tools/drills/media-drill.sh 5

echo "==> authenticated worker and shared-memory smoke test, offline"
CARGO_NET_OFFLINE=true ./tools/drills/worker-drill.sh 5

echo "network-denial: OK (canary blocked; suite, cache, job, media, and worker drills green with zero egress)"
