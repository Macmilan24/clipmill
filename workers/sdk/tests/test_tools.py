"""A worker runs the binary the daemon named, or it does not run.

The failure this guards against is quiet rather than loud: an unpinned decoder
on the PATH would work perfectly, produce slightly different pixels, and
publish them under a content address asserting the pinned build produced them.
Nothing downstream could tell. So the refusals below are all refusals to fall
back — there is no branch here that ends in "use whatever is available".
"""

from __future__ import annotations

import os
import stat
from pathlib import Path

import pytest
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk.tools import ToolUnavailableError, require_tool, verify_tool

BOM = "ffmpeg-8.1.2-btb-n8.1.2"


def executable(root: Path, name: str = "ffmpeg") -> Path:
    path = root / name
    path.write_bytes(b"#!/bin/sh\nexit 0\n")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)
    return path


def binding(path: Path, name: str = "ffmpeg", bom: str = BOM) -> worker_pb2.ToolBinding:
    return worker_pb2.ToolBinding(name=name, path=str(path), bom=bom)


def test_a_pinned_executable_resolves(tmp_path: Path) -> None:
    path = executable(tmp_path)
    tool = verify_tool(binding(path))
    assert tool.name == "ffmpeg"
    assert tool.path == path
    assert tool.bom == BOM


def test_a_binding_without_a_build_identity_is_refused(tmp_path: Path) -> None:
    """The identity is what reaches the artifact key. A binding that omits it
    would let a stage publish without saying what decoded the footage."""

    with pytest.raises(ToolUnavailableError, match="build identity"):
        verify_tool(binding(executable(tmp_path), bom=""))


def test_a_relative_path_is_refused(tmp_path: Path) -> None:
    executable(tmp_path)
    with pytest.raises(ToolUnavailableError, match="relative path"):
        verify_tool(worker_pb2.ToolBinding(name="ffmpeg", path="ffmpeg", bom=BOM))


def test_a_symlink_is_refused_rather_than_followed(tmp_path: Path) -> None:
    real = executable(tmp_path, "ffmpeg-real")
    link = tmp_path / "ffmpeg"
    link.symlink_to(real)
    # The link resolves to a perfectly good executable. That is the point: the
    # daemon staged a specific file, and something else is answering for it.
    with pytest.raises(ToolUnavailableError, match="symbolic link"):
        verify_tool(binding(link))


def test_a_directory_or_a_missing_file_is_refused(tmp_path: Path) -> None:
    directory = tmp_path / "ffmpeg"
    directory.mkdir()
    with pytest.raises(ToolUnavailableError, match="not a regular file"):
        verify_tool(binding(directory))
    with pytest.raises(ToolUnavailableError, match="not a regular file"):
        verify_tool(binding(tmp_path / "nothing-here"))


def test_a_file_nobody_may_execute_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "ffmpeg"
    path.write_bytes(b"not a program")
    path.chmod(0o600)
    with pytest.raises(ToolUnavailableError, match="not executable"):
        verify_tool(binding(path))


def test_a_lease_naming_no_tool_does_not_fall_back_to_the_path(tmp_path: Path) -> None:
    """The whole point. A worker whose lease forgot the decoder must fail the
    task, not quietly succeed with a decoder nobody pinned."""

    lease = worker_pb2.TaskLease(task_id="t", kind="detect-shots")
    assert os.environ.get("PATH")  # a PATH exists, and is not consulted
    with pytest.raises(ToolUnavailableError, match="provides no ffmpeg"):
        require_tool(lease, "ffmpeg")


def test_two_bindings_under_one_name_are_refused(tmp_path: Path) -> None:
    first = executable(tmp_path, "one")
    second = executable(tmp_path, "two")
    lease = worker_pb2.TaskLease(
        task_id="t",
        kind="detect-shots",
        tools=[binding(first), binding(second)],
    )
    with pytest.raises(ToolUnavailableError, match="names no choice"):
        require_tool(lease, "ffmpeg")


def test_the_named_tool_is_the_one_returned(tmp_path: Path) -> None:
    ffmpeg = executable(tmp_path, "ffmpeg")
    ffprobe = executable(tmp_path, "ffprobe")
    lease = worker_pb2.TaskLease(
        task_id="t",
        kind="detect-shots",
        tools=[binding(ffmpeg), binding(ffprobe, name="ffprobe")],
    )
    assert require_tool(lease, "ffprobe").path == ffprobe
    assert require_tool(lease, "ffmpeg").path == ffmpeg
