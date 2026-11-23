//! Implication conjectures over a boolean block.
//!
//! `P_i ⇒ P_j` (i ≠ j) is emitted when no object satisfies `P_i` and refutes `P_j`, i.e. when
//! `column_i & known_j & !column_j` is all zero: an object whose value of `P_j` is unknown refutes
//! nothing (and an unknown `P_i` has bit 0). The scan works on packed words, parallel over chunks of `i`,
//! and stops counting a pair's refuters at two. Alongside the emitted matrix it records, for every
//! object, the pairs that object alone refutes: exactly the decisions its removal would flip.

use ragusa_table::{BoolColumns, PackedColumns};
use rayon::prelude::*;

pub struct ImplicationScan {
    n: usize,
    words_per_row: usize,
    emitted: Vec<u64>,
    /// Per object: ordered pairs (i ≠ j) that this object alone refutes.
    pub unique_refutations: Vec<u64>,
    /// Ordered pairs (i ≠ j) refuted by at least one object.
    pub refuted: u64,
    /// Ordered pairs refuted by exactly one object.
    pub refuted_once: u64,
}

impl ImplicationScan {
    pub fn n(&self) -> usize {
        self.n
    }

    /// Ordered pairs considered: n·(n − 1).
    pub fn pairs(&self) -> u64 {
        (self.n as u64) * (self.n as u64).saturating_sub(1)
    }

    pub fn emitted(&self, i: usize, j: usize) -> bool {
        (self.emitted[i * self.words_per_row + (j >> 6)] >> (j & 63)) & 1 == 1
    }

    pub fn emitted_count(&self) -> u64 {
        self.emitted.iter().map(|w| u64::from(w.count_ones())).sum()
    }
}

const CHUNK: usize = 64;

struct Chunk {
    rows: Vec<u64>,
    unique: Vec<u64>,
    refuted: u64,
    refuted_once: u64,
}

/// `known` marks which values are known; `None` means all are.
pub fn scan(bits: &PackedColumns, known: Option<&PackedColumns>) -> ImplicationScan {
    let n = bits.n_columns();
    let objects = bits.n_objects();
    let words_per_row = n.div_ceil(64);
    let starts: Vec<usize> = (0..n).step_by(CHUNK).collect();
    let chunks: Vec<Chunk> = starts
        .par_iter()
        .map(|&start| {
            let end = (start + CHUNK).min(n);
            let mut chunk = Chunk {
                rows: vec![0; (end - start) * words_per_row],
                unique: vec![0; objects],
                refuted: 0,
                refuted_once: 0,
            };
            for i in start..end {
                let ci = bits.column_words(i);
                let row =
                    &mut chunk.rows[(i - start) * words_per_row..(i - start + 1) * words_per_row];
                for j in (0..n).filter(|&j| j != i) {
                    let cj = bits.column_words(j);
                    let kj = known.map(|k| k.column_words(j));
                    let (mut count, mut last) = (0u32, (0usize, 0u64));
                    for (w, (&a, &b)) in ci.iter().zip(cj).enumerate() {
                        let x = match kj {
                            Some(k) => a & k[w] & !b,
                            None => a & !b,
                        };
                        if x != 0 {
                            count += x.count_ones();
                            last = (w, x);
                            if count > 1 {
                                break;
                            }
                        }
                    }
                    match count {
                        0 => row[j >> 6] |= 1 << (j & 63),
                        1 => {
                            chunk.refuted += 1;
                            chunk.refuted_once += 1;
                            chunk.unique[last.0 * 64 + last.1.trailing_zeros() as usize] += 1;
                        }
                        _ => chunk.refuted += 1,
                    }
                }
            }
            chunk
        })
        .collect();

    let mut scan = ImplicationScan {
        n,
        words_per_row,
        emitted: Vec::with_capacity(n * words_per_row),
        unique_refutations: vec![0; objects],
        refuted: 0,
        refuted_once: 0,
    };
    for chunk in chunks {
        scan.emitted.extend(chunk.rows);
        scan.unique_refutations
            .iter_mut()
            .zip(&chunk.unique)
            .for_each(|(u, c)| *u += c);
        scan.refuted += chunk.refuted;
        scan.refuted_once += chunk.refuted_once;
    }
    scan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_table_by_hand() {
        // objects × predicates: p0 holds on {0, 1, 2}, p1 on {0, 1}, p2 on {2}
        let mut bits = PackedColumns::zeros(3, 3);
        for (r, c) in [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 2)] {
            bits.set(r, c);
        }
        let s = scan(&bits, None);
        assert!(s.emitted(1, 0) && s.emitted(2, 0));
        assert!(!s.emitted(0, 1) && !s.emitted(0, 2) && !s.emitted(1, 2) && !s.emitted(2, 1));
        assert_eq!(s.emitted_count(), 2);
        assert_eq!(s.refuted, 4);
        // p0 ⇒ p1 refuted only by object 2; p0 ⇒ p2 by objects 0 and 1; p1 ⇒ p2 by 0 and 1;
        // p2 ⇒ p1 only by object 2
        assert_eq!(s.refuted_once, 2);
        assert_eq!(s.unique_refutations, vec![0, 0, 2]);
    }

    #[test]
    fn unknown_values_refute_nothing() {
        // p0 true on both objects; p1 false on object 0 and unknown on object 1.
        let mut bits = PackedColumns::zeros(2, 2);
        bits.set(0, 0);
        bits.set(1, 0);
        let mut known = PackedColumns::zeros(2, 2);
        for r in 0..2 {
            known.set(r, 0);
        }
        known.set(0, 1);
        let with_unknown = scan(&bits, Some(&known));
        assert!(!with_unknown.emitted(0, 1), "object 0 refutes p0 => p1");
        assert_eq!(with_unknown.unique_refutations, vec![1, 0]);
        let as_false = scan(&bits, None);
        assert_eq!(
            as_false.unique_refutations,
            vec![0, 0],
            "both objects refute when unknown reads as false"
        );
    }
}
