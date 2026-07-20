"""Read-only, validated, zero-copy Arrow mappings for one-use daemon handles."""

from __future__ import annotations

import array
import ctypes
import hashlib
import mmap
import os
import re
import socket
import sys
from dataclasses import dataclass
from pathlib import Path

import pyarrow as pa
from clipmill.shm.v1 import shm_pb2

from .framing import MAX_FRAME_BYTES, _decode_varint, decode_framed_bytes, recv_frame, send_frame

_UINT64_MAX = (1 << 64) - 1
_LEASE_ID = re.compile(r"^lse_[0-9A-HJKMNP-TV-Z]{26}$")
_HANDLE_TOKEN = re.compile(r"^shm_[0-9a-f]{64}$")
_SHM_NAME = re.compile(r"^/cm_[0-9A-HJKMNP-TV-Z]{26}$")
_DTYPE_BYTES = {
    shm_pb2.DATA_TYPE_U8: 1,
    shm_pb2.DATA_TYPE_I16: 2,
    shm_pb2.DATA_TYPE_I32: 4,
    shm_pb2.DATA_TYPE_F16: 2,
    shm_pb2.DATA_TYPE_F32: 4,
}


class SharedMemorySocketError(ConnectionError):
    """The daemon shared-memory broker socket could not be reached."""


class SharedMemoryHandleError(ConnectionError):
    """The daemon-issued OS shared-memory handle could not be opened."""


@dataclass(slots=True)
class MappedBuffer:
    descriptor: shm_pb2.BufferDescriptor
    buffer: pa.Buffer | None
    _mapping: mmap.mmap

    def close(self) -> None:
        self.buffer = None
        self._mapping.close()

    def __enter__(self) -> MappedBuffer:
        return self

    def __exit__(self, *_args: object) -> None:
        self.close()


def map_shared_buffer(
    socket_path: Path,
    descriptor: shm_pb2.BufferDescriptor,
) -> MappedBuffer:
    validate_descriptor(descriptor)
    stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    stream.settimeout(10)
    mapping: mmap.mmap | None = None
    try:
        try:
            stream.connect(str(socket_path))
        except OSError as error:
            raise SharedMemorySocketError("shared-memory broker unavailable") from error
        send_frame(
            stream,
            shm_pb2.MapRequest(
                lease_id=descriptor.lease_id,
                handle_token=descriptor.handle_token,
            ),
        )
        if sys.platform.startswith("linux"):
            served, file_descriptor = _receive_memfd_descriptor(stream)
            try:
                if served != descriptor:
                    raise ValueError("shared-memory descriptor changed during transfer")
                mapping = mmap.mmap(
                    file_descriptor,
                    served.byte_len,
                    access=mmap.ACCESS_READ,
                )
            finally:
                os.close(file_descriptor)
        elif sys.platform == "darwin":
            served = recv_frame(stream, shm_pb2.BufferDescriptor)
            if served != descriptor:
                raise ValueError("shared-memory descriptor changed during transfer")
            try:
                file_descriptor = _shm_open_read_only(served.shm_name)
            except OSError as error:
                raise SharedMemoryHandleError("POSIX shared-memory handle unavailable") from error
            try:
                mapping = mmap.mmap(
                    file_descriptor,
                    served.byte_len,
                    access=mmap.ACCESS_READ,
                )
            finally:
                os.close(file_descriptor)
        else:
            raise RuntimeError("shared memory is supported only on macOS and Linux")
        if served != descriptor:
            raise ValueError("shared-memory descriptor changed during transfer")
        validate_mapping(served, mapping)
        arrow_buffer = pa.py_buffer(mapping)
        send_frame(
            stream,
            shm_pb2.MapAcknowledgement(
                lease_id=served.lease_id,
                handle_token=served.handle_token,
                mapped=True,
            ),
        )
        return MappedBuffer(served, arrow_buffer, mapping)
    except Exception:
        if mapping is not None:
            mapping.close()
        raise
    finally:
        stream.close()


def validate_descriptor(descriptor: shm_pb2.BufferDescriptor) -> None:
    if _LEASE_ID.fullmatch(descriptor.lease_id) is None:
        raise ValueError("invalid shared-memory lease ID")
    if _HANDLE_TOKEN.fullmatch(descriptor.handle_token) is None:
        raise ValueError("invalid one-use shared-memory handle")
    if descriptor.dtype not in _DTYPE_BYTES or not descriptor.shape:
        raise ValueError("invalid shared-memory dtype or shape")
    elements = 1
    for dimension in descriptor.shape:
        if dimension == 0 or elements > _UINT64_MAX // dimension:
            raise ValueError("shared-memory shape overflows")
        elements *= dimension
    if elements > _UINT64_MAX // _DTYPE_BYTES[descriptor.dtype]:
        raise ValueError("shared-memory byte length overflows")
    if elements * _DTYPE_BYTES[descriptor.dtype] != descriptor.byte_len:
        raise ValueError("shared-memory shape does not match byte length")
    if (
        descriptor.timebase.num <= 0
        or descriptor.timebase.den <= 0
        or descriptor.timebase.den > 1_000_000_000
    ):
        raise ValueError("invalid shared-memory timebase")
    if len(descriptor.sha256) != 64 or any(
        character not in "0123456789abcdef" for character in descriptor.sha256
    ):
        raise ValueError("invalid shared-memory digest")
    if sys.platform.startswith("linux"):
        if descriptor.transport_type != shm_pb2.TRANSPORT_TYPE_SCM_RIGHTS_MEMFD:
            raise ValueError("Linux requires an SCM_RIGHTS memfd transport")
        if descriptor.shm_name:
            raise ValueError("Linux memfd descriptors must not contain a POSIX name")
    elif sys.platform == "darwin":
        if descriptor.transport_type != shm_pb2.TRANSPORT_TYPE_POSIX_SHM:
            raise ValueError("macOS requires a POSIX shared-memory transport")
        if _SHM_NAME.fullmatch(descriptor.shm_name) is None:
            raise ValueError("invalid POSIX shared-memory name")


def _receive_memfd_descriptor(
    stream: socket.socket,
) -> tuple[shm_pb2.BufferDescriptor, int]:
    received, ancillary, flags, _address = stream.recvmsg(
        MAX_FRAME_BYTES + 10,
        socket.CMSG_SPACE(array.array("i").itemsize * 2),
    )
    descriptors = array.array("i")
    try:
        for level, kind, payload in ancillary:
            if level == socket.SOL_SOCKET and kind == socket.SCM_RIGHTS:
                aligned = len(payload) - len(payload) % descriptors.itemsize
                descriptors.frombytes(payload[:aligned])
        if flags & (socket.MSG_CTRUNC | socket.MSG_TRUNC) or len(descriptors) != 1:
            raise ValueError("SCM_RIGHTS response did not contain exactly one descriptor")

        framed = bytearray(received)
        while True:
            try:
                length, prefix = _decode_varint(framed)
                break
            except ValueError:
                if len(framed) >= 10:
                    raise
                framed.extend(_recv_required(stream, 1))
        if length <= 0 or length > MAX_FRAME_BYTES:
            raise ValueError("invalid shared-memory frame length")
        total = prefix + length
        if len(framed) > total:
            raise ValueError("shared-memory descriptor contained trailing bytes")
        framed.extend(_recv_required(stream, total - len(framed)))
        served = decode_framed_bytes(bytes(framed), shm_pb2.BufferDescriptor)
        return served, descriptors[0]
    except Exception:
        for descriptor in descriptors:
            os.close(descriptor)
        raise


def _recv_required(stream: socket.socket, length: int) -> bytes:
    chunks = bytearray()
    while len(chunks) < length:
        chunk = stream.recv(length - len(chunks))
        if not chunk:
            raise ConnectionError("shared-memory socket closed")
        chunks.extend(chunk)
    return bytes(chunks)


def validate_mapping(descriptor: shm_pb2.BufferDescriptor, mapping: mmap.mmap) -> None:
    if len(mapping) != descriptor.byte_len:
        raise ValueError("mapped shared-memory length changed")
    if hashlib.sha256(mapping).hexdigest() != descriptor.sha256:
        raise ValueError("mapped shared-memory digest changed")


def _shm_open_read_only(name: str) -> int:
    library = ctypes.CDLL(None, use_errno=True)
    shm_open = library.shm_open
    # `shm_open` is variadic on Darwin. Declaring the optional mode argument
    # as fixed uses the wrong arm64 calling convention; read-only opens pass
    # only the two required arguments.
    shm_open.argtypes = [ctypes.c_char_p, ctypes.c_int]
    shm_open.restype = ctypes.c_int
    descriptor = shm_open(name.encode(), os.O_RDONLY)
    if descriptor < 0:
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error), name)
    return descriptor
