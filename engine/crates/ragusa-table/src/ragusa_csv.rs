//! Loader for ragusa CSV, the neutral table format written by
//! `calibration/src/ragusa_a/ragusa_csv.py` (which documents it in full).
//!
//! - Header: `name`, then `<column>:num` or `<column>:bool` per column. No quoting.
//! - `num` cells: empty (missing), a point (`3`, `2.0298832128`, `997/60`), or a closed interval
//!   `lo..hi` with lo <= hi.
//! - `bool` cells: `1`, `0`, or empty (missing).
//!
//! Decimals parse correctly rounded, which reproduces Python's `float()`. A fraction `p/q` is read
//! as `p as f64 / q as f64`, which equals Python's `float(Fraction(p, q))` because both operands are
//! required to be exactly representable (|p|, |q| < 2^53) and IEEE division is correctly rounded.
//! Gate G-B6 checks every value against the exporter's manifest.

use std::path::Path;

use crate::bits::PackedColumns;
use crate::error::LoadError;
use crate::table::{sha256_hex, BooleanBlock, NumericBlock, SourceFile, Table};

enum Kind {
    Num,
    Bool,
}

const EXACT: i64 = 1 << 53;

fn parse_value(text: &str) -> Result<f64, String> {
    match text.split_once('/') {
        Some((p, q)) => {
            let p: i64 = p
                .parse()
                .map_err(|_| format!("bad numerator in {text:?}"))?;
            let q: i64 = q
                .parse()
                .map_err(|_| format!("bad denominator in {text:?}"))?;
            if q <= 0 || p.abs() >= EXACT || q >= EXACT {
                return Err(format!("fraction out of exact range: {text:?}"));
            }
            Ok(p as f64 / q as f64)
        }
        None => text
            .parse::<f64>()
            .map_err(|_| format!("not a number: {text:?}"))
            .and_then(|v| {
                if v.is_finite() {
                    Ok(v)
                } else {
                    Err(format!("non-finite value: {text:?}"))
                }
            }),
    }
}

/// `(lower, upper)`; both NaN when missing.
fn parse_num(cell: &str) -> Result<(f64, f64), String> {
    if cell.is_empty() {
        return Ok((f64::NAN, f64::NAN));
    }
    match cell.split_once("..") {
        Some((lo, hi)) => {
            let (lo, hi) = (parse_value(lo)?, parse_value(hi)?);
            if lo > hi {
                return Err(format!("interval with lower > upper: {cell:?}"));
            }
            Ok((lo, hi))
        }
        None => parse_value(cell).map(|v| (v, v)),
    }
}

pub fn load(path: &Path, name: &str) -> Result<Table, LoadError> {
    let bytes = std::fs::read(path).map_err(|e| LoadError::io(path, e))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|e| LoadError::format(path, format!("not UTF-8: {e}")))?;
    if text.contains('"') || text.contains('\r') {
        return Err(LoadError::format(
            path,
            "quotes and CR are not allowed in ragusa CSV",
        ));
    }
    let mut lines = text.lines();
    let header: Vec<&str> = lines
        .next()
        .ok_or_else(|| LoadError::format(path, "empty file"))?
        .split(',')
        .collect();
    if header.first() != Some(&"name") {
        return Err(LoadError::format(path, "first header field must be `name`"));
    }
    let mut columns = Vec::new();
    for field in &header[1..] {
        let (col, kind) = field.rsplit_once(':').ok_or_else(|| {
            LoadError::format(path, format!("header field without kind: {field:?}"))
        })?;
        let kind = match kind {
            "num" => Kind::Num,
            "bool" => Kind::Bool,
            other => {
                return Err(LoadError::format(
                    path,
                    format!("unknown column kind {other:?} for {col:?}"),
                ))
            }
        };
        columns.push((col.to_string(), kind));
    }

    let mut objects = Vec::new();
    let n_num = columns
        .iter()
        .filter(|(_, k)| matches!(k, Kind::Num))
        .count();
    let mut lower: Vec<Vec<f64>> = vec![Vec::new(); n_num];
    let mut upper: Vec<Vec<f64>> = vec![Vec::new(); n_num];
    let mut bools: Vec<Vec<Option<bool>>> = vec![Vec::new(); columns.len() - n_num];
    for (i, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != header.len() {
            return Err(LoadError::format(
                path,
                format!(
                    "line {}: {} fields, header has {}",
                    i + 2,
                    fields.len(),
                    header.len()
                ),
            ));
        }
        objects.push(fields[0].to_string());
        let (mut nc, mut bc) = (0, 0);
        for ((col, kind), &cell) in columns.iter().zip(&fields[1..]) {
            match kind {
                Kind::Num => {
                    let (lo, hi) = parse_num(cell).map_err(|e| {
                        LoadError::format(path, format!("line {}, column {col}: {e}", i + 2))
                    })?;
                    lower[nc].push(lo);
                    upper[nc].push(hi);
                    nc += 1;
                }
                Kind::Bool => {
                    bools[bc].push(match cell {
                        "1" => Some(true),
                        "0" => Some(false),
                        "" => None,
                        other => {
                            return Err(LoadError::format(
                                path,
                                format!("line {}, column {col}: bad boolean {other:?}", i + 2),
                            ))
                        }
                    });
                    bc += 1;
                }
            }
        }
    }

    let n = objects.len();
    let num_names: Vec<String> = columns
        .iter()
        .filter(|(_, k)| matches!(k, Kind::Num))
        .map(|(c, _)| c.clone())
        .collect();
    let bool_names: Vec<String> = columns
        .iter()
        .filter(|(_, k)| matches!(k, Kind::Bool))
        .map(|(c, _)| c.clone())
        .collect();
    let numeric =
        (!num_names.is_empty()).then(|| NumericBlock::from_intervals(num_names, lower, upper));
    let boolean = (!bool_names.is_empty()).then(|| {
        let mut values = PackedColumns::zeros(n, bool_names.len());
        let mut known = PackedColumns::zeros(n, bool_names.len());
        for (c, col) in bools.iter().enumerate() {
            for (r, v) in col.iter().enumerate() {
                if let Some(v) = v {
                    known.set(r, c);
                    if *v {
                        values.set(r, c);
                    }
                }
            }
        }
        BooleanBlock::with_known(bool_names, values, known)
    });

    let file = path.file_name().map_or_else(
        || path.display().to_string(),
        |f| f.to_string_lossy().into_owned(),
    );
    Ok(Table::new(
        name,
        objects,
        numeric,
        boolean,
        vec![],
        vec![SourceFile {
            file,
            sha256: sha256_hex(&bytes),
        }],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells() {
        assert_eq!(parse_num("3").unwrap(), (3.0, 3.0));
        assert_eq!(parse_num("997/60").unwrap(), (997.0 / 60.0, 997.0 / 60.0));
        assert_eq!(parse_num("2.0..4.0").unwrap(), (2.0, 4.0));
        let (lo, hi) = parse_num("").unwrap();
        assert!(lo.is_nan() && hi.is_nan());
        assert!(parse_num("4..2").is_err());
        assert!(parse_num("inf").is_err());
        assert!(parse_num("1/0").is_err());
    }
}
