"""The worker's half of the Windows handshake, and the Windows payload file.

These run on every platform: the handshake and the file are what Windows
uses, but nothing in them needs Windows to be checked.
"""

from __future__ import annotations

import json
import secrets
import socket
import sys
import threading
from pathlib import Path

import pytest
from clipmill.shm.v1 import shm_pb2
from clipmill_worker_sdk import endpoint, shared_memory

SECRET = bytes(range(32))


def serve_handshake(stream: socket.socket, secret: bytes) -> bytes | None:
    """The daemon's half, as crates/clipmilld/src/endpoint.rs runs it: the
    worker's proof when it is right, None when it is not."""
    greeting = _recv_exact(stream, len(endpoint.HELLO) + 32)
    assert greeting.startswith(endpoint.HELLO)
    worker_nonce = greeting[len(endpoint.HELLO) :]
    daemon_nonce = secrets.token_bytes(32)
    proof = endpoint.prove(secret, endpoint.SERVER_ROLE, worker_nonce, daemon_nonce)
    stream.sendall(daemon_nonce + proof)
    try:
        answer = _recv_exact(stream, 32)
    except ConnectionError:
        return None
    expected = endpoint.prove(secret, endpoint.CLIENT_ROLE, daemon_nonce, worker_nonce)
    return answer if answer == expected else None


def _recv_exact(stream: socket.socket, length: int) -> bytes:
    data = bytearray()
    while len(data) < length:
        chunk = stream.recv(length - len(data))
        if not chunk:
            raise ConnectionError("closed")
        data.extend(chunk)
    return bytes(data)


def test_proofs_match_the_daemon_byte_for_byte() -> None:
    # The same inputs and outputs as the Rust test
    # proofs_match_the_worker_kit_byte_for_byte.
    worker, daemon = bytes([1] * 32), bytes([2] * 32)
    assert (
        endpoint.prove(SECRET, endpoint.SERVER_ROLE, worker, daemon).hex()
        == "0eda7a24e885c375f511c1a32327029a8e23df26efeb1006f026764cae75b8c6"
    )
    assert (
        endpoint.prove(SECRET, endpoint.CLIENT_ROLE, daemon, worker).hex()
        == "b2b7c9f91249e72f9a732057c9890eabb86fb08fbfda216fe85c5b186b12866b"
    )
    assert endpoint.HELLO == b"CMILL/1\n"


def test_a_worker_with_the_secret_completes_the_handshake() -> None:
    worker, daemon = socket.socketpair()
    with worker, daemon:
        result: list[bytes | None] = []
        server = threading.Thread(target=lambda: result.append(serve_handshake(daemon, SECRET)))
        server.start()
        endpoint.handshake(worker, SECRET)
        server.join(timeout=5)
        assert result and result[0] is not None


def test_a_worker_refuses_a_daemon_that_cannot_prove_itself_and_says_nothing_more() -> None:
    worker, impostor = socket.socketpair()
    with worker, impostor:
        received = bytearray()

        def pretend() -> None:
            _recv_exact(impostor, len(endpoint.HELLO) + 32)
            impostor.sendall(bytes(64))
            impostor.settimeout(1)
            try:
                while chunk := impostor.recv(64):
                    received.extend(chunk)
            except (TimeoutError, OSError):
                pass

        server = threading.Thread(target=pretend)
        server.start()
        with pytest.raises(ConnectionRefusedError):
            endpoint.handshake(worker, SECRET)
        worker.close()
        server.join(timeout=5)
        assert not received


def test_an_endpoint_file_is_read_and_refused_when_it_is_not_one(tmp_path: Path) -> None:
    path = tmp_path / "workers.endpoint"
    path.write_text(
        json.dumps({"schema_version": endpoint.SCHEMA, "port": 50123, "secret": SECRET.hex()}),
        encoding="utf-8",
    )
    assert endpoint.read_endpoint(path) == (50123, SECRET)
    for bad in (
        {"schema_version": "other", "port": 1, "secret": SECRET.hex()},
        {"schema_version": endpoint.SCHEMA, "port": 0, "secret": SECRET.hex()},
        {"schema_version": endpoint.SCHEMA, "port": True, "secret": SECRET.hex()},
        {"schema_version": endpoint.SCHEMA, "port": 1, "secret": "abcd"},
    ):
        path.write_text(json.dumps(bad), encoding="utf-8")
        with pytest.raises(ValueError):
            endpoint.read_endpoint(path)
    with pytest.raises(ConnectionRefusedError):
        endpoint.read_endpoint(tmp_path / "missing.endpoint")


def test_connect_on_windows_reads_the_file_and_proves_itself_over_loopback(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    listener = socket.create_server(("127.0.0.1", 0))
    port = listener.getsockname()[1]
    path = tmp_path / "workers.endpoint"
    path.write_text(
        json.dumps({"schema_version": endpoint.SCHEMA, "port": port, "secret": SECRET.hex()}),
        encoding="utf-8",
    )
    result: list[bytes | None] = []

    def daemon() -> None:
        with listener:
            accepted, _ = listener.accept()
            with accepted:
                result.append(serve_handshake(accepted, SECRET))
                accepted.sendall(b"frame")

    server = threading.Thread(target=daemon)
    server.start()
    monkeypatch.setattr(sys, "platform", "win32")
    with endpoint.connect(path, timeout=5) as stream:
        assert _recv_exact(stream, 5) == b"frame"
    server.join(timeout=5)
    assert result and result[0] is not None


def private_descriptor(path: Path, data: bytes) -> shm_pb2.BufferDescriptor:
    return shm_pb2.BufferDescriptor(
        shm_name=str(path),
        byte_len=len(data),
        transport_type=shm_pb2.TRANSPORT_TYPE_PRIVATE_FILE,
    )


def test_a_windows_payload_is_copied_into_the_workers_own_memory(tmp_path: Path) -> None:
    folder = tmp_path / "clipmill-shm"
    folder.mkdir()
    path = folder / "cm_01J9ZZZZZZZZZZZZZZZZZZZZZZ"
    path.write_bytes(b"arrow")
    mapping = shared_memory._copy_private_file(private_descriptor(path, b"arrow"))
    try:
        assert mapping[:] == b"arrow"
        # The file is closed: the daemon may now delete it.
        path.unlink()
        assert mapping[:] == b"arrow"
    finally:
        mapping.close()


def test_a_windows_payload_of_the_wrong_size_is_refused(tmp_path: Path) -> None:
    folder = tmp_path / "clipmill-shm"
    folder.mkdir()
    path = folder / "cm_01J9ZZZZZZZZZZZZZZZZZZZZZZ"
    path.write_bytes(b"arrow and more")
    with pytest.raises(ValueError):
        shared_memory._copy_private_file(private_descriptor(path, b"arrow"))


def test_only_the_daemons_payload_folder_is_accepted(tmp_path: Path) -> None:
    good = tmp_path / "clipmill-shm" / "cm_01J9ZZZZZZZZZZZZZZZZZZZZZZ"
    assert shared_memory._private_file_path(str(good))
    assert not shared_memory._private_file_path("cm_01J9ZZZZZZZZZZZZZZZZZZZZZZ")
    assert not shared_memory._private_file_path(str(tmp_path / "elsewhere" / good.name))
    assert not shared_memory._private_file_path(str(good.parent / "cm_notaulid"))
