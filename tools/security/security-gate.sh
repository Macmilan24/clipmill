#!/usr/bin/env bash
# Reproducible W8 security policy checks. Vulnerability DB lookups run in the
# separate supply-chain job; this gate itself is safe in a no-network namespace.
set -euo pipefail
cd "$(dirname "$0")/../.."

test -s docs/threat-model.md || { echo "security-gate: threat model is missing" >&2; exit 1; }
test -s .github/pull_request_template.md || { echo "security-gate: PR threat checklist is missing" >&2; exit 1; }
python3 tools/security/check-actions.py
python3 tools/security/check-bom.py
python3 tools/security/check-models.py
python3 tools/security/check-node-licenses.py
python3 tools/security/check-python-licenses.py
python3 tools/security/scan-repository.py

echo "==> hostile corpus, signed attestation, artifact-path, and framing tests"
(cd eval/harness && uv run --offline --frozen pytest -q \
  tests/test_artifacts.py tests/test_attestation.py tests/test_corpus.py)
cargo test --quiet -p clipmill-artifacts path
cargo test --quiet -p clipmilld framing
echo "security-gate: OK (boundaries, licenses, model policy, secrets, hostile inputs, and publication paths)"
