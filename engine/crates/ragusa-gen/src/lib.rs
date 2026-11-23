//! ragusa-gen: conjecture generators over a [`ragusa_table::Table`].
//!
//! The first generators reproduce txgraffiti 0.4.1's documented shapes, because they have a
//! published implementation to check against (gate G-B2). Every generator emits [`Bound`]s in
//! one canonical form, so candidates from any source compare by equality.

mod bound;
pub mod convex_hull;
pub mod implications;
pub mod lp;
mod rational;
pub mod ratios;

use std::borrow::Cow;
use std::fmt;

use ragusa_table::NumericBlock;

pub use bound::{Bound, Op, ALWAYS};
pub use rational::{limit_denominator, Rational};

#[derive(Clone, Debug, PartialEq)]
pub enum GenError {
    NoNumericBlock,
    NoBooleanBlock,
    UnknownColumn(String),
    /// A fitted value `limit_denominator` cannot represent (NaN or infinite).
    Rounding(f64),
    /// qhull rejected the points even with joggling.
    Qhull(String),
}

/// The boolean column a generator restricts its rows to, or `None` for [`ALWAYS`] (every row).
pub(crate) fn hypothesis_column(
    names: &[String],
    hypothesis: &str,
) -> Result<Option<usize>, GenError> {
    if hypothesis == ALWAYS {
        Ok(None)
    } else {
        column(names, hypothesis).map(Some)
    }
}

/// The values of numeric column `c` that generators may fit on: exact cells, NaN where a cell is
/// missing or a genuine interval. Borrowed when the block has no intervals.
pub(crate) fn exact_values(numeric: &NumericBlock, c: usize) -> Cow<'_, [f64]> {
    if !numeric.has_intervals() {
        return Cow::Borrowed(numeric.lower(c));
    }
    Cow::Owned(
        numeric
            .lower(c)
            .iter()
            .zip(numeric.upper(c))
            .map(|(lo, hi)| {
                if lo.to_bits() == hi.to_bits() {
                    *lo
                } else {
                    f64::NAN
                }
            })
            .collect(),
    )
}

impl fmt::Display for GenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenError::NoNumericBlock => write!(f, "table has no numeric block"),
            GenError::NoBooleanBlock => write!(f, "table has no boolean block"),
            GenError::UnknownColumn(name) => write!(f, "unknown column {name:?}"),
            GenError::Rounding(x) => write!(f, "cannot round {x} to a fraction"),
            GenError::Qhull(message) => write!(f, "qhull: {message}"),
        }
    }
}

impl std::error::Error for GenError {}

pub(crate) fn column(names: &[String], name: &str) -> Result<usize, GenError> {
    names
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| GenError::UnknownColumn(name.to_string()))
}
