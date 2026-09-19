"""Optional local writing from the immutable captions an export actually uses."""

from __future__ import annotations

import re
import unicodedata
from typing import Annotated

from clipmill.ipc.v1 import daemon_pb2
from clipmill_worker_sdk import (
    DeterministicTaskError,
    LeaseInputs,
    RetryableTaskError,
    canonical_bytes,
    require_model,
)
from clipmill_worker_sdk.gen.schemas.edit_ir import EditIr
from google.protobuf.message import DecodeError
from pydantic import BaseModel, ConfigDict, Field, ValidationError, field_validator

from .inference import call, prompt_digest

IMPLEMENTATION = "clipmill-worker-editorial@0.2.0/metadata"
MAX_OUTPUT_TOKENS = 1024
MAX_IR_BYTES = 16 * 1024 * 1024
MAX_TRANSCRIPT_BYTES = 24_000


def validate_text(value: str, *, multiline: bool = False) -> str:
    if not value.strip() or value != value.strip():
        raise ValueError("metadata text must be nonempty and trimmed")
    if any(
        c in "<>" or (unicodedata.category(c).startswith("C") and not (multiline and c == "\n"))
        for c in value
    ):
        raise ValueError("metadata text contains unsupported characters")
    if re.search(r"https?://|www\.", value, re.IGNORECASE):
        raise ValueError("source links are appended by the application, never the model")
    return value


class MetadataReply(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)

    title: str = Field(min_length=1, max_length=100)
    description: str = Field(min_length=1, max_length=2000)
    tags: list[Annotated[str, Field(min_length=1, max_length=80)]] = Field(max_length=12)
    hashtags: list[Annotated[str, Field(pattern=r"^[A-Za-z][A-Za-z0-9_]{1,49}$")]] = Field(
        max_length=3
    )

    @field_validator("title")
    @classmethod
    def title_text(cls, value):
        return validate_text(value)

    @field_validator("description")
    @classmethod
    def description_text(cls, value):
        validate_text(value, multiline=True)
        if len(value.encode("utf-8")) > 4000:
            raise ValueError("description exceeds 4000 UTF-8 bytes")
        return value

    @field_validator("tags")
    @classmethod
    def tag_text(cls, values):
        for value in values:
            validate_text(value)
            if any(c in value for c in ',"#'):
                raise ValueError("tags must not contain commas, quotes, or hashtags")
        # Match the uploader/UI's conservative count: one separator per tag,
        # including the last, and quotes around tags containing spaces.
        size = sum(len(value.encode("utf-8")) + 1 + (2 if " " in value else 0) for value in values)
        if size > 500:
            raise ValueError("tags exceed the 500-byte combined limit")
        if len({value.casefold() for value in values}) != len(values):
            raise ValueError("tags must be distinct")
        return values

    @field_validator("hashtags")
    @classmethod
    def unique_hashtags(cls, values):
        if len({value.casefold() for value in values}) != len(values):
            raise ValueError("hashtags must be distinct")
        return values


def transcript_from_ir(raw: bytes) -> str:
    """Read only the saved reading cues; burn-in is a second presentation."""
    if len(raw) > MAX_IR_BYTES:
        raise DeterministicTaskError("The saved edit is too large for metadata writing")
    try:
        document = EditIr.model_validate_json(raw)
    except ValidationError as error:
        raise DeterministicTaskError("The saved edit is invalid; export the clip again") from error
    transcript = " ".join(
        word.text
        for cue in document.captions.cues or []
        for line in cue.lines
        for word in line.words
    ).strip()
    if not transcript:
        raise DeterministicTaskError(
            "This clip has no saved caption text; add captions or write the metadata manually"
        )
    if len(transcript.encode("utf-8")) > MAX_TRANSCRIPT_BYTES:
        raise DeterministicTaskError(
            "This clip's full transcript exceeds the metadata writing limit; write it manually"
        )
    return transcript


def execute_metadata(context) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    try:
        payload = daemon_pb2.YoutubeMetadataTaskPayloadV1.FromString(context.lease.payload)
    except DecodeError as error:
        raise DeterministicTaskError("invalid YouTube metadata lease") from error
    if (
        context.lease.kind != "youtube-metadata"
        or payload.key_version != "clipmill.youtube-metadata.v1"
        or payload.prompt_digest != prompt_digest("metadata")
        or payload.max_output_tokens != MAX_OUTPUT_TOKENS
    ):
        raise DeterministicTaskError(
            "YouTube metadata prompt or decoding policy differs from the planned recipe; "
            "update the worker"
        )
    edit_input = LeaseInputs(context).require("edit.ir.v1")
    if payload.ir_artifact_id != edit_input.artifact_id:
        raise DeterministicTaskError("YouTube metadata lease names a different saved edit")
    with context.artifact_file(edit_input.artifact, "edit-ir.json").open("rb") as source:
        transcript = transcript_from_ir(source.read(MAX_IR_BYTES + 1))
    context.cancellation.raise_if_cancelled()
    model = require_model(context.lease, "editorial")
    from .runtime import LocalModel

    runtime = None
    try:
        context.report_progress("metadata", 0, 1)
        runtime = LocalModel(model.root, context.cancellation)
        traces = []
        answer, failure = call(
            runtime,
            "metadata",
            {"clip_transcript": transcript},
            MetadataReply,
            MAX_OUTPUT_TOKENS,
            {"route": "local", "model": {"name": model.name, "digest": model.digest}},
            traces,
        )
        if failure:
            error_type = (
                RetryableTaskError
                if failure["failure_class"] == "transient"
                else DeterministicTaskError
            )
            raise error_type(failure["detail"])
        context.cancellation.raise_if_cancelled()
        document = {
            "schema_version": "clipmill.publishing.metadata.v1",
            "ir_artifact_id": edit_input.artifact_id,
            "producer": {
                "implementation": IMPLEMENTATION,
                "model": {"name": model.name, "digest": model.digest},
                "prompt_digest": payload.prompt_digest,
                "max_output_tokens": MAX_OUTPUT_TOKENS,
            },
            "metadata": answer,
        }
        # No success-shaped artifact is written on malformed replies or cancellation.
        context.staging.write_bytes("metadata.json", canonical_bytes(document))
        context.report_progress("metadata", 1, 1)
        return ("metadata.json",)
    finally:
        if runtime is not None:
            runtime.close()
