//! ragusa's own acceptance rules. They are inspired by the Graffiti lineage and cited as such,
//! but they are not txgraffiti's rules and are not claimed to equal them: all four are
//! order-independent, and comparisons are made only between bounds of the same direction.
//!
//! Each rule also states, in [`Filter::attribution`], which single-object removals flip its
//! decisions, derived from its own partition; `floor::tests` checks these against re-runs.

use std::collections::HashMap;

use ragusa_gen::Op;
use rayon::prelude::*;

use crate::{Class, Evaluated, Filter, Kind, Partition};

fn active_objects(active: &[bool]) -> Vec<usize> {
    (0..active.len()).filter(|&r| active[r]).collect()
}

/// Discards every candidate violated on some active object; the counterexamples are its witnesses.
///
/// Attribution: removing an object never adds a counterexample, so only discarded candidates can
/// flip, and one does exactly when its counterexample is unique.
pub struct Validity;

impl Filter for Validity {
    fn name(&self) -> &'static str {
        "validity"
    }

    fn kind(&self) -> Kind {
        Kind::Validity
    }

    fn partition(&self, ev: &Evaluated, input: &[usize], active: &[bool]) -> Partition {
        let mut classes = Vec::with_capacity(input.len());
        let mut witnesses = Vec::with_capacity(input.len());
        for &i in input {
            let counterexamples: Vec<usize> = (0..ev.n_objects())
                .filter(|&r| active[r] && ev.violated(i, r))
                .collect();
            classes.push(if counterexamples.is_empty() {
                Class::Representative
            } else {
                Class::Discarded { by: None }
            });
            witnesses.push(counterexamples);
        }
        Partition::new(input.to_vec(), classes, witnesses)
    }

    fn attribution(
        &self,
        _ev: &Evaluated,
        _input: &[usize],
        _active: &[bool],
        full: &Partition,
    ) -> Option<Vec<Vec<usize>>> {
        Some(unique_witness(full, |c| !c.is_representative()))
    }
}

/// For each position whose class satisfies `pred`, its witness if it has exactly one.
fn unique_witness(full: &Partition, pred: impl Fn(Class) -> bool) -> Vec<Vec<usize>> {
    full.classes
        .iter()
        .zip(&full.witnesses)
        .map(|(&c, w)| {
            if pred(c) && w.len() == 1 {
                w.clone()
            } else {
                Vec::new()
            }
        })
        .collect()
}

/// Marker for "hypothesis does not hold here" inside a table key.
const ABSENT: u64 = u64::MAX;

fn key_bits(v: f64) -> u64 {
    (if v == 0.0 { 0.0 } else { v }).to_bits()
}

/// Candidate `i`'s entry on object `r` in its table key: the right-hand side's lower and upper bits
/// where the hypothesis holds, [`ABSENT`] twice where it does not.
fn component(ev: &Evaluated, i: usize, r: usize) -> (u64, u64) {
    if ev.applies(i, r) {
        let (lo, hi) = ev.rhs_bounds(i, r);
        (key_bits(lo), key_bits(hi))
    } else {
        (ABSENT, ABSENT)
    }
}

/// Order-dependent 64-bit mix; used only to bucket keys, never to decide equality.
fn mix(h: u64, (lo, hi): (u64, u64)) -> u64 {
    let h = (h.rotate_left(27) ^ lo).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (h.rotate_left(27) ^ hi).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn key_hash(ev: &Evaluated, i: usize, objects: &[usize]) -> u64 {
    objects
        .iter()
        .fold(0xCBF2_9CE4_8422_2325, |h, &r| mix(h, component(ev, i, r)))
}

/// Candidates `i` and `j` state the same thing on `objects` (target and operator not compared).
fn same_values(ev: &Evaluated, i: usize, j: usize, objects: &[usize]) -> bool {
    objects
        .iter()
        .all(|&r| component(ev, i, r) == component(ev, j, r))
}

/// Classes whose keys share a hash: (class index, first candidate of the class).
type Bucket = Vec<(usize, usize)>;

/// For each input position, the index of its table class, numbered in order of first appearance.
/// A class is target, operator and the table key over the active objects. Keys are hashed to find
/// candidates for equality and then compared exactly, so no key is ever stored.
pub fn table_classes(ev: &Evaluated, input: &[usize], active: &[bool]) -> Vec<usize> {
    let objects = active_objects(active);
    let hashes: Vec<u64> = input
        .par_iter()
        .map(|&i| key_hash(ev, i, &objects))
        .collect();
    let mut buckets: HashMap<(&str, Op, u64), Bucket> = HashMap::new();
    let mut next = 0;
    input
        .iter()
        .zip(&hashes)
        .map(|(&i, &h)| {
            let b = ev.bound(i);
            let bucket = buckets.entry((b.target.as_str(), b.op, h)).or_default();
            match bucket
                .iter()
                .find(|&&(_, j)| same_values(ev, i, j, &objects))
            {
                Some(&(id, _)) => id,
                None => {
                    bucket.push((next, i));
                    next += 1;
                    next - 1
                }
            }
        })
        .collect()
}

/// Aliases candidates that state the same thing on the table: same target and operator, the same
/// active objects under the hypothesis, and bit-identical right-hand sides there (both ends of the
/// interval). The first in input order is the representative.
///
/// Attribution: removing an object deletes one entry from every key, so equal keys stay equal and
/// no alias flips. A representative becomes an alias exactly when some earlier representative with
/// the same target and operator differs from it on that one object only. Finding those pairs is
/// quadratic in the representatives of a group; above [`TableDuplicates::PAIR_BUDGET`] pairs the
/// attribution is not computed.
pub struct TableDuplicates;

impl TableDuplicates {
    pub const PAIR_BUDGET: u64 = 1_000_000_000;
    /// Keys are compared block by block through block hashes; a pair whose hashes differ in two
    /// blocks differs on at least two objects and is dismissed without a scan.
    const BLOCKS: usize = 64;
}

impl Filter for TableDuplicates {
    fn name(&self) -> &'static str {
        "table_duplicates"
    }

    fn kind(&self) -> Kind {
        Kind::Redundancy
    }

    fn partition(&self, ev: &Evaluated, input: &[usize], active: &[bool]) -> Partition {
        let ids = table_classes(ev, input, active);
        let mut first: HashMap<usize, usize> = HashMap::new();
        let classes = input
            .iter()
            .zip(&ids)
            .map(|(&i, &id)| match first.get(&id) {
                Some(&of) => Class::Alias { of },
                None => {
                    first.insert(id, i);
                    Class::Representative
                }
            })
            .collect();
        Partition::new(input.to_vec(), classes, vec![Vec::new(); input.len()])
    }

    fn attribution(
        &self,
        ev: &Evaluated,
        input: &[usize],
        active: &[bool],
        full: &Partition,
    ) -> Option<Vec<Vec<usize>>> {
        let mut groups: HashMap<(&str, Op), Vec<usize>> = HashMap::new();
        for (pos, &i) in input.iter().enumerate() {
            if full.classes[pos].is_representative() {
                let b = ev.bound(i);
                groups
                    .entry((b.target.as_str(), b.op))
                    .or_default()
                    .push(pos);
            }
        }
        let pairs: u64 = groups
            .values()
            .map(|g| (g.len() as u64) * (g.len() as u64).saturating_sub(1) / 2)
            .sum();
        if pairs > Self::PAIR_BUDGET {
            return None;
        }

        let objects = active_objects(active);
        let block = objects.len().div_ceil(Self::BLOCKS).max(1);
        let mut flips = vec![Vec::new(); input.len()];
        for group in groups.values().filter(|g| g.len() > 1) {
            let hashes: Vec<Vec<u64>> = group
                .par_iter()
                .map(|&pos| {
                    objects
                        .chunks(block)
                        .map(|chunk| key_hash(ev, input[pos], chunk))
                        .collect()
                })
                .collect();
            let found: Vec<(usize, Vec<usize>)> = (1..group.len())
                .into_par_iter()
                .map(|k| {
                    let mut objs: Vec<usize> = (0..k)
                        .filter_map(|q| {
                            single_difference(
                                ev,
                                input[group[k]],
                                input[group[q]],
                                &objects,
                                block,
                                &hashes[k],
                                &hashes[q],
                            )
                        })
                        .collect();
                    objs.sort_unstable();
                    objs.dedup();
                    (group[k], objs)
                })
                .collect();
            for (pos, objs) in found {
                flips[pos] = objs;
            }
        }
        Some(flips)
    }
}

/// The one object on which the keys of `i` and `j` differ, if they differ on exactly one.
fn single_difference(
    ev: &Evaluated,
    i: usize,
    j: usize,
    objects: &[usize],
    block: usize,
    hi: &[u64],
    hj: &[u64],
) -> Option<usize> {
    let mut differing = hi
        .iter()
        .zip(hj)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(k, _)| k);
    let first = differing.next();
    if differing.next().is_some() {
        return None;
    }
    let range = match first {
        Some(k) => k * block..((k + 1) * block).min(objects.len()),
        None => 0..objects.len(),
    };
    let mut found = None;
    for &r in &objects[range.clone()] {
        if component(ev, i, r) != component(ev, j, r) {
            if found.is_some() {
                return None;
            }
            found = Some(r);
        }
    }
    let r = found?;
    // Equal block hashes do not prove equal blocks; check the rest exactly.
    let rest_equal = same_values(ev, i, j, &objects[..range.start])
        && same_values(ev, i, j, &objects[range.end..]);
    rest_equal.then_some(r)
}

/// A candidate is representative iff on at least one active object where its hypothesis holds and
/// it is not violated, it is strictly tighter, by more than the tolerance, than every other such
/// input candidate with the same target and operator. Those objects are its witnesses.
///
/// With interval values "strictly tighter" means for every value: an upper bound wins when its
/// largest right-hand side is below every other candidate's smallest; a lower bound wins when its
/// smallest is above every other's largest. On exact values this is the plain comparison.
///
/// In the spirit of Fajtlowicz's Dalmatian heuristic as formalised by Larson and Van Cleemput
/// (Artificial Intelligence 231, 2016), not equal to it or to txgraffiti's `dalmatian_accept`: it
/// does not depend on input order, and a lower bound is compared only with lower bounds.
///
/// Attribution: whether a candidate wins on an object depends on that object alone, so removing an
/// object removes only its own witnesses. A representative flips exactly when its witness is
/// unique; a discarded candidate never flips.
pub struct SharpCoverage;

/// Per comparison group on one object.
#[derive(Clone, Copy, Default)]
struct Leader {
    /// The candidate whose own end is tightest: (input position, its upper end for `<=`, its
    /// lower end for `>=`).
    best: Option<(usize, f64)>,
    /// The most competitive opposite end among all candidates, with its position.
    first: Option<(usize, f64)>,
    /// The next most competitive opposite end.
    second: Option<f64>,
}

impl Filter for SharpCoverage {
    fn name(&self) -> &'static str {
        "sharp_coverage"
    }

    fn kind(&self) -> Kind {
        Kind::Redundancy
    }

    fn partition(&self, ev: &Evaluated, input: &[usize], active: &[bool]) -> Partition {
        let mut group_of: HashMap<(&str, Op), usize> = HashMap::new();
        let mut group_ops = Vec::new();
        let groups: Vec<usize> = input
            .iter()
            .map(|&i| {
                let b = ev.bound(i);
                *group_of
                    .entry((b.target.as_str(), b.op))
                    .or_insert_with(|| {
                        group_ops.push(b.op);
                        group_ops.len() - 1
                    })
            })
            .collect();

        let mut witnesses = vec![Vec::new(); input.len()];
        let mut leaders = vec![Leader::default(); group_ops.len()];
        for r in (0..ev.n_objects()).filter(|&r| active[r]) {
            leaders.iter_mut().for_each(|l| *l = Leader::default());
            for (pos, &i) in input.iter().enumerate() {
                if !ev.applies(i, r) || ev.violated(i, r) {
                    continue;
                }
                let (lo, hi) = ev.rhs_bounds(i, r);
                if lo.is_nan() || hi.is_nan() {
                    continue;
                }
                let g = groups[pos];
                let op = group_ops[g];
                let tighter = |a: f64, b: f64| match op {
                    Op::Le => a < b,
                    Op::Ge => a > b,
                };
                let (own, opposite) = match op {
                    Op::Le => (hi, lo),
                    Op::Ge => (lo, hi),
                };
                let leader = &mut leaders[g];
                if leader.best.is_none_or(|(_, b)| tighter(own, b)) {
                    leader.best = Some((pos, own));
                }
                match leader.first {
                    None => leader.first = Some((pos, opposite)),
                    Some((_, f)) if tighter(opposite, f) => {
                        leader.second = Some(f);
                        leader.first = Some((pos, opposite));
                    }
                    _ => {
                        if leader.second.is_none_or(|s| tighter(opposite, s)) {
                            leader.second = Some(opposite);
                        }
                    }
                }
            }
            for (g, leader) in leaders.iter().enumerate() {
                let Some((pos, own)) = leader.best else {
                    continue;
                };
                let others = match leader.first {
                    Some((p, _)) if p == pos => leader.second,
                    Some((_, f)) => Some(f),
                    None => None,
                };
                let margin = |other: f64| match group_ops[g] {
                    Op::Le => other - own,
                    Op::Ge => own - other,
                };
                if others.is_none_or(|o| margin(o) > ev.tolerance(input[pos], r)) {
                    witnesses[pos].push(r);
                }
            }
        }
        let classes = witnesses
            .iter()
            .map(|w| {
                if w.is_empty() {
                    Class::Discarded { by: None }
                } else {
                    Class::Representative
                }
            })
            .collect();
        Partition::new(input.to_vec(), classes, witnesses)
    }

    fn attribution(
        &self,
        _ev: &Evaluated,
        _input: &[usize],
        _active: &[bool],
        full: &Partition,
    ) -> Option<Vec<Vec<usize>>> {
        Some(unique_witness(full, Class::is_representative))
    }
}

/// How many tight objects the touch stage asks of a candidate whose hypothesis holds on `a` active
/// objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchThreshold {
    /// A fixed count.
    Count(usize),
    /// `⌈√a⌉`, the smallest `t` with `t² ≥ a`.
    SqrtApplicable,
}

impl TouchThreshold {
    /// `K` (a count) or `sqrt`.
    pub fn parse(text: &str) -> Option<TouchThreshold> {
        match text {
            "sqrt" => Some(TouchThreshold::SqrtApplicable),
            _ => text.parse().ok().map(TouchThreshold::Count),
        }
    }

    pub fn describe(self) -> String {
        match self {
            TouchThreshold::Count(k) => k.to_string(),
            TouchThreshold::SqrtApplicable => "sqrt".to_string(),
        }
    }

    pub fn needed(self, applicable: usize) -> usize {
        match self {
            TouchThreshold::Count(k) => k,
            TouchThreshold::SqrtApplicable => {
                let mut t = (applicable as f64).sqrt() as usize;
                while t * t < applicable {
                    t += 1;
                }
                while t > 0 && (t - 1) * (t - 1) >= applicable {
                    t -= 1;
                }
                t
            }
        }
    }
}

/// Discards a candidate tight on fewer active objects (attained within the tolerance, every value
/// exact) than its threshold asks, given the active objects where its hypothesis holds. The tight
/// objects are the witnesses, listed up to one more than needed.
///
/// The touch number of the Graffiti lineage, used as a threshold rather than a ranking. Held-out
/// rule studies (CONTROLS.md, Sprints 9 and 10) found bounds tight on at least 10 training objects,
/// and more so on at least `⌈√applicable⌉`, refuted on held-out objects far less often than the pool
/// they were drawn from, on `graph_data` and KnotInfo, in seed ranges not used to choose the rule.
///
/// Attribution, with `t` tight and `a` applicable active objects and `need(a)` the threshold:
/// removing a tight object gives `(t − 1, a − 1)`, removing an applicable object that is not tight
/// gives `(t, a − 1)`, removing any other object changes nothing. Since `need` never grows when `a`
/// shrinks, a representative can only flip by losing a tight object (when `t − 1 < need(a − 1)`),
/// and a discarded candidate only by losing a non-tight applicable object (when
/// `t ≥ need(a − 1)`, possible for `⌈√a⌉` when `a − 1` is a perfect square).
pub struct Touch {
    pub threshold: TouchThreshold,
}

impl Touch {
    fn applicable(ev: &Evaluated, i: usize, active: &[bool]) -> usize {
        (0..ev.n_objects())
            .filter(|&r| active[r] && ev.applies(i, r))
            .count()
    }
}

impl Filter for Touch {
    fn name(&self) -> &'static str {
        "touch"
    }

    fn kind(&self) -> Kind {
        Kind::Support
    }

    fn partition(&self, ev: &Evaluated, input: &[usize], active: &[bool]) -> Partition {
        let (classes, witnesses): (Vec<Class>, Vec<Vec<usize>>) = input
            .par_iter()
            .map(|&i| {
                let need = self.threshold.needed(Touch::applicable(ev, i, active));
                let tight: Vec<usize> = (0..ev.n_objects())
                    .filter(|&r| active[r] && ev.tight(i, r))
                    .take(need + 1)
                    .collect();
                let class = if tight.len() >= need {
                    Class::Representative
                } else {
                    Class::Discarded { by: None }
                };
                (class, tight)
            })
            .unzip();
        Partition::new(input.to_vec(), classes, witnesses)
    }

    fn attribution(
        &self,
        ev: &Evaluated,
        input: &[usize],
        active: &[bool],
        full: &Partition,
    ) -> Option<Vec<Vec<usize>>> {
        Some(
            (0..input.len())
                .into_par_iter()
                .map(|pos| {
                    let i = input[pos];
                    let a = Touch::applicable(ev, i, active);
                    let need = self.threshold.needed(a);
                    let w = &full.witnesses[pos];
                    // More tight objects than needed (the list is cut there): no single removal
                    // brings the count below a threshold that never grows.
                    if w.len() > need {
                        return Vec::new();
                    }
                    let t = w.len();
                    if full.classes[pos].is_representative() {
                        if t >= 1 && t - 1 < self.threshold.needed(a - 1) {
                            w.clone()
                        } else {
                            Vec::new()
                        }
                    } else if a > t && t >= self.threshold.needed(a - 1) {
                        (0..ev.n_objects())
                            .filter(|&r| active[r] && ev.applies(i, r) && !ev.tight(i, r))
                            .collect()
                    } else {
                        Vec::new()
                    }
                })
                .collect(),
        )
    }
}

/// Discards a candidate when another input candidate states the same conclusion (target, operator,
/// terms and constant) under a hypothesis that holds on a strict superset of its active objects.
/// The witnesses are the objects only the more general candidate covers; the lowest such
/// candidate index is recorded as `by`.
///
/// In the spirit of txgraffiti's `morgan_accept` (Davila), not equal to it: it does not depend on
/// input order, and equal hypothesis sets leave both candidates in place (TableDuplicates handles
/// those).
///
/// Attribution is exact, by the set argument of [`strictly_inside_attribution`] applied to the
/// objects where each hypothesis holds.
pub struct Generality;

/// One set of active objects per input position, as packed bits over the active objects, with the
/// positions split into groups; only groups of two or more are kept, since a lone set is never
/// strictly inside another.
struct SetFamily {
    objects: Vec<usize>,
    groups: Vec<Vec<usize>>,
    bits: HashMap<usize, Vec<u64>>,
}

/// `key(i)` groups candidate `i`; `member(i, r)` says whether object `r` is in its set. The set of
/// a candidate on an object must not depend on any other object, so that removing an object only
/// removes it from every set.
fn set_family<K: std::hash::Hash + Eq>(
    input: &[usize],
    active: &[bool],
    key: impl Fn(usize) -> K,
    member: impl Fn(usize, usize) -> bool + Sync,
) -> SetFamily {
    let objects = active_objects(active);
    let mut by_key: HashMap<K, Vec<usize>> = HashMap::new();
    for (pos, &i) in input.iter().enumerate() {
        by_key.entry(key(i)).or_default().push(pos);
    }
    let groups: Vec<Vec<usize>> = by_key.into_values().filter(|m| m.len() > 1).collect();
    let bits = groups
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|pos| {
            let mut words = vec![0u64; objects.len().div_ceil(64)];
            for (k, &r) in objects.iter().enumerate() {
                if member(input[pos], r) {
                    words[k >> 6] |= 1 << (k & 63);
                }
            }
            (pos, words)
        })
        .collect();
    SetFamily {
        objects,
        groups,
        bits,
    }
}

/// Discards every position whose set is strictly inside another set of its group; `by` is the
/// lowest such candidate index and the witnesses are the objects only that candidate's set holds.
fn strictly_inside_partition(f: &SetFamily, input: &[usize]) -> Partition {
    let mut classes = vec![Class::Representative; input.len()];
    let mut witnesses = vec![Vec::new(); input.len()];
    for members in &f.groups {
        for &a in members {
            let wider = members
                .iter()
                .filter(|&&b| {
                    difference(&f.bits[&a], &f.bits[&b]).0 == 0
                        && difference(&f.bits[&b], &f.bits[&a]).0 > 0
                })
                .min_by_key(|&&b| input[b]);
            if let Some(&b) = wider {
                classes[a] = Class::Discarded { by: Some(input[b]) };
                witnesses[a] = f
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|&(k, _)| {
                        let bit = |p: usize| (f.bits[&p][k >> 6] >> (k & 63)) & 1 == 1;
                        bit(b) && !bit(a)
                    })
                    .map(|(_, &r)| r)
                    .collect();
            }
        }
    }
    Partition::new(input.to_vec(), classes, witnesses)
}

/// Attribution of [`strictly_inside_partition`], with `A \ B` the objects in `A`'s set and not in
/// `B`'s: removing `r` puts `A` strictly inside `B` exactly when `A \ B = {r}` and `B \ A` is
/// non-empty, and takes it out exactly when `A \ B` is empty and `B \ A = {r}`. So a kept position
/// flips at every such `r` of the first kind; a discarded one flips at `r` when all its current
/// dominators are of the second kind for that same `r`. (No candidate of the first kind can then
/// block the rescue: the first kind needs `r` in `A`, the second needs `r` outside it.)
fn strictly_inside_attribution(f: &SetFamily, n_input: usize) -> Vec<Vec<usize>> {
    let mut flips = vec![Vec::new(); n_input];
    for members in &f.groups {
        for &a in members {
            let mut dominated = false;
            // Some(k): every dominator so far loses its dominance when object k goes.
            let mut rescued_by: Option<usize> = None;
            let mut rescuable = true;
            let mut new_dominance = Vec::new();
            for &b in members.iter().filter(|&&b| b != a) {
                let (b_only, b_elem) = difference(&f.bits[&b], &f.bits[&a]);
                if b_only == 0 {
                    continue;
                }
                let (a_only, a_elem) = difference(&f.bits[&a], &f.bits[&b]);
                match a_only {
                    0 => {
                        dominated = true;
                        match (b_only, rescued_by) {
                            (1, None) => rescued_by = Some(b_elem),
                            (1, Some(k)) if k == b_elem => {}
                            _ => rescuable = false,
                        }
                    }
                    1 => new_dominance.push(a_elem),
                    _ => {}
                }
            }
            flips[a] = if dominated {
                match rescued_by {
                    Some(k) if rescuable => vec![f.objects[k]],
                    _ => Vec::new(),
                }
            } else {
                new_dominance.sort_unstable();
                new_dominance.dedup();
                new_dominance.into_iter().map(|k| f.objects[k]).collect()
            };
        }
    }
    flips
}

/// Candidates grouped by conclusion, with the objects where their hypothesis holds.
fn conclusion_family(ev: &Evaluated, input: &[usize], active: &[bool]) -> SetFamily {
    set_family(
        input,
        active,
        |i| {
            let b = ev.bound(i);
            (b.target.as_str(), b.op, b.terms.as_slice(), b.constant)
        },
        |i, r| ev.applies(i, r),
    )
}

/// Candidates grouped by target and operator, with the objects where they are tight.
fn tight_family(ev: &Evaluated, input: &[usize], active: &[bool]) -> SetFamily {
    set_family(
        input,
        active,
        |i| {
            let b = ev.bound(i);
            (b.target.as_str(), b.op)
        },
        |i, r| ev.tight(i, r),
    )
}

/// `|x \ y|` capped at 2, with the index of one element when it is exactly 1.
fn difference(x: &[u64], y: &[u64]) -> (u32, usize) {
    let (mut count, mut last) = (0, 0);
    for (w, (&a, &b)) in x.iter().zip(y).enumerate() {
        let d = a & !b;
        if d != 0 {
            count += d.count_ones();
            last = w * 64 + d.trailing_zeros() as usize;
            if count > 1 {
                return (2, 0);
            }
        }
    }
    (count, last)
}

impl Filter for Generality {
    fn name(&self) -> &'static str {
        "generality"
    }

    fn kind(&self) -> Kind {
        Kind::Redundancy
    }

    fn partition(&self, ev: &Evaluated, input: &[usize], active: &[bool]) -> Partition {
        strictly_inside_partition(&conclusion_family(ev, input, active), input)
    }

    fn attribution(
        &self,
        ev: &Evaluated,
        input: &[usize],
        active: &[bool],
        _full: &Partition,
    ) -> Option<Vec<Vec<usize>>> {
        Some(strictly_inside_attribution(
            &conclusion_family(ev, input, active),
            input.len(),
        ))
    }
}

/// Discards a candidate whose tight objects are a strict subset of those of another input
/// candidate with the same target and operator; the witnesses are the objects only that other
/// candidate is tight on, and `by` is the lowest such candidate index.
///
/// A replacement for sharp coverage (Sprint 10). Sharp coverage keeps a bound for being strictly
/// best on some object, which Sprint 8 and 9 found selects bounds that fail on unseen objects. Here
/// a bound is kept unless another bound is attained wherever it is and somewhere else too, so
/// bounds attained often are favoured and being extreme on one object is not rewarded. Bounds with
/// equal tight sets are all kept. Order-independent; attribution is exact, by the set argument of
/// [`strictly_inside_attribution`].
pub struct MaximalTouch;

impl Filter for MaximalTouch {
    fn name(&self) -> &'static str {
        "maximal_touch"
    }

    fn kind(&self) -> Kind {
        Kind::Redundancy
    }

    fn partition(&self, ev: &Evaluated, input: &[usize], active: &[bool]) -> Partition {
        strictly_inside_partition(&tight_family(ev, input, active), input)
    }

    fn attribution(
        &self,
        ev: &Evaluated,
        input: &[usize],
        active: &[bool],
        _full: &Partition,
    ) -> Option<Vec<Vec<usize>>> {
        Some(strictly_inside_attribution(
            &tight_family(ev, input, active),
            input.len(),
        ))
    }
}

/// Within each target and operator, repeatedly keeps the candidate tight on the most objects not
/// yet covered by the candidates kept before it (ties: lowest input position), until no candidate
/// adds a tight object. A kept candidate's witnesses are the objects it added; the rest are
/// discarded.
///
/// A replacement for sharp coverage (Sprint 10): a greedy cover of tight objects, close to the
/// intent of Fajtlowicz's Dalmatian (keep what speaks for objects nothing kept already speaks for)
/// but counting attainment instead of strict tightness. Order matters only through ties.
///
/// Attribution. Removing an object `r` removes it from every tight set, which is the same as
/// starting the greedy cover with `r` already covered. Before the pick that first covers `r`, no
/// pick contains `r`, so every pick keeps its gain and still wins (other gains only drop). At that
/// pick, with gain `G`, the winner can change only if `G = 1`, or some remaining candidate not
/// tight at `r` has gain `G`, or gain `G − 1` at a lower position. If it does not change, both
/// covers hold `r` from then on and are identical. So only the objects where that condition holds
/// are rerun, each exactly; every other object flips nothing. The reruns are skipped, and the
/// attribution reported as not computed, when their estimated work exceeds
/// [`GreedyTouch::WORK_BUDGET`] word operations.
pub struct GreedyTouch;

impl GreedyTouch {
    pub const WORK_BUDGET: u64 = 200_000_000_000;
}

/// One pick of the greedy cover: the group member kept, the objects it added (bits over the active
/// objects), and those added objects whose removal could change this pick.
struct Pick {
    member: usize,
    added: Vec<u64>,
    divergent: Vec<u64>,
}

/// Candidates grouped by target and operator (input positions), each with its tight objects as bits
/// over `objects`.
fn greedy_groups(
    ev: &Evaluated,
    input: &[usize],
    objects: &[usize],
) -> Vec<(Vec<usize>, Vec<Vec<u64>>)> {
    let words = objects.len().div_ceil(64);
    let mut groups: HashMap<(&str, Op), Vec<usize>> = HashMap::new();
    for (pos, &i) in input.iter().enumerate() {
        let b = ev.bound(i);
        groups
            .entry((b.target.as_str(), b.op))
            .or_default()
            .push(pos);
    }
    groups
        .into_values()
        .map(|members| {
            let bits = members
                .par_iter()
                .map(|&pos| {
                    let mut w = vec![0u64; words];
                    for (k, &r) in objects.iter().enumerate() {
                        if ev.tight(input[pos], r) {
                            w[k >> 6] |= 1 << (k & 63);
                        }
                    }
                    w
                })
                .collect();
            (members, bits)
        })
        .collect()
}

/// The greedy cover of one group; `precovered` is an object position treated as covered from the
/// start. With `trace`, each pick also records its divergent objects (see [`GreedyTouch`]).
fn greedy_cover(
    bits: &[Vec<u64>],
    words: usize,
    precovered: Option<usize>,
    trace: bool,
) -> Vec<Pick> {
    let n = bits.len();
    let mut covered = vec![0u64; words];
    if let Some(k) = precovered {
        covered[k >> 6] |= 1 << (k & 63);
    }
    let mut kept = vec![false; n];
    let mut picks = Vec::new();
    loop {
        let gains: Vec<u32> = (0..n)
            .map(|m| {
                if kept[m] {
                    0
                } else {
                    bits[m]
                        .iter()
                        .zip(&covered)
                        .map(|(b, c)| (b & !c).count_ones())
                        .sum()
                }
            })
            .collect();
        // Members are in input order, so the first maximum is the lowest position.
        let mut best: Option<(usize, u32)> = None;
        for m in (0..n).filter(|&m| !kept[m]) {
            if best.is_none_or(|(_, g)| gains[m] > g) {
                best = Some((m, gains[m]));
            }
        }
        let Some((p, g)) = best else {
            break;
        };
        if g == 0 {
            break;
        }
        let added: Vec<u64> = bits[p].iter().zip(&covered).map(|(b, c)| b & !c).collect();
        let divergent = if !trace {
            Vec::new()
        } else if g == 1 {
            added.clone()
        } else {
            // Objects every contender is tight at cannot change this pick.
            let mut common: Option<Vec<u64>> = None;
            for m in (0..n)
                .filter(|&m| m != p && !kept[m] && (gains[m] == g || (gains[m] == g - 1 && m < p)))
            {
                match &mut common {
                    None => common = Some(bits[m].clone()),
                    Some(c) => c.iter_mut().zip(&bits[m]).for_each(|(c, b)| *c &= b),
                }
            }
            match common {
                None => vec![0; words],
                Some(c) => added.iter().zip(&c).map(|(a, c)| a & !c).collect(),
            }
        };
        for (c, b) in covered.iter_mut().zip(&bits[p]) {
            *c |= b;
        }
        kept[p] = true;
        picks.push(Pick {
            member: p,
            added,
            divergent,
        });
    }
    picks
}

fn bit_positions(words: &[u64]) -> impl Iterator<Item = usize> + '_ {
    words.iter().enumerate().flat_map(|(w, &word)| {
        let mut x = word;
        std::iter::from_fn(move || {
            (x != 0).then(|| {
                let t = x.trailing_zeros() as usize;
                x &= x - 1;
                w * 64 + t
            })
        })
    })
}

impl Filter for GreedyTouch {
    fn name(&self) -> &'static str {
        "greedy_touch"
    }

    fn kind(&self) -> Kind {
        Kind::Redundancy
    }

    fn partition(&self, ev: &Evaluated, input: &[usize], active: &[bool]) -> Partition {
        let objects = active_objects(active);
        let words = objects.len().div_ceil(64);
        let mut classes = vec![Class::Discarded { by: None }; input.len()];
        let mut witnesses = vec![Vec::new(); input.len()];
        for (members, bits) in greedy_groups(ev, input, &objects) {
            for pick in greedy_cover(&bits, words, None, false) {
                let pos = members[pick.member];
                classes[pos] = Class::Representative;
                witnesses[pos] = bit_positions(&pick.added).map(|k| objects[k]).collect();
            }
        }
        Partition::new(input.to_vec(), classes, witnesses)
    }

    fn attribution(
        &self,
        ev: &Evaluated,
        input: &[usize],
        active: &[bool],
        _full: &Partition,
    ) -> Option<Vec<Vec<usize>>> {
        let objects = active_objects(active);
        let words = objects.len().div_ceil(64);
        let groups: Vec<_> = greedy_groups(ev, input, &objects)
            .into_iter()
            .map(|(members, bits)| {
                let picks = greedy_cover(&bits, words, None, true);
                (members, bits, picks)
            })
            .collect();
        let work: u64 = groups
            .iter()
            .map(|(members, _, picks)| {
                let reruns = picks
                    .iter()
                    .map(|p| {
                        p.divergent
                            .iter()
                            .map(|w| w.count_ones() as u64)
                            .sum::<u64>()
                    })
                    .sum::<u64>();
                reruns * (picks.len() as u64 + 1) * members.len() as u64 * words as u64
            })
            .sum();
        if work > Self::WORK_BUDGET {
            return None;
        }
        let mut flips = vec![Vec::new(); input.len()];
        for (members, bits, picks) in &groups {
            let mut kept = vec![false; members.len()];
            for p in picks {
                kept[p.member] = true;
            }
            let divergent: Vec<usize> = picks
                .iter()
                .flat_map(|p| bit_positions(&p.divergent))
                .collect();
            let changes: Vec<(usize, Vec<usize>)> = divergent
                .par_iter()
                .map(|&k| {
                    let mut alt = vec![false; members.len()];
                    for p in greedy_cover(bits, words, Some(k), false) {
                        alt[p.member] = true;
                    }
                    (
                        k,
                        (0..members.len()).filter(|&m| alt[m] != kept[m]).collect(),
                    )
                })
                .collect();
            for (k, changed) in changes {
                for m in changed {
                    flips[members[m]].push(objects[k]);
                }
            }
        }
        for f in &mut flips {
            f.sort_unstable();
        }
        Some(flips)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use ragusa_gen::{Bound, Rational};
    use ragusa_table::{BooleanBlock, NumericBlock, PackedColumns, Table};

    /// Four objects: x = 1..4, y = [2, 3, 3, 5]; `p` holds everywhere, `q` on objects 0 and 1.
    pub(crate) fn tiny() -> (Table, Vec<Bound>) {
        let mut bits = PackedColumns::zeros(4, 2);
        for r in 0..4 {
            bits.set(r, 0);
        }
        bits.set(0, 1);
        bits.set(1, 1);
        let table = Table::new(
            "tiny",
            (0..4).map(|r| r.to_string()).collect(),
            Some(NumericBlock::from_columns(
                vec!["x".into(), "y".into()],
                vec![vec![1.0, 2.0, 3.0, 4.0], vec![2.0, 3.0, 3.0, 5.0]],
            )),
            Some(BooleanBlock::new(vec!["p".into(), "q".into()], bits)),
            vec![],
            vec![],
        );
        let r = |n, d| Rational::new(n, d);
        let bound = |h: &str, op, c: Rational, k: Rational| {
            Bound::new(h, "y", op, [("x".to_string(), c)], k)
        };
        let bounds = vec![
            bound("p", Op::Le, r(1, 1), r(1, 1)), // 0: y <= x + 1, valid, tight at 0, 1, 3
            bound("p", Op::Le, r(2, 1), r(0, 1)), // 1: y <= 2x, valid
            bound("p", Op::Le, r(1, 1), r(0, 1)), // 2: y <= x, violated at 0, 1, 3
            bound("q", Op::Le, r(1, 1), r(1, 1)), // 3: same conclusion as 0 under narrower q
            bound("p", Op::Le, r(1, 1), r(1, 1)), // 4: same statement as 0
            bound("p", Op::Ge, r(1, 1), r(0, 1)), // 5: y >= x, valid
            bound("p", Op::Ge, r(1, 1), r(1, 1)), // 6: y >= x + 1, violated only at 2
        ];
        (table, bounds)
    }

    const ALL: [bool; 4] = [true; 4];

    #[test]
    fn validity_discards_with_counterexamples() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        let p = Validity.partition(&ev, &[0, 1, 2, 5, 6], &ALL);
        assert_eq!(p.representatives(), vec![0, 1, 5]);
        assert_eq!(p.witnesses[2], vec![0, 1, 3]);
        assert_eq!(p.witnesses[4], vec![2]);
        assert_eq!(p.counts(), (3, 0, 2));
    }

    #[test]
    fn table_duplicates_alias_identical_statements() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        let p = TableDuplicates.partition(&ev, &[0, 1, 3, 4, 5], &ALL);
        assert_eq!(p.classes[3], Class::Alias { of: 0 });
        assert_eq!(p.counts(), (4, 1, 0));
    }

    #[test]
    fn sharp_coverage_keeps_unique_best_bounds() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        let p = SharpCoverage.partition(&ev, &[0, 1, 3, 5], &ALL);
        // objects 0 and 1 are ties between 0 and 3; 0 alone is best at 2 and 3; 5 is alone in its direction
        assert_eq!(p.witnesses[0], vec![2, 3]);
        assert_eq!(p.classes[1], Class::Discarded { by: None });
        assert_eq!(p.classes[2], Class::Discarded { by: None });
        assert_eq!(p.witnesses[3], vec![0, 1, 2, 3]);
        assert_eq!(p.representatives(), vec![0, 5]);
    }

    #[test]
    fn sharp_coverage_needs_separated_intervals() {
        // One object; y exact at 0; x in [1, 3]. Bounds y <= x (rhs [1, 3]) and y <= 2 (rhs 2)
        // overlap, so neither wins; y <= 5 against y <= x: [1, 3] lies entirely below 5, so y <= x wins.
        let mut bits = PackedColumns::zeros(1, 1);
        bits.set(0, 0);
        let table = Table::new(
            "overlap",
            vec!["o".into()],
            Some(NumericBlock::from_intervals(
                vec!["x".into(), "y".into()],
                vec![vec![1.0], vec![0.0]],
                vec![vec![3.0], vec![0.0]],
            )),
            Some(BooleanBlock::new(vec!["p".into()], bits)),
            vec![],
            vec![],
        );
        let r = |n| Rational::new(n, 1);
        let bounds = vec![
            Bound::new("p", "y", Op::Le, [("x".to_string(), r(1))], r(0)),
            Bound::new("p", "y", Op::Le, [], r(2)),
            Bound::new("p", "y", Op::Le, [], r(5)),
        ];
        let ev = Evaluated::new(&table, bounds).unwrap();
        assert!(SharpCoverage
            .partition(&ev, &[0, 1], &[true])
            .representatives()
            .is_empty());
        assert_eq!(
            SharpCoverage
                .partition(&ev, &[0, 2], &[true])
                .representatives(),
            vec![0]
        );
    }

    #[test]
    fn generality_discards_narrower_hypothesis() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        let p = Generality.partition(&ev, &[0, 1, 3, 5], &ALL);
        assert_eq!(p.classes[2], Class::Discarded { by: Some(0) });
        assert_eq!(p.witnesses[2], vec![2, 3]);
        assert_eq!(p.representatives(), vec![0, 1, 5]);
    }

    #[test]
    fn maximal_touch_discards_bounds_attained_inside_another() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        // tight sets: 0 {0,1,3}, 1 {0}, 3 {0,1} (all y <=); 5 {2} is alone among y >=
        let p = MaximalTouch.partition(&ev, &[0, 1, 3, 5], &ALL);
        assert_eq!(p.representatives(), vec![0, 5]);
        assert_eq!(p.classes[1], Class::Discarded { by: Some(0) });
        assert_eq!(p.witnesses[1], vec![1, 3]);
        assert_eq!(p.witnesses[2], vec![3]);
    }

    #[test]
    fn greedy_touch_covers_tight_objects() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        let p = GreedyTouch.partition(&ev, &[0, 1, 3, 5], &ALL);
        assert_eq!(p.representatives(), vec![0, 5]);
        assert_eq!(p.witnesses[0], vec![0, 1, 3]);
        assert_eq!(p.witnesses[3], vec![2]);
        // Without candidate 0, candidate 3 covers {0, 1} first and candidate 1 adds nothing.
        let q = GreedyTouch.partition(&ev, &[1, 3], &ALL);
        assert_eq!(q.representatives(), vec![3]);
    }

    #[test]
    fn touch_keeps_bounds_tight_often_enough() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        // candidate 0 (y <= x + 1) is tight at objects 0, 1, 3; candidate 1 (y <= 2x) only at 0
        let count = |k| Touch {
            threshold: TouchThreshold::Count(k),
        };
        let p = count(3).partition(&ev, &[0, 1], &ALL);
        assert_eq!(p.classes[0], Class::Representative);
        assert_eq!(p.witnesses[0], vec![0, 1, 3]);
        assert_eq!(p.classes[1], Class::Discarded { by: None });
        assert!(count(4)
            .partition(&ev, &[0], &ALL)
            .representatives()
            .is_empty());
        // √: candidate 0 applies on 4 objects (needs 2, has 3); 1 on 4 (needs 2, has 1);
        // 3 (hypothesis q) on 2 objects (needs 2, has {0, 1}).
        let sqrt = Touch {
            threshold: TouchThreshold::SqrtApplicable,
        };
        assert_eq!(
            sqrt.partition(&ev, &[0, 1, 3], &ALL).representatives(),
            vec![0, 3]
        );
        let needed: Vec<usize> = [0, 1, 2, 4, 5, 9, 10]
            .iter()
            .map(|&a| TouchThreshold::SqrtApplicable.needed(a))
            .collect();
        assert_eq!(needed, vec![0, 1, 2, 2, 3, 3, 4]);
    }

    #[test]
    fn inactive_objects_do_not_count() {
        let (table, bounds) = tiny();
        let ev = Evaluated::new(&table, bounds).unwrap();
        let without_2 = [true, true, false, true];
        let p = Validity.partition(&ev, &[6], &without_2);
        assert_eq!(p.representatives(), vec![6]);
    }
}
