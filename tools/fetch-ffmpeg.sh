#!/usr/bin/env bash
# Fetch the pinned FFmpeg + ffprobe static binaries per bom.toml into
# .cache/bin/, verifying every artifact's sha256 before install (R4).
# Idempotent: skips downloads whose pin is already installed.
set -euo pipefail
cd "$(dirname "$0")/.."

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) PLATFORM="macos-arm64" ;;
  Linux-x86_64) PLATFORM="linux-amd64" ;;
  *)
    echo "fetch-ffmpeg: unsupported platform $(uname -s)-$(uname -m)" >&2
    exit 1
    ;;
esac

bom_get() { # bom_get <section> <key>
  awk -v section="[ffmpeg.$1]" -v key="$2" '
    $0 == section { in_section = 1; next }
    /^\[/ { in_section = 0 }
    in_section && $1 == key { gsub(/"/, "", $3); print $3; exit }
  ' bom.toml
}

sha256() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'; else sha256sum "$1" | awk '{print $1}'; fi
}

mkdir -p .cache/bin
for tool in ffmpeg ffprobe; do
  url="$(bom_get "$PLATFORM" "${tool}_url")"
  want="$(bom_get "$PLATFORM" "${tool}_sha256")"
  [ -n "$url" ] && [ -n "$want" ] || { echo "fetch-ffmpeg: no pin for $tool/$PLATFORM in bom.toml" >&2; exit 1; }

  marker=".cache/bin/.${tool}.sha256"
  if [ -x ".cache/bin/$tool" ] && [ -f "$marker" ] && [ "$(cat "$marker")" = "$want" ]; then
    echo "$tool: pinned build already installed"
    continue
  fi

  echo "$tool: downloading $url"
  curl -sSfL "$url" -o ".cache/bin/$tool.zip"
  got="$(sha256 ".cache/bin/$tool.zip")"
  if [ "$got" != "$want" ]; then
    echo "fetch-ffmpeg: sha256 MISMATCH for $tool" >&2
    echo "  want $want" >&2
    echo "  got  $got" >&2
    rm -f ".cache/bin/$tool.zip"
    exit 1
  fi
  unzip -oq ".cache/bin/$tool.zip" -d .cache/bin
  rm -f ".cache/bin/$tool.zip"
  chmod +x ".cache/bin/$tool"
  echo "$want" > "$marker"
done

.cache/bin/ffmpeg -version | head -1
.cache/bin/ffprobe -version | head -1
echo "fetch-ffmpeg: OK (.cache/bin)"
