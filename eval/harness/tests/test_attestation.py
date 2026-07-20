import json
import stat
from pathlib import Path

import pytest
from clipmill_eval.attestation import (
    PUBLIC_FILENAMES,
    Phase0AttestationError,
    build_phase0_attestation,
    load_private_signing_key,
    verify_phase0_attestation,
    write_phase0_attestation,
)
from clipmill_eval.corpus import CorpusItem, LicenseGrant, VerifiedCorpus
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey


def fixture_corpus(root: Path, count: int = 40) -> VerifiedCorpus:
    items = tuple(
        CorpusItem(
            item_id=f"item-{index:02}",
            relative_path=f"media-{index:02}.mkv",
            sha256=f"{index:064x}",
            byte_size=index + 1,
            expected_result="structured_failure" if index == count - 1 else "success",
            expected_failure="probe" if index == count - 1 else None,
            license_id="private-evaluation-v1",
        )
        for index in range(count)
    )
    licenses = tuple(LicenseGrant(item.item_id, item.license_id, False, True) for item in items)
    return VerifiedCorpus(
        corpus_id="seed-40-v1",
        root=root,
        items=items,
        licenses=licenses,
        signing_public_key=bytes.fromhex("11" * 32),
        manifest_sha256="22" * 32,
        license_attestation_sha256="33" * 32,
    )


def fixture_run(corpus: VerifiedCorpus) -> dict[str, object]:
    results: list[dict[str, object]] = []
    for index, item in enumerate(corpus.items):
        common: dict[str, object] = {
            "item_id": item.item_id,
            "expected_result": item.expected_result,
            "cold_millis": index,
            "warm_millis": index + 1,
        }
        if item.expected_result == "success":
            artifact_id = f"sha256:{index:064x}"
            common.update(
                {
                    "observed_result": "success",
                    "source_fingerprint": f"sha256:{index + 100:064x}",
                    "source_map_artifact_id": artifact_id,
                    "cold_source_map_artifact_id": artifact_id,
                    "warm_source_map_artifact_id": artifact_id,
                    "warm_cache_hit": True,
                    "artifact_key_version": "clipmill.artifact.key.v1",
                    "ffmpeg_bom": "ffmpeg-8.1.2-btb-n8.1.2",
                    "mapping_algorithm": "clipmill.source-map.mapping.v1",
                    "probe_algorithm": "clipmill.ffprobe.normalize.v1",
                    "producer_implementation": "ffprobe-8.1.2+clipmill-map-v1",
                    "source_map_schema_version": "clipmill.source_map.v1",
                    "source_map_metrics": {
                        "chapters": 0,
                        "mapping_segments": 1,
                        "streams": 1,
                    },
                }
            )
        else:
            common.update(
                {
                    "observed_result": "structured_failure",
                    "warm_observed_result": "structured_failure",
                    "failure_code": "DETERMINISTIC",
                    "failure": item.expected_failure,
                }
            )
        results.append(common)
    return {
        "schema_version": "clipmill.eval.run.v1",
        "versions": {
            "artifact_manifest": "clipmill.artifact.manifest.v1",
            "evaluation": "clipmill.eval.run.v1",
            "source_map": "clipmill.source_map.v1",
        },
        "corpus_id": corpus.corpus_id,
        "corpus_signing_key": corpus.signing_public_key.hex(),
        "daemon_version": "0.0.1",
        "hardware_profile": {
            "artifact_id": f"sha256:{'44' * 32}",
            "hardware_fingerprint": f"sha256:{'55' * 32}",
            "measurement_generation": 1,
        },
        "items": results,
        "policy": "local-lock",
        "started_unix_millis": 1,
        "completed_unix_millis": 2,
    }


def test_phase0_attestation_round_trip_contains_only_public_evidence(tmp_path: Path) -> None:
    corpus = fixture_corpus(tmp_path)
    bundle = build_phase0_attestation(corpus, fixture_run(corpus), Ed25519PrivateKey.generate())
    output = tmp_path / "public"

    write_phase0_attestation(output, bundle)
    verified = verify_phase0_attestation(output)

    assert {path.name for path in output.iterdir()} == PUBLIC_FILENAMES
    assert verified.corpus_metadata["items_total"] == 40
    assert verified.corpus_metadata["valid_items"] == 39
    assert verified.corpus_metadata["hostile_items"] == 1
    assert verified.license_summary["evaluation_only_items"] == 40
    assert all(stat.S_IMODE(path.stat().st_mode) == 0o644 for path in output.iterdir())
    assert str(tmp_path) not in (output / "run-attestation.json").read_text(encoding="utf-8")


def test_phase0_attestation_rejects_wrong_count_identity_and_paths(tmp_path: Path) -> None:
    key = Ed25519PrivateKey.generate()
    short = fixture_corpus(tmp_path, 39)
    with pytest.raises(Phase0AttestationError, match="exactly 40"):
        build_phase0_attestation(short, fixture_run(short), key)

    corpus = fixture_corpus(tmp_path)
    changed_identity = fixture_run(corpus)
    changed_identity["items"][0]["warm_source_map_artifact_id"] = f"sha256:{'ff' * 32}"
    with pytest.raises(Phase0AttestationError, match="artifact identity"):
        build_phase0_attestation(corpus, changed_identity, key)

    leaked_path = fixture_run(corpus)
    leaked_path["private_path"] = str(tmp_path)
    with pytest.raises(Phase0AttestationError, match="path"):
        build_phase0_attestation(corpus, leaked_path, key)


def test_phase0_attestation_detects_public_file_tampering(tmp_path: Path) -> None:
    corpus = fixture_corpus(tmp_path)
    bundle = build_phase0_attestation(corpus, fixture_run(corpus), Ed25519PrivateKey.generate())
    output = tmp_path / "public"
    write_phase0_attestation(output, bundle)
    attestation = json.loads((output / "run-attestation.json").read_text(encoding="utf-8"))
    attestation["phase"] = "tampered"
    (output / "run-attestation.json").write_text(json.dumps(attestation), encoding="utf-8")

    with pytest.raises(Phase0AttestationError, match=r"canonical|signature"):
        verify_phase0_attestation(output)


def test_private_signing_key_requires_private_mode(tmp_path: Path) -> None:
    key_path = tmp_path / "phase0.key"
    private_key = Ed25519PrivateKey.generate()
    key_path.write_text(private_key.private_bytes_raw().hex() + "\n", encoding="ascii")
    key_path.chmod(0o644)
    with pytest.raises(Phase0AttestationError, match="0600"):
        load_private_signing_key(key_path)

    key_path.chmod(0o600)
    loaded = load_private_signing_key(key_path)
    assert loaded.public_key().public_bytes_raw() == private_key.public_key().public_bytes_raw()
