#!/usr/bin/env bash
# Fetch the pinned media substrate per bom.toml into .cache/, verifying every
# artifact's sha256 before install (R4): the FFmpeg + ffprobe static binaries
# and the one font libass is allowed to rasterize captions with.
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
  awk -v section="[$1]" -v key="$2" '
    $0 == section { in_section = 1; next }
    /^\[/ { in_section = 0 }
    in_section && $1 == key { gsub(/"/, "", $3); print $3; exit }
  ' bom.toml
}

sha256() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'; else sha256sum "$1" | awk '{print $1}'; fi
}

verify() { # verify <label> <path> <want>
  got="$(sha256 "$2")"
  if [ "$got" != "$3" ]; then
    echo "fetch-ffmpeg: sha256 MISMATCH for $1" >&2
    echo "  want $3" >&2
    echo "  got  $got" >&2
    rm -f "$2"
    exit 1
  fi
}

mkdir -p .cache/bin
for tool in ffmpeg ffprobe; do
  url="$(bom_get "ffmpeg.$PLATFORM" "${tool}_url")"
  want="$(bom_get "ffmpeg.$PLATFORM" "${tool}_sha256")"
  [ -n "$url" ] && [ -n "$want" ] || { echo "fetch-ffmpeg: no pin for $tool/$PLATFORM in bom.toml" >&2; exit 1; }

  marker=".cache/bin/.${tool}.sha256"
  if [ -x ".cache/bin/$tool" ] && [ -f "$marker" ] && [ "$(cat "$marker")" = "$want" ]; then
    echo "$tool: pinned build already installed"
    continue
  fi

  echo "$tool: downloading $url"
  curl -sSfL "$url" -o ".cache/bin/$tool.zip"
  verify "$tool" ".cache/bin/$tool.zip" "$want"
  unzip -oq ".cache/bin/$tool.zip" -d .cache/bin
  rm -f ".cache/bin/$tool.zip"
  chmod +x ".cache/bin/$tool"
  echo "$want" > "$marker"
done

# ---- Caption font -----------------------------------------------------------
# One face, pinned by the digest of the archive *and* of the member taken out
# of it, because the member's digest is what the render manifest records.

mkdir -p .cache/fonts
font_family="$(bom_get fonts.captions family)"
font_style="$(bom_get fonts.captions style)"
font_target=".cache/fonts/${font_family}-${font_style}.ttf"
font_want="$(bom_get fonts.captions member_sha256)"
font_marker=".cache/fonts/.font.sha256"
if [ -f "$font_target" ] && [ -f "$font_marker" ] && [ "$(cat "$font_marker")" = "$font_want" ]; then
  echo "font: pinned $font_family $font_style already installed"
else
  font_url="$(bom_get fonts.captions archive_url)"
  font_archive_want="$(bom_get fonts.captions archive_sha256)"
  font_member="$(bom_get fonts.captions member)"
  license_member="$(bom_get fonts.captions license_member)"
  license_want="$(bom_get fonts.captions license_sha256)"
  [ -n "$font_url" ] && [ -n "$font_archive_want" ] && [ -n "$font_member" ] \
    || { echo "fetch-ffmpeg: no font pin in bom.toml" >&2; exit 1; }

  echo "font: downloading $font_url"
  curl -sSfL "$font_url" -o .cache/fonts/font.zip
  verify "font archive" .cache/fonts/font.zip "$font_archive_want"
  rm -rf .cache/fonts/unpacked
  unzip -oq .cache/fonts/font.zip "$font_member" "$license_member" -d .cache/fonts/unpacked
  verify "$font_member" ".cache/fonts/unpacked/$font_member" "$font_want"
  verify "$license_member" ".cache/fonts/unpacked/$license_member" "$license_want"
  mv ".cache/fonts/unpacked/$font_member" "$font_target"
  mv ".cache/fonts/unpacked/$license_member" ".cache/fonts/${font_family}-LICENSE.txt"
  rm -rf .cache/fonts/font.zip .cache/fonts/unpacked
  echo "$font_want" > "$font_marker"
fi

# ---- Capability probe -------------------------------------------------------
# Burned-in captions are not optional, so a build without libass is a failed
# fetch rather than a render that discovers it hours later.
#
# Both probes capture before matching: `grep -q` exits on its first hit, which
# closes the pipe under FFmpeg and turns a successful probe into a SIGPIPE that
# `pipefail` would report as a failed fetch.
ffmpeg_version_output="$(.cache/bin/ffmpeg -hide_banner -version)"
case "$ffmpeg_version_output" in
  *--enable-libass*) ;;
  *)
    echo "fetch-ffmpeg: the pinned FFmpeg build has no libass; captions cannot burn in" >&2
    exit 1
    ;;
esac
ffmpeg_filters_output="$(.cache/bin/ffmpeg -hide_banner -filters 2>/dev/null)"
for filter in subtitles ass; do
  # A here-string rather than a pipe: `grep -q` stops at its first match, and a
  # writer still mid-output gets EPIPE, which `pipefail` would report as a
  # failed probe. That race is not reproducible on demand, so it must not be
  # possible at all.
  if ! grep -Eq "^ *[.TSC]+ +$filter +" <<<"$ffmpeg_filters_output"; then
    echo "fetch-ffmpeg: the pinned FFmpeg build has no $filter filter" >&2
    exit 1
  fi
done

# Same reason: capture, then take the first line, rather than piping into a
# reader that exits early.
printf '%s\n' "${ffmpeg_version_output%%$'\n'*}"
ffprobe_version_output="$(.cache/bin/ffprobe -hide_banner -version)"
printf '%s\n' "${ffprobe_version_output%%$'\n'*}"
echo "font: $font_family $font_style ($(bom_get fonts.captions license)) at $font_target"
echo "fetch-ffmpeg: OK (.cache/bin, .cache/fonts; libass present)"
