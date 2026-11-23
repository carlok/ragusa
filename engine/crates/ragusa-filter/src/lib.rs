//! ragusa-filter: acceptance rules as measured objects.
//!
//! Every rule implements [`Filter`] and returns a [`Partition`] of its input into representative,
//! alias and discarded candidates, never a filtered list, so a kill rate is a return value.
//! [`floor::leave_one_out`] attributes each rule's decisions to objects, and
//! [`chance::baseline`] compares each rule with random acceptance of the same size.

pub mod chance;
mod evaluate;
pub mod filters;
pub mod floor;
mod partition;

pub use evaluate::{EvalError, Evaluated, Verdict, SLACK_TOL};
pub use partition::{Class, Filter, Kind, Partition};
