//! G-B8: the attribution every stage reports equals attribution by re-running the filter once per
//! object, on the full graph_data run (every generator, every feature and hypothesis), for each
//! redundancy rule. The reported attribution is derived from the partition for every rule
//! (greedy_touch reruns only the objects whose removal can change a pick).
//! Random small tables with intervals and unknowns are covered in `ragusa_filter::floor::tests`.

use ragusa_filter::filters::{
    Generality, GreedyTouch, MaximalTouch, SharpCoverage, TableDuplicates, Validity,
};
use ragusa_filter::floor::leave_one_out;
use ragusa_filter::Filter;
use ragusa_report::{run, Redundancy, RunSpec};
use ragusa_table::sources;

#[test]
fn attribution_equals_reruns_on_graph_data() {
    let table = sources::load_standard("graph_data").unwrap();
    let active = vec![true; table.n_objects()];
    let rules: [(Redundancy, &dyn Filter); 3] = [
        (Redundancy::SharpCoverage, &SharpCoverage),
        (Redundancy::MaximalTouch, &MaximalTouch),
        (Redundancy::GreedyTouch, &GreedyTouch),
    ];
    for (rule, redundancy) in rules {
        let mut spec = RunSpec::full(&table, "independence_number").unwrap();
        spec.chance_draws = 0;
        spec.redundancy = rule;
        let report = run(&table, spec).unwrap();
        let filters: [&dyn Filter; 4] = [&Validity, &TableDuplicates, redundancy, &Generality];
        assert_eq!(report.stages.len(), filters.len());
        for (stage, filter) in report.stages.iter().zip(filters) {
            assert_eq!(stage.filter, filter.name());
            let rerun = leave_one_out(
                filter,
                &report.evaluated,
                &stage.partition.input,
                &active,
                &stage.partition,
            );
            assert_eq!(
                stage.floor.as_ref(),
                Some(&rerun),
                "{} with {}",
                stage.filter,
                rule.name()
            );
        }
    }
}
