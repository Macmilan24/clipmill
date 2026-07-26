#!/usr/bin/env bash
# Fetch the pinned models per models/registry/ into .cache/models/, verifying
# every file's sha256 before it is installed.
#
# Acquisition happens here, outside the Local Lock, and never inside the app:
# a creator tool that downloads weights while editing is a creator tool that
# phones home while editing. The daemon only ever reads what this script left
# behind, and refuses anything whose digest does not match.
#
#   ./tools/fetch-models.sh                 # fetch everything pinned
#   ./tools/fetch-models.sh silero-vad ...  # fetch named models only
#   ./tools/fetch-models.sh --verify-only   # check what is installed, fetch nothing
set -euo pipefail
cd "$(dirname "$0")/.."

VERIFY_ONLY=0
SELECTED=()
for argument in "$@"; do
  case "$argument" in
    --verify-only) VERIFY_ONLY=1 ;;
    -*)
      echo "fetch-models: unknown option $argument" >&2
      exit 2
      ;;
    *) SELECTED+=("$argument") ;;
  esac
done

REGISTRY="models/registry"
INSTALL_ROOT=".cache/models"
[ -d "$REGISTRY" ] || { echo "fetch-models: no registry at $REGISTRY" >&2; exit 2; }

# The licence policy runs first. A model whose terms forbid what users do with
# the output should never reach the disk in the first place.
python3 tools/security/check-models.py --registry "$REGISTRY"

sha256() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'; else sha256sum "$1" | awk '{print $1}'; fi
}

# Read one scalar out of a manifest without needing a TOML parser in shell.
manifest_get() { # manifest_get <file> <dotted.key>
  python3 - "$1" "$2" <<'PY'
import sys, tomllib
manifest = tomllib.load(open(sys.argv[1], "rb"))
value = manifest
for part in sys.argv[2].split("."):
    value = value[part]
print(value)
PY
}

manifest_files() { # manifest_files <file> -> "<path>\t<sha256>\t<bytes>" per line
  python3 - "$1" <<'PY'
import sys, tomllib
for entry in tomllib.load(open(sys.argv[1], "rb"))["files"]:
    print(f"{entry['path']}\t{entry['sha256']}\t{entry['bytes']}")
PY
}

selected() { # selected <name>
  [ ${#SELECTED[@]} -eq 0 ] && return 0
  for wanted in "${SELECTED[@]}"; do [ "$wanted" = "$1" ] && return 0; done
  return 1
}

total=0
verified=0
fetched=0
missing=0
for manifest in "$REGISTRY"/*.toml; do
  name="$(basename "$manifest" .toml)"
  selected "$name" || continue
  total=$((total + 1))
  repo="$(manifest_get "$manifest" source.repo)"
  revision="$(manifest_get "$manifest" source.revision)"
  provider="$(manifest_get "$manifest" source.provider)"
  target="$INSTALL_ROOT/$name"

  while IFS=$'\t' read -r path want bytes; do
    file="$target/$path"
    if [ -f "$file" ]; then
      got="$(sha256 "$file")"
      if [ "$got" = "$want" ]; then
        verified=$((verified + 1))
        continue
      fi
      echo "fetch-models: $name/$path does not match its pin" >&2
      if [ "$VERIFY_ONLY" -eq 1 ]; then
        echo "  want $want" >&2
        echo "  got  $got" >&2
        exit 1
      fi
      rm -f "$file"
    elif [ "$VERIFY_ONLY" -eq 1 ]; then
      echo "fetch-models: $name/$path is not installed" >&2
      missing=$((missing + 1))
      continue
    fi

    mkdir -p "$(dirname "$file")"
    url="$provider/$repo/resolve/$revision/$path"
    echo "$name: fetching $path ($((bytes / 1024 / 1024)) MiB)"
    curl -sSfL "$url" -o "$file"
    got="$(sha256 "$file")"
    if [ "$got" != "$want" ]; then
      echo "fetch-models: sha256 MISMATCH for $name/$path" >&2
      echo "  want $want" >&2
      echo "  got  $got" >&2
      rm -f "$file"
      exit 1
    fi
    fetched=$((fetched + 1))
  done < <(manifest_files "$manifest")
done

if [ "$VERIFY_ONLY" -eq 1 ]; then
  if [ "$missing" -gt 0 ]; then
    echo "fetch-models: $verified file(s) verified, $missing not installed" >&2
    exit 1
  fi
  echo "fetch-models: OK ($total model(s), $verified file(s) verified against their pins)"
  exit 0
fi
echo "fetch-models: OK ($total model(s); $fetched fetched, $verified already verified)"
