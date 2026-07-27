//! The one definition of a confidence pair, shared by everything that
//! summarizes an observation.
//!
//! There is a second implementation of this in Python
//! (`clipmill_worker_sdk.confidence`), on the far side of the worker boundary,
//! and the two must agree to the last digit: a transcript whose confidence
//! depended on which language happened to summarize it would be a transcript
//! nobody could reproduce. Keeping a third copy in the daemon was how that
//! would have started going wrong, so the daemon calls this.

/// Nearest-rank `(p50, p10)` over a set of scores.
///
/// The rank is floored after adding a half rather than handed to `round`,
/// because Rust rounds halves away from zero and Python rounds them to even —
/// the one place where two correct implementations of "round" disagree, and
/// exactly the sort of difference a byte-identical artifact cannot absorb.
#[must_use]
pub fn distribution(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let pick = |fraction: f64| {
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a rank inside a list whose length fits in usize"
        )]
        let rank = (fraction * (ordered.len() - 1) as f64 + 0.5).floor() as usize;
        ordered[rank.min(ordered.len() - 1)]
    };
    (pick(0.5), pick(0.1))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use super::distribution;

    #[test]
    fn an_empty_set_is_worth_nothing_rather_than_everything() {
        assert_eq!(distribution(&[]), (0.0, 0.0));
    }

    #[test]
    fn one_value_is_both_quantiles() {
        assert_eq!(distribution(&[0.7]), (0.7, 0.7));
    }

    /// The pessimistic reading is the point of carrying two numbers: a set with
    /// one bad member must not look as good as a set without one.
    #[test]
    fn the_tenth_percentile_follows_the_worst_members() {
        let good = [0.9, 0.92, 0.94, 0.96, 0.98];
        let mixed = [0.1, 0.92, 0.94, 0.96, 0.98];
        assert_eq!(distribution(&good).0, distribution(&mixed).0);
        assert!(distribution(&mixed).1 < distribution(&good).1);
    }

    #[test]
    fn the_input_order_does_not_change_the_answer() {
        let forward = [0.1, 0.3, 0.5, 0.7, 0.9];
        let backward = [0.9, 0.7, 0.5, 0.3, 0.1];
        assert_eq!(distribution(&forward), distribution(&backward));
    }
}
