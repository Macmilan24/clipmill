#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::num::NonZeroU64;

use clipmill_contracts::schemas::mind_clip_plan::{JoinKind, MindClipPlan, MindClipPlanForm};

use super::*;

const S: u64 = SECOND;

fn fixture(name: &str) -> MindClipPlan {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/mind.clip_plan/valid")
        .join(format!("{name}.json"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Words on whole seconds, each lasting 0.6 s: a boundary on a whole second
/// is always legal, one at x.3 s always cuts a word.
fn words() -> Vec<(u64, u64)> {
    (0..2000)
        .map(|second| (second * S, second * S + 3 * S / 5))
        .collect()
}

fn bounds(words: &[(u64, u64)]) -> Bounds<'_> {
    Bounds {
        words,
        lattice: None,
        duration: Some((15 * S, 90 * S)),
        source_ticks: 2000 * S,
    }
}

fn codes(plan: &MindClipPlan) -> Vec<&'static str> {
    let words = words();
    check(plan, &bounds(&words))
        .iter()
        .map(|v| v.code)
        .collect()
}

#[test]
fn the_fixture_stitch_and_straight_cut_are_plans_clipmill_makes() {
    assert_eq!(codes(&fixture("stitch")), Vec::<&str>::new());
    assert_eq!(codes(&fixture("straight")), Vec::<&str>::new());
    assert_eq!(programme_ticks(&fixture("stitch")), 34 * S);
}

#[test]
fn a_distant_join_needs_the_evidence_that_links_its_parts() {
    let mut plan = fixture("stitch");
    plan.joins[1].link = None;
    assert_eq!(codes(&plan), ["unjustified_join"]);
}

#[test]
fn a_join_marked_as_the_wrong_kind_is_named() {
    let mut plan = fixture("stitch");
    plan.joins[1].kind = JoinKind::Adjacent;
    assert_eq!(codes(&plan), ["join_kind", "form"]);
}

#[test]
fn a_boundary_inside_a_word_is_refused_with_where_to_move_it() {
    let mut plan = fixture("straight");
    plan.parts[0].start_ticks = 120 * S + 3 * S / 10;
    plan.duration_ticks = NonZeroU64::new(programme_ticks(&plan)).unwrap();
    let words = words();
    let found = check(&plan, &bounds(&words));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code, "cuts_word");
    assert!(
        found[0].message.contains("2:00.0–2:00.6"),
        "{}",
        found[0].message
    );
}

#[test]
fn a_removed_range_outside_its_part_or_at_its_edge_is_refused() {
    let mut plan = fixture("stitch");
    plan.parts[1].removed[0].start_ticks = 800 * S;
    plan.parts[1].removed[0].end_ticks = NonZeroU64::new(801 * S).unwrap();
    assert!(codes(&plan).contains(&"bad_removal"));

    let mut plan = fixture("stitch");
    plan.parts[1].removed[0].start_ticks = plan.parts[1].start_ticks;
    assert!(codes(&plan).contains(&"removal_at_edge"));
}

#[test]
fn parts_may_not_play_the_same_moment_twice() {
    let mut plan = fixture("stitch");
    plan.parts[2].start_ticks = 790 * S;
    plan.parts[2].end_ticks = NonZeroU64::new(799 * S).unwrap();
    assert!(codes(&plan).contains(&"parts_overlap"));
}

/// The stitch fixture as a cold open: the punchline, lifted from inside the
/// answer, plays first; `leave_out` takes it out of the answer.
fn cold_open(leave_out: bool) -> MindClipPlan {
    let mut plan = fixture("stitch");
    plan.form = MindClipPlanForm::ColdOpen;
    let mut hook = plan.parts[1].clone();
    hook.part_id = "p0".parse().unwrap();
    hook.role = PartRole::Hook;
    hook.start_ticks = 770 * S;
    hook.end_ticks = NonZeroU64::new(773 * S).unwrap();
    hook.removed.clear();
    if leave_out {
        let mut lifted = plan.parts[1].removed[0].clone();
        lifted.start_ticks = 770 * S;
        lifted.end_ticks = NonZeroU64::new(773 * S).unwrap();
        plan.parts[1].removed.push(lifted);
        plan.parts[1].removed.sort_by_key(|range| range.start_ticks);
    }
    plan.parts.insert(0, hook);
    let mut back = plan.joins[0].clone();
    back.after = "p0".parse().unwrap();
    back.kind = JoinKind::Reorder;
    back.link = None;
    plan.joins.insert(0, back);
    plan.duration_ticks = NonZeroU64::new(programme_ticks(&plan)).unwrap();
    plan
}

#[test]
fn a_cold_open_that_leaves_its_hook_out_of_the_body_needs_no_link_to_go_back() {
    assert_eq!(codes(&cold_open(true)), Vec::<&str>::new());
}

#[test]
fn a_cold_open_that_plays_its_hook_again_is_refused() {
    assert_eq!(codes(&cold_open(false)), ["parts_overlap"]);
}

#[test]
fn each_form_keeps_its_own_shape() {
    let mut plan = fixture("straight");
    plan.form = MindClipPlanForm::ThemeMontage;
    assert_eq!(codes(&plan), ["form"]);
    plan.form = MindClipPlanForm::Tightened;
    assert_eq!(codes(&plan), ["form"]);
    let mut plan = fixture("stitch");
    plan.form = MindClipPlanForm::Straight;
    assert_eq!(codes(&plan), ["form"]);
}

#[test]
fn the_duration_must_match_the_parts_and_the_brief() {
    let mut plan = fixture("straight");
    plan.duration_ticks = NonZeroU64::new(40 * S).unwrap();
    assert_eq!(codes(&plan), ["duration_mismatch"]);
    let words = words();
    let short = Bounds {
        duration: Some((60 * S, 90 * S)),
        ..bounds(&words)
    };
    let found: Vec<_> = check(&fixture("straight"), &short)
        .iter()
        .map(|v| v.code)
        .collect();
    assert_eq!(found, ["duration_range"]);
}

#[test]
fn a_tiny_part_reads_as_a_glitch_unless_it_is_a_reaction() {
    let mut plan = fixture("stitch");
    plan.parts[2].end_ticks = NonZeroU64::new(1759 * S).unwrap();
    plan.duration_ticks = NonZeroU64::new(programme_ticks(&plan)).unwrap();
    let words = words();
    let unbounded = Bounds {
        duration: None,
        ..bounds(&words)
    };
    let found: Vec<_> = check(&plan, &unbounded).iter().map(|v| v.code).collect();
    assert_eq!(found, ["part_too_short"]);
    plan.parts[2].role = PartRole::Reaction;
    assert!(check(&plan, &unbounded).is_empty());
}

#[test]
fn joins_must_follow_their_parts_one_for_one() {
    let mut plan = fixture("stitch");
    plan.joins.pop();
    assert!(codes(&plan).contains(&"join_count"));
    let mut plan = fixture("stitch");
    plan.joins.swap(0, 1);
    assert!(codes(&plan).contains(&"join_order"));
}

#[test]
fn an_empty_plan_says_so_and_nothing_else() {
    let mut plan = fixture("straight");
    plan.parts.clear();
    assert_eq!(codes(&plan), ["no_parts"]);
}

#[test]
fn silent_footage_allows_any_boundary() {
    let mut plan = fixture("straight");
    plan.parts[0].start_ticks = 120 * S + 3 * S / 10;
    plan.duration_ticks = NonZeroU64::new(programme_ticks(&plan)).unwrap();
    let silent = Bounds {
        words: &[],
        lattice: None,
        duration: None,
        source_ticks: 2000 * S,
    };
    assert!(check(&plan, &silent).is_empty());
}
