"""The worker's side of the shared-memory broker."""

from __future__ import annotations

import hashlib
import mmap
import sys
from pathlib import Path

import pytest
from clipmill.shm.v1 import shm_pb2
from clipmill_worker_sdk import shared_memory


class _Stream:
    closed = False

    def close(self) -> None:
        self.closed = True


def test_a_lost_acknowledgement_keeps_its_error_and_closes_the_mapping(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The daemon can die between serving a buffer and hearing it was mapped.
    The worker must see the connection error, which it survives by
    reconnecting, and not an error from closing a mapping still in use."""
    payload = bytes(range(16))
    descriptor = shm_pb2.BufferDescriptor(
        byte_len=len(payload), sha256=hashlib.sha256(payload).hexdigest()
    )
    stream = _Stream()
    mappings: list[mmap.mmap] = []

    def copy(served: shm_pb2.BufferDescriptor) -> mmap.mmap:
        mapping = mmap.mmap(-1, len(payload))
        mapping.write(payload)
        mapping.seek(0)
        mappings.append(mapping)
        return mapping

    def send(_stream: object, message: object) -> None:
        if isinstance(message, shm_pb2.MapAcknowledgement):
            raise BrokenPipeError(32, "Broken pipe")

    # The Windows branch copies the payload into an anonymous mapping, so the
    # test needs no broker on any platform.
    monkeypatch.setattr(sys, "platform", "win32")
    monkeypatch.setattr(shared_memory, "validate_descriptor", lambda _descriptor: None)
    monkeypatch.setattr(shared_memory.endpoint, "connect", lambda _path, timeout: stream)
    monkeypatch.setattr(shared_memory, "recv_frame", lambda _stream, _kind: descriptor)
    monkeypatch.setattr(shared_memory, "send_frame", send)
    monkeypatch.setattr(shared_memory, "_copy_private_file", copy)

    with pytest.raises(BrokenPipeError):
        shared_memory.map_shared_buffer(Path("unused"), descriptor)
    assert mappings[0].closed
    assert stream.closed
