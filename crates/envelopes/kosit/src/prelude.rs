//! Re-exports of external dependencies shared across the crate.

pub use en16931_core::{Entry, Error as CoreError, Severity, Wrapper};
pub use roxmltree::{Document, Node};
pub use std::fmt::{self, Display, Formatter};
