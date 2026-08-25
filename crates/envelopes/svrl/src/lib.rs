//! `Wrapper` for the plain SVRL envelope.
//!
//! The crate covers a bare SVRL report of a Schematron processor,
//! taken without any service envelope around it.
//! It pairs with a normalizer of the dialect that processor writes.

mod error;
mod prelude;
mod wrapper;

pub use error::Error;
pub use wrapper::Svrl;
