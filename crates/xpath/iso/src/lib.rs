//! `Normalizer` for the ISO-skeleton XPath dialect.
//!
//! The crate covers the addresses a processor of that skeleton writes.
//! It pairs with a wrapper of the service that answered with them.

mod error;
mod normalizer;
mod prelude;

pub use error::Error;
pub use normalizer::Iso;
