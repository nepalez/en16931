//! Core of the EN-16931 toolkit: the interfaces of the semantic model, the two bindings,
//! the profiles, and the two extension traits (`Envelope` and `Dialect`).
//!
//! The crate follows its own semver line.
//! Extension crates depend on it and pin a compatibility range.

mod binding;
mod context;
mod deserializable;
mod dialect;
mod document;
mod document_builder;
mod envelope;
mod error;
mod format;
mod invoice;
mod location;
mod namespace;
mod path;
mod prelude;
mod profile;
mod raw_report;
mod report;
mod serializable;
mod target;
mod term;
mod values;

pub use binding::Binding;
pub use context::{Context, Segment};
pub use deserializable::Deserializable;
pub use dialect::Dialect;
pub use document::Document;
pub use document::invalid::InvalidDocument;
pub use document::valid::ValidDocument;
pub use document_builder::DocumentBuilder;
pub use envelope::Envelope;
pub use error::Error;
pub use format::{Cii, Dictionary, Format, Parser, Serializer, Ubl, cii, ubl};
pub use invoice::*;
pub use location::{Location, LocationStep, RawNamespace};
pub use namespace::{Abbreviations, Namespace};
pub use path::{Path, Step};
pub use profile::Profile;
pub use raw_report::{Entry, RawReport, Severity};
pub use report::{Problem, Report};
pub use serializable::Serializable;
pub use target::Target;
pub use term::Term;
pub use values::*;
