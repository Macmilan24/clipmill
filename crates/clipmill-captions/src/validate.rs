//! What a cue set has to satisfy before anyone is asked to read it.
//!
//! The segmenter minimizes a cost, and a minimum is not a guarantee: it returns
//! the best grouping available, which on dense speech in a tight window can
//! still be one a viewer cannot keep up with. Something separate has to say so.
//!
//! This is deliberately not the segmenter's own opinion of its work. It reads
//! the finished cues and re-derives every number from them, so a bug in the
//! cost function shows up as a violation rather than as a cost that was low for
//! the wrong reason. The gate runs it over the goldens and refuses any
//! reading-rate violation in the accessibility intent — that intent is what the
//! sidecars are written from, and a sidecar is what a deaf viewer is left with
//! when the burn-in is not enough.
//!
//! It takes plain facts rather than either the segmenter's types or the
//! document's, because the caller that matters most is the one checking an
//! exported file it did not produce.

use crate::profile::{Profile, TICKS_PER_SECOND};

/// One cue, reduced to what can be checked about it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CueFacts<'a> {
    pub cue_id: &'a str,
    pub start_ticks: i64,
    pub end_ticks: i64,
    /// The rendered width of every line, in order.
    pub lines: &'a [usize],
}

impl CueFacts<'_> {
    /// Characters a reader has to get through, counting the space a line break
    /// stands in for.
    fn characters(&self) -> usize {
        self.lines.iter().sum::<usize>() + self.lines.len().saturating_sub(1)
    }

    fn duration_ticks(&self) -> i64 {
        self.end_ticks - self.start_ticks
    }

    /// Characters per second, measured from the window rather than trusted.
    pub fn reading_rate_cps(&self) -> f64 {
        width(self.characters()) / seconds(self.duration_ticks().max(1))
    }
}

/// Something a viewer would be right to complain about.
#[derive(Clone, Debug, PartialEq)]
pub enum Violation {
    ReadingRate {
        cue_id: String,
        measured_cps: f64,
        ceiling_cps: f64,
    },
    LineTooWide {
        cue_id: String,
        characters: usize,
        ceiling: usize,
    },
    TooManyLines {
        cue_id: String,
        lines: usize,
        ceiling: usize,
    },
    TooBrief {
        cue_id: String,
        ticks: i64,
        floor_ticks: i64,
    },
    HeldTooLong {
        cue_id: String,
        ticks: i64,
        ceiling_ticks: i64,
    },
    Crowds {
        cue_id: String,
        previous_cue_id: String,
        gap_ticks: i64,
        floor_ticks: i64,
    },
    SpansCut {
        cue_id: String,
        cut_ticks: i64,
    },
    OutOfOrder {
        cue_id: String,
    },
}

impl Violation {
    /// The cue this is about.
    pub fn cue_id(&self) -> &str {
        match self {
            Self::ReadingRate { cue_id, .. }
            | Self::LineTooWide { cue_id, .. }
            | Self::TooManyLines { cue_id, .. }
            | Self::TooBrief { cue_id, .. }
            | Self::HeldTooLong { cue_id, .. }
            | Self::Crowds { cue_id, .. }
            | Self::SpansCut { cue_id, .. }
            | Self::OutOfOrder { cue_id } => cue_id,
        }
    }

    /// A sentence a person can act on. Captions fail in ways users notice and
    /// cannot name, so the words matter as much as the check.
    pub fn message(&self) -> String {
        match self {
            Self::ReadingRate {
                measured_cps,
                ceiling_cps,
                ..
            } => format!(
                "asks for {measured_cps:.1} characters a second, and the profile allows \
                 {ceiling_cps:.1}"
            ),
            Self::LineTooWide {
                characters,
                ceiling,
                ..
            } => format!("has a line of {characters} characters, and {ceiling} is the ceiling"),
            Self::TooManyLines { lines, ceiling, .. } => {
                format!("is {lines} lines, and the profile allows {ceiling}")
            }
            Self::TooBrief { ticks, .. } => {
                format!(
                    "is on screen for {:.2}s, which is too brief to read",
                    seconds(*ticks)
                )
            }
            Self::HeldTooLong { ticks, .. } => format!(
                "is held for {:.2}s after its words ended, which reads as a caption that stuck",
                seconds(*ticks)
            ),
            Self::Crowds {
                previous_cue_id, ..
            } => format!("follows {previous_cue_id} with no blank between them"),
            Self::SpansCut { cut_ticks, .. } => format!(
                "is still on screen across the cut at {:.2}s",
                seconds(*cut_ticks)
            ),
            Self::OutOfOrder { .. } => "starts before the cue in front of it".to_owned(),
        }
    }
}

/// Ticks as seconds. Durations here are seconds rather than centuries, far
/// inside where a double stops being exact about integers.
#[allow(
    clippy::cast_precision_loss,
    reason = "a cue duration, bounded by the profile's maximum"
)]
fn seconds(ticks: i64) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

/// A character count as a number to divide by.
#[allow(
    clippy::cast_precision_loss,
    reason = "characters on at most two lines"
)]
fn width(characters: usize) -> f64 {
    characters as f64
}

/// Every way this cue set falls short of the profile, in cue order.
///
/// `shot_cuts` may be empty, which means nothing is known about where the picture
/// changes — not that it never does. An empty result is the only acceptable one
/// for the accessibility intent.
pub fn validate(cues: &[CueFacts<'_>], profile: Profile, shot_cuts: &[i64]) -> Vec<Violation> {
    let mut found = Vec::new();
    let mut previous: Option<&CueFacts<'_>> = None;

    for facts in cues {
        let cue_id = facts.cue_id.to_owned();
        if facts.end_ticks <= facts.start_ticks {
            found.push(Violation::OutOfOrder { cue_id });
            previous = Some(facts);
            continue;
        }

        let rate = facts.reading_rate_cps();
        if rate > profile.reading_rate_cps {
            found.push(Violation::ReadingRate {
                cue_id: cue_id.clone(),
                measured_cps: rate,
                ceiling_cps: profile.reading_rate_cps,
            });
        }
        if facts.lines.len() > profile.max_lines {
            found.push(Violation::TooManyLines {
                cue_id: cue_id.clone(),
                lines: facts.lines.len(),
                ceiling: profile.max_lines,
            });
        }
        for width in facts.lines {
            if *width > profile.max_line_characters {
                found.push(Violation::LineTooWide {
                    cue_id: cue_id.clone(),
                    characters: *width,
                    ceiling: profile.max_line_characters,
                });
            }
        }

        let duration = facts.duration_ticks();
        if duration < profile.min_duration_ticks {
            found.push(Violation::TooBrief {
                cue_id: cue_id.clone(),
                ticks: duration,
                floor_ticks: profile.min_duration_ticks,
            });
        }
        if duration > profile.max_duration_ticks {
            found.push(Violation::HeldTooLong {
                cue_id: cue_id.clone(),
                ticks: duration,
                ceiling_ticks: profile.max_duration_ticks,
            });
        }

        if let Some(cut) = shot_cuts
            .iter()
            .find(|cut| **cut > facts.start_ticks && **cut < facts.end_ticks)
        {
            found.push(Violation::SpansCut {
                cue_id: cue_id.clone(),
                cut_ticks: *cut,
            });
        }

        if let Some(before) = previous {
            if facts.start_ticks < before.end_ticks {
                found.push(Violation::OutOfOrder {
                    cue_id: cue_id.clone(),
                });
            } else {
                let gap = facts.start_ticks - before.end_ticks;
                if gap < profile.min_gap_ticks {
                    found.push(Violation::Crowds {
                        cue_id,
                        previous_cue_id: before.cue_id.to_owned(),
                        gap_ticks: gap,
                        floor_ticks: profile.min_gap_ticks,
                    });
                }
            }
        }
        previous = Some(facts);
    }
    found
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::panic
    )]

    use super::{CueFacts, Violation, validate};
    use crate::profile::{Profile, TICKS_PER_SECOND};

    fn facts<'a>(cue_id: &'a str, from: f64, to: f64, lines: &'a [usize]) -> CueFacts<'a> {
        #[allow(clippy::cast_possible_truncation)]
        CueFacts {
            cue_id,
            start_ticks: (from * TICKS_PER_SECOND as f64) as i64,
            end_ticks: (to * TICKS_PER_SECOND as f64) as i64,
            lines,
        }
    }

    #[test]
    fn a_comfortable_cue_set_reports_nothing() {
        let one = facts("cue_1", 0.0, 3.0, &[30, 24]);
        let two = facts("cue_2", 3.5, 6.0, &[28]);
        assert!(validate(&[one, two], Profile::ACCESSIBILITY_EN, &[]).is_empty());
    }

    #[test]
    fn a_cue_nobody_can_read_in_time_is_named_with_both_numbers() {
        // 42 characters in one second is more than twice the ceiling.
        let hot = facts("cue_1", 0.0, 1.0, &[42]);
        let found = validate(&[hot], Profile::ACCESSIBILITY_EN, &[]);
        let Some(Violation::ReadingRate {
            measured_cps,
            ceiling_cps,
            ..
        }) = found.first()
        else {
            panic!("expected a reading-rate violation, got {found:?}");
        };
        assert!((measured_cps - 42.0).abs() < 1e-9);
        assert!((ceiling_cps - 20.0).abs() < f64::EPSILON);
        assert!(found[0].message().contains("42.0"));
    }

    #[test]
    fn the_line_ceiling_is_checked_per_line_not_on_the_total() {
        let wide = facts("cue_1", 0.0, 6.0, &[43, 10]);
        let found = validate(&[wide], Profile::ACCESSIBILITY_EN, &[]);
        assert!(matches!(found.as_slice(), [Violation::LineTooWide { .. }]));
    }

    #[test]
    fn a_third_line_is_a_violation_rather_than_a_rounding() {
        let tall = facts("cue_1", 0.0, 6.0, &[10, 10, 10]);
        let found = validate(&[tall], Profile::ACCESSIBILITY_EN, &[]);
        assert!(found.iter().any(|item| matches!(
            item,
            Violation::TooManyLines {
                lines: 3,
                ceiling: 2,
                ..
            }
        )));
    }

    #[test]
    fn a_cue_that_flashes_and_a_cue_that_sticks_are_both_reported() {
        let flash = facts("cue_1", 0.0, 0.4, &[4]);
        let stuck = facts("cue_2", 2.0, 12.0, &[8]);
        let found = validate(&[flash, stuck], Profile::ACCESSIBILITY_EN, &[]);
        assert!(
            found
                .iter()
                .any(|item| matches!(item, Violation::TooBrief { .. }))
        );
        assert!(
            found
                .iter()
                .any(|item| matches!(item, Violation::HeldTooLong { .. }))
        );
    }

    #[test]
    fn two_cues_with_no_blank_between_them_read_as_one() {
        let one = facts("cue_1", 0.0, 3.0, &[20]);
        let two = facts("cue_2", 3.0, 6.0, &[20]);
        let found = validate(&[one, two], Profile::ACCESSIBILITY_EN, &[]);
        let Some(Violation::Crowds {
            previous_cue_id, ..
        }) = found.first()
        else {
            panic!("expected a crowding violation, got {found:?}");
        };
        assert_eq!(previous_cue_id, "cue_1");
    }

    #[test]
    fn a_cue_still_on_screen_at_a_cut_is_named_with_the_cut() {
        let over = facts("cue_1", 0.0, 4.0, &[30]);
        let cut = 2 * TICKS_PER_SECOND;
        let found = validate(&[over], Profile::ACCESSIBILITY_EN, &[cut]);
        assert!(matches!(
            found.as_slice(),
            [Violation::SpansCut { cut_ticks, .. }] if *cut_ticks == cut
        ));
    }

    #[test]
    fn cues_that_run_backwards_are_reported_and_not_measured() {
        let broken = facts("cue_1", 4.0, 4.0, &[10]);
        let found = validate(&[broken], Profile::ACCESSIBILITY_EN, &[]);
        assert!(matches!(found.as_slice(), [Violation::OutOfOrder { .. }]));
    }

    #[test]
    fn the_burn_in_profile_accepts_what_the_sidecar_profile_will_not() {
        let hot = facts("cue_1", 0.0, 0.6, &[18]);
        assert!(!validate(&[hot], Profile::ACCESSIBILITY_EN, &[]).is_empty());
        assert!(validate(&[hot], Profile::BURN_IN_EN, &[]).is_empty());
    }

    #[test]
    fn every_violation_names_its_cue_and_says_something_a_person_can_act_on() {
        let bad = facts("cue_7", 0.0, 0.5, &[60, 60, 60]);
        for item in validate(&[bad], Profile::ACCESSIBILITY_EN, &[]) {
            assert_eq!(item.cue_id(), "cue_7");
            assert!(item.message().len() > 20, "{item:?} says too little");
        }
    }
}
