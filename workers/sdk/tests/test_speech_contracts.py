"""The speech chain's contracts, Python leg.

Workers are what actually write these documents, so this leg carries the
checks the Rust types cannot make: pydantic enforces the `const` fields that
typify only carries, and those constants are the ones stopping a recognizer's
token positions from ever being mistaken for measured word timing.
"""

import json
from pathlib import Path

import pytest
from clipmill_worker_sdk.gen.schemas.speech_alignment import SpeechAlignment
from clipmill_worker_sdk.gen.schemas.speech_asr import SpeechAsr
from clipmill_worker_sdk.gen.schemas.speech_transcript import SpeechTranscript
from clipmill_worker_sdk.gen.schemas.speech_vad import SpeechVad
from pydantic import BaseModel, ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures"

MODELS: dict[str, type[BaseModel]] = {
    "speech.vad": SpeechVad,
    "speech.asr": SpeechAsr,
    "speech.alignment": SpeechAlignment,
    "speech.transcript": SpeechTranscript,
}


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


def fixtures(kind: str, validity: str) -> list[Path]:
    return sorted((FIXTURES / kind / validity).glob("*.json"))


@pytest.mark.parametrize("kind", sorted(MODELS))
def test_valid_fixtures_roundtrip_canonically(kind: str) -> None:
    model = MODELS[kind]
    paths = fixtures(kind, "valid")
    assert paths, f"{kind} has no valid fixtures"
    for path in paths:
        raw = path.read_text(encoding="utf-8")
        parsed = model.model_validate_json(raw)
        reserialized = parsed.model_dump(mode="json", exclude_none=True)
        assert canonical(reserialized) == raw, f"{path.name} did not round-trip"


@pytest.mark.parametrize("kind", sorted(MODELS))
def test_invalid_fixtures_are_rejected(kind: str) -> None:
    model = MODELS[kind]
    paths = fixtures(kind, "invalid")
    assert paths, f"{kind} has no invalid fixtures"
    for path in paths:
        with pytest.raises(ValidationError):
            model.model_validate_json(path.read_text(encoding="utf-8"))


def test_a_recognizer_cannot_declare_its_own_timing_authoritative() -> None:
    """The refusal the Rust type cannot make.

    `timing_authority` is a fixed value, and the generated Rust type carries
    fixed values without checking them. Here it is checked, which matters
    because the workers that write these documents are Python: the shape that
    would let decoder positions pass for word timing cannot be written by the
    code most likely to write it.
    """

    raw = (
        FIXTURES / "speech.asr" / "invalid" / "decoder_claimed_as_timing_authority.json"
    ).read_text(encoding="utf-8")
    with pytest.raises(ValidationError):
        SpeechAsr.model_validate_json(raw)

    # And the only accepted value is the stage that actually measures timing.
    good = SpeechAsr.model_validate_json(
        (FIXTURES / "speech.asr" / "valid" / "two_utterances.json").read_text(encoding="utf-8")
    )
    assert good.timing_authority == "forced_alignment"


def test_every_speech_document_declares_the_schema_it_claims_to_be() -> None:
    for kind, model in MODELS.items():
        for path in fixtures(kind, "valid"):
            parsed = model.model_validate_json(path.read_text(encoding="utf-8"))
            assert parsed.schema_version == f"clipmill.{kind}.v1"
