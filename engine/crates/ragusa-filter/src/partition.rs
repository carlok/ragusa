use crate::Evaluated;

/// Which of PROMPT.md section 2's two questions a filter answers, or neither.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Is the candidate true on the table?
    Validity,
    /// Does the candidate add something the other candidates do not?
    Redundancy,
    /// Is the candidate already a known result?
    Novelty,
    /// Does the table give the candidate enough evidence to be worth keeping?
    Support,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Validity => "validity",
            Kind::Redundancy => "redundancy",
            Kind::Novelty => "novelty",
            Kind::Support => "support",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Class {
    Representative,
    /// Says the same thing as candidate `of` (an index into the evaluated candidates).
    Alias {
        of: usize,
    },
    /// Rejected; `by` names the candidate responsible, when there is one.
    Discarded {
        by: Option<usize>,
    },
}

impl Class {
    pub fn is_representative(self) -> bool {
        matches!(self, Class::Representative)
    }

    pub fn label(self) -> &'static str {
        match self {
            Class::Representative => "representative",
            Class::Alias { .. } => "alias",
            Class::Discarded { .. } => "discarded",
        }
    }

    /// The candidate an alias points to or a discard was caused by.
    pub fn related(self) -> Option<usize> {
        match self {
            Class::Representative => None,
            Class::Alias { of } => Some(of),
            Class::Discarded { by } => by,
        }
    }
}

/// The classes of the candidates in `input`, in input order, with the objects that justify each
/// decision (counterexamples, sharp objects, or objects that make another candidate more general).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Partition {
    pub input: Vec<usize>,
    pub classes: Vec<Class>,
    pub witnesses: Vec<Vec<usize>>,
}

impl Partition {
    pub fn new(input: Vec<usize>, classes: Vec<Class>, witnesses: Vec<Vec<usize>>) -> Partition {
        assert_eq!(input.len(), classes.len(), "one class per input candidate");
        assert_eq!(
            input.len(),
            witnesses.len(),
            "one witness list per input candidate"
        );
        Partition {
            input,
            classes,
            witnesses,
        }
    }

    pub fn representatives(&self) -> Vec<usize> {
        self.input
            .iter()
            .zip(&self.classes)
            .filter(|(_, c)| c.is_representative())
            .map(|(&i, _)| i)
            .collect()
    }

    /// `(representative, alias, discarded)`; always sums to the input size.
    pub fn counts(&self) -> (usize, usize, usize) {
        self.classes.iter().fold((0, 0, 0), |(r, a, d), c| match c {
            Class::Representative => (r + 1, a, d),
            Class::Alias { .. } => (r, a + 1, d),
            Class::Discarded { .. } => (r, a, d + 1),
        })
    }
}

/// An acceptance rule. `active[r]` says whether object `r` counts; the reference attribution
/// re-runs a filter with one object switched off, so a filter must depend on the table only
/// through `active`.
pub trait Filter: Sync {
    fn name(&self) -> &'static str;
    fn kind(&self) -> Kind;
    fn partition(&self, ev: &Evaluated, input: &[usize], active: &[bool]) -> Partition;

    /// Per input position, the active objects whose single removal flips the candidate between
    /// representative and not, derived from `full` (this filter's partition of `input`) without
    /// re-running the filter. Must equal [`crate::floor::leave_one_out`] exactly. `None` when the
    /// filter has no such rule or the computation exceeds its budget.
    fn attribution(
        &self,
        _ev: &Evaluated,
        _input: &[usize],
        _active: &[bool],
        _full: &Partition,
    ) -> Option<Vec<Vec<usize>>> {
        None
    }
}
