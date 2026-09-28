"""Connect to one of the daemon's planes.

On Unix a plane is a socket in the daemon's private run directory: only its
user can open that directory, so only its user can connect. On Windows the
path names a small file holding a loopback port and a secret made for this
start of the daemon. A port is open to every process on the machine, so both
ends prove they hold the secret before anything else is said, the daemon first
(``crates/clipmilld/src/endpoint.rs`` holds the other half):

    worker -> daemon   HELLO, worker nonce
    daemon -> worker   daemon nonce, HMAC(secret, server role | worker nonce | daemon nonce)
    worker -> daemon   HMAC(secret, client role | daemon nonce | worker nonce)
"""

from __future__ import annotations

import hashlib
import hmac
import json
import secrets
import socket
import sys
from pathlib import Path

SCHEMA = "clipmill.endpoint.v1"
HELLO = b"CMILL/1\n"
SERVER_ROLE = b"clipmill endpoint server"
CLIENT_ROLE = b"clipmill endpoint client"
NONCE_BYTES = 32
PROOF_BYTES = 32


def connect(path: Path, timeout: float) -> socket.socket:
    """A connection to the plane at ``path``, with ``timeout`` on every call."""
    if sys.platform != "win32":
        stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        stream.settimeout(timeout)
        try:
            stream.connect(str(path))
        except BaseException:
            stream.close()
            raise
        return stream
    port, secret = read_endpoint(path)
    stream = socket.create_connection(("127.0.0.1", port), timeout=timeout)
    try:
        handshake(stream, secret)
    except BaseException:
        stream.close()
        raise
    return stream


def read_endpoint(path: Path) -> tuple[int, bytes]:
    """The port and secret an endpoint file names."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise ConnectionRefusedError(f"no daemon endpoint at {path}") from error
    if not isinstance(data, dict) or data.get("schema_version") != SCHEMA:
        raise ValueError("the endpoint file is not a ClipMill endpoint")
    port = data.get("port")
    if not isinstance(port, int) or isinstance(port, bool) or not 0 < port < 65536:
        raise ValueError("the endpoint file names no port")
    try:
        secret = bytes.fromhex(str(data.get("secret", "")))
    except ValueError as error:
        raise ValueError("the endpoint secret is not hex") from error
    if len(secret) != 32:
        raise ValueError("the endpoint secret is not 32 bytes")
    return port, secret


def handshake(stream: socket.socket, secret: bytes) -> None:
    """The worker's half: stop before proving anything if the daemon cannot."""
    worker_nonce = secrets.token_bytes(NONCE_BYTES)
    stream.sendall(HELLO + worker_nonce)
    reply = _recv_exact(stream, NONCE_BYTES + PROOF_BYTES)
    daemon_nonce, proof = reply[:NONCE_BYTES], reply[NONCE_BYTES:]
    if not hmac.compare_digest(proof, prove(secret, SERVER_ROLE, worker_nonce, daemon_nonce)):
        raise ConnectionRefusedError("the daemon could not prove it holds this endpoint's secret")
    stream.sendall(prove(secret, CLIENT_ROLE, daemon_nonce, worker_nonce))


def prove(secret: bytes, role: bytes, first: bytes, second: bytes) -> bytes:
    return hmac.new(secret, role + first + second, hashlib.sha256).digest()


def _recv_exact(stream: socket.socket, length: int) -> bytes:
    chunks = bytearray()
    while len(chunks) < length:
        chunk = stream.recv(length - len(chunks))
        if not chunk:
            raise ConnectionError("the daemon closed the connection during the handshake")
        chunks.extend(chunk)
    return bytes(chunks)


__all__ = ["SCHEMA", "connect", "handshake", "prove", "read_endpoint"]
