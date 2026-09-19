#!/usr/bin/env bash
# Explicit acquisition only. Never resolve/install packages at runtime.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PYTHON="$ROOT/integrations/youtube-import/.venv/bin/python"
if [ ! -x "$PYTHON" ]; then
  printf '%s\n' '{"event":"error","code":"setup_required","message":"YouTube import needs setup. Run just setup-youtube, then retry."}'
  exit 1
fi
exec "$PYTHON" -I -m clipmill_youtube_import "$@"
