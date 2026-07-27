"""The score card and the ranked set, Python leg.

No Python writes these — ranking runs in the daemon. What this leg carries is
the refusals the generated Rust type cannot make: typify holds `minItems` and
`maximum` as documentation, and both are load-bearing here. A percentile above
ninety-nine is not a percentile; a card with no factors is a number with no
explanation behind it; a boundary with no terms is a cut nobody can argue with.
"""

import json
from pathlib import Path

import pytest
from clipmill_worker_sdk.gen.schemas.ranking_set import RankingSet
from pydantic import ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures" / "ranking.set"


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


def valid(name: str) -> dict:
    return json.loads((FIXTURES / "valid" / name).read_text(encoding="utf-8"))


def test_valid_fixtures_roundtrip_canonically() -> None:
    paths = sorted((FIXTURES / "valid").glob("*.json"))
    assert paths, "ranking.set has no valid fixtures"
    for path in paths:
        raw = path.read_text(encoding="utf-8")
        parsed = RankingSet.model_validate_json(raw)
        reserialized = parsed.model_dump(mode="json", exclude_none=True)
        assert canonical(reserialized) == raw, f"{path.name} did not round-trip"


def test_invalid_fixtures_are_rejected() -> None:
    paths = sorted((FIXTURES / "invalid").glob("*.json"))
    assert paths, "ranking.set has no invalid fixtures"
    for path in paths:
        with pytest.raises(ValidationError):
            RankingSet.model_validate_json(path.read_text(encoding="utf-8"))


def test_a_percentile_above_ninety_nine_is_refused() -> None:
    """The displayed number is a rank within a cohort. A hundred would mean a
    clip that beat itself."""

    document = valid("interview.json")
    document["cohort"][0]["display_score"] = 100
    with pytest.raises(ValidationError):
        RankingSet.model_validate(document)


def test_a_card_with_no_factors_is_refused() -> None:
    """A score with nothing behind it is a number a user cannot argue with,
    which is the one thing the decomposition exists to prevent."""

    document = valid("interview.json")
    document["cohort"][0]["factors"] = []
    with pytest.raises(ValidationError):
        RankingSet.model_validate(document)


def test_a_boundary_with_no_terms_is_refused() -> None:
    document = valid("interview.json")
    document["cohort"][0]["boundary"]["terms"] = []
    with pytest.raises(ValidationError):
        RankingSet.model_validate(document)


def test_a_factor_value_outside_the_unit_interval_is_refused() -> None:
    document = valid("interview.json")
    document["cohort"][0]["factors"][0]["value"] = 1.5
    with pytest.raises(ValidationError):
        RankingSet.model_validate(document)


def test_a_penalty_of_zero_is_refused() -> None:
    """A penalty that subtracts nothing is not a penalty, and listing it would
    make a card look like it was marked down when it was not."""

    document = valid("interview.json")
    document["cohort"][0]["penalties"] = [{"reason": "repetition", "value": 0}]
    with pytest.raises(ValidationError):
        RankingSet.model_validate(document)


def test_the_schema_version_is_fixed() -> None:
    document = valid("interview.json")
    document["schema_version"] = "clipmill.ranking.set.v2"
    with pytest.raises(ValidationError):
        RankingSet.model_validate(document)
