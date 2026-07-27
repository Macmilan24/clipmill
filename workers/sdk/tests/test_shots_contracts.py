"""Shot detection's contract, Python leg.

The worker that writes these documents is Python, so this leg carries the
checks the generated Rust type cannot make: pydantic enforces the `const`
schema version and the numeric bounds that typify only carries. It also states
the arithmetic the worker is held to — spans tile coverage, cuts land on span
starts, and a confidence never leaves [0, 1] — because those are the properties
the boundary lattice will lean on without rechecking.
"""

import json
from pathlib import Path

import pytest
from clipmill_worker_sdk.gen.schemas.evidence_shots import EvidenceShots
from pydantic import ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures" / "evidence.shots"


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


def test_valid_fixtures_roundtrip_canonically() -> None:
    paths = sorted((FIXTURES / "valid").glob("*.json"))
    assert paths, "evidence.shots has no valid fixtures"
    for path in paths:
        raw = path.read_text(encoding="utf-8")
        parsed = EvidenceShots.model_validate_json(raw)
        reserialized = parsed.model_dump(mode="json", exclude_none=True)
        assert canonical(reserialized) == raw, f"{path.name} did not round-trip"


def test_invalid_fixtures_are_rejected() -> None:
    paths = sorted((FIXTURES / "invalid").glob("*.json"))
    assert paths, "evidence.shots has no invalid fixtures"
    for path in paths:
        with pytest.raises(ValidationError):
            EvidenceShots.model_validate_json(path.read_text(encoding="utf-8"))


def test_the_spans_tile_coverage_without_gaps_or_overlaps() -> None:
    """What a consumer is allowed to assume without checking.

    `shots` is derivable from `cuts` plus `coverage`, and is published anyway
    precisely so nobody has to derive it. That is only worth doing if the
    published version is right, so the arithmetic is asserted here rather than
    left as a comment in the schema.
    """

    document = EvidenceShots.model_validate_json(
        (FIXTURES / "valid" / "three_shots.json").read_text(encoding="utf-8")
    )
    assert document.shots[0].start_ticks == document.coverage.start_ticks
    assert document.shots[-1].end_ticks == document.coverage.end_ticks
    for earlier, later in zip(document.shots, document.shots[1:], strict=False):
        assert earlier.end_ticks == later.start_ticks
    for cut, shot in zip(document.cuts, document.shots[1:], strict=True):
        assert cut.t_ticks == shot.start_ticks


def test_a_confidence_outside_the_unit_interval_is_refused() -> None:
    """The refusal the Rust type does not make.

    typify carries `minimum`/`maximum` as documentation. A worker that mapped a
    raw content distance straight onto `p50` would emit numbers well above one,
    and the code most likely to do that is this one.
    """

    document = json.loads((FIXTURES / "valid" / "three_shots.json").read_text(encoding="utf-8"))
    document["cuts"][0]["confidence"]["p50"] = 42.5
    with pytest.raises(ValidationError):
        EvidenceShots.model_validate(document)


def test_a_zero_threshold_is_refused() -> None:
    """A threshold of zero calls every frame a cut, which is not a tuning."""

    document = json.loads((FIXTURES / "valid" / "three_shots.json").read_text(encoding="utf-8"))
    document["detection"]["threshold"] = 0
    with pytest.raises(ValidationError):
        EvidenceShots.model_validate(document)
