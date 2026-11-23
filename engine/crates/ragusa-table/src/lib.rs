//! ragusa-table: one table type for conjecturing over objects.
//!
//! A [`Table`] carries an optional dense numeric block (objects × invariants, `f64`, NaN for
//! a missing value) and an optional bit-packed boolean block (objects × predicates). A
//! TxGraffiti snapshot and a parsimagma signature corpus load into the same type, and
//! neither loader special-cases the other.

mod bits;
mod error;
pub mod parsimagma;
mod pin;
pub mod ragusa_csv;
pub mod sources;
mod table;
pub mod txgraffiti;

pub use bits::{BoolColumns, PackedColumns};
pub use error::LoadError;
pub use pin::Pin;
pub use table::{BooleanBlock, NumericBlock, SourceFile, Table};
