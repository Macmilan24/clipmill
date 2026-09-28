"""Use only the pinned executable named on the lease.

The daemon verifies the download against ``bom.toml``. Its build identity enters
the stage payload and artifact key; its machine-specific path does not.

Checks here require an absolute path to a regular executable without symlink
substitution. They do not verify the binary's digest: the BOM pins the downloaded
archive, not the extracted executable, so no independent task-time digest is
available."""

from __future__ import annotations

import os
import sys
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
    if not _executable(path):
        raise ToolUnavailableError(f"{binding.name} at {binding.path} is not executable")
    return VerifiedTool(name=binding.name, path=path, bom=binding.bom)


def _executable(path: Path) -> bool:
    """Whether the system would run ``path`` as a program. Windows has no
    permission bit for it (``os.access`` only reports that the file exists
    there): a program is a file the system runs by its extension."""
    if sys.platform == "win32":
        return path.suffix.lower() in {".exe", ".com"}
    return os.access(path, os.X_OK)


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
