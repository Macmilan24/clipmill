"""Verify public wording that can be traced to a particular unplayed source line."""

from __future__ import annotations

import re
from itertools import pairwise
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, create_model

WORDS = re.compile(r"[^\W_]+", re.UNICODE)
COMMON = set(
    WORDS.findall(
        "a an and are as at be by for from in into is it of on or that the their then these this "
        "to was with your they its our we you what how about after before when"
    )
)
EDITORIAL = set(
    WORDS.findall(
        "actionable actual argument claim clip cut demonstrates discusses edit example features "
        "includes included including instruction mentions moment passage point presenter shows "
        "speaker specific statement talks"
    )
)


def terms(text, skip=COMMON):
    words = [w for w in WORDS.findall(text.casefold()) if w not in skip]
    return [w[:-3] if len(w) >= 6 and w.endswith("ing") else w for w in words]


def source_witness(text, transcript, playback):
    heard = {word for line in playback for word in terms(line["text"])}
    words = terms(text, COMMON | EDITORIAL)
    missing = list(dict.fromkeys(w for w in words if w not in heard))
    if len(missing) < 2 or len(missing) * 3 < len(set(words)) * 2:
        return None
    played = {line["unit"] for line in playback}
    for i, unit in enumerate(transcript.units):
        if f"s{unit['index']}" in played:
            continue
        source = terms(unit["text"])
        if len(missing) == 2:
            # Two words must be a distinctive phrase in this exact order.
            if tuple(missing) in pairwise(source):
                return transcript.line(i)
        else:
            remaining = iter(source)
            if all(word in remaining for word in missing):
                return transcript.line(i)
    return None


class CopySupport(BaseModel):
    model_config = ConfigDict(extra="forbid")
    played_quote: str = Field(max_length=300)
    heard_claim: str = Field(max_length=180)
    finding: Literal["same_claim_plays", "claim_only_in_unplayed_line", "uncertain"]


def quoted_in_playback(quote, playback):
    quote = " ".join(quote.casefold().split())
    spoken = " ".join(" ".join(line["text"].casefold().split()) for line in playback)
    return len(WORDS.findall(quote)) >= 2 and quote in spoken


def check_copy(ctx, text, transcript, playback, titles=(), notes=None):
    """Check a possible source and, during a repair, choose a grounded replacement."""
    witness = source_witness(text, transcript, playback)
    if witness is None:
        return None, None, set()
    data = {"public_wording": text, "playback": playback, "unplayed_line": witness}
    fields = {}
    if titles:
        data["source_titles"] = titles
        fields["source_title"] = (Literal[tuple(titles)], ...)
    if notes:
        data["public_notes"] = notes
        fields["unsupported_note_ids"] = (list[Literal[tuple(notes)]], ...)
    answer, failure = ctx.ask(
        "critic.copy", data, create_model("CopyCheck", __base__=CopySupport, **fields)
    )
    if failure or not answer:
        return witness, None, set()
    if answer["finding"] == "same_claim_plays" and quoted_in_playback(
        answer["played_quote"], playback
    ):
        return None, None, set()
    return witness, answer.get("source_title"), set(answer.get("unsupported_note_ids", []))


class OmissionCheck(BaseModel):
    model_config = ConfigDict(extra="forbid")
    finding: Literal["named_statement_plays", "omission_is_possible"]
    played_quote: str = Field(max_length=300)


def contradicted_omission(ctx, note, playback):
    match = re.match(r"^(?:omits?|excludes?|skips?|removes?|leaves? out)\s+(.+)", note, re.I)
    if not match:
        return False
    # First locate the named subject in retained speech. A related topic alone
    # does not establish that the specific allegedly omitted point was heard.
    skip = COMMON | EDITORIAL | {"first", "initial", "original"}
    named = {w.removesuffix("s") for w in terms(match[1], skip)}
    if len(named) < 2:
        return False
    for i in range(len(playback)):
        nearby = playback[i : i + 3]
        heard = {w.removesuffix("s") for line in nearby for w in terms(line["text"])}
        if named <= heard:
            answer, failure = ctx.ask(
                "critic.omission",
                {"check": "omission_note", "public_note": note, "played_excerpt": nearby},
                OmissionCheck,
            )
            return bool(
                not failure
                and answer
                and answer["finding"] == "named_statement_plays"
                and quoted_in_playback(answer["played_quote"], nearby)
            )
    return False
