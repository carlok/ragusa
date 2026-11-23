//! G-B9: Jabłonowski's catalogue, projected onto KnotInfo columns, is not refuted by the KnotInfo
//! tables. Needs the git-ignored exports (`make frontier-gates`), so it is ignored by default:
//!
//!     cargo test --release -p ragusa-report --test g_b9 -- --ignored --nocapture
//!
//! Prints one line per catalogue bound: refuted / confirmed / undecided knots, derivation.

use ragusa_oracle::knots::Catalogue;
use ragusa_report::knots::check_catalogue;
use ragusa_table::sources;

#[test]
#[ignore = "needs the KnotInfo exports under external/"]
fn catalogue_holds_on_knotinfo() {
    let catalogue = Catalogue::standard().unwrap();
    let mut failures = Vec::new();
    for name in ["knotinfo_le12", "knotinfo_le13"] {
        let table = sources::load_standard(name).unwrap();
        let check = check_catalogue(&table, &catalogue).unwrap();
        println!(
            "{name}: {} non-trivial knots, {} catalogue bounds, {} skipped (absolute larger side), {} refuted",
            check.non_trivial_objects,
            check.bounds.len(),
            check.skipped_absolute,
            check.refuted()
        );
        for (c, result) in check.bounds.iter().zip(&check.checks) {
            let status = match result {
                None => "column missing".to_string(),
                Some(r) => format!(
                    "refuted {:>5}  confirmed {:>5}  undecided {:>5}",
                    r.refuted_on.len(),
                    r.confirmed,
                    r.undecided
                ),
            };
            println!(
                "  {status}  {}{}  [{}]",
                c.bound,
                if c.likely { "  (likely mapping)" } else { "" },
                c.derivation
            );
            if let Some(r) = result.as_ref().filter(|r| !r.refuted_on.is_empty()) {
                let examples: Vec<&str> = r
                    .refuted_on
                    .iter()
                    .take(5)
                    .map(|&k| table.objects()[k].as_str())
                    .collect();
                println!("      refuted on {}", examples.join(", "));
                failures.push(format!("{name}: {}", c.bound));
            }
        }
    }
    assert!(failures.is_empty(), "refuted: {failures:#?}");
}
