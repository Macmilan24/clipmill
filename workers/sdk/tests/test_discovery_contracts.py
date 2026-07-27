"""The candidate contract, Python leg.

No Python writes this document — discovery runs in the daemon. What this leg
carries is the refusals the generated Rust type cannot make: typify holds
`minItems` as documentation, and the empty-array cases are exactly the ones
that would turn a nomination into a claim nobody can check. A candidate with no
evidence cannot be explained; a lattice with no starts offers ranking nothing to
search; a candidate with no interval is not a clip.
"""

import json
from pathlib import Path

import pytest
from clipmill_worker_sdk.gen.schemas.discovery_candidates import DiscoveryCandidates
from pydantic import ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures" / "discovery.candidates"


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


def valid(name: str) -> dict:
    return json.loads((FIXTURES / "valid" / name).read_text(encoding="utf-8"))


def test_valid_fixtures_roundtrip_canonically() -> None:
    paths = sorted((FIXTURES / "valid").glob("*.json"))
    assert paths, "discovery.candidates has no valid fixtures"
    for path in paths:
        raw = path.read_text(encoding="utf-8")
        parsed = DiscoveryCandidates.model_validate_json(raw)
        reserialized = parsed.model_dump(mode="json", exclude_none=True)
        assert canonical(reserialized) == raw, f"{path.name} did not round-trip"


def test_invalid_fixtures_are_rejected() -> None:
    paths = sorted((FIXTURES / "invalid").glob("*.json"))
    assert paths, "discovery.candidates has no invalid fixtures"
    for path in paths:
        with pytest.raises(ValidationError):
            DiscoveryCandidates.model_validate_json(path.read_text(encoding="utf-8"))


def test_a_candidate_with_no_evidence_is_refused() -> None:
    """Rule 14.1 as a type constraint. A nomination nobody can walk back to the
    words is one ranking cannot defend, and the schema refuses to express it."""

    document = valid("interview.json")
    document["candidates"][0]["evidence"] = []
    with pytest.raises(ValidationError):
        DiscoveryCandidates.model_validate(document)


def test_a_lattice_with_no_boundaries_is_refused() -> None:
    """Discovery's promise is that ranking never has to search. An empty
    lattice is that promise unkept."""

    document = valid("interview.json")
    document["candidates"][0]["boundary_lattice"]["starts"] = []
    with pytest.raises(ValidationError):
        DiscoveryCandidates.model_validate(document)


def test_a_candidate_with_no_interval_is_refused() -> None:
    document = valid("interview.json")
    document["candidates"][0]["intervals"] = []
    with pytest.raises(ValidationError):
        DiscoveryCandidates.model_validate(document)


def test_a_rejection_counted_zero_times_is_refused() -> None:
    """A reason recorded as never firing reads like a term that was checked and
    passed. The terms this phase cannot measure are absent, not zero."""

    document = valid("interview.json")
    document["candidates"][0]["boundary_lattice"]["phi_rejects"] = [
        {"reason": "mid_word", "count": 0}
    ]
    with pytest.raises(ValidationError):
        DiscoveryCandidates.model_validate(document)


def test_a_preliminary_score_outside_the_unit_interval_is_refused() -> None:
    document = valid("interview.json")
    document["candidates"][0]["prelim_score"] = 4.2
    with pytest.raises(ValidationError):
        DiscoveryCandidates.model_validate(document)


def test_the_schema_version_is_fixed() -> None:
    document = valid("interview.json")
    document["schema_version"] = "clipmill.discovery.candidates.v2"
    with pytest.raises(ValidationError):
        DiscoveryCandidates.model_validate(document)
