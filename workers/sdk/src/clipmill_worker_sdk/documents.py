"""Serializing an artifact's JSON document.

The bytes are the artifact. Their digest is its address, the cache is a lookup
on that address, and two runs that agree about the content but disagree about
key order or indentation produce two artifacts and one wasted encode. So the
form is fixed here, once, in the shape the committed fixtures already use.
"""

from __future__ import annotations

import json
from typing import Any

from pydantic import BaseModel


def canonical_bytes(document: BaseModel | dict[str, Any]) -> bytes:
    """Sorted keys, two-space indent, trailing newline, UTF-8.

    `exclude_none` matters for round-tripping: an optional field the producer
    had nothing to say about must be absent, not present and null, or the
    document stops matching what the schema's fixtures show.
    """

    value = (
        document.model_dump(mode="json", exclude_none=True)
        if isinstance(document, BaseModel)
        else document
    )
    return (json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n").encode("utf-8")


__all__ = ["canonical_bytes"]
