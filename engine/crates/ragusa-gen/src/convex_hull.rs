//! Convex-hull bounds, mirroring txgraffiti 0.4.1 `generators.convex_hull`.
//!
//! txgraffiti runs scipy's `ConvexHull`, which calls qhull with option `Qt`, on the points
//! `(features…, target)` of the objects where `hypothesis` holds; here qhull runs through the
//! `qhull` crate with the same option. A facet `a·p + b0 = 0` whose target coefficient `a_y` has
//! magnitude at least 1e-8 becomes `target <= rhs` (if `a_y > 0`) or `target >= rhs`, with
//! `rhs = Σ (−a_j / a_y)·feature_j + (−b0 / a_y)`; every coefficient is rounded with
//! `limit_denominator`, and those below 1e-8 in magnitude are dropped.
//!
//! scipy bundles its own qhull and qhull-sys another, so facet order is not compared (gate G-B2
//! compares each call's bounds as a multiset). When qhull rejects the input, txgraffiti retries
//! with `QJ`, which joggles points randomly; the retry happens here too and is flagged in
//! [`HullBounds::joggled`], because neither system's joggled output is reproducible.
//!
//! Objects with a missing or interval value in the target or a feature are left out of the fit
//! (txgraffiti would pass NaN to qhull and its `safe_generator` would yield nothing for the call).

use qhull::Qh;
use ragusa_table::{BoolColumns, Table};

use crate::bound::{Bound, Op};
use crate::rational::limit_denominator;
use crate::ratios::MAX_DENOMINATOR;
use crate::{column, exact_values, hypothesis_column, GenError};

/// txgraffiti's `tol`: side facets and negligible coefficients.
pub const TOL: f64 = 1e-8;

pub struct HullBounds {
    pub bounds: Vec<Bound>,
    /// qhull rejected the exact points and the bounds come from the joggled (`QJ`) retry.
    pub joggled: bool,
}

pub fn convex_hull(
    table: &Table,
    target: &str,
    features: &[&str],
    hypothesis: &str,
) -> Result<HullBounds, GenError> {
    let numeric = table.numeric().ok_or(GenError::NoNumericBlock)?;
    let boolean = table.boolean().ok_or(GenError::NoBooleanBlock)?;
    let target_values = exact_values(numeric, column(numeric.names(), target)?);
    let feature_columns = features
        .iter()
        .map(|f| column(numeric.names(), f).map(|c| exact_values(numeric, c)))
        .collect::<Result<Vec<_>, _>>()?;
    let h = hypothesis_column(boolean.names(), hypothesis)?;
    let rows: Vec<usize> = (0..table.n_objects())
        .filter(|&r| {
            h.is_none_or(|h| boolean.bits().get(r, h))
                && !target_values[r].is_nan()
                && feature_columns.iter().all(|col| !col[r].is_nan())
        })
        .collect();

    let k = features.len();
    if rows.len() < k + 2 {
        return Ok(HullBounds {
            bounds: Vec::new(),
            joggled: false,
        });
    }
    let mut points = Vec::with_capacity(rows.len() * (k + 1));
    for &r in &rows {
        points.extend(feature_columns.iter().map(|col| col[r]));
        points.push(target_values[r]);
    }

    let (qh, joggled) = match hull(&points, k + 1, false) {
        Ok(qh) => (qh, false),
        Err(_) => (hull(&points, k + 1, true).map_err(GenError::Qhull)?, true),
    };

    let round = |x: f64| limit_denominator(x, MAX_DENOMINATOR).ok_or(GenError::Rounding(x));
    let mut bounds = Vec::new();
    for facet in qh.facets() {
        let Some(normal) = facet.normal() else {
            continue;
        };
        let (a_features, a_y) = (&normal[..k], normal[k]);
        if a_y.abs() < TOL {
            continue;
        }
        let constant = round(-facet.offset() / a_y)?;
        let mut terms = Vec::new();
        for (name, &a) in features.iter().zip(a_features) {
            let coefficient = -a / a_y;
            if coefficient.abs() < TOL {
                continue;
            }
            terms.push((name.to_string(), round(coefficient)?));
        }
        let op = if a_y > 0.0 { Op::Le } else { Op::Ge };
        bounds.push(Bound::new(hypothesis, target, op, terms, constant));
    }
    Ok(HullBounds { bounds, joggled })
}

fn hull(points: &[f64], dim: usize, joggle: bool) -> Result<Qh<'static>, String> {
    let args: &[&str] = if joggle { &["Qt", "QJ"] } else { &["Qt"] };
    Qh::builder()
        .qhull_args(args)
        .map_err(|e| format!("{e:?}"))?
        .build_managed(dim, points.to_vec())
        .map_err(|e| format!("{e:?}"))
}
