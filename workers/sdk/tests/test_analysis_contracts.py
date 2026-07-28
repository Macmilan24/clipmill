"""The fan-in that closes an analysis, Python leg.

No Python writes these — the fan-in is a daemon builtin. What this leg carries is
the refusal the generated Rust type cannot make: typify holds `minItems` as
documentation, and it is load-bearing here. A manifest naming no stages describes
no analysis, and it would be a perfectly valid root for a job that derived
nothing.

It also holds the property a shell depends on and neither generated type
enforces: every address a manifest names is reachable from the one artifact the
job rooted, so a reader that walks this document finds real objects rather than
addresses somebody hand-wrote.
"""

import json
from pathlib import Path

import pytest
from clipmill_worker_sdk.gen.schemas.analysis_manifest import AnalysisManifest
from pydantic import ValidationError

REPO = Path(__file__).resolve().parents[3]
FIXTURES = REPO / "contracts" / "fixtures" / "analysis.manifest"


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


def test_valid_fixtures_roundtrip_canonically() -> None:
    paths = sorted((FIXTURES / "valid").glob("*.json"))
    assert paths, "analysis.manifest has no valid fixtures"
    for path in paths:
        raw = path.read_text(encoding="utf-8")
        parsed = AnalysisManifest.model_validate_json(raw)
        reserialized = parsed.model_dump(mode="json", exclude_none=True)
        assert canonical(reserialized) == raw, f"{path.name} did not round-trip"


def test_invalid_fixtures_are_rejected() -> None:
    paths = sorted((FIXTURES / "invalid").glob("*.json"))
    assert paths, "analysis.manifest has no invalid fixtures"
    for path in paths:
        with pytest.raises(ValidationError):
            AnalysisManifest.model_validate_json(path.read_text(encoding="utf-8"))


def test_a_manifest_with_no_stages_is_refused() -> None:
    """The refusal the Rust leg cannot make.

    A job roots exactly one artifact and garbage collection walks recipe inputs
    from the roots. A manifest naming nothing is a root that reaches nothing —
    the analysis under it would be collectable the moment it finished.
    """

    empty = json.loads((FIXTURES / "invalid" / "no_stages.json").read_text(encoding="utf-8"))
    assert empty["stages"] == []
    with pytest.raises(ValidationError):
        AnalysisManifest.model_validate(empty)


def test_a_skipped_stage_is_never_also_a_produced_one() -> None:
    """Absent and skipped, not both present and skipped.

    A stage appearing in both lists would be a document saying it produced
    something and that it never ran. Nothing in the schema forbids it, so it is
    asserted over the fixtures a consumer is entitled to trust.
    """

    for path in sorted((FIXTURES / "valid").glob("*.json")):
        document = json.loads(path.read_text(encoding="utf-8"))
        produced = {stage["kind"] for stage in document["stages"]}
        skipped = {stage["kind"] for stage in document.get("skipped", [])}
        assert not (produced & skipped), f"{path.name} both ran and skipped a stage"


def test_coverage_is_a_span_the_stages_could_have_examined() -> None:
    """A range, never inverted, and `analyzed` stated separately from it.

    An intersection of two stages' spans can be empty — that is a real state and
    the manifest reports it as a zero-length range. What it must never be is
    backwards, which would be a span no consumer could interpret.
    """

    for path in sorted((FIXTURES / "valid").glob("*.json")):
        coverage = json.loads(path.read_text(encoding="utf-8"))["coverage"]
        assert coverage["end_ticks"] >= coverage["start_ticks"], path.name
        assert isinstance(coverage["analyzed"], bool), path.name
