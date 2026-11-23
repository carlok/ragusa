//! ragusa-oracle: known-results oracles, the novelty side of PROMPT.md section 2.
//!
//! The first oracle is complete: the Equational Theories Project's implication graph over the
//! 4694 magma laws of order at most 4, where every pair is known true or known false.
//!
//! The second is partial: Jabłonowski's 2026 catalogue of inequalities between integer knot
//! invariants, projected onto the columns of a KnotInfo table. A relation it contains is known; a
//! relation it does not contain is not thereby new.

pub mod etp;
pub mod knots;
