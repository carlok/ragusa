//! Loader for TxGraffiti's shipped CSV tables, typed the way `pandas.read_csv`,
//! `txgraffiti.example_data` and `processing.auto_wrap` type them.
//!
//! Gate G-B1 (`tests/g_b1.rs`) checks column names, order and every value hash against
//! `fixtures/txgraffiti_manifest.json`, which pandas itself produced. The rules:
//!
//! - an empty first header (pandas' `Unnamed: 0` index column) is dropped;
//! - a column named `name` is skipped, as `auto_wrap` skips it;
//! - boolean: every field is `True`/`TRUE`/`true`/`False`/`FALSE`/`false`, none missing;
//! - numeric: every field is a pandas missing-value token or parses as `f64`; missing is NaN;
//! - anything else is skipped and listed in [`Table::skipped`].
//!
//! Fields are split on `,` with no quoting support; a file containing `"` is rejected rather
//! than guessed at.

use std::path::Path;

use crate::bits::PackedColumns;
use crate::error::LoadError;
use crate::table::{sha256_hex, BooleanBlock, NumericBlock, SourceFile, Table};

/// pandas' default `na_values`.
const NA_TOKENS: &[&str] = &[
    "", "#N/A", "#N/A N/A", "#NA", "-1.#IND", "-1.#QNAN", "-NaN", "-nan", "1.#IND", "1.#QNAN",
    "<NA>", "N/A", "NA", "NULL", "NaN", "None", "n/a", "nan", "null",
];
const TRUE_TOKENS: &[&str] = &["True", "TRUE", "true"];
const FALSE_TOKENS: &[&str] = &["False", "FALSE", "false"];

enum Kind {
    Boolean,
    Numeric,
    Skipped,
}

fn classify(fields: &[&str]) -> Kind {
    if !fields.is_empty()
        && fields
            .iter()
            .all(|f| TRUE_TOKENS.contains(f) || FALSE_TOKENS.contains(f))
    {
        Kind::Boolean
    } else if fields
        .iter()
        .all(|f| NA_TOKENS.contains(f) || f.parse::<f64>().is_ok())
    {
        Kind::Numeric
    } else {
        Kind::Skipped
    }
}

pub fn load_csv(path: &Path, name: &str) -> Result<Table, LoadError> {
    let bytes = std::fs::read(path).map_err(|e| LoadError::io(path, e))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|e| LoadError::format(path, format!("not UTF-8: {e}")))?;
    if text.contains('"') {
        return Err(LoadError::format(
            path,
            "quoted CSV fields are not supported; refusing to guess",
        ));
    }
    if text.contains('\r') {
        return Err(LoadError::format(
            path,
            "CRLF line endings are not supported",
        ));
    }

    let mut lines = text.lines();
    let header: Vec<&str> = lines
        .next()
        .ok_or_else(|| LoadError::format(path, "empty file"))?
        .split(',')
        .collect();
    let rows = lines
        .enumerate()
        .map(|(i, line)| {
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() == header.len() {
                Ok(fields)
            } else {
                Err(LoadError::format(
                    path,
                    format!(
                        "line {}: {} fields, header has {}",
                        i + 2,
                        fields.len(),
                        header.len()
                    ),
                ))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let n = rows.len();

    let mut numeric_names = Vec::new();
    let mut numeric_columns = Vec::new();
    let mut boolean_columns: Vec<(String, Vec<bool>)> = Vec::new();
    let mut skipped = Vec::new();
    for (c, &head) in header.iter().enumerate() {
        if c == 0 && (head.is_empty() || head == "Unnamed: 0") {
            continue;
        }
        if head.is_empty() {
            return Err(LoadError::format(path, format!("unnamed column {c}")));
        }
        if head == "name" {
            skipped.push(head.to_string());
            continue;
        }
        let fields: Vec<&str> = rows.iter().map(|r| r[c]).collect();
        match classify(&fields) {
            Kind::Boolean => {
                boolean_columns.push((
                    head.to_string(),
                    fields.iter().map(|f| TRUE_TOKENS.contains(f)).collect(),
                ));
            }
            Kind::Numeric => {
                numeric_names.push(head.to_string());
                numeric_columns.push(
                    fields
                        .iter()
                        .map(|f| {
                            if NA_TOKENS.contains(f) {
                                f64::NAN
                            } else {
                                f.parse().expect("classified numeric")
                            }
                        })
                        .collect(),
                );
            }
            Kind::Skipped => skipped.push(head.to_string()),
        }
    }

    let numeric = (!numeric_names.is_empty())
        .then(|| NumericBlock::from_columns(numeric_names, numeric_columns));
    let boolean = (!boolean_columns.is_empty()).then(|| {
        let mut bits = PackedColumns::zeros(n, boolean_columns.len());
        for (c, (_, values)) in boolean_columns.iter().enumerate() {
            for (r, &v) in values.iter().enumerate() {
                if v {
                    bits.set(r, c);
                }
            }
        }
        BooleanBlock::new(
            boolean_columns.into_iter().map(|(name, _)| name).collect(),
            bits,
        )
    });

    let file = path.file_name().map_or_else(
        || path.display().to_string(),
        |f| f.to_string_lossy().into_owned(),
    );
    Ok(Table::new(
        name,
        (0..n).map(|r| r.to_string()).collect(),
        numeric,
        boolean,
        skipped,
        vec![SourceFile {
            file,
            sha256: sha256_hex(&bytes),
        }],
    ))
}
