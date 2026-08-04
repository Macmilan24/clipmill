"""Fetching the private evaluation corpus, once, on a machine allowed to.

This is the only thing in ClipMill that reaches the network on purpose, and it
is deliberately not part of the product: it runs on the developer's machine,
outside the Local Lock, before any evaluation starts. The daemon never does
this and never learns how.

Three rules shape it.

**Licences are recorded per item, from the spec, before the bytes arrive.** A
corpus assembled first and licensed afterwards is a corpus somebody has to
audit; one that cannot name a licence for an item refuses to fetch it.

**Media never enters Git.** The destination is checked against the repository's
own ignore rules before anything is written, because a forty-gigabyte corpus
committed by accident is not something a later commit removes.

**The output is the input to the existing signing flow.** This writes an
unsigned manifest and an unsigned licence attestation in exactly the shape
`verify_corpus` reads; signing them is a separate step with a separate key, so
fetching can never produce a corpus that claims to have been attested.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

#: Licences a corpus item may carry. Closed on purpose: the point of the corpus
#: is that every item can be named and defended, and a free-text field would let
#: "probably fine" in.
ALLOWED_LICENCES = (
    "CC0-1.0",
    "CC-BY-4.0",
    "CC-BY-SA-4.0",
    "CC-BY-3.0",
    "public-domain",
    "owner-permission",
)

#: Read in 1 MiB blocks; corpus items are gigabytes and hashing them whole in
#: memory would be a needless allocation on the one machine that does this.
_BLOCK = 1024 * 1024


class FetchError(RuntimeError):
    """The corpus could not be fetched, or should not have been."""


@dataclass(frozen=True, slots=True)
class SpecItem:
    item_id: str
    url: str
    license_id: str
    attribution: str
    redistributable: bool


def load_spec(path: Path) -> tuple[str, tuple[SpecItem, ...]]:
    """Read the corpus spec: what to fetch and under what licence."""

    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise FetchError(f"cannot read {path}: {error}") from error
    corpus_id = raw.get("corpus_id")
    if not isinstance(corpus_id, str) or not corpus_id:
        raise FetchError("the spec must name a corpus_id")
    entries = raw.get("items")
    if not isinstance(entries, list) or not entries:
        raise FetchError("the spec must list at least one item")
    items = tuple(_spec_item(entry) for entry in entries)
    seen: set[str] = set()
    for item in items:
        if item.item_id in seen:
            raise FetchError(f"duplicate item_id in the spec: {item.item_id}")
        seen.add(item.item_id)
    return corpus_id, items


def _spec_item(entry: Any) -> SpecItem:
    if not isinstance(entry, dict):
        raise FetchError("each spec item must be an object")
    for key in ("item_id", "url", "license_id", "attribution"):
        if not isinstance(entry.get(key), str) or not entry[key]:
            raise FetchError(f"each spec item needs a non-empty {key}")
    if entry["license_id"] not in ALLOWED_LICENCES:
        raise FetchError(
            f"{entry['item_id']} declares {entry['license_id']}, "
            f"which is not one of {ALLOWED_LICENCES}"
        )
    if "/" in entry["item_id"] or entry["item_id"].startswith("."):
        raise FetchError(f"{entry['item_id']} is not usable as a filename")
    return SpecItem(
        item_id=entry["item_id"],
        url=entry["url"],
        license_id=entry["license_id"],
        attribution=entry["attribution"],
        redistributable=bool(entry.get("redistributable", False)),
    )


def refuse_tracked_destination(destination: Path, repository: Path) -> None:
    """Refuse to write media anywhere Git would pick it up.

    Asked of Git rather than guessed from the path: a directory can be ignored
    by any of several files at any depth, and a rule written here would be a
    second implementation of ignore semantics that disagrees with the first.
    """

    resolved = destination.resolve()
    try:
        resolved.relative_to(repository.resolve())
    except ValueError:
        # Outside the repository entirely, which is the expected case.
        return
    result = subprocess.run(
        ["git", "check-ignore", "--quiet", str(resolved)],
        cwd=repository,
        check=False,
        capture_output=True,
    )
    # 0 means ignored, 1 means not ignored, anything else means git could not
    # answer — and an unanswered question here is a refusal, not a pass.
    if result.returncode != 0:
        raise FetchError(
            f"{destination} is inside the repository and not ignored by Git; "
            "corpus media must never be committed"
        )


def digest_file(path: Path) -> tuple[str, int]:
    hasher = hashlib.sha256()
    size = 0
    with path.open("rb") as handle:
        while block := handle.read(_BLOCK):
            hasher.update(block)
            size += len(block)
    return hasher.hexdigest(), size


def fetch_item(item: SpecItem, destination: Path, downloader: Sequence[str]) -> Path:
    """Fetch one item with the pinned downloader.

    The output template is fixed by item id rather than by the remote title, so
    a corpus is reproducible from its spec and a renamed upload does not become
    a different file.
    """

    destination.mkdir(parents=True, exist_ok=True)
    template = str(destination / f"{item.item_id}.%(ext)s")
    command = [
        *downloader,
        "--no-playlist",
        "--no-progress",
        "--output",
        template,
        item.url,
    ]
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "").strip().splitlines()
        raise FetchError(f"{item.item_id} could not be fetched: {detail[-1] if detail else '?'}")
    found = sorted(destination.glob(f"{item.item_id}.*"))
    media = [path for path in found if path.suffix not in (".json", ".md", ".part")]
    if len(media) != 1:
        raise FetchError(f"{item.item_id} produced {len(media)} media files; expected exactly one")
    return media[0]


def build_documents(
    corpus_id: str,
    items: Sequence[tuple[SpecItem, Path]],
    root: Path,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """The unsigned manifest and licence attestation, in the shape
    `verify_corpus` reads.

    Unsigned on purpose. Signing is a separate command with a separate key, so
    a fetch cannot produce something that looks attested.
    """

    manifest_items: list[dict[str, Any]] = []
    grants: list[dict[str, Any]] = []
    for item, path in sorted(items, key=lambda pair: pair[0].item_id):
        digest, size = digest_file(path)
        manifest_items.append(
            {
                "item_id": item.item_id,
                "relative_path": path.relative_to(root).as_posix(),
                "sha256": digest,
                "bytes": size,
                "expected_result": "success",
                "license_id": item.license_id,
            }
        )
        grants.append(
            {
                "item_id": item.item_id,
                "license_id": item.license_id,
                "attribution": item.attribution,
                "source_url": item.url,
                # Evaluation is always permitted for an item in this corpus —
                # that is what makes it admissible — while redistribution is
                # per licence and is what keeps the media out of the repository.
                "evaluation_permitted": True,
                "redistributable": item.redistributable,
            }
        )
    manifest = {
        "schema_version": "clipmill.corpus_manifest.v1",
        "corpus_id": corpus_id,
        "items": manifest_items,
    }
    attestation = {
        "schema_version": "clipmill.license_attestation.v1",
        "corpus_id": corpus_id,
        "licenses": grants,
    }
    return manifest, attestation


def write_documents(
    output: Path,
    manifest: dict[str, Any],
    attestation: dict[str, Any],
) -> tuple[Path, Path]:
    output.mkdir(parents=True, exist_ok=True)
    manifest_path = output / "corpus-manifest.unsigned.json"
    attestation_path = output / "license-attestation.unsigned.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    attestation_path.write_text(json.dumps(attestation, indent=2) + "\n", encoding="utf-8")
    return manifest_path, attestation_path
