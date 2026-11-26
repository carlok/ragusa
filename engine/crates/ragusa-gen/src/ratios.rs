//! Ratio bounds, mirroring txgraffiti 0.4.1 `generators.ratios`.
//!
//! Over the objects where `hypothesis` holds and the feature is non-zero, take the minimum and
//! maximum of `target / feature`, round each with Python's `limit_denominator`, and emit
//! `target >= c_min·feature` then `target <= c_max·feature`, feature by feature.
//!
//! One deliberate difference: when every ratio for a feature is NaN, txgraffiti raises and its
//! `safe_generator` returns nothing for the whole call, so the remaining features are dropped. Here that
//! feature is skipped and the others still run. Interval cells are fitted as missing values.

use ragusa_table::{BoolColumns, Table};

use crate::bound::{Bound, Op};
use crate::rational::{limit_denominator, Rational};
use crate::{column, exact_values, hypothesis_column, GenError};

/// txgraffiti calls `limit_denominator()` with Python's default.
pub const MAX_DENOMINATOR: i128 = 1_000_000;

pub fn ratios(
    table: &Table,
    target: &str,
    features: &[&str],
    hypothesis: &str,
) -> Result<Vec<Bound>, GenError> {
    let numeric = table.numeric().ok_or(GenError::NoNumericBlock)?;
    let boolean = table.boolean().ok_or(GenError::NoBooleanBlock)?;
    let target_values = exact_values(numeric, column(numeric.names(), target)?);
    let h = hypothesis_column(boolean.names(), hypothesis)?;
    let rows: Vec<usize> = (0..table.n_objects())
        .filter(|&r| h.is_none_or(|h| boolean.bits().get(r, h)))
        .collect();

    let mut out = Vec::new();
    if rows.is_empty() {
        return Ok(out);
    }
    for &feature in features {
        let feature_values = exact_values(numeric, column(numeric.names(), feature)?);
        let (mut lo, mut hi, mut nonzero) = (f64::INFINITY, f64::NEG_INFINITY, false);
        for &r in &rows {
            // NaN != 0, so a missing feature value counts as non-zero, as in pandas.
            if feature_values[r] != 0.0 {
                nonzero = true;
                let q = target_values[r] / feature_values[r];
                if !q.is_nan() {
                    lo = lo.min(q);
                    hi = hi.max(q);
                }
            }
        }
        if !nonzero || lo > hi {
            continue;
        }
        let round = |x: f64| limit_denominator(x, MAX_DENOMINATOR).ok_or(GenError::Rounding(x));
        for (op, c) in [(Op::Ge, round(lo)?), (Op::Le, round(hi)?)] {
            out.push(Bound::new(
                hypothesis,
                target,
                op,
                [(feature.to_string(), c)],
                Rational::ZERO,
            ));
        }
    }
    Ok(out)
}
