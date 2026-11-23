//! Chance baseline: what a rule that accepts the same number of candidates uniformly at random
//! would keep. Seeded ChaCha8, so a report is reproducible on any platform.

use rand::seq::index::sample;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::filters::table_classes;
use crate::Evaluated;

pub const METRICS: [&str; 4] = [
    "tight_objects",
    "false_kept",
    "hypotheses_kept",
    "distinct_table_classes",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    pub name: &'static str,
    pub rule: usize,
    pub random_mean: f64,
    /// Nearest-rank 2.5th percentile over the draws.
    pub random_low: usize,
    /// Nearest-rank 97.5th percentile over the draws.
    pub random_high: usize,
    /// Share of draws that match or exceed the rule.
    pub frac_random_at_least_rule: f64,
}

/// `kept` are positions in `input` that the rule accepted. Returns no metrics when `draws` is 0.
pub fn baseline(
    ev: &Evaluated,
    input: &[usize],
    kept: &[usize],
    active: &[bool],
    draws: usize,
    seed: u64,
) -> Vec<Metric> {
    if draws == 0 {
        return Vec::new();
    }
    let n = ev.n_objects();
    let words = n.div_ceil(64);
    // Tight objects are few per candidate, so they are kept as lists, not n-bit masks.
    let tight: Vec<Vec<usize>> = input
        .iter()
        .map(|&i| (0..n).filter(|&r| active[r] && ev.tight(i, r)).collect())
        .collect();
    let is_false: Vec<bool> = input
        .iter()
        .map(|&i| (0..n).any(|r| active[r] && ev.violated(i, r)))
        .collect();
    let classes = table_classes(ev, input, active);
    let n_classes = classes.iter().max().map_or(0, |m| m + 1);
    let mut hypothesis_ids = std::collections::HashMap::new();
    let hypotheses: Vec<usize> = input
        .iter()
        .map(|&i| {
            let next = hypothesis_ids.len();
            *hypothesis_ids
                .entry(ev.bound(i).hypothesis.as_str())
                .or_insert(next)
        })
        .collect();
    let n_hypotheses = hypothesis_ids.len();

    let measure = |positions: &[usize]| -> [usize; 4] {
        let mut covered = vec![0u64; words];
        let mut seen_h = vec![false; n_hypotheses];
        let mut seen_c = vec![false; n_classes];
        let mut false_kept = 0;
        for &p in positions {
            for &r in &tight[p] {
                covered[r >> 6] |= 1 << (r & 63);
            }
            seen_h[hypotheses[p]] = true;
            seen_c[classes[p]] = true;
            false_kept += usize::from(is_false[p]);
        }
        [
            covered.iter().map(|w| w.count_ones() as usize).sum(),
            false_kept,
            seen_h.iter().filter(|&&s| s).count(),
            seen_c.iter().filter(|&&s| s).count(),
        ]
    };

    let rule = measure(kept);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut samples: Vec<[usize; 4]> = Vec::with_capacity(draws);
    for _ in 0..draws {
        let positions = sample(&mut rng, input.len(), kept.len()).into_vec();
        samples.push(measure(&positions));
    }

    METRICS
        .iter()
        .enumerate()
        .map(|(m, &name)| {
            let mut values: Vec<usize> = samples.iter().map(|s| s[m]).collect();
            values.sort_unstable();
            let rank = |q: f64| values[((q * draws as f64).ceil() as usize).clamp(1, draws) - 1];
            Metric {
                name,
                rule: rule[m],
                random_mean: values.iter().sum::<usize>() as f64 / draws as f64,
                random_low: rank(0.025),
                random_high: rank(0.975),
                frac_random_at_least_rule: values.iter().filter(|&&v| v >= rule[m]).count() as f64
                    / draws as f64,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::tests::tiny;

    #[test]
    fn same_seed_same_baseline() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        let input: Vec<usize> = (0..ev.len()).collect();
        let active = [true; 4];
        let a = baseline(&ev, &input, &[0, 5], &active, 200, 7);
        let b = baseline(&ev, &input, &[0, 5], &active, 200, 7);
        assert_eq!(a, b);
        let false_kept = a.iter().find(|m| m.name == "false_kept").unwrap();
        assert_eq!(false_kept.rule, 0);
        assert!(
            false_kept.random_mean > 0.0,
            "two of seven candidates are false"
        );
    }
}
