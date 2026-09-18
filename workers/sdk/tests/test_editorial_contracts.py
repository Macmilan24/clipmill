"""The editorial contracts, Python leg.

The editorial worker is what writes proposals, judgments, looks and the trace,
so this is the leg that matters for authoring: every valid fixture round-trips
through the generated models byte for byte, every invalid one is refused, and
the two refusals the worker must make itself are stated — a reply the model
gave that names a timestamp or a score is not a document, whatever else it
says.
"""

import json
from pathlib import Path

import pytest
from clipmill_worker_sdk.gen.schemas.editorial_judgments import EditorialJudgments
from clipmill_worker_sdk.gen.schemas.editorial_looks import EditorialLooks
from clipmill_worker_sdk.gen.schemas.editorial_proposals import EditorialProposals
from clipmill_worker_sdk.gen.schemas.editorial_trace import EditorialTrace
from clipmill_worker_sdk.gen.schemas.editorial_windows import EditorialWindows
from pydantic import BaseModel, ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures"

KINDS: dict[str, type[BaseModel]] = {
    "editorial.windows": EditorialWindows,
    "editorial.proposals": EditorialProposals,
    "editorial.judgments": EditorialJudgments,
    "editorial.looks": EditorialLooks,
    "editorial.trace": EditorialTrace,
}


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


@pytest.mark.parametrize("kind", sorted(KINDS))
def test_valid_fixtures_roundtrip_canonically(kind: str) -> None:
    model = KINDS[kind]
    paths = sorted((FIXTURES / kind / "valid").glob("*.json"))
    assert paths, f"{kind} has no valid fixtures"
    for path in paths:
        raw = path.read_text(encoding="utf-8")
        parsed = model.model_validate_json(raw)
        reserialized = parsed.model_dump(mode="json", exclude_none=True)
        assert canonical(reserialized) == raw, f"{path.name} did not round-trip"


@pytest.mark.parametrize("kind", sorted(KINDS))
def test_invalid_fixtures_are_rejected(kind: str) -> None:
    model = KINDS[kind]
    paths = sorted((FIXTURES / kind / "invalid").glob("*.json"))
    assert paths, f"{kind} has no invalid fixtures"
    for path in paths:
        with pytest.raises(ValidationError):
            model.model_validate_json(path.read_text(encoding="utf-8"))


def test_a_proposal_cannot_carry_a_timestamp_and_a_judgment_cannot_carry_a_score() -> None:
    """The two things a model reply must never become a document with."""

    proposals = json.loads(
        (FIXTURES / "editorial.proposals" / "valid" / "talk.json").read_text(encoding="utf-8")
    )
    proposals["windows"][0]["proposals"][0]["start_ticks"] = 90_000
    with pytest.raises(ValidationError):
        EditorialProposals.model_validate(proposals)

    judgments = json.loads(
        (FIXTURES / "editorial.judgments" / "valid" / "talk.json").read_text(encoding="utf-8")
    )
    judgments["candidates"][0]["display_score"] = 99
    with pytest.raises(ValidationError):
        EditorialJudgments.model_validate(judgments)


def test_a_window_the_model_could_not_answer_is_not_an_empty_answer() -> None:
    proposals = EditorialProposals.model_validate_json(
        (FIXTURES / "editorial.proposals" / "valid" / "talk.json").read_text(encoding="utf-8")
    )
    by_status = {window.status.value: window for window in proposals.windows}
    assert by_status["none"].proposals == []
    assert by_status["none"].failure is None
    assert by_status["malformed"].proposals == []
    assert by_status["malformed"].failure is not None
