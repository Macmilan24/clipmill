//! The search's boundary lattice: the starts and ends discovery offered.
//!
//! The boundary optimizer chose the clip's cut, and its runner-up, from these
//! pairs, so a cut the search returns is checked against them before anything
//! is built from it. A cut a person sets is not held to the lattice — it may
//! land between any two words (R63, see `words`).

/// Where a clip may begin and end, as discovery published it.
#[derive(Clone, Copy, Debug)]
pub struct Lattice<'a> {
    pub starts: &'a [i64],
    pub ends: &'a [i64],
}

/// A start and an end, in source ticks.
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

/// How long the search was allowed to make a clip, echoed from the discovery
/// document so a cut it returns can be held to it.
#[derive(Clone, Copy, Debug)]
pub struct Duration {
    pub min_ticks: i64,
    pub max_ticks: i64,
}

/// Whether a boundary is one the lattice actually offers.
///
/// Checked rather than assumed for the cuts the ranking published — a document
/// read back from the store has been through processes this one does not
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
    use super::{Boundary, Duration, Lattice, is_legal};

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
    fn a_cut_the_search_returned_is_checked_against_the_lattice() {
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
