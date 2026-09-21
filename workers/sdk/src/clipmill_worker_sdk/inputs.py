"""Resolve lease inputs by their manifest-declared artifact kinds.

Standalone plans and dependency tasks deliver the same input list, keeping
artifact keys consistent and reads within the lease's authorization. Duplicate
inputs of one kind are rejected because their order cannot identify the right
one."""

from __future__ import annotations

from dataclasses import dataclass

from .artifacts import ArtifactVerificationError, VerifiedArtifact


class MissingInputError(RuntimeError):
    """A lease did not deliver an artifact the stage cannot run without."""


@dataclass(frozen=True, slots=True)
class ResolvedInput:
    """One artifact this lease delivered, opened and verified."""

    kind: str
    artifact_id: str
    artifact: VerifiedArtifact


class LeaseInputs:
    """Everything a lease delivered, indexed by kind.

    Opened once. A stage with three inputs would otherwise open the same
    artifacts repeatedly just to ask what they are, and every open re-hashes the
    payload it verifies.
    """

    def __init__(self, context) -> None:
        self._by_kind: dict[str, ResolvedInput] = {}
        for artifact_id in context.lease.input_artifact_ids:
            artifact = context.open_artifact(artifact_id)
            if artifact.kind in self._by_kind:
                raise MissingInputError(
                    f"this lease delivered two {artifact.kind} inputs and names no choice"
                )
            self._by_kind[artifact.kind] = ResolvedInput(
                kind=artifact.kind, artifact_id=artifact_id, artifact=artifact
            )

    def require(self, kind: str) -> ResolvedInput:
        """The one input of this kind, or a refusal naming what was delivered.

        A refusal rather than a fallback: a stage that guessed which of several
        inputs was meant is how a worker ends up reading last week's artifact and
        publishing it under this week's key.
        """

        found = self._by_kind.get(kind)
        if found is None:
            delivered = ", ".join(sorted(self._by_kind)) or "nothing"
            raise MissingInputError(f"this lease delivered {delivered}, not a {kind}")
        return found

    def optional(self, kind: str) -> ResolvedInput | None:
        """The same, for an input that may legitimately be absent.

        Absent is a fact the stage reports, not a failure it hides: a source with
        no video has no shot cuts, and that is a different observation from one
        whose shot detection was never run.
        """

        return self._by_kind.get(kind)

    def kinds(self) -> tuple[str, ...]:
        return tuple(sorted(self._by_kind))


def require_input(context, kind: str) -> ResolvedInput:
    """One input of a kind, for a stage that reads exactly one artifact.

    The `ArtifactVerificationError` a bad store raises is not caught here. It
    means the object the daemon pointed at is not what its manifest says, which
    is neither this stage's mistake nor something it can work around.
    """

    return LeaseInputs(context).require(kind)


__all__ = [
    "ArtifactVerificationError",
    "LeaseInputs",
    "MissingInputError",
    "ResolvedInput",
    "require_input",
]
