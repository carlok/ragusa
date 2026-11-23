//! Candidates evaluated on demand. A bound's value on an object is computed from the table when it
//! is asked for and never stored, so memory does not grow with candidates × objects.
//!
//! Numeric cells are intervals (a point when lower = upper), so a bound's right-hand side on an
//! object is an interval, and so is its slack. The rules:
//!
//! - **violated**: the hypothesis holds and the bound fails for every value in the intervals, by
//!   more than the tolerance;
//! - **tight**: the hypothesis holds, every value involved is exact, and the slack is within the
//!   tolerance;
//! - a missing value (NaN) is never violated and never tight;
//! - a hypothesis whose value is unknown does not hold.
//!
//! On a table of exact values these reduce, bit for bit, to the rules used before intervals existed.

use std::fmt;

use ragusa_gen::{Bound, Op, ALWAYS};
use ragusa_table::{BoolColumns, NumericBlock, PackedColumns, Table};

/// Relative tolerance for tightness and violation, scaled by `max(1, |rhs|)`.
///
/// Right-hand sides are evaluated in `f64` from exact rationals, so an exactly tight bound can
/// land a few ulps off. txgraffiti instead tests `slack == 0` exactly, and on graph_data calls
/// convex-hull bounds false at slack −2e-15; this is a deliberate difference, reported by G-B4.
pub const SLACK_TOL: f64 = 1e-9;

#[derive(Clone, Debug, PartialEq)]
pub enum EvalError {
    NoNumericBlock,
    NoBooleanBlock,
    UnknownColumn(String),
    /// The table has a boolean column named [`ALWAYS`].
    ReservedColumn,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::NoNumericBlock => write!(f, "table has no numeric block"),
            EvalError::NoBooleanBlock => write!(f, "table has no boolean block"),
            EvalError::UnknownColumn(name) => write!(f, "unknown column {name:?}"),
            EvalError::ReservedColumn => {
                write!(
                    f,
                    "boolean column {ALWAYS:?} is reserved for the hypothesis that always holds"
                )
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Lower and upper values of one numeric column.
type Column<'t> = (&'t [f64], &'t [f64]);

struct Compiled<'t> {
    /// `None` for [`ALWAYS`].
    hypothesis: Option<usize>,
    target: Column<'t>,
    terms: Vec<(&'t [f64], &'t [f64], f64)>,
    constant: f64,
    op: Op,
}

pub struct Evaluated<'t> {
    bounds: Vec<Bound>,
    n_objects: usize,
    hypotheses: &'t PackedColumns,
    compiled: Vec<Compiled<'t>>,
}

fn column<'t>(numeric: &'t NumericBlock, name: &str) -> Result<Column<'t>, EvalError> {
    numeric
        .names()
        .iter()
        .position(|c| c == name)
        .map(|c| (numeric.lower(c), numeric.upper(c)))
        .ok_or_else(|| EvalError::UnknownColumn(name.to_string()))
}

impl<'t> Evaluated<'t> {
    pub fn new(table: &'t Table, bounds: Vec<Bound>) -> Result<Evaluated<'t>, EvalError> {
        let numeric = table.numeric().ok_or(EvalError::NoNumericBlock)?;
        let boolean = table.boolean().ok_or(EvalError::NoBooleanBlock)?;
        if boolean.names().iter().any(|c| c == ALWAYS) {
            return Err(EvalError::ReservedColumn);
        }
        let compiled = bounds
            .iter()
            .map(|bound| {
                let hypothesis = if bound.hypothesis == ALWAYS {
                    None
                } else {
                    Some(
                        boolean
                            .names()
                            .iter()
                            .position(|c| *c == bound.hypothesis)
                            .ok_or_else(|| EvalError::UnknownColumn(bound.hypothesis.clone()))?,
                    )
                };
                let terms = bound
                    .terms
                    .iter()
                    .map(|(f, c)| column(numeric, f).map(|(lo, hi)| (lo, hi, c.to_f64())))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Compiled {
                    hypothesis,
                    target: column(numeric, &bound.target)?,
                    terms,
                    constant: bound.constant.to_f64(),
                    op: bound.op,
                })
            })
            .collect::<Result<Vec<_>, EvalError>>()?;
        Ok(Evaluated {
            bounds,
            n_objects: table.n_objects(),
            hypotheses: boolean.bits(),
            compiled,
        })
    }

    pub fn len(&self) -> usize {
        self.bounds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bounds.is_empty()
    }

    pub fn n_objects(&self) -> usize {
        self.n_objects
    }

    pub fn bounds(&self) -> &[Bound] {
        &self.bounds
    }

    pub fn bound(&self, i: usize) -> &Bound {
        &self.bounds[i]
    }

    /// The hypothesis of candidate `i` holds on object `r` (an unknown value does not hold).
    pub fn applies(&self, i: usize, r: usize) -> bool {
        self.compiled[i]
            .hypothesis
            .is_none_or(|h| self.hypotheses.get(r, h))
    }

    /// The right-hand side of candidate `i` on object `r` as `(lower, upper)`.
    pub fn rhs_bounds(&self, i: usize, r: usize) -> (f64, f64) {
        let c = &self.compiled[i];
        c.terms
            .iter()
            .fold((c.constant, c.constant), |(lo, hi), &(xl, xh, k)| {
                if k >= 0.0 {
                    (lo + k * xl[r], hi + k * xh[r])
                } else {
                    (lo + k * xh[r], hi + k * xl[r])
                }
            })
    }

    /// Room left by the bound as `(lower, upper)`: `rhs − target` for `<=`, `target − rhs` for
    /// `>=`, over every value in the intervals. NaN if a value is missing.
    pub fn slack_bounds(&self, i: usize, r: usize) -> (f64, f64) {
        let c = &self.compiled[i];
        let (lo, hi) = self.rhs_bounds(i, r);
        let (yl, yh) = (c.target.0[r], c.target.1[r]);
        match c.op {
            Op::Le => (lo - yh, hi - yl),
            Op::Ge => (yl - hi, yh - lo),
        }
    }

    /// Every value candidate `i` involves on object `r` is a point (or missing).
    pub fn exact(&self, i: usize, r: usize) -> bool {
        let c = &self.compiled[i];
        c.target.0[r].to_bits() == c.target.1[r].to_bits()
            && c.terms
                .iter()
                .all(|&(xl, xh, _)| xl[r].to_bits() == xh[r].to_bits())
    }

    /// `SLACK_TOL · max(1, |rhs|)` over the finite ends of the right-hand side, so that an
    /// unbounded end (a cell known only from one side) does not make the tolerance infinite.
    pub fn tolerance(&self, i: usize, r: usize) -> f64 {
        let (lo, hi) = self.rhs_bounds(i, r);
        SLACK_TOL
            * [lo, hi]
                .into_iter()
                .filter(|v| v.is_finite())
                .fold(1.0_f64, |m, v| m.max(v.abs()))
    }

    /// The hypothesis holds and the bound fails for every value, by more than the tolerance.
    pub fn violated(&self, i: usize, r: usize) -> bool {
        self.applies(i, r) && self.slack_bounds(i, r).1 < -self.tolerance(i, r)
    }

    /// The hypothesis holds, every value is exact, and the bound is attained within the tolerance.
    pub fn tight(&self, i: usize, r: usize) -> bool {
        self.applies(i, r)
            && self.exact(i, r)
            && self.slack_bounds(i, r).0.abs() <= self.tolerance(i, r)
    }

    /// What object `r` says about candidate `i`: [`Verdict::Refuted`] exactly when
    /// [`Evaluated::violated`]; [`Verdict::Confirmed`] when the bound holds for every value in the
    /// intervals, within the tolerance; [`Verdict::Undecided`] otherwise (a missing value, or
    /// intervals on both sides of the bound).
    pub fn verdict(&self, i: usize, r: usize) -> Verdict {
        if !self.applies(i, r) {
            return Verdict::NotApplicable;
        }
        let (lo, hi) = self.slack_bounds(i, r);
        let tol = self.tolerance(i, r);
        if hi < -tol {
            Verdict::Refuted
        } else if lo >= -tol {
            Verdict::Confirmed
        } else {
            Verdict::Undecided
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The hypothesis does not hold on the object (or its value is unknown).
    NotApplicable,
    Refuted,
    Confirmed,
    Undecided,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ragusa_gen::Rational;
    use ragusa_table::BooleanBlock;

    /// x = 1 everywhere; y is [2, 4] on object 0, [4, 5] on object 1, missing on object 2.
    /// Hypothesis p is true on objects 0 and 1 and unknown on object 2.
    fn interval_table() -> Table {
        let mut bits = PackedColumns::zeros(3, 1);
        bits.set(0, 0);
        bits.set(1, 0);
        let known = bits.clone();
        Table::new(
            "intervals",
            vec!["a".into(), "b".into(), "c".into()],
            Some(NumericBlock::from_intervals(
                vec!["x".into(), "y".into()],
                vec![vec![1.0, 1.0, 1.0], vec![2.0, 4.0, f64::NAN]],
                vec![vec![1.0, 1.0, 1.0], vec![4.0, 5.0, f64::NAN]],
            )),
            Some(BooleanBlock::with_known(vec!["p".into()], bits, known)),
            vec![],
            vec![],
        )
    }

    fn bound(op: Op, k: i128) -> Bound {
        Bound::new(
            "p",
            "y",
            op,
            [("x".to_string(), Rational::new(1, 1))],
            Rational::new(k, 1),
        )
    }

    #[test]
    fn intervals_violate_only_when_certain() {
        let table = interval_table();
        let ev = Evaluated::new(&table, vec![bound(Op::Le, 2), bound(Op::Ge, 1)]).unwrap();
        // y <= x + 2 = 3: object 0 might satisfy it (y in [2, 4]); object 1 cannot (y in [4, 5]).
        assert_eq!(ev.slack_bounds(0, 0), (-1.0, 1.0));
        assert!(!ev.violated(0, 0) && !ev.tight(0, 0));
        assert!(ev.violated(0, 1));
        // object 2: the hypothesis value is unknown, so nothing applies there.
        assert!(!ev.applies(0, 2) && !ev.violated(0, 2));
        // y >= x + 1 = 2 holds on object 0 for every value but is not tight: y is not exact.
        assert_eq!(ev.slack_bounds(1, 0), (0.0, 2.0));
        assert!(!ev.violated(1, 0) && !ev.tight(1, 0));
    }
}
