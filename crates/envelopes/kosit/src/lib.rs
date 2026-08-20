//! `Wrapper` for the KoSIT validator envelope.
//!
//! The crate covers the report of the KoSIT validation tool.
//! It pairs with a normalizer of the dialect the tool's processor writes.

mod error;
mod prelude;
mod wrapper;

pub use error::Error;
pub use wrapper::Kosit;
