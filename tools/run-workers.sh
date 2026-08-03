#!/usr/bin/env bash
# Enrol and launch the model workers for a development session.
#
# The daemon does not start workers and should not: a worker is a separate
# process precisely so a model crash costs a leased task rather than the
# daemon's state (decision D03). But that leaves a gap nothing filled — every
# gate starts its own workers inside a drill, and running the app for real had
# no equivalent. The analyze DAG would plan voice activity, recognition,
# alignment, shots and faces, and every one of them would sit unleased forever
# because nothing was listening.
#
# Enrolment is the part that is easy to get subtly wrong. A worker authenticates
# with an Ed25519 key whose public half the daemon must already trust, and the
# daemon reads the trust directory **once, at startup**. So enrolling after the
# daemon is running does nothing until it restarts — which is why `just app`
# runs the enrolment step first, and why this script says so plainly rather than
# letting a worker be rejected with an error nobody can place.
#
# Identities are development credentials. They live in the private state
# directory at mode 0600, are generated once, and are never committed.
set -euo pipefail
cd "$(dirname "$0")/.."

ENROL_ONLY=0
DATA_DIR="${CLIPMILL_DATA_DIR:-}"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --enrol-only) ENROL_ONLY=1 ;;
    --data-dir) DATA_DIR="${2:?--data-dir needs a path}"; shift ;;
    *) echo "run-workers: unknown argument $1" >&2; exit 2 ;;
  esac
  shift
done

if [ -z "$DATA_DIR" ]; then
  case "$(uname -s)" in
    Darwin) DATA_DIR="$HOME/Library/Application Support/dev.clipmill.ClipMill" ;;
    *) DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/clipmill" ;;
  esac
fi
STATE_DIR="$DATA_DIR/state"
RUN_DIR="$DATA_DIR/run"
TRUST_DIR="$STATE_DIR/worker-trust"
IDENTITY_DIR="$STATE_DIR/worker-dev-identity"
WORKER_SOCKET="$RUN_DIR/clipmill-workers.sock"

# The families a Phase 1 analyze DAG leases. `speech-mlx` is deliberately not
# here: which recognizer serves a capability is a measured per-device decision
# (D19), and launching both would put two workers up for one capability.
# Directory and entry point differ for one of them, so both are named rather
# than derived: guessing a console-script name from a directory name is the
# kind of cleverness that breaks the day somebody adds the sixth worker.
FAMILIES="vad:clipmill-worker-vad asr-whispercpp:clipmill-worker-asr align:clipmill-worker-align shots:clipmill-worker-shots faces:clipmill-worker-faces"

mkdir -p "$TRUST_DIR" "$IDENTITY_DIR"
chmod 700 "$TRUST_DIR" "$IDENTITY_DIR"

echo "==> enrolling development workers under $STATE_DIR"
for entry in $FAMILIES; do
  family="${entry%%:*}"
  identity="$IDENTITY_DIR/$family.json"
  if [ -f "$identity" ]; then
    continue
  fi
  python3 - "$identity" "$TRUST_DIR" <<'PY'
"""Generate one development worker identity and trust its public half."""
import json
import os
import secrets
import sys
from pathlib import Path

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

identity_path, trust_dir = Path(sys.argv[1]), Path(sys.argv[2])

# A ULID in Crockford base32, which is the shape `WorkerId` parses. Random
# rather than time-ordered: nothing sorts these, and a development identity
# should not carry when it was made.
ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
worker_id = "wrk_" + "".join(secrets.choice(ALPHABET) for _ in range(26))

private_key = Ed25519PrivateKey.generate()
private_bytes = private_key.private_bytes(
    encoding=serialization.Encoding.Raw,
    format=serialization.PrivateFormat.Raw,
    encryption_algorithm=serialization.NoEncryption(),
)
public_bytes = private_key.public_key().public_bytes(
    encoding=serialization.Encoding.Raw,
    format=serialization.PublicFormat.Raw,
)

# Written at 0600 before anything goes in: the daemon refuses an identity or a
# trust key any other user could read, and creating it permissive and fixing it
# afterwards leaves a window where it was not.
identity_path.write_text("")
identity_path.chmod(0o600)
identity_path.write_text(
    json.dumps(
        {
            "key_version": "clipmill.worker.identity.v1",
            "worker_id": worker_id,
            "private_key": private_bytes.hex(),
        }
    ),
    encoding="utf-8",
)
os.chmod(identity_path, 0o600)

public_path = trust_dir / f"{worker_id}.pub"
public_path.write_text("")
public_path.chmod(0o600)
public_path.write_text(public_bytes.hex() + "\n", encoding="utf-8")
os.chmod(public_path, 0o600)
print(f"  enrolled {identity_path.stem} as {worker_id}")
PY
done

enrolled="$(find "$TRUST_DIR" -name '*.pub' | wc -l | tr -d ' ')"
echo "    $enrolled worker keys trusted"

if [ "$ENROL_ONLY" -eq 1 ]; then
  exit 0
fi

if [ ! -S "$WORKER_SOCKET" ]; then
  echo "run-workers: no daemon is listening at $WORKER_SOCKET" >&2
  echo "run-workers: start the app first (just app), then run this" >&2
  exit 2
fi

# The daemon reads its trust directory once, when it starts. A key enrolled
# after that is a key it has never heard of, and the worker would be refused
# with an error that looks like a bug rather than an ordering problem. The
# socket's timestamp is when the daemon came up, so a newer key is exactly that
# case — said plainly instead of discovered.
stale="$(find "$TRUST_DIR" -name '*.pub' -newer "$WORKER_SOCKET" | wc -l | tr -d ' ')"
if [ "$stale" -gt 0 ]; then
  echo "run-workers: $stale key(s) were enrolled after the daemon started." >&2
  echo "run-workers: it reads the trust directory once, so restart the app" >&2
  echo "run-workers: (just app) and run this again." >&2
  exit 2
fi

pids=""
cleanup() {
  for pid in $pids; do
    kill -TERM "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "==> launching workers against $WORKER_SOCKET"
for entry in $FAMILIES; do
  family="${entry%%:*}"
  command="${entry##*:}"
  identity="$IDENTITY_DIR/$family.json"
  (
    cd "workers/$family"
    exec uv run "$command" \
      --identity "$identity" \
      --worker-socket "$WORKER_SOCKET"
  ) &
  pids="$pids $!"
  echo "    $family (pid $!)"
done

echo "==> workers are up; press Ctrl-C to stop them"
wait
