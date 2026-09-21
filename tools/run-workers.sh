#!/usr/bin/env bash
# Enroll and launch development workers as separate processes.
# The daemon reads trusted worker keys once at startup, so just app runs
# --enrol-only before starting it. Restart the daemon after enrolling new keys.
# Development identities are stored privately at mode 0600 and never committed.
set -euo pipefail
cd "$(dirname "$0")/.."

ENROL_ONLY=0
CLOUD_EDITORIAL="${CLIPMILL_EDITORIAL_CLOUD:-0}"
DATA_DIR="${CLIPMILL_DATA_DIR:-}"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --enrol-only) ENROL_ONLY=1 ;;
    --cloud-editorial) CLOUD_EDITORIAL=1 ;;
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

# Default analysis workers, with explicit directory and entry-point names.
# speech-mlx requires measured per-device selection and is not launched here.
FAMILIES="vad:clipmill-worker-vad asr-whispercpp:clipmill-worker-asr align:clipmill-worker-align shots:clipmill-worker-shots faces:clipmill-worker-faces"

# Editorial inference is explicitly selected and initially supported on Apple silicon.
if [ "${CLIPMILL_EDITORIAL:-1}" = 1 ] && [ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] && [ -d workers/editorial/.venv ]; then
  FAMILIES="$FAMILIES editorial:clipmill-worker-editorial"
fi

# This is a distinct process and credential, never a capability of the local
# worker. Starting it permits cloud leasing; each lease still needs the user's
# per-run transcript consent and a budget. Visual checks stay on the local worker.
if [ "$CLOUD_EDITORIAL" = 1 ]; then
  FAMILIES="$FAMILIES editorial-cloud:clipmill-worker-editorial-cloud"
fi

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
import time
from pathlib import Path

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

identity_path, trust_dir = Path(sys.argv[1]), Path(sys.argv[2])

# Canonical ULIDs encode a 48-bit timestamp; the leading character must be 0-7.
ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"


def crockford(value: int, length: int) -> str:
    out = []
    for _ in range(length):
        out.append(ALPHABET[value & 0x1F])
        value >>= 5
    return "".join(reversed(out))


timestamp = int(time.time() * 1000) & ((1 << 48) - 1)
randomness = secrets.randbits(80)
worker_id = "wrk_" + crockford(timestamp, 10) + crockford(randomness, 16)

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

# Set private permissions before writing key material.
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

if echo "$FAMILIES" | tr ' ' '\n' | rg -q '^editorial:'; then
  if ! workers/editorial/.venv/bin/python tools/editorial-runtime-check.py --data-dir "$DATA_DIR"; then
    echo "run-workers: editorial runtime is not ready; its stages will show as unavailable" >&2
  fi
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
  directory="$family"
  if [ "$family" = editorial-cloud ]; then directory=editorial; fi
  if [ ! -x "workers/$directory/.venv/bin/$command" ]; then
    echo "run-workers: $family is not installed; run uv sync --locked --directory workers/$directory during setup, then retry." >&2
    exit 2
  fi
  (
    cd "workers/$directory"
    # Worker startup is offline; dependency installation belongs to setup.
    exec uv run --offline --no-sync "$command" \
      --identity "$identity" \
      --worker-socket "$WORKER_SOCKET"
  ) &
  pids="$pids $!"
  echo "    $family (pid $!)"
done

echo "==> workers are up; press Ctrl-C to stop them"
wait
