//! Gate G-B1: every standard table loads into the one table type, the TxGraffiti tables are
//! typed exactly as pandas + txgraffiti type them, and content hashes match the committed pins.
//!
//! Numeric values must be bit-identical to pandas' correctly rounded parse
//! (`float_precision="round_trip"`). pandas' default parser, which txgraffiti computes on,
//! misrounds some long decimals; those deviations are recorded in the manifest and printed
//! here, not treated as loader errors.
//!
//! `fixtures/txgraffiti_manifest.json` comes from `calibration/src/ragusa_a/export_manifest.py`;
//! `fixtures/pins.json` from `ragusa pins --write`.

use std::fs;

use ragusa_table::{parsimagma::ETP_LAWS, sources, BoolColumns, Pin, Table};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn fixture(name: &str) -> Value {
    let path = sources::engine_dir().join("fixtures").join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn numeric_hash(col: &[f64]) -> String {
    let mut h = Sha256::new();
    for v in col {
        let v = if v.is_nan() {
            f64::from_bits(0x7ff8_0000_0000_0000)
        } else {
            *v
        };
        h.update(v.to_bits().to_le_bytes());
    }
    hex(&h.finalize())
}

fn bool_hash(t: &Table, c: usize) -> String {
    let bits = t.boolean().unwrap().bits();
    let bytes: Vec<u8> = (0..t.n_objects())
        .map(|r| u8::from(bits.get(r, c)))
        .collect();
    hex(&Sha256::digest(&bytes))
}

fn names(spec: &Value) -> Vec<&str> {
    spec.as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect()
}

#[test]
fn txgraffiti_tables_are_typed_like_pandas() {
    let manifest = fixture("txgraffiti_manifest.json");
    let mut problems = Vec::new();
    let mut deviations = 0usize;
    for (name, spec) in manifest["tables"].as_object().unwrap() {
        let t = sources::load_standard(name).unwrap_or_else(|e| panic!("{name}: {e}"));
        if spec["csv_sha256"].as_str() != Some(t.sources()[0].sha256.as_str()) {
            problems.push(format!(
                "{name}: CSV bytes differ from the manifest (package changed?)"
            ));
            continue;
        }
        if spec["n_rows"].as_u64() != Some(t.n_objects() as u64) {
            problems.push(format!(
                "{name}: {} rows, pandas {}",
                t.n_objects(),
                spec["n_rows"]
            ));
        }

        let want_bool = names(&spec["boolean"]);
        let got_bool: Vec<&str> = t
            .boolean()
            .map_or(vec![], |b| b.names().iter().map(String::as_str).collect());
        let want_num = names(&spec["numeric"]);
        let got_num: Vec<&str> = t
            .numeric()
            .map_or(vec![], |b| b.names().iter().map(String::as_str).collect());
        let want_skip: Vec<&str> = spec["skipped"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let got_skip: Vec<&str> = t.skipped().iter().map(String::as_str).collect();
        if want_bool != got_bool {
            problems.push(format!(
                "{name}: boolean columns {got_bool:?}, pandas {want_bool:?}"
            ));
        }
        if want_num != got_num {
            problems.push(format!(
                "{name}: numeric columns {got_num:?}, pandas {want_num:?}"
            ));
        }
        if want_skip != got_skip {
            problems.push(format!(
                "{name}: skipped {got_skip:?}, pandas {want_skip:?}"
            ));
        }
        if want_bool != got_bool || want_num != got_num {
            continue;
        }

        for (c, col) in spec["boolean"].as_array().unwrap().iter().enumerate() {
            let bits = t.boolean().unwrap().bits();
            if col["n_true"].as_u64() != Some(bits.count_ones(c))
                || col["sha256"].as_str() != Some(bool_hash(&t, c).as_str())
            {
                problems.push(format!("{name}.{}: boolean values differ", col["name"]));
            }
        }
        for (c, col) in spec["numeric"].as_array().unwrap().iter().enumerate() {
            let values = t.numeric().unwrap().column(c);
            let n_nan = values.iter().filter(|v| v.is_nan()).count() as u64;
            if col["n_nan"].as_u64() != Some(n_nan) {
                problems.push(format!(
                    "{name}.{}: {n_nan} NaN, pandas {}",
                    col["name"], col["n_nan"]
                ));
            } else if col["sha256_round_trip"].as_str() != Some(numeric_hash(values).as_str()) {
                problems.push(format!(
                    "{name}.{}: not bit-identical to pandas round_trip parse",
                    col["name"]
                ));
            } else if col["n_default_parse_differs"].as_u64() != Some(0) {
                deviations += 1;
                eprintln!(
                    "note: {name}.{}: pandas default parser differs on {} values (max {} ulps); loader matches round_trip",
                    col["name"], col["n_default_parse_differs"], col["max_default_parse_ulps"]
                );
            }
        }
    }
    eprintln!("G-B1: {deviations} columns where pandas' default parser misrounds (loader is exact against round_trip)");
    assert!(
        problems.is_empty(),
        "G-B1 mismatches ({}):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn magma_corpus_loads_as_boolean_only_table() {
    let t = sources::load_standard(sources::MAGMA_CORPUS).unwrap();
    assert!(t.numeric().is_none());
    let b = t.boolean().expect("boolean block");
    assert_eq!(b.names().len(), ETP_LAWS);
    assert_eq!(b.names()[0], "E1");

    // Recomputed from the index file here, independently of the loader's own check.
    let index = fs::read_to_string(sources::parsimagma_dir().join("corpus_index.tsv")).unwrap();
    assert_eq!(t.n_objects(), index.lines().count() - 1);
    for (r, line) in index.lines().skip(1).enumerate().step_by(97) {
        let laws: usize = line.rsplit('\t').next().unwrap().parse().unwrap();
        let row_count = (0..ETP_LAWS).filter(|&c| b.bits().get(r, c)).count();
        assert_eq!(row_count, laws, "row {r}");
    }

    // ETP equation 1 is x = x, which every magma satisfies. Equation 2 is x = y, which only
    // the trivial magma satisfies, and parsimagma drops magmas that satisfy every law.
    assert_eq!(b.bits().count_ones(0), t.n_objects() as u64);
    assert_eq!(b.bits().count_ones(1), 0);
}

#[test]
fn every_standard_table_matches_committed_pin() {
    let pins = fixture("pins.json");
    for name in sources::standard_names() {
        let pin = Pin::of(&sources::load_standard(name).unwrap());
        let want = &pins[name];
        assert_eq!(
            want["content_sha256"].as_str(),
            Some(pin.content_sha256.as_str()),
            "{name}: content changed"
        );
        assert_eq!(
            want["n_objects"].as_u64(),
            Some(pin.n_objects as u64),
            "{name}"
        );
        assert_eq!(
            want["n_numeric"].as_u64(),
            Some(pin.n_numeric as u64),
            "{name}"
        );
        assert_eq!(
            want["n_boolean"].as_u64(),
            Some(pin.n_boolean as u64),
            "{name}"
        );
    }
}

#[test]
fn content_hash_is_deterministic_across_loads() {
    let a = sources::load_standard("graph_data")
        .unwrap()
        .content_sha256();
    let b = sources::load_standard("graph_data")
        .unwrap()
        .content_sha256();
    assert_eq!(a, b);
}
