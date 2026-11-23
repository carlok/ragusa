//! The floor report on graph_data at PROMPT.md section 1.1's scope: structural invariants every
//! report must satisfy, independent of the numbers.

use ragusa_report::{run, Generator, RunSpec, FORMAT};
use ragusa_table::sources;

#[test]
fn section_1_1_scope_report_is_consistent() {
    let table = sources::load_standard("graph_data").unwrap();
    let mut spec = RunSpec::full(&table, "independence_number").unwrap();
    spec.features.truncate(12);
    spec.hypotheses.truncate(6);
    spec.generators = vec![Generator::ConvexHull, Generator::Ratios];
    spec.chance_draws = 20;
    let report = run(&table, spec).unwrap();

    assert_eq!(report.evaluated.len(), report.origins.len());
    assert_eq!(
        report.per_generator.values().sum::<usize>(),
        report.evaluated.len()
    );
    assert_eq!(report.stages.len(), 4);

    let mut expected_input: Vec<usize> = (0..report.evaluated.len()).collect();
    for stage in &report.stages {
        assert_eq!(
            stage.partition.input, expected_input,
            "{} input",
            stage.filter
        );
        let (rep, alias, disc) = stage.partition.counts();
        assert_eq!(rep + alias + disc, stage.partition.input.len());
        let floor = stage.floor.as_ref().expect("attribution computed");
        assert_eq!(floor.flips_by_object.len(), table.n_objects());
        assert_eq!(floor.fragile.len(), stage.partition.input.len());
        assert_eq!(stage.chance.len(), 4);
        expected_input = stage.partition.representatives();
    }

    let json = report.to_json();
    assert_eq!(json["format"], FORMAT);
    assert_eq!(
        json["survivors"].as_u64(),
        Some(report.survivors().len() as u64)
    );
    assert_eq!(
        json["pin"]["content_sha256"].as_str(),
        Some(report.pin.content_sha256.as_str())
    );
    let csv = report.candidates_csv();
    assert_eq!(csv.lines().count(), report.evaluated.len() + 1);
    assert!(report.to_markdown().starts_with("# ragusa floor report"));
}
