//! Loader for a parsimagma signature corpus, read as data.
//!
//! `corpus_signatures.bin.gz` holds 587 bytes per magma in corpus order; bit `i` of the
//! little-endian byte stream records whether ETP equation `i + 1` holds. `corpus_index.tsv`
//! has one line per signature in the same order. The result is a boolean-only table with one
//! column per law, named `E1` … `E4694`. The load fails if row numbers are out of order, a
//! padding bit is set, or a signature's popcount differs from its `laws_satisfied`.

use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;

use crate::bits::PackedColumns;
use crate::error::LoadError;
use crate::table::{sha256_hex, BooleanBlock, SourceFile, Table};

pub const ETP_LAWS: usize = 4694;
const SIGNATURE_BYTES: usize = ETP_LAWS.div_ceil(8);
const INDEX_HEADER: &str = "row\tfamily\tparams\tcarrier\tlaws_satisfied";

struct Entry {
    label: String,
    laws_satisfied: u32,
}

pub fn load_corpus(dir: &Path, name: &str) -> Result<Table, LoadError> {
    let sig_path = dir.join("corpus_signatures.bin.gz");
    let idx_path = dir.join("corpus_index.tsv");

    let compressed = std::fs::read(&sig_path).map_err(|e| LoadError::io(&sig_path, e))?;
    let mut raw = Vec::new();
    GzDecoder::new(&compressed[..])
        .read_to_end(&mut raw)
        .map_err(|e| LoadError::io(&sig_path, e))?;

    let index_bytes = std::fs::read(&idx_path).map_err(|e| LoadError::io(&idx_path, e))?;
    let index = std::str::from_utf8(&index_bytes)
        .map_err(|e| LoadError::format(&idx_path, format!("not UTF-8: {e}")))?;
    let mut lines = index.lines();
    if lines.next() != Some(INDEX_HEADER) {
        return Err(LoadError::format(
            &idx_path,
            format!("header is not {INDEX_HEADER:?}"),
        ));
    }
    let entries = lines
        .enumerate()
        .map(|(i, line)| {
            let f: Vec<&str> = line.split('\t').collect();
            let bad = |why: &str| LoadError::format(&idx_path, format!("line {}: {why}", i + 2));
            if f.len() != 5 {
                return Err(bad("expected 5 tab-separated fields"));
            }
            if f[0].parse::<usize>().ok() != Some(i) {
                return Err(bad("row number out of order"));
            }
            let laws_satisfied = f[4]
                .parse()
                .map_err(|_| bad("laws_satisfied is not an integer"))?;
            Ok(Entry {
                label: format!("{} {} {}", f[1], f[2], f[3]),
                laws_satisfied,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let n = entries.len();
    if raw.len() != n * SIGNATURE_BYTES {
        return Err(LoadError::format(
            &sig_path,
            format!(
                "{} bytes, expected {n} signatures x {SIGNATURE_BYTES} bytes",
                raw.len()
            ),
        ));
    }

    let mut bits = PackedColumns::zeros(n, ETP_LAWS);
    for (r, signature) in raw.chunks_exact(SIGNATURE_BYTES).enumerate() {
        let mut count = 0u32;
        for (k, &byte) in signature.iter().enumerate() {
            let mut b = byte;
            while b != 0 {
                let law = k * 8 + b.trailing_zeros() as usize;
                if law >= ETP_LAWS {
                    return Err(LoadError::format(
                        &sig_path,
                        format!("signature {r}: padding bit {law} set"),
                    ));
                }
                bits.set(r, law);
                count += 1;
                b &= b - 1;
            }
        }
        if count != entries[r].laws_satisfied {
            return Err(LoadError::format(
                &sig_path,
                format!(
                    "signature {r}: {count} laws set, index says {}",
                    entries[r].laws_satisfied
                ),
            ));
        }
    }

    Ok(Table::new(
        name,
        entries.into_iter().map(|e| e.label).collect(),
        None,
        Some(BooleanBlock::new(
            (1..=ETP_LAWS).map(|i| format!("E{i}")).collect(),
            bits,
        )),
        vec![],
        vec![
            SourceFile {
                file: "corpus_signatures.bin.gz".into(),
                sha256: sha256_hex(&compressed),
            },
            SourceFile {
                file: "corpus_index.tsv".into(),
                sha256: sha256_hex(&index_bytes),
            },
        ],
    ))
}
