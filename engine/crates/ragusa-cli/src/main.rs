//! ragusa command line.
//!
//!     ragusa pin <table>                       pinning block of a standard table
//!     ragusa pin-file txgraffiti <csv>         pinning block of any TxGraffiti-style CSV
//!     ragusa pin-file parsimagma <dir>         pinning block of any parsimagma corpus directory
//!     ragusa pins [--write]                    all standard tables (--write: engine/fixtures/pins.json)
//!     ragusa columns <table>                   numeric column names, one per line
//!     ragusa check <table> --bounds FILE [--edge COLUMN] [--complete-knots] [--out FILE]
//!                                              evaluate the bounds in FILE (one per line, as printed);
//!                                              --complete-knots first fills missing cells from the knot catalogue
//!     ragusa column-stats <table>              numeric column name, tab, number of distinct known cells
//!                                              (a point or an interval, NaN excluded)
//!     ragusa floor <table> --target <column> [--generators a,b] [--seed N] [--draws N] [--out DIR]
//!                  [--oracle knots]            run the pipeline; always writes floor.json, floor.md,
//!                                              candidates.csv (default DIR: results/floor/<table>_<target>,
//!                                              or external/results/floor/... for tables under external/);
//!                                              --oracle knots also writes novelty.json, novelty.md and
//!                                              fails if the catalogue gate fails
//!                  [--timings FILE]            wall-clock seconds per phase as JSON (never in floor.json)
//!                  [--redundancy RULE]         greedy_touch (default), sharp_coverage or maximal_touch
//!                  [--min-tight K]             add a touch stage after table_duplicates keeping bounds
//!                                              tight on at least K objects (default: no such stage)
//!                  [--hypotheses LIST]         comma-separated boolean columns; `all` is every boolean
//!                                              column, `TRUE` the hypothesis that always holds
//!                                              (default: all)
//!                  [--holdout RULE]            RULE is column>=value or fraction=F[,seed=S]: the
//!                                              pipeline runs on the other objects only, and
//!                                              heldout.json, heldout.md re-check its candidates on the
//!                                              held-out ones, candidates_heldout.csv has one line per
//!                                              candidate valid on training (DIR gets a _holdout suffix)
//!     ragusa implications <table> [--out DIR]  implication conjectures against the ETP graph; always
//!                                              writes floor.json, floor.md (default DIR:
//!                                              results/floor/<table>_implications); fails if unsound

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use ragusa_oracle::etp::{self, EtpImplications};
use ragusa_oracle::knots::Catalogue;
use ragusa_report::checks::{check_bounds, complete_with_catalogue};
use ragusa_report::heldout::{check_held_out, Holdout};
use ragusa_report::implications::run_implications;
use ragusa_report::knots::{check_catalogue, NoveltyOracle};
use ragusa_report::novelty::novelty_stage;
use ragusa_report::{run, Bound, Generator, Redundancy, RunSpec, TouchThreshold};
use ragusa_table::{parsimagma, sources, txgraffiti, Pin, Table};
use serde_json::{Map, Value};

const USAGE: &str = "usage: ragusa pin <table> | pin-file txgraffiti <csv> | pin-file parsimagma <dir> | pins [--write]\n       \
                     ragusa floor <table> --target <column> [--generators a,b] [--seed N] [--draws N] [--out DIR] [--oracle knots] [--hypotheses LIST] [--holdout RULE] [--timings FILE] [--min-tight K] [--redundancy RULE]\n       \
                     ragusa implications <table> [--out DIR]";

fn git_rev() -> Option<String> {
    let out = Command::new("git")
        .args(["-C", sources::ENGINE_DIR, "rev-parse", "HEAD"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map_or_else(|| path.to_string(), |s| s.to_string_lossy().into_owned())
}

fn print_pin(table: &Table) {
    let mut pin = Pin::of(table).to_json();
    pin["engine_git_rev"] = git_rev().map_or(Value::Null, Value::String);
    println!("{}", serde_json::to_string_pretty(&pin).expect("json"));
}

fn pins(write: bool) -> Result<(), String> {
    let mut all = Map::new();
    for name in sources::standard_names() {
        let table = sources::load_standard(name)?;
        all.insert(name.to_string(), Pin::of(&table).to_json());
    }
    let text = serde_json::to_string_pretty(&Value::Object(all)).expect("json") + "\n";
    if write {
        let path = sources::engine_dir().join("fixtures/pins.json");
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        eprintln!("wrote {}", path.display());
    } else {
        print!("{text}");
    }
    Ok(())
}

fn floor(table_name: &str, options: &[&str]) -> Result<(), String> {
    let mut target = None;
    let mut generators = None;
    let (mut seed, mut draws, mut out) = (None, None, None);
    let mut knots_oracle = false;
    let mut holdout = None;
    let mut hypotheses = None;
    let mut timings_file = None;
    let mut min_tight = None;
    let mut redundancy = None;
    let mut it = options.iter();
    while let Some(&flag) = it.next() {
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag {
            "--target" => target = Some(value.to_string()),
            "--holdout" => holdout = Some(Holdout::parse(value)?),
            "--hypotheses" => hypotheses = Some(value.to_string()),
            "--timings" => timings_file = Some(PathBuf::from(value)),
            "--redundancy" => {
                redundancy = Some(
                    Redundancy::parse(value)
                        .ok_or_else(|| format!("unknown redundancy rule {value:?}"))?,
                )
            }
            "--min-tight" => {
                min_tight = Some(TouchThreshold::parse(value).ok_or_else(|| {
                    format!("--min-tight: expected a count or `sqrt`, got {value:?}")
                })?)
            }
            "--oracle" => match *value {
                "knots" => knots_oracle = true,
                other => return Err(format!("unknown oracle {other:?}")),
            },
            "--generators" => {
                generators = Some(
                    value
                        .split(',')
                        .map(|g| {
                            Generator::parse(g).ok_or_else(|| format!("unknown generator {g:?}"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            "--seed" => seed = Some(value.parse::<u64>().map_err(|e| format!("--seed: {e}"))?),
            "--draws" => {
                draws = Some(
                    value
                        .parse::<usize>()
                        .map_err(|e| format!("--draws: {e}"))?,
                )
            }
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown option {flag}\n{USAGE}")),
        }
    }
    let target = target.ok_or("--target is required")?;

    let table = sources::load_standard(table_name)?;
    // With a hold-out rule, the whole pipeline sees only the training objects.
    let split = match &holdout {
        Some(h) => {
            let (train, held) = h.split(&table)?;
            let rule = h.describe();
            Some((
                table.subset(format!("{table_name}[train: not {rule}]"), &train),
                table.subset(format!("{table_name}[held out: {rule}]"), &held),
                rule,
            ))
        }
        None => None,
    };
    let run_table = split.as_ref().map_or(&table, |(train, _, _)| train);
    let mut spec = RunSpec::full(run_table, &target)?;
    if let Some(g) = generators {
        spec.generators = g;
    }
    if let Some(list) = hypotheses {
        let all = spec.hypotheses.clone();
        spec.hypotheses = list
            .split(',')
            .flat_map(|h| {
                if h == "all" {
                    all.clone()
                } else {
                    vec![h.to_string()]
                }
            })
            .collect();
    }
    if let Some(s) = seed {
        spec.seed = s;
    }
    if let Some(d) = draws {
        spec.chance_draws = d;
    }
    if min_tight.is_some() {
        spec.min_tight = min_tight;
    }
    if let Some(r) = redundancy {
        spec.redundancy = r;
    }

    let started = std::time::Instant::now();
    let mut report = run(run_table, spec)?;
    report.engine_git_rev = git_rev();
    if let Some(path) = &timings_file {
        let phases: Map<String, Value> = report
            .timings
            .iter()
            .map(|(name, s)| (name.clone(), Value::from(*s)))
            .collect();
        let doc = serde_json::json!({
            "table": table_name,
            "target": target,
            "objects": run_table.n_objects(),
            "candidates": report.evaluated.len(),
            "run_seconds": started.elapsed().as_secs_f64(),
            "phases": phases,
        });
        std::fs::write(
            path,
            serde_json::to_string_pretty(&doc).expect("json") + "\n",
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
    }

    // Tables under external/ are not necessarily redistributable, so their reports stay there too.
    let dir = out.unwrap_or_else(|| {
        let base = if sources::external_path(table_name).is_some() {
            sources::external_dir().join("results/floor")
        } else {
            sources::engine_dir().join("../results/floor")
        };
        let suffix = if holdout.is_some() { "_holdout" } else { "" };
        base.join(format!("{table_name}_{target}{suffix}"))
    });
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let write = |name: &str, text: String| {
        let path = dir.join(name);
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
    };
    write(
        "floor.json",
        serde_json::to_string_pretty(&report.to_json()).expect("json") + "\n",
    )?;
    write("floor.md", report.to_markdown())?;
    write("candidates.csv", report.candidates_csv())?;

    println!(
        "{} candidates on {} objects",
        report.evaluated.len(),
        report.objects.len()
    );
    for stage in &report.stages {
        let (rep, alias, disc) = stage.partition.counts();
        println!(
            "  {:<17} input {:>5}  representative {:>5}  alias {:>4}  discarded {:>5}  objects with a flip {:>3}",
            stage.filter,
            stage.partition.input.len(),
            rep,
            alias,
            disc,
            stage
                .floor
                .as_ref()
                .map_or("not computed".to_string(), |f| f
                    .objects_with_flips()
                    .to_string())
        );
    }
    println!("floor report -> {}", dir.display());

    if let Some((_, held, rule)) = &split {
        let ho = check_held_out(&report, held, rule.clone())?;
        write(
            "heldout.json",
            serde_json::to_string_pretty(&ho.to_json(&report)).expect("json") + "\n",
        )?;
        write("heldout.md", ho.to_markdown(&report))?;
        write("candidates_heldout.csv", ho.detail_csv(&report))?;
        println!(
            "held out ({rule}): trained on {} objects, re-checked on {}",
            ho.train_pin.n_objects, ho.held_pin.n_objects
        );
        for s in &ho.stages {
            println!(
                "  {:<17} kept {:>5}  refuted on held-out {:>5}  supported {:>5}  untested {:>5}",
                s.filter, s.kept, s.refuted, s.supported, s.untested
            );
        }
    }

    if knots_oracle {
        let catalogue = Catalogue::standard()?;
        let check = check_catalogue(&table, &catalogue)?;
        let oracle = NoveltyOracle::new(&catalogue);
        let stage = novelty_stage(&report, &check, &oracle);
        write(
            "novelty.json",
            serde_json::to_string_pretty(&stage.to_json(&report, &oracle, &catalogue.citation))
                .expect("json")
                + "\n",
        )?;
        write(
            "novelty.md",
            stage.to_markdown(&report, &oracle, &catalogue.citation),
        )?;
        println!(
            "catalogue gate: {} of {} bounds checkable, {} refuted on {} non-trivial knots",
            stage.gate_checked, stage.gate_bounds, stage.gate_refuted, stage.gate_objects
        );
        println!(
            "novelty (all candidates / survivors): implied {}/{}, same pair not implied {}/{}, new pair {}/{}, outside {}/{}",
            stage.all_counts[0],
            stage.survivor_counts[0],
            stage.all_counts[1],
            stage.survivor_counts[1],
            stage.all_counts[2],
            stage.survivor_counts[2],
            stage.all_counts[3],
            stage.survivor_counts[3]
        );
        if !stage.gate_pass() {
            return Err("catalogue gate failed: the novelty classes are not trustworthy".into());
        }
    }
    Ok(())
}

fn implications(table_name: &str, options: &[&str]) -> Result<(), String> {
    let mut out = None;
    let mut it = options.iter();
    while let Some(&flag) = it.next() {
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag {
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown option {flag}\n{USAGE}")),
        }
    }

    let table = sources::load_standard(table_name)?;
    let oracle = EtpImplications::load(&etp::vendored_matrix())?;
    let hard_core_path = etp::vendored_hard_core();
    let hard_core_text = std::fs::read_to_string(&hard_core_path)
        .map_err(|e| format!("{}: {e}", hard_core_path.display()))?;
    let hard_core = etp::parse_pairs(&hard_core_text)?;
    let mut report = run_implications(&table, &oracle, Some(&hard_core))?;
    report.engine_git_rev = git_rev();

    let dir = out.unwrap_or_else(|| {
        sources::engine_dir()
            .join("../results/floor")
            .join(format!("{table_name}_implications"))
    });
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (name, text) in [
        (
            "floor.json",
            serde_json::to_string_pretty(&report.to_json()).expect("json") + "\n",
        ),
        ("floor.md", report.to_markdown()),
    ] {
        let path = dir.join(name);
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }

    let o = report.oracle;
    println!(
        "{} laws, {} objects: {} ordered pairs, {} emitted ({} known true, {} known false), {} refuted ({} by one object)",
        report.scan.n(),
        report.objects.len(),
        report.scan.pairs(),
        report.scan.emitted_count(),
        o.emitted_true,
        o.emitted_false,
        report.scan.refuted,
        report.scan.refuted_once
    );
    println!("floor report -> {}", dir.display());
    if !report.sound() {
        return Err(format!(
            "soundness failed: {} pairs refuted by the table are true in the oracle",
            o.refuted_true
        ));
    }
    Ok(())
}

fn check(table_name: &str, options: &[&str]) -> Result<(), String> {
    let (mut bounds_file, mut edge, mut out, mut complete) = (None, None, None, false);
    let mut it = options.iter();
    while let Some(&flag) = it.next() {
        if flag == "--complete-knots" {
            complete = true;
            continue;
        }
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag {
            "--bounds" => bounds_file = Some(PathBuf::from(value)),
            "--edge" => edge = Some(value.to_string()),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown option {flag}\n{USAGE}")),
        }
    }
    let path = bounds_file.ok_or("--bounds is required")?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let bounds = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().parse::<Bound>())
        .collect::<Result<Vec<_>, _>>()?;
    let table = sources::load_standard(table_name)?;
    let (table, completion) = if complete {
        let (t, filled, contradictions) = complete_with_catalogue(&table, &Catalogue::standard()?)?;
        (t, Some((filled, contradictions)))
    } else {
        (table, None)
    };
    let mut doc = check_bounds(&table, bounds, edge.as_deref())?.to_json();
    doc["engine_git_rev"] = git_rev().map_or(Value::Null, Value::String);
    if let Some((filled, contradictions)) = completion {
        doc["completion"] =
            serde_json::json!({"filled_cells": filled, "contradictions": contradictions});
    }
    let text = serde_json::to_string_pretty(&doc).expect("json") + "\n";
    match out {
        Some(p) => std::fs::write(&p, text).map_err(|e| format!("{}: {e}", p.display()))?,
        None => print!("{text}"),
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["pin", name] => sources::load_standard(name).map(|t| print_pin(&t)),
        ["pin-file", "txgraffiti", path] => txgraffiti::load_csv(Path::new(path), &stem(path))
            .map(|t| print_pin(&t))
            .map_err(|e| e.to_string()),
        ["pin-file", "parsimagma", dir] => parsimagma::load_corpus(Path::new(dir), &stem(dir))
            .map(|t| print_pin(&t))
            .map_err(|e| e.to_string()),
        ["pins"] => pins(false),
        ["pins", "--write"] => pins(true),
        ["check", table, options @ ..] => check(table, options),
        ["column-stats", name] => sources::load_standard(name).map(|t| {
            if let Some(b) = t.numeric() {
                for (c, n) in b.names().iter().enumerate() {
                    let norm = |v: f64| (if v == 0.0 { 0.0 } else { v }).to_bits();
                    let distinct: HashSet<(u64, u64)> = b
                        .lower(c)
                        .iter()
                        .zip(b.upper(c))
                        .filter(|(lo, _)| !lo.is_nan())
                        .map(|(&lo, &hi)| (norm(lo), norm(hi)))
                        .collect();
                    println!("{n}\t{}", distinct.len());
                }
            }
        }),
        ["columns", name] => sources::load_standard(name).map(|t| {
            for n in t.numeric().map_or(&[][..], |b| b.names()) {
                println!("{n}");
            }
        }),
        ["floor", table, options @ ..] => floor(table, options),
        ["implications", table, options @ ..] => implications(table, options),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
