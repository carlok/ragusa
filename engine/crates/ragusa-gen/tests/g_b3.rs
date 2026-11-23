//! Gate G-B3: a published result reappears from the data.
//!
//! The TxGraffiti paper (arXiv:2409.19379) cites Caro et al.: for r-regular graphs the
//! independence number is at most the matching number, α ≤ μ. On graph_data the ratio generator
//! must state exactly `(regular) → (independence_number <= 1·matching_number)`, and must not state
//! it without the hypothesis, because many non-regular graphs in the table have α > μ.

use ragusa_gen::ratios::ratios;
use ragusa_gen::{Bound, Op, Rational};
use ragusa_table::sources;

fn upper_ratio(hypothesis: &str) -> Bound {
    let table = sources::load_standard("graph_data").unwrap();
    ratios(
        &table,
        "independence_number",
        &["matching_number"],
        hypothesis,
    )
    .unwrap()
    .into_iter()
    .find(|b| b.op == Op::Le)
    .expect("an upper ratio bound")
}

#[test]
fn alpha_at_most_mu_on_regular_graphs() {
    let want = Bound::new(
        "regular",
        "independence_number",
        Op::Le,
        [("matching_number".to_string(), Rational::new(1, 1))],
        Rational::ZERO,
    );
    assert_eq!(upper_ratio("regular"), want);
}

#[test]
fn alpha_at_most_mu_needs_the_hypothesis() {
    // `connected` holds on every graph in graph_data, so this is the unrestricted ratio.
    let bound = upper_ratio("connected");
    let coefficient = bound.terms[0].1.to_f64();
    assert!(
        coefficient > 1.0,
        "expected α/μ > 1 on some graph, got {bound}"
    );
}
