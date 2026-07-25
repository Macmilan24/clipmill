import hashlib
import json
from pathlib import Path

import pytest
from clipmill_eval.corpus import (
    CORPUS_DOMAIN,
    LICENSE_DOMAIN,
    CorpusError,
    verify_corpus,
)
from clipmill_eval.signing import canonical_json, sign_document
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey


def signed_corpus(root: Path) -> tuple[Path, Path, Ed25519PrivateKey]:
    media = root / "media.bin"
    media.write_bytes(b"verified media")
    key = Ed25519PrivateKey.generate()
    manifest = sign_document(
        {
            "schema_version": "clipmill.corpus_manifest.v1",
            "corpus_id": "fixture",
            "items": [
                {
                    "item_id": "one",
                    "relative_path": "media.bin",
                    "sha256": hashlib.sha256(media.read_bytes()).hexdigest(),
                    "bytes": media.stat().st_size,
                    "expected_result": "success",
                    "license_id": "CC0",
                }
            ],
        },
        key,
        CORPUS_DOMAIN,
    )
    license_attestation = sign_document(
        {
            "schema_version": "clipmill.license_attestation.v1",
            "corpus_id": "fixture",
            "licenses": [
                {
                    "item_id": "one",
                    "license_id": "CC0",
                    "redistributable": True,
                }
            ],
        },
        key,
        LICENSE_DOMAIN,
    )
    manifest_path = root / "manifest.json"
    license_path = root / "licenses.json"
    manifest_path.write_bytes(canonical_json(manifest))
    license_path.write_bytes(canonical_json(license_attestation))
    return manifest_path, license_path, key


def test_signed_corpus_verifies_every_byte_and_license(tmp_path: Path) -> None:
    manifest, licenses, key = signed_corpus(tmp_path)
    corpus = verify_corpus(
        tmp_path,
        manifest,
        licenses,
        key.public_key().public_bytes_raw(),
    )
    assert corpus.corpus_id == "fixture"
    assert corpus.items[0].sha256 == hashlib.sha256(b"verified media").hexdigest()
    assert len(corpus.manifest_sha256) == 64
    assert len(corpus.license_attestation_sha256) == 64


def test_media_and_signature_tampering_are_rejected(tmp_path: Path) -> None:
    manifest, licenses, _key = signed_corpus(tmp_path)
    (tmp_path / "media.bin").write_bytes(b"tampered media")
    with pytest.raises(CorpusError, match=r"byte size|SHA-256"):
        verify_corpus(tmp_path, manifest, licenses)

    manifest, licenses, _key = signed_corpus(tmp_path)
    value = json.loads(manifest.read_text())
    value["corpus_id"] = "tampered"
    manifest.write_text(json.dumps(value), encoding="utf-8")
    with pytest.raises(CorpusError, match="signature"):
        verify_corpus(tmp_path, manifest, licenses)


def test_symlinks_and_license_gaps_are_rejected(tmp_path: Path) -> None:
    manifest, licenses, _key = signed_corpus(tmp_path)
    target = tmp_path / "real.bin"
    target.write_bytes((tmp_path / "media.bin").read_bytes())
    (tmp_path / "media.bin").unlink()
    (tmp_path / "media.bin").symlink_to(target)
    with pytest.raises(CorpusError, match="symlink"):
        verify_corpus(tmp_path, manifest, licenses)

    (tmp_path / "media.bin").unlink()
    (tmp_path / "media.bin").write_bytes(b"verified media")
    licenses.write_text("{}", encoding="utf-8")
    with pytest.raises(CorpusError):
        verify_corpus(tmp_path, manifest, licenses)


def test_evaluation_only_rights_grant_is_accepted(tmp_path: Path) -> None:
    manifest, licenses, key = signed_corpus(tmp_path)
    value = json.loads(licenses.read_text(encoding="utf-8"))
    unsigned = {field: entry for field, entry in value.items() if field != "signature"}
    unsigned["licenses"][0]["redistributable"] = False
    unsigned["licenses"][0]["evaluation_permitted"] = True
    licenses.write_bytes(canonical_json(sign_document(unsigned, key, LICENSE_DOMAIN)))

    corpus = verify_corpus(tmp_path, manifest, licenses)

    assert corpus.licenses[0].redistributable is False
    assert corpus.licenses[0].evaluation_permitted is True


def test_license_without_distribution_or_evaluation_rights_is_rejected(
    tmp_path: Path,
) -> None:
    manifest, licenses, key = signed_corpus(tmp_path)
    value = json.loads(licenses.read_text(encoding="utf-8"))
    unsigned = {field: entry for field, entry in value.items() if field != "signature"}
    unsigned["licenses"][0]["redistributable"] = False
    licenses.write_bytes(canonical_json(sign_document(unsigned, key, LICENSE_DOMAIN)))

    with pytest.raises(CorpusError, match="license record"):
        verify_corpus(tmp_path, manifest, licenses)
