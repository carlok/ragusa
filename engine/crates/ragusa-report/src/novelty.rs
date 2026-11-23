//! The novelty stage for knot tables: survivors classified against the catalogue, next to the
//! classes of every generated candidate as the baseline, behind the catalogue gate.
//!
//! "Implied" is a sufficient test (see [`crate::knots::implies`]); the other classes say only what
//! this catalogue contains. A survivor in `new_pair` is not claimed to be new to the literature.

use std::fmt::Write as _;

use serde_json::{json, Value};

use crate::knots::{CatalogueCheck, Novelty, NoveltyOracle};
use crate::Report;

pub const FORMAT: &str = "ragusa-novelty-report/1";

const CLASSES: [&str; 4] = [
    "implied",
    "same_pair_not_implied",
    "new_pair",
    "outside_catalogue",
];

fn slot(n: Novelty) -> usize {
    match n {
        Novelty::Implied(_) => 0,
        Novelty::SamePair => 1,
        Novelty::NewPair => 2,
        Novelty::Outside => 3,
    }
}

pub struct NoveltyStage {
    /// (candidate index, class) for every survivor.
    pub survivors: Vec<(usize, Novelty)>,
    pub all_counts: [usize; 4],
    pub survivor_counts: [usize; 4],
    pub gate_bounds: usize,
    pub gate_checked: usize,
    pub gate_refuted: usize,
    pub gate_objects: usize,
}

impl NoveltyStage {
    pub fn gate_pass(&self) -> bool {
        self.gate_refuted == 0
    }
}

pub fn novelty_stage(
    report: &Report,
    check: &CatalogueCheck,
    oracle: &NoveltyOracle,
) -> NoveltyStage {
    let ev = &report.evaluated;
    let mut all_counts = [0; 4];
    for i in 0..ev.len() {
        all_counts[slot(oracle.classify(ev.bound(i)))] += 1;
    }
    let survivors: Vec<(usize, Novelty)> = report
        .survivors()
        .into_iter()
        .map(|i| (i, oracle.classify(ev.bound(i))))
        .collect();
    let mut survivor_counts = [0; 4];
    for &(_, n) in &survivors {
        survivor_counts[slot(n)] += 1;
    }
    NoveltyStage {
        survivors,
        all_counts,
        survivor_counts,
        gate_bounds: check.bounds.len(),
        gate_checked: check.checks.iter().flatten().count(),
        gate_refuted: check.refuted(),
        gate_objects: check.non_trivial_objects,
    }
}

fn counts_json(counts: &[usize; 4]) -> Value {
    CLASSES
        .iter()
        .zip(counts)
        .map(|(c, n)| ((*c).to_string(), json!(n)))
        .collect::<serde_json::Map<_, _>>()
        .into()
}

impl NoveltyStage {
    pub fn to_json(&self, report: &Report, oracle: &NoveltyOracle, citation: &str) -> Value {
        let survivors: Vec<Value> = self
            .survivors
            .iter()
            .map(|&(i, n)| {
                let implied_by = match n {
                    Novelty::Implied(k) => {
                        let c = &oracle.bounds[k];
                        json!({"bound": c.bound.to_string(), "derivation": c.derivation, "likely_mapping": c.likely})
                    }
                    _ => Value::Null,
                };
                json!({
                    "index": i,
                    "bound": report.evaluated.bound(i).to_string(),
                    "class": n.label(),
                    "implied_by": implied_by,
                })
            })
            .collect();
        json!({
            "format": FORMAT,
            "pin": report.pin.to_json(),
            "engine": {"version": env!("CARGO_PKG_VERSION"), "git_rev": report.engine_git_rev},
            "target": report.spec.target,
            "catalogue": {
                "citation": citation,
                "bounds": self.gate_bounds,
                "checked_on_table": self.gate_checked,
                "refuted_by_table": self.gate_refuted,
                "non_trivial_objects": self.gate_objects,
                "pass": self.gate_pass(),
            },
            "classes": {
                "all_candidates": counts_json(&self.all_counts),
                "survivors": counts_json(&self.survivor_counts),
            },
            "survivors": survivors,
        })
    }

    pub fn to_markdown(&self, report: &Report, oracle: &NoveltyOracle, citation: &str) -> String {
        let mut md = String::new();
        let _ = writeln!(
            md,
            "# ragusa novelty report: `{}`, target `{}`\n",
            report.pin.table, report.spec.target
        );
        let _ = writeln!(
            md,
            "- table content sha256 `{}`; engine {} (git {})",
            report.pin.content_sha256,
            env!("CARGO_PKG_VERSION"),
            report.engine_git_rev.as_deref().unwrap_or("unknown")
        );
        let _ = writeln!(md, "- oracle: {citation}");
        let _ = writeln!(
            md,
            "- catalogue gate: {} of {} bounds checkable on this table, {} refuted on {} non-trivial knots ({})\n",
            self.gate_checked,
            self.gate_bounds,
            self.gate_refuted,
            self.gate_objects,
            if self.gate_pass() { "pass" } else { "**FAIL**: classes below are not trustworthy" }
        );
        let _ = writeln!(md, "## Classes\n");
        let _ = writeln!(
            md,
            "\"implied\" is a sufficient test; \"new_pair\" means only that this catalogue does not relate the two columns.\n"
        );
        let _ = writeln!(md, "| class | all candidates | survivors |");
        let _ = writeln!(md, "|---|---:|---:|");
        for (k, c) in CLASSES.iter().enumerate() {
            let _ = writeln!(
                md,
                "| {c} | {} | {} |",
                self.all_counts[k], self.survivor_counts[k]
            );
        }
        for (k, c) in CLASSES.iter().enumerate() {
            let rows: Vec<&(usize, Novelty)> = self
                .survivors
                .iter()
                .filter(|(_, n)| slot(*n) == k)
                .collect();
            if rows.is_empty() {
                continue;
            }
            let _ = writeln!(md, "\n## Survivors: {c}\n");
            for &&(i, n) in &rows {
                let _ = write!(md, "- `{}`", report.evaluated.bound(i));
                if let Novelty::Implied(b) = n {
                    let cb = &oracle.bounds[b];
                    let _ = write!(
                        md,
                        " — by `{}` [{}{}]",
                        cb.bound,
                        cb.derivation,
                        if cb.likely { ", likely mapping" } else { "" }
                    );
                }
                md.push('\n');
            }
        }
        md
    }
}

#[cfg(test)]
mod tests {
    use ragusa_oracle::knots::Catalogue;
    use ragusa_table::{BooleanBlock, NumericBlock, PackedColumns, Table};

    use super::*;
    use crate::knots::check_catalogue;
    use crate::{run, Generator, RunSpec};

    #[test]
    fn stage_counts_are_consistent() {
        let mut bits = PackedColumns::zeros(5, 1);
        for r in [0, 2, 4] {
            bits.set(r, 0);
        }
        let table = Table::new(
            "knots",
            ["3_1", "4_1", "5_1", "5_2", "6_1"]
                .map(String::from)
                .to_vec(),
            Some(NumericBlock::from_columns(
                vec![
                    "crossing_number".into(),
                    "unknotting_number".into(),
                    "volume".into(),
                ],
                vec![
                    vec![3.0, 4.0, 5.0, 5.0, 6.0],
                    vec![1.0, 1.0, 2.0, 1.0, 1.0],
                    vec![0.0, 2.03, 0.0, 2.83, 3.16],
                ],
            )),
            Some(BooleanBlock::new(vec!["alternating".into()], bits)),
            vec![],
            vec![],
        );
        let mut spec = RunSpec::full(&table, "unknotting_number").unwrap();
        spec.generators = vec![Generator::Ratios];
        spec.chance_draws = 0;
        let report = run(&table, spec).unwrap();
        let catalogue = Catalogue::standard().unwrap();
        let check = check_catalogue(&table, &catalogue).unwrap();
        let oracle = NoveltyOracle::new(&catalogue);
        let stage = novelty_stage(&report, &check, &oracle);

        assert!(stage.gate_pass());
        assert_eq!(
            stage.all_counts.iter().sum::<usize>(),
            report.evaluated.len()
        );
        assert_eq!(
            stage.survivor_counts.iter().sum::<usize>(),
            report.survivors().len()
        );
        // volume is not a catalogue column
        assert!(stage.all_counts[3] > 0);
        let json = stage.to_json(&report, &oracle, &catalogue.citation);
        assert_eq!(json["format"], FORMAT);
        assert_eq!(
            json["survivors"].as_array().unwrap().len(),
            report.survivors().len()
        );
        assert!(stage
            .to_markdown(&report, &oracle, &catalogue.citation)
            .contains("| implied |"));
    }
}
