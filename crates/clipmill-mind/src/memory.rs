//! Checks on a Recording Memory before anything plans from it.
//!
//! The worker that writes a memory constrains the model to cite only units and
//! events it was shown, and derives every id and time in code, so a memory it
//! publishes should pass. These checks are for the reader: a memory is read
//! from disk by the scouts, by memory search and by agents, and whatever wrote
//! it, a link that runs backwards or an event outside its chapter would send a
//! stitch to the wrong place. Every problem is returned at once.

use std::collections::{BTreeMap, BTreeSet};

use clipmill_contracts::schemas::mind_memory::{LinkKind, MindMemory};

use crate::script::Source;
use crate::{Violation, violation};

/// Where a cited unit sits: its position in the recording and its times.
#[derive(Clone, Copy)]
struct Placed {
    index: u64,
    start: u64,
    end: u64,
}

fn place(units: &BTreeMap<u64, Placed>, unit: &str) -> Option<Placed> {
    let index = unit.strip_prefix('s')?.parse::<u64>().ok()?;
    units.get(&index).copied()
}

/// A span of units, checked to exist, run forward, and carry the times the
/// units have.
fn span(
    units: &BTreeMap<u64, Placed>,
    at: &str,
    (first, last): (&str, &str),
    (start, end): (u64, u64),
    problems: &mut Vec<Violation>,
) -> Option<(Placed, Placed)> {
    let (Some(a), Some(b)) = (place(units, first), place(units, last)) else {
        problems.push(violation(
            "unknown_unit",
            Some(at),
            format!("{first}..{last} cites a sentence the recording does not have"),
        ));
        return None;
    };
    if a.index > b.index {
        problems.push(violation(
            "backwards_span",
            Some(at),
            format!("{first} comes after {last}"),
        ));
        return None;
    }
    if (start, end) != (a.start, b.end) {
        problems.push(violation(
            "wrong_times",
            Some(at),
            format!(
                "says {start}..{end} but {first}..{last} runs {}..{}",
                a.start, b.end
            ),
        ));
    }
    Some((a, b))
}

fn unique<'a>(
    kind: &str,
    ids: impl Iterator<Item = &'a str>,
    problems: &mut Vec<Violation>,
) -> BTreeSet<&'a str> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            problems.push(violation(
                "duplicate_id",
                Some(id),
                format!("two {kind} have the id {id}"),
            ));
        }
    }
    seen
}

/// Every reason `memory` cannot be planned from as written, given the
/// recording it describes.
pub fn check(memory: &MindMemory, source: &Source) -> Vec<Violation> {
    let units: BTreeMap<u64, Placed> = source
        .sentences
        .iter()
        .map(|s| {
            (
                s.index,
                Placed {
                    index: s.index,
                    start: s.start_ticks,
                    end: s.end_ticks,
                },
            )
        })
        .collect();
    let mut problems = Vec::new();
    let chapters = check_chapters(memory, &units, &mut problems);
    let events = check_events(memory, &units, &chapters, &mut problems);
    check_links(memory, &events, &mut problems);
    problems
}

/// Chapters: known units, the times those units have, in order, no overlaps.
/// Returns each chapter's first and last unit.
fn check_chapters<'m>(
    memory: &'m MindMemory,
    units: &BTreeMap<u64, Placed>,
    problems: &mut Vec<Violation>,
) -> BTreeMap<&'m str, (u64, u64)> {
    unique(
        "chapters",
        memory.chapters.iter().map(|c| c.chapter_id.as_str()),
        problems,
    );
    let mut chapters = BTreeMap::new();
    let mut previous: Option<(String, u64)> = None;
    for chapter in &memory.chapters {
        let id = chapter.chapter_id.as_str();
        let Some((first, last)) = span(
            units,
            id,
            (chapter.first_unit.as_str(), chapter.last_unit.as_str()),
            (chapter.start_ticks, chapter.end_ticks.get()),
            problems,
        ) else {
            continue;
        };
        if let Some((before, end)) = &previous
            && first.index <= *end
        {
            problems.push(violation(
                "chapters_overlap",
                Some(id),
                format!("starts at s{} inside or before {before}", first.index),
            ));
        }
        previous = Some((id.to_owned(), last.index));
        chapters.insert(id, (first.index, last.index));
    }
    chapters
}

/// Events: known units inside their chapter, and entities that resolve both
/// ways. Returns each event's first and last unit.
fn check_events<'m>(
    memory: &'m MindMemory,
    units: &BTreeMap<u64, Placed>,
    chapters: &BTreeMap<&str, (u64, u64)>,
    problems: &mut Vec<Violation>,
) -> BTreeMap<&'m str, (u64, u64)> {
    let entity_ids = unique(
        "entities",
        memory.entities.iter().map(|e| e.entity_id.as_str()),
        problems,
    );
    let event_ids = unique(
        "events",
        memory.events.iter().map(|e| e.event_id.as_str()),
        problems,
    );
    let mut events = BTreeMap::new();
    for event in &memory.events {
        let id = event.event_id.as_str();
        let placed = span(
            units,
            id,
            (event.first_unit.as_str(), event.last_unit.as_str()),
            (event.start_ticks, event.end_ticks.get()),
            problems,
        );
        if let Some((first, last)) = placed {
            events.insert(id, (first.index, last.index));
            if let Some(chapter) = &event.chapter_id {
                match chapters.get(chapter.as_str()) {
                    None => problems.push(violation(
                        "unknown_chapter",
                        Some(id),
                        format!(
                            "names chapter {}, which the memory does not have",
                            **chapter
                        ),
                    )),
                    Some(&(a, b)) if first.index < a || last.index > b => {
                        problems.push(violation(
                            "outside_chapter",
                            Some(id),
                            format!(
                                "runs s{}..s{} outside {}",
                                first.index, last.index, **chapter
                            ),
                        ));
                    }
                    Some(_) => {}
                }
            }
        }
        for entity in &event.entities {
            if !entity_ids.contains(entity.as_str()) {
                problems.push(violation(
                    "unknown_entity",
                    Some(id),
                    format!("names entity {}, which the memory does not have", **entity),
                ));
            }
        }
    }
    for entity in &memory.entities {
        for event in &entity.events {
            if !event_ids.contains(event.as_str()) {
                problems.push(violation(
                    "unknown_event",
                    Some(&entity.entity_id),
                    format!("names event {}, which the memory does not have", **event),
                ));
            }
        }
    }
    events
}

/// Links: both events known, apart, and forward in time unless the link is a
/// contradiction.
fn check_links(
    memory: &MindMemory,
    events: &BTreeMap<&str, (u64, u64)>,
    problems: &mut Vec<Violation>,
) {
    unique(
        "links",
        memory.links.iter().map(|l| l.link_id.as_str()),
        problems,
    );
    for link in &memory.links {
        let id = link.link_id.as_str();
        let (Some(from), Some(to)) = (
            events.get(link.from_event.as_str()),
            events.get(link.to_event.as_str()),
        ) else {
            problems.push(violation(
                "unknown_event",
                Some(id),
                format!(
                    "links {} to {}, and the memory lacks one of them",
                    *link.from_event, *link.to_event
                ),
            ));
            continue;
        };
        let overlap = from.0 <= to.1 && to.0 <= from.1;
        if overlap {
            problems.push(violation(
                "linked_to_itself",
                Some(id),
                format!(
                    "{} and {} share sentences, so there is nothing to join",
                    *link.from_event, *link.to_event
                ),
            ));
        } else if link.kind != LinkKind::Contradiction && from.0 > to.0 {
            problems.push(violation(
                "backwards_link",
                Some(id),
                format!(
                    "a {} link runs forward, but {} comes after {}",
                    link.kind, *link.from_event, *link.to_event
                ),
            ));
        }
    }
}

#[cfg(test)]
#[path = "memory_tests.rs"]
mod tests;
