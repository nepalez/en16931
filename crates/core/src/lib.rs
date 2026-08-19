//! Core of the EN-16931 toolkit: the semantic model, the two bindings,
//! the profiles, and the two extension traits (`Wrapper` and `Normalizer`).
//!
//! The crate follows its own semver line.
//! Extension crates depend on it and pin a compatibility range.

mod binding;
mod context;
mod document;
mod document_builder;
mod error;
mod format;
mod invoice;
mod path;
mod prelude;
mod profile;
mod term;
mod values;

pub use binding::Binding;
pub use context::{Context, Segment};
pub use document::Document;
pub use document_builder::DocumentBuilder;
pub use error::Error;
pub use format::Dictionary;
pub use invoice::*;
pub use path::{Abbreviations, Namespace, Path, Step};
pub use profile::Profile;
pub use term::Term;
pub use values::*;
