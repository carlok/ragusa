//! Checks of given bounds on a table, for candidates found by earlier runs (Sprint 12).
//!
//! [`check_bounds`] evaluates each bound with the engine's interval rules and reports, besides
//! refuted / confirmed / undecided objects, the objects where it is tight and how many of those sit
//! at the largest value of an edge column (for KnotInfo, the crossing number: a bound tight only at
//! the edge of the table is a candidate range artefact), and on how many edge objects the hypothesis
//! is recorded at all.
//!
//! [`complete_with_catalogue`] fills missing cells of catalogue columns with the intervals the knot
//! catalogue implies from the other cells of the same knot, so that a candidate contradicted by a
//! known bound on an empty cell is refuted instead of undecided.

use ragusa_filter::{Evaluated, Verdict};
use ragusa_gen::{Bound, Op, ALWAYS};
use ragusa_oracle::knots::Catalogue;
use ragusa_table::{NumericBlock, Pin, Table};
use rayon::prelude::*;
use serde_json::{json, Value};

const EXAMPLES: usize = 5;

pub struct Checked {
    pub bound: Bound,
    pub applicable: usize,
    pub refuted: usize,
    pub confirmed: usize,
    pub undecided: usize,
    pub tight: usize,
    pub tight_at_edge: usize,
    /// Objects at the edge where the hypothesis value is known (all of them for `TRUE`).
    pub hypothesis_known_at_edge: usize,
    pub refuted_examples: Vec<usize>,
    pub tight_examples: Vec<usize>,
}

pub struct EdgeColumn {
    pub column: String,
    pub max: f64,
    pub objects: usize,
}

pub struct CheckReport {
    pub pin: Pin,
    pub objects: Vec<String>,
    pub edge: Option<EdgeColumn>,
    pub results: Vec<Checked>,
}

pub fn check_bounds(
    table: &Table,
    bounds: Vec<Bound>,
    edge: Option<&str>,
) -> Result<CheckReport, String> {
    let numeric = table.numeric().ok_or("table has no numeric block")?;
    let boolean = table.boolean().ok_or("table has no boolean block")?;
    let n = table.n_objects();
    let (at_edge, edge_column) = match edge {
        None => (vec![false; n], None),
        Some(name) => {
            let c = numeric
                .names()
                .iter()
                .position(|x| x == name)
                .ok_or_else(|| format!("unknown edge column {name:?}"))?;
            let (lo, hi) = (numeric.lower(c), numeric.upper(c));
            let exact = |r: usize| !lo[r].is_nan() && lo[r].to_bits() == hi[r].to_bits();
            let max = (0..n)
                .filter(|&r| exact(r))
                .map(|r| lo[r])
                .fold(f64::NEG_INFINITY, f64::max);
            let flags: Vec<bool> = (0..n).map(|r| exact(r) && lo[r] == max).collect();
            let objects = flags.iter().filter(|&&f| f).count();
            (
                flags,
                Some(EdgeColumn {
                    column: name.to_string(),
                    max,
                    objects,
                }),
            )
        }
    };
    let ev = Evaluated::new(table, bounds.clone()).map_err(|e| e.to_string())?;
    let results = bounds
        .into_par_iter()
        .enumerate()
        .map(|(i, bound)| {
            let h = (bound.hypothesis != ALWAYS).then(|| {
                boolean
                    .names()
                    .iter()
                    .position(|c| *c == bound.hypothesis)
                    .expect("validated by Evaluated::new")
            });
            let mut c = Checked {
                bound,
                applicable: 0,
                refuted: 0,
                confirmed: 0,
                undecided: 0,
                tight: 0,
                tight_at_edge: 0,
                hypothesis_known_at_edge: (0..n)
                    .filter(|&r| at_edge[r] && h.is_none_or(|h| boolean.is_known(r, h)))
                    .count(),
                refuted_examples: Vec::new(),
                tight_examples: Vec::new(),
            };
            for (r, &edge) in at_edge.iter().enumerate() {
                match ev.verdict(i, r) {
                    Verdict::NotApplicable => continue,
                    Verdict::Refuted => {
                        c.refuted += 1;
                        if c.refuted_examples.len() < EXAMPLES {
                            c.refuted_examples.push(r);
                        }
                    }
                    Verdict::Confirmed => c.confirmed += 1,
                    Verdict::Undecided => c.undecided += 1,
                }
                c.applicable += 1;
                if ev.tight(i, r) {
                    c.tight += 1;
                    c.tight_at_edge += usize::from(edge);
                    if c.tight_examples.len() < EXAMPLES {
                        c.tight_examples.push(r);
                    }
                }
            }
            c
        })
        .collect();
    Ok(CheckReport {
        pin: Pin::of(table),
        objects: table.objects().to_vec(),
        edge: edge_column,
        results,
    })
}

impl CheckReport {
    pub fn to_json(&self) -> Value {
        let names = |rows: &[usize]| -> Vec<&str> {
            rows.iter().map(|&r| self.objects[r].as_str()).collect()
        };
        json!({
            "format": "ragusa-check/1",
            "pin": self.pin.to_json(),
            "edge": self.edge.as_ref().map(|e| json!({"column": e.column, "max": e.max, "objects": e.objects})),
            "results": self.results.iter().map(|c| json!({
                "bound": c.bound.to_string(),
                "applicable": c.applicable,
                "refuted": c.refuted,
                "confirmed": c.confirmed,
                "undecided": c.undecided,
                "tight": c.tight,
                "tight_at_edge": c.tight_at_edge,
                "hypothesis_known_at_edge": c.hypothesis_known_at_edge,
                "refuted_examples": names(&c.refuted_examples),
                "tight_examples": names(&c.tight_examples),
            })).collect::<Vec<_>>(),
        })
    }
}

/// Knots with a crossing number below this are trivial, and the catalogue says nothing about them.
const NON_TRIVIAL_FROM: f64 = 1.0;
/// Propagation rounds: a cell filled in one round can bound another in the next.
const ROUNDS: usize = 3;

/// A copy of `table` in which every missing cell of a column the catalogue bounds is replaced by the
/// interval the catalogue implies from the same knot's other cells, rounded inwards to integers
/// (every catalogue column is an integer invariant); a side the catalogue does not bound stays
/// infinite, and a cell with no implied bound at all stays missing. Known cells are never changed.
/// Returns the table, the number of cells filled, and the number of cells where the implied bounds
/// contradict each other (left missing; a sign of a data or transcription problem).
pub fn complete_with_catalogue(
    table: &Table,
    catalogue: &Catalogue,
) -> Result<(Table, usize, usize), String> {
    let numeric = table.numeric().ok_or("table has no numeric block")?;
    let names = numeric.names();
    let n = table.n_objects();
    let col = |name: &str| names.iter().position(|c| c == name);
    let mut lower: Vec<Vec<f64>> = (0..names.len())
        .map(|c| numeric.lower(c).to_vec())
        .collect();
    let mut upper: Vec<Vec<f64>> = (0..names.len())
        .map(|c| numeric.upper(c).to_vec())
        .collect();
    let missing: Vec<Vec<bool>> = lower
        .iter()
        .map(|column| column.iter().map(|v| v.is_nan()).collect())
        .collect();
    for c in 0..names.len() {
        for r in 0..n {
            if missing[c][r] {
                lower[c][r] = f64::NEG_INFINITY;
                upper[c][r] = f64::INFINITY;
            }
        }
    }
    let non_trivial: Vec<bool> = match col("crossing_number") {
        Some(c) => (0..n)
            .map(|r| numeric.lower(c)[r] >= NON_TRIVIAL_FROM)
            .collect(),
        None => vec![true; n],
    };
    let (bounds, _) = catalogue.bounds();
    let mut touched = vec![vec![false; n]; names.len()];
    for _ in 0..ROUNDS {
        for cb in &bounds {
            let b = &cb.bound;
            let [(feature, a)] = b.terms.as_slice() else {
                continue;
            };
            let (Some(t), Some(f)) = (col(&b.target), col(feature)) else {
                continue;
            };
            let (a, k) = (a.to_f64(), b.constant.to_f64());
            for r in (0..n).filter(|&r| missing[t][r] && non_trivial[r]) {
                let (fl, fh) = (lower[f][r], upper[f][r]);
                match b.op {
                    Op::Le => {
                        let hi = if a >= 0.0 { a * fh + k } else { a * fl + k };
                        if hi.is_finite() && hi < upper[t][r] {
                            upper[t][r] = hi;
                            touched[t][r] = true;
                        }
                    }
                    Op::Ge => {
                        let lo = if a >= 0.0 { a * fl + k } else { a * fh + k };
                        if lo.is_finite() && lo > lower[t][r] {
                            lower[t][r] = lo;
                            touched[t][r] = true;
                        }
                    }
                }
            }
        }
    }
    let (mut filled, mut contradictions) = (0, 0);
    for c in 0..names.len() {
        for r in 0..n {
            if !missing[c][r] {
                continue;
            }
            let (mut lo, mut hi) = (lower[c][r], upper[c][r]);
            if touched[c][r] {
                if lo.is_finite() {
                    lo = (lo - 1e-9).ceil();
                }
                if hi.is_finite() {
                    hi = (hi + 1e-9).floor();
                }
            }
            if !touched[c][r] {
                (lo, hi) = (f64::NAN, f64::NAN);
            } else if lo > hi {
                contradictions += 1;
                (lo, hi) = (f64::NAN, f64::NAN);
            } else {
                filled += 1;
            }
            lower[c][r] = lo;
            upper[c][r] = hi;
        }
    }
    let completed = Table::new(
        format!("{}[catalogue-completed]", table.name()),
        table.objects().to_vec(),
        Some(NumericBlock::from_intervals(names.to_vec(), lower, upper)),
        table.boolean().cloned(),
        table.skipped().to_vec(),
        table.sources().to_vec(),
    );
    Ok((completed, filled, contradictions))
}

#[cfg(test)]
mod tests {
    use ragusa_table::{BooleanBlock, PackedColumns};

    use super::*;

    fn knots() -> Table {
        Table::new(
            "knots",
            vec!["a".into(), "b".into()],
            Some(NumericBlock::from_columns(
                vec!["crossing_number".into(), "three_genus".into()],
                vec![vec![3.0, 4.0], vec![5.0, f64::NAN]],
            )),
            Some(BooleanBlock::new(
                vec!["alternating".into()],
                PackedColumns::zeros(2, 1),
            )),
            vec![],
            vec![],
        )
    }

    #[test]
    fn completion_refutes_what_a_missing_cell_hid() {
        let table = knots();
        let bound: Bound = "(TRUE) → (three_genus >= 3)".parse().unwrap();
        let before = check_bounds(&table, vec![bound.clone()], Some("crossing_number")).unwrap();
        let c = &before.results[0];
        assert_eq!((c.refuted, c.confirmed, c.undecided), (0, 1, 1));
        let edge = before.edge.as_ref().unwrap();
        assert_eq!((edge.max, edge.objects), (4.0, 1));
        assert_eq!(c.hypothesis_known_at_edge, 1);

        // The catalogue gives three_genus <= crossing_number / 2 = 2 on knot b.
        let (completed, filled, contradictions) =
            complete_with_catalogue(&table, &Catalogue::standard().unwrap()).unwrap();
        assert_eq!((filled, contradictions), (1, 0));
        let numeric = completed.numeric().unwrap();
        assert_eq!(numeric.upper(1)[1], 2.0);
        assert_eq!(numeric.lower(1)[1], f64::NEG_INFINITY);
        let after = check_bounds(&completed, vec![bound], None).unwrap();
        let c = &after.results[0];
        assert_eq!((c.refuted, c.confirmed, c.undecided), (1, 1, 0));
        assert_eq!(after.to_json()["results"][0]["refuted_examples"][0], "b");
    }
}
