//! Per-object attribution: which decisions each object is pivotal for.
//!
//! A decision flips when removing one active object changes the candidate's representative
//! status. [`leave_one_out`] measures this by re-running the filter once per object, which does not
//! scale past a few thousand objects; [`attribute`] derives the same answer from each filter's own
//! partition (see [`Filter::attribution`]) and is what the report uses. Only single-object removals
//! are measured.

use rayon::prelude::*;

use crate::{Evaluated, Filter, Partition};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectFloor {
    /// Decisions that flip when object `r` is removed.
    pub flips_by_object: Vec<usize>,
    /// Per input position: flips under at least one single removal.
    pub fragile: Vec<bool>,
}

impl ObjectFloor {
    /// From, per input position, the objects whose removal flips it (each object at most once).
    pub fn from_flips(n_objects: usize, flips: &[Vec<usize>]) -> ObjectFloor {
        let mut flips_by_object = vec![0; n_objects];
        for &r in flips.iter().flatten() {
            flips_by_object[r] += 1;
        }
        ObjectFloor {
            flips_by_object,
            fragile: flips.iter().map(|f| !f.is_empty()).collect(),
        }
    }

    pub fn total_flips(&self) -> usize {
        self.flips_by_object.iter().sum()
    }

    pub fn objects_with_flips(&self) -> usize {
        self.flips_by_object.iter().filter(|&&f| f > 0).count()
    }

    /// Objects ordered by flips, most first, ties by object index; objects with no flips omitted.
    pub fn ranked_objects(&self) -> Vec<(usize, usize)> {
        let mut ranked: Vec<(usize, usize)> = self
            .flips_by_object
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, f)| f > 0)
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        ranked
    }
}

/// Attribution without re-running the filter; `None` when the filter cannot provide it.
pub fn attribute(
    filter: &dyn Filter,
    ev: &Evaluated,
    input: &[usize],
    active: &[bool],
    full: &Partition,
) -> Option<ObjectFloor> {
    filter
        .attribution(ev, input, active, full)
        .map(|flips| ObjectFloor::from_flips(ev.n_objects(), &flips))
}

/// The reference: one re-run of the filter per active object.
pub fn leave_one_out(
    filter: &dyn Filter,
    ev: &Evaluated,
    input: &[usize],
    active: &[bool],
    full: &Partition,
) -> ObjectFloor {
    ObjectFloor::from_flips(
        ev.n_objects(),
        &leave_one_out_flips(filter, ev, input, active, full),
    )
}

/// Per input position, the active objects whose removal flips it, by one re-run per object; for
/// filters without a closed-form attribution.
pub fn leave_one_out_flips(
    filter: &dyn Filter,
    ev: &Evaluated,
    input: &[usize],
    active: &[bool],
    full: &Partition,
) -> Vec<Vec<usize>> {
    let kept: Vec<bool> = full.classes.iter().map(|c| c.is_representative()).collect();
    let per_object: Vec<Vec<usize>> = (0..ev.n_objects())
        .into_par_iter()
        .map(|r| {
            if !active[r] {
                return Vec::new();
            }
            let mut without = active.to_vec();
            without[r] = false;
            let p = filter.partition(ev, input, &without);
            p.classes
                .iter()
                .zip(&kept)
                .enumerate()
                .filter(|(_, (c, &k))| c.is_representative() != k)
                .map(|(pos, _)| pos)
                .collect()
        })
        .collect();
    let mut flips = vec![Vec::new(); input.len()];
    for (r, positions) in per_object.iter().enumerate() {
        for &pos in positions {
            flips[pos].push(r);
        }
    }
    flips
}

#[cfg(test)]
mod tests {
    use ragusa_gen::{Bound, Op, Rational};
    use ragusa_table::{BooleanBlock, NumericBlock, PackedColumns, Table};
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::filters::tests::tiny;
    use crate::filters::{
        Generality, GreedyTouch, MaximalTouch, SharpCoverage, TableDuplicates, Touch,
        TouchThreshold, Validity,
    };

    #[test]
    fn removing_the_only_counterexample_flips_validity() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        let active = [true; 4];
        let input = [0, 1, 2, 5, 6];
        let full = Validity.partition(&ev, &input, &active);
        let floor = leave_one_out(&Validity, &ev, &input, &active, &full);
        // candidate 6 is violated only at object 2; candidate 2 has three counterexamples
        assert_eq!(floor.flips_by_object, vec![0, 0, 1, 0]);
        assert_eq!(floor.fragile, vec![false, false, false, false, true]);
        assert_eq!(floor.ranked_objects(), vec![(2, 1)]);
        assert_eq!(
            attribute(&Validity, &ev, &input, &active, &full),
            Some(floor)
        );
    }

    /// A small random table with many ties, duplicates, nested hypotheses, intervals and unknowns,
    /// so that every kind of flip occurs.
    fn random_case(seed: u64) -> (Table, Vec<Bound>, Vec<bool>) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let n = rng.random_range(1..9);
        let mut lower = vec![Vec::new(), Vec::new()];
        let mut upper = vec![Vec::new(), Vec::new()];
        for c in 0..2 {
            for _ in 0..n {
                let lo = f64::from(rng.random_range(0..4));
                let hi = if rng.random_bool(0.15) { lo + 1.0 } else { lo };
                lower[c].push(lo);
                upper[c].push(hi);
            }
        }
        let n_h = 3;
        let mut bits = PackedColumns::zeros(n, n_h);
        let mut known = PackedColumns::zeros(n, n_h);
        for r in 0..n {
            for h in 0..n_h {
                let is_known = !rng.random_bool(0.1);
                if is_known {
                    known.set(r, h);
                    if rng.random_bool(0.7) {
                        bits.set(r, h);
                    }
                }
            }
        }
        let table = Table::new(
            "random",
            (0..n).map(|r| r.to_string()).collect(),
            Some(NumericBlock::from_intervals(
                vec!["x".into(), "y".into()],
                lower,
                upper,
            )),
            Some(BooleanBlock::with_known(
                vec!["h0".into(), "h1".into(), "h2".into()],
                bits,
                known,
            )),
            vec![],
            vec![],
        );
        let bounds = (0..rng.random_range(1..25))
            .map(|_| {
                let op = if rng.random_bool(0.5) { Op::Le } else { Op::Ge };
                let h = format!("h{}", rng.random_range(0..n_h));
                let c = Rational::new(rng.random_range(0..3), 1);
                let k = Rational::new(rng.random_range(-1..3), 1);
                Bound::new(&h, "y", op, [("x".to_string(), c)], k)
            })
            .collect();
        let active = (0..n).map(|_| !rng.random_bool(0.1)).collect();
        (table, bounds, active)
    }

    #[test]
    fn attribution_equals_leave_one_out_on_random_tables() {
        let filters: [&dyn Filter; 8] = [
            &Validity,
            &TableDuplicates,
            &Touch {
                threshold: TouchThreshold::Count(2),
            },
            &SharpCoverage,
            &Generality,
            &MaximalTouch,
            &GreedyTouch,
            &Touch {
                threshold: TouchThreshold::SqrtApplicable,
            },
        ];
        // Per filter: fragile representatives, fragile non-representatives.
        let mut seen = [[0usize; 2]; 8];
        for seed in 0..3000 {
            let (table, bounds, active) = random_case(seed);
            let ev = Evaluated::new(&table, bounds).unwrap();
            // Every filter on the full candidate list, and on the pipeline's own inputs.
            let all: Vec<usize> = (0..ev.len()).collect();
            let mut pipeline = all.clone();
            for (f, filter) in filters.into_iter().enumerate() {
                for input in [&all, &pipeline] {
                    let full = filter.partition(&ev, input, &active);
                    let rerun = leave_one_out(filter, &ev, input, &active, &full);
                    for (fragile, class) in rerun.fragile.iter().zip(&full.classes) {
                        if *fragile {
                            seen[f][usize::from(!class.is_representative())] += 1;
                        }
                    }
                    assert_eq!(
                        attribute(filter, &ev, input, &active, &full),
                        Some(rerun),
                        "{} seed {seed} input {input:?}",
                        filter.name()
                    );
                }
                pipeline = filter.partition(&ev, &pipeline, &active).representatives();
            }
        }
        // The cases must exercise every kind of flip each rule admits.
        assert!(seen[0][1] > 0, "validity: discarded candidates that flip");
        assert!(
            seen[1][0] > 0,
            "table_duplicates: representatives that flip"
        );
        assert!(
            seen[2][0] > 0 && seen[2][1] == 0,
            "touch: representatives flip, discarded never {:?}",
            seen[2]
        );
        assert!(seen[3][0] > 0, "sharp_coverage: representatives that flip");
        assert!(
            seen[4][0] > 0 && seen[4][1] > 0,
            "generality: both directions {:?}",
            seen[4]
        );
        assert!(
            seen[5][0] > 0 && seen[5][1] > 0,
            "maximal_touch: both directions {:?}",
            seen[5]
        );
        assert!(
            seen[6][0] > 0 && seen[6][1] > 0,
            "greedy_touch: both directions {:?}",
            seen[6]
        );
        assert!(
            seen[7][0] > 0 && seen[7][1] > 0,
            "touch sqrt: both directions {:?}",
            seen[7]
        );
    }
}
