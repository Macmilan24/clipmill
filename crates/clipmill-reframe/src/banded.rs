//! A symmetric positive-definite banded system, and the Cholesky that solves it.
//!
//! The crop path's normal equations are pentadiagonal — the acceleration term
//! couples each sample to the two on either side and nothing further — so the
//! matrix is a band of width two around the diagonal and every entry outside it
//! is structurally zero. Storing and factoring only the band turns an O(n³)
//! solve into O(n·b²), which is what makes ch. 18's claim of "microseconds of
//! compute" true and what makes an interactive nudge free.
//!
//! Hand-rolled rather than delegated. A LAPACK binding would be a system
//! dependency, a build-time toolchain requirement, and a source of
//! platform-dependent floating-point reduction order — and this is forty lines
//! of arithmetic whose failure mode is a matrix that is not positive definite,
//! which the caller can be told about honestly.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BandedError {
    #[error("a banded system needs at least one unknown")]
    Empty,
    #[error("the system is not positive definite at row {row}")]
    NotPositiveDefinite { row: usize },
}

/// A symmetric banded matrix, lower triangle, `bandwidth` sub-diagonals.
///
/// `data[k][i]` is the entry `k` rows below the diagonal in column `i`, so
/// `data[0]` is the diagonal itself. Entries that would fall off the end are
/// unused and stay zero.
#[derive(Clone, Debug)]
pub struct Banded {
    order: usize,
    bandwidth: usize,
    data: Vec<Vec<f64>>,
}

impl Banded {
    pub fn new(order: usize, bandwidth: usize) -> Self {
        Self {
            order,
            bandwidth,
            data: vec![vec![0.0; order]; bandwidth + 1],
        }
    }

    #[cfg(test)]
    pub fn order(&self) -> usize {
        self.order
    }

    /// Add to the entry at `(row, column)`, which must be inside the band.
    ///
    /// Out-of-band writes are dropped rather than panicking: the assemblers
    /// below walk over sample windows that run past the ends of the sequence,
    /// and clipping there is the arithmetic rather than an error.
    pub fn add(&mut self, row: usize, column: usize, value: f64) {
        let (high, low) = if row >= column {
            (row, column)
        } else {
            (column, row)
        };
        let offset = high - low;
        if offset <= self.bandwidth && high < self.order {
            self.data[offset][low] += value;
        }
    }

    fn at(&self, row: usize, column: usize) -> f64 {
        let (high, low) = if row >= column {
            (row, column)
        } else {
            (column, row)
        };
        let offset = high - low;
        if offset <= self.bandwidth && high < self.order {
            self.data[offset][low]
        } else {
            0.0
        }
    }

    /// Solve `self · x = rhs` by banded Cholesky.
    ///
    /// The factorization is done into a scratch band rather than in place, so a
    /// system that turns out not to be positive definite leaves the caller's
    /// matrix intact and reusable — which matters because the same assembly is
    /// re-solved with different weights when somebody nudges the camera.
    pub fn solve(&self, rhs: &[f64]) -> Result<Vec<f64>, BandedError> {
        if self.order == 0 || rhs.len() != self.order {
            return Err(BandedError::Empty);
        }
        let bandwidth = self.bandwidth;
        // `lower[k][i]` mirrors `data`: the factor's k-th sub-diagonal.
        let mut lower = vec![vec![0.0_f64; self.order]; bandwidth + 1];

        for column in 0..self.order {
            // Diagonal: a(i,i) minus the squares already placed in this row.
            let mut diagonal = self.at(column, column);
            for step in 1..=bandwidth.min(column) {
                let value = lower[step][column - step];
                diagonal -= value * value;
            }
            if diagonal <= 0.0 {
                return Err(BandedError::NotPositiveDefinite { row: column });
            }
            let pivot = diagonal.sqrt();
            lower[0][column] = pivot;

            // Sub-diagonal entries of this column.
            for offset in 1..=bandwidth {
                let row = column + offset;
                if row >= self.order {
                    break;
                }
                let mut sum = self.at(row, column);
                for step in 1..=bandwidth {
                    if step > column {
                        break;
                    }
                    let left = row - (column - step);
                    if left <= bandwidth {
                        sum -= lower[left][column - step] * lower[step][column - step];
                    }
                }
                lower[offset][column] = sum / pivot;
            }
        }

        // Forward substitution, then back substitution.
        let mut intermediate = vec![0.0_f64; self.order];
        for row in 0..self.order {
            let mut sum = rhs[row];
            for offset in 1..=bandwidth.min(row) {
                sum -= lower[offset][row - offset] * intermediate[row - offset];
            }
            intermediate[row] = sum / lower[0][row];
        }
        let mut answer = vec![0.0_f64; self.order];
        for row in (0..self.order).rev() {
            let mut sum = intermediate[row];
            for (offset, band) in lower.iter().enumerate().skip(1).take(bandwidth) {
                let column = row + offset;
                if column >= self.order {
                    break;
                }
                sum -= band[row] * answer[column];
            }
            answer[row] = sum / lower[0][row];
        }
        Ok(answer)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{Banded, BandedError};

    /// Multiply back, because a solver is only right if `A·x` returns `b`.
    fn multiply(matrix: &Banded, vector: &[f64]) -> Vec<f64> {
        (0..matrix.order())
            .map(|row| {
                (0..matrix.order())
                    .map(|column| matrix.at(row, column) * vector[column])
                    .sum()
            })
            .collect()
    }

    fn assert_solves(matrix: &Banded, rhs: &[f64]) {
        let answer = matrix.solve(rhs).expect("a positive definite system");
        for (produced, wanted) in multiply(matrix, &answer).iter().zip(rhs) {
            assert!(
                (produced - wanted).abs() < 1e-9,
                "A·x gave {produced}, wanted {wanted}"
            );
        }
    }

    #[test]
    fn a_diagonal_system_is_division() {
        let mut matrix = Banded::new(3, 2);
        for index in 0..3 {
            matrix.add(index, index, 2.0);
        }
        // Compared with a tolerance, not for equality: a Cholesky takes a square
        // root and divides by it, so even 2x = 2 comes back one ulp short.
        let answer = matrix.solve(&[2.0, 4.0, 6.0]).unwrap();
        for (produced, wanted) in answer.iter().zip([1.0, 2.0, 3.0]) {
            assert!((produced - wanted).abs() < 1e-12);
        }
    }

    #[test]
    fn a_tridiagonal_system_round_trips() {
        let mut matrix = Banded::new(6, 2);
        for index in 0..6 {
            matrix.add(index, index, 4.0);
            if index + 1 < 6 {
                matrix.add(index + 1, index, -1.0);
            }
        }
        assert_solves(&matrix, &[1.0, -2.0, 3.0, 0.5, -0.25, 7.0]);
    }

    /// The shape the crop solver actually produces: an acceleration term
    /// reaching two samples either side.
    #[test]
    fn a_pentadiagonal_system_round_trips() {
        let mut matrix = Banded::new(9, 2);
        for index in 0..9 {
            matrix.add(index, index, 6.0);
            if index + 1 < 9 {
                matrix.add(index + 1, index, -2.0);
            }
            if index + 2 < 9 {
                matrix.add(index + 2, index, 0.5);
            }
        }
        assert_solves(&matrix, &[3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0, 5.0]);
    }

    /// A matrix with a zero on the diagonal is not a system with an answer, and
    /// saying which row failed is the difference between a bug report and a
    /// shrug.
    #[test]
    fn an_indefinite_system_names_the_row_it_failed_at() {
        let mut matrix = Banded::new(3, 2);
        matrix.add(0, 0, 1.0);
        matrix.add(2, 2, 1.0);
        assert_eq!(
            matrix.solve(&[1.0, 1.0, 1.0]),
            Err(BandedError::NotPositiveDefinite { row: 1 })
        );
    }

    #[test]
    fn a_system_with_no_unknowns_is_refused() {
        assert_eq!(Banded::new(0, 2).solve(&[]), Err(BandedError::Empty));
        assert_eq!(Banded::new(2, 2).solve(&[1.0]), Err(BandedError::Empty));
    }

    /// Writes past the end of the band are the assembler running off the ends of
    /// the sequence, which is arithmetic rather than a mistake.
    #[test]
    fn out_of_band_writes_are_dropped_rather_than_panicking() {
        let mut matrix = Banded::new(3, 1);
        matrix.add(0, 2, 5.0);
        matrix.add(9, 9, 5.0);
        for index in 0..3 {
            matrix.add(index, index, 1.0);
        }
        assert_solves(&matrix, &[1.0, 2.0, 3.0]);
    }
}
