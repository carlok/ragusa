//! The pinning block every run carries: which table, which bytes, which engine.

use serde_json::{json, Value};

use crate::table::{SourceFile, Table};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    pub table: String,
    pub content_sha256: String,
    pub n_objects: usize,
    pub n_numeric: usize,
    pub n_boolean: usize,
    pub skipped: Vec<String>,
    pub sources: Vec<SourceFile>,
    pub engine_version: &'static str,
}

impl Pin {
    pub fn of(table: &Table) -> Pin {
        Pin {
            table: table.name().to_string(),
            content_sha256: table.content_sha256(),
            n_objects: table.n_objects(),
            n_numeric: table.numeric().map_or(0, |b| b.n_columns()),
            n_boolean: table.boolean().map_or(0, |b| b.names().len()),
            skipped: table.skipped().to_vec(),
            sources: table.sources().to_vec(),
            engine_version: env!("CARGO_PKG_VERSION"),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "table": self.table,
            "content_sha256": self.content_sha256,
            "n_objects": self.n_objects,
            "n_numeric": self.n_numeric,
            "n_boolean": self.n_boolean,
            "skipped": self.skipped,
            "sources": self.sources.iter().map(|s| json!({"file": s.file, "sha256": s.sha256})).collect::<Vec<_>>(),
            "engine_version": self.engine_version,
        })
    }
}
