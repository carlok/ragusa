//! Gate G-B6: the frontier tables exported to ragusa CSV load exactly as their manifests say.
//!
//! The exports live in the git-ignored `external/` folder (KnotInfo's maintainers ask that the
//! database not be reposted, so it is downloaded from knotinfo.org rather than shipped), so
//! these tests are ignored by default. Run them after the exporters with `make frontier-gates`, or
//! `cargo test -p ragusa-table --test g_b6 -- --ignored`. When run, a missing export is a failure.

use std::fs;

use ragusa_table::{sources, BoolColumns, Table};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hash_f64(values: &[f64]) -> String {
    let mut h = Sha256::new();
    for v in values {
        let v = if v.is_nan() {
            f64::from_bits(0x7ff8_0000_0000_0000)
        } else {
            *v
        };
        h.update(v.to_bits().to_le_bytes());
    }
    hex(&h.finalize())
}

fn check(name: &str) -> Vec<String> {
    let csv = sources::external_path(name).unwrap();
    let manifest_path = csv.with_extension("manifest.json");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(&manifest_path)
            .unwrap_or_else(|e| panic!("{}: {e} (run the exporter)", manifest_path.display())),
    )
    .unwrap();
    let table: Table = sources::load_standard(name).unwrap_or_else(|e| panic!("{name}: {e}"));
    let mut problems = Vec::new();

    if manifest["file_sha256"].as_str() != Some(table.sources()[0].sha256.as_str()) {
        problems.push(format!("{name}: file bytes differ from the manifest"));
    }
    if manifest["n_objects"].as_u64() != Some(table.n_objects() as u64) {
        problems.push(format!(
            "{name}: {} objects, manifest {}",
            table.n_objects(),
            manifest["n_objects"]
        ));
    }
    let names_hash = hex(&Sha256::digest(table.objects().join("\n").as_bytes()));
    if manifest["names_sha256"].as_str() != Some(names_hash.as_str()) {
        problems.push(format!("{name}: object names differ"));
    }

    let numeric = table.numeric();
    let boolean = table.boolean();
    for col in manifest["columns"].as_array().unwrap() {
        let cname = col["name"].as_str().unwrap();
        match col["kind"].as_str().unwrap() {
            "num" => {
                let Some(b) = numeric else {
                    problems.push(format!("{name}.{cname}: no numeric block"));
                    continue;
                };
                let Some(c) = b.names().iter().position(|n| n == cname) else {
                    problems.push(format!("{name}.{cname}: missing numeric column"));
                    continue;
                };
                let (lo, hi) = (b.lower(c), b.upper(c));
                let missing = lo.iter().filter(|v| v.is_nan()).count() as u64;
                let points = lo
                    .iter()
                    .zip(hi)
                    .filter(|(l, h)| !l.is_nan() && l.to_bits() == h.to_bits())
                    .count() as u64;
                let intervals = lo.len() as u64 - missing - points;
                if col["n_missing"].as_u64() != Some(missing)
                    || col["n_point"].as_u64() != Some(points)
                    || col["n_interval"].as_u64() != Some(intervals)
                {
                    problems.push(format!("{name}.{cname}: counts (point {points}, interval {intervals}, missing {missing}) differ"));
                }
                if col["sha256_lower"].as_str() != Some(hash_f64(lo).as_str()) {
                    problems.push(format!("{name}.{cname}: lower values differ"));
                }
                if col["sha256_upper"].as_str() != Some(hash_f64(hi).as_str()) {
                    problems.push(format!("{name}.{cname}: upper values differ"));
                }
            }
            "bool" => {
                let Some(b) = boolean else {
                    problems.push(format!("{name}.{cname}: no boolean block"));
                    continue;
                };
                let Some(c) = b.names().iter().position(|n| n == cname) else {
                    problems.push(format!("{name}.{cname}: missing boolean column"));
                    continue;
                };
                let codes: Vec<u8> = (0..table.n_objects())
                    .map(|r| {
                        if !b.is_known(r, c) {
                            2
                        } else {
                            u8::from(b.bits().get(r, c))
                        }
                    })
                    .collect();
                let n_true = codes.iter().filter(|&&x| x == 1).count() as u64;
                let n_false = codes.iter().filter(|&&x| x == 0).count() as u64;
                let n_missing = codes.iter().filter(|&&x| x == 2).count() as u64;
                if col["n_true"].as_u64() != Some(n_true)
                    || col["n_false"].as_u64() != Some(n_false)
                    || col["n_missing"].as_u64() != Some(n_missing)
                    || col["sha256"].as_str() != Some(hex(&Sha256::digest(&codes)).as_str())
                {
                    problems.push(format!("{name}.{cname}: boolean values differ"));
                }
            }
            other => problems.push(format!("{name}.{cname}: unknown kind {other}")),
        }
    }
    problems
}

#[test]
#[ignore = "needs the git-ignored external/ exports; run `make frontier-gates`"]
fn external_tables_match_their_manifests() {
    let problems: Vec<String> = sources::external_names()
        .into_iter()
        .flat_map(check)
        .collect();
    assert!(
        problems.is_empty(),
        "G-B6 mismatches ({}):\n{}",
        problems.len(),
        problems.join("\n")
    );
}
