#!/usr/bin/env bash
# W26 golden aggregator: every stage's byte-stable expectation, in one place.
#
# Each of these already runs inside its own gate. Running them together is a
# different claim: that the whole chain of goldens is consistent at one commit,
# which is what a Phase-1 exit report has to be able to say in one line.
#
# Goldens are the cheapest evidence in the project — no daemon, no media, no
# models — so this is the gate to run before anything expensive.
set -euo pipefail
cd "$(dirname "$0")/../.."

echo "==> contracts: fixtures round-trip through generated types"
cargo test --quiet -p clipmill-contracts

echo "==> evidence index, discovery, ranking"
cargo test --quiet -p clipmill-evidence
cargo test --quiet -p clipmill-discovery

echo "==> captions, reframe, the edit IR, the render compiler"
cargo test --quiet -p clipmill-captions
cargo test --quiet -p clipmill-reframe
cargo test --quiet -p clipmill-edit-ir
cargo test --quiet -p clipmill-render

echo "==> the edit director and the delivery documents"
cargo test --quiet -p clipmill-director
cargo test --quiet -p clipmill-export

echo "gate-golden: OK (every stage golden agrees at this commit)"
