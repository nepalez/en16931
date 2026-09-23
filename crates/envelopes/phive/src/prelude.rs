//! Re-exports of external dependencies shared across the crate.

pub use en16931_core::{
    Binding, Entry, Error as CoreError, InvoiceKind, Profile, Severity, Target, Wrapper,
};
pub use roxmltree::{Document, Node};
pub use std::fmt::{self, Display, Formatter};
