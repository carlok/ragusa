//! Boolean block storage.
//!
//! [`BoolColumns`] is the interface kernels are written against. [`PackedColumns`] is the
//! scalar reference implementation and stays in the repository for good: column-major,
//! `ceil(n_objects / 64)` words per column, bits past `n_objects` always zero. Any faster
//! backend (Roaring, SIMD) must give bit-identical results to it.

pub trait BoolColumns {
    fn n_objects(&self) -> usize;
    fn n_columns(&self) -> usize;
    /// The packed words of one column; bit `r` is object `r`.
    fn column_words(&self, col: usize) -> &[u64];

    fn get(&self, row: usize, col: usize) -> bool {
        debug_assert!(row < self.n_objects());
        (self.column_words(col)[row >> 6] >> (row & 63)) & 1 == 1
    }

    fn count_ones(&self, col: usize) -> u64 {
        self.column_words(col)
            .iter()
            .map(|w| u64::from(w.count_ones()))
            .sum()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedColumns {
    n_objects: usize,
    n_columns: usize,
    words_per_column: usize,
    words: Vec<u64>,
}

impl PackedColumns {
    pub fn zeros(n_objects: usize, n_columns: usize) -> Self {
        let words_per_column = n_objects.div_ceil(64);
        PackedColumns {
            n_objects,
            n_columns,
            words_per_column,
            words: vec![0; words_per_column * n_columns],
        }
    }

    pub fn set(&mut self, row: usize, col: usize) {
        assert!(
            row < self.n_objects && col < self.n_columns,
            "bit ({row}, {col}) out of range"
        );
        self.words[col * self.words_per_column + (row >> 6)] |= 1u64 << (row & 63);
    }

    pub fn words_per_column(&self) -> usize {
        self.words_per_column
    }

    /// All words, column-major.
    pub fn words(&self) -> &[u64] {
        &self.words
    }
}

impl BoolColumns for PackedColumns {
    fn n_objects(&self) -> usize {
        self.n_objects
    }

    fn n_columns(&self) -> usize {
        self.n_columns
    }

    fn column_words(&self, col: usize) -> &[u64] {
        let start = col * self.words_per_column;
        &self.words[start..start + self.words_per_column]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_and_count() {
        let mut b = PackedColumns::zeros(130, 3);
        for r in [0, 63, 64, 129] {
            b.set(r, 1);
        }
        assert!(b.get(0, 1) && b.get(63, 1) && b.get(64, 1) && b.get(129, 1));
        assert!(!b.get(1, 1) && !b.get(0, 0) && !b.get(129, 2));
        assert_eq!(b.count_ones(1), 4);
        assert_eq!(b.count_ones(0), 0);
        assert_eq!(b.column_words(1).len(), 3);
    }

    #[test]
    fn padding_bits_stay_zero() {
        let mut b = PackedColumns::zeros(65, 1);
        b.set(64, 0);
        assert_eq!(b.column_words(0), &[0, 1]);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn out_of_range_panics() {
        PackedColumns::zeros(10, 1).set(10, 0);
    }
}
