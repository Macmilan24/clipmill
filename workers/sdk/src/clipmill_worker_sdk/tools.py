"""Running the pinned executable the daemon named, and no other.

Weights are not the only versioned input a stage can have. A worker that
decodes video is holding a second one: two FFmpeg builds hand a detector
different pixels, so a stage that resolved its own decoder from the PATH would
publish an observation under an address that says nothing about what produced
it — and the address would be identical on a machine with a different build.

So the daemon names the binary on the lease, having fetched and verified it
against `bom.toml`, and states its build identity. The build identity travels
separately into the stage payload, which is what puts it in the artifact key;
the path does not, because a path is machine-specific and would give the same
recording two addresses on two machines.

What is checked here is narrower than `weights.verify_model`, and deliberately
so. A pinned model is checked against per-file digests from an independent
manifest, which is an authority the worker can hold the bytes to. There is no
equivalent authority for the sidecar at task time — `bom.toml` pins the archive
that was downloaded, not the binary extracted from it — so re-hashing here
would only compare the daemon's file against itself. Claiming that as
verification would be worse than not claiming it. What is checked is what can
be: that the lease named an absolute path to a real, regular, executable file
that nothing has replaced with a link.
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path

from clipmill.worker.v1 import worker_pb2


class ToolUnavailableError(RuntimeError):
    """A lease named a tool that is missing, or is not something to execute."""


@dataclass(frozen=True, slots=True)
class VerifiedTool:
    name: str
    path: Path
    bom: str


def verify_tool(binding: worker_pb2.ToolBinding) -> VerifiedTool:
    """Check one binding names something this worker may actually spawn."""

    if not binding.name:
        raise ToolUnavailableError("tool binding names no tool")
    if not binding.bom:
        raise ToolUnavailableError(f"{binding.name} binding states no build identity")
    if not binding.path:
        raise ToolUnavailableError(f"{binding.name} binding states no path")
    path = Path(binding.path)
    if not path.is_absolute():
        raise ToolUnavailableError(
            f"{binding.name} is at a relative path, which would resolve against "
            "whatever directory this worker happens to be in"
        )
    # A symlink is refused rather than followed. The daemon staged a specific
    # binary; a link at that path means something else answered to its name,
    # and following it would run whatever that is.
    if path.is_symlink():
        raise ToolUnavailableError(f"{binding.name} at {binding.path} is a symbolic link")
    if not path.is_file():
        raise ToolUnavailableError(f"{binding.name} at {binding.path} is not a regular file")
    if not os.access(path, os.X_OK):
        raise ToolUnavailableError(f"{binding.name} at {binding.path} is not executable")
    return VerifiedTool(name=binding.name, path=path, bom=binding.bom)


def require_tool(lease: worker_pb2.TaskLease, name: str) -> VerifiedTool:
    """The one tool this lease provides under a name, checked.

    Exactly one: a stage handed two decoders has no basis for choosing, and a
    stage handed none cannot run. Both are refusals with a reason rather than a
    fallback to the PATH, which is the thing this module exists to prevent.
    """

    candidates = [binding for binding in lease.tools if binding.name == name]
    if not candidates:
        raise ToolUnavailableError(f"this lease provides no {name}")
    if len(candidates) > 1:
        raise ToolUnavailableError(
            f"this lease provides {len(candidates)} {name} binaries and names no choice"
        )
    return verify_tool(candidates[0])


__all__ = ["ToolUnavailableError", "VerifiedTool", "require_tool", "verify_tool"]
