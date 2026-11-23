//! The standard tables and where their bytes live, so the CLI and the tests load the same files.

use std::path::PathBuf;

use crate::{parsimagma, ragusa_csv, txgraffiti, Table};

/// Frontier tables exported to ragusa CSV under the git-ignored `external/` folder by
/// `calibration/src/ragusa_a/export_*.py`: (table name, path below the external folder).
/// They are not in [`standard_names`], so committed tests never need them; gate G-B6 does.
pub const EXTERNAL_TABLES: &[(&str, &str)] = &[
    ("knotinfo_le12", "knotinfo/export/knotinfo_le12.csv"),
    ("knotinfo_le13", "knotinfo/export/knotinfo_le13.csv"),
    (
        "knotinfo_le13_mirror",
        "knotinfo/export/knotinfo_le13_mirror.csv",
    ),
    ("fano3c", "fano3c/export/fano3c.csv"),
];

/// The external data folder: `RAGUSA_EXTERNAL` if set, otherwise `<repo>/external`.
pub fn external_dir() -> PathBuf {
    std::env::var("RAGUSA_EXTERNAL")
        .map_or_else(|_| engine_dir().join("../external"), PathBuf::from)
}

pub fn external_names() -> Vec<&'static str> {
    EXTERNAL_TABLES.iter().map(|(name, _)| *name).collect()
}

pub fn external_path(name: &str) -> Option<PathBuf> {
    EXTERNAL_TABLES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, file)| external_dir().join(file))
}

/// The `engine/` directory of the repository.
pub const ENGINE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// TxGraffiti 0.4.1's shipped tables: (table name, CSV file in `example_data`).
pub const TXGRAFFITI_TABLES: &[(&str, &str)] = &[
    ("graph_data", "graph_data.csv"),
    ("integer_data", "integer_data.csv"),
    ("polytope_data", "polytope_data.csv"),
    ("qubits_data", "Nqubits_data.csv"),
    ("calabi_yau_data", "calabi_yau_data.csv"),
    ("nba_game_data", "nba_game_data.csv"),
];

/// parsimagma's corpus over the 4694 ETP laws, vendored under `engine/data/parsimagma`.
pub const MAGMA_CORPUS: &str = "magma_corpus_etp4694";

pub fn engine_dir() -> PathBuf {
    PathBuf::from(ENGINE_DIR)
}

pub fn parsimagma_dir() -> PathBuf {
    engine_dir().join("data/parsimagma")
}

/// TxGraffiti's `example_data` directory: `RAGUSA_TXGRAFFITI_DATA` if set, otherwise the
/// package installed in the calibration environment (`calibration/.venv`, pinned by uv.lock).
pub fn txgraffiti_data_dir() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("RAGUSA_TXGRAFFITI_DATA") {
        return Ok(PathBuf::from(p));
    }
    let lib = engine_dir().join("../calibration/.venv/lib");
    let entries = std::fs::read_dir(&lib).map_err(|e| {
        format!(
            "{}: {e} (run `uv sync` in calibration/ or set RAGUSA_TXGRAFFITI_DATA)",
            lib.display()
        )
    })?;
    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("python3"))
        })
        .map(|p| p.join("site-packages/txgraffiti/example_data"))
        .filter(|p| p.is_dir())
        .collect();
    found.sort();
    found
        .pop()
        .ok_or_else(|| format!("no txgraffiti example_data under {}", lib.display()))
}

pub fn standard_names() -> Vec<&'static str> {
    TXGRAFFITI_TABLES
        .iter()
        .map(|(name, _)| *name)
        .chain([MAGMA_CORPUS])
        .collect()
}

pub fn load_standard(name: &str) -> Result<Table, String> {
    if name == MAGMA_CORPUS {
        return parsimagma::load_corpus(&parsimagma_dir(), name).map_err(|e| e.to_string());
    }
    if let Some(path) = external_path(name) {
        return ragusa_csv::load(&path, name).map_err(|e| e.to_string());
    }
    let (_, file) = TXGRAFFITI_TABLES
        .iter()
        .find(|(n, _)| *n == name)
        .ok_or_else(|| {
            format!(
                "unknown table {name:?}; standard tables: {}",
                standard_names().join(", ")
            )
        })?;
    txgraffiti::load_csv(&txgraffiti_data_dir()?.join(file), name).map_err(|e| e.to_string())
}
