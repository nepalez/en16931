//! The base EN-16931 invoice: the concrete types of the semantic model,
//! implementing the interfaces of the core and its contracts for both bindings.
//!
//! Every field of every type is public, so an invoice is built as a literal
//! and read as plain data. A CIUS profile uses these types as they are,
//! and an extension composes its own types around them.

mod buyer;
mod cii;
mod contact;
mod delivery;
mod invoice;
mod invoice_line;
mod item;
mod payee;
mod prelude;
mod seller;
mod tax_representative;
mod ubl;

pub use buyer::Buyer;
pub use contact::Contact;
pub use delivery::Delivery;
pub use invoice::Invoice;
pub use invoice_line::InvoiceLine;
pub use item::Item;
pub use payee::Payee;
pub use seller::Seller;
pub use tax_representative::TaxRepresentative;
