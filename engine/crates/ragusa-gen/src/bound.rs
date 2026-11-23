use std::fmt;

use crate::rational::Rational;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    Le,
    Ge,
}

impl Op {
    pub fn symbol(self) -> &'static str {
        match self {
            Op::Le => "<=",
            Op::Ge => ">=",
        }
    }
}

/// The hypothesis that holds on every object, for statements not conditioned on a boolean column
/// (a table may not have a boolean column of this name).
pub const ALWAYS: &str = "TRUE";

/// `hypothesis → target op Σ coef·feature + constant`, in canonical form: zero terms dropped,
/// terms sorted by feature name. Two bounds state the same conjecture iff they are equal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bound {
    pub hypothesis: String,
    pub target: String,
    pub op: Op,
    pub terms: Vec<(String, Rational)>,
    pub constant: Rational,
}

impl Bound {
    pub fn new(
        hypothesis: impl Into<String>,
        target: impl Into<String>,
        op: Op,
        terms: impl IntoIterator<Item = (String, Rational)>,
        constant: Rational,
    ) -> Bound {
        let mut terms: Vec<(String, Rational)> =
            terms.into_iter().filter(|(_, c)| !c.is_zero()).collect();
        terms.sort_by(|a, b| a.0.cmp(&b.0));
        assert!(
            terms.windows(2).all(|w| w[0].0 != w[1].0),
            "a feature appears twice in one bound"
        );
        Bound {
            hypothesis: hypothesis.into(),
            target: target.into(),
            op,
            terms,
            constant,
        }
    }
}

/// Parses exactly the text [`fmt::Display`] writes: `(hypothesis) → (target op c·feature + … + k)`.
impl std::str::FromStr for Bound {
    type Err = String;

    fn from_str(s: &str) -> Result<Bound, String> {
        let bad = || format!("not a bound: {s:?}");
        let (hypothesis, rest) = s
            .strip_prefix('(')
            .and_then(|t| t.split_once(") → ("))
            .ok_or_else(bad)?;
        let body = rest.strip_suffix(')').ok_or_else(bad)?;
        let (target, rest) = body.split_once(' ').ok_or_else(bad)?;
        let (op, rhs) = rest.split_once(' ').ok_or_else(bad)?;
        let op = match op {
            "<=" => Op::Le,
            ">=" => Op::Ge,
            _ => return Err(bad()),
        };
        let mut terms = Vec::new();
        let mut constant = Rational::ZERO;
        for part in rhs.split(" + ") {
            match part.split_once('·') {
                Some((c, feature)) => terms.push((feature.to_string(), c.parse::<Rational>()?)),
                None => constant = part.parse::<Rational>()?,
            }
        }
        Ok(Bound::new(hypothesis, target, op, terms, constant))
    }
}

impl fmt::Display for Bound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rhs: Vec<String> = self
            .terms
            .iter()
            .map(|(name, c)| format!("{c}·{name}"))
            .collect();
        if !self.constant.is_zero() || rhs.is_empty() {
            rhs.push(self.constant.to_string());
        }
        write!(
            f,
            "({}) → ({} {} {})",
            self.hypothesis,
            self.target,
            self.op.symbol(),
            rhs.join(" + ")
        )
    }
}
