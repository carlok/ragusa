//! A bound's printed text parses back to the same bound.

use ragusa_gen::{Bound, Op, Rational, ALWAYS};

#[test]
fn display_and_parse_round_trip() {
    let r = Rational::new;
    let bounds = [
        Bound::new(
            "alternating",
            "unknotting_number",
            Op::Le,
            [("crossing_number".to_string(), r(1, 2))],
            r(-1, 2),
        ),
        Bound::new(
            ALWAYS,
            "degree",
            Op::Ge,
            [("h1".to_string(), r(2, 1))],
            r(-6, 1),
        ),
        Bound::new("p", "y", Op::Le, [], r(0, 1)),
        Bound::new(
            "q",
            "y",
            Op::Ge,
            [("a".to_string(), r(-3, 7)), ("b".to_string(), r(5, 1))],
            r(0, 1),
        ),
    ];
    for b in bounds {
        let text = b.to_string();
        assert_eq!(text.parse::<Bound>().unwrap(), b, "{text}");
    }
    assert!("(p) → (y < 1)".parse::<Bound>().is_err());
    assert!("p → y <= 1".parse::<Bound>().is_err());
}
