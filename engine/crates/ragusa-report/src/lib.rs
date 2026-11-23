//! ragusa-report: a conjecturing run and the floor report it always emits.
//!
//! A run generates candidates, evaluates them once, and passes them through the filter pipeline
//! `validity → table_duplicates → [touch] → redundancy → generality`, where the optional touch
//! stage keeps bounds tight often enough and the redundancy stage is `greedy_touch` by default
//! (`sharp_coverage` or `maximal_touch` on request), each stage partitioning the previous stage's
//! representatives. For every stage the report carries the partition, the
//! per-object flips under single-object removal (derived from the partition, or marked not
//! computed when a filter's budget is exceeded), and a seeded chance baseline, plus the table's
//! pinning block. There is no way to run the pipeline without producing it.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::time::Instant;

use ragusa_filter::chance::{self, Metric};
use ragusa_filter::filters::{
    Generality, GreedyTouch, MaximalTouch, SharpCoverage, TableDuplicates, Touch, Validity,
};
use ragusa_filter::floor::{self, ObjectFloor};
use ragusa_filter::{Evaluated, Filter, Partition, SLACK_TOL};
use ragusa_gen::convex_hull::convex_hull;
use ragusa_gen::lp::linear_programming;
use ragusa_gen::ratios::ratios;
pub use ragusa_gen::Bound;
use ragusa_gen::GenError;
use ragusa_table::{Pin, Table};
use rayon::prelude::*;
use serde_json::{json, Value};

pub use ragusa_filter::filters::TouchThreshold;

pub mod checks;
pub mod heldout;
pub mod implications;
pub mod knots;
pub mod novelty;

pub const FORMAT: &str = "ragusa-floor-report/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Generator {
    LinearProgramming,
    ConvexHull,
    Ratios,
}

impl Generator {
    /// txgraffiti2's default order.
    pub const ALL: [Generator; 3] = [
        Generator::LinearProgramming,
        Generator::ConvexHull,
        Generator::Ratios,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Generator::LinearProgramming => "linear_programming",
            Generator::ConvexHull => "convex_hull",
            Generator::Ratios => "ratios",
        }
    }

    pub fn parse(name: &str) -> Option<Generator> {
        Generator::ALL.into_iter().find(|g| g.name() == name)
    }
}

#[derive(Clone, Debug)]
pub struct RunSpec {
    pub target: String,
    pub features: Vec<String>,
    pub hypotheses: Vec<String>,
    pub generators: Vec<Generator>,
    pub seed: u64,
    pub chance_draws: usize,
    /// `None`: no touch stage. Otherwise a touch stage after table duplicates keeps candidates tight
    /// on at least as many objects as the threshold asks.
    pub min_tight: Option<TouchThreshold>,
    /// The stage that removes candidates other candidates make redundant.
    pub redundancy: Redundancy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Redundancy {
    SharpCoverage,
    MaximalTouch,
    GreedyTouch,
}

impl Redundancy {
    pub const ALL: [Redundancy; 3] = [
        Redundancy::SharpCoverage,
        Redundancy::MaximalTouch,
        Redundancy::GreedyTouch,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Redundancy::SharpCoverage => "sharp_coverage",
            Redundancy::MaximalTouch => "maximal_touch",
            Redundancy::GreedyTouch => "greedy_touch",
        }
    }

    pub fn parse(name: &str) -> Option<Redundancy> {
        Redundancy::ALL.into_iter().find(|r| r.name() == name)
    }
}

impl RunSpec {
    /// Every other numeric column as a feature, every boolean column as a hypothesis, every generator.
    pub fn full(table: &Table, target: &str) -> Result<RunSpec, String> {
        let numeric = table.numeric().ok_or("table has no numeric block")?;
        let boolean = table.boolean().ok_or("table has no boolean block")?;
        if !numeric.names().iter().any(|n| n == target) {
            return Err(format!("target {target:?} is not a numeric column"));
        }
        Ok(RunSpec {
            target: target.to_string(),
            features: numeric
                .names()
                .iter()
                .filter(|n| *n != target)
                .cloned()
                .collect(),
            hypotheses: boolean.names().to_vec(),
            generators: Generator::ALL.to_vec(),
            seed: 0,
            chance_draws: 1000,
            min_tight: None,
            // Sprint 10: greedy_touch replaced sharp_coverage as the default (CONTROLS.md).
            redundancy: Redundancy::GreedyTouch,
        })
    }
}

pub struct Origin {
    pub generator: Generator,
    pub feature: String,
}

pub struct Stage {
    pub filter: &'static str,
    pub kind: &'static str,
    pub partition: Partition,
    /// `None`: attribution not computed (the filter's budget was exceeded).
    pub floor: Option<ObjectFloor>,
    pub chance: Vec<Metric>,
}

pub struct Report<'t> {
    pub pin: Pin,
    pub spec: RunSpec,
    pub objects: Vec<String>,
    pub evaluated: Evaluated<'t>,
    pub origins: Vec<Origin>,
    pub per_generator: BTreeMap<&'static str, usize>,
    pub joggled_hull_calls: usize,
    pub stages: Vec<Stage>,
    pub engine_git_rev: Option<String>,
    /// Wall-clock seconds per phase, in execution order. Never part of the floor report, which
    /// must be reproducible byte for byte.
    pub timings: Vec<(String, f64)>,
}

pub fn run(table: &Table, spec: RunSpec) -> Result<Report<'_>, String> {
    let mut bounds = Vec::new();
    let mut origins = Vec::new();
    let mut per_generator = BTreeMap::new();
    let mut joggled_hull_calls = 0;
    let mut timings: Vec<(String, f64)> = Vec::new();
    let mut generator_seconds: BTreeMap<&'static str, f64> = BTreeMap::new();
    let wall = Instant::now();
    let target = spec.target.as_str();
    // Generator calls in txgraffiti2's order. The LP calls are independent single-threaded HiGHS
    // solves of unchanged models, so they run in parallel; every result is consumed in this order,
    // so the candidate list is exactly that of a sequential run.
    let mut calls: Vec<(&str, Generator, &str)> = Vec::new();
    for f in &spec.features {
        for &g in &spec.generators {
            for h in &spec.hypotheses {
                calls.push((f.as_str(), g, h.as_str()));
            }
        }
    }
    type Timed = (Result<Vec<Bound>, GenError>, f64);
    let mut solved: Vec<Option<Timed>> = calls
        .par_iter()
        .map(|&(f, g, h)| {
            (g == Generator::LinearProgramming).then(|| {
                let start = Instant::now();
                let result = linear_programming(table, target, &[f], h);
                (result, start.elapsed().as_secs_f64())
            })
        })
        .collect();
    for (k, &(feature, generator, hypothesis)) in calls.iter().enumerate() {
        let start = Instant::now();
        let (result, seconds) = match generator {
            Generator::LinearProgramming => solved[k].take().expect("solved in parallel above"),
            Generator::Ratios => {
                let result = ratios(table, target, &[feature], hypothesis);
                (result, start.elapsed().as_secs_f64())
            }
            Generator::ConvexHull => {
                let result = convex_hull(table, target, &[feature], hypothesis).map(|hull| {
                    joggled_hull_calls += usize::from(hull.joggled);
                    hull.bounds
                });
                (result, start.elapsed().as_secs_f64())
            }
        };
        let out = result
            .map_err(|e| format!("{} on ({feature}, {hypothesis}): {e}", generator.name()))?;
        *generator_seconds.entry(generator.name()).or_insert(0.0) += seconds;
        *per_generator.entry(generator.name()).or_insert(0) += out.len();
        origins.extend(out.iter().map(|_| Origin {
            generator,
            feature: feature.to_string(),
        }));
        bounds.extend(out);
    }

    // Per generator: seconds summed over calls (CPU-like when calls run in parallel).
    timings.extend(
        generator_seconds
            .iter()
            .map(|(g, s)| (format!("generate.{g}.call_seconds"), *s)),
    );
    timings.push(("generate.wall".to_string(), wall.elapsed().as_secs_f64()));
    let mut lap = Instant::now();
    let mut clock = |timings: &mut Vec<(String, f64)>, name: String| {
        timings.push((name, lap.elapsed().as_secs_f64()));
        lap = Instant::now();
    };
    let evaluated = Evaluated::new(table, bounds).map_err(|e| e.to_string())?;
    clock(&mut timings, "evaluate".to_string());
    let active = vec![true; table.n_objects()];
    let touch = spec.min_tight.map(|threshold| Touch { threshold });
    let mut filters: Vec<&dyn Filter> = vec![&Validity, &TableDuplicates];
    if let Some(touch) = &touch {
        filters.push(touch);
    }
    let redundancy: &dyn Filter = match spec.redundancy {
        Redundancy::SharpCoverage => &SharpCoverage,
        Redundancy::MaximalTouch => &MaximalTouch,
        Redundancy::GreedyTouch => &GreedyTouch,
    };
    filters.extend([redundancy, &Generality]);
    let mut input: Vec<usize> = (0..evaluated.len()).collect();
    let mut stages = Vec::new();
    for (s, filter) in filters.into_iter().enumerate() {
        let partition = filter.partition(&evaluated, &input, &active);
        clock(&mut timings, format!("{}.partition", filter.name()));
        let floor = floor::attribute(filter, &evaluated, &input, &active, &partition);
        clock(&mut timings, format!("{}.attribution", filter.name()));
        let kept: Vec<usize> = partition
            .classes
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_representative())
            .map(|(p, _)| p)
            .collect();
        let chance = chance::baseline(
            &evaluated,
            &input,
            &kept,
            &active,
            spec.chance_draws,
            spec.seed.wrapping_add(s as u64),
        );
        clock(&mut timings, format!("{}.chance", filter.name()));
        let next = partition.representatives();
        stages.push(Stage {
            filter: filter.name(),
            kind: filter.kind().as_str(),
            partition,
            floor,
            chance,
        });
        input = next;
    }

    Ok(Report {
        pin: Pin::of(table),
        spec,
        objects: table.objects().to_vec(),
        evaluated,
        origins,
        per_generator,
        joggled_hull_calls,
        stages,
        engine_git_rev: None,
        timings,
    })
}

impl Report<'_> {
    pub fn survivors(&self) -> Vec<usize> {
        self.stages.last().map_or_else(
            || (0..self.evaluated.len()).collect(),
            |s| s.partition.representatives(),
        )
    }

    pub fn to_json(&self) -> Value {
        let stages: Vec<Value> = self
            .stages
            .iter()
            .map(|s| {
                let (rep, alias, disc) = s.partition.counts();
                let floor = match &s.floor {
                    None => json!({"computed": false}),
                    Some(floor) => {
                        let fragile_accepted = floor.fragile.iter().zip(&s.partition.classes).filter(|(f, c)| **f && c.is_representative()).count();
                        let fragile_total = floor.fragile.iter().filter(|&&f| f).count();
                        json!({
                            "fragile": fragile_total,
                            "fragile_accepted": fragile_accepted,
                            "fragile_rejected": fragile_total - fragile_accepted,
                            "total_flips": floor.total_flips(),
                            "objects_with_flips": floor.objects_with_flips(),
                            "top_objects": floor.ranked_objects().iter().take(10)
                                .map(|&(r, f)| json!({"object": self.objects[r], "flips": f})).collect::<Vec<_>>(),
                            "flips_by_object": floor.flips_by_object,
                        })
                    }
                };
                json!({
                    "filter": s.filter,
                    "kind": s.kind,
                    "input": s.partition.input.len(),
                    "representative": rep,
                    "alias": alias,
                    "discarded": disc,
                    "floor": floor,
                    "chance": s.chance.iter().map(|m| json!({
                        "metric": m.name,
                        "rule": m.rule,
                        "random_mean": m.random_mean,
                        "random_p2_5": m.random_low,
                        "random_p97_5": m.random_high,
                        "frac_random_at_least_rule": m.frac_random_at_least_rule,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let mut doc = json!({
            "format": FORMAT,
            "pin": self.pin.to_json(),
            "engine": {"version": env!("CARGO_PKG_VERSION"), "git_rev": self.engine_git_rev},
            "run": {
                "target": self.spec.target,
                "features": self.spec.features,
                "hypotheses": self.spec.hypotheses,
                "generators": self.spec.generators.iter().map(|g| g.name()).collect::<Vec<_>>(),
                "seed": self.spec.seed,
                "chance_draws": self.spec.chance_draws,
                "slack_tolerance": SLACK_TOL,
            },
            "generation": {
                "candidates": self.evaluated.len(),
                "per_generator": self.per_generator,
                "joggled_hull_calls": self.joggled_hull_calls,
            },
            "stages": stages,
            "survivors": self.survivors().len(),
        });
        // Only when the stage exists, so reports of runs without it are unchanged.
        if let Some(threshold) = self.spec.min_tight {
            doc["run"]["min_tight"] = json!(threshold.describe());
        }
        doc["run"]["redundancy"] = json!(self.spec.redundancy.name());
        doc
    }

    /// One line per candidate with its class at every stage it reached (empty where it did not).
    pub fn candidates_csv(&self) -> String {
        let positions: Vec<HashMap<usize, usize>> = self
            .stages
            .iter()
            .map(|s| {
                s.partition
                    .input
                    .iter()
                    .enumerate()
                    .map(|(p, &i)| (i, p))
                    .collect()
            })
            .collect();
        let mut out = String::from("index,generator,feature,bound");
        for s in &self.stages {
            let _ = write!(out, ",{0}_class,{0}_related", s.filter);
        }
        out.push('\n');
        for (i, origin) in self.origins.iter().enumerate() {
            let bound = self.evaluated.bound(i).to_string().replace('"', "\"\"");
            let _ = write!(
                out,
                "{i},{},{},\"{bound}\"",
                origin.generator.name(),
                origin.feature
            );
            for (s, stage) in self.stages.iter().enumerate() {
                match positions[s].get(&i) {
                    Some(&p) => {
                        let class = stage.partition.classes[p];
                        let related = class.related().map_or(String::new(), |j| j.to_string());
                        let _ = write!(out, ",{},{related}", class.label());
                    }
                    None => out.push_str(",,"),
                }
            }
            out.push('\n');
        }
        out
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        let _ = writeln!(
            md,
            "# ragusa floor report: `{}`, target `{}`\n",
            self.pin.table, self.spec.target
        );
        let _ = writeln!(
            md,
            "- table content sha256 `{}`, {} objects, {} numeric and {} boolean columns",
            self.pin.content_sha256, self.pin.n_objects, self.pin.n_numeric, self.pin.n_boolean
        );
        let _ = writeln!(
            md,
            "- engine {} (git {}), seed {}, {} chance draws per stage, slack tolerance {SLACK_TOL:e}",
            env!("CARGO_PKG_VERSION"),
            self.engine_git_rev.as_deref().unwrap_or("unknown"),
            self.spec.seed,
            self.spec.chance_draws
        );
        let per_gen: Vec<String> = self
            .per_generator
            .iter()
            .map(|(g, n)| format!("`{g}` {n}"))
            .collect();
        let _ = writeln!(
            md,
            "- {} candidates ({}); {} convex-hull calls needed joggling\n",
            self.evaluated.len(),
            per_gen.join(", "),
            self.joggled_hull_calls
        );

        let _ = writeln!(md, "## Stages\n");
        let _ = writeln!(md, "| stage | kind | input | representative | alias | discarded | fragile decisions | objects with a flip |");
        let _ = writeln!(md, "|---|---|---:|---:|---:|---:|---:|---:|");
        for s in &self.stages {
            let (rep, alias, disc) = s.partition.counts();
            let (fragile, with_flips) = s.floor.as_ref().map_or_else(
                || ("not computed".to_string(), "not computed".to_string()),
                |f| {
                    (
                        f.fragile.iter().filter(|&&x| x).count().to_string(),
                        f.objects_with_flips().to_string(),
                    )
                },
            );
            let _ = writeln!(
                md,
                "| `{}` | {} | {} | {rep} | {alias} | {disc} | {fragile} | {with_flips} |",
                s.filter,
                s.kind,
                s.partition.input.len(),
            );
        }
        let _ = writeln!(md, "\nSurvivors: {}\n", self.survivors().len());

        let _ = writeln!(md, "## Chance baseline\n");
        let _ = writeln!(
            md,
            "Random acceptance of the same number of candidates from the same stage input.\n"
        );
        let _ = writeln!(
            md,
            "| stage | metric | rule | random mean [2.5%, 97.5%] | P(random ≥ rule) |"
        );
        let _ = writeln!(md, "|---|---|---:|---|---:|");
        for s in &self.stages {
            for m in &s.chance {
                let _ = writeln!(
                    md,
                    "| `{}` | {} | {} | {:.1} [{}, {}] | {:.3} |",
                    s.filter,
                    m.name,
                    m.rule,
                    m.random_mean,
                    m.random_low,
                    m.random_high,
                    m.frac_random_at_least_rule
                );
            }
        }

        let _ = writeln!(md, "\n## Objects that decide the most\n");
        let _ = writeln!(
            md,
            "Decisions that flip when this one object is removed, per stage.\n"
        );
        let flips = |s: &Stage, r: usize| s.floor.as_ref().map_or(0, |f| f.flips_by_object[r]);
        let mut totals: Vec<(usize, usize)> = (0..self.objects.len())
            .map(|r| (r, self.stages.iter().map(|s| flips(s, r)).sum()))
            .filter(|&(_, t)| t > 0)
            .collect();
        totals.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let header: Vec<&str> = self.stages.iter().map(|s| s.filter).collect();
        let _ = writeln!(md, "| object | total | {} |", header.join(" | "));
        let _ = writeln!(md, "|---|---:|{}|", vec!["---:"; header.len()].join("|"));
        for &(r, total) in totals.iter().take(10) {
            let per: Vec<String> = self
                .stages
                .iter()
                .map(|s| {
                    s.floor.as_ref().map_or("not computed".to_string(), |f| {
                        f.flips_by_object[r].to_string()
                    })
                })
                .collect();
            let _ = writeln!(
                md,
                "| {} | {total} | {} |",
                self.objects[r],
                per.join(" | ")
            );
        }
        md
    }
}
