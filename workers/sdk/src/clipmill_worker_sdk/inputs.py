"""Finding the artifacts a lease delivered, by what each one is.

A stage's inputs reach it by one of two routes. Submitted as a standalone job,
what it reads was published by earlier jobs, so the plan declares their content
addresses. Run inside a larger plan, those same artifacts are the outputs of
tasks it depends on, so a dependency carries them. Either way the daemon
delivers one list on the lease, and this is how a worker reads it.

That the routes converge before the worker sees them is the point. A stage that
resolved an address out of its own payload on one route and a dependency on the
other would compute two artifact keys for one piece of work — and a
content-addressed store cannot notice two addresses for one observation after
the fact. It is also the only route that *can* work: a worker may open exactly
the artifacts its lease named, so an address that travelled only in the payload
named something the worker was forbidden to read.

Inputs are matched by the artifact kind each one's own manifest declares, never
by position. A stage handed two artifacts of one kind has no basis for choosing
between them and says so, rather than taking the first and being right most of
the time.
"""

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
