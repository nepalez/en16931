//! `Normalizer` for the SchXslt XPath dialect.
//!
//! The crate covers the addresses the SchXslt location function writes.
//! It pairs with a wrapper of the service that answered with them.

mod error;
mod normalizer;
mod prelude;

pub use error::Error;
pub use normalizer::Schxslt;
