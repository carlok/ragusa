//! Gate G-B5: ragusa on parsimagma's magma corpus, against the ETP implication graph.
//!
//! - Soundness: no implication the ETP graph knows to be true is refuted by a corpus magma.
//! - Independent cross-check: ragusa's counts equal parsimagma's own `summary.tsv` for the same
//!   corpus (false implications, those the corpus refutes, hard-core coverage).
//!
//! False implications that no corpus magma refutes are emitted; that number is the corpus's
//! floor, reported rather than required to be zero.

use std::collections::HashMap;
use std::fs;

use ragusa_oracle::etp::{parse_pairs, vendored_hard_core, vendored_matrix, EtpImplications};
use ragusa_report::implications::run_implications;
use ragusa_table::sources;

fn summary() -> HashMap<String, u64> {
    let text = fs::read_to_string(sources::parsimagma_dir().join("summary.tsv")).unwrap();
    text.lines()
        .skip(1)
        .map(|line| {
            let (k, v) = line.split_once('\t').unwrap();
            (k.to_string(), v.parse().unwrap())
        })
        .collect()
}

#[test]
fn magma_corpus_against_etp() {
    let table = sources::load_standard(sources::MAGMA_CORPUS).unwrap();
    let oracle = EtpImplications::load(&vendored_matrix()).unwrap();
    let hard_core = parse_pairs(&fs::read_to_string(vendored_hard_core()).unwrap()).unwrap();
    let report = run_implications(&table, &oracle, Some(&hard_core)).unwrap();
    let s = summary();
    let o = report.oracle;

    eprintln!(
        "G-B5: pairs {} emitted {} (known true {}, known false {}) refuted {} (once {}); \
         refuted-but-true {}; hard core {:?}; distinct law columns {}",
        report.scan.pairs(),
        report.scan.emitted_count(),
        o.emitted_true,
        o.emitted_false,
        report.scan.refuted,
        report.scan.refuted_once,
        o.refuted_true,
        report.hard_core,
        report.distinct_law_columns
    );

    assert_eq!(
        o.refuted_true, 0,
        "soundness: a corpus magma refutes a true implication"
    );
    assert_eq!(
        report.scan.emitted_count() + report.scan.refuted,
        report.scan.pairs()
    );
    assert_eq!(o.true_pairs + o.false_pairs, report.scan.pairs());
    assert_eq!(report.objects.len() as u64, s["corpus_rows"]);
    assert_eq!(report.scan.n() as u64, s["laws"]);
    assert_eq!(o.false_pairs, s["graph_false_implications"]);
    assert_eq!(o.refuted_false, s["graph_covered"]);
    let hc = report.hard_core.unwrap();
    assert_eq!(hc.pairs as u64, s["hard_core_pairs"]);
    assert_eq!(hc.refuted as u64, s["hard_core_covered"]);
}
