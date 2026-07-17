#!/usr/bin/env bash
# Network denial (Phase 0 exit gate 3, decision D16 - zero revisits allowed).
#
# Runs INSIDE a no-network namespace (CI: `unshare -rn`, local: a
# --network=none container). Two proofs, in order:
#
#   1. The egress canary: an outbound connection attempt MUST fail. If it
#      succeeds, the denial harness itself is broken and everything after
#      it would be theater - abort loudly.
#   2. The test suite passes with zero network: no hidden cloud dependency
#      anywhere in the workspace. Build artifacts are prepared outside the
#      namespace; in here everything runs --offline.
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

echo "network-denial: OK (canary blocked, full suite green with zero egress)"
