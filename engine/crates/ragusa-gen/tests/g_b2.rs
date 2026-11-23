//! Gate G-B2: ragusa's generators reproduce txgraffiti 0.4.1's candidates on graph_data.
//!
//! Fixtures come from `calibration/src/ragusa_a/export_generator_fixtures.py`. txgraffiti computes
//! on pandas' default-parsed values, which differ from ragusa's correctly rounded load on a few
//! graph_data cells, so the gates feed the generators those same values.
//!
//! - `limit_denominator`: identical to Python on every fixture case.
//! - ratios: identical to txgraffiti, candidate by candidate, in order.
//! - convex hull: each (feature, hypothesis) call's bounds equal txgraffiti's as a multiset,
//!   exactly or within [`REL_TOL`] on every coefficient (scipy and qhull-sys bundle different
//!   qhull builds); nothing may stay unmatched.
//! - LP: HiGHS and CBC may pick different optimal vertices, so a differing bound must still
//!   reach txgraffiti's total slack within [`SLACK_TOL`]; the bound agreement rate is reported.

use std::collections::HashMap;
use std::fs;

use ragusa_gen::convex_hull::convex_hull;
use ragusa_gen::lp::linear_programming;
use ragusa_gen::ratios::{ratios, MAX_DENOMINATOR};
use ragusa_gen::{limit_denominator, Bound, Op, Rational};
use ragusa_table::{sources, BoolColumns, NumericBlock, Table};
use serde_json::Value;

/// Relative tolerance on each coefficient's value when two hull bounds are not identical.
const REL_TOL: f64 = 1e-6;
/// Relative tolerance on total slack when two LP bounds are not identical.
const SLACK_TOL: f64 = 1e-4;

fn fixture(name: &str) -> Value {
    let path = sources::engine_dir().join("fixtures").join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn f64_from_hex(hex: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(hex, 16).unwrap())
}

fn rational(v: &Value) -> Rational {
    v.as_str().unwrap().parse().unwrap()
}

fn strs(v: &Value) -> Vec<&str> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect()
}

/// graph_data with the cells pandas' default parser reads differently set to pandas' values.
fn graph_data_as_txgraffiti_reads_it() -> Table {
    let table = sources::load_standard("graph_data").unwrap();
    let numeric = table.numeric().unwrap();
    let mut columns: Vec<Vec<f64>> = (0..numeric.n_columns())
        .map(|c| numeric.column(c).to_vec())
        .collect();
    for cell in fixture("graph_data_default_parse.json")["values"]
        .as_array()
        .unwrap()
    {
        let c = numeric
            .names()
            .iter()
            .position(|n| n == cell["column"].as_str().unwrap())
            .unwrap();
        let r = cell["row"].as_u64().unwrap() as usize;
        assert_eq!(
            columns[c][r].to_bits(),
            f64_from_hex(cell["round_trip"].as_str().unwrap()).to_bits(),
            "loader value at {c},{r} is not the round_trip value"
        );
        columns[c][r] = f64_from_hex(cell["default"].as_str().unwrap());
    }
    Table::new(
        table.name(),
        table.objects().to_vec(),
        Some(NumericBlock::from_columns(
            numeric.names().to_vec(),
            columns,
        )),
        table.boolean().cloned(),
        table.skipped().to_vec(),
        table.sources().to_vec(),
    )
}

fn expected(candidate: &Value, target: &str) -> Bound {
    let op = match candidate["op"].as_str().unwrap() {
        "<=" => Op::Le,
        ">=" => Op::Ge,
        other => panic!("unexpected op {other}"),
    };
    let terms = candidate["coefs"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(f, c)| (f.clone(), rational(c)));
    Bound::new(
        candidate["hypothesis"].as_str().unwrap(),
        target,
        op,
        terms,
        rational(&candidate["constant"]),
    )
}

struct Scope<'a> {
    name: &'a str,
    target: &'a str,
    features: Vec<&'a str>,
    predicates: Vec<&'a str>,
    /// txgraffiti's candidates per (feature, hypothesis) call, in generation order.
    calls: HashMap<(String, String), Vec<Bound>>,
    total: usize,
}

fn scopes<'a>(fx: &'a Value, generator: &str) -> Vec<Scope<'a>> {
    fx["scopes"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, spec)| {
            let target = spec["target"].as_str().unwrap();
            let rows = spec["generators"][generator].as_array().unwrap();
            let mut calls: HashMap<(String, String), Vec<Bound>> = HashMap::new();
            for c in rows {
                let key = (
                    c["feature"].as_str().unwrap().to_string(),
                    c["hypothesis"].as_str().unwrap().to_string(),
                );
                calls.entry(key).or_default().push(expected(c, target));
            }
            Scope {
                name,
                target,
                features: strs(&spec["features"]),
                predicates: strs(&spec["predicates"]),
                calls,
                total: rows.len(),
            }
        })
        .collect()
}

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol * a.abs().max(b.abs()).max(1.0)
}

fn close(g: &Bound, w: &Bound) -> bool {
    g.hypothesis == w.hypothesis
        && g.target == w.target
        && g.op == w.op
        && g.terms.len() == w.terms.len()
        && g.terms
            .iter()
            .zip(&w.terms)
            .all(|((gf, gc), (wf, wc))| gf == wf && near(gc.to_f64(), wc.to_f64(), REL_TOL))
        && near(g.constant.to_f64(), w.constant.to_f64(), REL_TOL)
}

/// Total slack of a bound over the objects where its hypothesis holds.
fn total_slack(table: &Table, bound: &Bound) -> f64 {
    let numeric = table.numeric().unwrap();
    let boolean = table.boolean().unwrap();
    let col = |name: &str| numeric.column(numeric.names().iter().position(|n| n == name).unwrap());
    let h = boolean
        .names()
        .iter()
        .position(|n| *n == bound.hypothesis)
        .unwrap();
    let y = col(&bound.target);
    let terms: Vec<(&[f64], f64)> = bound
        .terms
        .iter()
        .map(|(f, c)| (col(f), c.to_f64()))
        .collect();
    (0..table.n_objects())
        .filter(|&r| boolean.bits().get(r, h))
        .map(|r| {
            let rhs = bound.constant.to_f64() + terms.iter().map(|(v, c)| c * v[r]).sum::<f64>();
            match bound.op {
                Op::Le => rhs - y[r],
                Op::Ge => y[r] - rhs,
            }
        })
        .sum()
}

#[derive(Default, Debug)]
struct Tally {
    want: usize,
    got: usize,
    exact: usize,
    within_tolerance: usize,
    unmatched_ragusa: Vec<String>,
    unmatched_txgraffiti: Vec<String>,
}

/// Multiset comparison of one call: exact matches first, then matches within tolerance.
fn compare_call(tally: &mut Tally, got: Vec<Bound>, want: &[Bound]) {
    tally.got += got.len();
    tally.want += want.len();
    let mut remaining: Vec<&Bound> = want.iter().collect();
    let mut left = Vec::new();
    for g in got {
        match remaining.iter().position(|w| **w == g) {
            Some(i) => {
                remaining.remove(i);
                tally.exact += 1;
            }
            None => left.push(g),
        }
    }
    for g in left {
        match remaining.iter().position(|w| close(&g, w)) {
            Some(i) => {
                remaining.remove(i);
                tally.within_tolerance += 1;
            }
            None => tally.unmatched_ragusa.push(g.to_string()),
        }
    }
    tally
        .unmatched_txgraffiti
        .extend(remaining.iter().map(|w| w.to_string()));
}

#[test]
fn limit_denominator_matches_python() {
    let fx = fixture("limit_denominator.json");
    assert_eq!(fx["max_denominator"].as_i64(), Some(MAX_DENOMINATOR as i64));
    let cases = fx["cases"].as_array().unwrap();
    assert!(cases.len() > 2000, "fixture too small");
    let mismatches: Vec<String> = cases
        .iter()
        .filter_map(|case| {
            let x = f64_from_hex(case["x"].as_str().unwrap());
            let p: Rational = case["p"].as_str().unwrap().parse().unwrap();
            let q: i128 = case["q"].as_str().unwrap().parse().unwrap();
            let want = if q == 1 {
                p
            } else {
                Rational::new(p.as_ratio().unwrap().0, q)
            };
            let got = limit_denominator(x, MAX_DENOMINATOR);
            (got != Some(want)).then(|| format!("{x:e}: ragusa {got:?}, python {want}"))
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

fn ratio_mismatches(table: &Table) -> Vec<String> {
    let fx = fixture("txgraffiti_candidates.json");
    let mut problems = Vec::new();
    for (scope, spec) in fx["scopes"].as_object().unwrap() {
        let target = spec["target"].as_str().unwrap();
        let want: Vec<Bound> = spec["generators"]["ratios"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| expected(c, target))
            .collect();
        let mut got = Vec::new();
        for feature in strs(&spec["features"]) {
            for hypothesis in strs(&spec["predicates"]) {
                got.extend(ratios(table, target, &[feature], hypothesis).unwrap());
            }
        }
        if got.len() != want.len() {
            problems.push(format!(
                "{scope}: {} candidates, txgraffiti {}",
                got.len(),
                want.len()
            ));
            continue;
        }
        for (i, (g, w)) in got.iter().zip(&want).enumerate() {
            if g != w {
                problems.push(format!("{scope}[{i}]: ragusa {g}  txgraffiti {w}"));
            }
        }
    }
    problems
}

#[test]
fn ratios_reproduce_txgraffiti_exactly() {
    let problems = ratio_mismatches(&graph_data_as_txgraffiti_reads_it());
    assert!(
        problems.is_empty(),
        "G-B2 ratios ({}):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn ratios_on_correctly_rounded_graph_data() {
    let problems = ratio_mismatches(&sources::load_standard("graph_data").unwrap());
    eprintln!(
        "G-B2 ratios on the correctly rounded table: {} candidates differ from txgraffiti",
        problems.len()
    );
    for p in problems.iter().take(10) {
        eprintln!("  {p}");
    }
}

#[test]
fn convex_hull_reproduces_txgraffiti_within_tolerance() {
    let table = graph_data_as_txgraffiti_reads_it();
    let fx = fixture("txgraffiti_candidates.json");
    let mut failures = Vec::new();
    for scope in scopes(&fx, "convex_hull") {
        let mut tally = Tally::default();
        let mut joggled = 0;
        for &feature in &scope.features {
            for &hypothesis in &scope.predicates {
                let result = convex_hull(&table, scope.target, &[feature], hypothesis).unwrap();
                joggled += usize::from(result.joggled);
                let want = scope
                    .calls
                    .get(&(feature.to_string(), hypothesis.to_string()))
                    .map_or(&[][..], |v| v);
                compare_call(&mut tally, result.bounds, want);
            }
        }
        assert_eq!(tally.want, scope.total);
        eprintln!(
            "G-B2 convex_hull {}: txgraffiti {} ragusa {} exact {} within {REL_TOL:e} {} unmatched {}/{} joggled calls {joggled}",
            scope.name,
            tally.want,
            tally.got,
            tally.exact,
            tally.within_tolerance,
            tally.unmatched_ragusa.len(),
            tally.unmatched_txgraffiti.len()
        );
        if !tally.unmatched_ragusa.is_empty() || !tally.unmatched_txgraffiti.is_empty() {
            failures.push(format!(
                "{}: ragusa-only {:?}\n  txgraffiti-only {:?}",
                scope.name,
                &tally.unmatched_ragusa[..tally.unmatched_ragusa.len().min(5)],
                &tally.unmatched_txgraffiti[..tally.unmatched_txgraffiti.len().min(5)]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "G-B2 convex_hull:\n{}",
        failures.join("\n")
    );
}

#[test]
fn linear_programming_reaches_txgraffiti_optimum() {
    let table = graph_data_as_txgraffiti_reads_it();
    let fx = fixture("txgraffiti_candidates.json");
    let mut failures = Vec::new();
    for scope in scopes(&fx, "linear_programming") {
        let (mut want_total, mut got_total, mut identical, mut same_optimum) = (0, 0, 0, 0);
        for &feature in &scope.features {
            for &hypothesis in &scope.predicates {
                let got = linear_programming(&table, scope.target, &[feature], hypothesis).unwrap();
                let want = scope
                    .calls
                    .get(&(feature.to_string(), hypothesis.to_string()))
                    .map_or(&[][..], |v| v);
                want_total += want.len();
                got_total += got.len();
                for g in &got {
                    let Some(w) = want.iter().find(|w| w.op == g.op) else {
                        continue;
                    };
                    if g == w {
                        identical += 1;
                    } else if near(total_slack(&table, g), total_slack(&table, w), SLACK_TOL) {
                        same_optimum += 1;
                    } else {
                        failures.push(format!(
                            "{} slack {:.6} vs txgraffiti {} slack {:.6}",
                            g,
                            total_slack(&table, g),
                            w,
                            total_slack(&table, w)
                        ));
                    }
                }
            }
        }
        eprintln!(
            "G-B2 linear_programming {}: txgraffiti {want_total} ragusa {got_total} identical {identical} \
             same optimum, different bound {same_optimum}",
            scope.name
        );
        if want_total != got_total {
            failures.push(format!(
                "{}: {got_total} bounds, txgraffiti {want_total}",
                scope.name
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "G-B2 linear_programming ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
