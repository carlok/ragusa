//! The floor report for implication conjectures on a boolean-only table, measured against a
//! complete known-results oracle (the ETP implication graph).
//!
//! Stages: `refutation_by_table` (validity kind) splits every ordered law pair into emitted and
//! refuted, with per-object attribution; `etp_oracle` (novelty kind) splits the emitted pairs into
//! known true and known false. Soundness: no pair the oracle knows to be true may be refuted by an
//! object. Chance baseline: the oracle's true rate among all ordered pairs, against its true rate
//! among the emitted ones.

use std::collections::HashSet;
use std::fmt::Write as _;

use ragusa_gen::implications::{scan, ImplicationScan};
use ragusa_oracle::etp::EtpImplications;
use ragusa_table::{BoolColumns, Pin, Table};
use rayon::prelude::*;
use serde_json::{json, Value};

pub const FORMAT: &str = "ragusa-implication-floor-report/1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OracleCounts {
    pub true_pairs: u64,
    pub false_pairs: u64,
    pub emitted_true: u64,
    pub emitted_false: u64,
    pub refuted_true: u64,
    pub refuted_false: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardCore {
    pub pairs: usize,
    pub refuted: usize,
}

pub struct ImplicationReport {
    pub pin: Pin,
    pub objects: Vec<String>,
    pub scan: ImplicationScan,
    pub distinct_law_columns: usize,
    pub oracle: OracleCounts,
    pub hard_core: Option<HardCore>,
    pub engine_git_rev: Option<String>,
}

pub fn run_implications(
    table: &Table,
    oracle: &EtpImplications,
    hard_core: Option<&[(usize, usize)]>,
) -> Result<ImplicationReport, String> {
    let boolean = table.boolean().ok_or("table has no boolean block")?;
    let n = boolean.names().len();
    if n != oracle.n() {
        return Err(format!(
            "{n} boolean columns, oracle covers {} laws",
            oracle.n()
        ));
    }
    if let Some((k, name)) = boolean
        .names()
        .iter()
        .enumerate()
        .find(|(k, name)| **name != format!("E{}", k + 1))
    {
        return Err(format!("column {k} is {name:?}, expected E{}", k + 1));
    }

    let bits = boolean.bits();
    let scan = scan(bits, boolean.known_bits());
    let distinct_law_columns = (0..n)
        .map(|c| bits.column_words(c))
        .collect::<HashSet<_>>()
        .len();

    let oracle_counts = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut c = OracleCounts::default();
            for j in (0..n).filter(|&j| j != i) {
                match (oracle.holds(i, j), scan.emitted(i, j)) {
                    (true, true) => c.emitted_true += 1,
                    (true, false) => c.refuted_true += 1,
                    (false, true) => c.emitted_false += 1,
                    (false, false) => c.refuted_false += 1,
                }
            }
            c.true_pairs = c.emitted_true + c.refuted_true;
            c.false_pairs = c.emitted_false + c.refuted_false;
            c
        })
        .reduce(OracleCounts::default, |a, b| OracleCounts {
            true_pairs: a.true_pairs + b.true_pairs,
            false_pairs: a.false_pairs + b.false_pairs,
            emitted_true: a.emitted_true + b.emitted_true,
            emitted_false: a.emitted_false + b.emitted_false,
            refuted_true: a.refuted_true + b.refuted_true,
            refuted_false: a.refuted_false + b.refuted_false,
        });

    let hard_core = hard_core.map(|pairs| HardCore {
        pairs: pairs.len(),
        refuted: pairs.iter().filter(|&&(i, j)| !scan.emitted(i, j)).count(),
    });

    Ok(ImplicationReport {
        pin: Pin::of(table),
        objects: table.objects().to_vec(),
        scan,
        distinct_law_columns,
        oracle: oracle_counts,
        hard_core,
        engine_git_rev: None,
    })
}

fn ratio(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        a as f64 / b as f64
    }
}

impl ImplicationReport {
    pub fn sound(&self) -> bool {
        self.oracle.refuted_true == 0
    }

    fn ranked_objects(&self) -> Vec<(usize, u64)> {
        let mut ranked: Vec<(usize, u64)> = self
            .scan
            .unique_refutations
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, u)| u > 0)
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        ranked
    }

    pub fn to_json(&self) -> Value {
        let o = &self.oracle;
        let emitted = self.scan.emitted_count();
        let pairs = self.scan.pairs();
        json!({
            "format": FORMAT,
            "pin": self.pin.to_json(),
            "engine": {"version": env!("CARGO_PKG_VERSION"), "git_rev": self.engine_git_rev},
            "laws": self.scan.n(),
            "distinct_law_columns": self.distinct_law_columns,
            "candidates": pairs,
            "stages": [
                {
                    "filter": "refutation_by_table",
                    "kind": "validity",
                    "input": pairs,
                    "representative": emitted,
                    "alias": 0,
                    "discarded": self.scan.refuted,
                    "floor": {
                        "fragile": self.scan.refuted_once,
                        "objects_with_flips": self.scan.unique_refutations.iter().filter(|&&u| u > 0).count(),
                        "top_objects": self.ranked_objects().iter().take(10)
                            .map(|&(r, u)| json!({"object": self.objects[r], "flips": u})).collect::<Vec<_>>(),
                        "flips_by_object": self.scan.unique_refutations,
                    },
                },
                {
                    "filter": "etp_oracle",
                    "kind": "novelty",
                    "input": emitted,
                    "known_true": o.emitted_true,
                    "known_false": o.emitted_false,
                },
            ],
            "soundness": {"refuted_but_known_true": o.refuted_true, "pass": self.sound()},
            "oracle": {
                "true_pairs": o.true_pairs,
                "false_pairs": o.false_pairs,
                "false_pairs_refuted_by_table": o.refuted_false,
            },
            "chance": {
                "known_true_rate_all_pairs": ratio(o.true_pairs, pairs),
                "known_true_rate_emitted": ratio(o.emitted_true, emitted),
            },
            "hard_core": self.hard_core.map(|h| json!({"pairs": h.pairs, "refuted_by_table": h.refuted})),
        })
    }

    pub fn to_markdown(&self) -> String {
        let o = &self.oracle;
        let emitted = self.scan.emitted_count();
        let pairs = self.scan.pairs();
        let mut md = String::new();
        let _ = writeln!(
            md,
            "# ragusa implication floor report: `{}`\n",
            self.pin.table
        );
        let _ = writeln!(
            md,
            "- table content sha256 `{}`, {} objects, {} laws ({} distinct columns on these objects)",
            self.pin.content_sha256,
            self.pin.n_objects,
            self.scan.n(),
            self.distinct_law_columns
        );
        let _ = writeln!(
            md,
            "- engine {} (git {}); oracle: ETP implication graph\n",
            env!("CARGO_PKG_VERSION"),
            self.engine_git_rev.as_deref().unwrap_or("unknown")
        );
        let _ = writeln!(md, "## Stages\n");
        let _ = writeln!(md, "| stage | kind | input | kept | removed |");
        let _ = writeln!(md, "|---|---|---:|---:|---:|");
        let _ = writeln!(
            md,
            "| `refutation_by_table` | validity | {pairs} | {emitted} emitted | {} refuted |",
            self.scan.refuted
        );
        let _ = writeln!(
            md,
            "| `etp_oracle` | novelty | {emitted} | {} known false | {} known true |\n",
            o.emitted_false, o.emitted_true
        );
        let _ = writeln!(
            md,
            "Soundness: {} pairs refuted by the table are true in the oracle ({}).\n",
            o.refuted_true,
            if self.sound() { "pass" } else { "**FAIL**" }
        );
        let _ = writeln!(md, "## Chance baseline\n");
        let _ = writeln!(
            md,
            "Known-true rate: {:.4} over all {pairs} ordered pairs, {:.4} over the {emitted} emitted.\n",
            ratio(o.true_pairs, pairs),
            ratio(o.emitted_true, emitted)
        );
        if let Some(h) = self.hard_core {
            let _ = writeln!(
                md,
                "Hard core: {} of {} pairs refuted by the table.\n",
                h.refuted, h.pairs
            );
        }
        let _ = writeln!(md, "## Objects that decide the most\n");
        let _ = writeln!(
            md,
            "{} pairs are refuted by exactly one object; removing that object would emit them.\n",
            self.scan.refuted_once
        );
        let _ = writeln!(md, "| object | pairs it alone refutes |");
        let _ = writeln!(md, "|---|---:|");
        for &(r, u) in self.ranked_objects().iter().take(10) {
            let _ = writeln!(md, "| {} | {u} |", self.objects[r]);
        }
        md
    }
}
