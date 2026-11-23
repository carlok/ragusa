//! Exact rationals, and Python's `Fraction.limit_denominator`, which txgraffiti uses to turn
//! every fitted float into the coefficient a conjecture states.

use std::fmt;
use std::str::FromStr;

/// Magnitude from which an integer is held as the exact `f64` it came from rather than as i128.
const HUGE: f64 = 1_267_650_600_228_229_401_496_703_205_376.0; // 2^100

/// An exact rational in canonical form: either a reduced fraction with a positive denominator,
/// or an integer of magnitude at least 2^100 held as the exact `f64` that produced it.
///
/// The second form exists because txgraffiti rounds ratios such as `target / 1e-49` into
/// integers far beyond i128; every such integer came from a float, so the float is exact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rational(Repr);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Repr {
    Frac { num: i128, den: i128 },
    HugeInt(u64),
}

impl Rational {
    pub const ZERO: Rational = Rational(Repr::Frac { num: 0, den: 1 });

    /// Panics on a zero denominator, or on an integer of magnitude ≥ 2^100 that no `f64` holds
    /// exactly (no generator can produce one).
    pub fn new(num: i128, den: i128) -> Rational {
        assert!(den != 0, "zero denominator");
        let g = gcd(num.unsigned_abs(), den.unsigned_abs()) as i128;
        let sign = if den < 0 { -1 } else { 1 };
        let (num, den) = (sign * num / g, sign * den / g);
        if den == 1 && (num as f64).abs() >= HUGE {
            let x = num as f64;
            assert_eq!(x as i128, num, "integer {num} is not exactly representable");
            return Rational(Repr::HugeInt(x.to_bits()));
        }
        Rational(Repr::Frac { num, den })
    }

    /// `(numerator, denominator)` when both fit in i128.
    pub fn as_ratio(self) -> Option<(i128, i128)> {
        match self.0 {
            Repr::Frac { num, den } => Some((num, den)),
            Repr::HugeInt(_) => None,
        }
    }

    pub fn is_zero(self) -> bool {
        matches!(self.0, Repr::Frac { num: 0, .. })
    }

    pub fn to_f64(self) -> f64 {
        match self.0 {
            Repr::Frac { num, den } => num as f64 / den as f64,
            Repr::HugeInt(bits) => f64::from_bits(bits),
        }
    }
}

/// Written as Python's `str(Fraction)` writes it: `p/q`, or `p` when `q` is 1.
impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Repr::Frac { num, den: 1 } => write!(f, "{num}"),
            Repr::Frac { num, den } => write!(f, "{num}/{den}"),
            // Formatting with an explicit precision prints the exact decimal expansion.
            Repr::HugeInt(bits) => write!(f, "{:.0}", f64::from_bits(bits)),
        }
    }
}

impl FromStr for Rational {
    type Err = String;

    fn from_str(s: &str) -> Result<Rational, String> {
        let bad = || format!("not a fraction: {s:?}");
        match s.split_once('/') {
            Some((p, q)) => {
                let q: i128 = q.parse().map_err(|_| bad())?;
                if q == 0 {
                    return Err(bad());
                }
                Ok(Rational::new(p.parse().map_err(|_| bad())?, q))
            }
            None => match s.parse::<i128>() {
                Ok(n) if (n as f64).abs() < HUGE => Ok(Rational::new(n, 1)),
                _ => {
                    // A huge integer: accept it only if some f64 holds exactly this value.
                    let x: f64 = s.parse().map_err(|_| bad())?;
                    if x.abs() >= HUGE && format!("{x:.0}") == s {
                        Ok(Rational(Repr::HugeInt(x.to_bits())))
                    } else {
                        Err(bad())
                    }
                }
            },
        }
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// The exact value of a finite `x` with |x| in [2^-40, 2^100) as a reduced
/// `(numerator, denominator)`; both fit in i128.
fn exact_ratio(x: f64) -> (i128, i128) {
    let bits = x.to_bits();
    let sign: i128 = if bits >> 63 == 1 { -1 } else { 1 };
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = (bits & ((1u64 << 52) - 1)) as i128;
    let (mantissa, e) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1i128 << 52), exponent - 1075)
    };
    if mantissa == 0 {
        return (0, 1);
    }
    if e >= 0 {
        return (sign * (mantissa << e), 1);
    }
    let shift = (mantissa.trailing_zeros() as i32).min(-e);
    (sign * (mantissa >> shift), 1i128 << (-e - shift))
}

/// Python's `Fraction(x).limit_denominator(max_den)` for a finite `x`, exactly: the closest
/// fraction with denominator at most `max_den`, ties going to the smaller denominator.
///
/// Returns `None` for NaN and infinity, where Python raises. `max_den` must lie in `1..=2^39`;
/// then every |x| < 2^-40 rounds to 0, and every |x| ≥ 2^53 is an integer returned as itself,
/// which keeps the exact arithmetic inside i128. Checked value by value against Python in
/// `tests/g_b2.rs`.
pub fn limit_denominator(x: f64, max_den: i128) -> Option<Rational> {
    assert!((1..=1i128 << 39).contains(&max_den), "max_den out of range");
    if !x.is_finite() {
        return None;
    }
    if x.abs() >= HUGE {
        return Some(Rational(Repr::HugeInt(x.to_bits())));
    }
    if x.abs() < 2f64.powi(-40) {
        return Some(Rational::ZERO);
    }
    let (num, den) = exact_ratio(x);
    if den <= max_den {
        return Some(Rational::new(num, den));
    }

    // Continued-fraction convergents, as in CPython's fractions.py.
    let (mut p0, mut q0, mut p1, mut q1) = (0i128, 1i128, 1i128, 0i128);
    let (mut n, mut d) = (num, den);
    loop {
        let a = n.div_euclid(d);
        // An overflowing q2 is larger than max_den, which ends the loop just the same.
        match a.checked_mul(q1).and_then(|v| v.checked_add(q0)) {
            Some(q2) if q2 <= max_den => {
                (p0, q0, p1, q1) = (p1, q1, p0 + a * p1, q2);
                (n, d) = (d, n - a * d);
            }
            _ => break,
        }
    }
    let k = (max_den - q0).div_euclid(q1);
    // 2·d·(q0 + k·q1) <= den; an overflowing left side exceeds den, so the test is false.
    let closer_to_p1 = (q0 + k * q1)
        .checked_mul(2)
        .and_then(|v| v.checked_mul(d))
        .is_some_and(|lhs| lhs <= den);
    Some(if closer_to_p1 {
        Rational::new(p1, q1)
    } else {
        Rational::new(p0 + k * p1, q0 + k * q1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduces_and_normalizes_sign() {
        assert_eq!(Rational::new(6, -8), Rational::new(-3, 4));
        assert_eq!(Rational::new(0, 5), Rational::ZERO);
        assert_eq!(Rational::new(-3, 4).to_string(), "-3/4");
        assert_eq!(Rational::new(17, 1).to_string(), "17");
        assert_eq!("5/8".parse::<Rational>(), Ok(Rational::new(5, 8)));
    }

    #[test]
    fn exact_ratio_of_simple_floats() {
        assert_eq!(exact_ratio(0.5), (1, 2));
        assert_eq!(exact_ratio(-3.0), (-3, 1));
        assert_eq!(exact_ratio(0.1), (3602879701896397, 36028797018963968));
    }

    #[test]
    fn python_docstring_examples() {
        // Fraction('3.141592653589793').limit_denominator(10) == 22/7, (100) == 311/99;
        // that decimal is exactly f64::consts::PI.
        use std::f64::consts::PI;
        assert_eq!(limit_denominator(PI, 10), Some(Rational::new(22, 7)));
        assert_eq!(limit_denominator(PI, 100), Some(Rational::new(311, 99)));
    }

    #[test]
    fn huge_integers_round_trip_through_python_text() {
        // Seen in txgraffiti's ratio candidates on graph_data.
        let text = "8360502766108716818096799791169600360057550143488";
        let r: Rational = text.parse().unwrap();
        assert_eq!(r.to_string(), text);
        assert_eq!(limit_denominator(r.to_f64(), 1_000_000), Some(r));
        assert!("8360502766108716818096799791169600360057550143489"
            .parse::<Rational>()
            .is_err());
    }
}
