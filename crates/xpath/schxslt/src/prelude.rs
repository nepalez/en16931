//! Re-exports of external dependencies shared across the crate.

pub use en16931_core::{Error as CoreError, Location, LocationStep, Normalizer, RawNamespace};
pub use std::fmt::{self, Display, Formatter};
pub use std::num::NonZeroUsize;
