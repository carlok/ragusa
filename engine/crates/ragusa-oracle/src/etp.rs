//! The ETP implication graph (teorth/equational_theories, Apache-2.0), as vendored by parsimagma.
//!
//! A packed 4694 × 4694 bit matrix: bit `j` of row `i` (rows byte-packed, least significant bit
//! first) is set iff law `E_(i+1)` entails `E_(j+1)` for all magmas, diagonal included. It was
//! derived from Janota's exhaustive Vampire run (arXiv:2508.15856); see
//! `engine/data/parsimagma/PROVENANCE.txt`.

use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

pub const LAWS: usize = 4694;

/// The vendored copies under `engine/data/parsimagma`.
pub fn vendored_matrix() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/parsimagma/implications.bits.gz"
    ))
}

pub fn vendored_hard_core() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/parsimagma/hard_core.txt"
    ))
}

pub struct EtpImplications {
    n: usize,
    row_bytes: usize,
    bits: Vec<u8>,
}

impl EtpImplications {
    /// Loads a gzipped matrix over [`LAWS`] laws.
    pub fn load(path: &Path) -> Result<EtpImplications, String> {
        let compressed = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut bits = Vec::new();
        GzDecoder::new(&compressed[..])
            .read_to_end(&mut bits)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        EtpImplications::from_bytes(LAWS, bits).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn from_bytes(n: usize, bits: Vec<u8>) -> Result<EtpImplications, String> {
        let row_bytes = n.div_ceil(8);
        if bits.len() != n * row_bytes {
            return Err(format!(
                "{} bytes, expected {n} rows × {row_bytes} bytes",
                bits.len()
            ));
        }
        Ok(EtpImplications { n, row_bytes, bits })
    }

    pub fn n(&self) -> usize {
        self.n
    }

    /// Does `E_(i+1)` entail `E_(j+1)`? Indices are 0-based.
    pub fn holds(&self, i: usize, j: usize) -> bool {
        debug_assert!(i < self.n && j < self.n);
        (self.bits[i * self.row_bytes + (j >> 3)] >> (j & 7)) & 1 == 1
    }

    /// Set bits, diagonal included.
    pub fn count_true(&self) -> u64 {
        self.bits.iter().map(|b| u64::from(b.count_ones())).sum()
    }
}

/// `from to` per line, 1-based, `#` comments and blank lines ignored; returned 0-based.
pub fn parse_pairs(text: &str) -> Result<Vec<(usize, usize)>, String> {
    text.lines()
        .enumerate()
        .map(|(n, line)| (n, line.trim()))
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(n, line)| {
            let mut it = line.split_whitespace().map(str::parse::<usize>);
            match (it.next(), it.next()) {
                (Some(Ok(from)), Some(Ok(to))) if from >= 1 && to >= 1 => Ok((from - 1, to - 1)),
                _ => Err(format!("line {}: expected two 1-based law ids", n + 1)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendored_matrix_has_known_structure() {
        let m = EtpImplications::load(&vendored_matrix()).unwrap();
        assert_eq!(m.n(), LAWS);
        for i in 0..LAWS {
            assert!(m.holds(i, i), "E{} entails itself", i + 1);
            // E1 is x = x, entailed by every law; E2 is x = y, which entails every law.
            assert!(m.holds(i, 0), "E{} entails E1", i + 1);
            assert!(m.holds(1, i), "E2 entails E{}", i + 1);
        }
        assert!(!m.holds(0, 1), "x = x does not entail x = y");
    }

    #[test]
    fn hard_core_pairs_are_false_in_the_matrix() {
        let m = EtpImplications::load(&vendored_matrix()).unwrap();
        let pairs = parse_pairs(&std::fs::read_to_string(vendored_hard_core()).unwrap()).unwrap();
        assert!(!pairs.is_empty());
        assert!(pairs.iter().all(|&(i, j)| !m.holds(i, j)));
    }

    #[test]
    fn parse_pairs_rejects_zero_ids() {
        assert_eq!(parse_pairs("# c\n3 5\n\n"), Ok(vec![(2, 4)]));
        assert!(parse_pairs("0 5").is_err());
    }
}
