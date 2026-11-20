#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clipmill_contracts::schemas::mind_clip_plan::{JoinKind, MindClipPlanForm, PartRole};
use serde_json::json;

use super::*;
use crate::lattice::Lattice;
use crate::script::Sentence;

const S: u64 = SECOND;

/// Sixty sentences six seconds apart, five one-second words each.
fn recording() -> (Source, Vec<(u64, u64)>) {
    let sentences: Vec<Sentence> = (0..60)
        .map(|i| Sentence {
            index: i,
            start_ticks: i * 6 * S,
            end_ticks: i * 6 * S + 5 * S,
            speaker: Some(if i < 20 { "host" } else { "guest" }.to_owned()),
            text: match i {
                12 | 25 | 49 => format!("Line {i}: back when we were at Northwind."),
                // A retake: the speaker starts twice, then says it whole.
                13 => "We tried to get".to_owned(),
                14 => "we tried to get a loan from".to_owned(),
                15 => "We tried to get a loan from the bank, and they said no.".to_owned(),
                _ => format!("Line {i} of the conversation."),
            },
        })
        .collect();
    let words = sentences
        .iter()
        .flat_map(|s| (0..5).map(move |k| (s.start_ticks + k * S, s.start_ticks + (k + 1) * S)))
        .collect();
    (
        Source {
            duration_ticks: 360 * S,
            sentences,
            ..Source::default()
        },
        words,
    )
}

/// The markup: hooks, landings, a retake, a dependency.
fn scores() -> MindUnitScores {
    let mut units: Vec<serde_json::Value> = (0..60)
        .map(|i| json!({ "unit": format!("s{i}"), "hook": "none", "landing": "none", "trim": "keep" }))
        .collect();
    for (unit, field, value) in [
        (3, "hook", "strong"),
        (25, "hook", "strong"),
        (45, "hook", "good"),
        (18, "landing", "strong"),
        (30, "landing", "good"),
        (42, "landing", "strong"),
        (50, "landing", "good"),
        (13, "trim", "retake"),
        (14, "trim", "retake"),
    ] {
        units[unit][field] = json!(value);
    }
    units[11]["needs"] = json!("s10");
    serde_json::from_value(json!({
        "schema_version": "clipmill.mind.unit_scores.v1",
        "source_fingerprint": format!("sha256:{}", "a".repeat(64)),
        "script": { "level": "compact", "digest": format!("sha256:{}", "c".repeat(64)), "windows": 1 },
        "producer": { "stage": "mind-score", "implementation": "test", "model": "test", "prompt_version": "score.v1" },
        "units": units,
        "unscored": [],
    }))
    .unwrap()
}

fn event(id: &str, kind: &str, first: u64, last: u64, entities: &[&str]) -> serde_json::Value {
    let chapter = if first < 30 { "c1" } else { "c2" };
    json!({
        "chapter_id": chapter,
        "event_id": id, "kind": kind, "title": format!("Event {id}"), "mood": "neutral",
        "first_unit": format!("s{first}"), "last_unit": format!("s{last}"),
        "start_ticks": first * 6 * S, "end_ticks": last * 6 * S + 5 * S,
        "speakers": ["A"], "entities": entities,
    })
}

/// A question at s3 answered at s40, a story with a retake, a story with a
/// strong line in its middle, and one company named in three moments.
fn memory() -> MindMemory {
    serde_json::from_value(json!({
        "schema_version": "clipmill.mind.memory.v1",
        "source_fingerprint": format!("sha256:{}", "a".repeat(64)),
        "script": { "level": "compact", "digest": format!("sha256:{}", "c".repeat(64)), "windows": 1 },
        "producer": { "stage": "mind-memory", "implementation": "test", "model": "test", "prompt_versions": ["memory.read.v1"] },
        "chapters": [],
        "events": [
            event("m1", "question", 3, 3, &[]),
            event("m2", "story", 10, 18, &["e1"]),
            event("m3", "story", 22, 30, &["e1"]),
            event("m4", "answer", 40, 42, &[]),
            event("m5", "claim", 48, 50, &["e1"]),
        ],
        "entities": [{ "entity_id": "e1", "name": "Northwind", "kind": "organization", "aliases": [], "events": ["m2", "m3", "m5"] }],
        "links": [{ "link_id": "l1", "kind": "question_answer", "from_event": "m1", "to_event": "m4", "note": "The question is answered six minutes later." }],
        "dropped": [],
    }))
    .unwrap()
}

fn cut(with_memory: bool) -> Vec<Candidate> {
    let (source, words) = recording();
    let scores = scores();
    let memory = memory();
    candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        lattice: None,
        scores: Some(&scores),
        memory: with_memory.then_some(&memory),
        length: (15 * S, 60 * S),
    })
}

fn find(found: &[Candidate], form: MindClipPlanForm) -> &Candidate {
    found
        .iter()
        .find(|c| c.plan.form == form)
        .unwrap_or_else(|| panic!("no {form:?} among {:?}", forms(found)))
}

fn forms(found: &[Candidate]) -> Vec<MindClipPlanForm> {
    found.iter().map(|c| c.plan.form).collect()
}

#[test]
fn a_distant_answer_is_stitched_to_its_question() {
    let found = cut(true);
    let stitch = find(&found, MindClipPlanForm::Stitch);
    let plan = &stitch.plan;
    assert_eq!(
        plan.parts.iter().map(|p| p.role).collect::<Vec<_>>(),
        [PartRole::Question, PartRole::Answer]
    );
    assert_eq!(plan.parts[0].start_ticks, 18 * S);
    assert_eq!(plan.parts[1].start_ticks, 240 * S);
    assert_eq!(plan.joins[0].kind, JoinKind::Distant);
    assert!(
        plan.joins[0]
            .link
            .as_deref()
            .unwrap()
            .starts_with("l1 question_answer")
    );
    assert_eq!(stitch.scout, Scout::Link);
}

#[test]
fn a_long_linked_answer_uses_the_available_sentence_boundary() {
    let (source, words) = recording();
    let fingerprint = format!("sha256:{}", "a".repeat(64));
    let material = Material {
        source_fingerprint: &fingerprint,
        source: &source,
        words: &words,
        lattice: None,
        scores: None,
        memory: None,
        length: (15 * S, 60 * S),
    };
    let candidate = stitched(
        &material,
        &Table::new(&material),
        [
            (3, 3, "question", "Price?"),
            (30, 50, "answer", "The price"),
        ],
        "l1 question_answer: m1>m2",
        vec!["l1".to_owned()],
    )
    .expect("a 5-second question and a 53-second answer fit the 60-second budget");
    assert_eq!(candidate.plan.duration_ticks.get(), 58 * S);
    assert_eq!(
        candidate.plan.parts[1].units.first().unwrap().as_str(),
        "s30"
    );
    assert_eq!(
        candidate.plan.parts[1].units.last().unwrap().as_str(),
        "s38"
    );
}

#[test]
fn a_retake_inside_a_story_is_cut_out() {
    let found = cut(true);
    let tight = find(&found, MindClipPlanForm::Tightened);
    let [part] = tight.plan.parts.as_slice() else {
        panic!("one part");
    };
    let [removed] = part.removed.as_slice() else {
        panic!("one removal");
    };
    // From the end of s12 to the start of s15.
    assert_eq!(
        (removed.start_ticks, removed.end_ticks.get()),
        (77 * S, 90 * S)
    );
    assert!(!part.units.iter().any(|u| u.as_str() == "s13"));
}

#[test]
fn a_strong_line_inside_a_story_opens_it_cold_without_playing_twice() {
    let found = cut(true);
    // A cold open earns no bonus over the plain cut of the same moment, so it
    // may be offered as that cut's alternative rather than on its own.
    let plan = found
        .iter()
        .flat_map(|c| std::iter::once(&c.plan).chain(c.alternatives.iter()))
        .find(|plan| plan.form == MindClipPlanForm::ColdOpen)
        .unwrap_or_else(|| panic!("no cold open among {:?}", forms(&found)));
    assert_eq!(plan.parts[0].role, PartRole::Hook);
    assert_eq!(plan.parts[0].start_ticks, 150 * S);
    assert_eq!(plan.parts[1].start_ticks, 132 * S);
    assert_eq!(plan.joins[0].kind, JoinKind::Reorder);
    // The lifted line is taken out of the body, from the pause before it to
    // the pause after it.
    let lifted = plan.parts[1]
        .removed
        .iter()
        .find(|range| {
            range
                .note
                .as_deref()
                .is_some_and(|note| note.contains("hook"))
        })
        .expect("the body leaves the hook out");
    assert_eq!(lifted.start_ticks, 149 * S);
    assert_eq!(lifted.end_ticks.get(), 156 * S);
    let (source, words) = recording();
    let bounds = Bounds {
        words: &words,
        lattice: None,
        duration: None,
        source_ticks: source.duration_ticks,
    };
    assert!(crate::check(plan, &bounds).is_empty());
}

#[test]
fn three_moments_about_one_company_make_a_montage() {
    let found = cut(true);
    let montage = find(&found, MindClipPlanForm::ThemeMontage);
    assert_eq!(montage.plan.parts.len(), 3);
    assert!(montage.plan.parts.iter().all(|p| p.role == PartRole::Item));
    assert!(montage.plan.programme_ticks_fit((15 * S, 60 * S)));
}

#[test]
fn most_candidates_are_more_than_one_straight_span() {
    let found = cut(true);
    let shaped = found
        .iter()
        .filter(|c| c.plan.form != MindClipPlanForm::Straight)
        .count();
    assert!(shaped * 2 > found.len(), "{:?}", forms(&found));
}

#[test]
fn a_long_dependency_chain_cannot_replace_the_cited_event_with_its_distant_setup() {
    let (source, words) = recording();
    let mut table = Table::new(&Material {
        source_fingerprint: "",
        source: &source,
        words: &words,
        lattice: None,
        scores: None,
        memory: None,
        length: (15 * S, 25 * S),
    });
    for index in 1..=17 {
        table.units[index].needs = Some(index - 1);
    }
    let (start, end) = table.fit(17, 23, (15 * S, 25 * S)).unwrap();
    assert!((18..=23).contains(&start));
    assert!(end >= start && (15 * S..=25 * S).contains(&table.kept(start, end)));
    assert!(table.fit(17, 17, (15 * S, 25 * S)).is_none());
}

#[test]
fn a_short_event_can_borrow_nearby_setup_and_landing_without_losing_its_evidence() {
    let (source, words) = recording();
    let mut table = Table::new(&Material {
        source_fingerprint: "",
        source: &source,
        words: &words,
        lattice: None,
        scores: None,
        memory: None,
        length: (15 * S, 23 * S),
    });
    table.units[7].needs = Some(6);
    table.units[8].landing = 3;
    let (start, end) = table.fit(7, 7, (15 * S, 23 * S)).unwrap();
    assert_eq!((start, end), (6, 8));
    assert_eq!(table.kept(start, end), 17 * S);

    // An attractive but early landing between the linked events cannot
    // stand in for the later answer. Both endpoints must survive the cut.
    let question = (7, 7);
    let answer = (12, 12);
    assert!(
        table
            .fit_anchored(question, answer, (15 * S, 23 * S))
            .is_none()
    );
    let (start, end) = table
        .fit_anchored(question, answer, (15 * S, 45 * S))
        .unwrap();
    assert!(start <= 7 && end >= 12);
    table.units[12].cut = Some("tangent");
    assert!(
        table
            .fit_anchored(question, answer, (15 * S, 45 * S))
            .is_none()
    );
}

#[test]
fn no_candidate_opens_on_a_line_that_needs_an_earlier_one() {
    for candidate in cut(true).iter().chain(cut(false).iter()) {
        for part in &candidate.plan.parts {
            assert_ne!(
                part.units[0].as_str(),
                "s11",
                "{:?}",
                candidate.plan.plan_id
            );
        }
    }
}

#[test]
fn without_a_memory_moments_open_on_hooks() {
    let found = cut(false);
    assert!(!found.is_empty());
    assert!(
        found
            .iter()
            .all(|c| matches!(c.scout, Scout::Span | Scout::ColdOpen))
    );
    let starts: Vec<&str> = found
        .iter()
        .filter(|c| c.scout == Scout::Span)
        .map(|c| c.plan.parts[0].units[0].as_str())
        .collect();
    assert!(starts.contains(&"s3"), "{starts:?}");
}

#[test]
fn the_same_material_gives_the_same_plans() {
    let ids = |found: Vec<Candidate>| -> Vec<String> {
        found
            .into_iter()
            .map(|c| c.plan.plan_id.to_string())
            .collect()
    };
    assert_eq!(ids(cut(true)), ids(cut(true)));
}

trait Fits {
    fn programme_ticks_fit(&self, range: (u64, u64)) -> bool;
}

impl Fits for MindClipPlan {
    fn programme_ticks_fit(&self, (min, max): (u64, u64)) -> bool {
        (min..=max).contains(&crate::programme_ticks(self))
    }
}

#[test]
fn the_brief_scout_builds_a_montage_from_disjoint_requested_mentions() {
    let (mut source, words) = recording();
    for i in [3, 27, 50] {
        source.sentences[i].text = "The pricing changed in this quarter.".into();
    }
    let found = super::candidates_for_brief(
        &Material {
            source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
            source: &source,
            words: &words,
            lattice: None,
            scores: None,
            memory: None,
            length: (15 * S, 60 * S),
        },
        "Every time they talk about pricing",
    );
    let montage = find(&found, MindClipPlanForm::ThemeMontage);
    assert_eq!(montage.scout, Scout::Brief);
    assert_eq!(montage.plan.parts.len(), 3);
    assert!(montage.plan.joins.iter().all(|join| join.link.is_some()));
    assert_eq!(montage.alternatives.len(), 2);
    assert_eq!(
        montage.alternatives[0].parts[0].start_ticks,
        montage.plan.parts[1].start_ticks
    );
    assert_ne!(montage.alternatives[0].plan_id, montage.plan.plan_id);
}

#[test]
fn an_exhaustive_montage_reaches_a_sixth_mention_when_all_fit() {
    let (mut source, words) = recording();
    for line in [2, 12, 22, 32, 42, 52] {
        source.sentences[line].text = "Pricing changed that year.".into();
    }
    let matches = super::candidates_for_brief(
        &Material {
            source_fingerprint: &format!("sha256:{}", "c".repeat(64)),
            source: &source,
            words: &words,
            lattice: None,
            scores: None,
            memory: None,
            length: (15 * S, 60 * S),
        },
        "Every time they discuss pricing",
    );
    let montage = find(&matches, MindClipPlanForm::ThemeMontage);
    assert_eq!(montage.plan.parts.len(), 6);
    assert_eq!(montage.plan.evidence.len(), 6);
    assert_eq!(montage.plan.evidence.last().unwrap().as_str(), "s52");
}

#[test]
fn adjacent_mentions_retain_the_short_connective_between_them() {
    let (mut source, words) = recording();
    for line in [5, 30, 32] {
        source.sentences[line].text = "Pricing changed that year.".into();
    }
    source.sentences[31].text = "And excluded the basic plan.".into();
    let matches = super::candidates_for_brief(
        &Material {
            source_fingerprint: &format!("sha256:{}", "b".repeat(64)),
            source: &source,
            words: &words,
            lattice: None,
            scores: None,
            memory: None,
            length: (15 * S, 60 * S),
        },
        "Every time they discuss pricing",
    );
    let montage = find(&matches, MindClipPlanForm::ThemeMontage);
    assert_eq!(montage.plan.parts.len(), 2);
    assert_eq!(montage.plan.joins.len(), 1);
    assert!(
        montage.plan.parts[1]
            .units
            .iter()
            .any(|id| id.as_str() == "s31")
    );
    assert_eq!(montage.plan.parts[1].units.last().unwrap().as_str(), "s32");
}

#[test]
fn a_montage_keeps_qualified_mentions_even_when_the_global_markup_would_trim_them() {
    let (mut source, words) = recording();
    // s13 and s14 were marked as tangents to the source overall. Together
    // they contain one of the requested examples, including its qualifier.
    source.sentences[13].text = "They cut the pricing".into();
    source.sentences[14].text = "only for the first month.".into();
    source.sentences[15].text = "But only if you signed up before launch.".into();
    for line in [27, 50] {
        source.sentences[line].text = "The pricing changed after the launch.".into();
    }
    let scores = scores();
    let matches = super::candidates_for_brief(
        &Material {
            source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
            source: &source,
            words: &words,
            lattice: None,
            scores: Some(&scores),
            memory: None,
            length: (15 * S, 60 * S),
        },
        "Make a montage of every time the speaker talks about pricing.",
    );
    let montage = find(&matches, MindClipPlanForm::ThemeMontage);
    assert_eq!(montage.plan.parts.len(), 3);
    let quoted: Vec<_> = montage.plan.parts[0]
        .units
        .iter()
        .map(|u| u.as_str())
        .collect();
    assert!(quoted.windows(3).any(|ids| ids == ["s13", "s14", "s15"]));
    assert!(
        montage
            .plan
            .parts
            .iter()
            .all(|part| part.removed.is_empty())
    );
}

#[test]
fn the_reaction_scout_requires_a_nearby_spoken_line() {
    let (mut source, words) = recording();
    source.cues.push(crate::script::Cue {
        kind: "laughter".into(),
        start_ticks: 50 * S,
        end_ticks: 52 * S,
        label: None,
    });
    let found = candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        lattice: None,
        scores: None,
        memory: None,
        length: (15 * S, 60 * S),
    });
    assert!(found.iter().any(|row| row.scout == Scout::Reaction));
}

#[test]
fn edits_with_the_same_outer_edges_have_distinct_identities() {
    let (source, words) = recording();
    let edits = scores();
    let material = Material {
        source_fingerprint: "recording",
        source: &source,
        words: &words,
        lattice: None,
        scores: Some(&edits),
        memory: None,
        length: (15 * S, 60 * S),
    };
    let table = super::Table::new(&material);
    let untrimmed = table.plain();
    let body = super::Piece {
        lifted: None,
        a: 10,
        b: 16,
        role: "body",
        why: String::new(),
    };
    let hook = super::Piece {
        lifted: None,
        a: 10,
        b: 16,
        role: "hook",
        why: String::new(),
    };
    let id = |pieces: &[super::Piece], table: &super::Table| {
        super::plan_id("recording", "tightened", pieces, table)
    };
    assert_ne!(
        id(std::slice::from_ref(&body), &table),
        id(std::slice::from_ref(&body), &untrimmed)
    );
    assert_ne!(
        id(std::slice::from_ref(&body), &table),
        id(std::slice::from_ref(&hook), &table)
    );
}

/// A short talk once yielded no clip at all: the markup called nearly every
/// line a retake or an aside, and no moment had enough left. Markup is advice;
/// each event still yields its plain moment.
#[test]
fn over_marked_markup_still_leaves_every_moment() {
    let (source, words) = recording();
    let mut scores = serde_json::to_value(scores()).unwrap();
    for (index, row) in scores["units"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        if index % 7 != 0 {
            row["trim"] = json!(if index % 2 == 0 { "retake" } else { "aside" });
        }
    }
    let scores: MindUnitScores = serde_json::from_value(scores).unwrap();
    let memory = memory();
    let found = candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        lattice: None,
        scores: Some(&scores),
        memory: Some(&memory),
        length: (15 * S, 60 * S),
    });
    assert!(found.len() >= 3, "{:?}", forms(&found));
    for candidate in &found {
        let span: u64 = candidate
            .plan
            .parts
            .iter()
            .map(|p| p.end_ticks.get() - p.start_ticks)
            .sum();
        let kept = crate::programme_ticks(&candidate.plan);
        assert!(
            kept * 100 >= span * (100 - MAX_CUT_PERCENT),
            "{} hollowed out",
            candidate.plan.plan_id.as_str()
        );
    }
}

/// Cut points in the middle of every pause between sentences, except after
/// the sentences listed: there the speech runs on with no pause to cut in.
fn pauses_except(source: &Source, missing: &[u64]) -> Lattice {
    use crate::lattice::{CutPoint, Grade};
    Lattice::new(
        source
            .sentences
            .windows(2)
            .filter(|pair| !missing.contains(&pair[0].index))
            .map(|pair| CutPoint {
                at: pair[0].end_ticks + S / 2,
                pause_start: pair[0].end_ticks,
                pause_end: pair[1].start_ticks,
                grade: Grade::Clean,
            })
            .collect(),
    )
}

fn every_plan(found: &[Candidate]) -> impl Iterator<Item = &MindClipPlan> {
    found
        .iter()
        .flat_map(|c| std::iter::once(&c.plan).chain(c.alternatives.iter()))
}

fn edges(plan: &MindClipPlan) -> Vec<u64> {
    plan.parts
        .iter()
        .flat_map(|part| {
            [part.start_ticks, part.end_ticks.get()].into_iter().chain(
                part.removed
                    .iter()
                    .flat_map(|range| [range.start_ticks, range.end_ticks.get()]),
            )
        })
        .collect()
}

#[test]
fn with_measured_pauses_every_edge_and_removal_sits_in_one() {
    let (source, words) = recording();
    let (scores, memory) = (scores(), memory());
    // No pause after s18 (a story's strong landing) or after s29.
    let lattice = pauses_except(&source, &[18, 29]);
    let found = candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        lattice: Some(&lattice),
        scores: Some(&scores),
        memory: Some(&memory),
        length: (15 * S, 60 * S),
    });
    assert!(!found.is_empty());
    for plan in every_plan(&found) {
        for at in edges(plan) {
            assert!(lattice.at(at).is_some(), "{at} is not in a pause");
        }
        // A cut between s18 and s19, or s29 and s30, would land in speech.
        for at in edges(plan) {
            assert!(!(113 * S..=114 * S).contains(&at) && !(179 * S..=180 * S).contains(&at));
        }
        let bounds = Bounds {
            words: &words,
            lattice: Some(&lattice),
            duration: None,
            source_ticks: source.duration_ticks,
        };
        assert!(crate::check(plan, &bounds).is_empty());
    }
}

#[test]
fn a_clip_does_not_end_where_the_same_speaker_carries_on_nor_open_on_a_connective() {
    let (mut source, words) = recording();
    // The host goes on after s18's landing, and s22 opens on "And".
    source.sentences[19].text = "But that was only the start.".to_owned();
    source.sentences[22].text = "And then it all fell apart.".to_owned();
    let (scores, memory) = (scores(), memory());
    let found = candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        lattice: None,
        scores: Some(&scores),
        memory: Some(&memory),
        length: (15 * S, 60 * S),
    });
    assert!(!found.is_empty());
    for plan in every_plan(&found) {
        for part in &plan.parts {
            assert_ne!(
                part.end_ticks.get(),
                113 * S,
                "ends before the host's 'But…'"
            );
            assert_ne!(part.start_ticks, 132 * S, "opens on 'And…'");
        }
    }
}

#[test]
fn a_removal_that_saves_too_little_is_not_made_and_removals_are_spaced() {
    let (mut source, words) = recording();
    // Plain lines where the shared recording has its retake.
    for line in 13..=15 {
        source.sentences[line].text = format!("Line {line} of the conversation.");
    }
    let mut table = Table::new(&Material {
        source_fingerprint: "",
        source: &source,
        words: &words,
        lattice: None,
        scores: None,
        memory: None,
        length: (15 * S, 60 * S),
    });
    // Two seconds of filler: not worth the jump.
    table.units[14].text = "Um, you know, like.".to_owned();
    table.units[14].cut = Some("filler");
    table.units[13].close = Some(88 * S);
    table.units[15].open = Some(90 * S);
    assert!(table.removals(10, 18).is_empty());
    // Three seconds is.
    table.units[15].open = Some(91 * S);
    assert_eq!(table.removals(10, 18), [(88 * S, 91 * S, "filler")]);

    // Four asides in a minute: two per thirty seconds are kept, the longest.
    let mut table = Table::new(&Material {
        source_fingerprint: "",
        source: &source,
        words: &words,
        lattice: None,
        scores: None,
        memory: None,
        length: (15 * S, 60 * S),
    });
    for unit in [21, 23, 25, 27] {
        table.units[unit].text = "Uh, yeah, right.".to_owned();
        table.units[unit].cut = Some("filler");
    }
    table.units[24].close = Some(149 * S - S / 2);
    let kept = table.removals(20, 29);
    assert_eq!(kept.len(), 3);
    assert!(kept.windows(2).all(|pair| pair[0].0 < pair[1].0));
    assert!(kept.iter().any(|&(start, _, _)| start == 149 * S - S / 2));
}

fn plain_table(source: &Source, words: &[(u64, u64)]) -> Table {
    Table::new(&Material {
        source_fingerprint: "",
        source,
        words,
        lattice: None,
        scores: None,
        memory: None,
        length: (15 * S, 60 * S),
    })
}

/// A retake at s14 that s15 says again, in an otherwise plain recording.
fn retake() -> (Source, Vec<(u64, u64)>) {
    let (mut source, words) = recording();
    source.sentences[14].text = "the net worth".to_owned();
    source.sentences[15].text = "The net worth of people is up in every generation.".to_owned();
    (source, words)
}

#[test]
fn a_retake_the_next_line_says_again_is_cut_and_a_tangent_on_a_mark_alone_is_not() {
    let (source, words) = retake();
    let mut table = plain_table(&source, &words);
    table.units[14].cut = Some("false_start");
    assert_eq!(table.removals(10, 18), [(83 * S, 90 * S, "false_start")]);
    // Marked as a tangent, the same line stays: that is a judgement about
    // meaning, and code cannot check it.
    table.units[14].cut = Some("tangent");
    assert!(table.removals(10, 18).is_empty());
    // Two finished sentences that open alike are the speaker's parallelism,
    // not a retake, even when the markup calls the first one a repetition.
    let (mut source, words) = recording();
    source.sentences[14].text = "It's on you to make the decision to take on debt.".to_owned();
    source.sentences[15].text = "It's on you to choose what you study.".to_owned();
    let mut table = plain_table(&source, &words);
    table.units[14].cut = Some("repetition");
    assert!(table.removals(10, 18).is_empty());
    // A false start the next line does not say again stays too.
    let (source, words) = recording();
    let mut table = plain_table(&source, &words);
    table.units[20].cut = Some("false_start");
    assert!(table.removals(18, 24).is_empty());
}

#[test]
fn a_removal_is_not_made_when_what_stays_would_point_at_it() {
    // The promise before the retake would be kept and its delivery cut.
    let (mut source, words) = retake();
    source.sentences[13].text = "Let me put it to you by the stats.".to_owned();
    let mut table = plain_table(&source, &words);
    table.units[14].cut = Some("false_start");
    assert!(table.removals(10, 18).is_empty());

    // The line after carries on what was cut rather than saying it again.
    let (mut source, words) = recording();
    source.sentences[14].text = "They're worried about your ability".to_owned();
    source.sentences[15].text = "or their ability to see your ability in an interview.".to_owned();
    let mut table = plain_table(&source, &words);
    table.units[14].cut = Some("repetition");
    assert!(table.removals(10, 18).is_empty());

    // A question goes and leaves its answer.
    let (mut source, words) = recording();
    source.sentences[14].text = "Why did you leave?".to_owned();
    source.sentences[15].text = "Why did you leave the company, he asked.".to_owned();
    let mut table = plain_table(&source, &words);
    table.units[14].cut = Some("repetition");
    assert!(table.removals(10, 18).is_empty());
}

#[test]
fn a_moment_that_opens_on_an_answer_is_offered_with_its_question_first() {
    let (mut source, words) = recording();
    // The host asks just before the guest's story at s22.
    source.sentences[21].speaker = Some("host".to_owned());
    source.sentences[21].text = "What happened when the deal fell through?".to_owned();
    let (scores, memory) = (scores(), memory());
    let found = candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        lattice: None,
        scores: Some(&scores),
        memory: Some(&memory),
        length: (15 * S, 60 * S),
    });
    let framed = every_plan(&found)
        .find(|plan| {
            plan.parts.len() == 1
                && plan.parts[0].start_ticks == 126 * S
                && plan.parts[0].units.contains(&"s22".parse().unwrap())
        })
        .unwrap_or_else(|| panic!("no plan opens on the question: {:?}", forms(&found)));
    assert!(
        framed.parts[0]
            .why
            .as_deref()
            .is_some_and(|why| why.contains("question"))
    );
}

#[test]
fn a_clip_ends_where_a_sentence_closes_and_promises_nothing_more() {
    let (mut source, words) = recording();
    // s18 trails off unfinished, s30 promises what comes next, and s21 is
    // the end of a sentence s20 began.
    source.sentences[18].text = "And the reason we left was".to_owned();
    source.sentences[30].text = "I'm gonna give it back to you by the numbers.".to_owned();
    source.sentences[20].text = "Right".to_owned();
    source.sentences[21].text = "now nobody knows what happens next.".to_owned();
    // A stray full stop: s33 finishes the sentence s32 began.
    source.sentences[32].text = "Moving to a new city for work.".to_owned();
    source.sentences[33].text = "turned out to be harder than we thought.".to_owned();
    let (scores, memory) = (scores(), memory());
    let found = candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        lattice: None,
        scores: Some(&scores),
        memory: Some(&memory),
        length: (15 * S, 60 * S),
    });
    assert!(!found.is_empty());
    for plan in every_plan(&found) {
        for part in &plan.parts {
            let end = part.end_ticks.get();
            assert_ne!(end, 113 * S, "ends on an unfinished line");
            assert_ne!(end, 185 * S, "ends on a promise");
            assert_ne!(part.start_ticks, 126 * S, "opens on the end of a sentence");
            assert_ne!(
                end,
                197 * S,
                "ends where a lower-case line finishes the sentence"
            );
            assert_ne!(part.start_ticks, 198 * S, "opens on a lower-case line");
        }
    }
}

#[test]
fn a_transcript_without_punctuation_still_gives_clips() {
    let (mut source, words) = recording();
    for sentence in &mut source.sentences {
        sentence.text = sentence.text.trim_end_matches('.').to_owned();
    }
    let (scores, memory) = (scores(), memory());
    let found = candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        lattice: None,
        scores: Some(&scores),
        memory: Some(&memory),
        length: (15 * S, 60 * S),
    });
    assert!(!found.is_empty());
}

#[test]
fn a_lower_case_line_finishes_the_sentence_before_it() {
    let (mut source, words) = recording();
    source.sentences[32].text = "Moving to a new city for work.".to_owned();
    source.sentences[33].text = "turned out to be harder than we thought.".to_owned();
    let table = plain_table(&source, &words);
    assert!(!table.may_end(32), "the next line finishes this sentence");
    assert!(
        !table.may_open(33),
        "a lower-case line is the rest of a sentence"
    );
    assert!(table.may_end(31) && table.may_open(32));
    assert!(table.may_end(33));
}

#[test]
fn a_clip_does_not_end_before_the_same_speaker_points_back_at_its_last_line() {
    let (mut source, words) = recording();
    // A recogniser split "It was never the plan that the shop…" in two.
    source.sentences[32].text = "It was never the plan.".to_owned();
    source.sentences[33].text = "That the shop would close so soon.".to_owned();
    let table = plain_table(&source, &words);
    assert!(!table.may_end(32));
    // From another voice the same line starts their turn instead.
    source.sentences[33].speaker = Some("host".to_owned());
    let table = plain_table(&source, &words);
    assert!(table.may_end(32));
}

#[test]
fn the_cutter_finds_filler_and_restarts_without_a_model() {
    let (mut source, words) = recording();
    source.sentences[21].text = "Um, you know, like.".to_owned();
    source.sentences[23].text = "We went to the".to_owned();
    source.sentences[24].text = "We went to the bank and asked for a loan.".to_owned();
    // Two finished sentences that open alike stay.
    source.sentences[26].text = "It's on you to pay.".to_owned();
    source.sentences[27].text = "It's on you to choose.".to_owned();
    let table = plain_table(&source, &words);
    assert_eq!(table.units[21].cut, Some("filler"));
    assert_eq!(table.units[23].cut, Some("false_start"));
    assert_eq!(table.units[24].cut, None);
    assert_eq!(table.units[26].cut, None);
    assert_eq!(table.units[22].cut, None);
}
