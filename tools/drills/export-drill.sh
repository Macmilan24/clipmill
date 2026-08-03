#!/usr/bin/env bash
# W25 export gate.
#
# An export is the last thing a user does and the first thing they would blame,
# so what has to be true of it is narrow and checkable.
#
# The strip refuses rather than warns. A cut inside a word, a sidecar nobody can
# read at the speed it runs, a rights claim nobody made, a disk that will not
# hold it — each of these is a file somebody would find out about after
# uploading, so each blocks with its reason attached. The one thing that must
# *not* block is the burn-in caption track running hot, because running hot is
# what the kinetic intent is for.
#
# The names are resolved once. There is exactly one implementation of the naming
# pattern and the preview asks the daemon for it, so the preview a user approved
# is the name they get. This is the same rule the editor's preview follows.
#
# The archive is the promise that the work outlives this application, so it is
# checked the way somebody without ClipMill would check it: the zip is opened by
# an unrelated tool, every entry is verified against the digest the published
# index claims for it, and the documents that come out are compared with what
# went in. An archive that only this project can read is not an archive.
#
# Determinism is the last claim. The writer contributes no timestamp, no host,
# and no permission bits, so two archives of an unchanged project are the same
# bytes — which is what makes the round-trip comparable at all.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "export-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "export-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> the validation strip: what blocks, what advises, and what neither"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "export-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmill-export
done

echo "==> the delivery: destinations, naming, and the archive round-trip"
# Also leaves two archives in target/export-drill/ for the reader below.
rm -rf target/export-drill
cargo test -p clipmilld --lib export::

echo "==> the Local Lock is read from the registry rather than asserted"
cargo test -p clipmilld --lib policy::

echo "==> the published documents are the documents the schemas describe"
cargo test -p clipmill-export --test contract

echo "==> an archive opens in a tool that has never heard of this project"
WORK="target/export-drill"
if [ ! -f "$WORK/project.zip" ] || [ ! -f "$WORK/again.zip" ]; then
  echo "export-drill: the delivery tests left no archive to verify" >&2
  exit 1
fi
python3 - "$WORK" <<'PY'
"""Verify an archive with nothing from ClipMill but the published schema."""
import hashlib
import json
import pathlib
import sys
import zipfile

work = pathlib.Path(sys.argv[1])
archive = work / "project.zip"

# 1. It is a zip, and the standard library agrees.
with zipfile.ZipFile(archive) as bundle:
    bad = bundle.testzip()
    if bad is not None:
        raise SystemExit(f"export-drill: CRC mismatch on {bad}")
    names = set(bundle.namelist())
    index = json.loads(bundle.read("archive-index.json"))

    # 2. It says which format it is, in the way the published schema requires.
    if index["schema_version"] != "clipmill.archive_index.v1":
        raise SystemExit(f"export-drill: unexpected schema {index['schema_version']}")

    # 3. Every entry it names is present, at the size and digest it claims.
    for entry in index["entries"]:
        if entry["path"] not in names:
            raise SystemExit(f"export-drill: {entry['path']} is named but absent")
        payload = bundle.read(entry["path"])
        if len(payload) != entry["bytes"]:
            raise SystemExit(f"export-drill: {entry['path']} is the wrong size")
        digest = hashlib.sha256(payload).hexdigest()
        if digest != entry["sha256"]:
            raise SystemExit(f"export-drill: {entry['path']} does not hash to its record")
        # 4. And it is JSON, which is what every entry kind here is.
        json.loads(payload)

    # 5. Nothing may extract outside the directory it was extracted into.
    for name in names:
        if name.startswith("/") or ".." in pathlib.PurePosixPath(name).parts:
            raise SystemExit(f"export-drill: {name} would escape the extraction directory")

    # 6. Sources are named, never carried.
    for source in index["sources"]:
        if not source["fingerprint"]:
            raise SystemExit("export-drill: a source was archived without a fingerprint")
    media = [name for name in names if name.endswith((".mov", ".mp4", ".mkv", ".wav"))]
    if media:
        raise SystemExit(f"export-drill: the archive carried media: {media}")

print(f"export-drill: archive verified ({len(index['entries'])} entries, "
      f"{len(index['sources'])} sources named)")
PY

echo "==> the same project archives to the same bytes twice"
if ! cmp -s "$WORK/project.zip" "$WORK/again.zip"; then
  echo "export-drill: two archives of one project differ" >&2
  exit 1
fi

echo "==> the screens: refusals stop an export, and the names shown are the daemon's"
pnpm --filter @clipmill/desktop test

echo "export-drill: OK ($ITERATIONS iterations; strip refusals, burn-in advisory, one naming implementation, archive round-trip through an unrelated reader, byte-identical re-archive)"
