//! Core of the EN-16931 toolkit: the semantic model, the two bindings,
//! the profiles, and the two extension traits (`Wrapper` and `Normalizer`).
//!
//! The crate follows its own semver line.
//! Extension crates depend on it and pin a compatibility range.

mod account_number;
mod amount;
mod error;
mod invoice_type;
mod non_empty_string;
mod prelude;
mod vat_category;

pub use account_number::AccountNumber;
pub use amount::Amount;
pub use error::Error;
pub use invoice_type::{DocumentKind, InvoiceType};
pub use non_empty_string::NonEmptyString;
pub use prelude::{CountryCode, Currency, Date, Decimal};
pub use vat_category::VatCategory;
