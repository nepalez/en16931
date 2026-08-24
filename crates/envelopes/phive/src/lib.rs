//! `Wrapper` for the phive validator envelope.
//!
//! The crate covers the answer of the phive validation service.
//! It also names the rule set that service checks a document against.
//! It pairs with a normalizer of the dialect the service's processor writes.

mod error;
mod prelude;
mod vendor_id;
mod wrapper;

pub use error::Error;
pub use wrapper::Phive;
