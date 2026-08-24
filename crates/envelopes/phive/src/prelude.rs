//! Re-exports of external dependencies shared across the crate.

pub use en16931_core::{
    Binding, DocumentKind, Entry, Error as CoreError, Profile, Severity, Target, Wrapper,
};
pub use roxmltree::{Document, Node};
pub use std::fmt::{self, Display, Formatter};
