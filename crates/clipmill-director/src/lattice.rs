//! Snapping a boundary a person dragged onto one the search says is legal.
//!
//! Discovery published, per candidate, the starts and ends a clip may actually
//! use — points where a sentence begins, a speaker stops, a shot changes. The
//! ranking then scored every legal pair and chose one. What the Inspector gives
//! a user is a handle to move, and what this module decides is where letting go
//! puts it.
//!
//! Snapping rather than accepting the raw drag is the whole point. A boundary
//! placed a few frames off a sentence edge is the mid-word cut the boundary
//! optimizer exists to avoid, and no amount of care with a mouse gets a person
//! within a frame of the right instant. The lattice is what "legal" means, so
//! the lattice is what a drag resolves to.
//!
//! Everything here is integer arithmetic over sorted lists, so the same drag
//! lands on the same edge on every machine — which matters because the document
//! it produces is compared against a golden.

use thiserror::Error;

/// Where a clip may begin and end, as discovery published it.
#[derive(Clone, Copy, Debug)]
pub struct Lattice<'a> {
    pub starts: &'a [i64],
    pub ends: &'a [i64],
}

/// A start and an end, both on the lattice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Boundary {
    pub start_ticks: i64,
    pub end_ticks: i64,
}

impl Boundary {
    pub fn duration_ticks(self) -> i64 {
        self.end_ticks - self.start_ticks
    }
}

/// How long a clip is allowed to be, echoed from the discovery document so the
/// snap cannot produce a clip the search would never have proposed.
#[derive(Clone, Copy, Debug)]
pub struct Duration {
    pub min_ticks: i64,
    pub max_ticks: i64,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SnapError {
    #[error("this candidate's lattice offers no legal start")]
    NoStarts,
    #[error("this candidate's lattice offers no legal end")]
    NoEnds,
    #[error("no legal pair in this lattice lands inside the duration the search was given")]
    NothingInRange,
}

/// The nearest lattice point to `at`, preferring the earlier one on a tie.
///
/// A tie means the drag landed exactly between two legal edges, which is rare
/// and has to resolve somehow; resolving it earlier rather than later is
/// arbitrary but fixed, and fixed is what a golden needs.
pub fn nearest(points: &[i64], at: i64) -> Option<i64> {
    points
        .iter()
        .copied()
        .min_by_key(|point| ((point - at).abs(), *point))
}

/// Put a dragged boundary on the lattice, inside the duration the search used.
///
/// The edge the user moved is honoured first and the other is adjusted only if
/// it has to be. Someone dragging the start is answering "where should this
/// begin"; silently moving the start because the end no longer fits would
/// answer a question they did not ask.
pub fn snap(
    lattice: Lattice<'_>,
    wanted: Boundary,
    duration: Duration,
    moved: Edge,
) -> Result<Boundary, SnapError> {
    if lattice.starts.is_empty() {
        return Err(SnapError::NoStarts);
    }
    if lattice.ends.is_empty() {
        return Err(SnapError::NoEnds);
    }

    let (anchor, free) = match moved {
        Edge::Start => (
            nearest(lattice.starts, wanted.start_ticks).ok_or(SnapError::NoStarts)?,
            wanted.end_ticks,
        ),
        Edge::End => (
            nearest(lattice.ends, wanted.end_ticks).ok_or(SnapError::NoEnds)?,
            wanted.start_ticks,
        ),
    };

    // Every legal partner for the edge that was held, ordered by how far it is
    // from where the other edge already was — so a drag of one handle moves the
    // other as little as it can get away with.
    let partners = match moved {
        Edge::Start => lattice.ends,
        Edge::End => lattice.starts,
    };
    let mut ordered: Vec<i64> = partners
        .iter()
        .copied()
        .filter(|partner| {
            let candidate = match moved {
                Edge::Start => Boundary {
                    start_ticks: anchor,
                    end_ticks: *partner,
                },
                Edge::End => Boundary {
                    start_ticks: *partner,
                    end_ticks: anchor,
                },
            };
            let length = candidate.duration_ticks();
            length >= duration.min_ticks && length <= duration.max_ticks
        })
        .collect();
    ordered.sort_by_key(|partner| ((partner - free).abs(), *partner));

    let partner = ordered.first().copied().ok_or(SnapError::NothingInRange)?;
    Ok(match moved {
        Edge::Start => Boundary {
            start_ticks: anchor,
            end_ticks: partner,
        },
        Edge::End => Boundary {
            start_ticks: partner,
            end_ticks: anchor,
        },
    })
}

/// Which handle the user moved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Edge {
    Start,
    End,
}

/// Whether a boundary is one the lattice actually offers.
///
/// Checked rather than assumed for boundaries that did not come from `snap` —
/// a boundary arriving over IPC has been through a process this one does not
/// control.
pub fn is_legal(lattice: Lattice<'_>, boundary: Boundary, duration: Duration) -> bool {
    let length = boundary.duration_ticks();
    lattice.starts.contains(&boundary.start_ticks)
        && lattice.ends.contains(&boundary.end_ticks)
        && length >= duration.min_ticks
        && length <= duration.max_ticks
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{Boundary, Duration, Edge, Lattice, SnapError, is_legal, nearest, snap};

    const SECOND: i64 = 90_000;

    fn lattice() -> ([i64; 4], [i64; 4]) {
        (
            [0, 10 * SECOND, 22 * SECOND, 30 * SECOND],
            [18 * SECOND, 40 * SECOND, 55 * SECOND, 70 * SECOND],
        )
    }

    fn duration() -> Duration {
        Duration {
            min_ticks: 15 * SECOND,
            max_ticks: 60 * SECOND,
        }
    }

    #[test]
    fn a_drag_lands_on_the_nearest_legal_edge() {
        assert_eq!(nearest(&[0, 100, 250], 90), Some(100));
        assert_eq!(nearest(&[0, 100, 250], 40), Some(0));
        assert_eq!(nearest(&[], 40), None);
    }

    #[test]
    fn an_exact_tie_resolves_earlier_and_resolves_the_same_way_twice() {
        // Fifty is equidistant from both. Arbitrary, but fixed — a golden
        // needs the same answer on every machine.
        assert_eq!(nearest(&[0, 100], 50), Some(0));
        assert_eq!(nearest(&[0, 100], 50), nearest(&[0, 100], 50));
    }

    #[test]
    fn the_edge_the_user_moved_is_the_one_that_is_honoured() {
        let (starts, ends) = lattice();
        let moved = snap(
            Lattice {
                starts: &starts,
                ends: &ends,
            },
            Boundary {
                // Dragged to just past the 22s start, with the end where it was.
                start_ticks: 23 * SECOND,
                end_ticks: 40 * SECOND,
            },
            duration(),
            Edge::Start,
        )
        .expect("a legal pair");

        assert_eq!(moved.start_ticks, 22 * SECOND, "the dragged edge snapped");
        assert_eq!(moved.end_ticks, 40 * SECOND, "the other edge stayed put");
    }

    #[test]
    fn the_other_edge_moves_only_as_far_as_it_has_to() {
        let (starts, ends) = lattice();
        // Start dragged to 30s: 40s end would be a 10s clip, under the floor,
        // so the end has to move — and 55s is the nearest end that works.
        let moved = snap(
            Lattice {
                starts: &starts,
                ends: &ends,
            },
            Boundary {
                start_ticks: 29 * SECOND,
                end_ticks: 40 * SECOND,
            },
            duration(),
            Edge::Start,
        )
        .expect("a legal pair");

        assert_eq!(moved.start_ticks, 30 * SECOND);
        assert_eq!(moved.end_ticks, 55 * SECOND);
        assert!(moved.duration_ticks() >= duration().min_ticks);
    }

    #[test]
    fn dragging_the_end_holds_the_start() {
        let (starts, ends) = lattice();
        let moved = snap(
            Lattice {
                starts: &starts,
                ends: &ends,
            },
            Boundary {
                start_ticks: 10 * SECOND,
                end_ticks: 53 * SECOND,
            },
            duration(),
            Edge::End,
        )
        .expect("a legal pair");

        assert_eq!(moved.end_ticks, 55 * SECOND);
        assert_eq!(moved.start_ticks, 10 * SECOND);
    }

    #[test]
    fn a_snap_never_produces_a_clip_the_search_would_not_have_proposed() {
        let (starts, ends) = lattice();
        for at in (0..70).step_by(3) {
            for edge in [Edge::Start, Edge::End] {
                let wanted = Boundary {
                    start_ticks: at * SECOND,
                    end_ticks: (at + 20) * SECOND,
                };
                if let Ok(moved) = snap(
                    Lattice {
                        starts: &starts,
                        ends: &ends,
                    },
                    wanted,
                    duration(),
                    edge,
                ) {
                    assert!(starts.contains(&moved.start_ticks));
                    assert!(ends.contains(&moved.end_ticks));
                    assert!(moved.duration_ticks() >= duration().min_ticks);
                    assert!(moved.duration_ticks() <= duration().max_ticks);
                }
            }
        }
    }

    #[test]
    fn a_lattice_with_no_legal_pair_in_range_is_refused_rather_than_forced() {
        let starts = [0_i64];
        let ends = [SECOND];
        let error = snap(
            Lattice {
                starts: &starts,
                ends: &ends,
            },
            Boundary {
                start_ticks: 0,
                end_ticks: SECOND,
            },
            duration(),
            Edge::Start,
        )
        .expect_err("a one-second clip is under the floor");
        assert_eq!(error, SnapError::NothingInRange);
    }

    #[test]
    fn an_empty_lattice_says_which_half_is_missing() {
        let ends = [SECOND];
        assert_eq!(
            snap(
                Lattice {
                    starts: &[],
                    ends: &ends
                },
                Boundary {
                    start_ticks: 0,
                    end_ticks: SECOND
                },
                duration(),
                Edge::Start,
            ),
            Err(SnapError::NoStarts)
        );
        assert_eq!(
            snap(
                Lattice {
                    starts: &ends,
                    ends: &[]
                },
                Boundary {
                    start_ticks: 0,
                    end_ticks: SECOND
                },
                duration(),
                Edge::Start,
            ),
            Err(SnapError::NoEnds)
        );
    }

    #[test]
    fn a_boundary_that_did_not_come_from_here_is_checked_against_the_lattice() {
        let (starts, ends) = lattice();
        let at = Lattice {
            starts: &starts,
            ends: &ends,
        };
        assert!(is_legal(
            at,
            Boundary {
                start_ticks: 10 * SECOND,
                end_ticks: 40 * SECOND
            },
            duration()
        ));
        // On the lattice, but too long.
        assert!(!is_legal(
            at,
            Boundary {
                start_ticks: 0,
                end_ticks: 70 * SECOND
            },
            duration()
        ));
        // The right length, but a start nothing proposed.
        assert!(!is_legal(
            at,
            Boundary {
                start_ticks: 11 * SECOND,
                end_ticks: 40 * SECOND
            },
            duration()
        ));
    }
}
