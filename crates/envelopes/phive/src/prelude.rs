//! Re-exports of external dependencies shared across the crate.

pub use en16931_core::{
    Binding, Entry, Envelope, Error as CoreError, InvoiceKind, Profile, Severity, Target,
};
pub use roxmltree::{Document, Node};
pub use std::fmt::{self, Display, Formatter};
