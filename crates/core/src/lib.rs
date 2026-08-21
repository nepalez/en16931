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
mod invalid_document;
mod invoice;
mod location;
mod normalizer;
mod path;
mod prelude;
mod profile;
mod raw_report;
mod report;
mod term;
mod valid_document;
mod values;
mod wrapper;

pub use binding::Binding;
pub use context::{Context, Segment};
pub use document::Document;
pub use document_builder::DocumentBuilder;
pub use error::Error;
pub use format::Dictionary;
pub use invalid_document::InvalidDocument;
pub use invoice::*;
pub use location::{Location, LocationStep, RawNamespace};
pub use normalizer::Normalizer;
pub use path::{Abbreviations, Namespace, Path, Step};
pub use profile::Profile;
pub use raw_report::{Entry, RawReport, Severity};
pub use report::{Problem, Report};
pub use term::Term;
pub use valid_document::ValidDocument;
pub use values::*;
pub use wrapper::Wrapper;
