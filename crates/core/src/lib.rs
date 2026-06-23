//! Core of the EN-16931 toolkit: the semantic model, the two bindings,
//! the profiles, and the two extension traits (`Wrapper` and `Normalizer`).
//!
//! The crate follows its own semver line.
//! Extension crates depend on it and pin a compatibility range.

mod error;
mod invoice;
mod path;
mod prelude;
mod values;

pub use error::Error;
pub use invoice::*;
pub use path::{Namespace, Step};
pub use values::*;
