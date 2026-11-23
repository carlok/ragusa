//! The hypothesis `ALWAYS` selects every row: each generator gives exactly the bounds it gives
//! under a boolean column that is true on every object.

use ragusa_gen::convex_hull::convex_hull;
use ragusa_gen::lp::linear_programming;
use ragusa_gen::ratios::ratios;
use ragusa_gen::{Bound, ALWAYS};
use ragusa_table::{BooleanBlock, NumericBlock, PackedColumns, Table};

fn table() -> Table {
    let n = 7;
    let mut bits = PackedColumns::zeros(n, 1);
    for r in 0..n {
        bits.set(r, 0);
    }
    Table::new(
        "t",
        (0..n).map(|r| r.to_string()).collect(),
        Some(NumericBlock::from_columns(
            vec!["x".into(), "y".into()],
            vec![
                vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
                vec![2.0, 3.0, 3.0, 6.0, 5.0, 9.0, 8.0],
            ],
        )),
        Some(BooleanBlock::new(vec!["p".into()], bits)),
        vec![],
        vec![],
    )
}

fn renamed(bounds: Vec<Bound>) -> Vec<Bound> {
    bounds
        .into_iter()
        .map(|b| Bound {
            hypothesis: ALWAYS.to_string(),
            ..b
        })
        .collect()
}

#[test]
fn always_equals_an_all_true_column() {
    let t = table();
    let f = ["x"];
    let ratio = |h| ratios(&t, "y", &f, h).unwrap();
    let lp = |h| linear_programming(&t, "y", &f, h).unwrap();
    let hull = |h| convex_hull(&t, "y", &f, h).unwrap().bounds;
    assert_eq!(ratio(ALWAYS), renamed(ratio("p")));
    assert_eq!(lp(ALWAYS), renamed(lp("p")));
    assert_eq!(hull(ALWAYS), renamed(hull("p")));
    assert!(!ratio(ALWAYS).is_empty() && !lp(ALWAYS).is_empty() && !hull(ALWAYS).is_empty());
}
