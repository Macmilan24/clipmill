"""The Recording Memory: what a recording is about, read once (architecture §4.3).

Two bounded passes, so a small model on a small machine can do it:

1. **Read**, one window of the annotated script at a time: the events in it
   (with the sentences they span) and the people and things they name. The
   model may cite only the window's own sentences.
2. **Link**, over the list of every event: chapters as runs of events, and the
   links between distant events that make stitched clips possible. The model
   may cite only event ids it was shown.

The model chooses; code does the rest. Ids, times and speakers come from the
script's units, entities are merged across windows by name and alias, chapters
are made to tile the recording, and anything that fails a check is recorded
under `dropped` with the reason rather than silently lost. The Rust reader
(`clipmill_mind::memory::check`) applies the same rules to whatever it loads.
"""

from __future__ import annotations

from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field, create_model

from .inference import call, prompt_digest

EVENT_KINDS = (
    "story",
    "claim",
    "insight",
    "joke",
    "question",
    "answer",
    "explanation",
    "demo_step",
    "reveal",
    "hot_take",
    "reaction",
    "promise",
    "other",
)
MOODS = ("funny", "serious", "tense", "emotional", "inspiring", "surprising", "neutral")
ENTITY_KINDS = ("person", "organization", "product", "place", "work", "concept", "other")
LINK_KINDS = (
    "question_answer",
    "setup_payoff",
    "callback",
    "claim_evidence",
    "promise_delivery",
    "contradiction",
)
# How many words of an event's first sentence the link pass sees.
OPENING_WORDS = 14

Text80 = Annotated[str, Field(min_length=1, max_length=80)]


class ReadEntity(BaseModel):
    model_config = ConfigDict(extra="forbid")
    name: Text80
    kind: Literal[ENTITY_KINDS]
    aliases: list[Text80] = Field(max_length=5)


class LinkChapterBase(BaseModel):
    model_config = ConfigDict(extra="forbid")
    title: Text80
    summary: str = Field(min_length=1, max_length=300)


class LinkBase(BaseModel):
    model_config = ConfigDict(extra="forbid")
    kind: Literal[LINK_KINDS]
    note: str = Field(min_length=1, max_length=200)


def read_reply(units: list[str]):
    """The read pass's answer type, citing only this window's sentences."""

    unit = Literal[tuple(units)]
    event = create_model(
        "ReadEvent",
        __config__=ConfigDict(extra="forbid"),
        kind=(Literal[EVENT_KINDS], ...),
        title=(str, Field(min_length=1, max_length=100)),
        mood=(Literal[MOODS], ...),
        first_unit=(unit, ...),
        last_unit=(unit, ...),
        names=(list[Text80], Field(max_length=6)),
    )
    return create_model(
        "ReadReply",
        __config__=ConfigDict(extra="forbid"),
        events=(list[event], Field(max_length=16)),
        entities=(list[ReadEntity], Field(max_length=16)),
    )


def link_reply(events: list[str]):
    """The link pass's answer type, citing only events that exist."""

    event = Literal[tuple(events)]
    chapter = create_model(
        "LinkChapter",
        __base__=LinkChapterBase,
        first_event=(event, ...),
        last_event=(event, ...),
    )
    link = create_model(
        "LinkItem",
        __base__=LinkBase,
        from_event=(event, ...),
        to_event=(event, ...),
    )
    return create_model(
        "LinkReply",
        __config__=ConfigDict(extra="forbid"),
        chapters=(list[chapter], Field(max_length=30)),
        links=(list[link], Field(max_length=40)),
    )


def _readable(schema: dict, model: str, fields: tuple[str, ...], what: str) -> dict:
    """The schema as the prompt shows it: id fields described, not enumerated.

    The grammar still holds the model to the exact list; the prompt does not
    need hundreds of ids spelled out to say so.
    """

    for name in fields:
        schema["$defs"][model]["properties"][name] = {"type": "string", "description": what}
    return schema


def _unit_id(index: int) -> str:
    return f"s{index}"


def _unit_index(unit: str) -> int:
    return int(unit[1:])


def _opening(text: str) -> str:
    words = text.split()
    return " ".join(words[:OPENING_WORDS]) + ("…" if len(words) > OPENING_WORDS else "")


def _key(name: str) -> str:
    """A name as entities are matched: case, spaces and punctuation ignored,
    so "KV cache" and "KVCache" are one thing."""
    return "".join(character for character in name.casefold() if character.isalnum())


def _stamp(ticks: int) -> str:
    seconds = ticks // 90_000
    hours, minutes, seconds = seconds // 3600, seconds // 60 % 60, seconds % 60
    return f"{hours}:{minutes:02}:{seconds:02}" if hours else f"{minutes:02}:{seconds:02}"


class Builder:
    """Collects both passes' answers and turns them into the memory document."""

    def __init__(self, script: dict) -> None:
        self.units = {unit["index"]: unit for unit in script["units"]}
        self.order = sorted(self.units)
        self.position = {index: position for position, index in enumerate(self.order)}
        self.events: list[dict] = []
        self.entities: list[dict] = []
        self.dropped: list[dict] = []

    def drop(self, item: str, code: str, message: str) -> None:
        self.dropped.append({"item": item, "code": code, "message": message[:400]})

    # The read pass.

    def add_window(self, window_number: int, answer: dict) -> None:
        for event in answer["events"]:
            first = _unit_index(event["first_unit"])
            last = _unit_index(event["last_unit"])
            if self.position[first] > self.position[last]:
                self.drop(
                    f"window {window_number}",
                    "backwards_span",
                    f"An event ran from {event['first_unit']} back to {event['last_unit']}.",
                )
                continue
            self.events.append({**event, "first": first, "last": last})
        for entity in answer["entities"]:
            self._merge_entity(entity)

    def _merge_entity(self, entity: dict) -> None:
        spellings = {_key(entity["name"]), *(_key(a) for a in entity["aliases"])}
        spellings.discard("")
        for known in self.entities:
            if spellings & known["keys"]:
                known["keys"] |= spellings
                for alias in [entity["name"], *entity["aliases"]]:
                    if (
                        alias.casefold() != known["name"].casefold()
                        and alias not in known["aliases"]
                    ):
                        known["aliases"].append(alias)
                return
        self.entities.append(
            {
                "name": entity["name"],
                "kind": entity["kind"],
                "aliases": [
                    a for a in entity["aliases"] if a.casefold() != entity["name"].casefold()
                ],
                "keys": spellings,
            }
        )

    def finish_events(self) -> list[dict]:
        """Events in time order with ids, times, voices and entities."""

        unique = {}
        for event in self.events:
            key = (self.position[event["first"]], self.position[event["last"]], event["kind"])
            unique.setdefault(key, event)
        ordered = [unique[key] for key in sorted(unique)]
        entity_ids = {}
        for number, entity in enumerate(self.entities, start=1):
            entity["entity_id"] = f"e{number}"
            entity["events"] = []
            for key in entity["keys"]:
                entity_ids[key] = entity
        finished = []
        for number, event in enumerate(ordered, start=1):
            event_id = f"m{number}"
            span = self.order[self.position[event["first"]] : self.position[event["last"]] + 1]
            speakers = []
            for index in span:
                speaker = self.units[index].get("speaker")
                if speaker and speaker not in speakers:
                    speakers.append(speaker)
            named = []
            for name in event["names"]:
                entity = entity_ids.get(_key(name))
                if entity and entity["entity_id"] not in named:
                    named.append(entity["entity_id"])
                    entity["events"].append(event_id)
            finished.append(
                {
                    "event_id": event_id,
                    "kind": event["kind"],
                    "title": event["title"],
                    "mood": event["mood"],
                    "first_unit": _unit_id(event["first"]),
                    "last_unit": _unit_id(event["last"]),
                    "start_ticks": self.units[event["first"]]["start_ticks"],
                    "end_ticks": self.units[event["last"]]["end_ticks"],
                    "speakers": speakers,
                    "entities": named,
                }
            )
        return finished

    # The link pass.

    def event_list(self, events: list[dict]) -> str:
        lines = []
        for event in events:
            first = self.units[_unit_index(event["first_unit"])]
            who = "/".join(event["speakers"]) or "-"
            lines.append(
                f"{event['event_id']} [{_stamp(event['start_ticks'])}] {who} "
                f"{event['kind']} ({event['mood']}): {event['title']} "
                f'- "{_opening(first["text"])}"'
            )
        return "\n".join(lines)

    def chapters(self, events: list[dict], proposed: list[dict]) -> list[dict]:
        """Chapters as runs of events, made to tile the recording."""

        place = {event["event_id"]: position for position, event in enumerate(events)}
        runs = []
        for number, chapter in enumerate(proposed, start=1):
            first, last = place[chapter["first_event"]], place[chapter["last_event"]]
            if first > last:
                self.drop(f"chapter {number}", "backwards_span", "A chapter ended before it began.")
                continue
            runs.append((first, last, chapter))
        runs.sort(key=lambda run: (run[0], run[1]))
        kept = []
        for first, last, chapter in runs:
            if kept and first <= kept[-1][1]:
                self.drop(
                    chapter["title"][:64],
                    "chapters_overlap",
                    f"The chapter {chapter['title']!r} began inside the one before it.",
                )
                continue
            kept.append((first, last, chapter))
        finished = []
        for number, (first, _last, chapter) in enumerate(kept, start=1):
            start_position = (
                0 if number == 1 else self.position[_unit_index(events[first]["first_unit"])]
            )
            if number < len(kept):
                next_first = kept[number][0]
                end_position = self.position[_unit_index(events[next_first]["first_unit"])] - 1
            else:
                end_position = len(self.order) - 1
            first_unit, last_unit = self.order[start_position], self.order[end_position]
            finished.append(
                {
                    "chapter_id": f"c{number}",
                    "title": chapter["title"],
                    "summary": chapter["summary"],
                    "first_unit": _unit_id(first_unit),
                    "last_unit": _unit_id(last_unit),
                    "start_ticks": self.units[first_unit]["start_ticks"],
                    "end_ticks": self.units[last_unit]["end_ticks"],
                }
            )
        for event in events:
            first = self.position[_unit_index(event["first_unit"])]
            last = self.position[_unit_index(event["last_unit"])]
            for chapter in finished:
                lo = self.position[_unit_index(chapter["first_unit"])]
                hi = self.position[_unit_index(chapter["last_unit"])]
                if lo <= first and last <= hi:
                    event["chapter_id"] = chapter["chapter_id"]
        return finished

    def links(self, events: list[dict], proposed: list[dict]) -> list[dict]:
        span = {
            event["event_id"]: (
                self.position[_unit_index(event["first_unit"])],
                self.position[_unit_index(event["last_unit"])],
            )
            for event in events
        }
        finished = []
        seen = set()
        for number, link in enumerate(proposed, start=1):
            source, target = span[link["from_event"]], span[link["to_event"]]
            label = f"link {number}"
            if source[0] <= target[1] and target[0] <= source[1]:
                self.drop(
                    label,
                    "linked_to_itself",
                    f"{link['from_event']} and {link['to_event']} share sentences.",
                )
            elif link["kind"] != "contradiction" and source[0] > target[0]:
                self.drop(
                    label,
                    "backwards_link",
                    f"A {link['kind']} link ran from {link['from_event']} back to "
                    f"{link['to_event']}.",
                )
            elif (key := (link["kind"], link["from_event"], link["to_event"])) not in seen:
                seen.add(key)
                finished.append(
                    {
                        "link_id": f"l{len(finished) + 1}",
                        "kind": link["kind"],
                        "from_event": link["from_event"],
                        "to_event": link["to_event"],
                        "note": link["note"],
                    }
                )
        return finished

    def entity_documents(self) -> list[dict]:
        return [
            {
                "entity_id": entity["entity_id"],
                "name": entity["name"],
                "kind": entity["kind"],
                "aliases": entity["aliases"],
                "events": entity["events"],
            }
            for entity in self.entities
        ]


def build_memory(
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
    """Read a recording's script into a `clipmill.mind.memory.v1` document."""

    builder = Builder(script)
    windows = script["windows"]
    for number, window in enumerate(windows, start=1):
        progress("memory_windows", number - 1, len(windows))
        cancellation.raise_if_cancelled()
        units = [_unit_id(index) for index in window["units"]]
        reply = read_reply(units)
        prompt_schema = _readable(
            reply.model_json_schema(),
            "ReadEvent",
            ("first_unit", "last_unit"),
            "a sentence id from citable_units",
        )
        answer, failure = call(
            runtime,
            "memory.read",
            {"citable_units": f"{units[0]} to {units[-1]}", "script": window["text"]},
            reply,
            max_tokens,
            {**identity, "window_index": number - 1},
            traces,
            prompt_schema=prompt_schema,
        )
        if failure:
            builder.drop(f"window {number}", "unread_window", failure["detail"])
            continue
        builder.add_window(number, answer)
    progress("memory_windows", len(windows), len(windows))

    events = builder.finish_events()
    chapters, links = [], []
    if events:
        cancellation.raise_if_cancelled()
        reply = link_reply([event["event_id"] for event in events])
        schema = reply.model_json_schema()
        _readable(schema, "LinkChapter", ("first_event", "last_event"), "an event id")
        _readable(schema, "LinkItem", ("from_event", "to_event"), "an event id")
        answer, failure = call(
            runtime,
            "memory.link",
            {"events": builder.event_list(events)},
            reply,
            max_tokens,
            identity,
            traces,
            prompt_schema=schema,
        )
        if failure:
            builder.drop("links", "unlinked", failure["detail"])
        else:
            chapters = builder.chapters(events, answer["chapters"])
            links = builder.links(events, answer["links"])

    return {
        "schema_version": "clipmill.mind.memory.v1",
        "source_fingerprint": source_fingerprint,
        "script": {
            "level": script["level"],
            "digest": script["digest"],
            "windows": max(1, len(windows)),
        },
        "producer": {
            "stage": "mind-memory",
            "implementation": implementation,
            "model": model,
            "prompt_versions": [
                f"{operation}.v1/{prompt_digest(operation)}"
                for operation in ("memory.read", "memory.link")
            ],
        },
        "chapters": chapters,
        "events": events,
        "entities": builder.entity_documents(),
        "links": links,
        "dropped": builder.dropped,
    }


__all__ = ["Builder", "build_memory", "link_reply", "read_reply"]
