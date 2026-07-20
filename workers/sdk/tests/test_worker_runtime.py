from __future__ import annotations

import json
import os
import socket
import threading
import time
from array import array
from pathlib import Path

import pytest
from clipmill.shm.v1 import shm_pb2
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk.framing import encode_frame, recv_frame, send_frame
from clipmill_worker_sdk.identity import WorkerIdentity, registration_preimage
from clipmill_worker_sdk.shared_memory import _receive_memfd_descriptor, validate_descriptor
from clipmill_worker_sdk.staging import StagingArea, validate_artifact_path
from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey


def identity_file(path: Path) -> WorkerIdentity:
    key = Ed25519PrivateKey.generate()
    private = key.private_bytes(
        serialization.Encoding.Raw,
        serialization.PrivateFormat.Raw,
        serialization.NoEncryption(),
    )
    path.write_text(
        json.dumps(
            {
                "key_version": "clipmill.worker.identity.v1",
                "private_key": private.hex(),
                "worker_id": "wrk_01J00000000000000000000000",
            }
        )
    )
    path.chmod(0o600)
    return WorkerIdentity.load(path)


def test_identity_signs_fresh_challenge_and_canonical_capabilities(tmp_path: Path) -> None:
    identity = identity_file(tmp_path / "identity.json")
    challenge = worker_pb2.RegistrationChallenge(
        nonce=b"x" * 32,
        supported_protocol_versions=["1.1", "1.0"],
    )
    descriptor = identity.signed_descriptor(
        challenge,
        family="echo",
        capabilities=("demo-left", "demo-right"),
        protocol_version="1.1",
        backend="cpu",
        max_memory_bytes=1024,
    )
    identity.private_key.public_key().verify(
        descriptor.signature,
        registration_preimage(challenge, descriptor),
    )
    changed = worker_pb2.RegistrationChallenge(
        nonce=b"y" * 32,
        supported_protocol_versions=["1.1", "1.0"],
    )
    with pytest.raises(InvalidSignature):
        identity.private_key.public_key().verify(
            descriptor.signature,
            registration_preimage(changed, descriptor),
        )


def test_framing_handles_socket_fragmentation() -> None:
    sender, receiver = socket.socketpair()
    try:
        request = worker_pb2.WorkerRequest(work_request=worker_pb2.WorkRequest(max_wait_ms=25))
        send_frame(sender, request)
        assert recv_frame(receiver, worker_pb2.WorkerRequest) == request
    finally:
        sender.close()
        receiver.close()


def test_staging_rejects_traversal_symlinks_and_undeclared_files(tmp_path: Path) -> None:
    root = tmp_path / "stg_01J00000000000000000000000"
    root.mkdir(mode=0o700)
    staging = StagingArea(root.name, str(root))
    staging.write_bytes("nested/result.json", b"result")
    declaration = staging.declare("nested/result.json")
    assert declaration.byte_size == 6
    assert len(declaration.sha256) == 64
    with pytest.raises(ValueError):
        validate_artifact_path("../escape")
    with pytest.raises(ValueError):
        staging.declare("not-created.bin")
    (root / "link").symlink_to(root / "nested" / "result.json")
    with pytest.raises(ValueError):
        staging.declare("link")
    staging.abandon()
    assert not (root / "nested" / "result.json").exists()


def test_shared_memory_descriptor_rejects_overflow_and_wrong_transport() -> None:
    transport = (
        shm_pb2.TRANSPORT_TYPE_SCM_RIGHTS_MEMFD
        if os.uname().sysname == "Linux"
        else shm_pb2.TRANSPORT_TYPE_POSIX_SHM
    )
    descriptor = shm_pb2.BufferDescriptor(
        shm_name="/cm_01J00000000000000000000000" if os.uname().sysname == "Darwin" else "",
        shape=[4],
        dtype=shm_pb2.DATA_TYPE_U8,
        timebase={"num": 1, "den": 90_000},
        byte_len=4,
        sha256="00" * 32,
        lease_id="lse_01J00000000000000000000000",
        transport_type=transport,
        handle_token="shm_" + "0" * 64,
    )
    validate_descriptor(descriptor)
    descriptor.shape[:] = [1 << 63, 3]
    with pytest.raises(ValueError, match="overflows"):
        validate_descriptor(descriptor)


def test_memfd_descriptor_reassembles_fragmented_frame() -> None:
    sender, receiver = socket.socketpair()
    read_descriptor, write_descriptor = os.pipe()
    expected = shm_pb2.BufferDescriptor(
        shape=[4],
        dtype=shm_pb2.DATA_TYPE_U8,
        byte_len=4,
        lease_id="lse_01J00000000000000000000000",
        handle_token="shm_" + "0" * 64,
    )
    framed = encode_frame(expected)

    def send_tail() -> None:
        time.sleep(0.01)
        sender.sendall(framed[1:])

    try:
        sender.sendmsg(
            [framed[:1]],
            [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array("i", [read_descriptor]))],
        )
        thread = threading.Thread(target=send_tail)
        thread.start()
        served, received_descriptor = _receive_memfd_descriptor(receiver)
        thread.join()
        assert served == expected
        os.fstat(received_descriptor)
        os.close(received_descriptor)
    finally:
        os.close(read_descriptor)
        os.close(write_descriptor)
        sender.close()
        receiver.close()
