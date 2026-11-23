//! Sum-of-slacks linear bounds, mirroring txgraffiti 0.4.1 `generators.linear_programming`.
//!
//! Over the objects where `hypothesis` holds, minimise `Σ s_i` subject to
//! `a·x_i + b − y_i = s_i` (upper bound) or `y_i − (a·x_i + b) = s_i` (lower bound), with
//! `s_i ≥ 0` and `a`, `b` free; then round `b` and each |a_j| ≥ 1e-8 with `limit_denominator`.
//! txgraffiti solves this model with CBC through PuLP; here HiGHS solves the same model on one
//! thread. The optimal `(a, b)` is often not unique, so the two solvers can return different
//! bounds with the same total slack; gate G-B2 checks the optimum, not the vertex.
//!
//! txgraffiti's error path is kept: no rows yields nothing, a non-optimal upper solve yields
//! nothing, a non-optimal lower solve keeps the upper bound. Objects with a missing or interval
//! value in the target or a feature are left out of the fit.

use highs::{HighsModelStatus, RowProblem, Sense};
use ragusa_table::{BoolColumns, Table};

use crate::bound::{Bound, Op};
use crate::rational::limit_denominator;
use crate::ratios::MAX_DENOMINATOR;
use crate::{column, exact_values, hypothesis_column, GenError};

/// txgraffiti's `tol` for negligible coefficients.
pub const TOL: f64 = 1e-8;

#[derive(Clone, Copy)]
enum Side {
    Upper,
    Lower,
}

pub fn linear_programming(
    table: &Table,
    target: &str,
    features: &[&str],
    hypothesis: &str,
) -> Result<Vec<Bound>, GenError> {
    let numeric = table.numeric().ok_or(GenError::NoNumericBlock)?;
    let boolean = table.boolean().ok_or(GenError::NoBooleanBlock)?;
    let y = exact_values(numeric, column(numeric.names(), target)?);
    let x_columns = features
        .iter()
        .map(|f| column(numeric.names(), f).map(|c| exact_values(numeric, c)))
        .collect::<Result<Vec<_>, _>>()?;
    let x: Vec<&[f64]> = x_columns.iter().map(|c| c.as_ref()).collect();
    let h = hypothesis_column(boolean.names(), hypothesis)?;
    let rows: Vec<usize> = (0..table.n_objects())
        .filter(|&r| {
            h.is_none_or(|h| boolean.bits().get(r, h))
                && !y[r].is_nan()
                && x.iter().all(|col| !col[r].is_nan())
        })
        .collect();
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let round = |v: f64| limit_denominator(v, MAX_DENOMINATOR).ok_or(GenError::Rounding(v));
    let mut out = Vec::new();
    for (side, op) in [(Side::Upper, Op::Le), (Side::Lower, Op::Ge)] {
        let Some((a, b)) = solve(&x, &y, &rows, side) else {
            break;
        };
        let mut terms = Vec::new();
        for (name, &coefficient) in features.iter().zip(&a) {
            if coefficient.abs() < TOL {
                continue;
            }
            terms.push((name.to_string(), round(coefficient)?));
        }
        out.push(Bound::new(hypothesis, target, op, terms, round(b)?));
    }
    Ok(out)
}

/// `(a, b)` at the optimum, or `None` when HiGHS does not report an optimal solution.
fn solve(x: &[&[f64]], y: &[f64], rows: &[usize], side: Side) -> Option<(Vec<f64>, f64)> {
    let mut pb = RowProblem::default();
    let a: Vec<_> = x.iter().map(|_| pb.add_column::<f64, _>(0.0, ..)).collect();
    let b = pb.add_column::<f64, _>(0.0, ..);
    for &r in rows {
        let s = pb.add_column(1.0, 0.0..);
        let mut factors: Vec<_> = a
            .iter()
            .zip(x)
            .map(|(&col, values)| (col, values[r]))
            .collect();
        factors.push((b, 1.0));
        // Upper: a·x + b − s = y.  Lower: a·x + b + s = y.
        factors.push((
            s,
            match side {
                Side::Upper => -1.0,
                Side::Lower => 1.0,
            },
        ));
        pb.add_row(y[r]..=y[r], &factors);
    }
    let mut model = pb.optimise(Sense::Minimise);
    model.make_quiet();
    model.set_option("threads", 1);
    let solved = model.solve();
    if solved.status() != HighsModelStatus::Optimal {
        return None;
    }
    let columns = solved.get_solution().columns().to_vec();
    Some((columns[..x.len()].to_vec(), columns[x.len()]))
}
