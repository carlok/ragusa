use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::bits::{BoolColumns, PackedColumns};

/// Dense numeric block: objects × invariants, column-major, NaN for a missing value.
///
/// A cell is a closed interval `[lower, upper]`; a point value has lower = upper. The upper values
/// are stored only when some cell is a genuine interval, so a block of points has exactly the
/// representation (and content hash) it had before intervals existed.
#[derive(Clone, Debug)]
pub struct NumericBlock {
    names: Vec<String>,
    n_objects: usize,
    values: Vec<f64>,
    upper: Option<Vec<f64>>,
}

impl NumericBlock {
    /// `columns[c][r]` is invariant `c` on object `r`. At least one column.
    pub fn from_columns(names: Vec<String>, columns: Vec<Vec<f64>>) -> Self {
        assert_eq!(names.len(), columns.len(), "one name per numeric column");
        assert!(
            !columns.is_empty(),
            "an empty numeric block is `None`, not zero columns"
        );
        let n_objects = columns[0].len();
        assert!(
            columns.iter().all(|c| c.len() == n_objects),
            "numeric columns differ in length"
        );
        NumericBlock {
            names,
            n_objects,
            values: columns.concat(),
            upper: None,
        }
    }

    /// `lower[c][r]` and `upper[c][r]` bound invariant `c` on object `r`: both NaN when missing,
    /// otherwise lower <= upper. Upper values equal to their lower values are not stored.
    pub fn from_intervals(names: Vec<String>, lower: Vec<Vec<f64>>, upper: Vec<Vec<f64>>) -> Self {
        assert_eq!(
            lower.len(),
            upper.len(),
            "one upper column per lower column"
        );
        let mut block = NumericBlock::from_columns(names, lower);
        let upper = upper.concat();
        assert_eq!(
            upper.len(),
            block.values.len(),
            "upper columns differ in length"
        );
        let mut all_points = true;
        for (lo, hi) in block.values.iter().zip(&upper) {
            match (lo.is_nan(), hi.is_nan()) {
                (true, true) => {}
                (false, false) => {
                    assert!(lo <= hi, "interval with lower {lo} > upper {hi}");
                    all_points &= lo.to_bits() == hi.to_bits();
                }
                _ => panic!("a cell is missing in only one of lower and upper"),
            }
        }
        if !all_points {
            block.upper = Some(upper);
        }
        block
    }

    /// Some cell is a genuine interval.
    pub fn has_intervals(&self) -> bool {
        self.upper.is_some()
    }

    pub fn lower(&self, c: usize) -> &[f64] {
        &self.values[c * self.n_objects..(c + 1) * self.n_objects]
    }

    pub fn upper(&self, c: usize) -> &[f64] {
        match &self.upper {
            Some(u) => &u[c * self.n_objects..(c + 1) * self.n_objects],
            None => self.lower(c),
        }
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn n_objects(&self) -> usize {
        self.n_objects
    }

    pub fn n_columns(&self) -> usize {
        self.names.len()
    }

    /// The exact values of column `c`. Panics on a block with interval cells, so that no caller
    /// can mistake a lower bound for a value; use [`NumericBlock::lower`] and
    /// [`NumericBlock::upper`] there.
    pub fn column(&self, c: usize) -> &[f64] {
        assert!(
            self.upper.is_none(),
            "column(): the block has interval cells; use lower() and upper()"
        );
        self.lower(c)
    }
}

/// Boolean block: objects × predicates, bit-packed.
///
/// A value may be unknown. The mask of known values is stored only when some value is unknown;
/// an unknown value's bit in [`BooleanBlock::bits`] is always 0.
#[derive(Clone, Debug)]
pub struct BooleanBlock {
    names: Vec<String>,
    bits: PackedColumns,
    known: Option<PackedColumns>,
}

impl BooleanBlock {
    pub fn new(names: Vec<String>, bits: PackedColumns) -> Self {
        assert_eq!(names.len(), bits.n_columns(), "one name per boolean column");
        assert!(
            !names.is_empty(),
            "an empty boolean block is `None`, not zero columns"
        );
        BooleanBlock {
            names,
            bits,
            known: None,
        }
    }

    /// `known` marks which values are known; `bits` must be 0 wherever `known` is 0.
    pub fn with_known(names: Vec<String>, bits: PackedColumns, known: PackedColumns) -> Self {
        assert_eq!(
            (bits.n_objects(), bits.n_columns()),
            (known.n_objects(), known.n_columns()),
            "known mask has a different shape"
        );
        assert!(
            bits.words()
                .iter()
                .zip(known.words())
                .all(|(b, k)| b & !k == 0),
            "a value is set where it is not known"
        );
        let mut block = BooleanBlock::new(names, bits);
        let n = known.n_objects() as u64;
        if (0..known.n_columns()).any(|c| known.count_ones(c) != n) {
            block.known = Some(known);
        }
        block
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn bits(&self) -> &PackedColumns {
        &self.bits
    }

    pub fn known_bits(&self) -> Option<&PackedColumns> {
        self.known.as_ref()
    }

    pub fn is_known(&self, row: usize, col: usize) -> bool {
        self.known.as_ref().is_none_or(|k| k.get(row, col))
    }
}

/// A file a table was loaded from, by name and content hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    pub file: String,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub struct Table {
    name: String,
    objects: Vec<String>,
    numeric: Option<NumericBlock>,
    boolean: Option<BooleanBlock>,
    skipped: Vec<String>,
    sources: Vec<SourceFile>,
}

impl Table {
    pub fn new(
        name: impl Into<String>,
        objects: Vec<String>,
        numeric: Option<NumericBlock>,
        boolean: Option<BooleanBlock>,
        skipped: Vec<String>,
        sources: Vec<SourceFile>,
    ) -> Self {
        let n = objects.len();
        if let Some(b) = &numeric {
            assert_eq!(
                b.n_objects(),
                n,
                "numeric block has {} objects, table has {n}",
                b.n_objects()
            );
        }
        if let Some(b) = &boolean {
            assert_eq!(
                b.bits().n_objects(),
                n,
                "boolean block has {} objects, table has {n}",
                b.bits().n_objects()
            );
        }
        {
            let mut seen = HashSet::new();
            let names = numeric
                .iter()
                .flat_map(|b| b.names())
                .chain(boolean.iter().flat_map(|b| b.names()));
            for col in names {
                assert!(seen.insert(col.as_str()), "duplicate column name {col:?}");
            }
        }
        Table {
            name: name.into(),
            objects,
            numeric,
            boolean,
            skipped,
            sources,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn objects(&self) -> &[String] {
        &self.objects
    }

    pub fn n_objects(&self) -> usize {
        self.objects.len()
    }

    pub fn numeric(&self) -> Option<&NumericBlock> {
        self.numeric.as_ref()
    }

    pub fn boolean(&self) -> Option<&BooleanBlock> {
        self.boolean.as_ref()
    }

    /// Source columns the loader did not type as numeric or boolean.
    pub fn skipped(&self) -> &[String] {
        &self.skipped
    }

    pub fn sources(&self) -> &[SourceFile] {
        &self.sources
    }

    /// The given rows, in the given order, as a new table with the same columns, sources and
    /// skipped columns. Upper values and the known-value mask are stored only if the chosen rows
    /// need them, so a subset of every row in order has the table's content hash.
    pub fn subset(&self, name: impl Into<String>, rows: &[usize]) -> Table {
        let numeric = self.numeric.as_ref().map(|b| {
            let pick = |col: &[f64]| rows.iter().map(|&r| col[r]).collect::<Vec<f64>>();
            NumericBlock::from_intervals(
                b.names.clone(),
                (0..b.n_columns()).map(|c| pick(b.lower(c))).collect(),
                (0..b.n_columns()).map(|c| pick(b.upper(c))).collect(),
            )
        });
        let boolean = self.boolean.as_ref().map(|b| {
            let n_columns = b.names.len();
            let mut bits = PackedColumns::zeros(rows.len(), n_columns);
            let mut known = PackedColumns::zeros(rows.len(), n_columns);
            for (k, &r) in rows.iter().enumerate() {
                for c in 0..n_columns {
                    if b.bits.get(r, c) {
                        bits.set(k, c);
                    }
                    if b.is_known(r, c) {
                        known.set(k, c);
                    }
                }
            }
            BooleanBlock::with_known(b.names.clone(), bits, known)
        });
        Table::new(
            name,
            rows.iter().map(|&r| self.objects[r].clone()).collect(),
            numeric,
            boolean,
            self.skipped.clone(),
            self.sources.clone(),
        )
    }

    /// sha256 of a canonical serialization of the content: object names, column names,
    /// numeric values as little-endian `f64` with every NaN written as 0x7ff8000000000000,
    /// boolean words little-endian. Upper interval values and the known-value mask are appended
    /// only when stored, so tables without them hash exactly as before they existed.
    /// Independent of the file format it came from.
    pub fn content_sha256(&self) -> String {
        let mut h = Sha256::new();
        h.update(b"ragusa-table-v1\0");
        put_strs(&mut h, &self.objects);
        match &self.numeric {
            None => h.update([0u8]),
            Some(b) => {
                h.update([1u8]);
                put_strs(&mut h, &b.names);
                for v in &b.values {
                    h.update(canonical_f64(*v).to_bits().to_le_bytes());
                }
                if let Some(upper) = &b.upper {
                    h.update(b"upper\0");
                    for v in upper {
                        h.update(canonical_f64(*v).to_bits().to_le_bytes());
                    }
                }
            }
        }
        match &self.boolean {
            None => h.update([0u8]),
            Some(b) => {
                h.update([1u8]);
                put_strs(&mut h, &b.names);
                for w in b.bits.words() {
                    h.update(w.to_le_bytes());
                }
                if let Some(known) = &b.known {
                    h.update(b"known\0");
                    for w in known.words() {
                        h.update(w.to_le_bytes());
                    }
                }
            }
        }
        to_hex(&h.finalize())
    }
}

/// Every NaN as the one quiet NaN pandas and this crate agree on.
pub(crate) fn canonical_f64(v: f64) -> f64 {
    if v.is_nan() {
        f64::from_bits(0x7ff8_0000_0000_0000)
    } else {
        v
    }
}

pub(crate) fn sha256_hex(data: &[u8]) -> String {
    to_hex(&Sha256::digest(data))
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn put_strs(h: &mut Sha256, items: &[String]) {
    h.update((items.len() as u64).to_le_bytes());
    for s in items {
        h.update((s.len() as u64).to_le_bytes());
        h.update(s.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intervals_of_points_collapse_to_points() {
        let points = NumericBlock::from_columns(vec!["x".into()], vec![vec![1.0, f64::NAN]]);
        let same = NumericBlock::from_intervals(
            vec!["x".into()],
            vec![vec![1.0, f64::NAN]],
            vec![vec![1.0, f64::NAN]],
        );
        assert!(!same.has_intervals());
        let t = |b: NumericBlock| {
            Table::new(
                "t",
                vec!["a".into(), "b".into()],
                Some(b),
                None,
                vec![],
                vec![],
            )
            .content_sha256()
        };
        assert_eq!(t(points), t(same));

        let wide = NumericBlock::from_intervals(
            vec!["x".into()],
            vec![vec![1.0, f64::NAN]],
            vec![vec![2.0, f64::NAN]],
        );
        assert!(wide.has_intervals());
        assert_eq!(wide.lower(0)[0], 1.0);
        assert_eq!(wide.upper(0)[0], 2.0);
        let narrow = NumericBlock::from_intervals(
            vec!["x".into()],
            vec![vec![1.0, f64::NAN]],
            vec![vec![1.0, f64::NAN]],
        );
        assert_ne!(t(wide), t(narrow));
    }

    #[test]
    #[should_panic(expected = "interval cells")]
    fn column_refuses_interval_blocks() {
        NumericBlock::from_intervals(vec!["x".into()], vec![vec![1.0]], vec![vec![2.0]]).column(0);
    }

    #[test]
    fn unknown_booleans_are_hashed_and_queryable() {
        let mut bits = PackedColumns::zeros(2, 1);
        bits.set(0, 0);
        let mut known = PackedColumns::zeros(2, 1);
        known.set(0, 0);
        let partial = BooleanBlock::with_known(vec!["p".into()], bits.clone(), known);
        assert!(partial.is_known(0, 0) && !partial.is_known(1, 0));

        let mut all = PackedColumns::zeros(2, 1);
        all.set(0, 0);
        all.set(1, 0);
        let full = BooleanBlock::with_known(vec!["p".into()], bits.clone(), all);
        assert!(full.known_bits().is_none());

        let t = |b: BooleanBlock| {
            Table::new(
                "t",
                vec!["a".into(), "b".into()],
                None,
                Some(b),
                vec![],
                vec![],
            )
            .content_sha256()
        };
        assert_ne!(t(partial), t(full.clone()));
        assert_eq!(t(full), t(BooleanBlock::new(vec!["p".into()], bits)));
    }

    fn tiny(bit: Option<usize>, nan: f64) -> Table {
        let mut bits = PackedColumns::zeros(3, 1);
        if let Some(r) = bit {
            bits.set(r, 0);
        }
        Table::new(
            "tiny",
            vec!["a".into(), "b".into(), "c".into()],
            Some(NumericBlock::from_columns(
                vec!["x".into()],
                vec![vec![1.0, nan, 3.0]],
            )),
            Some(BooleanBlock::new(vec!["p".into()], bits)),
            vec![],
            vec![],
        )
    }

    #[test]
    fn content_hash_sees_one_bit() {
        assert_ne!(
            tiny(Some(0), f64::NAN).content_sha256(),
            tiny(Some(1), f64::NAN).content_sha256()
        );
        assert_ne!(
            tiny(None, f64::NAN).content_sha256(),
            tiny(Some(0), f64::NAN).content_sha256()
        );
    }

    #[test]
    fn content_hash_ignores_nan_payload() {
        let other_nan = f64::from_bits(0x7ff8_0000_0000_0001);
        assert!(other_nan.is_nan());
        assert_eq!(
            tiny(None, f64::NAN).content_sha256(),
            tiny(None, other_nan).content_sha256()
        );
    }

    #[test]
    #[should_panic(expected = "duplicate column name")]
    fn duplicate_names_across_blocks_rejected() {
        Table::new(
            "dup",
            vec!["a".into()],
            Some(NumericBlock::from_columns(
                vec!["p".into()],
                vec![vec![1.0]],
            )),
            Some(BooleanBlock::new(
                vec!["p".into()],
                PackedColumns::zeros(1, 1),
            )),
            vec![],
            vec![],
        );
    }
}
