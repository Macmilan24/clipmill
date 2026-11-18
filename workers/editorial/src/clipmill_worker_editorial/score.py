"""The editor's markup: hooks, landings, cuttable ranges and dependencies.

The model marks only what stands out in each window of the annotated script;
code writes the row for every sentence, so the cutter can tighten a moment
(leave out what is marked cuttable), start a clip on its strongest line, and
never open on a line that needs an earlier one.
"""

from __future__ import annotations

from typing import Literal

from pydantic import ConfigDict, Field, create_model

from .inference import call, cited_window, prompt_digest

LEVELS = ("none", "fair", "good", "strong")
CUTS = ("tangent", "retake", "false_start", "filler", "aside", "ad_read")


def markup_reply(units: list[str]):
    """The answer type for one window, citing only that window's sentences."""

    unit = Literal[tuple(units)]
    rated = create_model(
        "Rated",
        __config__=ConfigDict(extra="forbid"),
        unit=(unit, ...),
        level=(Literal[LEVELS[1:]], ...),
    )
    cut = create_model(
        "Cut",
        __config__=ConfigDict(extra="forbid"),
        first_unit=(unit, ...),
        last_unit=(unit, ...),
        why=(Literal[CUTS], ...),
    )
    need = create_model(
        "Need",
        __config__=ConfigDict(extra="forbid"),
        unit=(unit, ...),
        on=(unit, ...),
    )
    return create_model(
        "Markup",
        __config__=ConfigDict(extra="forbid"),
        hooks=(list[rated], Field(max_length=12)),
        landings=(list[rated], Field(max_length=12)),
        cuts=(list[cut], Field(max_length=16)),
        needs=(list[need], Field(max_length=16)),
    )


def _prompt_schema(reply) -> dict:
    schema = reply.model_json_schema()
    described = {"type": "string", "description": "a sentence id from citable_units"}
    for model, fields in (
        ("Rated", ("unit",)),
        ("Cut", ("first_unit", "last_unit")),
        ("Need", ("unit", "on")),
    ):
        for name in fields:
            schema["$defs"][model]["properties"][name] = described
    return schema


def apply(rows: dict[int, dict], position: dict[int, int], answer: dict) -> None:
    """Fold one window's markup into the rows for its sentences.

    A sentence rated twice keeps the higher rating; a cut or a dependency that
    runs backwards is ignored rather than guessed at.
    """

    for key in ("hooks", "landings"):
        field = key[:-1]
        for mark in answer[key]:
            row = rows[int(mark["unit"][1:])]
            if LEVELS.index(mark["level"]) > LEVELS.index(row[field]):
                row[field] = mark["level"]
                row["trim"] = "keep"
    for cut in answer["cuts"]:
        first, last = int(cut["first_unit"][1:]), int(cut["last_unit"][1:])
        width = position[last] - position[first] + 1
        if width <= 0 or (cut["why"] in ("false_start", "filler", "retake") and width > 2):
            continue
        for index, row in rows.items():
            if (
                position[first] <= position[index] <= position[last]
                and row["trim"] == "keep"
                and row["hook"] == row["landing"] == "none"
            ):
                row["trim"] = cut["why"]
    for need in answer["needs"]:
        unit, on = int(need["unit"][1:]), int(need["on"][1:])
        if position[on] < position[unit]:
            rows[unit]["needs"] = f"s{on}"


def score_units(
    runtime,
    script,
    *,
    source_fingerprint,
    implementation,
    model,
    max_tokens,
    identity,
    traces,
    cancellation,
    progress=lambda *args: None,
):
    """Mark up a recording's script into a `clipmill.mind.unit_scores.v1` document."""

    script_units = {unit["index"]: unit for unit in script["units"]}
    order = list(script_units)
    position = {index: place for place, index in enumerate(order)}
    rows = {
        index: {"unit": f"s{index}", "hook": "none", "landing": "none", "trim": "keep"}
        for index in order
    }
    unscored = []
    windows = script["windows"]
    for number, window in enumerate(windows, start=1):
        progress("score_windows", number - 1, len(windows))
        cancellation.raise_if_cancelled()
        units = [f"s{index}" for index in window["units"]]
        reply = markup_reply(units)
        answer, failure = call(
            runtime,
            "score",
            cited_window(script_units, window),
            reply,
            max_tokens,
            {**identity, "window_index": number - 1},
            traces,
            prompt_schema=_prompt_schema(reply),
        )
        if failure:
            unscored.append(
                {
                    "first_unit": units[0],
                    "last_unit": units[-1],
                    "message": failure["detail"][:400],
                }
            )
            continue
        apply(rows, position, answer)
    progress("score_windows", len(windows), len(windows))
    return {
        "schema_version": "clipmill.mind.unit_scores.v1",
        "source_fingerprint": source_fingerprint,
        "script": {
            "level": script["level"],
            "digest": script["digest"],
            "windows": max(1, len(windows)),
        },
        "producer": {
            "stage": "mind-score",
            "implementation": implementation,
            "model": model,
            "prompt_version": f"score.v1/{prompt_digest('score')}",
        },
        "units": [rows[index] for index in order],
        "unscored": unscored,
    }


__all__ = ["apply", "markup_reply", "score_units"]
