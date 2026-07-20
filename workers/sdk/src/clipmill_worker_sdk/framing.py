"""Bounded Protobuf varint framing shared by worker protocol clients."""

from __future__ import annotations

import socket

from google.protobuf.message import Message

MAX_FRAME_BYTES = 4 * 1024 * 1024


def encode_frame(message: Message) -> bytes:
    payload = message.SerializeToString(deterministic=True)
    if not payload or len(payload) > MAX_FRAME_BYTES:
        raise ValueError("invalid worker frame length")
    return _encode_varint(len(payload)) + payload


def send_frame(stream: socket.socket, message: Message) -> None:
    stream.sendall(encode_frame(message))


def recv_frame[MessageT: Message](stream: socket.socket, message_type: type[MessageT]) -> MessageT:
    length = _read_varint(stream)
    if length <= 0 or length > MAX_FRAME_BYTES:
        raise ValueError("invalid worker frame length")
    payload = _recv_exact(stream, length)
    message = message_type()
    message.ParseFromString(payload)
    return message


def decode_framed_bytes[MessageT: Message](data: bytes, message_type: type[MessageT]) -> MessageT:
    length, prefix = _decode_varint(data)
    if length <= 0 or length > MAX_FRAME_BYTES or len(data) != prefix + length:
        raise ValueError("invalid worker frame")
    message = message_type()
    message.ParseFromString(data[prefix:])
    return message


def _encode_varint(value: int) -> bytes:
    encoded = bytearray()
    while value >= 0x80:
        encoded.append((value & 0x7F) | 0x80)
        value >>= 7
    encoded.append(value)
    return bytes(encoded)


def _decode_varint(data: bytes) -> tuple[int, int]:
    value = 0
    for index, byte in enumerate(data[:10]):
        if index == 9 and byte > 1:
            raise ValueError("malformed frame varint")
        value |= (byte & 0x7F) << (index * 7)
        if byte & 0x80 == 0:
            return value, index + 1
    raise ValueError("truncated or malformed frame varint")


def _read_varint(stream: socket.socket) -> int:
    value = 0
    for index in range(10):
        byte = _recv_exact(stream, 1)[0]
        if index == 9 and byte > 1:
            raise ValueError("malformed frame varint")
        value |= (byte & 0x7F) << (index * 7)
        if byte & 0x80 == 0:
            return value
    raise ValueError("malformed frame varint")


def _recv_exact(stream: socket.socket, length: int) -> bytes:
    chunks = bytearray()
    while len(chunks) < length:
        chunk = stream.recv(length - len(chunks))
        if not chunk:
            raise ConnectionError("worker socket closed")
        chunks.extend(chunk)
    return bytes(chunks)
