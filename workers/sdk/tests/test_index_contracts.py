"""The evidence index's contract, Python leg.

No Python writes this document — the index is derived in the daemon. What this
leg is for is the checks the generated Rust type carries but does not enforce:
pydantic holds the `const` schema version and the numeric bounds, and those
bounds are what stop a derivation bug from publishing a confidence above one or
a topic with no sentences in it.
"""

import json
from pathlib import Path

import pytest
from clipmill_worker_sdk.gen.schemas.index_transcript import IndexTranscript
from pydantic import ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures" / "index.transcript"


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


def test_valid_fixtures_roundtrip_canonically() -> None:
    paths = sorted((FIXTURES / "valid").glob("*.json"))
    assert paths, "index.transcript has no valid fixtures"
    for path in paths:
        raw = path.read_text(encoding="utf-8")
        parsed = IndexTranscript.model_validate_json(raw)
        reserialized = parsed.model_dump(mode="json", exclude_none=True)
        assert canonical(reserialized) == raw, f"{path.name} did not round-trip"


def test_invalid_fixtures_are_rejected() -> None:
    paths = sorted((FIXTURES / "invalid").glob("*.json"))
    assert paths, "index.transcript has no invalid fixtures"
    for path in paths:
        with pytest.raises(ValidationError):
            IndexTranscript.model_validate_json(path.read_text(encoding="utf-8"))


def valid(name: str) -> dict:
    return json.loads((FIXTURES / "valid" / name).read_text(encoding="utf-8"))


def test_a_confidence_outside_the_unit_interval_is_refused() -> None:
    """typify carries `minimum`/`maximum` as documentation. A derivation that
    let a raw score through would produce numbers a ranking stage would then
    compare against real probabilities."""

    document = valid("ten_words.json")
    document["utterances"][0]["confidence"]["p50"] = 1.5
    with pytest.raises(ValidationError):
        IndexTranscript.model_validate(document)


def test_a_topic_with_no_sentences_is_refused() -> None:
    """An empty unit is a unit nobody can walk back to words."""

    document = valid("ten_words.json")
    document["topics"][0]["sentence_count"] = 0
    with pytest.raises(ValidationError):
        IndexTranscript.model_validate(document)


def test_a_negative_speaking_rate_is_refused() -> None:
    document = valid("ten_words.json")
    document["utterances"][0]["words_per_minute"] = -1.0
    with pytest.raises(ValidationError):
        IndexTranscript.model_validate(document)


def test_the_schema_version_is_fixed() -> None:
    document = valid("ten_words.json")
    document["schema_version"] = "clipmill.index.transcript.v2"
    with pytest.raises(ValidationError):
        IndexTranscript.model_validate(document)
