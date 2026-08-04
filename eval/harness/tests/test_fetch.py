"""Fetching the corpus, and the two things that must never happen.

Media must never enter Git, and an item whose licence nobody can name must never
be fetched. Both are checked before any byte is written, because neither is
something a later commit undoes.
"""

from __future__ import annotations

import json
import subprocess

import pytest
from clipmill_eval.fetch import (
    FetchError,
    build_documents,
    digest_file,
    load_spec,
    refuse_tracked_destination,
)

SPEC = {
    "corpus_id": "seed-dialogue",
    "items": [
        {
            "item_id": "interview-01",
            "url": "https://example.invalid/one",
            "license_id": "CC-BY-4.0",
            "attribution": "A Speaker, CC BY 4.0",
        }
    ],
}


def write_spec(tmp_path, payload):
    path = tmp_path / "spec.json"
    path.write_text(json.dumps(payload), encoding="utf-8")
    return path


class TestSpec:
    def test_a_well_formed_spec_loads(self, tmp_path) -> None:
        corpus_id, items = load_spec(write_spec(tmp_path, SPEC))
        assert corpus_id == "seed-dialogue"
        assert items[0].license_id == "CC-BY-4.0"
        # Redistribution is off unless the spec says otherwise: assuming a
        # licence permits it is how media ends up somewhere it may not be.
        assert items[0].redistributable is False

    def test_a_licence_nobody_named_is_refused(self, tmp_path) -> None:
        payload = json.loads(json.dumps(SPEC))
        payload["items"][0]["license_id"] = "probably-fine"
        with pytest.raises(FetchError, match="probably-fine"):
            load_spec(write_spec(tmp_path, payload))

    def test_an_item_with_no_attribution_is_refused(self, tmp_path) -> None:
        payload = json.loads(json.dumps(SPEC))
        payload["items"][0]["attribution"] = ""
        with pytest.raises(FetchError, match="attribution"):
            load_spec(write_spec(tmp_path, payload))

    def test_an_item_id_that_is_a_path_is_refused(self, tmp_path) -> None:
        # The id becomes a filename, so a slash in one would write outside the
        # corpus directory.
        payload = json.loads(json.dumps(SPEC))
        payload["items"][0]["item_id"] = "../escape"
        with pytest.raises(FetchError, match="not usable as a filename"):
            load_spec(write_spec(tmp_path, payload))

    def test_two_items_cannot_share_an_id(self, tmp_path) -> None:
        payload = json.loads(json.dumps(SPEC))
        payload["items"].append(dict(payload["items"][0]))
        with pytest.raises(FetchError, match="duplicate item_id"):
            load_spec(write_spec(tmp_path, payload))


class TestDestination:
    def test_a_directory_outside_the_repository_is_allowed(self, tmp_path) -> None:
        outside = tmp_path / "corpus"
        outside.mkdir()
        # No exception: this is the expected case.
        refuse_tracked_destination(outside, tmp_path / "repo")

    def test_a_tracked_directory_inside_the_repository_is_refused(self, tmp_path) -> None:
        repository = tmp_path / "repo"
        (repository / "media").mkdir(parents=True)
        subprocess.run(["git", "init", "--quiet"], cwd=repository, check=True)
        with pytest.raises(FetchError, match="never be committed"):
            refuse_tracked_destination(repository / "media", repository)

    def test_an_ignored_directory_inside_the_repository_is_allowed(self, tmp_path) -> None:
        repository = tmp_path / "repo"
        (repository / "media").mkdir(parents=True)
        subprocess.run(["git", "init", "--quiet"], cwd=repository, check=True)
        (repository / ".gitignore").write_text("media/\n", encoding="utf-8")
        refuse_tracked_destination(repository / "media", repository)


class TestDocuments:
    def test_the_manifest_is_in_the_shape_the_verifier_reads(self, tmp_path) -> None:
        _corpus_id, items = load_spec(write_spec(tmp_path, SPEC))
        media = tmp_path / "interview-01.mkv"
        media.write_bytes(b"not really video")
        manifest, attestation = build_documents("seed-dialogue", [(items[0], media)], tmp_path)

        assert manifest["schema_version"] == "clipmill.corpus_manifest.v1"
        entry = manifest["items"][0]
        assert entry["relative_path"] == "interview-01.mkv"
        assert entry["bytes"] == len(b"not really video")
        assert entry["sha256"] == digest_file(media)[0]
        assert entry["expected_result"] == "success"

        assert attestation["schema_version"] == "clipmill.license_attestation.v1"
        grant = attestation["licenses"][0]
        assert grant["item_id"] == "interview-01"
        assert grant["evaluation_permitted"] is True
        assert grant["redistributable"] is False
        # The attribution and the URL travel with the grant, so a corpus can be
        # credited without going back to the spec.
        assert grant["attribution"] == "A Speaker, CC BY 4.0"

    def test_items_come_out_in_a_stable_order(self, tmp_path) -> None:
        payload = json.loads(json.dumps(SPEC))
        payload["items"].append(
            {
                "item_id": "aaa-first",
                "url": "https://example.invalid/two",
                "license_id": "CC0-1.0",
                "attribution": "Nobody",
            }
        )
        _corpus_id, items = load_spec(write_spec(tmp_path, payload))
        media = []
        for item in items:
            path = tmp_path / f"{item.item_id}.mkv"
            path.write_bytes(item.item_id.encode())
            media.append((item, path))
        manifest, _attestation = build_documents("seed-dialogue", list(reversed(media)), tmp_path)
        assert [entry["item_id"] for entry in manifest["items"]] == ["aaa-first", "interview-01"]

    def test_nothing_it_writes_claims_to_have_been_signed(self, tmp_path) -> None:
        _corpus_id, items = load_spec(write_spec(tmp_path, SPEC))
        media = tmp_path / "interview-01.mkv"
        media.write_bytes(b"x")
        manifest, attestation = build_documents("seed-dialogue", [(items[0], media)], tmp_path)
        for document in (manifest, attestation):
            assert "signature" not in document
            assert "public_key" not in document
