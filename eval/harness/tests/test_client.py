import socket
import threading
import uuid
from pathlib import Path

import pytest
from clipmill.ipc.v1 import daemon_pb2
from clipmill_eval.client import DaemonClient, DaemonClientError, FramingError, _encode_varint


def serve_once(socket_path: Path, response: daemon_pb2.Response) -> threading.Thread:
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    listener.bind(str(socket_path))
    listener.listen(1)

    def serve() -> None:
        with listener:
            connection, _address = listener.accept()
            with connection:
                length = _read_varint(connection)
                request = daemon_pb2.Request.FromString(_read_exact(connection, length))
                response.request_id = request.request_id
                payload = response.SerializeToString(deterministic=True)
                connection.sendall(_encode_varint(len(payload)) + payload)

    thread = threading.Thread(target=serve)
    thread.start()
    return thread


def short_socket_path() -> Path:
    return Path("/tmp") / f"clipmill-eval-{uuid.uuid4().hex[:12]}.sock"


def test_control_client_uses_varint_protobuf_framing() -> None:
    socket_path = short_socket_path()
    thread = serve_once(
        socket_path,
        daemon_pb2.Response(
            health=daemon_pb2.HealthResponse(
                daemon_version="fixture",
                started_unix_millis=1,
                local_lock=True,
            )
        ),
    )
    health = DaemonClient(socket_path).health()
    thread.join(timeout=2)
    socket_path.unlink(missing_ok=True)
    assert health.daemon_version == "fixture"
    assert health.local_lock


def test_daemon_errors_remain_structured() -> None:
    socket_path = short_socket_path()
    thread = serve_once(
        socket_path,
        daemon_pb2.Response(
            error=daemon_pb2.Error(
                code=daemon_pb2.ERROR_CODE_POLICY_DENIED,
                message="denied",
            )
        ),
    )
    with pytest.raises(DaemonClientError, match="denied") as captured:
        DaemonClient(socket_path).health()
    thread.join(timeout=2)
    socket_path.unlink(missing_ok=True)
    assert captured.value.code == daemon_pb2.ERROR_CODE_POLICY_DENIED


def test_control_client_rejects_the_wrong_response_body() -> None:
    socket_path = short_socket_path()
    thread = serve_once(
        socket_path,
        daemon_pb2.Response(),
    )
    with pytest.raises(FramingError, match="expected health"):
        DaemonClient(socket_path).health()
    thread.join(timeout=2)
    socket_path.unlink(missing_ok=True)


def _read_varint(connection: socket.socket) -> int:
    value = 0
    for index in range(10):
        byte = _read_exact(connection, 1)[0]
        value |= (byte & 0x7F) << (index * 7)
        if byte & 0x80 == 0:
            return value
    raise AssertionError("bad test frame")


def _read_exact(connection: socket.socket, length: int) -> bytes:
    output = bytearray()
    while len(output) < length:
        output.extend(connection.recv(length - len(output)))
    return bytes(output)
