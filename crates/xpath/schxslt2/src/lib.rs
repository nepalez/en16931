//! `Normalizer` for the SchXslt2 XPath dialect.
//!
//! The crate covers the addresses the second SchXslt version writes
//! through the standard `fn:path` function.
//! It pairs with a wrapper of the service that answered with them.

mod error;
mod normalizer;
mod prelude;

pub use error::Error;
pub use normalizer::Schxslt2;
