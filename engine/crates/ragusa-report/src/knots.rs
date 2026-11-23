//! The knot catalogue as an oracle.
//!
//! [`check_catalogue`] is the gate before any novelty claim: every catalogue bound is evaluated on
//! the table's non-trivial knots with the engine's interval rules. A bound is refuted on a knot when
//! it fails for every value in the intervals, confirmed when it holds for every value, undecided
//! otherwise (missing values, overlapping intervals). A refuted bound means a transcription error, a
//! wrong column mapping, a data error, or a convention mismatch; it is reported, never dropped.
//!
//! [`NoveltyOracle::classify`] sorts a single-feature bound on two catalogue columns into:
//! implied by one catalogue bound (a sufficient test, see [`implies`]); a column pair the catalogue
//! relates without implying this bound; a pair it does not relate; or outside the catalogue.

use std::cmp::Ordering;
use std::collections::HashSet;

use ragusa_filter::{Evaluated, Verdict};
use ragusa_gen::{Bound, Op, Rational};
use ragusa_oracle::knots::{Catalogue, CatalogueBound};
use ragusa_table::Table;

/// Knots with a crossing number below this are trivial, and the catalogue says nothing about them.
const NON_TRIVIAL_FROM: f64 = 1.0;

pub struct BoundCheck {
    pub refuted_on: Vec<usize>,
    pub confirmed: usize,
    pub undecided: usize,
}

pub struct CatalogueCheck {
    pub bounds: Vec<CatalogueBound>,
    /// Per bound: `None` when a column it names is not in the table.
    pub checks: Vec<Option<BoundCheck>>,
    /// Implied relations not stated because their larger side is an absolute value.
    pub skipped_absolute: usize,
    pub non_trivial_objects: usize,
}

impl CatalogueCheck {
    pub fn refuted(&self) -> usize {
        self.checks
            .iter()
            .flatten()
            .filter(|c| !c.refuted_on.is_empty())
            .count()
    }

    pub fn pass(&self) -> bool {
        self.refuted() == 0
    }
}

pub fn check_catalogue(table: &Table, catalogue: &Catalogue) -> Result<CatalogueCheck, String> {
    let numeric = table.numeric().ok_or("table has no numeric block")?;
    let has = |c: &str| numeric.names().iter().any(|n| n == c);
    let (bounds, skipped_absolute) = catalogue.bounds();
    let present: Vec<usize> = (0..bounds.len())
        .filter(|&k| {
            let b = &bounds[k].bound;
            has(&b.target) && b.terms.iter().all(|(f, _)| has(f))
        })
        .collect();
    let objects: Vec<usize> = match numeric.names().iter().position(|n| n == "crossing_number") {
        Some(c) => (0..table.n_objects())
            .filter(|&r| numeric.lower(c)[r] >= NON_TRIVIAL_FROM)
            .collect(),
        None => (0..table.n_objects()).collect(),
    };
    let ev = Evaluated::new(
        table,
        present.iter().map(|&k| bounds[k].bound.clone()).collect(),
    )
    .map_err(|e| e.to_string())?;

    let mut checks: Vec<Option<BoundCheck>> = (0..bounds.len()).map(|_| None).collect();
    for (i, &k) in present.iter().enumerate() {
        let mut check = BoundCheck {
            refuted_on: Vec::new(),
            confirmed: 0,
            undecided: 0,
        };
        for &r in &objects {
            match ev.verdict(i, r) {
                Verdict::Refuted => check.refuted_on.push(r),
                Verdict::Confirmed => check.confirmed += 1,
                // Catalogue bounds hold under ALWAYS, so every object is applicable.
                Verdict::Undecided | Verdict::NotApplicable => check.undecided += 1,
            }
        }
        checks[k] = Some(check);
    }
    Ok(CatalogueCheck {
        bounds,
        checks,
        skipped_absolute,
        non_trivial_objects: objects.len(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Novelty {
    /// Implied by the catalogue bound with this index.
    Implied(usize),
    /// Both columns are in the catalogue, which relates them, but this bound is not shown implied.
    SamePair,
    /// Both columns are in the catalogue, which does not relate them.
    NewPair,
    /// A column is not in the catalogue, or the bound does not have exactly one feature.
    Outside,
}

impl Novelty {
    pub fn label(self) -> &'static str {
        match self {
            Novelty::Implied(_) => "implied",
            Novelty::SamePair => "same_pair_not_implied",
            Novelty::NewPair => "new_pair",
            Novelty::Outside => "outside_catalogue",
        }
    }
}

pub struct NoveltyOracle {
    pub bounds: Vec<CatalogueBound>,
    columns: HashSet<String>,
    signed: HashSet<String>,
    pairs: HashSet<(String, String)>,
}

fn compare(x: Rational, y: Rational) -> Option<Ordering> {
    let ((p, q), (r, s)) = (x.as_ratio()?, y.as_ratio()?);
    Some(p.checked_mul(s)?.cmp(&r.checked_mul(q)?))
}

/// Whether `T op a·F + b` (the candidate) follows from `T op a'·F + b'` (the known bound) for every
/// value of `F`: equal slopes and a constant no tighter; or, when `F` is known to be non-negative, a
/// slope and a constant both no tighter. Sufficient, not necessary: a bound that follows only
/// because of other facts about `F` (such as arc index ≥ 2) is not recognised.
pub fn implies(known: &Bound, candidate: &Bound, feature_non_negative: bool) -> bool {
    let ([(f, a)], [(g, a2)]) = (candidate.terms.as_slice(), known.terms.as_slice()) else {
        return false;
    };
    if candidate.target != known.target || candidate.op != known.op || f != g {
        return false;
    }
    let (Some(slope), Some(constant)) = (
        compare(*a, *a2),
        compare(candidate.constant, known.constant),
    ) else {
        return false;
    };
    let looser = match candidate.op {
        Op::Le => Ordering::Greater,
        Op::Ge => Ordering::Less,
    };
    let no_tighter = |o: Ordering| o == Ordering::Equal || o == looser;
    no_tighter(constant)
        && (slope == Ordering::Equal || (feature_non_negative && no_tighter(slope)))
}

impl NoveltyOracle {
    pub fn new(catalogue: &Catalogue) -> NoveltyOracle {
        let (bounds, _) = catalogue.bounds();
        let pairs = bounds
            .iter()
            .flat_map(|c| {
                let t = c.bound.target.clone();
                c.bound
                    .terms
                    .iter()
                    .flat_map(move |(f, _)| [(t.clone(), f.clone()), (f.clone(), t.clone())])
            })
            .collect();
        NoveltyOracle {
            columns: catalogue.columns().into_iter().map(String::from).collect(),
            signed: catalogue
                .signed_columns()
                .into_iter()
                .map(String::from)
                .collect(),
            pairs,
            bounds,
        }
    }

    /// The hypothesis is ignored: the catalogue holds for every non-trivial knot, so it holds under
    /// any hypothesis.
    pub fn classify(&self, bound: &Bound) -> Novelty {
        let [(feature, _)] = bound.terms.as_slice() else {
            return Novelty::Outside;
        };
        if !self.columns.contains(&bound.target) || !self.columns.contains(feature) {
            return Novelty::Outside;
        }
        let non_negative = !self.signed.contains(feature);
        if let Some(k) = self
            .bounds
            .iter()
            .position(|c| implies(&c.bound, bound, non_negative))
        {
            return Novelty::Implied(k);
        }
        if self
            .pairs
            .contains(&(bound.target.clone(), feature.clone()))
        {
            Novelty::SamePair
        } else {
            Novelty::NewPair
        }
    }
}

#[cfg(test)]
mod tests {
    use ragusa_table::{BooleanBlock, NumericBlock, PackedColumns};

    use super::*;

    fn bound(text_target: &str, op: Op, feature: &str, a: (i128, i128), b: (i128, i128)) -> Bound {
        Bound::new(
            "alternating",
            text_target,
            op,
            [(feature.to_string(), Rational::new(a.0, a.1))],
            Rational::new(b.0, b.1),
        )
    }

    #[test]
    fn gate_uses_interval_rules_and_skips_the_unknot() {
        // (c, u): a confirmed knot, an undecided one (u in [1, 2] against 3/2), the unknot, and a
        // knot that refutes 2u <= c - 1.
        let table = Table::new(
            "knots",
            ["3_1", "4_1", "0_1", "bad"].map(String::from).to_vec(),
            Some(NumericBlock::from_intervals(
                vec!["crossing_number".into(), "unknotting_number".into()],
                vec![vec![3.0, 4.0, 0.0, 3.0], vec![1.0, 1.0, 0.0, 2.0]],
                vec![vec![3.0, 4.0, 0.0, 3.0], vec![1.0, 2.0, 0.0, 2.0]],
            )),
            Some(BooleanBlock::new(
                vec!["alternating".into()],
                PackedColumns::zeros(4, 1),
            )),
            vec![],
            vec![],
        );
        let check = check_catalogue(&table, &Catalogue::standard().unwrap()).unwrap();
        assert_eq!(check.non_trivial_objects, 3);
        let s4 = check
            .bounds
            .iter()
            .position(|c| {
                c.bound.to_string() == "(TRUE) → (unknotting_number <= 1/2·crossing_number + -1/2)"
            })
            .unwrap();
        let c = check.checks[s4].as_ref().unwrap();
        assert_eq!(
            (c.refuted_on.clone(), c.confirmed, c.undecided),
            (vec![3], 1, 1)
        );
        assert!(!check.pass());
        let signature = check
            .bounds
            .iter()
            .position(|c| c.bound.target == "signature")
            .unwrap();
        assert!(check.checks[signature].is_none(), "no signature column");
    }

    #[test]
    fn novelty_classes() {
        let oracle = NoveltyOracle::new(&Catalogue::standard().unwrap());
        let class = |b: Bound| oracle.classify(&b).label();
        let (u, c) = ("unknotting_number", "crossing_number");
        assert_eq!(class(bound(u, Op::Le, c, (1, 2), (0, 1))), "implied");
        assert_eq!(class(bound(u, Op::Le, c, (1, 1), (0, 1))), "implied");
        assert_eq!(
            class(bound(u, Op::Le, c, (1, 3), (1, 1))),
            "same_pair_not_implied"
        );
        let (sigma, g4) = ("signature", "smooth_four_genus");
        assert_eq!(class(bound(sigma, Op::Le, g4, (3, 1), (0, 1))), "implied");
        assert_eq!(
            class(bound(sigma, Op::Le, g4, (1, 1), (1, 1))),
            "same_pair_not_implied"
        );
        // signature can be negative: a smaller slope is not implied
        assert_eq!(class(bound(g4, Op::Ge, sigma, (1, 2), (0, 1))), "implied");
        assert_eq!(
            class(bound(g4, Op::Ge, sigma, (1, 4), (0, 1))),
            "same_pair_not_implied"
        );
        assert_eq!(
            class(bound(
                "double_slice_genus",
                Op::Le,
                "arc_index",
                (1, 1),
                (0, 1)
            )),
            "new_pair"
        );
        assert_eq!(
            class(bound("volume", Op::Le, c, (2, 1), (0, 1))),
            "outside_catalogue"
        );
    }
}
