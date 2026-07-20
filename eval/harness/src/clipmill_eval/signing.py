"""Canonical JSON signing used by corpus, license, and run attestations."""

from __future__ import annotations

import json
from collections.abc import Mapping
from copy import deepcopy
from typing import Any

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)

SIGNATURE_FIELD = "signature"


class AttestationError(ValueError):
    """A signed document is malformed or has an invalid signature."""


def canonical_json(value: Any) -> bytes:
    """Serialize the integer/string-only attestation surface deterministically."""

    try:
        text = json.dumps(
            value,
            ensure_ascii=False,
            allow_nan=False,
            separators=(",", ":"),
            sort_keys=True,
        )
    except (TypeError, ValueError) as error:
        raise AttestationError(f"document is not canonicalizable: {error}") from error
    return text.encode("utf-8")


def signing_preimage(document: Mapping[str, Any], domain: bytes) -> bytes:
    unsigned = deepcopy(dict(document))
    unsigned.pop(SIGNATURE_FIELD, None)
    return domain + canonical_json(unsigned)


def sign_document(
    document: Mapping[str, Any],
    private_key: Ed25519PrivateKey,
    domain: bytes,
) -> dict[str, Any]:
    """Return a signed copy without exposing or serializing private material."""

    signed = deepcopy(dict(document))
    signature = private_key.sign(signing_preimage(signed, domain))
    signed[SIGNATURE_FIELD] = {
        "algorithm": "ed25519",
        "public_key": private_key.public_key().public_bytes_raw().hex(),
        "signature": signature.hex(),
    }
    return signed


def verify_document(
    document: Mapping[str, Any],
    domain: bytes,
    expected_public_key: bytes | None = None,
) -> bytes:
    """Verify an Ed25519 envelope and return its public key."""

    envelope = document.get(SIGNATURE_FIELD)
    if not isinstance(envelope, Mapping) or envelope.get("algorithm") != "ed25519":
        raise AttestationError("document omitted an Ed25519 signature envelope")
    try:
        public_key = bytes.fromhex(str(envelope["public_key"]))
        signature = bytes.fromhex(str(envelope["signature"]))
    except (KeyError, ValueError) as error:
        raise AttestationError("signature envelope contains invalid hexadecimal data") from error
    if len(public_key) != 32 or len(signature) != 64:
        raise AttestationError("signature envelope has an invalid key or signature length")
    if expected_public_key is not None and public_key != expected_public_key:
        raise AttestationError("document was signed by an unexpected key")
    try:
        Ed25519PublicKey.from_public_bytes(public_key).verify(
            signature,
            signing_preimage(document, domain),
        )
    except (InvalidSignature, ValueError) as error:
        raise AttestationError("document signature is invalid") from error
    return public_key
