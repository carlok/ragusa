//! Held-out objects. The pipeline runs on the training objects only (generators, filters,
//! attribution and chance baseline never see the rest), and every candidate that passed validity
//! on training is then evaluated on the held-out objects with the engine's interval rules.
//!
//! The report gives, for each stage, how many of the candidates it kept are refuted on the held-out
//! objects, so the survivors' refutation rate can be read against the rate of everything valid on
//! training. It is reported, not forced: a refuted survivor is a finding about the table, and a
//! survivor that is never refuted is not thereby true.

use std::collections::HashMap;
use std::fmt::Write as _;

use ragusa_filter::{Evaluated, Verdict};
use ragusa_table::{Pin, Table};
use rand::seq::index::sample;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde_json::{json, Value};

use crate::Report;

pub const FORMAT: &str = "ragusa-heldout-report/1";

/// Counterexamples listed per survivor.
const EXAMPLES: usize = 5;

#[derive(Clone, Debug, PartialEq)]
pub enum Holdout {
    /// Objects whose exact value in `column` is at least `min` are held out.
    AtLeast { column: String, min: f64 },
    /// `round(fraction · n)` objects, drawn with a seeded ChaCha8 stream, are held out.
    Fraction { fraction: f64, seed: u64 },
}

impl Holdout {
    /// `column>=value`, or `fraction=F` with an optional `,seed=S` (default 0).
    pub fn parse(text: &str) -> Result<Holdout, String> {
        let bad = || {
            format!("--holdout: expected `column>=value` or `fraction=F[,seed=S]`, got {text:?}")
        };
        if let Some((column, min)) = text.split_once(">=") {
            let min: f64 = min.trim().parse().map_err(|_| bad())?;
            if column.trim().is_empty() || !min.is_finite() {
                return Err(bad());
            }
            return Ok(Holdout::AtLeast {
                column: column.trim().to_string(),
                min,
            });
        }
        let (mut fraction, mut seed) = (None, 0);
        for part in text.split(',') {
            match part.split_once('=') {
                Some(("fraction", v)) => fraction = Some(v.parse::<f64>().map_err(|_| bad())?),
                Some(("seed", v)) => seed = v.parse::<u64>().map_err(|_| bad())?,
                _ => return Err(bad()),
            }
        }
        match fraction {
            Some(f) if f > 0.0 && f < 1.0 => Ok(Holdout::Fraction { fraction: f, seed }),
            _ => Err(bad()),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Holdout::AtLeast { column, min } => format!("{column}>={min}"),
            Holdout::Fraction { fraction, seed } => format!("fraction={fraction},seed={seed}"),
        }
    }

    /// `(training rows, held-out rows)`, each ascending; an error if either is empty.
    pub fn split(&self, table: &Table) -> Result<(Vec<usize>, Vec<usize>), String> {
        let n = table.n_objects();
        let held: Vec<bool> = match self {
            Holdout::AtLeast { column, min } => {
                let numeric = table.numeric().ok_or("table has no numeric block")?;
                let c = numeric
                    .names()
                    .iter()
                    .position(|x| x == column)
                    .ok_or_else(|| format!("--holdout: unknown numeric column {column:?}"))?;
                let (lo, hi) = (numeric.lower(c), numeric.upper(c));
                if let Some(r) =
                    (0..n).find(|&r| lo[r].is_nan() || lo[r].to_bits() != hi[r].to_bits())
                {
                    return Err(format!(
                        "--holdout: {column} is not an exact value on {}",
                        table.objects()[r]
                    ));
                }
                lo.iter().map(|&v| v >= *min).collect()
            }
            Holdout::Fraction { fraction, seed } => {
                let k = ((fraction * n as f64).round() as usize).clamp(1, n.max(1));
                let mut held = vec![false; n];
                if n > 0 {
                    for r in sample(&mut ChaCha8Rng::seed_from_u64(*seed), n, k) {
                        held[r] = true;
                    }
                }
                held
            }
        };
        let (train, out): (Vec<usize>, Vec<usize>) = (0..n).partition(|&r| !held[r]);
        if train.is_empty() || out.is_empty() {
            return Err(format!(
                "--holdout {}: one side of the split is empty",
                self.describe()
            ));
        }
        Ok((train, out))
    }
}

/// What the held-out objects say about one candidate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    /// Objects where the hypothesis holds.
    pub applicable: usize,
    pub refuted: usize,
    pub confirmed: usize,
    pub undecided: usize,
    /// The first refuting objects, at most five.
    pub examples: Vec<usize>,
}

pub fn tally(ev: &Evaluated, i: usize) -> Tally {
    let mut t = Tally::default();
    for r in 0..ev.n_objects() {
        match ev.verdict(i, r) {
            Verdict::NotApplicable => continue,
            Verdict::Refuted => {
                t.refuted += 1;
                if t.examples.len() < EXAMPLES {
                    t.examples.push(r);
                }
            }
            Verdict::Confirmed => t.confirmed += 1,
            Verdict::Undecided => t.undecided += 1,
        }
        t.applicable += 1;
    }
    t
}

pub struct StageHeldOut {
    pub filter: &'static str,
    /// Representatives of this stage.
    pub kept: usize,
    /// ... refuted on at least one held-out object.
    pub refuted: usize,
    /// ... whose hypothesis holds on no held-out object.
    pub untested: usize,
    /// ... confirmed on at least one held-out object and refuted on none. The rest of the kept
    /// candidates that are neither refuted nor untested are undecided everywhere (missing values).
    pub supported: usize,
}

pub struct HeldOut {
    pub rule: String,
    pub train_pin: Pin,
    pub held_pin: Pin,
    pub held_objects: Vec<String>,
    pub stages: Vec<StageHeldOut>,
    /// Per survivor of the training run: its candidate index and tally on the held-out objects.
    pub survivors: Vec<(usize, Tally)>,
    /// Every candidate valid on training, ascending by index: its tally on the held-out objects,
    /// the number of training objects where it is tight, and where its hypothesis holds.
    pub valid: Vec<(usize, Tally, usize, usize)>,
}

/// Largest denominator among a bound's coefficients and constant (1 for huge integers).
fn max_denominator(bound: &ragusa_gen::Bound) -> i128 {
    bound
        .terms
        .iter()
        .map(|(_, c)| *c)
        .chain(std::iter::once(bound.constant))
        .map(|r| r.as_ratio().map_or(1, |(_, q)| q))
        .max()
        .unwrap_or(1)
}

/// `report` must come from a run on the training table.
pub fn check_held_out(report: &Report, held: &Table, rule: String) -> Result<HeldOut, String> {
    let first = report.stages.first().ok_or("the run has no stages")?;
    let valid = first.partition.representatives();
    let ev = Evaluated::new(
        held,
        valid
            .iter()
            .map(|&i| report.evaluated.bound(i).clone())
            .collect(),
    )
    .map_err(|e| e.to_string())?;
    let tallies: HashMap<usize, Tally> = (0..valid.len())
        .into_par_iter()
        .map(|k| (valid[k], tally(&ev, k)))
        .collect();
    let stages = report
        .stages
        .iter()
        .map(|s| {
            let kept = s.partition.representatives();
            StageHeldOut {
                filter: s.filter,
                kept: kept.len(),
                refuted: kept.iter().filter(|i| tallies[i].refuted > 0).count(),
                untested: kept.iter().filter(|i| tallies[i].applicable == 0).count(),
                supported: kept
                    .iter()
                    .filter(|i| tallies[i].refuted == 0 && tallies[i].confirmed > 0)
                    .count(),
            }
        })
        .collect();
    let survivors = report
        .survivors()
        .into_iter()
        .map(|i| (i, tallies[&i].clone()))
        .collect();
    let train = &report.evaluated;
    let mut valid: Vec<(usize, Tally, usize, usize)> = valid
        .par_iter()
        .map(|&i| {
            let tight = (0..train.n_objects())
                .filter(|&r| train.tight(i, r))
                .count();
            let applicable = (0..train.n_objects())
                .filter(|&r| train.applies(i, r))
                .count();
            (i, tallies[&i].clone(), tight, applicable)
        })
        .collect();
    valid.sort_by_key(|v| v.0);
    Ok(HeldOut {
        rule,
        train_pin: report.pin.clone(),
        held_pin: Pin::of(held),
        held_objects: held.objects().to_vec(),
        stages,
        survivors,
        valid,
    })
}

fn rate(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

impl HeldOut {
    /// One line per candidate valid on training: what the training run knew about it and what the
    /// held-out objects say, so acceptance rules can be compared without rerunning the pipeline.
    /// `duplicates_class` and `sharp_witnesses` are empty for candidates that did not reach those
    /// stages; `sharp_witnesses` counts the training objects where sharp coverage found the
    /// candidate strictly tightest.
    pub fn detail_csv(&self, report: &Report) -> String {
        let class_at = |filter: &str| -> HashMap<usize, (&'static str, usize)> {
            report
                .stages
                .iter()
                .find(|s| s.filter == filter)
                .map(|s| {
                    s.partition
                        .input
                        .iter()
                        .zip(&s.partition.classes)
                        .zip(&s.partition.witnesses)
                        .map(|((&i, c), w)| (i, (c.label(), w.len())))
                        .collect()
                })
                .unwrap_or_default()
        };
        let duplicates = class_at("table_duplicates");
        let sharp = class_at("sharp_coverage");
        let survivors: std::collections::HashSet<usize> =
            self.survivors.iter().map(|(i, _)| *i).collect();
        let mut out = String::from(
            "index,generator,feature,hypothesis,op,bound,duplicates_class,sharp_witnesses,survivor,tight_training,applicable_training,max_denominator,applicable,refuted,confirmed,undecided\n",
        );
        for (i, t, tight, applicable_training) in &self.valid {
            let b = report.evaluated.bound(*i);
            let _ = writeln!(
                out,
                "{i},{},{},{},{},\"{}\",{},{},{},{tight},{applicable_training},{},{},{},{},{}",
                report.origins[*i].generator.name(),
                report.origins[*i].feature,
                b.hypothesis,
                b.op.symbol(),
                b.to_string().replace('"', "\"\""),
                duplicates.get(i).map_or("", |d| d.0),
                sharp.get(i).map_or(String::new(), |s| s.1.to_string()),
                u8::from(survivors.contains(i)),
                max_denominator(b),
                t.applicable,
                t.refuted,
                t.confirmed,
                t.undecided
            );
        }
        out
    }

    pub fn to_json(&self, report: &Report) -> Value {
        json!({
            "format": FORMAT,
            "rule": self.rule,
            "engine": {"version": env!("CARGO_PKG_VERSION"), "git_rev": report.engine_git_rev},
            "target": report.spec.target,
            "train": self.train_pin.to_json(),
            "held_out": self.held_pin.to_json(),
            "stages": self.stages.iter().map(|s| json!({
                "filter": s.filter,
                "kept": s.kept,
                "refuted_on_held_out": s.refuted,
                "untested_on_held_out": s.untested,
                "supported_on_held_out": s.supported,
                "refuted_rate": rate(s.refuted, s.kept),
            })).collect::<Vec<_>>(),
            "survivors": self.survivors.iter().map(|(i, t)| json!({
                "index": i,
                "bound": report.evaluated.bound(*i).to_string(),
                "applicable": t.applicable,
                "refuted": t.refuted,
                "confirmed": t.confirmed,
                "undecided": t.undecided,
                "first_counterexamples": t.examples.iter().map(|&r| self.held_objects[r].as_str()).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })
    }

    pub fn to_markdown(&self, report: &Report) -> String {
        let mut md = String::new();
        let _ = writeln!(
            md,
            "# ragusa held-out report: target `{}`, rule `{}`\n",
            report.spec.target, self.rule
        );
        for (label, pin) in [("training", &self.train_pin), ("held out", &self.held_pin)] {
            let _ = writeln!(
                md,
                "- {label}: `{}`, {} objects, content sha256 `{}`",
                pin.table, pin.n_objects, pin.content_sha256
            );
        }
        let _ = writeln!(
            md,
            "- engine {} (git {})\n",
            env!("CARGO_PKG_VERSION"),
            report.engine_git_rev.as_deref().unwrap_or("unknown")
        );
        let _ = writeln!(md, "## Refuted on held-out objects, by stage\n");
        let _ = writeln!(
            md,
            "Each row is the candidates a stage kept on the training objects. Earlier rows are the baseline for later ones.\n"
        );
        let _ = writeln!(
            md,
            "supported: confirmed on some held-out object, refuted on none; untested: the hypothesis holds on no held-out object; the remainder is undecided everywhere (missing or interval values).\n"
        );
        let _ = writeln!(
            md,
            "| stage | kept | refuted | rate | supported | untested |"
        );
        let _ = writeln!(md, "|---|---:|---:|---:|---:|---:|");
        for s in &self.stages {
            let _ = writeln!(
                md,
                "| `{}` | {} | {} | {:.3} | {} | {} |",
                s.filter,
                s.kept,
                s.refuted,
                rate(s.refuted, s.kept),
                s.supported,
                s.untested
            );
        }
        let refuted: Vec<&(usize, Tally)> = self
            .survivors
            .iter()
            .filter(|(_, t)| t.refuted > 0)
            .collect();
        let _ = writeln!(md, "\n## Survivors refuted on held-out objects\n");
        if refuted.is_empty() {
            let _ = writeln!(md, "None.");
        }
        for (i, t) in refuted {
            let examples: Vec<&str> = t
                .examples
                .iter()
                .map(|&r| self.held_objects[r].as_str())
                .collect();
            let _ = writeln!(
                md,
                "- `{}`: refuted on {} of {} applicable (first: {})",
                report.evaluated.bound(*i),
                t.refuted,
                t.applicable,
                examples.join(", ")
            );
        }
        md
    }
}

#[cfg(test)]
mod tests {
    use ragusa_table::{BooleanBlock, NumericBlock, PackedColumns, Table};

    use super::*;
    use crate::{run, Generator, Redundancy, RunSpec};

    /// y = x on objects 0..5; the last object (x = 6) has y = 8.
    fn table() -> Table {
        let mut bits = PackedColumns::zeros(6, 1);
        for r in 0..6 {
            bits.set(r, 0);
        }
        Table::new(
            "t",
            (0..6).map(|r| format!("o{r}")).collect(),
            Some(NumericBlock::from_columns(
                vec!["x".into(), "y".into()],
                vec![
                    vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                    vec![1.0, 2.0, 3.0, 4.0, 5.0, 8.0],
                ],
            )),
            Some(BooleanBlock::new(vec!["p".into()], bits)),
            vec![],
            vec![],
        )
    }

    #[test]
    fn parse_rules() {
        assert_eq!(
            Holdout::parse("crossing_number>=13").unwrap(),
            Holdout::AtLeast {
                column: "crossing_number".into(),
                min: 13.0
            }
        );
        assert_eq!(
            Holdout::parse("fraction=0.25,seed=7").unwrap(),
            Holdout::Fraction {
                fraction: 0.25,
                seed: 7
            }
        );
        assert!(Holdout::parse("fraction=1.5").is_err());
        assert!(Holdout::parse("x<3").is_err());
    }

    #[test]
    fn fraction_split_is_seeded() {
        let t = table();
        let h = Holdout::parse("fraction=0.5,seed=3").unwrap();
        let (a, b) = h.split(&t).unwrap();
        assert_eq!((a.len(), b.len()), (3, 3));
        assert_eq!(h.split(&t).unwrap(), (a, b));
    }

    #[test]
    fn survivors_are_rechecked_on_held_out_objects() {
        let t = table();
        let all: Vec<usize> = (0..6).collect();
        assert_eq!(t.subset("copy", &all).content_sha256(), t.content_sha256());

        let h = Holdout::parse("x>=6").unwrap();
        let (train, held) = h.split(&t).unwrap();
        assert_eq!(held, vec![5]);
        let (train_t, held_t) = (t.subset("train", &train), t.subset("held", &held));
        let mut spec = RunSpec::full(&train_t, "y").unwrap();
        spec.generators = vec![Generator::Ratios];
        spec.chance_draws = 0;
        // The detail CSV's sharp_witnesses column is checked below.
        spec.redundancy = Redundancy::SharpCoverage;
        let report = run(&train_t, spec).unwrap();
        let ho = check_held_out(&report, &held_t, h.describe()).unwrap();

        let by_text = |s: &str| {
            ho.survivors
                .iter()
                .find(|(i, _)| report.evaluated.bound(*i).to_string() == s)
                .map(|(_, t)| t.clone())
        };
        let le = by_text("(p) → (y <= 1·x)").expect("the upper ratio bound survives training");
        assert_eq!((le.applicable, le.refuted, le.examples), (1, 1, vec![0]));
        let ge = by_text("(p) → (y >= 1·x)").expect("the lower ratio bound survives training");
        assert_eq!((ge.refuted, ge.confirmed), (0, 1));
        assert_eq!(ho.stages[0].refuted, 1);
        assert_eq!((ho.stages[0].supported, ho.stages[0].untested), (1, 0));
        assert_eq!(ho.stages.last().unwrap().refuted, 1);
        assert!(ho
            .to_markdown(&report)
            .contains("refuted on 1 of 1 applicable"));
        let csv = ho.detail_csv(&report);
        assert_eq!(csv.lines().count(), ho.valid.len() + 1);
        assert_eq!(
            ho.valid.len(),
            report.stages[0].partition.representatives().len()
        );
        // y <= 1·x: a survivor, strictly tightest on all 5 training objects, tight on all 5,
        // refuted on the single held-out object.
        let line = csv
            .lines()
            .find(|l| l.contains("(p) → (y <= 1·x)"))
            .expect("a line for the upper ratio bound");
        assert!(
            line.ends_with(",representative,5,1,5,5,1,1,1,0,0"),
            "{line}"
        );
    }
}
